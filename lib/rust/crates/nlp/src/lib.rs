//! NLP orchestration. Android executes plans; no JVM references or callbacks enter this crate.
#![forbid(unsafe_code)]

mod policy;
mod protocol;
mod session;
pub mod stores;
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
pub fn text(value: &str) -> Field {
    value.encode_utf16().collect()
}
