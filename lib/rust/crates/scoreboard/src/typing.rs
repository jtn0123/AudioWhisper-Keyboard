//! Typing a sentence word by word through the keyboard's own suggestion code, as the phone does:
//! the same [`Session`] and [`Suggester`] the JNI calls, with the settings it passes by default
//! (offensive words blocked, the modest auto-correction threshold, eight candidates) and nothing
//! learned. The strip shows the first three.

use crate::geometry::Keyboard;
use crate::rng::Rng;
use crate::text::Sentence;
use crate::touch::{self, Hand, KeyResize, Typed};
use std::sync::Arc;
use std::time::{Duration, Instant};
use suggest::auto_correction::MODEST_THRESHOLD;
use suggest::language_model::Tuning;
use suggest::personal_words::PersonalWords;
use suggest::previous_words::PreviousWords;
use suggest::session::Session;
use suggest::suggester::{Arranged, Suggester, TypingState};
use suggest::tap::TapOffset;

/// `LatinLanguageProvider.suggest`'s `maxCandidateCount`.
pub const MAX_CANDIDATES: i32 = 8;
/// The candidates the strip shows.
pub const SHOWN: usize = 3;

/// One thread's keyboard.
#[derive(Debug)]
pub struct Engine {
    session: Session,
    personal: PersonalWords,
    /// Time spent asking for strips while typing a word, and how many were asked for.
    pub typing_time: Vec<Duration>,
    /// The same for the strips of next words after a space.
    pub prediction_time: Vec<Duration>,
    /// Time spent finding each tap's key after the plain hit test: the letter odds, when on, and
    /// the hit test that leans on them.
    pub hit_time: Vec<Duration>,
}

/// An n-gram model to score with, and how the session uses it.
pub type WithModel = (Arc<ngram::Model>, Tuning);

