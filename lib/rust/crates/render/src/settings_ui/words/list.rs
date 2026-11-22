//! A language's words, as the editable list (`items.rs`) shows them without Move up and Move
//! down: each word titled by itself and summarised by its frequency and shortcut, as
//! UserDictionaryScreen did, but in alphabetical order, narrowed by the search, with a Delete
//! button and an undo.
use super::{Page, Word, decode, encode, route};
use crate::settings_ui::{
    App,
    items::{Deleted, Item, Op, Source, UNDO_ID, Words},
};
use wire::settings_ui::words::effect;

pub const WORDS: Words = Words {
    empty: "settings__udm__no_words_in_dictionary",
    add: "settings__udm__dialog__title_add",
    reorder: false,
};

impl App {
    /// The words of the language on screen, in its dictionary's order.
    fn group_words(&self) -> Vec<Word> {
        let (Some(r), Some((_, Page::Words(key)))) = (self.word_report(), self.word_place()) else {
            return vec![];
        };
        r.words
            .iter()
            .filter(|w| super::group(&r.dictionary, w.locale.as_deref()) == key)
            .cloned()
            .collect()
    }
    /// Whether the language on screen has a word, whatever the search.
    pub fn group_has_words(&self) -> bool {
        !self.group_words().is_empty()
    }
    /// The words the list shows: those whose word or shortcut holds the search, ignoring case,
    /// sorted alphabetically ignoring case.
    pub fn shown_words(&self) -> Vec<Word> {
        let term = self.words.search.trim().to_lowercase();
        let holds = |s: &str| s.to_lowercase().contains(&term);
        let mut out: Vec<Word> = self
            .group_words()
            .into_iter()
            .filter(|w| holds(&w.word) || w.shortcut.as_deref().is_some_and(holds))
            .collect();
        out.sort_by_cached_key(|w| (w.word.to_lowercase(), w.word.clone(), w.id));
        out
    }
    /// "Frequency: {freq}", with " | Shortcut: {shortcut}" when it has one.
    pub fn word_summary(&self, w: &Word) -> String {
        let freq = w.freq.to_string();
        match &w.shortcut {
            Some(s) => self
                .label("settings__udm__word_summary_freq_shortcut")
                .replace("{freq}", &freq)
                .replace("{shortcut}", s),
            None => self
                .label("settings__udm__word_summary_freq")
                .replace("{freq}", &freq),
        }
    }
    /// The list's items, keyed by the word's id.
    pub fn word_items(&self) -> Vec<Item> {
        self.shown_words()
            .into_iter()
            .map(|w| Item {
                key: w.id.to_string(),
                summary: self.word_summary(&w),
                title: w.word,
            })
            .collect()
    }
    /// Carries out a tap on the word list.
    pub(in crate::settings_ui) fn word_op(&mut self, op: Op) {
        let Some((d, _)) = self.word_place() else {
            return;
        };
        let words = self.shown_words();
        match op {
            Op::Open(i) => {
                if let Some(w) = words.get(i) {
                    self.navigate(&route(d, &Page::Edit(w.id)));
                }
            }
            Op::Delete(i) => {
                if let Some(w) = words.get(i).cloned() {
                    self.remove_word(i, w);
                }
            }
            Op::Add => self.navigate(&route(d, &Page::Add)),
            Op::Undo => self.undo_word(),
            // The list has no order to change.
            Op::Up(_) | Op::Down(_) => {}
        }
    }
    /// Deletes `w`, shown at `index`, keeping it to undo. It leaves the list at once; the
    /// report that follows confirms it.
    pub(super) fn remove_word(&mut self, index: usize, w: Word) {
        self.word_effect(effect::DELETE, &[&w]);
        if let Some(r) = &mut self.words.report {
            r.words.retain(|x| x.id != w.id);
        }
        self.list.deleted = Some(Deleted {
            source: Source::Words,
            index,
            value: serde_json::Value::from(encode(&w)).to_string(),
            title: w.word,
        });
        self.focus = UNDO_ID;
    }
    /// Stores the deleted word again, as it was.
    fn undo_word(&mut self) {
        let Some(gone) = self.list.deleted.take_if(|d| d.source == Source::Words) else {
            return;
        };
        let Some(w) = deleted_word(&gone.value) else {
            return;
        };
        self.word_effect(effect::RESTORE, &[&w]);
        if let Some(r) = &mut self.words.report
            && !r.words.iter().any(|x| x.id == w.id)
        {
            r.words.push(w);
        }
        self.focus = -1;
    }
}

/// The word a [`Deleted`]'s value holds.
pub fn deleted_word(value: &str) -> Option<Word> {
    let fields: Vec<String> = serde_json::from_str(value).ok()?;
    decode(&fields)
}
