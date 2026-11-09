//! The keyboard and the taps on it (`suggest/core/layout`): which keys are near a tap, how far
//! it is from each, and how a typed code point compares with a letter of a dictionary word.
//!
//! Only typing is ported: the app's `isGesture` is 0, so the gesture machinery is skipped (each
//! module says what and why), and the keyboard has no sweet spots. Every float is an `f32` and
//! every integer an `i32`, as in the C++, in the same order of operations; no `libm` function
//! (`hypotf`, `atan2f`, `expf`, ...) is reachable for typing, so the answers don't depend on the
//! platform's.
//!
//! - [`ProximityInfo`] (`proximity_info`, built from [`KeyboardGeometry`], the arguments of the
//!   JNI method `setProximityInfoNative`): the keyboard.
//! - [`ProximityInfoState`] (`proximity_info_state`, `proximity_info_state_utils`): the taps of
//!   one decode, given as [`TypingInput`].
//! - [`proximity_info_utils`]: key lookup and the list of letters near a tap.
//! - [`additional_proximity_chars`]: the vowels English counts as near each other.
//! - [`touch_position_correction_utils`], [`geometry_utils`]: the two small float helpers.
//!
//! Not ported because gesture input is the only user: `normal_distribution`,
//! `normal_distribution_2d` (their only caller is `updateAlignPointProbabilities`) and all of
//! `proximity_info_params` (every constant is used by gesture code or by sweet spots).
//!
//! Where the C++ reads outside an array or divides by zero, these return a harmless value instead
//! and say so; the differences are only for inputs the app never makes.

pub mod additional_proximity_chars;
pub mod geometry_utils;
pub mod proximity_info;
pub mod proximity_info_state;
pub mod proximity_info_state_utils;
pub mod proximity_info_utils;
pub mod touch_position_correction_utils;

pub use proximity_info::{KeyboardGeometry, LayoutError, ProximityInfo};
pub use proximity_info_state::{ProximityInfoState, TypingInput};

#[cfg(test)]
mod tests;