impl Engine {
    pub fn open(dictionary: Arc<[u8]>, model: Option<&WithModel>) -> Option<Engine> {
        let mut session = Session::open(dictionary)?;
        if let Some((model, tuning)) = model {
            session.attach_tuned(model.clone(), *tuning);
        }
        Some(Engine {
            session,
            personal: PersonalWords::new(Vec::new()),
            typing_time: Vec::new(),
            prediction_time: Vec::new(),
            hit_time: Vec::new(),
        })
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The candidate strip with `typed` typed after `before` (`taps` for each of its units).
    pub fn strip(
        &mut self,
        before: &[u16],
        typed: &[u16],
        taps: &[Option<TapOffset>],
        max: i32,
    ) -> Vec<Arranged> {
        let state = TypingState {
            typed: typed.to_vec(),
            previous_words: PreviousWords::from(before),
            is_cursor_mid_word: false,
        };
        let mut dictionary = self.session.request(&state.previous_words, true, taps);
        Suggester {
            dictionary: &mut dictionary,
            personal: &self.personal,
            learned: None,
            threshold: MODEST_THRESHOLD,
        }
        .suggest(&state, max)
        .unwrap_or_default()
    }

    /// Where `word` is in the strip shown for `typed` after `before`, if it is shown.
    fn shown_at(&mut self, before: &[u16], typed: &[u16], word: &[u16]) -> Option<usize> {
        let started = Instant::now();
        let strip = self.strip(before, typed, &[], MAX_CANDIDATES);
        if typed.is_empty() {
            self.prediction_time.push(started.elapsed());
        } else {
            self.typing_time.push(started.elapsed());
        }
        strip
            .iter()
            .take(SHOWN)
            .position(|candidate| chars::equals_ignore_case(&candidate.text, word))
    }

    /// Where word `index` of `sentence` is in the strip after the space before it.
    pub fn next_word(&mut self, sentence: &Sentence, index: usize) -> Option<usize> {
        self.shown_at(sentence.before(index), &[], sentence.word(index))
    }

    /// How many taps word `index` takes when the user taps it in the strip as soon as it shows up
    /// (counting that tap) instead of typing it all; its length if it never shows.
    pub fn keystrokes(&mut self, sentence: &Sentence, index: usize) -> usize {
        let word = sentence.word(index);
        let before = sentence.before(index);
        (0..word.len())
            .find(|&typed| self.shown_at(before, &word[..typed], word).is_some())
            .map_or(word.len(), |typed| typed + 1)
    }

    /// Types word `index` of `sentence` with simulated taps, then ends it as the space bar does:
    /// committing the auto-correction if there is one. With `resize`, each tap's key leans on the
    /// odds of each letter coming next after what was typed so far ([`touch::key_for`]).
    pub fn autocorrect(
        &mut self,
        keyboard: &Keyboard,
        sentence: &Sentence,
        index: usize,
        (hand, resize): (Hand, Option<&KeyResize>),
        rng: &mut Rng,
    ) -> Corrected {
        let word = sentence.word(index);
        let previous = PreviousWords::from(sentence.before(index));
        let mut typed: Vec<u16> = Vec::with_capacity(word.len());
        let mut offsets: Vec<Option<TapOffset>> = Vec::with_capacity(word.len());
        let mut plain = true;
        for &unit in word {
            let tap = match touch::land(keyboard, unit, hand, (0.0, 0.0), rng) {
                Some((hit, x, y)) => {
                    plain &= touch::typed(&keyboard.keys[hit], unit, x, y).unit == unit;
                    let key = self.key_for(keyboard, hit, (x, y), (&previous, &typed), resize);
                    touch::typed(&keyboard.keys[key], unit, x, y)
                }
                None => Typed { unit, offset: None },
            };
            typed.push(tap.unit);
            offsets.push(tap.offset);
        }
        // `autoCommitCandidateFor`: one candidate, and only if it is the auto-correction.
        let strip = self.strip(sentence.before(index), &typed, &offsets, 1);
        let committed = strip
            .into_iter()
            .find(|candidate| candidate.is_auto_correction)
            .map_or_else(|| typed.clone(), |candidate| candidate.text);
        let right = chars::equals_ignore_case(&committed, word);
        Corrected {
            clean: typed == word,
            changed: committed != typed,
            right,
            plain,
            // Set by the caller when it scores the plain hit test too.
            plain_right: right,
        }
    }
}

impl Engine {
    /// The key a tap at `at` on `hit` types after `typed` (and the words before), as
    /// [`touch::key_for`] finds it with this session's letter odds; `hit` without `resize`.
    pub fn key_for(
        &mut self,
        keyboard: &Keyboard,
        hit: usize,
        at: (f32, f32),
        (previous, typed): (&PreviousWords, &[u16]),
        resize: Option<&KeyResize>,
    ) -> usize {
        let started = Instant::now();
        let odds = resize
            .and(self.session.language_model())
            .and_then(|m| m.letter_odds(previous, typed));
        let key = touch::key_for(keyboard, hit, at, |_| None, odds.as_ref(), resize);
        self.hit_time.push(started.elapsed());
        key
    }
}

/// What auto-correction made of one word.
#[derive(Clone, Copy, Debug)]
pub struct Corrected {
    /// The taps typed the word as meant.
    pub clean: bool,
    /// Auto-correction replaced what was typed.
    pub changed: bool,
    /// The word committed is the word meant.
    pub right: bool,
    /// The plain hit test, before any retargeting, would have typed the word as meant.
    pub plain: bool,
    /// The word committed with the plain hit test is the word meant.
    pub plain_right: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::Sentence;
    use std::path::Path;

    fn engine() -> Engine {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../latin/tests/fixtures/aosp_main_en-US.dict");
        Engine::open(std::fs::read(path).unwrap().into(), None).unwrap()
    }

    #[test]
    fn common_words_need_fewer_taps_than_letters() {
        let mut engine = engine();
        let sentence = Sentence::new("I would like to thank you for everything.");
        let taps: usize = (0..sentence.words.len())
            .map(|i| engine.keystrokes(&sentence, i))
            .sum();
        let letters: usize = sentence.words.iter().map(|(s, e)| e - s).sum();
        assert!(taps < letters * 3 / 4, "{taps} of {letters}");
    }

    #[test]
    fn auto_correction_leaves_a_cleanly_typed_word_alone() {
        let mut engine = engine();
        let sentence = Sentence::new("Thank you very much.");
        let strip = engine.strip(sentence.before(1), &crate::text::utf16("yoi"), &[], 1);
        assert!(strip.first().is_some_and(|c| c.is_auto_correction));
        let strip = engine.strip(sentence.before(1), &crate::text::utf16("you"), &[], 1);
        assert!(strip.iter().all(|c| !c.is_auto_correction));
    }
}
