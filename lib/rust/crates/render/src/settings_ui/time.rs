//! Jetpref's LocalTimePickerPreference and the Material 3 (1.4.0) TimePicker and TimeInput its
//! dialog shows, as plain state: the stored form, the summary, the dial's angles and the typed
//! entry's rules. `time_events.rs` wires it to the dialog; `time_dialog.rs` and `time_dial.rs`
//! draw it.
use super::editor::Editor;
use std::f32::consts::TAU;

/// Parses Jetpref's stored `LocalTime`, "HH:mm:ss.SSS": the hour and minute, or None for text
/// its pattern `^(2[0-3]|[0-1][0-9]):([0-5][0-9]):([0-5][0-9])[.]([0-9]{3})$` turns down.
pub fn parse(stored: &str) -> Option<(u8, u8)> {
    let b = stored.as_bytes();
    let shape = b.len() == 12 && b[2] == b':' && b[5] == b':' && b[8] == b'.';
    let digits = [0, 1, 3, 4, 6, 7, 9, 10, 11]
        .iter()
        .all(|&i| b.get(i).is_some_and(u8::is_ascii_digit));
    if !shape || !digits {
        return None;
    }
    let two = |i: usize| (b[i] - b'0') * 10 + b[i + 1] - b'0';
    let (hour, minute, second) = (two(0), two(3), two(6));
    (hour < 24 && minute < 60 && second < 60).then_some((hour, minute))
}

/// What OK stores: `LocalTime(hour, minute)` serialized, its seconds and milliseconds zero.
pub fn store(hour: u8, minute: u8) -> String {
    format!("{hour:02}:{minute:02}:00.000")
}

/// `hourForDisplay`: the hour as the clock shows it, 12 for midnight and noon on a 12-hour clock.
pub fn display_hour(hour: u8, is24: bool) -> u8 {
    match hour {
        _ if is24 => hour % 24,
        _ if hour.is_multiple_of(12) => 12,
        12.. => hour - 12,
        _ => hour,
    }
}

/// Jetpref's `stringRepresentation`, the row's summary: "18:05", or "6:05 PM" on a 12-hour clock
/// (the AM and PM are Jetpref's own English).
pub fn shown(hour: u8, minute: u8, is24: bool) -> String {
    let h = display_hour(hour, is24);
    if is24 {
        format!("{h}:{minute:02}")
    } else {
        format!("{h}:{minute:02} {}", if hour >= 12 { "PM" } else { "AM" })
    }
}

/// The clockwise angle from 12 o'clock of a point `(dx, dy)` from the dial's centre, in [0, 2π).
pub fn turn(dx: f32, dy: f32) -> f32 {
    dx.atan2(-dy).rem_euclid(TAU)
}

/// A drag's value (`rotateTo`): the nearest of the dial's `marks` (12 hours or 60 minutes), a
/// half mark rounding up.
pub fn nearest(angle: f32, marks: u8) -> u8 {
    ((angle / TAU * f32::from(marks) + 0.5).floor() as u32 % u32::from(marks)) as u8
}

/// A tap's value (`onTap`): the nearest hour, or the nearest fifth minute, a half rounding to
/// even as Kotlin's `round` does.
pub fn snapped(angle: f32, minutes: bool) -> u8 {
    let twelfths = (angle / TAU * 12.0).round_ties_even() as u32 % 12;
    (if minutes { twelfths * 5 } else { twelfths }) as u8
}

