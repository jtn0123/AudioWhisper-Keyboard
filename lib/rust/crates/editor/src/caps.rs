//! Android 16 TextUtils.getCapsMode, with Java's UTF-16 category and letter semantics.
//! Source: AOSP frameworks/base android16-release/core/java/android/text/TextUtils.java.
//! Copyright (C) 2006 The Android Open Source Project; Apache-2.0.
use crate::punctuation::{CLOSE, OPEN};
fn quote(value: u16, ranges: &[(u16, u16)]) -> bool {
    value == 34 || value == 39 || ranges.iter().any(|&(a, b)| (a..=b).contains(&value))
}
pub fn flags(value: &[u16], cursor: i32, requested: i32) -> i32 {
    if cursor < 0 {
        return 0;
    }
    let mut mode = requested & 0x1000;
    if requested & 0x6000 == 0 {
        return mode;
    }
    let mut i = (cursor as usize).min(value.len());
    while i > 0 && quote(value[i - 1], OPEN) {
        i -= 1;
    }
    let mut j = i;
    while j > 0 && matches!(value[j - 1], 32 | 9) {
        j -= 1;
    }
    if j == 0 || value[j - 1] == 10 {
        return mode | 0x2000;
    }
    if requested & 0x4000 == 0 {
        if i != j {
            mode |= 0x2000;
        }
        return mode;
    }
    if i == j {
        return mode;
    }
    while j > 0 && quote(value[j - 1], CLOSE) {
        j -= 1;
    }
    if j > 0 && matches!(value[j - 1], 46 | 63 | 33) {
        if value[j - 1] == 46 {
            for &c in value[..j - 1].iter().rev() {
                if c == 46 {
                    return mode;
                }
                if !chars::is_letter(c) {
                    break;
                }
            }
        }
        return mode | 0x4000;
    }
    mode
}
