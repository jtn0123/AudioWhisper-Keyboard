//! Scoring a swipe: which words to try, and how likely each one is.
//!
//! A word's ideal path runs through the centres of its keys. Each candidate's path is compared
//! with the swipe twice: by shape (both scaled into a unit box) and by location (as drawn), each
//! distance turned into a probability by a Gaussian, then weighed by the word's frequency. See
//! Étienne Desticourt's write-up at https://github.com/AnySoftKeyboard/AnySoftKeyboard/pull/1870.

use std::fmt;

use crate::gesture::{Gesture, SAMPLES, distance};
use crate::index::GlideIndex;
use crate::layout::Layout;
use crate::tuning::{self, Tuning};

/// How far a word's path may be from the swipe's length, in key radii, and still be scored.
const LENGTH_THRESHOLD: f64 = 8.42;

/// The spread of shape distances between swipes of the same word; shapes are normalized, so this
/// holds for any keyboard size.
const SHAPE_STD: f32 = 22.08;

/// The spread of location distances between swipes of the same word, in key radii.
const LOCATION_STD: f32 = 0.5109;

/// Why a swipe could not be scored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScoreError {
    /// There is not one frequency per candidate.
    FrequencyCount,
    /// A candidate is not a word in the index.
    NoSuchWord(u32),
}

impl fmt::Display for ScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScoreError::FrequencyCount => f.write_str("each candidate needs one frequency"),
            ScoreError::NoSuchWord(word) => write!(f, "word {word} is not in the index"),
        }
    }
}

impl std::error::Error for ScoreError {}

/// The likeliest words so far, most likely (lowest weight) first; a word is in once, at its best.
#[derive(Clone, Debug, PartialEq)]
pub struct Ranking {
    max: usize,
    words: Vec<u32>,
    weights: Vec<f32>,
}

impl Ranking {
    pub fn new(max: usize) -> Self {
        Ranking {
            max,
            words: Vec::with_capacity(max + 1),
            weights: Vec::with_capacity(max + 1),
        }
    }

    pub fn words(&self) -> &[u32] {
        &self.words
    }

    /// Each word's weight: the inverse of its likelihood, so lower is likelier.
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    /// Whether a word at `weight` would not get in: the ranking is full of likelier words.
    pub(crate) fn would_drop(&self, weight: f32) -> bool {
        self.words.len() >= self.max && self.weights.last().is_some_and(|&last| last <= weight)
    }

    /// Adds `word` at `weight`, keeping the order and at most `max` words. A word already in at
    /// an equal or better weight stays; at a worse one, it moves up.
    pub fn insert(&mut self, word: u32, weight: f32) {
        let mut at = 0;
        while at < self.words.len() && self.weights[at] <= weight {
            if self.words[at] == word {
                return;
            }
            at += 1;
        }
        if at >= self.max {
            return;
        }
        // A word with a repeated letter is scored twice, with and without loops; the search above
        // stops before a less likely copy, so that copy goes here.
        if let Some(copy) = self.words[at..].iter().position(|&w| w == word) {
            self.words.remove(at + copy);
            self.weights.remove(at + copy);
        }
        self.words.insert(at, word);
        self.weights.insert(at, weight);
        self.words.truncate(self.max);
        self.weights.truncate(self.max);
    }
}

/// A swipe's points, and when each was drawn.
#[derive(Clone, Copy, Debug)]
pub struct Swipe<'a> {
    pub gesture: &'a Gesture,
    /// Each point's time in milliseconds, or none; [`Tuning::dwell`] needs them.
    pub times: &'a [f32],
}

impl GlideIndex {
    /// The words worth scoring for `swipe`, in the order to score them: those starting on one of
    /// the two keys nearest its first point and ending on one of the two nearest its last, whose
    /// path is close enough to the swipe's length.
    pub fn candidates(&self, swipe: &Gesture, layout: &Layout) -> Vec<u32> {
        let (Some(radius), Some(last)) = (layout.radius(), swipe.len().checked_sub(1)) else {
            return Vec::new();
        };
        let firsts = layout.closest_two(swipe.x(0), swipe.y(0));
        let lasts = layout.closest_two(swipe.x(last), swipe.y(last));
        let swipe_length = swipe.length();
        let limit = LENGTH_THRESHOLD * radius as f64;
        let mut path = Box::new(Gesture::new());
        let mut candidates = Vec::new();
        for &first in &firsts {
            for &last in &lasts {
                for &word in self.bucket(first, last) {
                    self.plain_path(word, layout, &mut path);
                    if (((swipe_length - path.length()).abs()) as f64) < limit {
                        candidates.push(word);
                    }
                }
            }
        }
        candidates
    }

