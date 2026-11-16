//! Simulated swipes through glide typing as the phone runs it: the dictionary's swipeable words
//! indexed by the `glide` crate (described character by character as `SwipeIndex.describe` does),
//! each scored at the frequency `SwipeWords.frequency` gives it, the eight best re-ranked by
//! `glide_ranker` with the words before them. The first is committed; the strip offers the next.
//!
//! Two simulators draw the finger, so that a change tuned to one is checked on the other:
//! [`trace`], a path drawn through the keys by rule, and [`lognormal`], the kinematic theory of
//! rapid human movements. Each swipes neutrally or with the left thumb. Of the finger's samples
//! the keyboard keeps a point, with the time it was drawn, only once it is a quarter of a key's
//! width from the last one it kept, as `StatisticalGlideTypingClassifier.addGesturePoint` does.

pub mod lognormal;
pub mod trace;

use crate::geometry::Keyboard;
use crate::rng::Rng;
use crate::touch::{self, Aim, Hand};
use glide::{CharKeys, Gesture, GlideIndex, Layout, NO_KEY};
use std::collections::HashSet;
use suggest::glide_ranker::{self, Query};
use suggest::learned_words::LearnedWords;
use suggest::previous_words::PreviousWords;
use suggest::raw::NOT_A_PROBABILITY;
use suggest::session::Session;
use suggest::swipe_words::{self, Boost, Curve, LEARNED_PROBABILITY};

/// `GlideTypingManager.MAX_SUGGESTION_COUNT`.
const MAX_GUESSES: usize = 8;

/// A sample of the finger: where it was, in pixels, and when, in milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub x: f32,
    pub y: f32,
    pub t: f32,
}

/// Which simulator draws a swipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Simulator {
    Trace,
    Lognormal,
}

impl Simulator {
    pub fn name(self) -> &'static str {
        match self {
            Simulator::Trace => "trace",
            Simulator::Lognormal => "lognormal",
        }
    }
}

/// Every way the scoreboard swipes, in the order it reports them.
pub const VARIANTS: [(Simulator, Hand); 4] = [
    (Simulator::Trace, Hand::Neutral),
    (Simulator::Trace, Hand::LeftThumb),
    (Simulator::Lognormal, Hand::Neutral),
    (Simulator::Lognormal, Hand::LeftThumb),
];

/// How swipes are decoded: what the scorer adds to FlorisBoard's classifier, and the frequency
/// curve it weighs words by.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decoder {
    pub glide: glide::Tuning,
    pub curve: Curve,
    /// How the user's own use raises a learned word.
    pub boost: Boost,
}

impl Decoder {
    /// As the keyboard decodes.
    pub const DEFAULT: Decoder = Decoder {
        glide: glide::Tuning::DEFAULT,
        curve: Curve::DEFAULT,
        boost: Boost::DEFAULT,
    };

    /// FlorisBoard's classifier on the flat frequency curve, as before these were measured.
    pub const ORIGINAL: Decoder = Decoder {
        glide: glide::Tuning::ORIGINAL,
        curve: Curve::FLAT,
        boost: Boost::NONE,
    };
}

/// The keys as the simulators see them.
#[derive(Debug)]
pub struct Board {
    pub keys: Vec<glide::Key>,
    /// How each hand aims at each key: neutral, then the left thumb.
    aims: Vec<[Aim; 2]>,
    /// Where the left thumb pivots.
    pub pivot: (f32, f32),
}

impl Board {
    fn new(keyboard: &Keyboard) -> Board {
        let keys = keyboard.glide_keys();
        let aims = keys
            .iter()
            .map(|key| {
                let (x, y) = (key.center_x, key.center_y);
                [Hand::Neutral, Hand::LeftThumb].map(|hand| touch::aim(keyboard, hand, x, y))
            })
            .collect();
        Board {
            keys,
            aims,
            pivot: touch::thumb_pivot(keyboard),
        }
    }

    /// How `hand` aims at key `k`.
    pub fn aim(&self, hand: Hand, k: usize) -> Aim {
        self.aims[k][usize::from(hand == Hand::LeftThumb)]
    }
}

/// The dictionary's swipeable words, indexed for the phone's keyboard.
#[derive(Debug)]
pub struct Swiper {
    index: GlideIndex,
    words: Vec<Vec<u16>>,
    frequencies: Vec<f32>,
    board: Board,
    codes: Vec<i32>,
    decoder: Decoder,
    /// The squared distance the finger moves before the keyboard keeps another point.
    threshold: f32,
}

