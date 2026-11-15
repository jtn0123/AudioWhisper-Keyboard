//! Native settings UI. Platform effects and persisted IO are returned as plans.
pub mod catalogue;
mod dialog;
mod dialog_buttons;
mod dialog_metrics;
pub mod editor;
mod events;
mod field;
mod layout;
mod material;
mod scrollbar;
mod widgets;
use crate::panel_scroll::Fling;
use catalogue::{KEYBOARD, Kind};
use editor::Editor;
use keys::Rect;
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    None,
    Back,
    Route(String),
    Row(usize),
    Switch(usize),
    ModalSwitch,
    Choose(String),
    Confirm,
    Cancel,
    Default,
    Slider(usize),
    Editor,
    SelectionHandle(bool),
    Picker,
}
#[derive(Clone, Debug)]
pub struct Control {
    pub id: i32,
    pub bounds: Rect,
    pub label: String,
    pub summary: String,
    pub role: &'static str,
    pub enabled: bool,
    pub checked: Option<bool>,
    pub selected: bool,
    pub action: Action,
    pub range: Option<[f32; 3]>,
}
#[derive(Clone, Debug)]
pub struct Modal {
    pub row: usize,
    pub selected: String,
    pub values: Vec<String>,
    pub scroll: f32,
    pub enabled: bool,
}
#[derive(Debug)]
pub struct App {
    pub stack: Vec<String>,
    pub scrolls: [f32; 2],
    pub modal: Option<Modal>,
    pub prefs: BTreeMap<String, String>,
    pub strings: BTreeMap<String, String>,
    pub editor: Editor,
    pub config: [f32; 4],
    pub colors: [u32; 13],
    pub dialog_window_width: f32,
    pub sp_pixels: [f32; 33],
    pub controls: Vec<Control>,
    pub max_scroll: f32,
    pub clock: i64,
    pub scrollbar_start: [i64; 2],
    pub scrollbar_last: [i64; 2],
    pub scrollbar_exists: [bool; 2],
    pub focus: i32,
    pub accepted: bool,
    pub reveal_focus: bool,
    pub reveal_target: Option<f32>,
    pub editor_layout: Option<std::sync::Arc<crate::android_text::Layout>>,
    pub editor_origin: [f32; 2],
    pub toolbar_dismissed: bool,
    pub back_progress: f32,
    pub ime_enabled: bool,
    pub ime_selected: bool,
    pub effects: Vec<Vec<String>>,
    pub fling: Option<Fling>,
    pub down: Option<(i32, f32, f32, i64)>,
    pub handle_drag_offset: f32,
    pub handle_drag_other: usize,
    /// The pointer's position at the last event, as Compose's `previousPosition`.
    pub last: (f32, f32),
    pub dragging: bool,
    pub velocity: crate::compose_velocity::ComposeVelocity,
}
impl Default for App {
    fn default() -> Self {
        Self {
            stack: vec!["settings/home".into()],
            scrolls: [0.0; 2],
            modal: None,
            prefs: BTreeMap::new(),
            strings: BTreeMap::new(),
            editor: Editor::default(),
            config: [1.0, 1.0, 1.0, 1.0],
            colors: [
                0xfff8f8f8, 0xff161616, 0xff555555, 0xff376a20, 0xffdce9d0, 0xffe9e9e9, 0xffffffff,
                0xff777777, 0xfff8f8f8, 0xff555b66, 0xffc5c5c5, 0xffeeeeee, 0xffeeeeee,
            ],
            dialog_window_width: 0.0,
            sp_pixels: [0.0; 33],
            controls: vec![],
            max_scroll: 0.0,
            clock: 0,
            scrollbar_start: [-1; 2],
            scrollbar_last: [-1; 2],
            scrollbar_exists: [false; 2],
            focus: -1,
            accepted: true,
            reveal_focus: false,
            reveal_target: None,
            editor_layout: None,
            editor_origin: [0.0; 2],
            toolbar_dismissed: false,
            back_progress: 0.0,
            ime_enabled: true,
            ime_selected: true,
            effects: vec![],
            fling: None,
            down: None,
            handle_drag_offset: 0.0,
            handle_drag_other: 0,
            last: (0.0, 0.0),
            dragging: false,
            velocity: Default::default(),
        }
    }
}
impl App {
    pub fn value(&self, key: &str) -> String {
        self.prefs.get(key).cloned().unwrap_or_else(|| {
            settings::catalogue::get(&settings::text(key))
                .map_or(String::new(), |p| p.default.into())
        })
    }
    pub fn yes(&self, key: &str) -> bool {
        self.value(key) == "true"
    }
    pub fn label(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_else(|| key.into())
    }
    pub fn visible(&self, i: usize) -> bool {
        KEYBOARD[i].key != "keyboard__utility_key_action"
            || self.yes("keyboard__utility_key_enabled")
    }
    pub fn enabled(&self, i: usize) -> bool {
        match KEYBOARD[i].key {
            "keyboard__hinted_number_row_mode" => !self.yes("keyboard__number_row"),
            "keyboard__one_handed_mode_scale_factor" => {
                self.yes("keyboard__one_handed_mode_enabled")
            }
            _ => true,
        }
    }
    pub fn choices(i: usize) -> Vec<String> {
        let p = settings::catalogue::get(&settings::text(KEYBOARD[i].key))
            .expect("static settings key");
        if p.key.starts_with("keyboard__hinted_") {
            return ["ACCENT_PRIORITY", "HINT_PRIORITY", "SMART_PRIORITY"]
                .map(String::from)
                .into();
        }
        p.choices
            .split('|')
            .filter(|s| *s != "DISABLED")
            .map(String::from)
            .collect()
    }
    pub fn choice_label(&self, i: usize, choice: &str) -> String {
        let Kind::Choice(prefix) = KEYBOARD[i].kind else {
            return String::new();
        };
        self.label(&format!("enum__{prefix}__{}", choice.to_ascii_lowercase()))
    }
    pub fn navigate(&mut self, route: &str) {
        self.fling = None;
        if self.editor.focused {
            self.editor.focused = false;
            self.editor.composing = None;
            self.effects.push(vec!["hideEditor".into()]);
        }
        if matches!(route, "settings/home" | "settings/keyboard") {
            if route == "settings/home" {
                self.stack = vec![route.into()];
            } else if let Some(at) = self.stack.iter().position(|r| r == route) {
                self.stack.truncate(at + 1);
            } else {
                // A pushed destination is a new back-stack entry: Compose gives it a fresh
                // ScrollState, so it opens at the top whatever an earlier visit left.
                self.stack.push(route.into());
                self.scrolls[self.screen_index()] = 0.0;
            }
            self.modal = None;
            self.scrollbar_start[0] = -1;
            self.scrollbar_last[0] = -1;
            self.focus = -1;
        } else if route == "@enable" {
            self.effects.push(vec!["enable".into()]);
        } else {
            self.effects.push(vec!["compose".into(), route.into()]);
        }
    }
    pub fn screen_index(&self) -> usize {
        usize::from(self.stack.last().is_some_and(|r| r == "settings/keyboard"))
    }
    pub fn scroll(&self) -> f32 {
        self.modal
            .as_ref()
            .map_or(self.scrolls[self.screen_index()], |m| m.scroll)
    }
    /// Where content sits: Compose's ScrollState and LazyListState move layout by whole pixels
    /// and carry the remainder, so they show round(position).
    pub fn shown_scroll(&self) -> f32 {
        shown(self.scroll())
    }
    pub fn set_scroll(&mut self, value: f32) {
        let v = value.clamp(0.0, self.max_scroll);
        if v != self.scroll() {
            self.scrollbar_last[usize::from(self.modal.is_some())] = -1;
        }
        if let Some(m) = &mut self.modal {
            m.scroll = v;
        } else {
            let i = self.screen_index();
            self.scrolls[i] = v;
        }
    }
    pub fn write(&mut self, key: &str, value: String) {
        self.prefs.insert(key.into(), value.clone());
        self.effects.push(vec!["write".into(), key.into(), value]);
    }
    pub fn reset(&mut self, key: &str) {
        let value = settings::catalogue::get(&settings::text(key))
            .map_or(String::new(), |p| p.default.into());
        self.prefs.insert(key.into(), value);
        self.effects.push(vec!["reset".into(), key.into()]);
    }
    pub fn back(&mut self) {
        self.fling = None;
        self.back_progress = 0.0;
        if self.modal.take().is_some() {
            self.focus = -1;
        } else if self.editor.focused {
            self.editor.focused = false;
            self.editor.composing = None;
            self.effects.push(vec!["hideEditor".into()]);
        } else if self.stack.len() > 1 {
            self.stack.pop();
            self.focus = -1;
        } else {
            self.effects.push(vec!["finish".into()]);
        }
    }
    pub fn tick(&mut self, now: i64) -> bool {
        for time in self
            .scrollbar_start
            .iter_mut()
            .chain(self.scrollbar_last.iter_mut())
        {
            if *time < 0 {
                *time = now;
            }
        }
        self.clock = now;
        if let Some(f) = &self.fling {
            let value = f.value(now);
            let done = now >= f.start + f.duration || value <= 0.0 || value >= self.max_scroll;
            self.set_scroll(value);
            if done {
                self.fling = None;
            }
        }
        self.animating()
    }
}
pub(super) fn shown(scroll: f32) -> f32 {
    (scroll + 0.5).floor()
}