    /// Scores `candidates` (from [`GlideIndex::candidates`]) against `swipe`, each at its
    /// frequency in `frequencies` (0 to 1; 0 for a word the dictionary lacks), as FlorisBoard's
    /// classifier did ([`Tuning::ORIGINAL`]). Returns the `max` likeliest.
    pub fn score(
        &self,
        candidates: &[u32],
        frequencies: &[f32],
        swipe: &Gesture,
        layout: &Layout,
        max: usize,
    ) -> Result<Ranking, ScoreError> {
        let swipe = Swipe {
            gesture: swipe,
            times: &[],
        };
        self.score_with(
            candidates,
            frequencies,
            swipe,
            layout,
            max,
            &Tuning::ORIGINAL,
        )
    }

    /// As [`GlideIndex::score`], with `tuning`'s additions.
    pub fn score_with(
        &self,
        candidates: &[u32],
        frequencies: &[f32],
        swipe: Swipe<'_>,
        layout: &Layout,
        max: usize,
        tuning: &Tuning,
    ) -> Result<Ranking, ScoreError> {
        let Swipe {
            gesture: swipe,
            times,
        } = swipe;
        if candidates.len() != frequencies.len() {
            return Err(ScoreError::FrequencyCount);
        }
        if let Some(&word) = candidates
            .iter()
            .find(|&&word| word as usize >= self.word_count())
        {
            return Err(ScoreError::NoSuchWord(word));
        }
        let mut ranking = Ranking::new(max);
        let Some(radius) = layout.radius() else {
            return Ok(ranking);
        };
        if swipe.is_empty() {
            return Ok(ranking);
        }
        let mut g = Box::new(Scratch::default());
        swipe.resample_into(&mut g.swipe);
        g.swipe.normalize_into(&mut g.swipe_shape);
        let location_std = LOCATION_STD * radius;
        let original = tuning.is_original();
        let weights = tuning::dwell_weights(tuning.dwell, swipe, times, radius);
        for (&word, &frequency) in candidates.iter().zip(frequencies) {
            let Scratch {
                swipe: resampled,
                swipe_shape,
                plain,
                loops,
                path,
                shape,
            } = &mut *g;
            let has_loops = self.paths(word, layout, plain, loops);
            if plain.is_empty() {
                continue;
            }
            let frequency = 255.0f32 * frequency;
            let ends = if original {
                1.0
            } else {
                let last = plain.len() - 1;
                let (first, last) = ((plain.x(0), plain.y(0)), (plain.x(last), plain.y(last)));
                tuning::ends_probability(tuning.ends, swipe, first, last, radius)
            };
            let ideals = [&*plain, &*loops];
            for ideal in &ideals[..if has_loops { 2 } else { 1 }] {
                ideal.resample_into(path);
                path.normalize_into(shape);
                let shape_probability = gaussian(shape_distance(shape, swipe_shape), SHAPE_STD);
                let weight = if original {
                    let location = location_distance(path, resampled);
                    1.0 / (shape_probability * gaussian(location, location_std) * frequency)
                } else {
                    let location = tuning::weighted_location_distance(path, resampled, &weights);
                    let location_probability = gaussian(location, location_std);
                    let weight =
                        1.0 / (shape_probability * location_probability * frequency * ends);
                    // Reach only lowers a word's chances, so a word already out stays out.
                    if tuning.reach == 0.0 || ranking.would_drop(weight) {
                        weight
                    } else {
                        weight / tuning::reach_probability(tuning.reach, swipe, ideal, radius)
                    }
                };
                ranking.insert(self.spelling(word), weight);
            }
        }
        Ok(ranking)
    }
}

/// The gestures one scoring reuses for every word.
#[derive(Default)]
struct Scratch {
    swipe: Gesture,
    swipe_shape: Gesture,
    plain: Gesture,
    loops: Gesture,
    path: Gesture,
    shape: Gesture,
}