/// A line `2 * half` wide from `a` to `b` with flat ends in ARGB `color`, as `w` x `h` pixels,
/// each pixel's alpha the share of it the line covers: the dial's hand, as panels draw no
/// slanted lines.
pub fn segment(
    w: usize,
    h: usize,
    a: (f32, f32),
    b: (f32, f32),
    half: f32,
    color: u32,
) -> Vec<i32> {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let length = dx.hypot(dy);
    let (ux, uy) = if length > 0.0 {
        (dx / length, dy / length)
    } else {
        (1.0, 0.0)
    };
    let alpha = (color >> 24) as f32;
    (0..w * h)
        .map(|i| {
            let x = (i % w) as f32 + 0.5 - a.0;
            let y = (i / w) as f32 + 0.5 - a.1;
            let along = x * ux + y * uy;
            let across = (x * uy - y * ux).abs();
            let cover = (half + 0.5 - across).clamp(0.0, 1.0)
                * (along + 0.5).clamp(0.0, 1.0)
                * (length - along + 0.5).clamp(0.0, 1.0);
            (((alpha * cover).round() as u32) << 24 | color & 0x00ff_ffff) as i32
        })
        .collect()
}

/// Kotlin's `String.toInt` over ASCII digits, a sign allowed. A negative number is None: Material
/// would set the hour to it and the dialog's OK would then throw.
fn kotlin_int(text: &str) -> Option<u32> {
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u64 = digits.parse().ok()?;
    if negative && n != 0 {
        return None;
    }
    u32::try_from(n).ok()
}

/// The open time dialog: TimePickerState (hour, minute, selection), AnalogTimePickerState's hand,
/// TimeInput's two text fields and its `userOverride`, and Jetpref's display mode.
#[derive(Clone, Debug, PartialEq)]
pub struct TimePicker {
    /// The hour, 0 to 23, and the minute.
    pub hour: u8,
    pub minute: u8,
    /// `selection`: the minute is being set, not the hour.
    pub minutes: bool,
    /// Jetpref's `displayMode` is Input: TimeInput's text fields in place of the dial.
    pub input: bool,
    /// TimeInput's `hourValue` and `minuteValue`; the one holding the keyboard lives in
    /// `App.editor` meanwhile.
    pub fields: [Editor; 2],
    /// TimeInput's `userOverride` (None until its effect first runs) and the hour and minute
    /// that effect last ran for.
    user_override: Option<bool>,
    seen: (u8, u8),
    /// A finger on the dial: where it went down, and once it passes the touch slop, the slop
    /// vector detectDragGestures leaves between the finger and the point it tracks.
    pub press: Option<(f32, f32)>,
    pub slop: Option<(f32, f32)>,
    /// The hand's angle while a drag holds it off its value; it snaps back as the drag ends.
    pub hand: Option<f32>,
}

