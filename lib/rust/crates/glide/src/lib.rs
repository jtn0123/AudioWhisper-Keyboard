//! Swipe typing's word scorer: given a swipe across the keyboard, the words whose path over the
//! keys it most resembles.
//!
//! A port of FlorisBoard's `StatisticalGlideTypingClassifier` that gives exactly the same words at
//! exactly the same weights; the keyboard's tests compare the two (see docs/rust.md). The keyboard
//! builds a [`GlideIndex`] of its words for a layout, then for each swipe asks it for
//! [`GlideIndex::candidates`], looks up their frequencies and has it [`GlideIndex::score`] them.

#![forbid(unsafe_code)]

mod gesture;
mod index;
mod layout;
mod score;

pub use gesture::{Gesture, MAX_POINTS, SAMPLES};
pub use index::{CharKeys, GlideIndex, IndexError, NO_KEY, distinct_units};
pub use layout::{Key, Layout};
pub use score::{Ranking, ScoreError};
