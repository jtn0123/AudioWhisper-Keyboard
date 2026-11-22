//! The word editor: UserDictionaryScreen's add and edit dialog, as the item editor's screen. A
//! draft starts as the stored word when editing; when adding, with the frequency 255 and, unlike
//! the dialog, the language whose words were showing. Save checks each field with the rules
//! `UserDictionaryValidation` called (`utilities::validation`), refuses a word the language has
//! already, and builds the entry as the dialog built it; Kotlin normalises its locale.
use super::{Page, Word, group};
use crate::settings_ui::{
    App,
    catalogue::{Kind, words::Part},
};
use wire::settings_ui::words::effect;
use wire::utilities::{EMPTY, FREQUENCY, INVALID, SHORTCUT, WORD};

/// What the word and shortcut errors name as the rule: `UserDictionaryValidation.WordRegex`.
pub const WORD_REGEX: &str = r"^[^\s;,]+$";
/// `FREQUENCY_MIN` and `FREQUENCY_MAX`, both inclusive.
pub const FREQUENCY_RANGE: [i32; 2] = [1, 255];
/// The frequency a new word starts with: `UserDictionaryEntryToAdd`'s.
pub const NEW_FREQUENCY: &str = "255";

/// The word being edited, its four fields as typed: word, frequency, shortcut, locale.
#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub fields: [String; 4],
    /// The stored word being changed, or None when adding.
    pub stored: Option<Word>,
    /// A save found a field invalid: errors show from now on, as `showValidationErrors`.
    pub checked: bool,
}

/// `value` as Kotlin's `trim()` leaves it.
fn trimmed(value: &str) -> String {
    String::from_utf16_lossy(chars::trim(&settings::text(value)))
}

/// The rule `UserDictionaryValidation` names for field `k`, as `utilities::validation` judges
/// `value`: "" when valid, else `EMPTY` or `INVALID`. The locale has no rule: `FlorisLocale`
/// reads any text.
pub fn verdict(k: usize, value: &str) -> &'static str {
    let rule = match k {
        0 => WORD,
        1 => FREQUENCY,
        2 => SHORTCUT,
        _ => return "",
    };
    let [min, max] = FREQUENCY_RANGE;
    utilities::validation::validate(&settings::text(rule), &settings::text(value), min, max)
}

/// The string key of field `k`'s error for `verdict`, None when it has none.
pub fn error_key(k: usize, verdict: &str) -> Option<&'static str> {
    match (k, verdict) {
        (0, EMPTY) => Some("settings__udm__dialog__word_error_empty"),
        (0, INVALID) => Some("settings__udm__dialog__word_error_invalid"),
        (1, EMPTY) => Some("settings__udm__dialog__freq_error_empty"),
        (1, INVALID) => Some("settings__udm__dialog__freq_error_invalid"),
        (2, INVALID) => Some("settings__udm__dialog__shortcut_error_invalid"),
        _ => None,
    }
}

impl Draft {
    /// The entry the dialog's `onConfirm` built: the word, shortcut and locale trimmed, a blank
    /// shortcut or locale null, the frequency in decimal. The locale is normalised by Kotlin.
    pub fn entry(&self) -> Option<Word> {
        let freq = chars::text::to_int_or_null(chars::trim(&settings::text(&self.fields[1])))?;
        let optional = |k: usize| Some(trimmed(&self.fields[k])).filter(|s| !s.is_empty());
        Some(Word {
            id: self.stored.as_ref().map_or(0, |w| w.id),
            word: trimmed(&self.fields[0]),
            freq,
            locale: optional(3),
            shortcut: optional(2),
        })
    }
}

