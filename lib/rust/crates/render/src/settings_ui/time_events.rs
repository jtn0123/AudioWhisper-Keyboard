//! What the time dialog's controls do: Jetpref's LocalTimePickerPreference dialog around
//! Material's TimePicker (the dial) or TimeInput (two number fields), switched by its icon button.
//! The keyboard follows TimeInput's focus rules: the selected field asks for focus, Next on the
//! hour moves to the minute, and a non-character key at the start of the minute goes back.
use super::{
    App,
    catalogue::Kind,
    editor::Editor,
    picker::{HEX_FIELD, Picker},
    time::{self, TimePicker},
};
use keys::Rect;

/// The hour and minute fields' indices in `App.field` (controls 7001 and 7002).
pub const HOUR_FIELD: usize = HEX_FIELD + 1;
pub const MINUTE_FIELD: usize = HEX_FIELD + 2;

/// A tap on one of the time dialog's own controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TimeTap {
    /// A TimeSelector: the hour (false) or the minute.
    Select(bool),
    /// The period toggle's AM (false) or PM.
    Period(bool),
    /// The icon button switching between the dial and the keyboard.
    Mode,
    /// A clock number, by the value it shows.
    Number(u8),
}

impl App {
    /// The system's 24-hour setting (`DateFormat.is24HourFormat`), which Kotlin reports.
    pub fn is24(&self) -> bool {
        self.fact("is24hour") == "true"
    }
    /// Material's `autoSwitchToMinute`: off while touch exploration, Switch Access or Voice
    /// Access runs.
    fn autoswitch(&self) -> bool {
        self.fact("accessibility_touch") != "true"
    }
    /// A time row's stored hour and minute; its default when the text is no time.
    pub fn time_of(&self, key: &str) -> (u8, u8) {
        let default = settings::catalogue::get(&settings::text(key)).map_or("", |p| p.default);
        time::parse(&self.value(key))
            .or_else(|| time::parse(default))
            .unwrap_or((0, 0))
    }
    /// Row `i`'s dialog opening: the stored time with the hour selected, on the dial unless the
    /// keyboard was chosen for this row since the screen opened.
    pub(super) fn open_time(&mut self, key: &'static str) -> Picker {
        let (hour, minute) = self.time_of(key);
        let input = self.time_inputs.contains(&key);
        Picker::Time(TimePicker::new(hour, minute, input, self.is24()))
    }
    /// The open time dialog's state.
    pub fn time_picker(&self) -> Option<&TimePicker> {
        let m = self.modal.as_ref()?;
        match (self.rows().get(m.row)?.kind, &self.picker) {
            (Kind::Time, Some(Picker::Time(p))) => Some(p),
            _ => None,
        }
    }
    fn time_picker_mut(&mut self) -> Option<&mut TimePicker> {
        self.time_picker()?;
        match &mut self.picker {
            Some(Picker::Time(p)) => Some(p),
            _ => None,
        }
    }
    /// OK stores the time, the neutral button resets it; either closes the dialog.
    pub(super) fn time_button(&mut self, key: &'static str, confirm: bool) {
        let Some(p) = self.time_picker() else {
            return;
        };
        let stored = time::store(p.hour, p.minute);
        self.close_picker();
        if confirm {
            self.write(key, stored);
        } else {
            self.reset(key);
        }
    }
    pub(super) fn time_tap(&mut self, t: TimeTap) {
        let (is24, autoswitch) = (self.is24(), self.autoswitch());
        let key = self.modal.as_ref().map(|m| self.rows()[m.row].key);
        let Some(p) = self.time_picker_mut() else {
            return;
        };
        let was = p.minutes;
        match t {
            TimeTap::Select(minutes) => p.minutes = minutes,
            TimeTap::Period(pm) => p.set_pm(pm),
            TimeTap::Number(value) => {
                p.pick(value, is24);
                p.minutes |= autoswitch;
            }
            TimeTap::Mode => {
                p.input = !p.input;
                let input = p.input;
                if input {
                    p.start_input(is24);
                }
                if let Some(key) = key {
                    self.time_inputs.retain(|k| *k != key);
                    if input {
                        self.time_inputs.push(key);
                        self.focus_time_field();
                    } else {
                        self.leave_dialog_field();
                    }
                }
                return;
            }
        }
        self.time_changed(was);
    }
    /// After a change: TimeInput's effect, and the newly selected field asking for focus.
    fn time_changed(&mut self, was_minutes: bool) {
        let is24 = self.is24();
        let Some(p) = self.time_picker_mut() else {
            return;
        };
        let moved = p.input && p.minutes != was_minutes;
        if p.settle(is24)
            && let Some(j) = self.time_field()
        {
            let fresh = self.time_picker().map(|p| p.fields[j].clone());
            self.editor = Editor {
                focused: self.editor.focused,
                ..fresh.unwrap_or_default()
            };
            self.effects.push(vec!["restartInput".into()]);
        }
        if moved {
            self.focus_time_field();
        }
    }
    /// Which time field holds the keyboard: 0 the hour, 1 the minute.
    fn time_field(&self) -> Option<usize> {
        self.field
            .and_then(|f| f.checked_sub(HOUR_FIELD))
            .filter(|j| *j < 2)
    }
    /// The selected field takes focus, and with it the keyboard, as TimePickerTextField's
    /// `LaunchedEffect(state.selection)` asks.
    pub(super) fn focus_time_field(&mut self) {
        let held = self.time_field();
        let editor = self.editor.clone();
        let Some(p) = self.time_picker_mut() else {
            return;
        };
        if let Some(j) = held {
            p.fields[j] = Editor {
                focused: false,
                ..editor
            };
        }
        let target = if p.minutes { MINUTE_FIELD } else { HOUR_FIELD };
        self.own_field(Some(target));
        self.editor.focused = true;
        self.effects.push(vec!["editor".into()]);
    }
    /// The field control `id` shows while the keyboard is elsewhere: only the selected one is
    /// a field, the other a TimeSelector.
    pub(super) fn time_dialog_field(&self, id: i32) -> Option<Editor> {
        let p = self.time_picker().filter(|p| p.input)?;
        let j = usize::try_from(id - 1000).ok()?.checked_sub(HOUR_FIELD)?;
        (j == usize::from(p.minutes)).then(|| p.fields[j].clone())
    }
    /// After the keyboard changed a field: `timeInputOnChange`, reverting what it turns down.
    pub(super) fn time_typed(&mut self) {
        let Some(j) = self.time_field() else {
            return;
        };
        let (is24, value) = (self.is24(), self.editor.clone());
        let Some(p) = self.time_picker_mut() else {
            return;
        };
        let old = &p.fields[j];
        let same = (&old.text, old.anchor, old.focus, old.composing)
            == (&value.text, value.anchor, value.focus, value.composing);
        if same {
            return;
        }
        let was = p.minutes;
        let field = p
            .typed(j, &value, is24)
            .unwrap_or_else(|| p.fields[j].clone());
        p.fields[j] = Editor {
            focused: false,
            ..field.clone()
        };
        if field.text != value.text {
            self.editor = Editor {
                focused: value.focused,
                ..field
            };
            self.effects.push(vec!["restartInput".into()]);
        }
        self.time_changed(was);
    }
    /// A key on a time field before the field takes it: true when it was used up. Enter is the
    /// hour's Next and the minute's Done; a non-character key at the minute's start goes back
    /// to the hour, as TimeInput's `onPreviewKeyEvent` has it.
    pub fn field_key(&mut self, key: i32, unicode: i32) -> bool {
        let Some(j) = self.time_field() else {
            return false;
        };
        let back = j == 1 && unicode == 0 && self.editor.selection().0 == 0;
        if key == 66 && j == 1 {
            self.editor.focused = false;
            self.editor.composing = None;
            self.effects.push(vec!["hideEditor".into()]);
            return true;
        }
        if key != 66 && !back {
            return false;
        }
        if let Some(p) = self.time_picker_mut() {
            p.minutes = !back;
        }
        self.focus_time_field();
        true
    }
    /// The keyboard a time field asks for: numbers, with Next on the hour and Done on the minute.
    pub fn number_field(&self) -> Option<&'static str> {
        match self.time_field()? {
            0 => Some("next"),
            _ => Some("done"),
        }
    }
    /// A finger on the dial `b`: down records it; past the 8 dp touch slop it drags the hand,
    /// the tracked point trailing the finger by the slop as detectDragGestures leaves it.
    pub(super) fn dial_at(&mut self, b: Rect, (x, y): (f32, f32), first: bool) {
        let (is24, d) = (self.is24(), self.config[2]);
        let slop = (8.0 * d + 0.5).floor();
        let (centre, reach) = self.dial_centre(b);
        let Some(p) = self.time_picker_mut() else {
            return;
        };
        if first {
            (p.press, p.slop) = (Some((x, y)), None);
            return;
        }
        let Some((px, py)) = p.press else {
            return;
        };
        let (sx, sy) = match p.slop {
            Some(s) => s,
            None => {
                let (dx, dy) = (x - px, y - py);
                let distance = dx.hypot(dy);
                if distance < slop {
                    return;
                }
                let s = (dx / distance * slop, dy / distance * slop);
                p.slop = Some(s);
                s
            }
        };
        let (dx, dy) = (x - sx - centre.0, y - sy - centre.1);
        p.drag_dial(dx, dy, dx.hypot(dy) < reach, is24);
    }
    /// The finger lifting off the dial: a drag's end, or a tap where it lifted when it never
    /// passed the slop. Either moves on to the minute unless autoswitching is off.
    pub(super) fn dial_released(&mut self, b: Rect, (x, y): (f32, f32)) {
        let (is24, autoswitch) = (self.is24(), self.autoswitch());
        let (centre, reach) = self.dial_centre(b);
        let Some(p) = self.time_picker_mut() else {
            return;
        };
        let was = p.minutes;
        match (p.press.take(), p.slop.take()) {
            (Some(_), Some(_)) => p.hand = None,
            (Some(_), None) if b.contains(x, y) => {
                let (dx, dy) = (x - centre.0, y - centre.1);
                p.tap_dial(dx, dy, dx.hypot(dy) < reach, is24);
            }
            _ => return,
        }
        p.minutes |= autoswitch;
        self.time_changed(was);
    }
    /// The dial's centre (`size.center` in whole pixels) and the inner ring's reach, 74 dp
    /// scaled as the dial is.
    fn dial_centre(&self, b: Rect) -> ((f32, f32), f32) {
        let half = (b.width() / 2.0).floor();
        let reach = 74.0 * self.config[2] * b.width() / self.px(256.0);
        ((b.left + half, b.top + half), reach)
    }
    /// The time dialog's focusable controls in order, ahead of its buttons.
    pub(super) fn time_focus(&self) -> Vec<i32> {
        let Some(p) = self.time_picker() else {
            return vec![];
        };
        let mut ids = if p.input {
            vec![
                if p.minutes { 7150 } else { 7001 },
                if p.minutes { 7002 } else { 7151 },
            ]
        } else {
            vec![7150, 7151]
        };
        if !self.is24() {
            ids.extend([7152, 7153]);
        }
        if !p.input {
            ids.extend(7160..7172);
            if self.is24() && !p.minutes {
                ids.extend(7172..7184);
            }
        }
        ids.push(7154);
        ids
    }
    /// Whether a time field should keep the keyboard: only while the dialog shows the fields.
    pub(super) fn time_input_open(&self) -> bool {
        self.time_picker().is_some_and(|p| p.input)
    }
}
