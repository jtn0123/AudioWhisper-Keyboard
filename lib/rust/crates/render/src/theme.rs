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
        })
    }
    pub fn properties(&mut self, r: &Request) -> Properties {
        self.resolve(r).0
    }
    pub fn style(&mut self, r: &Request) -> Style {
        self.resolve(r).1
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
            "key-popup-element" => Some("key-popup-box"),
            _ => Some("window"),
        };
        let parent_selector = match r.element.as_str() {
            "key-hint" | "smartbar-candidate-word-text" | "smartbar-candidate-clip-text" => {
                r.selector
            }
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
        let style = Style::from_properties(&properties, self.density, self.font_scale);
        self.cache
            .insert(r.clone(), (properties.clone(), style.clone()));
        (properties, style)
    }
}
