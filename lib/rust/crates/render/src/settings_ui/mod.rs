//! Native settings UI. Platform effects and persisted IO are returned as plans.
mod cards;
pub mod catalogue;
mod dialog;
mod dialog_buttons;
mod dialog_metrics;
pub mod editor;
mod events;
pub use cards::SpellCard;
mod field;
mod fields;
mod layout;
pub mod licences;
mod material;
mod overscroll;
mod rows;
pub use rows::CODE_KEYS;
mod scrollbar;
mod widgets;
use crate::panel_scroll::Fling;
use editor::{Batches, Editor};
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
    /// The text field row `i` draws (DictationScreen's vocabulary).
    Field(usize),
    SelectionHandle(bool),
    Picker,
    /// A library row (its index in the sorted list) opens its license dialog.
    Library(usize),
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
impl Control {
    /// Whether accessibility reports the control enabled. `enabled` is how it is drawn. A
    /// passive row (`text`) is drawn faded when its condition fails, but JetPref's Preference
    /// without onClick fades by alpha alone: Material's ListItem node carries no disabled state,
    /// so TalkBack reads it as plain text either way.
    pub fn exposed_enabled(&self) -> bool {
        self.enabled || self.role == "text"
    }
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
    /// Each back-stack entry's own ScrollState, parallel to `stack`.
    pub scrolls: Vec<f32>,
    pub modal: Option<Modal>,
    pub prefs: BTreeMap<String, String>,
    pub strings: BTreeMap<String, String>,
    /// Device facts Kotlin reads for the screens (settings op 20): the app version, whether a
    /// physical keyboard is attached, the system spell checker. Replaced whole on each report.
    pub facts: BTreeMap<String, String>,
    /// Pictures Kotlin rasterises at their intrinsic size (the app icon, the spell checker's).
    pub images: BTreeMap<String, crate::media_cache::Frame>,
    /// The editor of the field being edited: the preview field's unless `field` names a row's.
    pub editor: Editor,
    /// The row whose text field owns `editor`; None while the preview field does.
    pub field: Option<usize>,
    /// The preview field's editor while a row's field owns `editor`.
    pub parked: Editor,
    /// Each painted field's text layout and origin, for placing the caret where a tap lands.
    pub field_layouts: Vec<(i32, std::sync::Arc<crate::android_text::Layout>, [f32; 2])>,
    pub config: [f32; 4],
    /// The palette; 13 is M3 Card's content colour, `contentColorFor(surfaceContainerHighest)`,
    /// 14 onPrimaryContainer and 15 onBackground.
    pub colors: [u32; 16],
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
    /// The keyboard's open batch edits on the field.
    pub batches: Batches,
    pub back_progress: f32,
    pub ime_enabled: bool,
    pub ime_selected: bool,
    /// `hasVibrator()` and `hasAmplitudeControl()`.
    pub vibrator: [bool; 2],
    pub effects: Vec<Vec<String>>,
    pub fling: Option<Fling>,
    pub stretch: overscroll::Stretch,
    pub down: Option<(i32, f32, f32, i64)>,
    pub handle_drag_offset: f32,
    pub handle_drag_other: usize,
    /// The pointer's position at the last event, as Compose's `previousPosition`.
    pub last: (f32, f32),
    pub dragging: bool,
    pub velocity: crate::compose_velocity::ComposeVelocity,
    pub licences: licences::State,
}
impl Default for App {
    fn default() -> Self {
        Self {
            stack: vec!["settings/home".into()],
            scrolls: vec![0.0],
            modal: None,
            prefs: BTreeMap::new(),
            strings: BTreeMap::new(),
            facts: BTreeMap::new(),
            images: BTreeMap::new(),
            editor: Editor::default(),
            field: None,
            parked: Editor::default(),
            field_layouts: vec![],
            config: [1.0, 1.0, 1.0, 1.0],
            colors: [
                0xfff8f8f8, 0xff161616, 0xff555555, 0xff376a20, 0xffdce9d0, 0xffe9e9e9, 0xffffffff,
                0xff777777, 0xfff8f8f8, 0xff555b66, 0xffc5c5c5, 0xffeeeeee, 0xffeeeeee, 0xff161616,
                0xff0b2000, 0xff161616,
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
            batches: Batches::default(),
            back_progress: 0.0,
            ime_enabled: true,
            ime_selected: true,
            vibrator: [false; 2],
            effects: vec![],
            fling: None,
            stretch: overscroll::Stretch::default(),
            down: None,
            handle_drag_offset: 0.0,
            handle_drag_other: 0,
            last: (0.0, 0.0),
            dragging: false,
            velocity: Default::default(),
            licences: Default::default(),
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
    /// A device fact, or empty when Kotlin reported none (a null setting).
    pub fn fact(&self, key: &str) -> &str {
        self.facts.get(key).map_or("", String::as_str)
    }
    pub fn navigate(&mut self, route: &str) {
        self.fling = None;
        self.end_scroll_cycle();
        if self.editor.focused {
            self.editor.focused = false;
            self.editor.composing = None;
            self.effects.push(vec!["hideEditor".into()]);
        }
        if catalogue::native(route) {
            self.release_field();
            if route == licences::ROUTE {
                // A new LazyListState: `visibleItems` is remembered again at the first frame.
                self.licences.visible_items = None;
                self.licences.first = None;
                // A load lost with an earlier view is asked for again.
                self.licences.requested = false;
            }
            if route == catalogue::HOME_ROUTE {
                self.stack = vec![route.into()];
                self.scrolls.truncate(1);
            } else if let Some(at) = self.stack.iter().position(|r| r == route) {
                self.stack.truncate(at + 1);
                self.scrolls.truncate(at + 1);
            } else {
                // A pushed destination is a new back-stack entry: Compose gives it a fresh
                // ScrollState, so it opens at the top whatever an earlier visit left.
                self.stack.push(route.into());
                self.scrolls.push(0.0);
            }
            self.modal = None;
            self.scrollbar_start[0] = -1;
            self.scrollbar_last[0] = -1;
            self.focus = -1;
            self.check_models();
        } else if route == "@enable" {
            self.effects.push(vec!["enable".into()]);
        } else {
            // The fallback hands back any destination Rust draws, so it is told which.
            self.effects.push(vec![
                "compose".into(),
                route.into(),
                catalogue::native_routes().join("|"),
            ]);
        }
    }
    /// The top back-stack entry's own scroll, whatever dialog is over it.
    pub fn screen_scroll(&self) -> f32 {
        self.scrolls.last().copied().unwrap_or(0.0)
    }
    pub fn scroll(&self) -> f32 {
        self.modal
            .as_ref()
            .map_or(self.screen_scroll(), |m| m.scroll)
    }
    /// Where content sits: Compose's ScrollState and LazyListState move layout by whole pixels
    /// and carry the remainder, so they show round(position).
    pub fn shown_scroll(&self) -> f32 {
        shown(self.scroll())
    }
    /// ScrollState's `canScrollBackward` and `canScrollForward`, over the whole-pixel position
    /// it shows: a fling that stops a fraction short of the end shows the end, so it is there.
    pub fn can_scroll(&self) -> [bool; 2] {
        let at = self.shown_scroll();
        [at > 0.0, at < shown(self.max_scroll)]
    }
    pub fn set_scroll(&mut self, value: f32) {
        let v = value.clamp(0.0, self.max_scroll);
        if v != self.scroll() {
            self.scrollbar_last[usize::from(self.modal.is_some())] = -1;
        }
        if let Some(m) = &mut self.modal {
            m.scroll = v;
        } else if let Some(scroll) = self.scrolls.last_mut() {
            *scroll = v;
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
        self.end_scroll_cycle();
        self.back_progress = 0.0;
        if self.modal.take().is_some() {
            self.focus = -1;
        } else if self.editor.focused {
            self.editor.focused = false;
            self.editor.composing = None;
            self.effects.push(vec!["hideEditor".into()]);
        } else if self.stack.len() > 1 {
            self.release_field();
            self.stack.pop();
            self.scrolls.pop();
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
        if let Some(f) = self.fling.take() {
            let value = self.fling_value(&f, now);
            // Only the edge the fling moves toward stops it: one that starts at the bottom
            // and runs backward (relaxing a stretch first) carries on.
            let done = now >= f.start + f.duration
                || (f.distance < 0.0 && value <= 0.0)
                || (f.distance > 0.0 && value >= self.max_scroll);
            self.set_scroll(value);
            if done {
                self.end_scroll_cycle();
            } else {
                self.fling = Some(f);
            }
        }
        self.animating()
    }
}
pub(super) fn shown(scroll: f32) -> f32 {
    (scroll + 0.5).floor()
}
