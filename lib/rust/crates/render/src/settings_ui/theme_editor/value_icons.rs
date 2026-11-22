//! SnyggValueIcon, the preview of a value the property editor and the rules list show beside it:
//! a colour's swatch, a box of the background colour at 4 dp elevation with the checkered backing
//! under a translucent colour, for a static colour and for a dynamic one as the palette has it
//! (`dynamic.rs`). It is 24 dp with 8 dp corners and 3 dp squares, or 16 dp with 4 dp corners and
//! 2 dp squares in its small form. Other kinds show an icon or a shape's outline (`glyphs.rs`),
//! and a variable its link over the small icon of what `@defines` gives it. The rules list shows
//! it as each property's ListItem's `trailing`, here in the 48 dp slot before Delete.
use super::{glyphs, property::PropertyForm, ui::Dropdown};
use crate::settings_ui::{App, color};
use crate::{
    panels::{Panel, rect},
    style,
    text::Text,
};
use keys::Rect;
use snygg::{
    grammar::Rule,
    resolve::PropertySets,
    value::{Encoder, Value},
};

impl App {
    /// Whether Compose drew a preview icon for `v` that this can draw: a dynamic colour's needs
    /// the palette.
    pub fn has_icon(&self, v: &Value) -> bool {
        match v {
            Value::Color(_) | Value::Shape(..) | Value::Variable(_) => true,
            Value::DynamicColor { .. } => self.dynamic_color(v).is_some(),
            _ => glyphs::icon_name(v).is_some(),
        }
    }
    /// Value `v`'s preview icon in `b`, small or not; nothing where Compose drew none.
    pub(super) fn value_icon(
        &mut self,
        p: &mut Panel,
        v: &Value,
        b: Rect,
        small: bool,
        clip: Rect,
    ) {
        if let Some(name) = glyphs::icon_name(v) {
            let ink = self.colors[2];
            return self.paint_icon(p, name, b, ink, clip);
        }
        let color = match v {
            Value::Color(c) => Some(style::argb(*c)),
            Value::Shape(..) => return self.shape_icon(p, v, b, small, clip),
            Value::Variable(key) => return self.variable_icon(p, key, b, small, clip),
            _ => self.dynamic_color(v),
        };
        if let Some(c) = color {
            self.color_icon(p, c, b, small, clip);
        }
    }
    /// A variable's icon: the link, over the small icon of the value `@defines` gives it, as
    /// SnyggValueIcon stacked them 2 dp up; just the link for a name nothing defines, and in the
    /// small form (where Compose went on to draw the value's own small icon, without end for a
    /// variable naming itself).
    fn variable_icon(&mut self, p: &mut Panel, key: &[u16], b: Rect, small: bool, clip: Rect) {
        let (d, ink) = (self.config[2], self.colors[2]);
        let Some(real) = self.defined(key).filter(|_| !small) else {
            return self.paint_icon(p, "link", b, ink, clip);
        };
        let (x, y, s) = (b.left, b.top - 2.0 * d, 16.0 * d);
        self.value_icon(p, &real, rect(x + 8.0 * d, y + 8.0 * d, s, s), true, clip);
        // The link's backing, 16 by 12 dp with the 8 dp corners Compose fitted to 6 dp.
        let backing = rect(x + d, y + 2.0 * d, s, 12.0 * d);
        self.paint_clipped_box(p, backing, self.colors[0], 6.0 * d, clip);
        self.paint_icon(p, "link", rect(x, y, s, s), ink, clip);
    }
    /// What `@defines` gives variable `key`.
    fn defined(&self, key: &[u16]) -> Option<Value> {
        let (_, sets) = self
            .theme_editor
            .rules
            .iter()
            .find(|(r, _)| *r == Rule::Defines)?;
        let PropertySets::Single(props) = sets else {
            return None;
        };
        props.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }
    /// The value row's list, with the value's icon after it where EditPropertyDialog drew one:
    /// for a dynamic colour, and for a variable, its link showing before it names one; its
    /// height.
    pub(super) fn value_dropdown(
        &mut self,
        p: &mut Panel,
        d: Dropdown,
        f: &PropertyForm,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let v = match f.value() {
            Some(v @ Value::DynamicColor { .. }) if self.has_icon(&v) => v,
            v if f.encoder == Some(Encoder::Variable) => v.unwrap_or(Value::Variable(vec![])),
            _ => return self.dropdown(p, d, at, clip, text),
        };
        self.iconed_dropdown(p, d, &v, at, clip, text)
    }
    /// Property row `id`'s icon for `v`, 24 dp in the slot before Delete of the row's bounds
    /// `b`; the row's tap reaches over it, as the whole ListItem took Compose's.
    pub(super) fn trailing_icon(&mut self, p: &mut Panel, id: i32, v: &Value, b: Rect, clip: Rect) {
        if !self.has_icon(v) {
            return;
        }
        let (slot, size) = (self.end_button(b, 2), self.px(24.0));
        let inset = ((slot.width() - size) / 2.0).round();
        let at = rect(slot.left + inset, slot.top + inset, size, size);
        self.value_icon(p, v, at, false, clip);
        let row = self.controls.iter_mut().rev().find(|c| c.id == id);
        if let Some(c) = row.filter(|c| c.bounds.bottom > c.bounds.top) {
            c.bounds.right = slot.right.min(clip.right);
        }
    }
    /// A dropdown with `v`'s icon after it, as EditPropertyDialog's Row put SnyggValueIcon 12 dp
    /// after the dropdown, centred on it; its height.
    pub(super) fn iconed_dropdown(
        &mut self,
        p: &mut Panel,
        d: Dropdown,
        v: &Value,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let size = self.px(24.0);
        let h = self.dropdown_in(p, (d, self.px(12.0) + size), at, clip, text);
        let bottom = at.1 + h - self.px(8.0);
        let top = bottom - self.px(56.0) + ((self.px(56.0) - size) / 2.0).round();
        let left = self.config[0] - self.px(16.0) - size;
        self.value_icon(p, v, rect(left, top, size, size), false, clip);
        h
    }
    /// SnyggValueColorBox: the surface, the checkered backing where `c` lets it through, and `c`.
    fn color_icon(&mut self, p: &mut Panel, c: u32, b: Rect, small: bool, clip: Rect) {
        let d = self.config[2];
        let (radius, grid) = if small { (4.0, 2.0) } else { (8.0, 3.0) };
        let (radius, grid, reach) = (radius * d, grid * d, 4.0 * d * 3.0);
        let outer = rect(
            b.left - reach,
            b.top - reach,
            b.width() + 2.0 * reach,
            b.height() + 2.0 * reach,
        );
        self.paint_clipped_box(p, outer, self.colors[0], radius, clip);
        if let Some(item) = p.items.last_mut() {
            for s in &mut item.styles {
                (s.margin, s.shadow) = ([reach; 4], 4.0 * d);
            }
        }
        if c >> 24 != 0xff {
            let n = b.width().round().max(1.0) as usize;
            let even = 0x2900_0000 | self.colors[15] & 0x00ff_ffff;
            let (kind, name) = (format!("@squares{n}:"), format!("@squares{n}:{even:08x}"));
            self.cached(&name, n, n, &kind, |w, h| {
                color::checkered_box(w, h, radius, grid, even)
            });
            self.paint_image(p, &name, b, clip);
        }
        self.paint_clipped_box(p, b, c, radius, clip);
    }
}
