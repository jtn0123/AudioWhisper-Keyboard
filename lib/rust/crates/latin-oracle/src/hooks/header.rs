//! `HeaderPolicy` and the header readers (`dictionary/header`), from `cpp/hooks/header.cpp`. A
//! [`Policy`] is a C++ `HeaderPolicy` built from a copy of a dictionary's bytes, and an
//! [`Attributes`] is an attribute map fetched from them. Each is freed on drop.

use std::ffi::{CString, c_char, c_void};
use std::ptr;
use std::slice;

/// Zeros after a buffer, enough for the reads the C++ makes past its end. A header's attributes
/// are read up to the header size, and the last one can run on: a key of 257 code points and a
/// value of 2049, at 3 bytes each, is 6918 bytes.
const PADDING: usize = 8 * 1024;
/// The most code points a value holds (`MAX_ATTRIBUTE_VALUE_LENGTH`).
const VALUE_CAPACITY: usize = 2048;

// SAFETY: cpp/hooks/header.cpp defines these with the same C signatures.
unsafe extern "C" {
    fn aw_hook_header_new(buffer: *const u8, version: i32, now: i32) -> *mut c_void;
    fn aw_hook_header_free(handle: *mut c_void);
    fn aw_hook_header_size(handle: *const c_void) -> i32;
    fn aw_hook_header_format_version_number(handle: *const c_void) -> i32;
    fn aw_hook_header_is_valid(handle: *const c_void) -> i32;
    fn aw_hook_header_multi_word_cost_multiplier(handle: *const c_void) -> f32;
    fn aw_hook_header_is_decaying_dict(handle: *const c_void) -> i32;
    fn aw_hook_header_requires_german_umlaut_processing(handle: *const c_void) -> i32;
    fn aw_hook_header_date(handle: *const c_void) -> i32;
    fn aw_hook_header_last_decayed_time(handle: *const c_void) -> i32;
    fn aw_hook_header_ngram_count(handle: *const c_void, ngram_type: i32, max: i32) -> i32;
    fn aw_hook_header_extended_region_size(handle: *const c_void) -> i32;
    fn aw_hook_header_has_historical_info_of_words(handle: *const c_void) -> i32;
    fn aw_hook_header_should_boost_exact_matches(handle: *const c_void) -> i32;
    fn aw_hook_header_forgetting_curve_table_id(handle: *const c_void) -> i32;
    fn aw_hook_header_supports_beginning_of_sentence(handle: *const c_void) -> i32;
    fn aw_hook_header_code_point_table(handle: *const c_void, length: *mut i32) -> *const i32;
    fn aw_hook_header_locale(handle: *const c_void, out: *mut i32, capacity: i32) -> i32;
    fn aw_hook_header_read_value_or_question_mark(
        handle: *const c_void,
        key: *const c_char,
        out: *mut i32,
        out_size: i32,
    );
    fn aw_hook_header_attribute_map(handle: *const c_void) -> *const c_void;
    fn aw_hook_attribute_map_size(map: *const c_void) -> i32;
    fn aw_hook_attribute_map_entry(
        map: *const c_void,
        index: i32,
        key: *mut *const i32,
        key_length: *mut i32,
        value: *mut *const i32,
        value_length: *mut i32,
    );
    fn aw_hook_get_header_size(buffer: *const u8) -> i32;
    fn aw_hook_get_flags(buffer: *const u8) -> i32;
    fn aw_hook_fetch_attributes(buffer: *const u8) -> *mut c_void;
    fn aw_hook_attributes_free(map: *mut c_void);
    fn aw_hook_read_code_point_table(map: *mut c_void, length: *mut i32) -> *const i32;
    fn aw_hook_read_code_point_vector_attribute(
        map: *const c_void,
        key: *const c_char,
        out: *mut i32,
        capacity: i32,
    ) -> i32;
    fn aw_hook_read_bool_attribute(map: *const c_void, key: *const c_char, default: i32) -> i32;
    fn aw_hook_read_int_attribute(map: *const c_void, key: *const c_char, default: i32) -> i32;
}

/// A copy of `buffer` with [`PADDING`] zeros after it.
fn padded(buffer: &[u8]) -> Vec<u8> {
    let mut copy = buffer.to_vec();
    copy.resize(buffer.len() + PADDING, 0);
    copy
}

fn c_key(key: &str) -> CString {
    CString::new(key).expect("a header key has no NUL")
}

/// Copies the `length` ints at `pointer`, which the C++ gave for a vector of that length. An empty
/// vector may give a null pointer, so nothing is read for a length of 0.
fn copy_ints(pointer: *const i32, length: i32) -> Vec<i32> {
    let length = usize::try_from(length).expect("the C++ gives a length");
    if length == 0 {
        return Vec::new();
    }
    // SAFETY: the C++ gave `pointer` for a vector of `length` ints, which lives in a map the
    // caller keeps until this copy is made.
    unsafe { slice::from_raw_parts(pointer, length) }.to_vec()
}

