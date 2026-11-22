//! Material You dynamic colours as this device draws them. Kotlin puts the palette the keyboard
//! draws the component in into the report (`snyggPalette`: the accent colour's scheme, light and
//! dark, under the component's Material You flags, as ProvideSnyggTheme makes it), and the theme
//! editor shows a dynamic colour's swatch from it: beside each name in the light and dark
//! dropdowns, after the value row and in the rules list. EditPropertyDialog's swatches came from
//! `dynamicColorScheme` with materialkolor's default palette style, so under any other style
//! they were not the colours the keyboard showed; these are. A report whose palette is missing
//! or whose lists disagree draws no dynamic swatch.
use crate::panels::{Panel, rect};
use crate::settings_ui::{
    App,
    catalogue::{Kind, theme_editor::Part},
};
use keys::Rect;
use snygg::{
    text,
    value::{Encoder, Value},
};
use wire::settings_ui::theme_editor::report;

/// The palette's colour names, and each one's light and dark colour as ARGB.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Palette {
    pub names: Vec<String>,
    pub light: Vec<u32>,
    pub dark: Vec<u32>,
}
impl Palette {
    /// Report `p`'s palette; none when a colour does not read or the lists' lengths differ.
    pub fn read(p: &[String]) -> Self {
        let names: Vec<String> = parts(&p[report::COLOR_NAMES]).map(Into::into).collect();
        let colors = |i: usize| {
            parts(&p[i])
                .map(|c| c.parse().ok())
                .collect::<Option<Vec<_>>>()
        };
        match (colors(report::LIGHT_COLORS), colors(report::DARK_COLORS)) {
            (Some(light), Some(dark))
                if light.len() == names.len() && dark.len() == names.len() =>
            {
                Self { names, light, dark }
            }
            _ => Self::default(),
        }
    }
    /// Colour `name` of the dark palette or the light one.
    pub fn color(&self, name: &str, dark: bool) -> Option<u32> {
        let i = self.names.iter().position(|n| n == name)?;
        Some(if dark { self.dark[i] } else { self.light[i] })
    }
}
fn parts(s: &str) -> impl Iterator<Item = &str> {
    s.split('/').filter(|n| !n.is_empty())
}

impl App {
    /// The colour dynamic colour `v` is on this device; None for any other value, or a name the
    /// palette lacks.
    pub fn dynamic_color(&self, v: &Value) -> Option<u32> {
        let Value::DynamicColor { dark, name } = v else {
            return None;
        };
        let name = String::from_utf16_lossy(name);
        self.theme_editor.component.palette.color(&name, *dark)
    }
    /// Whether the list open is a dynamic colour's names: the dark ones or the light.
    fn dynamic_list(&self) -> Option<bool> {
        let m = self.modal.as_ref()?;
        let row = self.rows().get(m.row)?;
        if !matches!(row.kind, Kind::ThemeEdit(Part::PropValue)) {
            return None;
        }
        match self.theme_editor.prop.as_ref()?.encoder? {
            Encoder::DynamicLight => Some(false),
            Encoder::DynamicDark => Some(true),
            _ => None,
        }
    }
    /// What a list row keeps at its end for a swatch: the 24 dp swatch and a 12 dp gap in a
    /// dynamic colour's list with a palette, else nothing.
    pub(in crate::settings_ui) fn swatch_room(&self) -> f32 {
        let palette = !self.theme_editor.component.palette.names.is_empty();
        if palette && self.dynamic_list().is_some() {
            self.px(36.0)
        } else {
            0.0
        }
    }
    /// Entry `choice`'s swatch 16 dp from the end of its row `b`, centred on it, in a dynamic
    /// colour's list.
    pub(in crate::settings_ui) fn choice_swatch(
        &mut self,
        p: &mut Panel,
        choice: &str,
        b: Rect,
        clip: Rect,
    ) {
        let Some(dark) = self.dynamic_list() else {
            return;
        };
        let v = Value::DynamicColor {
            dark,
            name: text(choice),
        };
        let size = self.px(24.0);
        let top = b.top + ((b.height() - size) / 2.0).round();
        let at = rect(b.right - self.px(16.0) - size, top, size, size);
        self.value_icon(p, &v, at, false, clip);
    }
}
