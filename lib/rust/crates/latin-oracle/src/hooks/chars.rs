//! `CharUtils`, from `cpp/hooks/chars.cpp`.

// SAFETY: cpp/hooks/chars.cpp defines these with the same C signatures.
unsafe extern "C" {
    fn aw_hook_to_lower_case(c: i32) -> i32;
    fn aw_hook_to_base_code_point(c: i32) -> i32;
    fn aw_hook_to_base_lower_case(c: i32) -> i32;
}

/// `CharUtils::toLowerCase`.
pub fn to_lower_case(c: i32) -> i32 {
    // SAFETY: a pure function of an int.
    unsafe { aw_hook_to_lower_case(c) }
}

/// `CharUtils::toBaseCodePoint`. Not for negative code points: the C++ indexes its table with them.
pub fn to_base_code_point(c: i32) -> i32 {
    assert!(c >= 0, "the C++ reads outside its table for {c}");
    // SAFETY: a pure function of an int, which is in the table's range or above it.
    unsafe { aw_hook_to_base_code_point(c) }
}

/// `CharUtils::toBaseLowerCase`. Not for negative code points, as [`to_base_code_point`].
pub fn to_base_lower_case(c: i32) -> i32 {
    assert!(c >= 0, "the C++ reads outside its table for {c}");
    // SAFETY: as to_base_code_point.
    unsafe { aw_hook_to_base_lower_case(c) }
}
