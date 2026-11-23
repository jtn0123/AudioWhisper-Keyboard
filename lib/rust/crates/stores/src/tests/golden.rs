//! The committed golden files, which pin the format: each must decode to the value built here,
//! and that value must encode to its bytes exactly. `STORES_RECORD=1` rewrites them, for a new
//! version only; a version already shipped never changes.
//!
//! Failures name the file and never print the values, which stand for the user's clipboard and
//! words.

use std::path::PathBuf;

use crate::clips::{Clip, Clips, FileInfo, IMAGE, TEXT, VIDEO};
use crate::store::{self, Payload};
use crate::words::{Word, Words};

/// UTF-16 of `text`.
pub fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

/// Panics, naming `what`, unless `actual` is `expected`; never prints either.
pub fn check<T: PartialEq>(actual: &T, expected: &T, what: &str) {
    assert!(actual == expected, "{what} differs");
}

/// A clip as the history keeps one.
pub fn clip(id: i64, kind: i32, created: i64, text: Option<&str>) -> Clip {
    Clip {
        id,
        kind,
        created,
        text: text.map(units),
        mime_types: units("text/plain"),
        ..Clip::default()
    }
}

/// A clipboard with every kind of clip and field, including a lone surrogate.
pub fn sample_clips() -> Clips {
    let mut lone = units("synthetic ");
    lone.push(0xd800);
    let image = Clip {
        uri: Some(units(
            "content://synthetic.provider.clipboard/clips/images/101",
        )),
        mime_types: units("image/png,image/*"),
        pinned: true,
        ..clip(3, IMAGE, 1_700_000_000_300, None)
    };
    let video = Clip {
        uri: Some(units(
            "content://synthetic.provider.clipboard/clips/videos/102",
        )),
        mime_types: units("video/mp4"),
        remote: true,
        ..clip(7, VIDEO, 1_700_000_000_700, Some("caption"))
    };
    let sensitive = Clip {
        sensitive: true,
        ..clip(9, TEXT, 1_700_000_000_900, Some("synthetic secret"))
    };
    Clips {
        next_id: 12,
        imported: vec![0x0123_4567_89ab_cdef],
        items: vec![
            clip(1, TEXT, 1_700_000_000_100, Some("synthetic one")),
            Clip {
                text: Some(lone),
                ..clip(2, TEXT, 1_700_000_000_200, None)
            },
            image,
            video,
            sensitive,
            clip(10, TEXT, 1_700_000_000_900, Some("")),
            Clip {
                mime_types: Vec::new(),
                ..clip(11, TEXT, -5, None)
            },
        ],
        files: vec![
            FileInfo {
                id: 101,
                name: units("Image"),
                size: 4096,
                orientation: 90,
                mime_types: units("image/png,image/*"),
            },
            FileInfo {
                id: 102,
                name: units("clip.mp4"),
                size: 0,
                orientation: 0,
                mime_types: units("video/mp4"),
            },
        ],
    }
}

/// A dictionary with every field, set and unset.
pub fn sample_words() -> Words {
    let word = |id, text: &str, frequency, locale: Option<&str>, shortcut: Option<&str>| Word {
        id,
        word: units(text),
        frequency,
        locale: locale.map(units),
        shortcut: shortcut.map(units),
    };
    Words {
        next_id: 8,
        imported: vec![42],
        words: vec![
            word(1, "synthetica", 250, None, None),
            word(2, "synthétique", 1, Some("fr"), Some("syn")),
            word(5, "Synthetic", 255, Some("en_US"), None),
            word(7, "", 0, Some(""), Some("")),
        ],
    }
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn golden<T: Payload>(name: &str, value: &T) {
    let path = fixture(name);
    let bytes = store::encode(value);
    if std::env::var_os("STORES_RECORD").is_some() {
        std::fs::write(&path, &bytes).expect("fixture written");
    }
    let committed = std::fs::read(&path).expect("the golden file is committed");
    check(&bytes, &committed, name);
    let decoded = store::decode::<T>(&committed);
    assert!(decoded.is_ok(), "{name} does not decode");
    check(&decoded.ok(), &Some(value.clone()), name);
}

#[test]
fn the_clipboard_golden_file_is_the_sample() {
    golden("clips-v1.store", &sample_clips());
}

#[test]
fn the_dictionary_golden_file_is_the_sample() {
    golden("words-v1.store", &sample_words());
}

#[test]
fn the_empty_stores_golden_files_are_empty() {
    golden("clips-v1-empty.store", &Clips::default());
    golden("words-v1-empty.store", &Words::default());
}

#[test]
fn an_empty_clipboard_is_these_bytes() {
    let mut expected = b"AWSTCLIP".to_vec();
    expected.extend_from_slice(&1u32.to_le_bytes());
    expected.extend_from_slice(&20u64.to_le_bytes());
    expected.extend_from_slice(&[0; 20]); // next id, then three empty counts
    let crc = crate::codec::crc32(&expected);
    expected.extend_from_slice(&crc.to_le_bytes());
    check(
        &store::encode(&Clips::default()),
        &expected,
        "empty clipboard",
    );
}
