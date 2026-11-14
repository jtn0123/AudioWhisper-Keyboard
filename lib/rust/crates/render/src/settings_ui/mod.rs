//! Native settings UI. Platform effects and persisted IO are returned as plans.
pub mod catalogue;
mod dialog;
pub mod editor;
mod events;
mod field;
mod layout;
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
    pub colors: [u32; 6],
    pub controls: Vec<Control>,
    pub max_scroll: f32,
    pub focus: i32,
    pub accepted: bool,
    pub reveal_focus: bool,
    pub reveal_target: Option<f32>,
    pub editor_layout: Option<parley::Layout<()>>,
    pub editor_origin: [f32; 2],
    pub back_progress: f32,
    pub ime_enabled: bool,
    pub ime_selected: bool,
    pub effects: Vec<Vec<String>>,
    pub fling: Option<Fling>,
    pub down: Option<(i32, f32, f32, i64)>,
    pub last: (f32, i64),
    pub dragging: bool,
    pub velocity: f32,
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
                0xfff8f8f8, 0xff161616, 0xff555555, 0xff376a20, 0xffdce9d0, 0xffe9e9e9,
            ],
            controls: vec![],
            max_scroll: 0.0,
            focus: -1,
            accepted: true,
            reveal_focus: false,
            reveal_target: None,
            editor_layout: None,
            editor_origin: [0.0; 2],
            back_progress: 0.0,
            ime_enabled: true,
            ime_selected: true,
            effects: vec![],
            fling: None,
            down: None,
            last: (0.0, 0),
            dragging: false,
            velocity: 0.0,
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
                self.stack.push(route.into());
            }
            self.modal = None;
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
    pub fn set_scroll(&mut self, value: f32) {
        let v = value.clamp(0.0, self.max_scroll);
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
        if let Some(f) = &self.fling {
            let value = f.value(now);
            let done = now >= f.start + f.duration || value <= 0.0 || value >= self.max_scroll;
            self.set_scroll(value);
            if done {
                self.fling = None;
            }
        }
        self.fling.is_some()
    }
}
