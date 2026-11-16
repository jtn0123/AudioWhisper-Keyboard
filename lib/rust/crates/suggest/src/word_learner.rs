//! Feeds the learned words the words the user finishes. A typed word is finished by a space,
//! punctuation or Enter (after any auto-correction), or by tapping a candidate for it. A swiped
//! word waits for what comes next: the next word or a separator keeps it, a candidate tap or a
//! backspace means it was wrong. Finishing a different word where the last one began (the user
//! went back and changed it) takes the last one back first, as does undoing its auto-correction.
//! Nothing of an e-mail address or URL is learned ([`addresses`]).
//!
//! Was Kotlin's `WordLearner`, which still tells this when a word is finished. Each call gets the
//! store to feed, locked only while it adds or removes a word (suggestions read it from another
//! thread), and the dictionary if it is open, which says how to spell what is learned.

use std::sync::{Mutex, MutexGuard, PoisonError};

use chars::lowercase;

use crate::addresses;
use crate::dictionary::Lookup;
use crate::learned_words::{LearnedWords, is_storable, previous_key};
use crate::previous_words::PreviousWords;
use crate::raw::NOT_A_PROBABILITY;
use crate::word_case::WordCase;

/// Where a swiped word starts is not known: the space before it may be added later.
const NO_POSITION: i32 = -1;

/// What one learn stored, so it can be taken back: `word` as committed, `spelling` as stored.
#[derive(Debug)]
struct Learned {
    word: Vec<u16>,
    spelling: Vec<u16>,
    previous: Option<Vec<u16>>,
    start: i32,
}

#[derive(Debug)]
struct Swiped {
    word: Vec<u16>,
    text_before: Vec<u16>,
}

/// What the learner remembers between words: the last word it learned, and a swiped word waiting
/// to be kept.
#[derive(Debug, Default)]
pub struct WordLearner {
    last: Option<Learned>,
    swiped: Option<Swiped>,
}

impl WordLearner {
    /// `word`, which starts at `start` in the editor right after `text_before`, is finished.
    /// Returns whether `store` changed.
    pub fn finished<L: Lookup + ?Sized>(
        &mut self,
        store: &Mutex<LearnedWords>,
        mut lookup: Option<&mut L>,
        word: &[u16],
        text_before: &[u16],
        start: i32,
    ) -> Result<bool, L::Error> {
        let pending = self.swiped.take();
        // A different word after a swiped one: the swipe was kept and typing went on.
        let mut changed = match pending {
            Some(pending) if pending.word != word => {
                let words = (&pending.word[..], &pending.text_before[..]);
                self.learn(store, lookup.as_deref_mut(), words, NO_POSITION)?
            }
            _ => false,
        };
        let replaces = |last: &Learned| start >= 0 && last.start == start;
        if let Some(last) = self.last.as_ref().filter(|&last| replaces(last)) {
            if last.word == word {
                return Ok(changed);
            }
            self.take_back(store);
            changed = true;
        }
        Ok(self.learn(store, lookup, (word, text_before), start)? || changed)
    }

    /// A swiped `word` went in after `text_before`; a swiped word still waiting before it is
    /// kept. Returns whether `store` changed.
    pub fn swiped<L: Lookup + ?Sized>(
        &mut self,
        store: &Mutex<LearnedWords>,
        lookup: Option<&mut L>,
        word: &[u16],
        text_before: &[u16],
    ) -> Result<bool, L::Error> {
        let swiped = Swiped {
            word: word.to_vec(),
            text_before: text_before.to_vec(),
        };
        match self.swiped.replace(swiped) {
            Some(pending) => {
                let words = (&pending.word[..], &pending.text_before[..]);
                self.learn(store, lookup, words, NO_POSITION)
            }
            None => Ok(false),
        }
    }

    /// The swiped word waiting to be kept was replaced or deleted.
    pub fn drop_swiped(&mut self) {
        self.swiped = None;
    }

    /// The auto-correction that starts at `start` was undone. Returns whether `store` changed.
    pub fn undo(&mut self, store: &Mutex<LearnedWords>, start: i32) -> bool {
        let undone = self
            .last
            .as_ref()
            .is_some_and(|last| start >= 0 && last.start == start);
        if undone {
            self.take_back(store);
        }
        undone
    }

    /// Another field: positions there say nothing about what was learned here.
    pub fn reset(&mut self) {
        self.last = None;
        self.swiped = None;
    }

