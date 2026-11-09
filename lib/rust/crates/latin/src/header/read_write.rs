//! `HeaderReadWriteUtils` (`dictionary/header/header_read_write_utils`): the header's fixed fields,
//! its attributes (`key=value` strings after them) and the readers for their values. The C++
//! writers were deleted and are not ported.

use std::collections::BTreeMap;

use crate::bytes::{read_string_advance, read_u16, read_u32};

/// `DictionaryHeaderStructurePolicy::AttributeMap`. Keys and values are code points. A `BTreeMap`
/// iterates in the order of the C++ `std::map`: both compare vectors element by element.
pub type AttributeMap = BTreeMap<Vec<i32>, Vec<i32>>;

/// `HeaderReadWriteUtils::MAX_ATTRIBUTE_KEY_LENGTH`.
const MAX_ATTRIBUTE_KEY_LENGTH: usize = 256;
/// `HeaderReadWriteUtils::MAX_ATTRIBUTE_VALUE_LENGTH`.
const MAX_ATTRIBUTE_VALUE_LENGTH: usize = 2048;
/// The magic number (4 bytes), dictionary version (2) and flags (2) come before the header size.
const HEADER_SIZE_POSITION: i32 = 8;
/// The magic number (4 bytes) and dictionary version (2) come before the flags.
const HEADER_FLAGS_POSITION: i32 = 6;
/// `HeaderReadWriteUtils::getHeaderOptionsPosition`: the attributes start after the fixed fields,
/// magic number (4), version (2), flags (2) and header size (4).
const HEADER_OPTIONS_POSITION: i32 = 12;
/// `HeaderReadWriteUtils::CODE_POINT_TABLE_KEY`.
const CODE_POINT_TABLE_KEY: &str = "codePointTable";

/// `HeaderReadWriteUtils::getHeaderSize`: the header size field as the C++'s `int`, so a size of
/// 2^31 or more is negative.
pub fn get_header_size(buffer: &[u8]) -> i32 {
    read_u32(buffer, HEADER_SIZE_POSITION) as i32
}

/// `HeaderReadWriteUtils::getFlags`.
pub fn get_flags(buffer: &[u8]) -> u16 {
    read_u16(buffer, HEADER_FLAGS_POSITION)
}

/// `HeaderReadWriteUtils::fetchAllHeaderAttributes`: reads the attributes, each a key then a value,
/// from the options position up to the header size. A key that is already in the map keeps its
/// first value, as `std::map::insert` does.
///
/// Where the C++ reads past the buffer (a header size larger than the file), this stops at the
/// buffer's end instead. The C++ reads whatever lies beyond the file, which is undefined; a
/// buffer that ends early still gives the attributes it holds, and no read can overflow `i32`.
pub fn fetch_all_header_attributes(buffer: &[u8]) -> AttributeMap {
    let buffer_end = i32::try_from(buffer.len()).unwrap_or(i32::MAX);
    let end = get_header_size(buffer).min(buffer_end);
    let mut attributes = AttributeMap::new();
    let mut pos = HEADER_OPTIONS_POSITION;
    let mut key_buffer = [0; MAX_ATTRIBUTE_KEY_LENGTH];
    let mut value_buffer = vec![0; MAX_ATTRIBUTE_VALUE_LENGTH];
    while pos < end {
        // The values in the header don't use the code point table for their encoding.
        let key_length = read_string_advance(
            buffer,
            MAX_ATTRIBUTE_KEY_LENGTH,
            None,
            &mut key_buffer,
            &mut pos,
        );
        let value_length = read_string_advance(
            buffer,
            MAX_ATTRIBUTE_VALUE_LENGTH,
            None,
            &mut value_buffer,
            &mut pos,
        );
        attributes
            .entry(key_buffer[..key_length].to_vec())
            .or_insert_with(|| value_buffer[..value_length].to_vec());
    }
    attributes
}

/// `HeaderReadWriteUtils::readCodePointTable`: the code points of the `codePointTable` attribute,
/// or `None` when there is no such attribute. An attribute with no code points is `None` too: the
/// C++ returns `std::vector::data()`, which is null for an empty vector, and a null table is how
/// the trie code says there is none.
pub fn read_code_point_table(attributes: &AttributeMap) -> Option<&[i32]> {
    attributes
        .get(&insert_characters_into_vector(CODE_POINT_TABLE_KEY))
        .filter(|table| !table.is_empty())
        .map(Vec::as_slice)
}

/// `HeaderReadWriteUtils::readCodePointVectorAttributeValue`: the value's code points, or none when
/// the key is missing.
pub fn read_code_point_vector_attribute_value(attributes: &AttributeMap, key: &str) -> Vec<i32> {
    attributes
        .get(&insert_characters_into_vector(key))
        .cloned()
        .unwrap_or_default()
}

/// `HeaderReadWriteUtils::readBoolAttributeValue`: the integer value, as [`read_int_attribute_value`]
/// reads it, is not 0. A missing or non-numeric value is `default_value`.
pub fn read_bool_attribute_value(
    attributes: &AttributeMap,
    key: &str,
    default_value: bool,
) -> bool {
    read_int_attribute_value(attributes, key, i32::from(default_value)) != 0
}

/// `HeaderReadWriteUtils::readIntAttributeValue`: the integer value of `key`, or `default_value`
/// when the key is missing or its value is not a number.
pub fn read_int_attribute_value(attributes: &AttributeMap, key: &str, default_value: i32) -> i32 {
    match attributes.get(&insert_characters_into_vector(key)) {
        Some(value) => parse_int(value).unwrap_or(default_value),
        None => default_value,
    }
}

/// The digit loop of `readIntAttributeValueInner`: an optional `-` first, then decimal digits. An
/// empty value (or a lone `-`) is 0. Anything else is not a number. The arithmetic wraps, where the
/// C++ overflows an `int`, which is undefined behaviour there.
fn parse_int(value: &[i32]) -> Option<i32> {
    let (negative, digits) = match value.split_first() {
        Some((&first, rest)) if first == i32::from(b'-') => (true, rest),
        _ => (false, value),
    };
    let mut result: i32 = 0;
    for &code_point in digits {
        if !(i32::from(b'0')..=i32::from(b'9')).contains(&code_point) {
            return None;
        }
        result = result
            .wrapping_mul(10)
            .wrapping_add(code_point - i32::from(b'0'));
    }
    Some(if negative {
        result.wrapping_neg()
    } else {
        result
    })
}

/// `HeaderReadWriteUtils::insertCharactersIntoVector`: a key as the code points of its characters.
/// The C++ takes a C string and stops at its NUL; the keys here have none.
pub fn insert_characters_into_vector(key: &str) -> Vec<i32> {
    key.bytes().map(i32::from).collect()
}
