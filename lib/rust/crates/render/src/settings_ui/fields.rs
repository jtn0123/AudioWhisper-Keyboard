//! The editor family beyond the preview field: DictationScreen's vocabulary OutlinedTextField,
//! its bodyMedium note and ModelStatus's summaries.
//!
//! One `Editor` takes the InputConnection at a time. While a row's field owns it, the preview
//! field's editor waits in `parked`; a row's field starts from, and writes every change back
//! to, its string preference, as the screen's `onValueChange` does.
use super::{
    Action, App, Control,
    catalogue::{Kind, Screen},
    editor::Editor,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// ModelStatus's `describe` strings: ready, missing and damaged (`%1$s` the files), per model.
const SPEECH: [&str; 3] = [
    "dictation__models__speech_ready",
    "dictation__models__speech_missing",
    "dictation__models__speech_damaged",
];
const VOICE: [&str; 3] = [
    "dictation__models__voice_ready",
    "dictation__models__voice_missing",
    "dictation__models__voice_damaged",
];
pub(super) const MODEL_KEYS: [&str; 7] = [
    "dictation__models__checking",
    SPEECH[0],
    SPEECH[1],
    SPEECH[2],
    VOICE[0],
    VOICE[1],
    VOICE[2],
];

pub(super) fn with_text(value: &str) -> Editor {
    let text: Vec<u16> = value.encode_utf16().collect();
    let end = text.len();
    Editor {
        text,
        anchor: end,
        focus: end,
        ..Editor::default()
    }
}

impl App {
    /// The control id of the field that owns `editor`.
    pub fn field_id(&self) -> i32 {
        self.field.map_or(2, |i| 1000 + i as i32)
    }
    /// The editor field `id` shows, and whether it is the one being edited.
    pub fn field_editor(&self, id: i32) -> (Editor, bool) {
        if id == self.field_id() {
            return (self.editor.clone(), true);
        }
        if let Some(e) = self.dialog_field(id) {
            return (e, false);
        }
        match usize::try_from(id - 1000) {
            Ok(i)
                if self
                    .rows()
                    .get(i)
                    .is_some_and(|r| matches!(r.kind, Kind::Field)) =>
            {
                let key = self.rows()[i].key;
                (
                    with_text(self.prefs.get(key).map_or("", String::as_str)),
                    false,
                )
            }
            _ => (self.parked.clone(), false),
        }
    }
    /// The preview field's editor, wherever it is: what a saved state keeps.
    pub fn preview_editor(&self) -> Editor {
        if self.field.is_some() {
            self.parked.clone()
        } else {
            self.editor.clone()
        }
    }
    /// Hands `editor` to `target` (None: the preview field), parking the preview field's; the
    /// IME restarts its input on the new field.
    pub(super) fn own_field(&mut self, target: Option<usize>) {
        if target == self.field {
            return;
        }
        let (next, _) = self.field_editor(target.map_or(2, |i| 1000 + i as i32));
        let mut previous = std::mem::replace(&mut self.editor, next);
        previous.focused = false;
        previous.composing = None;
        if self.field.is_none() {
            self.parked = previous;
        }
        self.field = target;
        self.effects.push(vec!["restartInput".into()]);
    }
    /// Hands the editor to the field control `id` (2, the preview field, or a Field row);
    /// false when this screen shows no such field.
    pub fn focus_field(&mut self, id: i32) -> bool {
        let target = match usize::try_from(id - 1000) {
            _ if id == 2 && self.screen().is_some_and(|s| s.preview) => None,
            Ok(i) if self.dialog_field(id).is_some() => Some(i),
            Ok(i)
                if self
                    .rows()
                    .get(i)
                    .is_some_and(|r| matches!(r.kind, Kind::Field)) =>
            {
                Some(i)
            }
            _ => return false,
        };
        self.own_field(target);
        true
    }
    /// Gives the preview field its editor back when the screen with a row's field goes away.
    pub(super) fn release_field(&mut self) {
        if self.field.take().is_some() {
            self.editor = std::mem::take(&mut self.parked);
        }
    }
    /// After an edit: a row's field saves its text when it changed.
    pub fn sync_field(&mut self) {
        let Some(i) = self.field else {
            return;
        };
        if i >= super::picker::HEX_FIELD {
            return self.sync_dialog_field();
        }
        let key = self.rows()[i].key;
        let value = String::from_utf16_lossy(&self.editor.text);
        if self.prefs.get(key) != Some(&value) {
            self.write(key, value);
        }
    }
    /// Screens showing ModelStatus ask Kotlin to read the model files when they open.
    pub(super) fn check_models(&mut self) {
        let models = |s: &Screen| s.rows.iter().any(|r| matches!(r.kind, Kind::Status(_)));
        if self.screen().is_some_and(models) {
            self.effects.push(vec!["checkModels".into()]);
        }
    }
    /// ModelStatus's `describe` from the `<model>_state`, `<model>_files` and `models_checked`
    /// facts; empty before Kotlin reports any.
    pub fn model_summary(&self, model: &str) -> String {
        let keys = if model == "speech" { SPEECH } else { VOICE };
        match self.fact(&format!("{model}_state")) {
            "ready" if self.fact("models_checked") != "true" => {
                self.label("dictation__models__checking")
            }
            "ready" => self.label(keys[0]),
            "missing" => self.label(keys[1]),
            "damaged" => self
                .label(keys[2])
                .replace("%1$s", self.fact(&format!("{model}_files"))),
            _ => String::new(),
        }
    }
    /// A Text in bodyMedium with 16 dp by 8 dp padding; returns its row height.
    pub(super) fn note(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0];
        let [h, v] = [self.px(16.0), self.px(8.0)];
        let label = self.label(self.rows()[i].label);
        let size = self.sp(14);
        let layout = self.android(text, &label, size, Some(w - 2.0 * h));
        let b = rect(x + h, y + v, layout.width.ceil(), layout.height.ceil());
        self.paint_text(p, &label, b, size, self.colors[1], clip);
        self.push_control(super::cards::plain(1000 + i as i32, b, label, "text"), clip);
        b.height() + 2.0 * v
    }
    /// An OutlinedTextField with `minLines = 4`, then `heightIn(min = 120.dp)` and 16 dp by 8 dp
    /// padding in that order, so the 120 dp minimum includes the padding; returns its row height.
    pub(super) fn text_field(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0];
        let [h, v, pad] = [self.px(16.0), self.px(8.0), self.px(16.0)];
        let id = 1000 + i as i32;
        let (editor, active) = self.field_editor(id);
        let inner = w - 2.0 * h - 2.0 * pad;
        let size = self.sp(16);
        let value = String::from_utf16_lossy(&editor.text);
        let lines = self.android(text, &value, size, Some(inner)).height;
        let line = self.android(text, " ", size, None).height;
        let height = (lines.max(4.0 * line).ceil() + 2.0 * pad).max(self.px(120.0) - 2.0 * v);
        let b = rect(x + h, y + v, w - 2.0 * h, height);
        // OutlinedTextFieldDefaults: a 4 dp shape, outline 1 dp, primary 2 dp when focused.
        let focused = (active && editor.focused) || self.focus == id;
        let (stroke, color) = if focused {
            (self.px(2.0), self.colors[3])
        } else {
            (self.px(1.0), self.colors[7])
        };
        let radius = self.px(4.0);
        self.paint_clipped_box(p, b, color, radius, clip);
        let inside = rect(
            b.left + stroke,
            b.top + stroke,
            b.width() - 2.0 * stroke,
            b.height() - 2.0 * stroke,
        );
        self.paint_clipped_box(p, inside, self.colors[0], radius - stroke, clip);
        let field = Control {
            id,
            bounds: b,
            label: self.label(self.rows()[i].label),
            summary: String::new(),
            role: "edit",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Field(i),
            range: None,
        };
        self.add(p, field, clip, text);
        height + 2.0 * v
    }
}