    /// Learns `word`, typed right after `text_before`, if it can be stored.
    fn learn<L: Lookup + ?Sized>(
        &mut self,
        store: &Mutex<LearnedWords>,
        lookup: Option<&mut L>,
        (word, text_before): (&[u16], &[u16]),
        start: i32,
    ) -> Result<bool, L::Error> {
        if !is_storable(word) {
            return Ok(false);
        }
        if addresses::is_inside(text_before, word) {
            return Ok(self.take_back_address(store, text_before, start));
        }
        let context = PreviousWords::from(text_before);
        let first_word = context.words.first().map(Vec::as_slice);
        let previous = previous_key(first_word, context.is_beginning_of_sentence);
        let starts_sentence = context.words.is_empty() && context.is_beginning_of_sentence;
        let spelling = spelling_to_learn(lookup, word, starts_sentence)?;
        // A capital the sentence start may have put there doesn't replace a spelling seen
        // mid-sentence.
        let keep_spelling =
            starts_sentence && spelling == word && WordCase::of(word) == WordCase::Capitalized;
        lock(store).add(&spelling, previous, keep_spelling);
        self.last = Some(Learned {
            word: word.to_vec(),
            spelling,
            previous: previous.map(<[u16]>::to_vec),
            start,
        });
        Ok(true)
    }

    /// A piece of an address or URL, typed at `start` after `text_before`, is not learned, and
    /// the piece before it in the same address, learned before anything showed it was one
    /// ("example" when the '.' of "example.com" was typed), is taken back. Returns whether `store`
    /// changed.
    fn take_back_address(
        &mut self,
        store: &Mutex<LearnedWords>,
        text_before: &[u16],
        start: i32,
    ) -> bool {
        let glued = i32::try_from(addresses::glued(text_before).len()).unwrap_or(i32::MAX);
        let address = start.saturating_sub(glued)..start;
        let earlier = start >= 0
            && self
                .last
                .as_ref()
                .is_some_and(|last| address.contains(&last.start));
        if earlier {
            self.take_back(store);
        }
        earlier
    }

    /// Takes back the last word learned.
    fn take_back(&mut self, store: &Mutex<LearnedWords>) {
        if let Some(last) = self.last.take() {
            lock(store).remove(&last.spelling, last.previous.as_deref());
        }
    }
}

/// The spelling to keep for `word`: a dictionary word capitalised only because it starts the
/// sentence ("The"), or typed in caps lock ("HELLO"), is learned in lower case. Words the
/// dictionary knows as written ("John", "USA") and words it doesn't know keep their capitals, as
/// does every word while no dictionary is open.
fn spelling_to_learn<L: Lookup + ?Sized>(
    lookup: Option<&mut L>,
    word: &[u16],
    starts_sentence: bool,
) -> Result<Vec<u16>, L::Error> {
    let Some(lookup) = lookup else {
        return Ok(word.to_vec());
    };
    let lower = lowercase(word);
    if lower == word || lookup.probability(word)? != NOT_A_PROBABILITY {
        return Ok(word.to_vec());
    }
    let case = WordCase::of(word);
    let capital_only_for_sentence = case == WordCase::Capitalized && starts_sentence;
    if !capital_only_for_sentence && case != WordCase::AllCaps {
        return Ok(word.to_vec());
    }
    if lookup.probability(&lower)? != NOT_A_PROBABILITY {
        Ok(lower)
    } else {
        Ok(word.to_vec())
    }
}

