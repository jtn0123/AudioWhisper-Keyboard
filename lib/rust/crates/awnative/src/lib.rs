//! libawnative.so, the keyboard's one Rust library: the JNI entry points for the crates beside
//! this one, which hold the logic and know nothing of Java. Kotlin reaches them through lib/rust's
//! `com.jtn0123.audiowhisper.rust` package.

use std::ffi::c_void;

use jni_sys::{JNI_VERSION_1_6, JavaVM, jint};

mod dictation;
mod emoji;
mod gestures;
mod gif;
mod giphy;
mod glide;
mod handle;
mod jni;
mod keys;
mod panic;
mod reader;
mod screen;
mod spacing;
mod suggest;
mod text;

/// Called once when Java loads the library.
#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(_: *mut JavaVM, _: *mut c_void) -> jint {
    panic::log_to_logcat();
    JNI_VERSION_1_6
}
