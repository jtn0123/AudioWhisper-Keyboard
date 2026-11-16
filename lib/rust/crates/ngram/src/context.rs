//! Which of the model's words a spelling is, and which words a prediction follows. The builder
//! counts the corpus through these same functions, so a word gets the same id when it is counted
//! and when the keyboard asks about it.

/// A vocabulary that can be asked about spellings: the model file, or the builder's.
pub trait Spellings {
    /// The id of exactly this spelling.
    fn exact(&self, word: &[u16]) -> Option<u32>;
    /// Whether a sentence's first word spelled as word `id` is taken as its lower-case spelling.
    fn prefers_lower_at_start(&self, id: u32) -> bool;
    /// The sentence start's id.
    fn sentence_start(&self) -> u32;
}

/// The words a next word follows, newest first: none, the word before it, or the two before it.
/// A sentence start counts as a word.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Context {
    pub newest: Option<u32>,
    /// The word before `newest`, if it is known; never without `newest`.
    pub older: Option<u32>,
}

impl Context {
    pub fn is_empty(&self) -> bool {
        self.newest.is_none()
    }
}

/// The word `word` is: the keyboard's lookup (as written, else lower-cased), except that a
/// sentence's first word takes the spelling the corpus mostly writes mid-sentence.
pub fn resolve(spellings: &impl Spellings, word: &[u16], sentence_initial: bool) -> Option<u32> {
    let lower = || {
        let lower = chars::lowercase(word);
        (lower != word).then(|| spellings.exact(&lower)).flatten()
    };
    match spellings.exact(word) {
        Some(id) if sentence_initial && spellings.prefers_lower_at_start(id) => {
            lower().or(Some(id))
        }
        Some(id) => Some(id),
        None => lower(),
    }
}

/// The context for a word after `newest_first` (the keyboard's `PreviousWords`: at most three
/// words, most recent first) where `sentence_start` says a sentence starts before the oldest of
/// them. A word the model doesn't know ends the context there.
pub fn context(
    spellings: &impl Spellings,
    newest_first: &[Vec<u16>],
    sentence_start: bool,
) -> Context {
    let initial = |i: usize| sentence_start && i + 1 == newest_first.len();
    let newest = match newest_first.first() {
        None => sentence_start.then(|| spellings.sentence_start()),
        Some(word) => resolve(spellings, word, initial(0)),
    };
    if newest.is_none() {
        return Context::default();
    }
    let older = match newest_first.get(1) {
        Some(word) => resolve(spellings, word, initial(1)),
        None => (newest_first.len() == 1 && sentence_start).then(|| spellings.sentence_start()),
    };
    Context { newest, older }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    struct Words(HashMap<Vec<u16>, u32>, Vec<u32>);

    impl Spellings for Words {
        fn exact(&self, word: &[u16]) -> Option<u32> {
            self.0.get(word).copied()
        }
        fn prefers_lower_at_start(&self, id: u32) -> bool {
            self.1.contains(&id)
        }
        fn sentence_start(&self) -> u32 {
            99
        }
    }

    fn utf16(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }

    fn words() -> Words {
        let list = ["will", "Will", "go", "Paris", "the"];
        let ids = list.iter().enumerate().map(|(i, w)| (utf16(w), i as u32));
        // "Will" (1) is mostly "will" mid-sentence.
        Words(ids.collect(), vec![1])
    }

    fn previous(list: &[&str]) -> Vec<Vec<u16>> {
        list.iter().map(|w| utf16(w)).collect()
    }

    #[test]
    fn a_sentence_start_takes_the_usual_spelling() {
        let w = words();
        assert_eq!(resolve(&w, &utf16("Will"), false), Some(1));
        assert_eq!(resolve(&w, &utf16("Will"), true), Some(0));
        assert_eq!(resolve(&w, &utf16("Paris"), true), Some(3));
        assert_eq!(resolve(&w, &utf16("The"), false), Some(4));
        assert_eq!(resolve(&w, &utf16("Rome"), true), None);
    }

    #[test]
    fn contexts_follow_the_previous_words() {
        let w = words();
        let at = |newest, older| Context { newest, older };
        assert_eq!(context(&w, &[], false), Context::default());
        assert_eq!(context(&w, &[], true), at(Some(99), None));
        assert_eq!(
            context(&w, &previous(&["Will"]), true),
            at(Some(0), Some(99))
        );
        assert_eq!(context(&w, &previous(&["Will"]), false), at(Some(1), None));
        assert_eq!(
            context(&w, &previous(&["go", "Will"]), true),
            at(Some(2), Some(0))
        );
        assert_eq!(
            context(&w, &previous(&["go", "Will", "the"]), false),
            at(Some(2), Some(1))
        );
        // An unknown word ends the context.
        assert_eq!(
            context(&w, &previous(&["Rome", "the"]), false),
            Context::default()
        );
        assert_eq!(
            context(&w, &previous(&["go", "Rome"]), true),
            at(Some(2), None)
        );
    }
}
