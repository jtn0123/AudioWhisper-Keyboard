//! Drawing values materialized from a resolved Snygg property set, in physical pixels.
use snygg::{
    resolve::{Properties, get},
    text,
    value::{Corners, Encoder, ShapeKind, Unit, Value},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Style {
    pub background: u32,
    pub foreground: u32,
    pub size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub shadow: f32,
    pub shape: ShapeKind,
    pub corners: [f32; 4],
    pub percent: bool,
    pub mono: bool,
    pub weight: i32,
    pub italic: bool,
    pub margin: [f32; 4],
    pub padding: [f32; 4],
    /// How text too big for its box breaks and ends: `text-max-lines`, `text-overflow` and
    /// `text-align`, which only a label that does not fit uses (`label_fit`).
    pub rules: crate::label_fit::TextRules,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            background: 0,
            foreground: 0xff000000,
            size: 14.0,
            line_height: 0.0,
            letter_spacing: 0.0,
            shadow: 0.0,
            shape: ShapeKind::Rectangle,
            corners: [0.0; 4],
            percent: false,
            mono: false,
            weight: 400,
            italic: false,
            margin: [0.0; 4],
            padding: [0.0; 4],
            rules: Default::default(),
        }
    }
}
impl Style {
    pub fn from_properties(p: &Properties, density: f32, font_scale: f32) -> Self {
        let value = |name: &str| get(p, &text(name));
        let color = |name, fallback| match value(name) {
            Some(Value::Color(c)) => argb(*c),
            _ => fallback,
        };
        let mut out = Self {
            background: color("background", 0),
            foreground: color("foreground", 0xff000000),
            size: match value("font-size") {
                Some(Value::Dimension(Unit::Sp, n)) => n * density * font_scale,
                _ => 14.0 * density * font_scale,
            },
            shadow: match value("shadow-elevation") {
                Some(Value::Dimension(Unit::Dp, n)) => (n * density).max(0.0),
                _ => 0.0,
            },
            // Fontique selects real system faces or variable-font weight instances.
            mono: matches!(value("font-family"), Some(Value::Enum(Encoder::GenericFont, n)) if *n == text("monospace")),
            italic: matches!(value("font-style"),Some(Value::Enum(Encoder::FontStyle,n)) if *n==text("italic")),
            weight: match value("font-weight") {
                Some(Value::FontWeight(w)) => *w,
                _ => 400,
            },
            margin: match value("margin") {
                Some(Value::Padding(a)) => a.map(|v| (v * density).round()),
                _ => [0.0; 4],
            },
            padding: match value("padding") {
                Some(Value::Padding(a)) => a.map(|v| (v * density).round()),
                _ => [0.0; 4],
            },
            rules: crate::label_fit::TextRules::from_properties(p),
            ..Self::default()
        };
        if let Some(Value::Shape(kind, corners)) = value("shape") {
            out.shape = *kind;
            match corners {
                Corners::Dp(c) => out.corners = c.map(|v| v * density),
                Corners::Percent(c) => {
                    out.corners = c.map(|v| v as f32 / 100.0);
                    out.percent = true;
                }
            }
        }
        out
    }
    /// Snygg corner order: top-start, top-end, bottom-end, bottom-start (LTR).
    pub fn radii(&self, width: f32, height: f32) -> [f32; 4] {
        let size = width.min(height).max(0.0);
        if self.shape == ShapeKind::Circle {
            return [size / 2.0; 4];
        }
        let mut c = self
            .corners
            .map(|v| (v * if self.percent { size } else { 1.0 }).max(0.0));
        let total = (c[0] + c[3])
            .max(c[1] + c[2])
            .max(c[0] + c[1])
            .max(c[2] + c[3]);
        if total > size {
            c = c.map(|v| v * size / total);
        }
        c
    }
}
pub fn argb(c: [f32; 4]) -> u32 {
    let b = c.map(|n| (n.clamp(0.0, 1.0) * 255.0).round() as u32);
    b[3] << 24 | b[0] << 16 | b[1] << 8 | b[2]
}
pub fn color(argb: u32) -> Value {
    Value::Color([
        (argb >> 16 & 255) as f32 / 255.0,
        (argb >> 8 & 255) as f32 / 255.0,
        (argb & 255) as f32 / 255.0,
        (argb >> 24) as f32 / 255.0,
    ])
}
