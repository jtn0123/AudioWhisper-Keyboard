//! The frame and its pieces: checksums, and every way a file can be damaged.

use crate::clips::Clips;
use crate::codec::{self, Error, Reader, Writer};
use crate::store::{self, Payload};
use crate::words::Words;

use super::golden::{check, sample_clips, sample_words};

#[test]
fn checksums_are_the_standard_ones() {
    assert_eq!(codec::crc32(b"123456789"), 0xcbf4_3926);
    assert_eq!(codec::crc32(b""), 0);
    assert_eq!(codec::fnv64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(codec::fnv64(b"a"), 0xaf63_dc4c_8601_ec8c);
}

#[test]
fn every_cut_is_refused() {
    let bytes = store::encode(&sample_clips());
    for len in 0..bytes.len() {
        assert!(
            store::decode::<Clips>(&bytes[..len]).is_err(),
            "cut at {len}"
        );
    }
}

#[test]
fn every_flipped_bit_is_refused() {
    let bytes = store::encode(&sample_words());
    for at in 0..bytes.len() * 8 {
        let mut flipped = bytes.clone();
        flipped[at / 8] ^= 1 << (at % 8);
        assert!(store::decode::<Words>(&flipped).is_err(), "bit {at}");
    }
}

#[test]
fn another_kind_is_refused() {
    let bytes = store::encode(&sample_words());
    assert_eq!(store::decode::<Clips>(&bytes).err(), Some(Error::Kind));
    assert_eq!(
        store::decode::<Clips>(b"not a store at all").err(),
        Some(Error::Kind)
    );
}

#[test]
fn a_newer_version_is_known_as_one() {
    let bytes = codec::frame(Clips::KIND, Clips::VERSION + 1, b"anything");
    assert_eq!(store::decode::<Clips>(&bytes).err(), Some(Error::Newer(2)));
}

#[test]
fn a_wrong_length_is_refused_even_with_a_good_checksum() {
    let mut bytes = store::encode(&Clips::default());
    bytes[12] = 21;
    let body = bytes.len() - 4;
    let crc = codec::crc32(&bytes[..body]);
    bytes[body..].copy_from_slice(&crc.to_le_bytes());
    assert_eq!(
        store::decode::<Clips>(&bytes).err(),
        Some(Error::Damaged("length"))
    );
}

/// A payload framed with a good checksum, to test what the payload's own checks refuse.
fn framed<T: Payload>(write: impl FnOnce(&mut Writer)) -> Result<T, Error> {
    let mut writer = Writer::default();
    write(&mut writer);
    store::decode::<T>(&codec::frame(T::KIND, T::VERSION, &writer.into_bytes()))
}

#[test]
fn payloads_that_break_their_rules_are_refused() {
    let trailing = framed::<Clips>(|w| {
        w.i64(0);
        (0..3).for_each(|_| w.count(0));
        w.u8(0);
    });
    assert_eq!(trailing.err(), Some(Error::Damaged("trailing bytes")));
    let huge = framed::<Clips>(|w| {
        w.i64(0);
        w.u32(u32::MAX);
    });
    assert_eq!(huge.err(), Some(Error::Damaged("count")));
    let unordered = framed::<Words>(|w| {
        w.i64(9);
        w.count(0);
        w.count(2);
        for id in [5, 3] {
            w.i64(id);
            w.i32(0);
            w.text(&[]);
            w.optional_text(None);
            w.optional_text(None);
        }
    });
    assert_eq!(unordered.err(), Some(Error::Damaged("word ids")));
    let behind = framed::<Words>(|w| {
        w.i64(3);
        w.count(0);
        w.count(1);
        w.i64(3);
        w.i32(0);
        w.text(&[]);
        w.optional_text(None);
        w.optional_text(None);
    });
    assert_eq!(behind.err(), Some(Error::Damaged("word ids")));
}

#[test]
fn clip_ids_must_rise_and_stay_below_the_next_id() {
    let clips = |next_id: i64, ids: &[i64]| {
        framed::<Clips>(|w| {
            w.i64(next_id);
            w.count(0);
            w.count(ids.len());
            for &id in ids {
                w.i64(id);
                w.i32(1);
                w.i64(0);
                w.u8(0);
                w.optional_text(None);
                w.optional_text(None);
                w.text(&[]);
            }
            w.count(0);
        })
    };
    assert!(clips(9, &[3, 5]).is_ok());
    for (next_id, ids) in [(9, [5, 3]), (9, [3, 3]), (5, [3, 5])] {
        let refused = clips(next_id, &ids);
        assert_eq!(
            refused.err(),
            Some(Error::Damaged("clip ids")),
            "{ids:?} before {next_id}"
        );
    }
}

#[test]
fn unknown_flags_and_tags_are_refused() {
    let flagged = framed::<Clips>(|w| {
        w.i64(2);
        w.count(0);
        w.count(1);
        w.i64(1);
        w.i32(1);
        w.i64(0);
        w.u8(8);
        w.optional_text(None);
        w.optional_text(None);
        w.text(&[]);
        w.count(0);
    });
    assert_eq!(flagged.err(), Some(Error::Damaged("clip flags")));
    let mut reader = Reader::new(&[2]);
    assert_eq!(
        reader.optional_text().err(),
        Some(Error::Damaged("optional text"))
    );
}

#[test]
fn text_keeps_every_unit() {
    let units: Vec<u16> = vec![0, 0xd800, 0xdfff, 0xffff, 0x41];
    let mut writer = Writer::default();
    writer.text(&units);
    writer.optional_text(Some(&[]));
    writer.optional_text(None);
    let bytes = writer.into_bytes();
    let mut reader = Reader::new(&bytes);
    check(&reader.text().ok(), &Some(units), "text");
    check(
        &reader.optional_text().ok(),
        &Some(Some(Vec::new())),
        "empty text",
    );
    check(&reader.optional_text().ok(), &Some(None), "no text");
    assert!(reader.finish().is_ok());
}
