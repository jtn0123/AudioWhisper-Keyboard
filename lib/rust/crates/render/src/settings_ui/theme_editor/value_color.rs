//! The property editor's static colour. Its row shows the colour as fine-tune's colour
//! representation writes it with the alpha (`ColorRepresentation.formatColor(color, withAlpha =
//! true)`) and its swatch after it; a tap opens the settings' colour picker on it with the alpha
//! bar and the field in that representation (`picker.rs`). EditPropertyDialog drew the picker
//! inline, so every drag changed the value; here the picker is a dialog whose OK sets the form's
//! value and shows it on the keyboard, as every other kind's choice does, and whose Cancel keeps
//! the colour it opened on. Compose kept no text field beside the picker, whose own field takes
//! the representation's text forms, so neither does this.
use super::{Tap, property::PropTap, ui::Dropdown};
use crate::settings_ui::{
    Action, App,
    color::{self, Format},
    picker::{ColorPicker, Picker},
};
use crate::{panels::Panel, style, text::Text};
use keys::Rect;
use snygg::value::{Encoder, Value};

impl App {
    /// Fine-tune's colour representation, the form the theme editor shows colours in.
    pub fn color_representation(&self) -> Format {
        match self.value("theme__editor_color_representation").as_str() {
            "RGB" => Format::Rgb,
            "HSV" => Format::Hsv,
            _ => Format::Hex,
        }
    }
    /// The property editor's static colour, if it holds one.
    pub fn form_color(&self) -> Option<u32> {
        match self.theme_editor.prop.as_ref()?.value()? {
            Value::Color(c) => Some(style::argb(c)),
            _ => None,
        }
    }
    /// Value row `i` tapped: the picker opens on the form's colour.
    pub(super) fn open_value_picker(&mut self, i: usize) {
        let Some(c) = self.form_color() else {
            return;
        };
        self.open_modal(i, String::new());
        let picker = ColorPicker::theme(c, self.color_representation());
        self.picker = Some(Picker::Color(picker));
    }
    /// Whether `p` is the property editor's picker: one with the alpha bar, while the form edits
    /// a static colour.
    pub(in crate::settings_ui) fn picks_theme(&self, p: &ColorPicker) -> bool {
        let f = self.theme_editor.prop.as_ref();
        p.alpha.is_some() && f.is_some_and(|f| f.encoder == Some(Encoder::Color))
    }
    /// The picker's OK: the form takes colour `c`, and the keyboard shows it.
    pub(in crate::settings_ui) fn theme_color_picked(&mut self, c: u32) {
        let written = Encoder::Color.serialize(&style::color(c));
        if let (Some(f), Ok(t)) = (self.theme_editor.prop.as_mut(), written) {
            f.texts[0] = String::from_utf16_lossy(&t);
        }
        self.property_changed();
    }
    /// Value row `i` for a static colour: a dropdown showing the colour, which opens the picker,
    /// and the colour's swatch; its height.
    pub(super) fn color_value(
        &mut self,
        p: &mut Panel,
        i: usize,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let c = self.form_color();
        let d = Dropdown {
            id: 1000 + i as i32,
            title: String::new(),
            value: c.map_or_else(String::new, |c| {
                color::format_alpha(self.color_representation(), c)
            }),
            error: c.is_none(),
            enabled: true,
            action: Action::ThemeEdit(Tap::Prop(PropTap::Picker(i))),
        };
        match c {
            Some(c) => self.iconed_dropdown(p, d, &style::color(c), at, clip, text),
            None => self.dropdown(p, d, at, clip, text),
        }
    }
}