impl TimePicker {
    /// The dialog as it opens: the stored time with the hour selected, in `input` mode.
    pub fn new(hour: u8, minute: u8, input: bool, is24: bool) -> Self {
        let mut p = Self {
            hour,
            minute,
            minutes: false,
            input,
            fields: Default::default(),
            user_override: None,
            seen: (hour, minute),
            press: None,
            slop: None,
            hand: None,
        };
        if input {
            p.start_input(is24);
        }
        p
    }
    pub fn pm(&self) -> bool {
        self.hour >= 12
    }
    /// TimeInput composing: both fields start on the time in two digits, the caret at the start,
    /// and its effect runs once without touching them.
    pub fn start_input(&mut self, is24: bool) {
        self.fields = self.texts(is24);
        self.user_override = Some(true);
        self.seen = (self.hour, self.minute);
    }
    fn texts(&self, is24: bool) -> [Editor; 2] {
        [display_hour(self.hour, is24), self.minute].map(|v| Editor {
            text: format!("{v:02}").encode_utf16().collect(),
            ..Editor::default()
        })
    }
    /// The period toggle's AM (false) or PM.
    pub fn set_pm(&mut self, pm: bool) {
        if pm && !self.pm() {
            self.hour += 12;
        } else if !pm && self.pm() {
            self.hour -= 12;
        }
    }
    /// A clock number's own action: what a tap at its centre sets. `value` is the number shown
    /// (1 to 12, 0 to 23 on a 24-hour clock, or a fifth minute).
    pub fn pick(&mut self, value: u8, is24: bool) {
        if self.minutes {
            self.minute = value % 60;
        } else if is24 {
            self.hour = value % 24;
        } else {
            self.hour = value % 12 + if self.pm() { 12 } else { 0 };
        }
        self.hand = None;
    }
    /// The hour on the dial's `twelfth` mark: a 24-hour clock's inner ring is the afternoon
    /// (`moveSelector`), a 12-hour clock keeps its period.
    fn hour_at(&self, twelfth: u8, inner: bool, is24: bool) -> u8 {
        let pm = if is24 { inner } else { self.pm() };
        twelfth + if pm { 12 } else { 0 }
    }
    /// A tap on the dial at (`dx`, `dy`) from its centre; `inner` is whether it landed within the
    /// inner ring's reach.
    pub fn tap_dial(&mut self, dx: f32, dy: f32, inner: bool, is24: bool) {
        let v = snapped(turn(dx, dy), self.minutes);
        if self.minutes {
            self.minute = v;
        } else {
            self.hour = self.hour_at(v, inner, is24);
        }
        self.hand = None;
    }
    /// A drag over the dial: the value follows the nearest mark, the hand the point itself.
    pub fn drag_dial(&mut self, dx: f32, dy: f32, inner: bool, is24: bool) {
        let angle = turn(dx, dy);
        if self.minutes {
            self.minute = nearest(angle, 60);
        } else {
            self.hour = self.hour_at(nearest(angle, 12), inner, is24);
        }
        self.hand = Some(angle);
    }
    /// The hand's angle: the drag's, or the selected value's.
    pub fn angle(&self) -> f32 {
        self.hand.unwrap_or(if self.minutes {
            f32::from(self.minute) * TAU / 60.0
        } else {
            f32::from(self.hour % 12) * TAU / 12.0
        })
    }
    /// Whether the hand reaches only the inner ring: a 24-hour clock's afternoon hours.
    pub fn inner(&self, is24: bool) -> bool {
        is24 && self.pm() && !self.minutes
    }
    /// TimeInput's `LaunchedEffect(hour, minute)`: a change the fields did not type themselves
    /// puts both back on the time in two digits. True when it replaced them.
    pub fn settle(&mut self, is24: bool) -> bool {
        if !self.input || (self.hour, self.minute) == self.seen {
            return false;
        }
        let reset = self.user_override == Some(true);
        if reset {
            self.fields = self.texts(is24);
        }
        self.user_override = Some(true);
        self.seen = (self.hour, self.minute);
        reset
    }
    /// TimeInput's `timeInputOnChange` for field `j` (0 the hour, 1 the minute) given its new
    /// value: what the field keeps, or None when it keeps its old value. A number the field
    /// accepts sets the time, and a 12-hour clock's hour above 1 moves on to the minute.
    pub fn typed(&mut self, j: usize, value: &Editor, is24: bool) -> Option<Editor> {
        self.user_override = Some(false);
        if value.text == self.fields[j].text {
            return Some(value.clone());
        }
        if value.text.is_empty() {
            if j == 0 {
                self.hour = if self.pm() && !is24 { 12 } else { 0 };
            } else {
                self.minute = 0;
            }
            return Some(value.clone());
        }
        let text = String::from_utf16_lossy(&value.text);
        let n = if value.text.len() == 3 && value.anchor == 1 {
            text.chars().next()?.to_digit(10)?
        } else {
            kotlin_int(&text)?
        };
        let max = match j {
            1 => 59,
            _ if is24 => 23,
            _ => 12,
        };
        if n > max {
            return None;
        }
        let n = n as u8;
        if j == 0 {
            self.hour = match n {
                12 if self.pm() => 12,
                12 if !is24 => 0,
                _ => n + if self.pm() && !is24 { 12 } else { 0 },
            };
            if n > 1 && !is24 {
                self.minutes = true;
            }
        } else {
            self.minute = n;
        }
        if value.text.len() <= 2 {
            return Some(value.clone());
        }
        // `value.copy(text = text[0])`: the selection held to the one character left.
        Some(Editor {
            text: value.text[..1].to_vec(),
            anchor: value.anchor.min(1),
            focus: value.focus.min(1),
            composing: None,
            ..value.clone()
        })
    }
}
