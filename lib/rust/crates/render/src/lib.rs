//! Pure Rust keyboard drawing; Android surfaces belong to awnative.
#![forbid(unsafe_code)]

pub mod accessibility;
pub mod accessibility_labels;
pub mod android_text;
mod backend;
mod color_glyphs;
pub mod compose_velocity;
pub mod gesture_config;
mod glide_touch;
#[cfg(feature = "hybrid")]
pub mod hybrid;
pub mod key_content;
mod keyboard_swipe;
mod keyboard_touch;
pub mod popup_scene;
pub mod profile;
pub mod protocol;
pub mod raster;
pub mod scene;
mod shadow_raster;
pub mod smartbar;
pub mod smartbar_fit;
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

pub mod modal_scene;
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

pub mod settings_ui;

#[cfg(test)]
mod tests {
    use std::{mem::size_of, sync::Arc};

    /// The tests build what ships, and that leaves out the GPU renderer (docs/decisions.md): a
    /// GIF frame then keeps only the CPU's copy of its pixels, not a second one for the GPU.
    #[test]
    fn the_gpu_renderer_is_built_only_on_request() {
        assert_eq!(
            size_of::<crate::media_cache::Frame>(),
            size_of::<Arc<vello_cpu::Pixmap>>() + size_of::<i64>(),
            "the hybrid feature must stay off unless asked for"
        );
    }
}
