//! The keyboard's touch gestures, as FlorisBoard's Kotlin had them: which way a finger swiped and
//! by how many units ([`swipe`], from `SwipeGesture.Detector`), and whether a touch is the start of
//! a glide across the keys ([`glide_start`], from `GlideTypingGesture.Detector`). Kotlin still reads
//! Android's touch events, measures velocity and acts on what these decide.

#![forbid(unsafe_code)]

pub mod glide_start;
pub mod java_math;
pub mod swipe;
