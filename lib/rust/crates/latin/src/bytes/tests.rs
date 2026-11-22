use super::*;
use crate::golden::{same, tape};

/// SplitMix64, for seeded random buffers.
struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next() % bound
    }

    /// Bytes weighted towards the ones code points are made of: the terminator, small three-byte
    /// starts and Latin-1.
    fn buffer(&mut self, length: usize) -> Vec<u8> {
        (0..length)
            .map(|_| match self.below(10) {
                0 => 0x1F,
                1 | 2 => self.below(0x20) as u8,
                _ => self.below(0x100) as u8,
            })
            .collect()
    }
}

/// A read from `pos` that moves it: what it read, and where it left `pos`.
fn advance<T>(pos: i32, read: impl FnOnce(&mut i32) -> T) -> (T, i32) {
    let mut pos = pos;
    let value = read(&mut pos);
    (value, pos)
}

#[test]
fn random_buffers_match_the_goldens() {
    #[cfg(feature = "oracle")]
    use latin_oracle::hooks::bytes as cpp;
    let mut tape = tape("bytes.random_buffers");
    let mut random = Random(20_261_109);
    let table: Vec<i32> = (0..224).map(|it| 0x400 + it * 3).collect();
    for round in 0..20_000 {
        let length = 1 + random.below(40) as usize;
        let buffer = random.buffer(length);
        let pos = random.below(buffer.len() as u64 + 1) as i32;
        for size in 0..=5 {
            same!(
                tape,
                read_uint(&buffer, size, pos),
                cpp::read_uint(&buffer, size, pos)
            );
        }
        same!(
            tape,
            advance(pos, |at| read_s24_advance(&buffer, at)),
            advance(pos, |at| cpp::read_s24_advance(&buffer, at))
        );
        let with_table = (round % 3 == 0).then_some(table.as_slice());
        same!(
            tape,
            advance(pos, |at| read_code_point_advance(&buffer, with_table, at)),
            advance(pos, |at| cpp::read_code_point_advance(
                &buffer, with_table, at
            ))
        );
        let max_length = random.below(60) as usize;
        same!(
            tape,
            advance(pos, |at| {
                let mut out = vec![0; max_length];
                let length = read_string_advance(&buffer, max_length, with_table, &mut out, at);
                out.truncate(length);
                out
            }),
            advance(pos, |at| cpp::read_string_advance(
                &buffer, max_length, with_table, at
            )),
            "{buffer:?} at {pos}"
        );
        same!(
            tape,
            advance(pos, |at| advance_past_string(&buffer, max_length, at)
                as i32),
            advance(pos, |at| cpp::advance_past_string(&buffer, max_length, at))
        );
    }
    tape.finish();
}

#[test]
fn format_versions_match_the_goldens() {
    #[cfg(feature = "oracle")]
    use latin_oracle::hooks::bytes as cpp;
    let mut tape = tape("bytes.format_versions");
    let mut buffer = vec![0x9B, 0xC1, 0x3A, 0xFE, 0, 0, 0, 0, 0, 0, 0, 0];
    for version in 0..=0xFFFF_u16 {
        buffer[4..6].copy_from_slice(&version.to_be_bytes());
        same!(
            tape,
            detect_format_version(&buffer) as i32,
            cpp::detect_format_version(&buffer)
        );
    }
    for broken in [
        &buffer[..11],
        &[0x9B, 0xC1, 0x3A, 0xFF, 0, 202, 0, 0, 0, 0, 0, 0][..],
        &[],
    ] {
        same!(
            tape,
            detect_format_version(broken) as i32,
            cpp::detect_format_version(broken)
        );
    }
    for (_, path) in latin_golden::dictionaries() {
        let bytes = std::fs::read(path).unwrap();
        assert_eq!(detect_format_version(&bytes), FormatVersion::Version202);
        same!(
            tape,
            detect_format_version(&bytes) as i32,
            cpp::detect_format_version(&bytes)
        );
    }
    tape.finish();
}

#[test]
fn reads_past_the_end_as_zeros() {
    assert_eq!(read_u32(&[1, 2], 0), 0x0102_0000);
    assert_eq!(read_u8(&[1], -1), 0);
    assert_eq!(read_u8(&[1], i32::MIN), 0);
}
