//! A typing scoreboard: how well the keyboard predicts, completes, corrects and swipes held-out
//! English, measured on the host through the same Rust the phone runs. See docs/rust.md
//! ("The typing scoreboard") for how to run it and docs/language-model-results.md for results.
//!
//! Host-only: nothing here ships in the app.

#![forbid(unsafe_code)]

pub mod geometry;
pub mod learning;
pub mod learning_run;
pub mod report;
pub mod rng;
pub use lexicon::text;
pub mod swipe;
pub mod touch;
pub mod typing;

use geometry::Keyboard;
use report::Tally;
use rng::Rng;
use std::sync::Arc;
use std::time::Instant;
use suggest::learned_words::{LearnedWords, WORD_CAPACITY};
use suggest::raw::NOT_A_PROBABILITY;
use suggest::session::Session;
use suggest::word_case::WordCase;
use swipe::{Decoder, Swiper, VARIANTS};
use text::Sentence;
use touch::{Hand, KeyResize};
use typing::{Engine, WithModel};

/// A named set of test sentences.
#[derive(Debug)]
pub struct Source {
    pub name: String,
    pub sentences: Vec<Sentence>,
}

/// The typists auto-correction is scored for, each with its own stream of taps; in the order of
/// [`report::Tally::hands`].
pub const HANDS: [(Hand, u64); 3] = [
    (Hand::Neutral, NEUTRAL_STREAM),
    (Hand::LeftThumb, LEFT_THUMB_STREAM),
    (Hand::RightThumb, RIGHT_THUMB_STREAM),
];

/// Seeds for the separate random streams, so adding one doesn't change the others.
const NEUTRAL_STREAM: u64 = 1;
const LEFT_THUMB_STREAM: u64 = 2;
const RIGHT_THUMB_STREAM: u64 = 7;
/// One for each of [`VARIANTS`], in order: the first was the only one once.
const SWIPE_STREAMS: [u64; 4] = [3, 4, 5, 6];

/// How a run scores.
#[derive(Clone, Copy, Debug)]
pub struct Settings {
    /// The seed for taps and swipes.
    pub seed: u64,
    pub threads: usize,
    /// How swipes are decoded.
    pub decoder: Decoder,
    /// Score swipes only, not next words, keystrokes or auto-correction.
    pub swipe_only: bool,
    /// The auto-correction threshold, `None` for off (`suggest::auto_correction::Strength`).
    pub threshold: Option<f32>,
    /// Score auto-correction only, not next words, keystrokes or swipes.
    pub correction_only: bool,
    /// The hit test leans on the letter odds, so (`keys::resize`).
    pub resize: Option<KeyResize>,
}

/// Scores every source with `settings.threads` threads, each with its own session on
/// `dictionary` (and `model`, shared), for a user who has typed `history` before.
pub fn run(
    dictionary: Arc<[u8]>,
    model: Option<&WithModel>,
    keyboard: &Keyboard,
    sources: &[Source],
    settings: Settings,
    history: &[Sentence],
) -> Result<Vec<Tally>, String> {
    let probe = Engine::open(dictionary.clone(), model).ok_or("the dictionary did not open")?;
    let learned = learn(probe.session(), history);
    let swiper = Swiper::new(probe.session(), keyboard, settings.decoder, &learned)?;
    drop(probe);
    let Settings { seed, threads, .. } = settings;
    let threads = threads.max(1);
    let mut tallies = Vec::with_capacity(sources.len());
    for (s, source) in sources.iter().enumerate() {
        let parts = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads)
                .map(|t| {
                    let dictionary = dictionary.clone();
                    let swiper = &swiper;
                    scope.spawn(move || {
                        let mut engine =
                            Engine::open(dictionary, model).expect("opened once already");
                        engine.threshold = settings.threshold;
                        let mut tally = Tally::default();
                        let stream = (s as u64) << 32;
                        for (i, sentence) in source.sentences.iter().enumerate() {
                            if i % threads == t {
                                let at = (stream | i as u64, seed);
                                let tally = &mut tally;
                                let typing = (keyboard, swiper);
                                score(&mut engine, typing, sentence, at, &settings, tally);
                            }
                        }
                        tally.typing_time.append(&mut engine.typing_time);
                        tally.prediction_time.append(&mut engine.prediction_time);
                        tally.hit_time.append(&mut engine.hit_time);
                        tally
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| "a scoring thread panicked".to_string())
                })
                .collect::<Result<Vec<Tally>, String>>()
        })?;
        let mut tally = Tally::default();
        for part in parts {
            tally.add(part);
        }
        tallies.push(tally);
    }
    Ok(tallies)
}