fn shape_distance(a: &Gesture, b: &Gesture) -> f32 {
    let mut total = 0.0f32;
    for i in 0..SAMPLES {
        total += distance(a.x(i), a.y(i), b.x(i), b.y(i));
    }
    total
}

fn location_distance(a: &Gesture, b: &Gesture) -> f32 {
    let mut total = 0.0f32;
    for i in 0..SAMPLES {
        total += (a.x(i) - b.x(i)).abs() + (a.y(i) - b.y(i)).abs();
    }
    total / SAMPLES as f32 / 2.0
}

/// The density at `value` of a normal distribution centred on 0, worked in doubles as Kotlin did.
fn gaussian(value: f32, standard_deviation: f32) -> f32 {
    let factor = 1.0 / (standard_deviation as f64 * (2.0 * std::f64::consts::PI).sqrt());
    let z = (value / standard_deviation) as f64;
    (factor * (-1.0 / 2.0 * (z * z)).exp()) as f32
}

#[cfg(test)]
#[path = "score_tuned_tests.rs"]
mod tuned_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::tests::{ascii_table, index};
    use crate::layout::Key;

    fn ranking(entries: &[(u32, f32)], max: usize) -> Ranking {
        let mut ranking = Ranking::new(max);
        for &(word, weight) in entries {
            ranking.insert(word, weight);
        }
        ranking
    }

    // Words as numbers: 1 pill, 2 pull, 3 pal, 4 poll, 5 pool; 10, 11, 12, 13 a, b, c, d.
    #[test]
    fn keeps_the_likeliest_first_up_to_the_limit() {
        let ranked = ranking(&[(4, 4.0), (1, 2.0), (2, 6.0), (3, 3.0)], 3);

        assert_eq!(ranked.words(), &[1, 3, 4]);
        assert_eq!(ranked.weights(), &[2.0, 3.0, 4.0]);
    }

    #[test]
    fn a_less_likely_copy_is_ignored() {
        let ranked = ranking(&[(5, 3.0), (4, 4.0), (5, 5.0)], 3);

        assert_eq!(ranked.words(), &[5, 4]);
        assert_eq!(ranked.weights(), &[3.0, 4.0]);
    }

    #[test]
    fn a_more_likely_copy_replaces_the_first() {
        let ranked = ranking(&[(5, 5.0), (4, 4.0), (5, 3.0)], 3);

        assert_eq!(ranked.words(), &[5, 4]);
        assert_eq!(ranked.weights(), &[3.0, 4.0]);
    }

    #[test]
    fn a_more_likely_copy_frees_its_slot_for_the_next_word() {
        let mut ranked = ranking(&[(10, 1.0), (11, 2.0), (5, 5.0), (5, 1.5), (12, 3.0)], 3);
        assert_eq!(ranked.words(), &[10, 5, 11]);

        ranked.max = 4;
        ranked.insert(13, 2.5);
        assert_eq!(ranked.words(), &[10, 5, 11, 13]);
    }

    #[test]
    fn nothing_fits_in_no_room() {
        assert!(ranking(&[(1, 1.0)], 0).words().is_empty());
    }

    /// A QWERTY letter layout with 100 x 150 keys, each row shifted half a key.
    pub(super) fn qwerty() -> Vec<Key> {
        ["qwertyuiop", "asdfghjkl", "zxcvbnm"]
            .iter()
            .enumerate()
            .flat_map(|(row, letters)| {
                letters.chars().enumerate().map(move |(i, c)| Key {
                    code: c as i32,
                    center_x: 50.0 + 100.0 * i as f32 + 50.0 * row as f32,
                    center_y: 75.0 + 150.0 * row as f32,
                    width: 100.0,
                    height: 150.0,
                })
            })
            .collect()
    }

    /// A swipe through `word`'s keys, with points every 20 px along the way.
    pub(super) fn swipe_through(word: &str, keys: &[Key]) -> Gesture {
        let centre = |c: char| {
            keys.iter()
                .find(|k| k.code == c as i32)
                .map(|k| (k.center_x, k.center_y))
                .unwrap()
        };
        let mut swipe = Gesture::new();
        let mut at = centre(word.chars().next().unwrap());
        swipe.push(at.0, at.1);
        for c in word.chars().skip(1) {
            let to = centre(c);
            let steps = (distance(at.0, at.1, to.0, to.1) / 20.0).ceil().max(1.0) as usize;
            for s in 1..=steps {
                let t = s as f32 / steps as f32;
                swipe.push(at.0 + (to.0 - at.0) * t, at.1 + (to.1 - at.1) * t);
            }
            at = to;
        }
        swipe
    }

    fn guesses(words: &[&str], swiped: &str, max: usize) -> Vec<String> {
        let keys = qwerty();
        let index = index(words, &ascii_table(&[]));
        let layout = Layout::new(&keys, index.codes());
        let swipe = swipe_through(swiped, &keys);
        let candidates = index.candidates(&swipe, &layout);
        let frequencies = vec![1.0; candidates.len()];
        let ranking = index
            .score(&candidates, &frequencies, &swipe, &layout, max)
            .unwrap();
        ranking
            .words()
            .iter()
            .map(|&w| words[w as usize].to_string())
            .collect()
    }

    #[test]
    fn a_clean_swipe_finds_its_word_first() {
        let words = [
            "hello", "help", "hell", "jello", "world", "word", "would", "hold",
        ];

        assert_eq!(guesses(&words, "hello", 3)[0], "hello");
        assert_eq!(guesses(&words, "world", 3)[0], "world");
        assert_eq!(guesses(&words, "help", 3)[0], "help");
    }

    #[test]
    fn a_loop_tells_a_repeated_letter_apart() {
        let words = ["pol", "pool", "poll", "pl"];

        // The swipe has no loop, so the loopless "pol" wins, while "pool" and "poll" still rank.
        let found = guesses(&words, "pol", 4);
        assert_eq!(found[0], "pol");
        assert!(found.contains(&"pool".to_string()));
    }

    #[test]
    fn a_word_on_other_end_keys_is_not_tried() {
        let keys = qwerty();
        let index = index(&["hello", "quiz"], &ascii_table(&[]));
        let layout = Layout::new(&keys, index.codes());

        assert_eq!(
            index.candidates(&swipe_through("hello", &keys), &layout),
            vec![0]
        );
    }

    #[test]
    fn a_much_longer_word_is_not_tried() {
        let keys = qwerty();
        let index = index(&["ha", "hjklhjklhjkla"], &ascii_table(&[]));
        let layout = Layout::new(&keys, index.codes());

        assert_eq!(
            index.candidates(&swipe_through("ha", &keys), &layout),
            vec![0]
        );
    }

    #[test]
    fn a_spelling_is_ranked_once() {
        assert_eq!(guesses(&["to", "too", "to"], "to", 3), vec!["to", "too"]);
    }

    #[test]
    fn nothing_is_scored_without_keys_or_points() {
        let index = index(&["a"], &ascii_table(&[]));
        let keys = qwerty();
        let layout = Layout::new(&keys, index.codes());
        let no_keys = Layout::new(&[], index.codes());
        let a = swipe_through("a", &keys);

        assert!(index.candidates(&Gesture::new(), &layout).is_empty());
        assert!(index.candidates(&a, &no_keys).is_empty());
        assert!(
            index
                .score(&[0], &[1.0], &Gesture::new(), &layout, 3)
                .unwrap()
                .words()
                .is_empty()
        );
        assert!(
            index
                .score(&[0], &[1.0], &a, &no_keys, 3)
                .unwrap()
                .words()
                .is_empty()
        );
    }

    #[test]
    fn bad_candidates_are_refused() {
        let index = index(&["a"], &ascii_table(&[]));
        let keys = qwerty();
        let layout = Layout::new(&keys, index.codes());
        let a = swipe_through("a", &keys);

        assert_eq!(
            index.score(&[0], &[], &a, &layout, 3),
            Err(ScoreError::FrequencyCount)
        );
        assert_eq!(
            index.score(&[1], &[1.0], &a, &layout, 3),
            Err(ScoreError::NoSuchWord(1))
        );
    }

    #[test]
    fn the_gaussian_peaks_at_zero_and_is_symmetric() {
        let peak = 1.0 / (22.08f64 * (2.0 * std::f64::consts::PI).sqrt());

        assert_eq!(gaussian(0.0, 22.08), peak as f32);
        assert_eq!(gaussian(-5.0, 22.08), gaussian(5.0, 22.08));
        assert!(gaussian(30.0, 22.08) < gaussian(5.0, 22.08));
    }
}
