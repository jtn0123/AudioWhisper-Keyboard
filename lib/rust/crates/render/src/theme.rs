//! One compiled stylesheet and bounded style cache per native session. Never queried by raster.
use crate::style::Style;
use snygg::{
    resolve::{CompiledTheme, Properties, Query},
    text,
    value::{Unit, Value},
};
use std::collections::HashMap;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Request {
    pub element: String,
    pub mode: String,
    pub shift: String,
    pub code: i32,
    pub key_type: String,
    pub selector: usize,
    pub auto_commit: bool,
    /// Popup elements inherit from the originating key box, not the choice code.
    pub popup_source: Option<i32>,
}
impl Request {
    pub fn element(&self, element: &str, selector: usize) -> Self {
        Self {
            element: element.into(),
            selector,
            ..self.clone()
        }
    }
}
#[derive(Debug)]
pub struct Theme {
    compiled: CompiledTheme,
    light: Properties,
    dark: Properties,
    density: f32,
    font_scale: f32,
    multiplier: f32,
    cache: HashMap<Request, (Properties, Style)>,
    /// The `@font` rules that loaded ([`Theme::load_fonts`]), by name.
    fonts: Vec<(snygg::Text, crate::text::Font)>,
}
impl Theme {
    pub fn new(
        json: &str,
        light: Properties,
        dark: Properties,
        density: f32,
        font_scale: f32,
        multiplier: f32,
    ) -> Result<Self, String> {
        if [density, font_scale, multiplier]
            .iter()
            .any(|v| !v.is_finite() || *v <= 0.0)
        {
            return Err("invalid theme scale".into());
        }
        Ok(Self {
            compiled: CompiledTheme::from_json(json).map_err(|e| e.to_string())?,
            light,
            dark,
            density,
            font_scale,
            multiplier,
            cache: HashMap::new(),
            fonts: Vec::new(),
        })
    }
    /// Loads the files the stylesheet's `@font` rules name, `files` giving each `src` URI's
    /// absolute path as Kotlin's asset resolver resolved it; see [`crate::theme_fonts::load`].
    /// Elements whose `font-family` names a rule that loaded draw in it from then on.
    pub fn load_fonts(&mut self, files: &[(String, String)]) {
        self.fonts = crate::theme_fonts::load(&self.compiled.fonts, files);
        self.cache.clear();
    }
    /// The `@font` rules that loaded, by name.
    pub fn fonts(&self) -> &[(snygg::Text, crate::text::Font)] {
        &self.fonts
    }
    pub fn properties(&mut self, r: &Request) -> Properties {
        self.resolve(r).0
    }
    pub fn style(&mut self, r: &Request) -> Style {
        self.resolve(r).1
    }
    pub fn density(&self) -> f32 {
        self.density
    }
    pub fn font_scale(&self) -> f32 {
        self.font_scale
    }
    pub fn cached_styles(&self) -> usize {
        self.cache.len()
    }
    fn resolve(&mut self, r: &Request) -> (Properties, Style) {
        if let Some(value) = self.cache.get(r) {
            return value.clone();
        }
        // Cache keys contain bounded domain metadata only, never candidate/typed text.
        if self.cache.len() >= 4096 {
            self.cache.clear();
        }
        let parent_name = match r.element.as_str() {
            "root" => None,
            "window" => Some("root"),
            "key-hint" => Some("key"),
            "smartbar-candidates-row"
            | "smartbar-shared-actions-toggle"
            | "smartbar-extended-actions-toggle"
            | "smartbar-shared-actions-row"
            | "smartbar-extended-actions-row" => Some("smartbar"),
            "smartbar-action-key" => Some("smartbar-shared-actions-row"),
            "smartbar-action-key-icon" | "smartbar-action-key-text" => Some("smartbar-action-key"),
            "smartbar-candidate-word" | "smartbar-candidate-clip" => {
                Some("smartbar-candidates-row")
            }
            "smartbar-candidate-word-text" => Some("smartbar-candidate-word"),
            "smartbar-candidate-clip-text"
            | "smartbar-candidate-clip-icon"
            | "smartbar-candidate-clip-secondary-text" => Some("smartbar-candidate-clip"),
            "smartbar-candidate-word-secondary-text" => Some("smartbar-candidate-word"),
            "media-emoji-key"
            | "media-emoji-subheader"
            | "media-emoji-tab"
            | "media-bottom-row" => Some("media"),
            "media-bottom-row-button" => Some("media-bottom-row"),
            "media-emoji-key-popup-box" => Some("media-emoji-key"),
            "media-emoji-key-popup-element" => Some("media-emoji-key-popup-box"),
            "smartbar-action-tile" => Some("smartbar-actions-overflow"),
            "smartbar-action-tile-icon" | "smartbar-action-tile-text" => {
                Some("smartbar-action-tile")
            }
            "smartbar-actions-overflow-customize-button" => Some("smartbar-actions-overflow"),
            "smartbar-actions-editor-header"
            | "smartbar-actions-editor-subheader"
            | "smartbar-actions-editor-tile-grid" => Some("smartbar-actions-editor"),
            "smartbar-actions-editor-header-button" => Some("smartbar-actions-editor-header"),
            "smartbar-actions-editor-tile" => Some("smartbar-actions-editor-tile-grid"),
            "smartbar-actions-editor-tile-icon" | "smartbar-actions-editor-tile-text" => {
                Some("smartbar-actions-editor-tile")
            }
            "clipboard-header" | "clipboard-content" => Some("window"),
            "clipboard-header-button" | "clipboard-header-text" => Some("clipboard-header"),
            "clipboard-filter-row"
            | "clipboard-grid"
            | "clipboard-subheader"
            | "clipboard-history-disabled-title"
            | "clipboard-history-disabled-message"
            | "clipboard-history-disabled-button"
            | "clipboard-history-locked-title"
            | "clipboard-history-locked-message" => Some("clipboard-content"),
            "clipboard-filter-chip" => Some("clipboard-filter-row"),
            "clipboard-filter-chip-text" | "clipboard-filter-chip-icon" => {
                Some("clipboard-filter-chip")
            }
            "clipboard-item" => Some("clipboard-grid"),
            "clipboard-item-description" => Some("clipboard-item"),
            "clipboard-item-popup"
            | "clipboard-item-timestamp"
            | "clipboard-item-actions"
            | "clipboard-clear-all-dialog" => Some("clipboard-content"),
            "clipboard-item-action" => Some("clipboard-item-actions"),
            "clipboard-item-action-icon" | "clipboard-item-action-text" => {
                Some("clipboard-item-action")
            }
            "clipboard-clear-all-dialog-message" | "clipboard-clear-all-dialog-buttons" => {
                Some("clipboard-clear-all-dialog")
            }
            "clipboard-clear-all-dialog-button" => Some("clipboard-clear-all-dialog-buttons"),
            "one-handed-panel-button" => Some("one-handed-panel"),
            "subtype-panel" | "extracted-landscape-input-layout" => Some("root"),
            "subtype-panel-header" | "subtype-panel-list" => Some("subtype-panel"),
            "subtype-panel-list-item" => Some("subtype-panel-list"),
            "subtype-panel-list-item-icon-leading" | "subtype-panel-list-item-text" => {
                Some("subtype-panel-list-item")
            }
            "extracted-landscape-input-field" | "extracted-landscape-input-action" => {
                Some("extracted-landscape-input-layout")
            }
            "key-popup-element" | "key-popup-extended-indicator" => Some("key-popup-box"),
            _ => Some("window"),
        };
        let parent_selector = match r.element.as_str() {
            "key-hint"
            | "smartbar-candidate-word-text"
            | "smartbar-candidate-clip-text"
            | "smartbar-action-tile-text"
            | "smartbar-action-tile-icon"
            | "smartbar-actions-editor-tile-text"
            | "smartbar-actions-editor-tile-icon" => r.selector,
            _ => 0,
        };
        let parent = parent_name.map_or_else(
            || vec![(text("font-size"), Value::Dimension(Unit::Sp, 14.0))],
            |name| {
                let mut parent = r.element(name, parent_selector);
                if r.element == "key-popup-element" {
                    parent.code = r.popup_source.unwrap_or(r.code);
                    parent.popup_source = None;
                }
                self.properties(&parent)
            },
        );
        let attrs = vec![
            (text("mode"), text(&r.mode)),
            (text("shiftstate"), text(&r.shift)),
            (text("code"), text(&r.code.to_string())),
            (text("type"), text(&r.key_type)),
            (
                text("state"),
                text(if r.auto_commit { "active" } else { "inactive" }),
            ),
            (
                text("auto-commit"),
                text(if r.auto_commit { "1" } else { "0" }),
            ),
        ];
        let properties = self.compiled.resolve_with_palette(
            &Query {
                element: &text(&r.element),
                attributes: &attrs,
                selector: r.selector,
            },
            &parent,
            self.multiplier,
            &self.light,
            &self.dark,
        );
        let mut style = Style::from_properties(&properties, self.density, self.font_scale);
        if let Some(Value::Font(name)) = snygg::resolve::get(&properties, &text("font-family")) {
            let font = self.fonts.iter().find(|(n, _)| n == name);
            style.font = font.map_or(style.font, |(_, font)| *font);
        }
        self.cache
            .insert(r.clone(), (properties.clone(), style.clone()));
        (properties, style)
    }
}
