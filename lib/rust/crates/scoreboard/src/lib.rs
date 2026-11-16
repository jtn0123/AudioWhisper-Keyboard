//! A typing scoreboard: how well the keyboard predicts, completes, corrects and swipes held-out
//! English, measured on the host through the same Rust the phone runs. See docs/rust.md
//! ("The typing scoreboard") for how to run it and docs/language-model-results.md for results.
//!
//! Host-only: nothing here ships in the app.

#![forbid(unsafe_code)]

pub mod geometry;
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
use touch::Hand;
use typing::{Engine, WithModel};

/// A named set of test sentences.
#[derive(Debug)]
pub struct Source {
    pub name: String,
    pub sentences: Vec<Sentence>,
}

/// Seeds for the separate random streams, so adding one doesn't change the others.
const NEUTRAL_STREAM: u64 = 1;
const LEFT_THUMB_STREAM: u64 = 2;
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
                        let mut tally = Tally::default();
                        let stream = (s as u64) << 32;
                        for (i, sentence) in source.sentences.iter().enumerate() {
                            if i % threads == t {
                                let at = (stream | i as u64, seed);
                                let only = settings.swipe_only;
                                let tally = &mut tally;
                                score(&mut engine, keyboard, swiper, sentence, at, only, tally);
                            }
                        }
                        tally.typing_time.append(&mut engine.typing_time);
                        tally.prediction_time.append(&mut engine.prediction_time);
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

/// Scores every word of `sentence` (item `at.0` under seed `at.1`); only its swipes if
/// `swipe_only`.
fn score(
    engine: &mut Engine,
    keyboard: &Keyboard,
    swiper: &Swiper,
    sentence: &Sentence,
    at: (u64, u64),
    swipe_only: bool,
    tally: &mut Tally,
) {
    let (item, seed) = at;
    for w in 0..sentence.words.len() {
        let word = sentence.word(w);
        tally.words += 1;
        if !swipe_only {
            let shown = engine.next_word(sentence, w);
            tally.next_top1 += usize::from(shown == Some(0));
            tally.next_top3 += usize::from(shown.is_some());
            tally.letters += word.len();
            tally.keystrokes += engine.keystrokes(sentence, w);
            for (hand, stream, ac) in [
                (Hand::Neutral, NEUTRAL_STREAM, &mut tally.neutral),
                (Hand::LeftThumb, LEFT_THUMB_STREAM, &mut tally.left_thumb),
            ] {
                let mut rng = Rng::for_item(seed ^ stream, item, w as u64);
                ac.add(engine.autocorrect(keyboard, sentence, w, hand, &mut rng));
            }
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
