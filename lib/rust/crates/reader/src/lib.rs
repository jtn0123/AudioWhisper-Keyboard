//! Reading a page aloud: the decisions behind it that don't need Android, as the keyboard's
//! Kotlin had them in its `reader` package. Which block reading starts at, how the pieces of text a
//! walk of the screen found join into blocks, where a selection is, where reading carries on after
//! a scroll and how text is cut for the speech engine ([`page_text`]); whether the block being
//! read fits in view ([`follow_along`]); which of the screen's lists is the one that scrolls,
//! what is in it and how to bring a web page's piece into view ([`screen`]); and whether a press
//! of the speaker key was a tap or a hold ([`speaker_key`]); which of the phone's voices may
//! read, never one that needs the network ([`voices`]); a reading's steps, from the stop button
//! to the end of the page ([`reading`]); which window is the app's ([`window`]); and how long the
//! neural voice stays loaded and a reading waits for the service ([`lifetime`]).
//!
//! Text is UTF-16 units (`&[u16]`), as a Kotlin `String` is, judged by Kotlin's character rules
//! (the `chars` crate).

#![forbid(unsafe_code)]

pub mod follow_along;
pub mod lifetime;
pub mod page_text;
pub mod reading;
pub mod screen;
pub mod speaker_key;
pub mod voices;
pub mod window;

pub mod generator;
pub mod play_queue;
pub mod walk;
pub mod wave;
