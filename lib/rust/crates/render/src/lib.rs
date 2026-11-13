//! Pure Rust keyboard drawing; Android surfaces belong to awnative.
#![forbid(unsafe_code)]

pub mod accessibility;
pub mod accessibility_labels;
mod backend;
pub mod gesture_config;
mod glide_touch;
pub mod hybrid;
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
