//! An editable list drawn from data, for any screen whose rows are records the user adds, opens,
//! reorders and deletes: Languages & Layouts' subtypes and the user dictionary's words. A
//! [`Source`] names where the items come from; this module paints them and turns taps into
//! [`Op`]s, which the source carries out (`languages/list.rs`, `words/list.rs`).
//!
//! Each item is a ListItem that opens the item's editor, then three 48 dp icon buttons: move up,
//! move down and delete, or only delete for a list kept in its own order ([`Words::reorder`]).
//! The buttons are the accessible way to reorder, and the item's node offers the same as custom
//! actions, so TalkBack's actions menu moves an item without hunting for the buttons. An item keeps its node ids while it moves (`State::slots`), so
//! accessibility focus stays on it. After a delete, an undo bar stays under the list until the
//! user leaves the screen or changes the list again: there is no timer to beat.
use super::{Action, App, cards::plain};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// Where an editable list's items come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The keyboard's subtypes, stored in `localization__subtypes`.
    Subtypes,
    /// A user dictionary's words in one language, stored by Kotlin.
    Words,
}

/// What a tap on an editable list asks its source to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    /// Open item `i`'s editor.
    Open(usize),
    Up(usize),
    Down(usize),
    Delete(usize),
    /// Open the editor for a new item.
    Add,
    /// Put back the item deleted last.
    Undo,
}

/// One item as the list shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// What identifies the item while it moves: a subtype's id.
    pub key: String,
    pub title: String,
    pub summary: String,
}

/// A delete the user can still undo: the list, where the item stood, and its stored value.
#[derive(Clone, Debug, PartialEq)]
pub struct Deleted {
    pub source: Source,
    pub index: usize,
    pub value: String,
    pub title: String,
}

/// The lists' state: the delete to undo, and the item keys in the order they first showed,
/// whose positions are the items' stable node ids.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct State {
    pub deleted: Option<Deleted>,
    pub slots: Vec<String>,
}

/// What a list says when it is empty and what its Add row says, as string keys, and whether
/// its items can be moved.
#[derive(Clone, Copy, Debug)]
pub struct Words {
    pub empty: &'static str,
    pub add: &'static str,
    /// Whether Move up and Move down show: false for a list kept in its own order.
    pub reorder: bool,
}

/// The control id of the Add row.
pub const ADD_ID: i32 = 7900;
/// The control id of the undo bar's button; its message is [`UNDO_ID`] + 1.
pub const UNDO_ID: i32 = 7901;
/// The control id of the empty list's card.
pub const EMPTY_ID: i32 = 7903;
/// The controls of the item in slot `s` start at `ITEM_ID + 4 * s`: the item, then up, down and
/// delete.
pub const ITEM_ID: i32 = 8000;
/// The string keys the list shows, whatever its source.
pub const KEYS: [&str; 5] = [
    "settings__list__move_up",
    "settings__list__move_down",
    "action__delete",
    "settings__list__deleted",
    "action__undo",
];
/// The icons of an item's buttons, in order after the item itself.
const BUTTON_ICONS: [&str; 3] = ["arrow_upward", "arrow_downward", "delete"];

/// The control id of the item in `slot`'s `part`: 0 the item, 1 up, 2 down, 3 delete.
pub fn item_id(slot: usize, part: usize) -> i32 {
    ITEM_ID + 4 * slot as i32 + part as i32
}

/// The order after moving the item at `from` one place up or down; None past either end.
pub fn moved(len: usize, from: usize, up: bool) -> Option<Vec<usize>> {
    let to = if up { from.checked_sub(1)? } else { from + 1 };
    if from >= len || to >= len {
        return None;
    }
    let mut order: Vec<usize> = (0..len).collect();
    order.swap(from, to);
    Some(order)
}

/// A ListItem the list and the language screens paint: title, summary and where it sits.
pub(super) struct Entry {
    pub id: i32,
    pub title: String,
    pub summary: String,
    /// A leading icon, or empty for the 16 dp inset.
    pub icon: &'static str,
    /// The title's and summary's colours; onSurface and onSurfaceVariant when None.
    pub ink: [Option<u32>; 2],
    /// How many 48 dp buttons sit at the end.
    pub buttons: usize,
    /// How far its end stays from the screen's: a card's or box's inset.
    pub right: f32,
    pub enabled: bool,
    pub action: Action,
}

