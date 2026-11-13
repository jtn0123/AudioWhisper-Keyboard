//! FlorisBoard's text composers, which turn a keystroke and the text before the cursor into what
//! to delete and what to type instead:
//!
//! - [`rules`]: a table of endings and what they become, such as Telex for Vietnamese
//!   (`WithRules`).
//!
//! Each answers `None` where the Kotlin answered `0 to toInsert`: nothing to delete, and the
//! keystroke typed as it is. Text is UTF-16 units (`&[u16]`), as a Kotlin `String` is, and need
//! not be valid UTF-16. Case mapping (only [`rules`] does any) is Kotlin's, from `chars`.

#![forbid(unsafe_code)]

pub mod rules;

/// What a composer does with a keystroke: delete this many UTF-16 units before the cursor, then
/// type `text` in place of the keystroke.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Action {
    /// Units to delete; a rule with an empty ending asks for -1, as the Kotlin did.
    pub delete: i32,
    pub text: Vec<u16>,
}

impl Action {
    fn new(delete: i32, text: Vec<u16>) -> Self {
        Action { delete, text }
    }
}
