//! The rule editor's painting: why Add or Apply refused, the element dropdown, the font's name,
//! the selector chips, the codes field and the mode and shift state chips. A selector chip
//! toggles, as `updateCurrentRule` did; a value chip removes the value and the last chip, Add,
//! lists the values the rule lacks. Selectors and attributes show only for an element.
use super::{
    Tap,
    form::FONT,
    paint::DELETE_ID,
    rules::SELECTOR_KEYS,
    ui::{Chip, Dropdown},
};
use crate::settings_ui::{Action, App, catalogue::theme_editor::Part};
use crate::{panels::Panel, text::Text};
use keys::Rect;
use snygg::grammar::SELECTORS;

pub const ERROR_ID: i32 = 400_060;
/// Selector `s`'s chip.
pub const SELECTOR_ID: i32 = 400_070;
/// Attribute `k`'s value chips start at `VALUE_ID + 16 * k`; its Add chip is the 15th.
pub const VALUE_ID: i32 = 400_100;

fn value_id(k: usize, j: usize) -> i32 {
    VALUE_ID + 16 * k as i32 + j.min(15) as i32
}

impl App {
    /// Paints the rule editor's part `part` at row `i`; its height.
    pub(super) fn rule_part(
        &mut self,
        p: &mut Panel,
        (i, part): (usize, Part),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(form) = self.theme_editor.form.clone() else {
            return 0.0;
        };
        let title = self.label(self.rows()[i].label);
        match part {
            Part::RuleError => {
                let (error, ink) = (self.rule_error(), self.colors[18]);
                self.theme_note(p, (ERROR_ID, error, ink, self.px(8.0)), at, clip, text)
            }
            Part::Element => {
                let d = Dropdown {
                    id: 1000 + i as i32,
                    title,
                    value: self.element_choice(&form.element),
                    error: form.checked && form.element.is_empty(),
                    enabled: form.original.is_none(),
                    action: Action::ThemeEdit(Tap::Open(i)),
                };
                self.dropdown(p, d, at, clip, text)
            }
            Part::Font if form.element == FONT => {
                self.line_box(p, (i, title, String::new()), at, clip, text)
            }
            Part::Attribute(0) if form.is_element() => {
                self.line_box(p, (i, title, String::new()), at, clip, text)
            }
            Part::Selectors if form.is_element() => {
                let top = self.field_title(p, &title, at, clip, text);
                let chips = (1..SELECTORS.len())
                    .map(|s| Chip {
                        id: SELECTOR_ID + s as i32,
                        label: self.selector_label(s),
                        selected: form.selector == s,
                        action: Action::ThemeEdit(Tap::Selector(s)),
                    })
                    .collect();
                self.chips(p, chips, (at.0, top), clip, text) - at.1
            }
            Part::Attribute(k) if form.is_element() => {
                let top = self.field_title(p, &title, at, clip, text);
                self.value_chips(p, (i, k), (at.0, top), clip, text) - at.1
            }
            _ => 0.0,
        }
    }
    /// Selector `s` as the chip names it: translated, or its id at the developer level.
    pub fn selector_label(&self, s: usize) -> String {
        if self.theme_developer() {
            SELECTORS[s][1..].into()
        } else {
            self.label(SELECTOR_KEYS[s])
        }
    }
    /// Attribute `k`'s values as chips, or the note that none narrows the rule, and Add while a
    /// value is left to add; where they end.
    fn value_chips(
        &mut self,
        p: &mut Panel,
        (i, k): (usize, usize),
        (x, mut y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let values = self
            .theme_editor
            .form
            .as_ref()
            .map_or(vec![], |f| f.values(k));
        if values.is_empty() {
            let note = self.label("settings__theme_editor__no_codes_defined");
            let ink = self.colors[2];
            y += self.theme_note(p, (value_id(k, 14), note, ink, 0.0), (x, y), clip, text);
        }
        let mut chips: Vec<Chip> = values
            .iter()
            .enumerate()
            .map(|(j, v)| Chip {
                id: value_id(k, j),
                label: self.attr_value_label(k, v),
                selected: true,
                action: Action::ThemeEdit(Tap::Remove(k, v.clone())),
            })
            .collect();
        if self.theme_choices(i).is_some_and(|c| !c.is_empty()) {
            chips.push(Chip {
                id: value_id(k, 15),
                label: self.label("action__add"),
                selected: false,
                action: Action::ThemeEdit(Tap::Open(i)),
            });
        }
        self.chips(p, chips, (x, y), clip, text)
    }
    /// The rule editor's focus order, row by row: Delete in the top bar first when editing.
    pub(super) fn rule_focus(&self, i: usize, part: Part) -> Vec<i32> {
        let Some(form) = self.theme_editor.form.as_ref() else {
            return vec![];
        };
        let field = 1000 + i as i32;
        match part {
            Part::RuleError if form.original.is_some() => vec![DELETE_ID],
            Part::Element if form.original.is_none() => vec![field],
            Part::Font if form.element == FONT => vec![field],
            Part::Attribute(0) if form.is_element() => vec![field],
            Part::Selectors if form.is_element() => (1..SELECTORS.len())
                .map(|s| SELECTOR_ID + s as i32)
                .collect(),
            Part::Attribute(k) if form.is_element() => {
                let n = form.values(k).len();
                let mut out: Vec<i32> = (0..n).map(|j| value_id(k, j)).collect();
                if self.theme_choices(i).is_some_and(|c| !c.is_empty()) {
                    out.push(value_id(k, 15));
                }
                out
            }
            _ => vec![],
        }
    }
}