impl App {
    /// The words of `source`'s list.
    fn words(&self, source: Source) -> Words {
        match source {
            Source::Subtypes => super::languages::list::WORDS,
            Source::Words => super::words::list::WORDS,
        }
    }
    /// The items of `source`, in stored order.
    pub fn list_items(&self, source: Source) -> Vec<Item> {
        match source {
            Source::Subtypes => self.subtype_items(),
            Source::Words => self.word_items(),
        }
    }
    /// Carries out `op` on `source`'s list.
    pub(super) fn list_op(&mut self, source: Source, op: Op) {
        match source {
            Source::Subtypes => self.subtype_op(op),
            Source::Words => self.word_op(op),
        }
    }
    /// Whether `source` has nothing to list: a search that hides every word is not that.
    fn list_empty(&self, source: Source) -> bool {
        match source {
            Source::Subtypes => self.subtype_items().is_empty(),
            Source::Words => !self.group_has_words(),
        }
    }
    /// Drops `source`'s delete to undo: its screen was left.
    pub(super) fn forget_deleted(&mut self, source: Source) {
        if self
            .list
            .deleted
            .as_ref()
            .is_some_and(|d| d.source == source)
        {
            self.list.deleted = None;
        }
    }
    /// Which of an item's buttons show, in order up, down and delete, for item `i` of `n`.
    fn buttons_shown(&self, source: Source, i: usize, n: usize) -> [bool; 3] {
        let reorder = self.words(source).reorder;
        [reorder && i > 0, reorder && i + 1 < n, true]
    }
    /// Control `id`'s custom accessibility actions, each the control it activates and its label.
    pub fn custom_actions_for(&self, id: i32) -> Vec<(i32, String)> {
        self.custom_actions
            .iter()
            .find(|c| c.0 == id)
            .map_or(vec![], |c| c.1.clone())
    }
    /// The stable slot of the item `key`, given one the first time it shows.
    pub fn slot(&mut self, key: &str) -> usize {
        match self.list.slots.iter().position(|k| k == key) {
            Some(at) => at,
            None => {
                self.list.slots.push(key.into());
                self.list.slots.len() - 1
            }
        }
    }
    /// The focus order of `source`'s list: each item, then its enabled buttons, the undo
    /// button, and Add.
    pub(super) fn list_focus(&mut self, source: Source) -> Vec<i32> {
        let items = self.list_items(source);
        let n = items.len();
        let mut out = vec![];
        for (i, item) in items.iter().enumerate() {
            let slot = self.slot(&item.key);
            out.push(item_id(slot, 0));
            let shown = self.buttons_shown(source, i, n);
            out.extend((1..=3).filter(|b| shown[b - 1]).map(|b| item_id(slot, b)));
        }
        if self
            .list
            .deleted
            .as_ref()
            .is_some_and(|d| d.source == source)
        {
            out.push(UNDO_ID);
        }
        out.push(ADD_ID);
        out
    }
    /// Paints `source`'s list from `(x, y)`; its height.
    pub(super) fn paint_list(
        &mut self,
        p: &mut Panel,
        source: Source,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let words = self.words(source);
        let items = self.list_items(source);
        let mut at = y;
        if self.list_empty(source) {
            at += self.warning_card(p, EMPTY_ID, self.label(words.empty), (x, at), clip, text);
        }
        let n = items.len();
        let first = if words.reorder { 0 } else { 2 };
        for (i, item) in items.into_iter().enumerate() {
            let slot = self.slot(&item.key);
            let entry = Entry {
                id: item_id(slot, 0),
                title: item.title.clone(),
                summary: item.summary,
                icon: "",
                ink: [None, None],
                buttons: 3 - first,
                right: 0.0,
                enabled: true,
                action: Action::List(source, Op::Open(i)),
            };
            let (rh, b) = self.paint_entry(p, entry, (x, at), clip, text);
            let ops = [Op::Up(i), Op::Down(i), Op::Delete(i)];
            let shown = self.buttons_shown(source, i, n);
            let mut custom = vec![];
            for (k, (op, on)) in ops.into_iter().zip(shown).enumerate().skip(first) {
                let id = item_id(slot, k + 1);
                let label = self.label(KEYS[k]);
                let button = self.end_button(b, 3 - k);
                let ink = self.enabled_color(self.colors[2], on);
                self.paint_button_icon(p, id, BUTTON_ICONS[k], button, ink, clip);
                let mut c = plain(id, button, label.clone(), "button");
                c.summary = item.title.clone();
                c.enabled = on;
                c.action = Action::List(source, op);
                self.push_control(c, clip);
                if on {
                    custom.push((id, label));
                }
            }
            self.custom_actions.push((item_id(slot, 0), custom));
            at += rh;
        }
        if let Some(gone) = self.list.deleted.clone().filter(|d| d.source == source) {
            let undo = (gone.title.as_str(), Action::List(gone.source, Op::Undo));
            at += self.undo_bar(p, undo, (x, at, self.config[0] - x), clip, text);
        }
        let add = Entry {
            id: ADD_ID,
            title: self.label(words.add),
            summary: String::new(),
            icon: "add",
            ink: [Some(self.colors[3]), None],
            buttons: 0,
            right: 0.0,
            enabled: true,
            action: Action::List(source, Op::Add),
        };
        at += self.paint_entry(p, add, (x, at), clip, text).0;
        at - y
    }
    /// The 48 dp square of the `k`th button from the end of row `b`, 4 dp in from its edge.
    pub(super) fn end_button(&self, b: Rect, k: usize) -> Rect {
        let size = self.px(48.0);
        let right = b.right - self.px(4.0) - size * (k - 1) as f32;
        rect(
            right - size,
            b.top + ((b.height() - size) / 2.0).round(),
            size,
            size,
        )
    }
    /// A 24 dp icon centred in button `b`, over the focus highlight when it has focus.
    pub(super) fn paint_button_icon(
        &self,
        p: &mut Panel,
        id: i32,
        icon: &str,
        b: Rect,
        ink: u32,
        clip: Rect,
    ) {
        if id == self.focus {
            self.paint_clipped_box(p, b, self.colors[4], b.height() / 2.0, clip);
        }
        let size = self.px(24.0);
        let at = rect(
            b.left + ((b.width() - size) / 2.0).round(),
            b.top + ((b.height() - size) / 2.0).round(),
            size,
            size,
        );
        self.paint_icon(p, icon, at, ink, clip);
    }
    /// Paints `e` as an M3 ListItem from `(x, y)`; its height and bounds.
    pub(super) fn paint_entry(
        &mut self,
        p: &mut Panel,
        e: Entry,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> (f32, Rect) {
        let w = self.config[0] - x - e.right;
        let lead = self.px(if e.icon.is_empty() { 16.0 } else { 56.0 });
        let buttons = if e.buttons > 0 {
            self.px(4.0) + self.px(48.0) * e.buttons as f32
        } else {
            0.0
        };
        let end = if e.buttons > 0 {
            buttons
        } else {
            self.px(16.0)
        };
        let width = (w - lead - end).max(1.0);
        let title = self.row_text(text, &e.title, self.sp(16), width, 1);
        let summary = self.row_text(text, &e.summary, self.sp(14), width, 2);
        let lh = self.measure(text, &title, self.sp(16), width);
        let sh = self.measure(text, &summary, self.sp(14), width);
        let three = self.multiline(sh);
        let least = match (summary.is_empty(), three) {
            (true, _) => 56.0,
            (false, false) => 72.0,
            (false, true) => 88.0,
        };
        let pad = self.px(if three { 12.0 } else { 8.0 });
        let rh = (lh + sh + 2.0 * pad).max(self.px(least));
        let b = rect(x, y, w, rh);
        let item = rect(x, y, w - buttons, rh);
        if e.id == self.focus {
            self.paint_clipped_box(p, item, self.colors[4], self.px(8.0), clip);
        }
        let top = y + ((rh - lh - sh) / 2.0).round();
        if !e.icon.is_empty() {
            let size = self.px(24.0);
            let at = rect(
                x + self.px(16.0),
                y + ((rh - size) / 2.0).round(),
                size,
                size,
            );
            let ink = e.ink[0].unwrap_or(self.colors[2]);
            self.paint_icon(p, e.icon, at, self.enabled_color(ink, e.enabled), clip);
        }
        let ink = self.enabled_color(e.ink[0].unwrap_or(self.colors[1]), e.enabled);
        let at = rect(x + lead, top, width, lh);
        self.paint_text(p, &title, at, self.sp(16), ink, clip);
        if sh > 0.0 {
            let ink = self.enabled_color(e.ink[1].unwrap_or(self.colors[2]), e.enabled);
            let at = rect(x + lead, top + lh, width, sh);
            self.paint_text(p, &summary, at, self.sp(14), ink, clip);
        }
        let role = if matches!(e.action, Action::None) {
            "text"
        } else {
            "button"
        };
        let mut c = plain(e.id, item, e.title, role);
        c.summary = e.summary;
        c.enabled = e.enabled;
        c.action = e.action;
        self.push_control(c, clip);
        (rh, b)
    }
    /// "Deleted {name}" and an Undo button that does `action`, across `w` from `x`; its height.
    pub(super) fn undo_bar(
        &mut self,
        p: &mut Panel,
        (name, action): (&str, Action),
        (x, y, w): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let undo = self.label(KEYS[4]);
        let layout = self.button_label(text, &undo);
        let bw = (layout.width.ceil() + 2.0 * self.px(12.0)).max(self.px(58.0));
        let message = self.label(KEYS[3]).replace("{name}", name);
        let width = (w - 2.0 * self.px(16.0) - bw - self.px(8.0)).max(1.0);
        let mh = self.measure(text, &message, self.sp(14), width).ceil();
        let rh = (mh + 2.0 * self.px(16.0)).max(self.px(64.0));
        let bar = rect(
            x + self.px(8.0),
            y + self.px(4.0),
            w - 2.0 * self.px(8.0),
            rh - self.px(8.0),
        );
        self.paint_clipped_box(p, bar, self.colors[12], self.px(8.0), clip);
        let at = rect(x + self.px(16.0), y + ((rh - mh) / 2.0).round(), width, mh);
        self.paint_text(p, &message, at, self.sp(14), self.colors[1], clip);
        self.push_control(plain(UNDO_ID + 1, at, message, "status"), clip);
        let touch = self.px(48.0);
        let b = rect(
            x + w - self.px(16.0) - bw,
            y + ((rh - touch) / 2.0).round(),
            bw,
            touch,
        );
        if self.focus == UNDO_ID {
            self.paint_clipped_box(p, b, self.colors[4], touch / 2.0, clip);
        }
        let (lw, lh) = (layout.width.ceil(), layout.height.ceil());
        let label_at = rect(
            b.left + ((bw - lw) / 2.0).round(),
            b.top + ((touch - lh) / 2.0).round(),
            lw + 1.0,
            lh,
        );
        let style = super::cards::LABEL_LARGE;
        self.paint_styled(p, &undo, label_at, style, self.colors[3], clip);
        let mut c = plain(UNDO_ID, b, undo, "button");
        c.summary = name.into();
        c.action = action;
        self.push_control(c, clip);
        rh
    }
    /// FlorisWarningCard 8 dp in: a warning icon and `label` on its fixed yellow; its height
    /// with the padding.
    pub(super) fn warning_card(
        &mut self,
        p: &mut Panel,
        id: i32,
        label: String,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let card = ("warning", super::cards::WARNING, 8.0);
        self.icon_card(p, (id, label), card, at, clip, text)
    }
    /// A card in `fill` round `icon` and `label` in titleMedium, both in `ink`, `inset` dp in
    /// from each side and 8 dp from the rows round it: FlorisSimpleCard as the warning and info
    /// cards draw it. Its height with the 8 dp round it.
    pub(super) fn icon_card(
        &mut self,
        p: &mut Panel,
        (id, label): (i32, String),
        (icon, [fill, ink], inset): (&str, [u32; 2], f32),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0] - x;
        let icon_box = 2.0 * self.px(12.0) + self.px(24.0);
        let card = rect(
            x + self.px(inset),
            y + self.px(8.0),
            w - 2.0 * self.px(inset),
            0.0,
        );
        let column = (card.width() - icon_box - self.px(16.0)).max(1.0);
        let style = super::cards::TITLE_MEDIUM;
        let th = self.styled(text, &label, style, Some(column)).height.ceil();
        let row = icon_box.max(th);
        let b = rect(card.left, card.top, card.width(), row + 2.0 * self.px(8.0));
        self.paint_clipped_box(p, b, fill, self.px(12.0), clip);
        let size = self.px(24.0);
        let icon_top = b.top + self.px(8.0) + ((row - icon_box) / 2.0).round() + self.px(12.0);
        let icon_rect = rect(b.left + self.px(12.0), icon_top, size, size);
        self.paint_icon(p, icon, icon_rect, ink, clip);
        let top = b.top + self.px(8.0) + ((row - th) / 2.0).round();
        let at = rect(b.left + icon_box, top, column, th);
        self.paint_styled(p, &label, at, style, ink, clip);
        self.push_control(plain(id, b, label, "card"), clip);
        b.height() + 2.0 * self.px(8.0)
    }
}
