//! Java-free Snygg grammar, values, compilation, cascade and editor model.
//! Text is UTF-16, including lone surrogates. `resolve::CompiledTheme::from_json`
//! and `resolve::Query` are directly usable by a renderer without JNI or Kotlin.
//! Kotlin keeps its existing JSON codec/diagnostics and Compose object conversion.
#![forbid(unsafe_code)]

pub mod attributes;
pub mod document;
pub mod editor;
pub mod external;
pub mod grammar;
pub mod json;
pub mod properties;
pub mod resolve;
pub mod spec;
pub mod value;
pub mod value_codec;
pub mod value_spec;
pub mod value_uri;

pub type Text = Vec<u16>;

pub fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}

/// Java String.compareTo returns the actual unit/length difference, not just its sign.
pub fn compare(a: &[u16], b: &[u16]) -> i32 {
    a.iter()
        .zip(b)
        .find_map(|(&a, &b)| (a != b).then_some(i32::from(a) - i32::from(b)))
        .unwrap_or_else(|| (a.len() as i32).wrapping_sub(b.len() as i32))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Number(Text),
    MissingSchema,
    InvalidSchema(Text),
    UnsupportedSchema(Text),
    InvalidRule(Text),
}

impl Error {
    pub fn message(&self) -> Text {
        let (prefix, value, suffix) = match self {
            Self::Number(value) => ("For input string: \"", value.as_slice(), "\""),
            Self::MissingSchema => ("Stylesheet is missing schema reference", &[][..], ""),
            Self::InvalidSchema(value) => (
                "Stylesheet references invalid schema '",
                value.as_slice(),
                "'",
            ),
            Self::UnsupportedSchema(value) => (
                "Stylesheet references unsupported schema '",
                value.as_slice(),
                "'",
            ),
            Self::InvalidRule(value) => ("Invalid rule '", value.as_slice(), "'"),
        };
        [text(prefix), value.to_vec(), text(suffix)].concat()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod value_tests;
