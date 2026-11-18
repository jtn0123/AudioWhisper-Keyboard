//! libawnative.so, the keyboard's one Rust library: the JNI entry points for the crates beside
//! this one, which hold the logic and know nothing of Java. Kotlin reaches them through lib/rust's
//! `com.jtn0123.audiowhisper.rust` package.

use std::ffi::c_void;

use jni_sys::{JNI_VERSION_1_6, JavaVM, jint};

mod backup_utils;
mod composing;
mod dictation;
mod editor;
mod emoji;
mod extensions;
mod files;
mod gestures;
mod gif;
mod giphy;
mod glide;
mod handle;
mod ime_window;
mod jni;
mod key_models;
mod keys;
#[cfg(not(target_os = "android"))]
mod latinime;
mod layout_rules;
mod panic;
mod popup;
mod reader;
mod render;
mod render_accessibility;
mod render_content;
mod render_input;
mod render_profile;
mod render_smartbar;
#[cfg(target_os = "android")]
mod render_surface;
mod render_theme;
mod render_trail;
#[cfg(target_os = "android")]
mod render_window;
mod screen;
mod shift;
mod snygg;
mod snygg_resolve;
mod snygg_value;

mod spacing;
mod suggest;
#[cfg(any(target_os = "android", test))]
mod surface_bounds;
mod text;

/// Called once when Java loads the library.
#[unsafe(no_mangle)]
pub extern "system" fn JNI_OnLoad(_: *mut JavaVM, _: *mut c_void) -> jint {
    panic::log_to_logcat();
    JNI_VERSION_1_6
}

mod panel_rules;

mod render_panels;

mod render_emoji;

mod panel_oracle;

mod clipboard;

mod render_clipboard;

mod render_media;

mod render_gif;

mod render_chrome;
mod render_decorations;
mod render_key_preview;
mod render_search;
mod render_subtypes;

mod media_oracle;

mod input;

mod nlp;

mod nlp_stores;

mod dictation_session;

mod sherpa;

#[cfg(not(target_os = "android"))]
mod dictation_segment_test;

mod reader_runtime;
mod tts;

mod render_settings;
mod render_settings_state;
