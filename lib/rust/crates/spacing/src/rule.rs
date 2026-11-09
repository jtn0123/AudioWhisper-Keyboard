//! A punctuation rule, as FlorisBoard's `PunctuationRule` has it: which symbols come before and
//! after an auto-space and a phantom space. Keyboard extensions define rules in their manifests,
//! which Kotlin reads; [`fallback`] is the one used when the subtype's rule isn't found.
//!
//! Each string is a set of symbols, one UTF-16 unit each, as Kotlin's `String.contains(Char)`
//! looks for them.

/// The symbols a rule lists for spacing.
#[derive(Clone, Copy, Debug, Default)]
pub struct Rule<'a> {
    /// Symbols an auto-space is typed after (`symbolsPrecedingAutoSpace`).
    pub preceding_auto_space: &'a [u16],
    /// Symbols an auto-space is typed before (`symbolsFollowingAutoSpace`).
    pub following_auto_space: &'a [u16],
    /// Symbols a phantom space may follow (`symbolsPrecedingPhantomSpace`).
    pub preceding_phantom_space: &'a [u16],
    /// Symbols a phantom space may come before (`symbolsFollowingPhantomSpace`).
    pub following_phantom_space: &'a [u16],
}

/// Whether `symbols` lists `unit`.
pub(crate) fn lists(symbols: &[u16], unit: u16) -> bool {
    symbols.contains(&unit)
}

/// `PunctuationRule.Fallback`: bare-bones matching for spaces when a proper rule isn't found.
pub mod fallback {
    pub const ID: &str = "fallback";
    pub const LABEL: &str = "Fallback";
    pub const SYMBOLS_PRECEDING_AUTO_SPACE: &str = ".,?!";
    pub const SYMBOLS_FOLLOWING_AUTO_SPACE: &str = "";
    pub const SYMBOLS_PRECEDING_PHANTOM_SPACE: &str = ".,?!";
    pub const SYMBOLS_FOLLOWING_PHANTOM_SPACE: &str = "";
    pub const SYMBOLS_TERMINATING_SENTENCE: &str = ".?!";
}
