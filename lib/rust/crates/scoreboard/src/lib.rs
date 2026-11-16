//! A typing scoreboard: how well the keyboard predicts, completes, corrects and swipes held-out
//! English, measured on the host through the same Rust the phone runs. See docs/rust.md
//! ("The typing scoreboard") for how to run it and docs/language-model-results.md for results.
//!
//! Host-only: nothing here ships in the app.

#![forbid(unsafe_code)]

pub mod geometry;
pub mod report;
pub mod rng;
pub mod swipe;
pub mod text;
pub mod touch;
pub mod typing;

use geometry::Keyboard;
use report::Tally;
use rng::Rng;
use std::sync::Arc;
use swipe::Swiper;
use text::Sentence;
use touch::Hand;
use typing::Engine;

/// A named set of test sentences.
#[derive(Debug)]
pub struct Source {
    pub name: String,
    pub sentences: Vec<Sentence>,
}

/// Seeds for the separate random streams, so adding one doesn't change the others.
const NEUTRAL_STREAM: u64 = 1;
const LEFT_THUMB_STREAM: u64 = 2;
const SWIPE_STREAM: u64 = 3;

/// Scores every source with `threads` threads, each with its own session on `dictionary`.
pub fn run(
    dictionary: Arc<[u8]>,
    keyboard: &Keyboard,
    sources: &[Source],
    seed: u64,
    threads: usize,
) -> Result<Vec<Tally>, String> {
    let probe = Engine::open(dictionary.clone()).ok_or("the dictionary did not open")?;
    let swiper = Swiper::new(probe.session(), keyboard)?;
    drop(probe);
    let threads = threads.max(1);
    let mut tallies = Vec::with_capacity(sources.len());
    for (s, source) in sources.iter().enumerate() {
        let parts = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..threads)
                .map(|t| {
                    let dictionary = dictionary.clone();
                    let swiper = &swiper;
                    scope.spawn(move || {
                        let mut engine = Engine::open(dictionary).expect("opened once already");
                        let mut tally = Tally::default();
                        let stream = (s as u64) << 32;
                        for (i, sentence) in source.sentences.iter().enumerate() {
                            if i % threads == t {
                                let at = (stream | i as u64, seed);
                                score(&mut engine, keyboard, swiper, sentence, at, &mut tally);
                            }
                        }
                        tally.typing_time.append(&mut engine.typing_time);
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

/// Scores every word of `sentence` (item `at.0` under seed `at.1`).
fn score(
    engine: &mut Engine,
    keyboard: &Keyboard,
    swiper: &Swiper,
    sentence: &Sentence,
    at: (u64, u64),
    tally: &mut Tally,
) {
    let (item, seed) = at;
    for w in 0..sentence.words.len() {
        let word = sentence.word(w);
        tally.words += 1;
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
        if let Some(path) = swiper.path(word) {
            let mut rng = Rng::for_item(seed ^ SWIPE_STREAM, item, w as u64);
            let guesses = swiper.guesses(&path, sentence.before(w), engine.session(), &mut rng);
            let at = guesses
                .iter()
                .take(typing::SHOWN)
                .position(|guess| chars::equals_ignore_case(guess, word));
            tally.swipes += 1;
            tally.swipe_top1 += usize::from(at == Some(0));
            tally.swipe_top3 += usize::from(at.is_some());
        }
    }
}
