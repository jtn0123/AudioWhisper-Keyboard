//! User dictionary rules; generic Validation.kt is resource/callback/type-contract glue.
use crate::{blank, text};
use wire::utilities::{EMPTY, FREQUENCY, INVALID, WORD};
pub fn validate(rule: &[u16], value: &[u16], min: i32, max: i32) -> &'static str {
    let start = value
        .iter()
        .position(|&u| !chars::is_whitespace(u))
        .unwrap_or(value.len());
    let end = value
        .iter()
        .rposition(|&u| !chars::is_whitespace(u))
        .map_or(start, |i| i + 1);
    let trimmed = &value[start..end];
    if rule == text(FREQUENCY) {
        match chars::text::to_int_or_null(trimmed) {
            None => EMPTY,
            Some(n) if n < min || n > max => INVALID,
            Some(_) => "",
        }
    } else if blank(value) {
        if rule == text(WORD) { EMPTY } else { "" }
    } else if trimmed
        .iter()
        .any(|&u| chars::is_regex_space(u) || u == 59 || u == 44)
    {
        INVALID
    } else {
        ""
    }
}