/// Every entry of an attribute map, in the map's order.
fn entries(map: *const c_void) -> Vec<(Vec<i32>, Vec<i32>)> {
    // SAFETY: `map` is a live attribute map, from a policy or from `aw_hook_fetch_attributes`.
    let count = unsafe { aw_hook_attribute_map_size(map) };
    (0..count)
        .map(|index| {
            let mut key: *const i32 = ptr::null();
            let mut key_length = 0;
            let mut value: *const i32 = ptr::null();
            let mut value_length = 0;
            // SAFETY: `index` is below the map's size, and each out pointer is valid to write.
            unsafe {
                aw_hook_attribute_map_entry(
                    map,
                    index,
                    &mut key,
                    &mut key_length,
                    &mut value,
                    &mut value_length,
                );
            }
            (copy_ints(key, key_length), copy_ints(value, value_length))
        })
        .collect()
}

/// A C++ `HeaderPolicy`, built from a dictionary's bytes.
#[derive(Debug)]
pub struct Policy {
    handle: *mut c_void,
}

impl Policy {
    /// `HeaderPolicy(dictBuf, formatVersion)`, with `version` as the `FORMAT_VERSION` number and
    /// `now` as the time `TimeKeeper` reports for a missing date. Holds the C++ lock, since
    /// `TimeKeeper` is global.
    pub fn new(buffer: &[u8], version: i32, now: i32) -> Self {
        let _guard = crate::exclusive_lock();
        let copy = padded(buffer);
        // SAFETY: `copy` holds the buffer and PADDING zeros after it, enough for every read the
        // constructor makes. The handle is freed by `drop`.
        let handle = unsafe { aw_hook_header_new(copy.as_ptr(), version, now) };
        Self { handle }
    }

    fn int(&self, getter: unsafe extern "C" fn(*const c_void) -> i32) -> i32 {
        // SAFETY: the handle is a live HeaderPolicy until `drop`, and the getters only read it.
        unsafe { getter(self.handle.cast_const()) }
    }

    fn flag(&self, getter: unsafe extern "C" fn(*const c_void) -> i32) -> bool {
        self.int(getter) != 0
    }

    pub fn size(&self) -> i32 {
        self.int(aw_hook_header_size)
    }

    pub fn format_version_number(&self) -> i32 {
        self.int(aw_hook_header_format_version_number)
    }

    pub fn is_valid(&self) -> bool {
        self.flag(aw_hook_header_is_valid)
    }

    pub fn multi_word_cost_multiplier(&self) -> f32 {
        // SAFETY: the handle is a live HeaderPolicy until `drop`, and the getter only reads it.
        unsafe { aw_hook_header_multi_word_cost_multiplier(self.handle.cast_const()) }
    }

    pub fn is_decaying_dict(&self) -> bool {
        self.flag(aw_hook_header_is_decaying_dict)
    }

    pub fn requires_german_umlaut_processing(&self) -> bool {
        self.flag(aw_hook_header_requires_german_umlaut_processing)
    }

    pub fn date(&self) -> i32 {
        self.int(aw_hook_header_date)
    }

    pub fn last_decayed_time(&self) -> i32 {
        self.int(aw_hook_header_last_decayed_time)
    }

    /// `getNgramCount` of `getNgramCounts` (`max` false) or `getMaxNgramCounts` (`max` true), for
    /// the `NgramType` with the value `ngram_type`.
    pub fn ngram_count(&self, ngram_type: i32, max: bool) -> i32 {
        // SAFETY: the handle is a live HeaderPolicy until `drop`, and the getter only reads it.
        unsafe { aw_hook_header_ngram_count(self.handle.cast_const(), ngram_type, i32::from(max)) }
    }

    pub fn extended_region_size(&self) -> i32 {
        self.int(aw_hook_header_extended_region_size)
    }

    pub fn has_historical_info_of_words(&self) -> bool {
        self.flag(aw_hook_header_has_historical_info_of_words)
    }

    pub fn should_boost_exact_matches(&self) -> bool {
        self.flag(aw_hook_header_should_boost_exact_matches)
    }

    pub fn forgetting_curve_table_id(&self) -> i32 {
        self.int(aw_hook_header_forgetting_curve_table_id)
    }

    pub fn supports_beginning_of_sentence(&self) -> bool {
        self.flag(aw_hook_header_supports_beginning_of_sentence)
    }

    /// `getCodePointTable`: `None` for a null pointer.
    pub fn code_point_table(&self) -> Option<Vec<i32>> {
        let mut length = 0;
        // SAFETY: `length` is valid to write, and the table, if any, is in the policy's map.
        let table =
            unsafe { aw_hook_header_code_point_table(self.handle.cast_const(), &mut length) };
        (!table.is_null()).then(|| copy_ints(table, length))
    }

