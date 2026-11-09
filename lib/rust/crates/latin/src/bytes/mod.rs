//! Reading a dictionary's bytes (`dictionary/utils/byte_array_utils` and `format_utils`):
//! big-endian integers, code points in the dictionary's own encoding, and the format version.
//!
//! Positions are `i32`, as the C++ has them, because the trie code does arithmetic on them and
//! uses `NOT_A_DICT_POS` (`i32::MIN`) as a position. The C++ read without bounds checks; here a
//! byte outside the buffer reads as 0, which only a broken dictionary reaches.

use crate::defines::NOT_A_CODE_POINT;

const MINIMUM_ONE_BYTE_CHARACTER_VALUE: u8 = 0x20;
const CHARACTER_ARRAY_TERMINATOR: u8 = 0x1F;

/// `FormatUtils::MAGIC_NUMBER`, the first four bytes of every dictionary.
pub const MAGIC_NUMBER: u32 = 0x9BC1_3AFE;
/// Magic number (4 bytes), version (2), flags (2) and header size (4).
const DICTIONARY_MINIMUM_SIZE: usize = 12;

/// `FormatUtils::FORMAT_VERSION`, with the C++'s values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatVersion {
    Version2 = 2,
    Version201 = 201,
    Version202 = 202,
    Version4OnlyForTesting = 399,
    Version402 = 402,
    Version403 = 403,
    Unknown = -1,
}

/// The byte at `pos`, or 0 outside the buffer.
pub fn read_u8(buffer: &[u8], pos: i32) -> u8 {
    usize::try_from(pos)
        .ok()
        .and_then(|at| buffer.get(at))
        .copied()
        .unwrap_or(0)
}

/// `ByteArrayUtils::readUint16`.
pub fn read_u16(buffer: &[u8], pos: i32) -> u16 {
    (u16::from(read_u8(buffer, pos)) << 8) ^ u16::from(read_u8(buffer, pos + 1))
}

/// `ByteArrayUtils::readUint24`.
pub fn read_u24(buffer: &[u8], pos: i32) -> u32 {
    (u32::from(read_u8(buffer, pos)) << 16)
        ^ (u32::from(read_u8(buffer, pos + 1)) << 8)
        ^ u32::from(read_u8(buffer, pos + 2))
}

/// `ByteArrayUtils::readUint32`.
pub fn read_u32(buffer: &[u8], pos: i32) -> u32 {
    (u32::from(read_u8(buffer, pos)) << 24)
        ^ (u32::from(read_u8(buffer, pos + 1)) << 16)
        ^ (u32::from(read_u8(buffer, pos + 2)) << 8)
        ^ u32::from(read_u8(buffer, pos + 3))
}

/// `ByteArrayUtils::readUint`: a `size`-byte integer, 1 to 4; 0 for any other size.
pub fn read_uint(buffer: &[u8], size: i32, pos: i32) -> u32 {
    match size {
        1 => u32::from(read_u8(buffer, pos)),
        2 => u32::from(read_u16(buffer, pos)),
        3 => read_u24(buffer, pos),
        4 => read_u32(buffer, pos),
        _ => 0,
    }
}

/// `readUint8AndAdvancePosition`.
pub fn read_u8_advance(buffer: &[u8], pos: &mut i32) -> u8 {
    let value = read_u8(buffer, *pos);
    *pos += 1;
    value
}

/// `readUint16AndAdvancePosition`.
pub fn read_u16_advance(buffer: &[u8], pos: &mut i32) -> u16 {
    let value = read_u16(buffer, *pos);
    *pos += 2;
    value
}

/// `readUint24AndAdvancePosition`.
pub fn read_u24_advance(buffer: &[u8], pos: &mut i32) -> u32 {
    let value = read_u24(buffer, *pos);
    *pos += 3;
    value
}

/// `readUint32AndAdvancePosition`.
pub fn read_u32_advance(buffer: &[u8], pos: &mut i32) -> u32 {
    let value = read_u32(buffer, *pos);
    *pos += 4;
    value
}

