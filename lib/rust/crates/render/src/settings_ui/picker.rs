//! What the colour and time dialogs' controls do; the time dialog's own rules are in
//! `time_events.rs`. The colour dialog is Jetpref's ColorPickerPreference with
//! `enableAdvancedLayout = true` and no alpha slider: a Presets page (19 swatches and the system
//! accent as Default) and a Custom page (JetPrefColorPicker: a saturation/value plane, a hue bar
//! and a text field), the neutral button switching between them. Its state is Compose's, kept
//! the way Compose keeps it: `dialogValue`, `selectedPreset`, `advanced`, the picker state's hue,
//! saturation, value and editor mode, which outlive a page switch, and the field's format, text
//! and error, which a page switch or leaving the editor starts again.
use super::{
    Action, App,
    catalogue::Kind,
    color::{self, Format, PRESETS},
    time::TimePicker,
    time_events::TimeTap,
};

/// The dialog field index `App.field` holds for the colour text field (control 7000); the time
/// input's hour and minute fields follow it.
pub const HEX_FIELD: usize = 6000;

/// A tap on one of the dialogs' own controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tap {
    /// Preset swatch `j`.
    Swatch(usize),
    /// The Default row: the system accent.
    Accent,
    /// The text field's Edit, Apply changes and Discard changes icons.
    Edit,
    Apply,
    Discard,
    /// One of the time dialog's.
    Time(TimeTap),
}

/// A surface the finger drags.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Surface {
    Plane,
    Hue,
    /// The time picker's clock dial.
    Dial,
}