impl Swiper {
    /// The dictionary's words on `keyboard`, weighed by `decoder` for a user who has `learned`.
    pub fn new(
        session: &Session,
        keyboard: &Keyboard,
        decoder: Decoder,
        learned: &LearnedWords,
    ) -> Result<Swiper, String> {
        let words: Vec<Vec<u16>> = session
            .words()
            .into_iter()
            .filter(|word| swipe_words::is_swipeable(word))
            .collect();
        let mut chars = Vec::new();
        let mut starts = vec![0u32];
        for word in &words {
            chars.extend_from_slice(word);
            starts.push(chars.len() as u32);
        }
        let key_codes: HashSet<i32> = keyboard.keys.iter().map(|key| key.code).collect();
        let table: Vec<CharKeys> = glide::distinct_units(&chars)
            .into_iter()
            .map(|unit| describe(unit, &key_codes))
            .collect();
        let index = GlideIndex::build(&chars, &starts, &table).map_err(|e| e.to_string())?;
        let frequencies = words
            .iter()
            .map(|word| {
                // As SwipeWords.tunedFrequency: the learned words weigh a word in the spelling
                // the user uses.
                let user = if learned.spelling_of(word) == Some(word.as_slice()) {
                    LEARNED_PROBABILITY
                } else {
                    NOT_A_PROBABILITY
                };
                let boost = decoder.boost.of(learned, word);
                decoder
                    .curve
                    .boosted_frequency(session.probability(word), user, boost)
                    as f32
            })
            .collect();
        let board = Board::new(keyboard);
        let first_width = board.keys.first().map_or(0.0, |key| key.width);
        let threshold = ((first_width / 4.0) as i32).pow(2) as f32;
        let codes = index.codes().to_vec();
        Ok(Swiper {
            index,
            words,
            frequencies,
            board,
            codes,
            decoder,
            threshold,
        })
    }

    /// The keys a swipe for `word` visits, a doubled letter once; none if a swipe can't spell it.
    pub fn path(&self, word: &[u16]) -> Option<Vec<usize>> {
        if !swipe_words::is_swipeable(word) {
            return None;
        }
        let keys = &self.board.keys;
        let codes: HashSet<i32> = keys.iter().map(|key| key.code).collect();
        let mut path: Vec<usize> = Vec::new();
        for &unit in word {
            let code = describe(unit, &codes).path_key;
            if code == NO_KEY {
                continue;
            }
            let key = keys.iter().position(|key| key.code == code)?;
            if path.last() != Some(&key) {
                path.push(key);
            }
        }
        Some(path).filter(|path| path.len() >= 2)
    }

    /// The finger's samples for a swipe along `path`, as `simulator` draws it with `hand`.
    pub fn draw(
        &self,
        simulator: Simulator,
        hand: Hand,
        path: &[usize],
        rng: &mut Rng,
    ) -> Vec<Sample> {
        match simulator {
            Simulator::Trace => trace::draw(&self.board, path, hand, rng),
            Simulator::Lognormal => lognormal::draw(&self.board, path, hand, rng),
        }
    }

    /// The words glide typing offers for a swipe drawn as `samples` after `before`, best first:
    /// the first is committed, the rest go to the strip.
    pub fn guesses(&self, samples: &[Sample], before: &[u16], session: &Session) -> Vec<&[u16]> {
        let (gesture, times) = self.keep(samples);
        let layout = Layout::new(&self.board.keys, &self.codes);
        let candidates = self.index.candidates(&gesture, &layout);
        let frequencies: Vec<f32> = candidates
            .iter()
            .map(|&w| self.frequencies[w as usize])
            .collect();
        let swipe = glide::Swipe {
            gesture: &gesture,
            times: &times,
        };
        let tuning = &self.decoder.glide;
        let Ok(ranking) = self.index.score_with(
            &candidates,
            &frequencies,
            swipe,
            &layout,
            MAX_GUESSES,
            tuning,
        ) else {
            return Vec::new();
        };
        let words: Vec<&[u16]> = ranking
            .words()
            .iter()
            .map(|&w| self.words[w as usize].as_slice())
            .collect();
        // Kotlin's ln of a Float.
        let costs: Vec<f32> = ranking
            .weights()
            .iter()
            .map(|&w| f64::from(w).ln() as f32)
            .collect();
        let previous = PreviousWords::from(&next(before));
        let order = glide_ranker::rank_from(&costs, &words, &previous, |query| {
            Ok::<i32, ()>(match query {
                Query::Ngram(previous, word) => session.context_probability(previous, word),
                Query::Probability(word) => session.probability(word),
                Query::Pair(..) => 0,
            })
        });
        match order {
            Ok(Some(order)) => order.into_iter().map(|i| words[i]).collect(),
            _ => words,
        }
    }

