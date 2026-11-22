//! What the user dictionary screens paint beyond the editable list: the languages, the action
//! rows that UserDictionaryScreen kept in its overflow menu and floating button, the word list's
//! search, and the editor's fields, each a title over a 56 dp outlined field like the subtype
//! editor's, with the dialog's error under it.
use super::Tap;
use crate::settings_ui::{
    Action, App, Control,
    cards::{TITLE_SMALL, plain},
    catalogue::words::{Act, Part},
    items::{EMPTY_ID, Entry},
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// The control id of the `k`th language on the first screen.
pub const GROUP_ID: i32 = 60000;
/// The control id of field `k`'s error message.
pub const ERROR_ID: i32 = 7940;
/// The action rows' icons: add a word, import, export and the system's manager.
const ACTION_ICONS: [&str; 4] = ["add", "input", "archive", "open_in_new"];

impl App {
    /// Paints row `i`, part `part`; its height.
    pub(in crate::settings_ui) fn words_part(
        &mut self,
        p: &mut Panel,
        i: usize,
        part: Part,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        match part {
            Part::Languages => self.groups_row(p, i, at, clip, text),
            Part::Action(a) => self.action_row(p, i, a, at, clip, text),
            Part::Search if self.group_has_words() => self.search_row(p, i, at, clip, text),
            Part::NotFound if self.group_has_words() && self.shown_words().is_empty() => {
                let term = self.words.search.clone();
                self.not_found(p, i, &term, at, clip, text)
            }
            Part::Field(k) => self.word_field(p, i, k, at, clip, text),
            Part::Search | Part::NotFound => 0.0,
        }
    }
    /// Row `i`'s place in the focus order. The first field stands for the top bar's Delete
    /// when editing, so it follows Back.
    pub(in crate::settings_ui) fn words_focus(&self, i: usize, part: Part) -> Vec<i32> {
        let id = 1000 + i as i32;
        match part {
            Part::Languages if self.word_groups().is_empty() => vec![EMPTY_ID],
            Part::Languages => (0..self.word_groups().len() as i32)
                .map(|k| GROUP_ID + k)
                .collect(),
            Part::Action(_) => vec![id],
            Part::Search if self.group_has_words() => vec![id],
            Part::NotFound if self.group_has_words() && self.shown_words().is_empty() => vec![id],
            Part::Field(0) if self.word_to_delete().is_some() => {
                vec![crate::settings_ui::languages::form::DELETE_ID, id]
            }
            Part::Field(k) if !self.word_error(k).is_empty() => vec![id, ERROR_ID + k as i32],
            Part::Field(_) => vec![id],
            _ => vec![],
        }
    }
    /// The languages words are stored under, or the card saying there are none.
    fn groups_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let groups = self.word_groups();
        if groups.is_empty() {
            let label = self.label(self.rows()[i].label);
            return self.warning_card(p, EMPTY_ID, label, (x, y), clip, text);
        }
        let mut at = y;
        for (k, g) in groups.into_iter().enumerate() {
            let entry = Entry {
                id: GROUP_ID + k as i32,
                title: g.label,
                summary: String::new(),
                icon: "language",
                ink: [None, None],
                buttons: 0,
                right: 0.0,
                enabled: true,
                action: Action::Words(Tap::Group(g.key)),
            };
            at += self.paint_entry(p, entry, (x, at), clip, text).0;
        }
        at - y
    }
    /// Whether action `a` can run: the dictionary is on, no file work runs, and Export has words.
    fn action_enabled(&self, a: Act) -> bool {
        let Some(r) = self.word_report() else {
            return false;
        };
        match a {
            Act::Add | Act::Manager => r.on,
            Act::Import => r.on && self.files.busy.is_none(),
            Act::Export => r.on && self.files.busy.is_none() && !r.words.is_empty(),
        }
    }
    fn action_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        a: Act,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let k = [Act::Add, Act::Import, Act::Export, Act::Manager]
            .iter()
            .position(|x| *x == a)
            .unwrap_or_default();
        // Add stands out in primary, as the list's own Add does.
        let ink = (a == Act::Add).then_some(self.colors[3]);
        let entry = Entry {
            id: 1000 + i as i32,
            title: self.label(self.rows()[i].label),
            summary: String::new(),
            icon: ACTION_ICONS[k],
            ink: [ink, None],
            buttons: 0,
            right: 0.0,
            enabled: self.action_enabled(a),
            action: Action::Words(Tap::Action(a)),
        };
        self.paint_entry(p, entry, at, clip, text).0
    }
    /// Field `k`: its title in titleSmall, a 56 dp outlined single-line field, and its error
    /// in bodyMedium and the error colour once a save found it invalid; its height.
    fn word_field(
        &mut self,
        p: &mut Panel,
        i: usize,
        k: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let [min, max] = super::editor::FREQUENCY_RANGE;
        let title = self
            .label(self.rows()[i].label)
            .replace("{f_min}", &min.to_string())
            .replace("{f_max}", &max.to_string());
        let th = self
            .styled(text, &title, TITLE_SMALL, Some(width))
            .height
            .ceil();
        let at = rect(left, y + self.px(16.0), width, th);
        self.paint_styled(p, &title, at, TITLE_SMALL, self.colors[1], clip);
        let b = rect(left, at.bottom + self.px(8.0), width, self.px(56.0));
        let id = 1000 + i as i32;
        let error = self.word_error(k);
        let (_, active) = self.field_editor(id);
        let (stroke, edge) = if self.focus == id || (active && self.editor.focused) {
            (
                self.px(2.0),
                self.colors[if error.is_empty() { 3 } else { 18 }],
            )
        } else if !error.is_empty() {
            (self.px(2.0), self.colors[18])
        } else {
            (self.px(1.0), self.colors[7])
        };
        let radius = self.px(4.0);
        self.paint_clipped_box(p, b, edge, radius, clip);
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
            label: title,
            summary: error.clone(),
            role: "edit",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Field(i),
            range: None,
        };
        // The title is above the field, so an empty field shows no placeholder.
        let mut drawn = field.clone();
        drawn.label = String::new();
        self.paint_editor(p, &drawn, text, clip);
        self.push_control(field, clip);
        let mut bottom = b.bottom;
        if !error.is_empty() {
            let eh = self.measure(text, &error, self.sp(14), width).ceil();
            let at = rect(left, b.bottom + self.px(4.0), width, eh);
            self.paint_text(p, &error, at, self.sp(14), self.colors[18], clip);
            self.push_control(plain(ERROR_ID + k as i32, at, error, "text"), clip);
            bottom = at.bottom;
        }
        bottom + self.px(if k == 3 { 16.0 } else { 0.0 }) - y
    }
}