/// `readSint24AndAdvancePosition`: a sign bit and 23 bits of magnitude.
pub fn read_s24_advance(buffer: &[u8], pos: &mut i32) -> i32 {
    let first = read_u8(buffer, *pos);
    if first < 0x80 {
        read_u24_advance(buffer, pos) as i32
    } else {
        *pos += 1;
        -(((i32::from(first) & 0x7F) << 16) ^ i32::from(read_u16_advance(buffer, pos)))
    }
}

/// `readCodePoint`: the code point at `pos`, without a code point table.
pub fn read_code_point(buffer: &[u8], pos: i32) -> i32 {
    let mut at = pos;
    read_code_point_advance(buffer, None, &mut at)
}

/// `readCodePointAndAdvancePosition`. One byte from 0x20 is a character (Latin-1, or an index into
/// the dictionary's `table` of frequent characters); 0x1F ends a string ([`NOT_A_CODE_POINT`]);
/// anything lower starts a three-byte code point. An index past the table's end reads as
/// [`NOT_A_CODE_POINT`], where the C++ read past the array.
pub fn read_code_point_advance(buffer: &[u8], table: Option<&[i32]>, pos: &mut i32) -> i32 {
    let first = read_u8(buffer, *pos);
    if first < MINIMUM_ONE_BYTE_CHARACTER_VALUE {
        if first == CHARACTER_ARRAY_TERMINATOR {
            *pos += 1;
            NOT_A_CODE_POINT
        } else {
            read_u24_advance(buffer, pos) as i32
        }
    } else {
        *pos += 1;
        match table {
            Some(table) => table
                .get(usize::from(first - MINIMUM_ONE_BYTE_CHARACTER_VALUE))
                .copied()
                .unwrap_or(NOT_A_CODE_POINT),
            None => i32::from(first),
        }
    }
}

/// `readStringAndAdvancePosition`: code points into `out` until the terminator, at most
/// `max_length` (and at most `out.len()`) of them; returns how many. As in the C++, the code point
/// after the last one kept is read (and the position moved past it) even when it isn't kept.
pub fn read_string_advance(
    buffer: &[u8],
    max_length: usize,
    table: Option<&[i32]>,
    out: &mut [i32],
    pos: &mut i32,
) -> usize {
    let max_length = max_length.min(out.len());
    let mut length = 0;
    let mut code_point = read_code_point_advance(buffer, table, pos);
    while code_point != NOT_A_CODE_POINT && length < max_length {
        out[length] = code_point;
        length += 1;
        code_point = read_code_point_advance(buffer, table, pos);
    }
    length
}

/// `advancePositionToBehindString`: skips a string of at most `max_length` code points and returns
/// its length.
pub fn advance_past_string(buffer: &[u8], max_length: usize, pos: &mut i32) -> usize {
    let mut length = 0;
    let mut code_point = read_code_point_advance(buffer, None, pos);
    while code_point != NOT_A_CODE_POINT && length < max_length {
        code_point = read_code_point_advance(buffer, None, pos);
        length += 1;
    }
    length
}

/// `FormatUtils::getFormatVersion`: the versions this code knows. 2 and 201 are too old to read.
pub fn format_version(version: i32) -> FormatVersion {
    match version {
        202 => FormatVersion::Version202,
        399 => FormatVersion::Version4OnlyForTesting,
        402 => FormatVersion::Version402,
        403 => FormatVersion::Version403,
        _ => FormatVersion::Unknown,
    }
}

/// `FormatUtils::detectFormatVersion`: the version a dictionary's first bytes declare.
pub fn detect_format_version(buffer: &[u8]) -> FormatVersion {
    if buffer.len() < DICTIONARY_MINIMUM_SIZE || read_u32(buffer, 0) != MAGIC_NUMBER {
        return FormatVersion::Unknown;
    }
    format_version(i32::from(read_u16(buffer, 4)))
}

#[cfg(test)]
mod tests;
