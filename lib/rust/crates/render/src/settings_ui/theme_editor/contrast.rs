//! The property editor's contrast warning, which EditPropertyDialog did not have: a note under
//! the value, "Low contrast: 2.1:1, below 4.5:1", while the colours being edited fall below
//! WCAG 2.x's 4.5:1 for normal text. It is a note, not an error: Apply still takes the value.
//!
//! The pair checked, and only this pair: the property the editor holds, when it is `foreground`
//! or `background` of an element rule, with its value as the form has it now, against the other
//! of the two in the same rule and the same property set, as the draft holds it (so before
//! Apply). Nothing else is compared: not a selector's rule against the plain rule it overrides,
//! not a foreground inherited from a parent element, not a border or a shadow. Each side must
//! come to a colour: a static one, a dynamic one through the palette, or a variable through
//! `@defines` one level, as the keyboard resolves it; an image, a keyword, an undefined variable
//! or a dynamic colour without the palette is not checked.
//!
//! A translucent foreground is laid over the background. Under a translucent background lies
//! the plain `keyboard` rule's background (the one with no attributes and no selector), which
//! the keys and the rest are drawn on, when it is an opaque colour. When it is not one, or the
//! rule is the keyboard's own, what lies beneath is not known (an image, or the app behind the
//! keyboard), so the background is laid over black and over white and the worse ratio counts.
use super::property::PropertyForm;
use super::property_paint::PROP_ERROR_ID;
use crate::settings_ui::{App, color};
use crate::{panels::Panel, style, text::Text};
use keys::Rect;
use snygg::{grammar::Rule, resolve::PropertySets, text, value::Value};

/// The note's id, after the property editor's others.
pub const CONTRAST_ID: i32 = PROP_ERROR_ID + 4;
/// WCAG 2.x's minimum for normal text (success criterion 1.4.3).
pub const MINIMUM: f64 = 4.5;
pub const LOW_CONTRAST: &str = "settings__theme_editor__low_contrast";

/// WCAG 2.x's contrast ratio of two opaque colours, (L1 + 0.05) / (L2 + 0.05) with L1 the
/// lighter's relative luminance: 1 to 21.
pub fn ratio(a: u32, b: u32) -> f64 {
    let (la, lb) = (color::luminance(a), color::luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// The ratio of foreground `fg` drawn on background `bg`, both possibly translucent, with
/// `under` beneath them: `under` (opaque) when known, else the worse of black and white.
pub fn contrast(fg: u32, bg: u32, under: Option<u32>) -> f64 {
    let on = |under: u32| {
        let bg = color::over(bg, under);
        ratio(color::over(fg, bg), bg)
    };
    match under {
        Some(u) => on(u),
        None => on(0xff00_0000).min(on(0xffff_ffff)),
    }
}

/// The ratio as the note shows it: one decimal, rounded down, so a ratio short of 4.5 never
/// reads as 4.5.
pub fn shown(r: f64) -> String {
    format!("{:.1}", (r * 10.0).floor() / 10.0)
}

/// Value `name` of set `set` of `rule` in `rules`.
fn held<'a>(
    rules: &'a [(Rule, PropertySets<Value>)],
    (rule, set): (&Rule, usize),
    name: &str,
) -> Option<&'a Value> {
    let (_, sets) = rules.iter().find(|(r, _)| r == rule)?;
    let props = match sets {
        PropertySets::Single(props) => (set == 0).then_some(props)?,
        PropertySets::Multiple(all) => all.get(set)?,
    };
    props.iter().find(|(k, _)| *k == text(name)).map(|(_, v)| v)
}

impl App {
    /// The edited pair's ratio when it falls below [`MINIMUM`]; None when it does not, or when
    /// the form holds no pair this checks.
    pub fn low_contrast(&self) -> Option<f64> {
        let f = self.theme_editor.prop.as_ref()?;
        let Rule::Element { name, .. } = &f.rule else {
            return None;
        };
        let other = match f.name.as_str() {
            "foreground" => "background",
            "background" => "foreground",
            _ => return None,
        };
        let rules = &self.theme_editor.rules;
        let mine = self.resolved(&f.value()?)?;
        let theirs = self.resolved(held(rules, (&f.rule, f.set), other)?)?;
        let (fg, bg) = if other == "background" {
            (mine, theirs)
        } else {
            (theirs, mine)
        };
        let under = self
            .keyboard_background()
            .filter(|_| *name != text("keyboard"));
        Some(contrast(fg, bg, under)).filter(|r| *r < MINIMUM)
    }
    /// Value `v` as a colour: a variable through `@defines` one level, a dynamic colour through
    /// the palette.
    fn resolved(&self, v: &Value) -> Option<u32> {
        let v = match v {
            Value::Variable(key) => self.defined(key)?,
            v => v.clone(),
        };
        match v {
            Value::Color(c) => Some(style::argb(c)),
            v => self.dynamic_color(&v),
        }
    }
    /// The plain `keyboard` rule's background, when it is an opaque colour.
    fn keyboard_background(&self) -> Option<u32> {
        let rules = &self.theme_editor.rules;
        let (keyboard, _) = rules.iter().find(|(r, _)| r.format() == text("keyboard"))?;
        let c = self.resolved(held(rules, (keyboard, 0), "background")?)?;
        (c >> 24 == 0xff).then_some(c)
    }
    /// The value row, and the low contrast note under it; their height.
    pub(super) fn checked_value(
        &mut self,
        p: &mut Panel,
        (i, f): (usize, &PropertyForm),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let h = self.value_part(p, (i, f), at, clip, text);
        let Some(r) = self.low_contrast() else {
            return h;
        };
        let note = self.label(LOW_CONTRAST).replace("{ratio}", &shown(r));
        let ink = self.colors[2];
        let note = (CONTRAST_ID, note, ink, 0.0);
        h + self.theme_note(p, note, (at.0, at.1 + h), clip, text)
    }
}
