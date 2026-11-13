//! Pure Rust keyboard drawing; Android surfaces belong to awnative.
#![forbid(unsafe_code)]

mod backend;
pub mod hybrid;
pub mod profile;
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
