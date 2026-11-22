//! The subtype editor's painting, as SubtypeEditorScreen laid it out: an 8 dp column of
//! SubtypeProperty fields (a titleSmall title over a 56 dp outlined dropdown), 32 dp gaps between
//! groups, Add subtype's suggested presets card on top, and Delete in the top bar when editing.
//! Material's Button and TextButton, which the other language screens use too, are here.
use super::Tap;
use crate::settings_ui::{
    Action, App,
    cards::{BODY_BOLD, LABEL_LARGE, TITLE_SMALL, plain},
    catalogue::languages::Field,
    items::Entry,
    words::Tap as WordTap,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// The top bar's Delete, the presets card's title, its message when no preset is suggested,
/// Show all, and the first suggested preset.
pub const DELETE_ID: i32 = 7904;
pub const NONE_ID: i32 = 7907;
pub const CARD_ID: i32 = 7908;
pub const ALL_ID: i32 = 7909;
pub const SUGGESTION_ID: i32 = 7910;
/// No more suggested presets than fit between [`SUGGESTION_ID`] and the list's ids.
pub const MOST: usize = 80;

/// A Material button: its label, an 18 dp icon before it or none, filled or text, its ink.
pub(in crate::settings_ui) struct Button {
    pub id: i32,
    pub label: String,
    pub icon: &'static str,
    pub filled: bool,
    pub ink: u32,
    pub action: Action,
}

impl App {
    /// Row `i`, field `f`: SubtypeProperty's 8 dp padding round its title and dropdown, inside
    /// the column's 8 dp; its height.
    pub(in crate::settings_ui) fn pick_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        f: Field,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let title = self.label(self.rows()[i].label);
        let layout = self.styled(text, &title, TITLE_SMALL, Some(width));
        let th = layout.height.ceil();
        let at = rect(left, y + self.px(8.0), width, th);
        self.paint_styled(p, &title, at, TITLE_SMALL, self.colors[1], clip);
        let b = rect(left, at.bottom + self.px(8.0), width, self.px(56.0));
        let id = 1000 + i as i32;
        let error = self.draft.as_ref().is_some_and(|d| d.checked && d.unset(f));
        let (stroke, edge) = if self.focus == id {
            (self.px(2.0), self.colors[3])
        } else if error {
            (self.px(2.0), self.colors[18])
        } else {
            (self.px(1.0), self.colors[7])
        };
        let radius = self.px(8.0);
        self.paint_clipped_box(p, b, edge, radius, clip);
        let inside = rect(
            b.left + stroke,
            b.top + stroke,
            b.width() - 2.0 * stroke,
            b.height() - 2.0 * stroke,
        );
        self.paint_clipped_box(p, inside, self.colors[0], radius - stroke, clip);
        let value = self.pick_text(f);
        let size = self.px(24.0);
        let room = (b.width() - self.px(16.0) - self.px(12.0) - size - self.px(8.0)).max(1.0);
        let shown = self.row_text(text, &value, self.sp(16), room, 1);
        let vh = self.measure(text, &shown, self.sp(16), room).ceil();
        let at = rect(
            b.left + self.px(16.0),
            b.top + ((b.height() - vh) / 2.0).round(),
            room,
            vh,
        );
        self.paint_text(p, &shown, at, self.sp(16), self.colors[1], clip);
        let icon = rect(
            b.right - self.px(12.0) - size,
            b.top + ((b.height() - size) / 2.0).round(),
            size,
            size,
        );
        let ink = self.colors[if error { 18 } else { 2 }];
        self.paint_icon(p, "arrow_drop_down", icon, ink, clip);
        let mut c = plain(id, b, title, "button");
        c.summary = value;
        c.action = Action::Row(i);
        self.push_control(c, clip);
        let last = f == Field::Layout(super::codec::LAYOUTS.len() - 1);
        b.bottom + self.px(if last { 16.0 } else { 8.0 }) - y
    }
    /// The column's 8 dp top padding, and when adding, the suggested presets card under it.
    pub(in crate::settings_ui) fn presets_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let pad = self.px(8.0);
        if !self.draft.as_ref().is_some_and(|d| d.adding) {
            return pad;
        }
        let card_x = x + pad;
        let card_w = self.config[0] - x - 2.0 * pad;
        let right = self.config[0] - card_x - card_w;
        let top = y + pad;
        // The card goes under what is painted on it; its height is known at the end.
        let under = p.items.len();
        let card = rect(card_x, top, card_w, 0.0);
        self.paint_clipped_box(p, card, self.colors[5], self.px(12.0), clip);
        let mut at = top + pad;
        let r = self.rows()[i];
        let title = self.label(r.label);
        let width = (card_w - 2.0 * self.px(16.0)).max(1.0);
        let shown = self.row_text(text, &title, self.sp(16), width, 1);
        let th = self.styled(text, &shown, BODY_BOLD, None).height.ceil();
        let title_at = rect(card_x + self.px(16.0), at + pad, width, th);
        self.paint_styled(p, &shown, title_at, BODY_BOLD, self.colors[3], clip);
        self.push_control(plain(CARD_ID, title_at, title, "heading"), clip);
        at = title_at.bottom + pad;
        let suggested = self.suggested_presets();
        for (k, index) in suggested.iter().copied().enumerate().take(MOST) {
            let (name, layout) = self.preset_text(index);
            let entry = Entry {
                id: SUGGESTION_ID + k as i32,
                title: name,
                summary: layout,
                icon: "",
                ink: [Some(self.colors[13]), None],
                buttons: 0,
                right,
                enabled: true,
                action: Action::Lang(Tap::Preset(index)),
            };
            at += self.paint_entry(p, entry, (card_x, at), clip, text).0;
        }
        if suggested.is_empty() {
            let none = self.label(r.summary);
            let nh = self.measure(text, &none, self.sp(16), width).ceil();
            let b = rect(card_x + self.px(16.0), at + pad, width, nh);
            self.paint_text(p, &none, b, self.sp(16), self.colors[13], clip);
            self.push_control(plain(NONE_ID, b, none, "text"), clip);
            at = b.bottom + pad;
        }
        let all = Button {
            id: ALL_ID,
            label: self.label("settings__localization__subtype_presets_view_all"),
            icon: "",
            filled: true,
            ink: self.colors[6],
            action: Action::Lang(Tap::ShowAll),
        };
        let bw = self.button_width(text, &all);
        let touch = rect(card_x + card_w - self.px(16.0) - bw, at, bw, self.px(48.0));
        self.material_button(p, all, touch, clip, text);
        let card = rect(card_x, top, card_w, touch.bottom + pad - top);
        p.items[under].bounds = card;
        card.bottom + self.px(16.0) - y
    }
    /// The presets whose locale is one of the system's, in the system's order.
    pub fn suggested_presets(&self) -> Vec<usize> {
        let Some(r) = self.languages.resources.as_ref() else {
            return vec![];
        };
        r.system
            .iter()
            .filter_map(|s| r.presets.iter().position(|p| p.base == s.base))
            .collect()
    }
    /// A preset's language, as the names setting asks, and its characters layout's label, or
    /// its component name when the keyboard has no such layout.
    pub fn preset_text(&self, index: usize) -> (String, String) {
        let Some(preset) = self
            .languages
            .resources
            .as_ref()
            .and_then(|r| r.presets.get(index))
        else {
            return Default::default();
        };
        let id = super::field_text(&preset.value, &["layoutMap", "characters"]);
        let component = id.rsplit(':').next().unwrap_or_default().to_owned();
        let layout = self.layout_label("characters", &id).unwrap_or(component);
        (self.locale_name(&preset.tag), layout)
    }
    /// The top bar's Delete, when editing a stored subtype or word: a 48 dp IconButton 4 dp from
    /// the end.
    pub(in crate::settings_ui) fn header_delete(&self) -> bool {
        let subtype = self.draft.as_ref().is_some_and(|d| !d.adding)
            && self
                .stack
                .last()
                .is_some_and(|r| super::edit_id(r).is_some());
        subtype || self.word_to_delete().is_some() || self.theme_header().is_some()
    }
    pub(in crate::settings_ui) fn paint_header_delete(&mut self, p: &mut Panel, header: f32) {
        if !self.header_delete() {
            return;
        }
        let size = self.px(48.0);
        let w = self.config[0];
        let b = rect(
            w - self.px(4.0) - size,
            ((header.min(self.px(64.0)) - size) / 2.0).round(),
            size,
            size,
        );
        let window = rect(0.0, 0.0, w, self.config[1]);
        // The theme editor's Save or Delete, else the subtype's or word's Delete.
        let (id, icon_name, key, summary, action) = match self.theme_header() {
            Some(header) => header,
            None => match self.word_to_delete() {
                Some(w) => (
                    DELETE_ID,
                    "delete",
                    "action__delete",
                    w.word.clone(),
                    Action::Words(WordTap::Delete),
                ),
                None => (
                    DELETE_ID,
                    "delete",
                    "action__delete",
                    self.deleted_name(),
                    Action::Lang(Tap::DeleteSubtype),
                ),
            },
        };
        if self.focus == id {
            self.paint_clipped_box(p, b, self.colors[4], size / 2.0, window);
        }
        let icon = self.px(24.0);
        let at = rect(b.left + self.px(12.0), b.top + self.px(12.0), icon, icon);
        self.paint_icon(p, icon_name, at, self.colors[2], window);
        let mut c = plain(id, b, self.label(key), "button");
        (c.summary, c.action) = (summary, action);
        self.push_control(c, window);
    }
    /// The name of the subtype the editor would delete.
    fn deleted_name(&self) -> String {
        let locale = self.draft.as_ref().map(|d| d.get(Field::Locale));
        self.locale_name(&locale.unwrap_or_default())
    }
    /// How wide `b` is: its label in labelLarge, the icon and 8 dp, in 24 dp padding filled or
    /// 12 dp as text, at least 58 dp.
    pub(in crate::settings_ui) fn button_width(&self, text: &mut Text, b: &Button) -> f32 {
        let lw = self.styled(text, &b.label, LABEL_LARGE, None).width.ceil();
        let icon = if b.icon.is_empty() {
            0.0
        } else {
            self.px(18.0) + self.px(8.0)
        };
        let pad = self.px(if b.filled { 24.0 } else { 12.0 });
        (lw + icon + 2.0 * pad).max(self.px(58.0))
    }
    /// Paints `b` in its 48 dp `touch` area, its 40 dp container centred, and records it.
    pub(in crate::settings_ui) fn material_button(
        &mut self,
        p: &mut Panel,
        b: Button,
        touch: Rect,
        clip: Rect,
        text: &mut Text,
    ) {
        let container = rect(
            touch.left,
            touch.top + self.px(4.0),
            touch.width(),
            self.px(40.0),
        );
        let radius = container.height() / 2.0;
        if self.focus == b.id {
            let ring = if b.filled { self.px(3.0) } else { 0.0 };
            let around = rect(
                container.left - ring,
                container.top - ring,
                container.width() + 2.0 * ring,
                container.height() + 2.0 * ring,
            );
            self.paint_clipped_box(p, around, self.colors[4], radius + ring, clip);
        }
        if b.filled {
            self.paint_clipped_box(p, container, self.colors[3], radius, clip);
        }
        let layout = self.styled(text, &b.label, LABEL_LARGE, None);
        let (lw, lh) = (layout.width.ceil(), layout.height.ceil());
        let icon = if b.icon.is_empty() {
            0.0
        } else {
            self.px(18.0) + self.px(8.0)
        };
        let start = container.left + ((container.width() - icon - lw) / 2.0).round();
        if !b.icon.is_empty() {
            let size = self.px(18.0);
            let glyph = rect(
                start,
                container.top + ((container.height() - size) / 2.0).round(),
                size,
                size,
            );
            self.paint_icon(p, b.icon, glyph, b.ink, clip);
        }
        let at = rect(
            start + icon,
            container.top + ((container.height() - lh) / 2.0).round(),
            lw + 1.0,
            lh,
        );
        self.paint_styled(p, &b.label, at, LABEL_LARGE, b.ink, clip);
        let mut c = plain(b.id, touch, b.label, "button");
        c.action = b.action;
        self.push_control(c, clip);
    }
}
