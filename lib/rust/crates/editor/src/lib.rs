//! The editor owns UTF-16 windows, expected-state reconciliation and ordered Android effects.
//! Platform ICU and provider queries are whole-window requests, outside any native lock.
#![forbid(unsafe_code)]
pub mod attributes;
pub mod caps;
mod content;
mod edit;
mod extracted;
mod high;
mod operations;
mod protocol;
mod punctuation;
mod routing;
mod session;
pub use content::{Content, Range};
pub use session::Session;
pub type Field = Vec<u16>;
pub fn number(value: i32) -> Field {
    value.to_string().encode_utf16().collect()
}
pub fn integer(fields: &[Field], at: usize) -> i32 {
    fields
        .get(at)
        .and_then(|s| String::from_utf16(s).ok()?.parse().ok())
        .unwrap_or(0)
}
pub fn text(value: &str) -> Field {
    value.encode_utf16().collect()
}
