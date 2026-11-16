//! Simulated swipes through glide typing as the phone runs it: the dictionary's swipeable words
//! indexed by the `glide` crate (described character by character as `SwipeIndex.describe` does),
//! each scored at the frequency `SwipeWords.frequency` gives it, the eight best re-ranked by
//! `glide_ranker` with the words before them. The first is committed; the strip offers the next.
//!
//! A swipe aims at each key of the word in turn (a doubled letter once), missing its visible centre
//! by a Gaussian of a fifth of the key across and down. Corners are cut: each turn is pulled
//! towards the midpoint of its neighbours by up to a third of a key (uniformly drawn, and never
//! past the midpoint). The finger's path between them
//! is drawn as a Catmull-Rom curve sampled every 4 px, with a little tremor, and the keyboard
//! keeps a point only once it is a quarter of a key's width from the last one it kept, as
//! `StatisticalGlideTypingClassifier.addGesturePoint` does.

use crate::geometry::Keyboard;
use crate::rng::Rng;
use glide::{CharKeys, Gesture, GlideIndex, Layout, NO_KEY};
use std::collections::HashSet;
use suggest::glide_ranker::{self, Query};
use suggest::previous_words::PreviousWords;
use suggest::raw::NOT_A_PROBABILITY;
use suggest::session::Session;
use suggest::swipe_words;

/// `GlideTypingManager.MAX_SUGGESTION_COUNT`.
const MAX_GUESSES: usize = 8;
/// How far an aim misses a key's centre, in visible key widths and heights.
const AIM_SPREAD: f32 = 0.2;
/// How far turns are pulled in towards their neighbours' midpoint at most, in key widths.
const CUT_MAX: f32 = 0.33;
/// The spacing of the drawn path, and the tremor along it, in pixels.
const STEP: f32 = 4.0;
const TREMOR: f32 = 1.5;

/// The dictionary's swipeable words, indexed for the phone's keyboard.
#[derive(Debug)]
pub struct Swiper {
    index: GlideIndex,
    words: Vec<Vec<u16>>,
    frequencies: Vec<f32>,
    keys: Vec<glide::Key>,
    codes: Vec<i32>,
    /// The squared distance the finger moves before the keyboard keeps another point.
    threshold: f32,
}

impl Swiper {
    pub fn new(session: &Session, keyboard: &Keyboard) -> Result<Swiper, String> {
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
            .map(|word| swipe_words::frequency(session.probability(word), NOT_A_PROBABILITY) as f32)
            .collect();
        let keys = keyboard.glide_keys();
        let first_width = keys.first().map_or(0.0, |key| key.width);
        let threshold = ((first_width / 4.0) as i32).pow(2) as f32;
        let codes = index.codes().to_vec();
        Ok(Swiper {
            index,
            words,
            frequencies,
            keys,
            codes,
            threshold,
        })
    }

    /// The keys a swipe for `word` visits, a doubled letter once; none if a swipe can't spell it.
    pub fn path(&self, word: &[u16]) -> Option<Vec<usize>> {
        if !swipe_words::is_swipeable(word) {
            return None;
        }
        let codes: HashSet<i32> = self.keys.iter().map(|key| key.code).collect();
        let mut path: Vec<usize> = Vec::new();
        for &unit in word {
            let code = describe(unit, &codes).path_key;
            if code == NO_KEY {
                continue;
            }
            let key = self.keys.iter().position(|key| key.code == code)?;
            if path.last() != Some(&key) {
                path.push(key);
            }
        }
        Some(path).filter(|path| path.len() >= 2)
    }