impl App {
    /// A fresh draft for the editor at `page`; none while the word to edit is not reported.
    pub(super) fn open_word(&mut self, page: &Page) {
        let draft = match page {
            Page::Edit(id) => {
                let stored = self
                    .word_report()
                    .and_then(|r| r.words.iter().find(|w| w.id == *id).cloned());
                stored.map(|w| Draft {
                    fields: [
                        w.word.clone(),
                        w.freq.to_string(),
                        w.shortcut.clone().unwrap_or_default(),
                        w.locale.clone().unwrap_or_default(),
                    ],
                    stored: Some(w),
                    checked: false,
                })
            }
            _ => Some(Draft {
                fields: [
                    String::new(),
                    NEW_FREQUENCY.into(),
                    String::new(),
                    self.adding_locale(),
                ],
                stored: None,
                checked: false,
            }),
        };
        self.words.draft = draft;
    }
    /// The language a new word starts in: the one whose words the editor was opened from, as
    /// Gboard does, or none from the first screen or all languages.
    fn adding_locale(&self) -> String {
        let below = self.stack.iter().rev().nth(1).and_then(|r| super::place(r));
        match below {
            Some((_, Page::Words(Some(key)))) => key,
            _ => String::new(),
        }
    }
    /// Whether the draft's word is stored already under its language, other than as itself.
    pub fn word_exists(&self, d: &Draft) -> bool {
        let (Some(r), Some(new)) = (self.word_report(), d.entry()) else {
            return false;
        };
        let key = group(&r.dictionary, new.locale.as_deref());
        let own = d.stored.as_ref().map(|w| w.id);
        r.words.iter().any(|w| {
            Some(w.id) != own
                && w.word == new.word
                && group(&r.dictionary, w.locale.as_deref()) == key
        })
    }
    /// Field `k`'s error message, or empty: shown once a save found the draft invalid.
    pub fn word_error(&self, k: usize) -> String {
        let Some(d) = self.words.draft.as_ref().filter(|d| d.checked) else {
            return String::new();
        };
        match self.error_of(k, d) {
            Some(key) => self.label(key).replace("{regex}", WORD_REGEX),
            None => String::new(),
        }
    }
    fn error_of(&self, k: usize, d: &Draft) -> Option<&'static str> {
        let error = error_key(k, verdict(k, &d.fields[k]));
        match error {
            None if k == 0 && self.word_exists(d) => Some("settings__udm__word_error_exists"),
            other => other,
        }
    }
    /// Field `k` as typed.
    pub fn word_field_text(&self, k: usize) -> String {
        self.words
            .draft
            .as_ref()
            .map(|d| d.fields[k].clone())
            .unwrap_or_default()
    }
    /// Field `k` changed.
    pub(in crate::settings_ui) fn word_typed(&mut self, k: usize, value: String) {
        if let Some(d) = &mut self.words.draft {
            d.fields[k] = value;
        }
    }
    /// Whether the field being edited is a word's frequency, which asks for a number keyboard.
    pub fn frequency_field(&self) -> bool {
        self.field
            .and_then(|i| self.rows().get(i))
            .is_some_and(|r| matches!(r.kind, Kind::Words(Part::Field(1))))
    }
    /// Save: an invalid field marks the fields; otherwise the word is added or changed and the
    /// editor closes.
    pub(in crate::settings_ui) fn save_word(&mut self) {
        let Some(d) = self.words.draft.clone() else {
            return;
        };
        let entry = d.entry();
        let invalid = (0..4).any(|k| self.error_of(k, &d).is_some());
        let Some(w) = entry.filter(|_| !invalid) else {
            if let Some(d) = &mut self.words.draft {
                d.checked = true;
            }
            return;
        };
        let kind = if d.stored.is_some() {
            effect::CHANGE
        } else {
            effect::ADD
        };
        self.word_effect(kind, &[&w]);
        self.close_editor();
    }
    /// Whether the word editor is on screen.
    pub fn word_editor(&self) -> bool {
        matches!(self.word_place(), Some((_, Page::Add | Page::Edit(_))))
    }
    /// Closes an editor, hiding the keyboard first: Back alone would only hide it.
    pub(in crate::settings_ui) fn close_editor(&mut self) {
        if self.editor.focused {
            self.editor.focused = false;
            self.editor.composing = None;
            self.effects.push(vec!["hideEditor".into()]);
        }
        self.back();
    }
    /// The word the top bar's Delete deletes: the stored word the editor is changing.
    pub fn word_to_delete(&self) -> Option<&Word> {
        matches!(self.word_place(), Some((_, Page::Edit(_))))
            .then(|| self.words.draft.as_ref()?.stored.as_ref())
            .flatten()
    }
    /// The editor's Delete: deletes the stored word, keeping it to undo, and closes onto the
    /// list with focus on Undo, as a delete in the list leaves it.
    pub(super) fn delete_word(&mut self) {
        let Some(w) = self.word_to_delete().cloned() else {
            return;
        };
        self.remove_word(0, w);
        self.close_editor();
        self.focus = crate::settings_ui::items::UNDO_ID;
    }
}
