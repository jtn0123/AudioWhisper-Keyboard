//! Pure Rust keyboard drawing; Android surfaces belong to awnative.
#![forbid(unsafe_code)]

pub mod accessibility;
pub mod accessibility_labels;
mod backend;
pub mod gesture_config;
mod glide_touch;
pub mod hybrid;
pub mod key_content;
mod keyboard_swipe;
mod keyboard_touch;
pub mod popup_scene;
pub mod profile;
pub mod protocol;
pub mod raster;
pub mod scene;
pub mod smartbar;
mod smartbar_raster;
pub mod smartbar_scene;
mod smartbar_touch;
pub mod style;
pub mod text;
pub mod theme;
mod theme_scene;
pub mod touch;
pub mod trail;
mod velocity;

pub mod arrangement;

mod panel_layout;
mod panel_raster;
pub mod panel_scroll;
mod panel_touch;
pub mod panels;

mod panel_accessibility;

pub mod editing;

pub mod emoji_panel;

mod panel_commands;

pub mod panel_geometry;

pub mod clipboard_policy;

mod clipboard_layout;
pub mod clipboard_panel;

pub mod media_cache;

pub mod gif_panel;

pub mod gif_search;

pub mod clipboard_dialog;
pub mod extract_chrome;
pub mod ime_chrome;
pub mod ime_decorations;
pub mod key_preview;
pub mod subtype_panel;
