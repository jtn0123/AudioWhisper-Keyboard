//! NLP orchestration. Android executes plans; no JVM references or callbacks enter this crate.
#![forbid(unsafe_code)]

pub mod counters;
mod policy;
mod protocol;
mod session;
pub mod stores;
#[cfg(test)]
mod tests;
pub mod typing;
#[cfg(test)]
mod typing_tests;
pub use policy::Context;
pub use session::Session;
pub type Field = Vec<u16>;
pub fn number(value: impl ToString) -> Field {
    value.to_string().encode_utf16().collect()
}
pub fn int(fields: &[Field], at: usize) -> i64 {
    fields
        .get(at)
        .and_then(|s| String::from_utf16(s).ok()?.parse().ok())
        .unwrap_or(0)
}
/// Whether field `at` is a [`wire::flag`] that is true; a missing one is false.
pub fn flag_at(fields: &[Field], at: usize) -> bool {
    fields
        .get(at)
        .is_some_and(|value| wire::flag::decode(value))
}
pub fn text(value: &str) -> Field {
    value.encode_utf16().collect()
}
