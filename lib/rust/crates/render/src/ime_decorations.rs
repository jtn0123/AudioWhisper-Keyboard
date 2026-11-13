//! Decorative incognito watermark and the read-only dictation preview.
use crate::{
    panels::{Command, Item, Panel},
    scene::Scene,
    style::Style,
    text::Text,
    theme::{Request, Theme},
};
use keys::Rect;
use unicode_segmentation::UnicodeSegmentation;
#[derive(Clone, Debug, PartialEq)]
pub struct Watermark {
    pub bounds: Rect,
    pub icon: String,
    pub style: Style,
}

/// Keep the newest grapheme clusters; a short viewport never exposes a broken cluster.
pub fn start_ellipsis(text: &str, width: f32, mut measure: impl FnMut(&str) -> f32) -> String {
    if measure(text) <= width {
        return text.into();
    }
    if measure("…") > width {
        return String::new();
    }
    for (index, _) in text.grapheme_indices(true).skip(1) {
        let candidate = format!("…{}", &text[index..]);
        if measure(&candidate) <= width {
            return candidate;
        }
    }
    "…".into()
}
pub fn show_watermark(incognito: bool, behind: bool) -> bool {
    incognito && behind
}

pub fn prepare(
    scene: &mut Scene,
    theme: &mut Theme,
    text: &mut Text,
    r: &Request,
    icon: &str,
    preview: &str,
    keyboard_height: f32,
) {
    scene.watermark = if !icon.is_empty() && scene.panel.is_none() {
        let mut style = theme.style(&r.element("incognito-mode-indicator", 0));
        let bounds = Rect {
            left: 0.0,
            top: scene.height as f32 - keyboard_height,
            right: scene.width as f32,
            bottom: scene.height as f32,
        };
        style.size = bounds.width().min(bounds.height());
        Some(Watermark {
            bounds,
            icon: icon.into(),
            style,
        })
    } else {
        None
    };
    scene.preview = None;
    if preview.is_empty() || scene.panel.is_some() || scene.search.is_some() {
        return;
    }
    let Some(bar) = &scene.smartbar else { return };
    let bounds = bar.layout.center;
    let mut item = Item::new(
        810_000,
        bounds,
        preview.into(),
        "smartbar-candidate-word-text",
        Command::None,
    );
    let style = theme.style(&r.element("smartbar-candidate-word-text", 0));
    item.glyph = start_ellipsis(preview, bounds.width(), |s| {
        text.measure(s, style.size, style.mono, style.weight)
    });
    item.styles = [style.clone(), style];
    scene.preview = Some(Panel {
        items: vec![item],
        ..Default::default()
    });
}
