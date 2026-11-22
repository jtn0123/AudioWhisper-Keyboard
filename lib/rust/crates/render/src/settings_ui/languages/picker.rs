//! Select language, as SelectLocaleScreen had it: a search field, then every locale the keyboard
//! knows sorted by its shown name, filtered by the search as SelectLocaleScreen filtered, and a
//! message when nothing matches. With no search, the system's languages come first under
//! "Suggested languages", as Gboard lists them, then "All languages".
use super::Tap;
use crate::settings_ui::{Action, App, Control, cards::plain, items::Entry};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// The control ids of the suggested languages and of the whole list's.
pub const SUGGESTED_ID: i32 = 30000;
pub const LOCALE_ID: i32 = 40000;
/// The section headings' ids.
const HEADING_ID: [i32; 2] = [29998, 29999];

impl App {
    /// The locales the list shows, as indices into the report's, sorted by shown name in lower
    /// case (compared as Kotlin compares strings) and filtered by the search.
    pub fn picker_locales(&self) -> Vec<usize> {
        let Some(r) = self.languages.resources.as_ref() else {
            return vec![];
        };
        let mut order: Vec<(Vec<u16>, usize)> = r
            .locales
            .iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    self.shown_name(n).to_lowercase().encode_utf16().collect(),
                    i,
                )
            })
            .collect();
        order.sort_by(|a, b| a.0.cmp(&b.0));
        let term = self.languages.search.trim().to_lowercase();
        order
            .into_iter()
            .map(|(_, i)| i)
            .filter(|&i| {
                let n = &r.locales[i];
                let has = |s: &str| s.to_lowercase().contains(&term);
                let starts = |s: &str| s.to_lowercase().starts_with(&term);
                term.is_empty()
                    || has(&n.label)
                    || has(&n.native)
                    || has(&n.english)
                    || starts(&n.tag)
                    || starts(&n.locale_tag)
            })
            .collect()
    }
    /// With no search: the system's languages the list has, in the system's order.
    pub fn picker_suggestions(&self) -> Vec<usize> {
        let Some(r) = self.languages.resources.as_ref() else {
            return vec![];
        };
        if !self.languages.search.trim().is_empty() {
            return vec![];
        }
        let mut out: Vec<usize> = vec![];
        for s in &r.system {
            if let Some(i) = r.locales.iter().position(|n| n.tag == s.tag)
                && !out.contains(&i)
            {
                out.push(i);
            }
        }
        out
    }
    /// The search field: a 56 dp TextField on surfaceContainerHighest, square, with no
    /// indicator line, a leading search icon and its placeholder while empty; its height.
    pub(in crate::settings_ui) fn search_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0] - x;
        let h = self.px(56.0);
        self.paint_clipped_box(p, rect(x, y, w, h), self.colors[5], 0.0, clip);
        let size = self.px(24.0);
        let icon = rect(
            x + self.px(12.0),
            y + ((h - size) / 2.0).round(),
            size,
            size,
        );
        self.paint_icon(p, "search", icon, self.colors[2], clip);
        let id = 1000 + i as i32;
        let placeholder = self.label(self.rows()[i].label);
        let (editor, _) = self.field_editor(id);
        if editor.text.is_empty() && editor.composing.is_none() {
            let room = (w - self.px(52.0) - self.px(16.0)).max(1.0);
            let shown = self.row_text(text, &placeholder, self.sp(16), room, 1);
            let ph = self.measure(text, &shown, self.sp(16), room).ceil();
            let at = rect(x + self.px(52.0), y + ((h - ph) / 2.0).round(), room, ph);
            self.paint_text(p, &shown, at, self.sp(16), self.colors[2], clip);
        }
        let field = Control {
            id,
            bounds: rect(x + self.px(36.0), y, w - self.px(36.0), h),
            label: placeholder,
            summary: String::new(),
            role: "edit",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::Field(i),
            range: None,
        };
        self.add(p, field, clip, text);
        h
    }
    /// The list, or the message when nothing matches; its height.
    pub(in crate::settings_ui) fn locales_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let shown = self.picker_locales();
        if shown.is_empty() {
            let term = self.languages.search.clone();
            return self.not_found(p, i, &term, (x, y), clip, text);
        }
        let suggested = self.picker_suggestions();
        let mut at = y;
        let mut sections = vec![];
        if !suggested.is_empty() {
            sections.push((Some(0), suggested, SUGGESTED_ID));
            sections.push((Some(1), shown, LOCALE_ID));
        } else {
            sections.push((None, shown, LOCALE_ID));
        }
        // A one-line item's height, for the items out of sight that are not measured.
        let one = (self.measure(text, "Ag", self.sp(16), self.config[0]) + self.px(16.0))
            .max(self.px(56.0));
        for (heading, list, first) in sections {
            if let Some(h) = heading {
                at += self.section_heading(p, h, (x, at), clip, text);
            }
            for (k, index) in list.into_iter().enumerate() {
                let id = first + k as i32;
                if id != self.focus && (at > clip.bottom || at + one < clip.top) {
                    at += one;
                    continue;
                }
                let Some(n) = self
                    .languages
                    .resources
                    .as_ref()
                    .map(|r| r.locales[index].clone())
                else {
                    break;
                };
                let entry = Entry {
                    id,
                    title: self.shown_name(&n),
                    summary: String::new(),
                    icon: "",
                    ink: [None, None],
                    buttons: 0,
                    right: 0.0,
                    enabled: true,
                    action: Action::Lang(Tap::Locale(n.tag)),
                };
                at += self.paint_entry(p, entry, (x, at), clip, text).0;
            }
        }
        at - y
    }
    /// "Suggested languages" or "All languages", as a settings group's heading; its height.
    fn section_heading(
        &mut self,
        p: &mut Panel,
        which: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let key = [
            "settings__localization__suggested_languages",
            "settings__localization__all_languages",
        ][which];
        let label = self.label(key);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let lh = self.measure(text, &label, self.sp(16), width);
        let rh = (lh + self.px(16.0)).max(self.px(56.0));
        let at = rect(x + self.px(16.0), y + ((rh - lh) / 2.0).round(), width, lh);
        self.paint_text(p, &label, at, self.sp(16), self.colors[9], clip);
        p.items
            .last_mut()
            .expect("paint_text pushed an item")
            .text
            .weight = 700;
        self.push_control(
            plain(
                HEADING_ID[which],
                rect(x, y, self.config[0] - x, rh),
                label,
                "heading",
            ),
            clip,
        );
        rh
    }
    /// Row `i`'s "nothing found for “{search_term}”", centred with 16 dp padding, faded to 54%.
    pub(in crate::settings_ui) fn not_found(
        &mut self,
        p: &mut Panel,
        i: usize,
        term: &str,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let label = self
            .label(self.rows()[i].label)
            .replace("{search_term}", term);
        let room = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let layout = self.android(text, &label, self.sp(16), Some(room));
        let (lw, lh) = (layout.width.ceil(), layout.height.ceil());
        let left = x + ((self.config[0] - x - lw) / 2.0).round();
        let at = rect(left, y + self.px(16.0), lw, lh);
        let faded = (self.colors[1] & 0x00ff_ffff) | 0x8a00_0000;
        self.paint_text(p, &label, at, self.sp(16), faded, clip);
        self.push_control(plain(1000 + i as i32, at, label, "text"), clip);
        lh + 2.0 * self.px(16.0)
    }
    /// The focus order of the list: the suggestions, then every shown locale.
    pub(in crate::settings_ui) fn locales_focus(&self, i: usize) -> Vec<i32> {
        let shown = self.picker_locales();
        if shown.is_empty() {
            return vec![1000 + i as i32];
        }
        let suggested = self.picker_suggestions().len() as i32;
        (0..suggested)
            .map(|k| SUGGESTED_ID + k)
            .chain((0..shown.len() as i32).map(|k| LOCALE_ID + k))
            .collect()
    }
}