/// The open dialog's state.
#[derive(Clone, Debug, PartialEq)]
pub enum Picker {
    Color(ColorPicker),
    Time(TimePicker),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorPicker {
    /// `dialogValue`: what OK stores.
    pub value: u32,
    /// The system accent: the Default swatch, and what OK stores as no colour at all.
    pub accent: u32,
    /// `selectedPreset`; Custom forgets it.
    pub preset: Option<usize>,
    /// `advanced`: the Custom page.
    pub custom: bool,
    /// The picker state's hue (degrees), saturation and value.
    pub hsv: [f32; 3],
    /// `isEditorMode`: the text field is editable and the plane and hue bar are greyed out.
    pub editing: bool,
    /// `isEditorError`: Apply found no colour in the text.
    pub error: bool,
    /// `selectedRepresentation`: the form the read-only field shows.
    pub format: Format,
    /// The editable field's text while the keyboard is not on it.
    pub text: String,
}

impl ColorPicker {
    /// The dialog as it opens on `value` (the row's colour or the system accent): the matching
    /// preset ticked, or Custom when the colour is neither a preset nor the accent.
    pub fn new(value: u32, accent: u32) -> Self {
        let opaque = value | 0xff00_0000;
        let preset = PRESETS.iter().position(|&p| p == opaque);
        let [h, s, v, _] = color::to_hsv(value);
        Self {
            value,
            accent,
            preset,
            custom: preset.is_none() && opaque != accent,
            hsv: [h, s, v],
            editing: false,
            error: false,
            format: Format::Hex,
            text: String::new(),
        }
    }
    /// The picker state's colour, `Color.hsv(h, s, v)`: the plane and preview show it.
    pub fn color(&self) -> u32 {
        let [h, s, v] = self.hsv;
        color::hsv(h, s, v, 1.0)
    }
    /// `setColor`: the picker state takes `c` apart.
    fn set(&mut self, c: u32) {
        let [h, s, v, _] = color::to_hsv(c);
        self.hsv = [h, s, v];
    }
    /// Entering the editor, or the Custom page coming back while in it: the field starts on the
    /// colour as the read-only field showed it, with no error.
    fn start_text(&mut self) {
        self.text = color::format(self.format, self.color());
        self.error = false;
    }
}

impl App {
    /// The system accent (`system_accent1_500`), which Kotlin reports as a fact; the palette's
    /// primary until it does.
    pub fn system_accent(&self) -> u32 {
        self.fact("system_accent").parse().unwrap_or(self.colors[3])
    }
    /// A colour row's swatch: its stored colour, or the system accent for none (`safeValue`).
    pub fn swatch(&self, key: &str) -> u32 {
        color::decode(&self.value(key)).unwrap_or_else(|| self.system_accent())
    }
    /// Row `i`'s dialog just opened: its picker starts on the row's value.
    pub(super) fn open_picker(&mut self, i: usize) {
        let r = self.rows()[i];
        self.picker = match r.kind {
            Kind::Color => Some(Picker::Color(ColorPicker::new(
                self.swatch(r.key),
                self.system_accent(),
            ))),
            Kind::Time => Some(self.open_time(r.key)),
            _ => None,
        };
        if self.time_input_open() {
            self.focus_time_field();
        }
    }
    /// The open colour dialog's state.
    pub fn color_picker(&self) -> Option<&ColorPicker> {
        let m = self.modal.as_ref()?;
        match (self.rows().get(m.row)?.kind, &self.picker) {
            (Kind::Color, Some(Picker::Color(p))) => Some(p),
            _ => None,
        }
    }
    fn color_picker_mut(&mut self) -> Option<&mut ColorPicker> {
        self.color_picker()?;
        match &mut self.picker {
            Some(Picker::Color(p)) => Some(p),
            _ => None,
        }
    }
    /// Whether the dialog showing is a colour or time picker, whose buttons act on it.
    pub(super) fn picker_open(&self) -> bool {
        self.color_picker().is_some() || self.time_picker().is_some()
    }
    /// The neutral button (`confirm` false) or OK.
    pub(super) fn picker_button(&mut self, confirm: bool) {
        let Some(m) = &self.modal else {
            return;
        };
        let key = self.rows()[m.row].key;
        if self.time_picker().is_some() {
            return self.time_button(key, confirm);
        }
        let Some(p) = self.color_picker_mut() else {
            return;
        };
        if confirm {
            let stored = color::encode(p.value, p.accent);
            self.close_picker();
            self.write(key, stored);
            return;
        }
        // Presets and Custom: `key(advanced)` composes the page again, so the field starts over.
        p.custom = !p.custom;
        if p.custom {
            p.preset = None;
            p.format = Format::Hex;
            p.start_text();
        }
        self.leave_dialog_field();
    }
    /// Closes the dialog, taking the keyboard back from its field.
    pub(super) fn close_picker(&mut self) {
        self.modal = None;
        self.focus = -1;
        self.leave_dialog_field();
    }
    pub(super) fn tap(&mut self, t: Tap) {
        if let Tap::Time(t) = t {
            return self.time_tap(t);
        }
        let typed =
            (self.field == Some(HEX_FIELD)).then(|| String::from_utf16_lossy(&self.editor.text));
        let Some(p) = self.color_picker_mut() else {
            return;
        };
        match t {
            Tap::Swatch(j) => {
                let c = PRESETS[j] & 0x00ff_ffff | p.value & 0xff00_0000;
                p.preset = Some(j);
                p.value = c;
                p.set(c);
            }
            Tap::Accent => {
                p.preset = None;
                p.value = p.accent;
                p.set(p.accent);
            }
            Tap::Edit => {
                p.editing = true;
                p.start_text();
            }
            Tap::Apply => {
                let text = typed.unwrap_or_else(|| p.text.clone());
                let Some((c, format)) = color::parse(&text) else {
                    p.error = true;
                    return;
                };
                p.set(c);
                p.value = c;
                p.format = format;
                // `isEditorError` is remembered per editor mode, so leaving the editor clears it.
                (p.editing, p.error) = (false, false);
                self.leave_dialog_field();
            }
            Tap::Discard => {
                (p.editing, p.error) = (false, false);
                self.leave_dialog_field();
            }
            Tap::Time(_) => {}
        }
    }
    /// A finger down (`first`) or moving on control `id`: the plane or hue bar follows it,
    /// clamped to its ends, and the dial its own way. False when `id` is not such a surface.
    pub(super) fn drag_at(&mut self, id: i32, x: f32, y: f32, first: bool) -> bool {
        let Some(c) = self.controls.iter().find(|c| c.id == id).cloned() else {
            return false;
        };
        let Action::Drag(surface) = c.action else {
            return false;
        };
        let b = c.bounds;
        if surface == Surface::Dial {
            self.dial_at(b, (x, y), first);
            return true;
        }
        let Some(p) = self.color_picker_mut() else {
            return true;
        };
        if p.editing {
            return true;
        }
        let along = |at: f32, from: f32, size: f32| ((at - from) / size).clamp(0.0, 1.0);
        match surface {
            Surface::Plane => {
                p.hsv[1] = along(x, b.left, b.width());
                p.hsv[2] = 1.0 - along(y, b.top, b.height());
            }
            // The hue bar's control is the track, made 48 dp tall as a touch reaches it.
            Surface::Hue => p.hsv[0] = along(x, b.left, b.width()) * 360.0,
            Surface::Dial => {}
        }
        p.value = p.color();
        true
    }
    /// The finger lifted at (`x`, `y`) from control `id`: a slider reports its value, the dial
    /// ends its drag or takes the tap.
    pub(super) fn released(&mut self, id: i32, x: f32, y: f32) {
        let control = self.controls.iter().find(|c| c.id == id);
        match control.map(|c| (c.action.clone(), c.bounds)) {
            Some((Action::Slider(slot), _)) => self.slider_finished(slot),
            Some((Action::Drag(Surface::Dial), b)) => self.dial_released(b, (x, y)),
            _ => {}
        }
    }
    /// The dialog's focusable controls in order, ahead of its buttons.
    pub(super) fn picker_focus(&self) -> Vec<i32> {
        let Some(p) = self.color_picker() else {
            return self.time_focus();
        };
        match (p.custom, p.editing) {
            (false, _) => (7100..7100 + PRESETS.len() as i32).chain([7120]).collect(),
            (true, false) => vec![7000, 7140],
            (true, true) => vec![7000, 7141, 7142],
        }
    }
    /// Gives the preview field the keyboard back from a dialog field, hiding the keyboard as
    /// Compose does when the field goes read-only or away.
    pub(super) fn leave_dialog_field(&mut self) {
        if self.field.is_some_and(|i| i >= HEX_FIELD) {
            if self.editor.focused {
                self.effects.push(vec!["hideEditor".into()]);
            }
            self.release_field();
        }
    }
    /// The text field a dialog shows as control `id`: its editor when it is open for editing.
    pub(super) fn dialog_field(&self, id: i32) -> Option<super::editor::Editor> {
        let Some(p) = self.color_picker() else {
            return self.time_dialog_field(id);
        };
        (id == 1000 + HEX_FIELD as i32 && p.custom && p.editing)
            .then(|| super::fields::with_text(&p.text))
    }
    /// After an edit in a dialog field: the colour field keeps its text, a time field follows
    /// TimeInput's rules.
    pub(super) fn sync_dialog_field(&mut self) {
        if self.field != Some(HEX_FIELD) {
            return self.time_typed();
        }
        let typed = String::from_utf16_lossy(&self.editor.text);
        if let Some(p) = self.color_picker_mut() {
            p.text = typed;
        }
    }
    /// A dialog field still holding the keyboard after its dialog went away returns it.
    pub(super) fn settle_picker(&mut self) {
        let open =
            self.color_picker().is_some_and(|p| p.custom && p.editing) || self.time_input_open();
        if !open {
            self.leave_dialog_field();
        }
    }
}