    /// The points the keyboard keeps of `samples`, and their times.
    fn keep(&self, samples: &[Sample]) -> (Gesture, Vec<f32>) {
        let mut gesture = Gesture::new();
        let mut times = Vec::new();
        let mut last: Option<(f32, f32)> = None;
        for &Sample { x, y, t } in samples {
            let keep =
                last.is_none_or(|(lx, ly)| (lx - x).powi(2) + (ly - y).powi(2) > self.threshold);
            if keep && gesture.len() < glide::MAX_POINTS {
                gesture.push(x, y);
                times.push(t);
                last = Some((x, y));
            }
        }
        (gesture, times)
    }
}

/// `NlpState.call("next", ...)`: the text before the next word, a space after a word.
fn next(before: &[u16]) -> Vec<u16> {
    let mut text = before.to_vec();
    if text.last().is_some_and(|&u| !chars::is_whitespace(u)) {
        text.push(u16::from(b' '));
    }
    text
}

/// `SwipeIndex.describe`: the key a path visits for `unit` (its lower case, or that without its
/// accent), and the accentless key a word starting or ending with it is filed under.
fn describe(unit: u16, keys: &HashSet<i32>) -> CharKeys {
    let lower = chars::to_lower(unit);
    let base = chars::code_point::decomposed_first(lower) as i32;
    let path_key = if keys.contains(&i32::from(lower)) {
        i32::from(lower)
    } else if keys.contains(&base) {
        base
    } else {
        NO_KEY
    };
    let end_key = if keys.contains(&base) { base } else { NO_KEY };
    CharKeys {
        unit,
        lower,
        path_key,
        end_key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::{string, utf16};
    use std::path::Path;
    use std::sync::Arc;

    fn setup() -> (Session, Keyboard) {
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = here.join("../../../../app/src/main/assets");
        // The AOSP dictionary, whose ranking the test below states.
        let bytes: Arc<[u8]> =
            std::fs::read(here.join("../latin/tests/fixtures/aosp_main_en-US.dict"))
                .unwrap()
                .into();
        (
            Session::open(bytes).unwrap(),
            Keyboard::phone(&root).unwrap(),
        )
    }

    #[test]
    fn a_swipe_finds_a_common_word() {
        let (session, keyboard) = setup();
        let swiper = Swiper::new(
            &session,
            &keyboard,
            Decoder::DEFAULT,
            &LearnedWords::new(0, 0),
        )
        .unwrap();
        let path = swiper.path(&utf16("hello")).unwrap();
        assert_eq!(path.len(), 4);
        // In the top three, as the phone's own dictionary ranks "help" (149) above "hello" (120),
        // however it is drawn; the lognormal finger is the sloppier.
        for (simulator, hand) in VARIANTS {
            let mut found = 0;
            for i in 0..20 {
                let samples = swiper.draw(simulator, hand, &path, &mut Rng::new(i));
                let guesses = swiper.guesses(&samples, &utf16("Well, "), &session);
                found += usize::from(guesses.iter().take(3).any(|g| string(g) == "hello"));
            }
            let least = if simulator == Simulator::Trace {
                18
            } else {
                15
            };
            assert!(found >= least, "{simulator:?} {hand:?}: {found}");
        }
    }

    #[test]
    fn the_keyboard_keeps_points_a_quarter_key_apart_with_their_times() {
        let (session, keyboard) = setup();
        let swiper = Swiper::new(
            &session,
            &keyboard,
            Decoder::DEFAULT,
            &LearnedWords::new(0, 0),
        )
        .unwrap();
        let samples: Vec<Sample> = (0..100)
            .map(|i| Sample {
                x: i as f32 * 10.0,
                y: 300.0,
                t: i as f32 * 4.0,
            })
            .collect();
        let (gesture, times) = swiper.keep(&samples);
        // A quarter of a key is 33 px, to the pixel: every fourth sample, 40 px on.
        assert_eq!(gesture.len(), 25);
        assert_eq!(times.len(), 25);
        assert_eq!((gesture.x(1), times[1]), (40.0, 16.0));
    }

    #[test]
    fn describes_accents_as_the_keyboard_does() {
        let keys: HashSet<i32> = "abcde".chars().map(|c| c as i32).collect();
        let e = describe(0xC9, &keys);
        assert_eq!(
            (e.lower, e.path_key, e.end_key),
            (0xE9, 'e' as i32, 'e' as i32)
        );
        assert_eq!(describe(u16::from(b'\''), &keys).path_key, NO_KEY);
        assert_eq!(string(&next(&utf16("so"))), "so ");
    }
}