/// The store, locked. A panic aborts, so no lock is ever poisoned; take it back if one somehow
/// were.
fn lock(store: &Mutex<LearnedWords>) -> MutexGuard<'_, LearnedWords> {
    store.lock().unwrap_or_else(PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    /// A dictionary of four words.
    struct Known;

    impl Lookup for Known {
        type Error = Infallible;

        fn probability(&mut self, word: &[u16]) -> Result<i32, Infallible> {
            let known = ["the", "hello", "John", "later"]
                .iter()
                .any(|&known| utf16(known) == word);
            Ok(if known { 150 } else { NOT_A_PROBABILITY })
        }
    }

    struct Learning {
        store: Mutex<LearnedWords>,
        learner: WordLearner,
        dictionary: bool,
    }

    impl Learning {
        fn new() -> Learning {
            Learning {
                store: Mutex::new(LearnedWords::new(100, 100)),
                learner: WordLearner::default(),
                dictionary: true,
            }
        }

        fn lookup(&self) -> Option<Known> {
            self.dictionary.then_some(Known)
        }

        fn finished(&mut self, word: &str, text_before: &str, start: i32) -> bool {
            let mut lookup = self.lookup();
            let (word, before) = (utf16(word), utf16(text_before));
            let learned =
                self.learner
                    .finished(&self.store, lookup.as_mut(), &word, &before, start);
            learned.unwrap_or_else(|never| match never {})
        }

        fn swiped(&mut self, word: &str, text_before: &str) -> bool {
            let mut lookup = self.lookup();
            let (word, before) = (utf16(word), utf16(text_before));
            let learned = self
                .learner
                .swiped(&self.store, lookup.as_mut(), &word, &before);
            learned.unwrap_or_else(|never| match never {})
        }

        fn undo(&mut self, start: i32) -> bool {
            self.learner.undo(&self.store, start)
        }

        /// How many times `word` was learned after `previous` (`None`: a sentence start).
        fn count(&self, previous: Option<&str>, word: &str) -> i32 {
            let previous = previous.map(utf16);
            let store = self.store.lock().unwrap();
            store.pair_count(previous.as_deref(), previous.is_none(), &utf16(word))
        }

        /// The words learned after `previous` (`None`: a sentence start), spelled as stored.
        fn after(&self, previous: Option<&str>) -> Vec<String> {
            let previous = previous.map(utf16);
            let store = self.store.lock().unwrap();
            let words = store
                .followers(previous.as_deref(), previous.is_none(), 1, 5)
                .unwrap();
            words
                .iter()
                .map(|word| String::from_utf16_lossy(word))
                .collect()
        }
    }

    #[test]
    fn learns_a_finished_word_and_the_word_before_it() {
        let mut learning = Learning::new();
        assert!(learning.finished("later", "see you ", 8));
        assert_eq!(learning.count(Some("you"), "later"), 1);
        assert!(learning.finished("later", "talk to you ", 20));
        assert_eq!(learning.count(Some("you"), "later"), 2);
        // Once a word can't be said to come after anything, it is learned alone.
        assert!(learning.finished("later", "you, ", 5));
        assert_eq!(learning.count(Some("you"), "later"), 2);
        let store = learning.store.lock().unwrap();
        assert_eq!(
            store.spelling_of(&utf16("later")),
            Some(&utf16("later")[..])
        );
    }

    #[test]
    fn a_word_finished_twice_in_one_place_counts_once() {
        let mut learning = Learning::new();
        learning.finished("later", "see you ", 8);
        assert!(!learning.finished("later", "see you ", 8));
        assert_eq!(learning.count(Some("you"), "later"), 1);
        // Where a word starts is unknown below 0: it is never the same place.
        learning.finished("later", "see you ", -1);
        assert!(learning.finished("later", "see you ", -1));
        assert_eq!(learning.count(Some("you"), "later"), 3);
    }

    #[test]
    fn a_word_changed_in_place_replaces_the_old_one() {
        let mut learning = Learning::new();
        learning.finished("teh", "", 0);
        assert!(learning.finished("the", "", 0));
        assert_eq!(learning.count(None, "teh"), 0);
        assert_eq!(learning.count(None, "the"), 1);
        // Even with something that can't be learned.
        assert!(learning.finished("th3", "", 0));
        assert_eq!(learning.count(None, "the"), 0);
    }

    #[test]
    fn nothing_of_an_address_or_url_is_learned() {
        let mut learning = Learning::new();
        // "example" ended by the '.' of "example.com" is learned, until "com" shows the address.
        assert!(learning.finished("example", "see ", 4));
        assert!(learning.finished("com", "see example.", 12));
        assert_eq!(learning.count(Some("see"), "example"), 0);
        assert_eq!(learning.count(None, "com"), 0);
        // "bob" is never ended by its '@', which continues the word.
        assert!(!learning.finished("example", "write to bob@", 13));
        assert!(learning.finished("https", "read ", 5));
        assert!(learning.finished("example", "read https://", 13));
        assert!(!learning.finished("news", "read https://example.com/", 25));
        for word in ["example", "com", "https", "news"] {
            assert_eq!(
                learning.store.lock().unwrap().uses(&utf16(word)),
                0,
                "{word}"
            );
        }
        assert!(learning.after(Some("read")).is_empty());
        // The word after an address is learned, alone.
        assert!(learning.finished("wrote", "bob@example.com ", 16));
        assert_eq!(learning.store.lock().unwrap().uses(&utf16("wrote")), 1);
        assert!(learning.after(Some("bob@example.com")).is_empty());
    }

    #[test]
    fn undoing_an_auto_correction_takes_it_back() {
        let mut learning = Learning::new();
        learning.finished("the", "", 0);
        assert!(!learning.undo(5));
        assert!(!learning.undo(-1));
        assert!(learning.undo(0));
        assert_eq!(learning.count(None, "the"), 0);
        assert!(!learning.undo(0));
    }

    #[test]
    fn a_swiped_word_is_kept_only_once_typing_goes_on() {
        let mut learning = Learning::new();
        assert!(!learning.swiped("hello", ""));
        assert_eq!(learning.count(None, "hello"), 0);
        assert!(learning.finished("world", "hello ", 6));
        assert_eq!(learning.count(None, "hello"), 1);
        assert_eq!(learning.count(Some("hello"), "world"), 1);
        // Kept with no place, so the next word in its place doesn't replace it.
        learning.swiped("hello", "");
        learning.finished("there", "hello ", 6);
        assert!(learning.finished("world", "", -1));
        assert_eq!(learning.count(None, "hello"), 2);
    }

    #[test]
    fn a_second_swipe_keeps_the_first() {
        let mut learning = Learning::new();
        learning.swiped("hello", "");
        assert!(learning.swiped("world", "hello "));
        assert_eq!(learning.count(None, "hello"), 1);
        assert_eq!(learning.count(Some("hello"), "world"), 0);
    }

    #[test]
    fn a_swiped_word_finished_as_itself_counts_once() {
        let mut learning = Learning::new();
        learning.swiped("hello", "");
        assert!(learning.finished("hello", "", 0));
        assert_eq!(learning.count(None, "hello"), 1);
    }

    #[test]
    fn a_dropped_or_forgotten_swipe_is_never_learned() {
        let mut learning = Learning::new();
        learning.swiped("helo", "");
        learning.learner.drop_swiped();
        learning.finished("hello", "", 0);
        assert_eq!(learning.count(None, "helo"), 0);
        assert_eq!(learning.count(None, "hello"), 1);
        learning.swiped("helo", "");
        learning.learner.reset();
        learning.finished("hello", "", 0);
        assert_eq!(learning.count(None, "helo"), 0);
    }

    #[test]
    fn a_sentence_start_capital_is_not_learned() {
        let mut learning = Learning::new();
        learning.finished("The", "", 0);
        assert_eq!(learning.after(None), ["the"]);
        learning.finished("The", "see ", 4);
        assert_eq!(learning.after(Some("see")), ["The"]);
    }

    #[test]
    fn caps_lock_is_not_learned_for_dictionary_words() {
        let mut learning = Learning::new();
        learning.finished("HELLO", "say ", 4);
        learning.finished("NASA", "at ", 3);
        assert_eq!(learning.after(Some("say")), ["hello"]);
        assert_eq!(learning.after(Some("at")), ["NASA"]);
    }

    #[test]
    fn keeps_the_dictionarys_own_capitals() {
        let mut learning = Learning::new();
        learning.finished("John", "", 0);
        assert_eq!(learning.after(None), ["John"]);
    }

    #[test]
    fn a_sentence_start_does_not_recapitalise_a_word() {
        let mut learning = Learning::new();
        learning.finished("zorblax", "a ", 2);
        learning.finished("Zorblax", "", 0);
        let store = learning.store.lock().unwrap();
        assert_eq!(
            store.spelling_of(&utf16("Zorblax")),
            Some(&utf16("zorblax")[..])
        );
    }

    #[test]
    fn another_field_forgets_where_words_were() {
        let mut learning = Learning::new();
        learning.finished("teh", "", 0);
        learning.learner.reset();
        learning.finished("the", "", 0);
        assert_eq!(learning.count(None, "teh"), 1);
        assert_eq!(learning.count(None, "the"), 1);
    }

    #[test]
    fn only_words_are_learned() {
        let mut learning = Learning::new();
        assert!(!learning.finished("mp3", "", 0));
        assert!(!learning.swiped("a1", ""));
        assert!(!learning.finished("b2", "", 1));
        assert_eq!(learning.count(None, "mp3"), 0);
        assert_eq!(learning.count(None, "a1"), 0);
    }

    #[test]
    fn without_a_dictionary_words_are_kept_as_typed() {
        let mut learning = Learning::new();
        learning.dictionary = false;
        learning.finished("The", "", 0);
        learning.finished("HELLO", "say ", 4);
        assert_eq!(learning.after(None), ["The"]);
        assert_eq!(learning.after(Some("say")), ["HELLO"]);
    }
}