    /// The words glide typing offers for a simulated swipe along `path` after `before`, best first:
    /// the first is committed, the rest go to the strip.
    pub fn guesses(
        &self,
        path: &[usize],
        before: &[u16],
        session: &Session,
        rng: &mut Rng,
    ) -> Vec<&[u16]> {
        let gesture = self.gesture(path, rng);
        let layout = Layout::new(&self.keys, &self.codes);
        let candidates = self.index.candidates(&gesture, &layout);
        let frequencies: Vec<f32> = candidates
            .iter()
            .map(|&w| self.frequencies[w as usize])
            .collect();
        let Ok(ranking) =
            self.index
                .score(&candidates, &frequencies, &gesture, &layout, MAX_GUESSES)
        else {
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
                Query::Ngram(previous, word) => session.ngram_probability(previous, word),
                Query::Probability(word) => session.probability(word),
                Query::Pair(..) => 0,
            })
        });
        match order {
            Ok(Some(order)) => order.into_iter().map(|i| words[i]).collect(),
            _ => words,
        }
    }

    /// The points the keyboard keeps of a swipe along `path`.
    fn gesture(&self, path: &[usize], rng: &mut Rng) -> Gesture {
        let mut aims: Vec<(f32, f32)> = path
            .iter()
            .map(|&k| {
                let key = &self.keys[k];
                (
                    key.center_x + rng.gaussian() as f32 * AIM_SPREAD * key.width,
                    key.center_y + rng.gaussian() as f32 * AIM_SPREAD * key.height,
                )
            })
            .collect();
        let corners = aims.clone();
        for i in 1..aims.len().saturating_sub(1) {
            let mid_x = (corners[i - 1].0 + corners[i + 1].0) / 2.0;
            let mid_y = (corners[i - 1].1 + corners[i + 1].1) / 2.0;
            let (dx, dy) = (mid_x - aims[i].0, mid_y - aims[i].1);
            let distance = dx.hypot(dy);
            if distance > 0.0 {
                let width = self.keys[path[i]].width;
                let cut = (rng.uniform() as f32 * CUT_MAX * width).min(distance) / distance;
                aims[i].0 += dx * cut;
                aims[i].1 += dy * cut;
            }
        }
        let mut gesture = Gesture::new();
        let mut last: Option<(f32, f32)> = None;
        for (x, y) in curve(&aims) {
            let x = x + rng.gaussian() as f32 * TREMOR;
            let y = y + rng.gaussian() as f32 * TREMOR;
            let keep =
                last.is_none_or(|(lx, ly)| (lx - x).powi(2) + (ly - y).powi(2) > self.threshold);
            if keep {
                gesture.push(x, y);
                last = Some((x, y));
            }
        }
        gesture
    }
}

/// A Catmull-Rom curve through `points`, sampled about every [`STEP`] pixels.
fn curve(points: &[(f32, f32)]) -> Vec<(f32, f32)> {
    let at = |i: isize| points[i.clamp(0, points.len() as isize - 1) as usize];
    let mut out = vec![points[0]];
    for i in 0..points.len() as isize - 1 {
        let (p0, p1, p2, p3) = (at(i - 1), at(i), at(i + 1), at(i + 2));
        let length = (p2.0 - p1.0).hypot(p2.1 - p1.1);
        let steps = ((length / STEP).ceil() as usize).max(1);
        for s in 1..=steps {
            let t = s as f32 / steps as f32;
            let blend = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b
                    + (c - a) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t * t
                    + (3.0 * b - a - 3.0 * c + d) * t * t * t)
            };
            out.push((blend(p0.0, p1.0, p2.0, p3.0), blend(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    out
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
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main/assets");
        let bytes: Arc<[u8]> = std::fs::read(root.join("dicts/main_en-US.dict"))
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
        let swiper = Swiper::new(&session, &keyboard).unwrap();
        let path = swiper.path(&utf16("hello")).unwrap();
        assert_eq!(path.len(), 4);
        // In the top three, as the phone's own dictionary ranks "help" (149) above "hello" (120).
        let mut found = 0;
        for i in 0..20 {
            let guesses = swiper.guesses(&path, &utf16("Well, "), &session, &mut Rng::new(i));
            found += usize::from(guesses.iter().take(3).any(|g| string(g) == "hello"));
        }
        assert!(found >= 18, "{found}");
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
