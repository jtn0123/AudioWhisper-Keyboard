//! The search and its scoring, ported from `suggest/core` and `suggest/policyimpl/typing`: the
//! typing decoder that turns taps (and the words before them) into suggestions.
//!
//! The public surface is small: [`suggest`] (the app's call, shaped like the JNI method
//! `getSuggestionsNative`), [`suggest_with`] (any options array and weight), [`Query`], [`Output`]
//! and [`DicTraverseSession`], which a caller keeps between calls so that the node cache continues
//! from keystroke to keystroke. Everything else is private to the module.
//!
//! # What is ported
//!
//! - `suggest/core/dicnode`: `dicnode` (node, its states, pool, queue, cache, vector, utils).
//! - `suggest/core/dictionary`: `dictionary.cpp`'s `getSuggestions` and `getPredictions` with
//!   `NgramListenerForPrediction` (`dictionary`, `suggest::get_suggestions`), `digraph_utils`
//!   (`digraph`) and `error_type_utils` (`error_type`).
//! - `suggest/core/policy`, `suggest/policyimpl/typing`: `typing` and `scoring_params`. The C++
//!   interfaces (`Traversal`, `Weighting`, `Scoring`, `SuggestPolicy`) are virtual with one
//!   reachable implementation (typing; `isGesture` is 0), so they are plain functions.
//! - `suggest/core/result`, `suggest/core/session`, `suggest/core/suggest.{h,cpp}`,
//!   `suggest_options.h`: `result`, `session`, `suggest`, `suggest_options`.
//! - The trie policy's `createAndGetAllChildDicNodes` and `createAndGetLeavingChildNode`:
//!   `children`.
//! - libc++'s `std::priority_queue` push and pop: `heap` (the C++ order is not a strict weak
//!   order, so the heap's moves show in the results).
//! - The pure logic of `latinime_BinaryDictionary_getSuggestions`: [`jni`].
//!
//! # What is not, and why that is unreachable for typing
//!
//! - `policyimpl/gesture`, `normal_distribution*`, the gesture parts of `geometry_utils` and
//!   `proximity_info_state_utils`: they run only when `SuggestOptions::isGesture` is true, which
//!   the app never sets.
//! - `dic_node_profiler`, the `DEBUG_*` dumps, `PROF_*` timers and `TimeKeeper`: debug builds.
//! - `DicNode_InputStateG`, `updateInputIndexG`, `DicNodeStateInput::updateInputIndexG`,
//!   `getProximityTypeG`, `getPrevCodePointG`, `getTerminalDiffCostG`: set or called only by the
//!   gesture weighting (`mNeedsToUpdateInputStateG` stays false for typing).
//! - `Scoring::getMostProbableString`: a no-op for typing.
//! - The second pointer (`MAX_POINTER_COUNT_G` is 2 for gestures; typing has 1).
//!
//! # Floating point
//!
//! Scores are `f32`, with the C++'s types and order of operations, and compared by bits. clang
//! (the C++ is built with the default `-ffp-contract=on`, as is the Android NDK's; `Android.mk`
//! adds no fast-math flag) fuses a multiply and an add of one expression into a multiply-add on
//! arm64. Two expressions in the typing path are like that, and are written with `mul_add`:
//! `DicNodeStateScoring::getCompoundDistance` (`spatial + language * weight`) and
//! `TypingScoring::calculateFinalScore`'s `maxDistance` (`1.1214 + inputSize * 0.1`, where the
//! fusing does change the last bit). On aarch64 `mul_add` is the `fmadd` instruction. On a target
//! with no fused multiply-add instruction (x86-64 without FMA, an Android emulator) it is a call
//! to `fmaf`, which is correctly rounded in bionic as everywhere, but there clang does not fuse
//! the C++ either, so the two could differ in the last bit of `maxDistance`.
//!
//! No other float function is reachable for typing: `sqrtf`, `hypotf`, `expf`, `logf`, `atan2f`,
//! `sinf`, `cosf`, `floorf` and `fabsf` appear in the C++ only in gesture code and in the
//! keyboard's `KEYBOARD_HYPOTENUSE`, which typing never reads, so on arm64 the answers do not
//! depend on the platform's math library (Android's bionic included).

mod children;
mod dicnode;
mod dictionary;
mod digraph;
mod error_type;
mod heap;
pub mod jni;
mod result;
mod scoring_params;
mod session;
mod suggest;
mod suggest_options;
mod suggest_process;
mod typing;

pub use jni::{Output, Query, suggest, suggest_with};
pub use session::DicTraverseSession;

#[cfg(all(test, not(target_os = "android")))]
mod tests;