    /// `getLocale`.
    pub fn locale(&self) -> Vec<i32> {
        let mut out = vec![0; VALUE_CAPACITY];
        let capacity = i32::try_from(out.len()).expect("a small capacity");
        // SAFETY: `out` has `capacity` ints for the C++ to write into.
        let size =
            unsafe { aw_hook_header_locale(self.handle.cast_const(), out.as_mut_ptr(), capacity) };
        assert!(size <= capacity, "a locale is at most one value long");
        out.truncate(usize::try_from(size).expect("a length"));
        out
    }

    /// `readHeaderValueOrQuestionMark` into `out`: the C++ writes the same entries as the port does.
    pub fn read_value_or_question_mark(&self, key: &str, out: &mut [i32]) {
        let key = c_key(key);
        let size = i32::try_from(out.len()).expect("a small buffer");
        // SAFETY: `key` is a C string, and `out` has `size` ints for the C++ to write.
        unsafe {
            aw_hook_header_read_value_or_question_mark(
                self.handle.cast_const(),
                key.as_ptr(),
                out.as_mut_ptr(),
                size,
            );
        }
    }

    /// `getAttributeMap`, as (key, value) pairs in the map's order.
    pub fn attributes(&self) -> Vec<(Vec<i32>, Vec<i32>)> {
        // SAFETY: the map is the policy's member, live until `drop`.
        let map = unsafe { aw_hook_header_attribute_map(self.handle.cast_const()) };
        entries(map)
    }
}

impl Drop for Policy {
    fn drop(&mut self) {
        // SAFETY: the handle came from `aw_hook_header_new` and is freed only here.
        unsafe { aw_hook_header_free(self.handle) }
    }
}

/// `HeaderReadWriteUtils::getHeaderSize`.
pub fn get_header_size(buffer: &[u8]) -> i32 {
    let copy = padded(buffer);
    // SAFETY: `copy` is padded, so the 4 bytes at the size field are inside it.
    unsafe { aw_hook_get_header_size(copy.as_ptr()) }
}

/// `HeaderReadWriteUtils::getFlags`, as the C++'s `int` holds the `uint16_t`.
pub fn get_flags(buffer: &[u8]) -> i32 {
    let copy = padded(buffer);
    // SAFETY: `copy` is padded, so the 2 bytes of the flags are inside it.
    unsafe { aw_hook_get_flags(copy.as_ptr()) }
}

/// An attribute map from `HeaderReadWriteUtils::fetchAllHeaderAttributes`, with the static readers
/// that take one.
#[derive(Debug)]
pub struct Attributes {
    handle: *mut c_void,
}

impl Attributes {
    /// `fetchAllHeaderAttributes` on `buffer`, the whole file.
    pub fn fetch(buffer: &[u8]) -> Self {
        let copy = padded(buffer);
        // SAFETY: `copy` is padded for every read the fetch makes. The handle is freed by `drop`.
        let handle = unsafe { aw_hook_fetch_attributes(copy.as_ptr()) };
        Self { handle }
    }

    /// Every entry, as (key, value) pairs in the map's order.
    pub fn entries(&self) -> Vec<(Vec<i32>, Vec<i32>)> {
        entries(self.handle.cast_const())
    }

    /// `readIntAttributeValue`.
    pub fn read_int(&self, key: &str, default: i32) -> i32 {
        let key = c_key(key);
        // SAFETY: `key` is a C string, and the map is live until `drop`.
        unsafe { aw_hook_read_int_attribute(self.handle.cast_const(), key.as_ptr(), default) }
    }

    /// `readBoolAttributeValue`.
    pub fn read_bool(&self, key: &str, default: bool) -> bool {
        let key = c_key(key);
        // SAFETY: `key` is a C string, and the map is live until `drop`.
        let value = unsafe {
            aw_hook_read_bool_attribute(self.handle.cast_const(), key.as_ptr(), i32::from(default))
        };
        value != 0
    }

    /// `readCodePointVectorAttributeValue`: empty for a missing key.
    pub fn read_code_point_vector(&self, key: &str) -> Vec<i32> {
        let key = c_key(key);
        let mut out = vec![0; VALUE_CAPACITY];
        let capacity = i32::try_from(out.len()).expect("a small capacity");
        // SAFETY: `key` is a C string, and `out` has `capacity` ints for the C++ to write into.
        let size = unsafe {
            aw_hook_read_code_point_vector_attribute(
                self.handle.cast_const(),
                key.as_ptr(),
                out.as_mut_ptr(),
                capacity,
            )
        };
        assert!(size <= capacity, "a value is at most one value long");
        out.truncate(usize::try_from(size).expect("a length"));
        out
    }

    /// `readCodePointTable`: `None` for a null pointer.
    pub fn code_point_table(&self) -> Option<Vec<i32>> {
        let mut length = 0;
        // SAFETY: `length` is valid to write, and the table, if any, is in the map.
        let table = unsafe { aw_hook_read_code_point_table(self.handle, &mut length) };
        (!table.is_null()).then(|| copy_ints(table, length))
    }
}

impl Drop for Attributes {
    fn drop(&mut self) {
        // SAFETY: the handle came from `aw_hook_fetch_attributes` and is freed only here.
        unsafe { aw_hook_attributes_free(self.handle) }
    }
}