/// What the keyboard learns from a user typing `sentences`: each word in the spelling its word
/// learner keeps (`suggest::word_learner`), so a capital that only starts a sentence is dropped
/// when the dictionary has the word without it.
pub fn learn(session: &Session, sentences: &[Sentence]) -> LearnedWords {
    let mut learned = LearnedWords::new(WORD_CAPACITY, 0);
    for sentence in sentences {
        for w in 0..sentence.words.len() {
            let word = sentence.word(w);
            let case = WordCase::of(word);
            let starts_sentence = w == 0;
            let lower = chars::lowercase(word);
            let recased =
                (case == WordCase::Capitalized && starts_sentence) || case == WordCase::AllCaps;
            let spelling = if lower != word
                && recased
                && session.probability(word) == NOT_A_PROBABILITY
                && session.probability(&lower) != NOT_A_PROBABILITY
            {
                lower
            } else {
                word.to_vec()
            };
            let keep = starts_sentence && spelling == word && case == WordCase::Capitalized;
            learned.add(&spelling, None, keep);
        }
    }
    learned
}

/// Where a word stands with the keyboard's vocabulary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rarity {
    /// The n-gram model doesn't have it, so the letter odds don't know it.
    pub outside_model: bool,
    /// Nor does the dictionary.
    pub outside_dictionary: bool,
}

impl Rarity {
    /// `word` as the session looks it up: as written, else lower-cased.
    pub fn of(session: &Session, word: &[u16]) -> Rarity {
        let known = |w: &[u16]| session.probability(w) != NOT_A_PROBABILITY;
        let outside_dictionary = !known(word) && !known(&chars::lowercase(word));
        let outside_model = (session.language_model())
            .map_or(outside_dictionary, |m| m.model().word(word).is_none());
        Rarity {
            outside_model,
            outside_dictionary,
        }
    }
}

/// Scores every word of `sentence` (item `at.0` under seed `at.1`), as `settings` say.
fn score(
    engine: &mut Engine,
    (keyboard, swiper): (&Keyboard, &Swiper),
    sentence: &Sentence,
    at: (u64, u64),
    settings: &Settings,
    tally: &mut Tally,
) {
    let (item, seed) = at;
    for w in 0..sentence.words.len() {
        let word = sentence.word(w);
        tally.words += 1;
        if !settings.swipe_only {
            if !settings.correction_only {
                let shown = engine.next_word(sentence, w);
                tally.next_top1 += usize::from(shown == Some(0));
                tally.next_top3 += usize::from(shown.is_some());
                tally.letters += word.len();
                tally.keystrokes += engine.keystrokes(sentence, w);
            }
            let rarity = Rarity::of(engine.session(), word);
            for (h, (hand, stream)) in HANDS.into_iter().enumerate() {
                let rng = Rng::for_item(seed ^ stream, item, w as u64);
                let typist = (hand, settings.resize.as_ref());
                let mut corrected =
                    engine.autocorrect(keyboard, sentence, w, typist, &mut rng.clone());
                if settings.resize.is_some() {
                    // The same taps through the plain hit test, to count what each gets right; its
                    // time is not the one scored.
                    let timed = engine.hit_time.len();
                    let plain =
                        engine.autocorrect(keyboard, sentence, w, (hand, None), &mut rng.clone());
                    corrected.plain_right = plain.right;
                    engine.hit_time.truncate(timed);
                }
                tally.hands[h].add(corrected);
                if rarity.outside_model {
                    tally.outside_model[h].add(corrected);
                }
                if rarity.outside_dictionary {
                    tally.outside_dictionary[h].add(corrected);
                }
            }
            tally
                .no_apostrophes
                .add(engine.without_apostrophes(sentence, w));
        }
        if settings.correction_only {
            continue;
        }
        let Some(path) = swiper.path(word) else {
            continue;
        };
        for (v, (simulator, hand)) in VARIANTS.into_iter().enumerate() {
            let mut rng = Rng::for_item(seed ^ SWIPE_STREAMS[v], item, w as u64);
            let samples = swiper.draw(simulator, hand, &path, &mut rng);
            let started = Instant::now();
            let guesses = swiper.guesses(&samples, sentence.before(w), engine.session());
            tally.swipe_time.push(started.elapsed());
            let at = guesses
                .iter()
                .take(typing::SHOWN)
                .position(|guess| chars::equals_ignore_case(guess, word));
            tally.swipe[v].add(at);
        }
    }
}
