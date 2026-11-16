//! The tap-aim learning run: a simulated user types one long stream of held-out sentences while
//! the keyboard learns their aim from the words they keep, as the phone does
//! (`suggest::tap_learning`), and the later sentences are scored.
//!
//! Each user is typed three ways: with learning off, with the learned aim moving what the decoder
//! hears (`TapAim::aimed`, "(a)"), and with it also choosing the key on an ambiguous tap
//! (`TapAim::retarget`, "(a)+(b)"). The taps are drawn the same way for all three, word by word.
//!
//! The simulated user commits what auto-correction gives (or what they typed), then fixes a wrong
//! word with a backspace [`FIXES`] of the time and leaves the rest: a typo left as typed is then
//! "kept" and teaches the aim wrongly, as on the phone. Space, a fix's retyping and the strip are
//! not simulated beyond that.

use crate::geometry::Keyboard;
use crate::report::Corrections;
use crate::rng::Rng;
use crate::text::Sentence;
use crate::touch::{self, Hand, KeyResize, Typed};
use crate::typing::{Corrected, Engine, WithModel};
use std::ops::Range;
use std::sync::Arc;
use suggest::java;
use suggest::previous_words::PreviousWords;
use suggest::qwerty;
use suggest::tap::{TapOffset, TapTrail};
use suggest::tap_aim::AimKey;
use suggest::tap_learning::Taps;

/// How often the user fixes a wrong word.
pub const FIXES: f64 = 0.7;
/// The most a user's own aim at a key is off its centre, across and down, in key pitches.
const OWN_AIM_X: f32 = 0.12;
const OWN_AIM_Y: f32 = 0.1;
/// Sentences scored separately right after a user's grip changes.
const AFTER_CHANGE: usize = 500;

/// How the learned aim is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Off,
    Decoder,
    DecoderAndKeys,
}

impl Mode {
    pub const ALL: [Mode; 3] = [Mode::Off, Mode::Decoder, Mode::DecoderAndKeys];

    pub fn name(self) -> &'static str {
        match self {
            Mode::Off => "off",
            Mode::Decoder => "(a)",
            Mode::DecoderAndKeys => "(a)+(b)",
        }
    }
}

/// What an auto-corrected word the user kept teaches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FromCorrected {
    /// Nothing: only words committed as typed teach (what ships).
    Nothing,
    /// Its letters that the correction left alone.
    Unchanged,
    /// Those, and each changed letter's tap moved onto the key the correction says was meant,
    /// when that key is next to the one tapped.
    Rebased,
}

/// One way of tapping: a hand, and a user's own aim at each key drawn from a seed.
#[derive(Clone, Copy, Debug)]
struct Grip {
    hand: Hand,
    own_aim: Option<u64>,
}

/// A simulated user: a grip, and another grip from halfway through if it changes.
#[derive(Clone, Copy, Debug)]
pub struct User {
    pub name: &'static str,
    first: Grip,
    second: Option<Grip>,
}

pub fn users() -> [User; 5] {
    let grip = |hand, own_aim| Grip { hand, own_aim };
    [
        User {
            name: "neutral",
            first: grip(Hand::Neutral, None),
            second: None,
        },
        User {
            name: "left thumb",
            first: grip(Hand::LeftThumb, None),
            second: None,
        },
        User {
            name: "left thumb, own aim per key",
            first: grip(Hand::LeftThumb, Some(1)),
            second: None,
        },
        User {
            name: "changes grip halfway",
            first: grip(Hand::LeftThumb, Some(1)),
            second: Some(grip(Hand::RightThumb, Some(2))),
        },
        // The grip changed to, from the start: what the change's later half should come to.
        User {
            name: "right thumb, other aim per key",
            first: grip(Hand::RightThumb, Some(2)),
            second: None,
        },
    ]
}

impl User {
    /// The stretches of a stream of `count` sentences scored apart, after `warm_up`.
    pub fn windows(&self, count: usize, warm_up: usize) -> Vec<(&'static str, Range<usize>)> {
        if self.second.is_none() {
            return vec![("after warm-up", warm_up..count)];
        }
        let half = count / 2;
        let settled = (half + AFTER_CHANGE).min(count);
        vec![
            ("before the change", warm_up.min(half)..half),
            ("first 500 after", half..settled),
            ("rest", settled..count),
        ]
    }
}

/// Each key's own aim for `grip`, by its index in [`Keyboard::keys`].
fn own_aims(keyboard: &Keyboard, grip: Grip) -> Vec<(f32, f32)> {
    let Some(seed) = grip.own_aim else {
        return vec![(0.0, 0.0); keyboard.keys.len()];
    };
    let mut rng = Rng::new(seed);
    keyboard
        .keys
        .iter()
        .map(|_| {
            let x = (rng.uniform() as f32 * 2.0 - 1.0) * OWN_AIM_X;
            let y = (rng.uniform() as f32 * 2.0 - 1.0) * OWN_AIM_Y;
            (x, y)
        })
        .collect()
}

/// The sentences of every source in turn, one from each, so the stream mixes them evenly.
pub fn stream(sources: &[Vec<Sentence>]) -> Vec<&Sentence> {
    let longest = sources.iter().map(Vec::len).max().unwrap_or(0);
    (0..longest)
        .flat_map(|i| sources.iter().filter_map(move |source| source.get(i)))
        .collect()
}

/// One user typed one way, with what it was scored on.
#[derive(Debug)]
pub struct Run<'a> {
    pub keyboard: &'a Keyboard,
    pub stream: &'a [&'a Sentence],
    pub user: User,
    pub mode: Mode,
    pub from_corrected: FromCorrected,
    pub seed: u64,
    pub windows: Vec<Range<usize>>,
    /// Whether the hit test leans on the letter odds too.
    pub resize: Option<KeyResize>,
}

/// What the keyboard knows at a tap: the learned aim's keys and what was typed before it.
struct AtTap<'a> {
    aim_keys: &'a [AimKey],
    previous: &'a PreviousWords,
    typed: &'a [u16],
}

impl Run<'_> {
    /// Types the whole stream, learning as it goes; auto-correction's results in each window.
    pub fn score(&self, dictionary: Arc<[u8]>, model: Option<&WithModel>) -> Vec<Corrections> {
        let mut engine = Engine::open(dictionary, model).expect("the dictionary opens");
        let grips = [self.user.first, self.user.second.unwrap_or(self.user.first)];
        let aims = grips.map(|grip| own_aims(self.keyboard, grip));
        let aim_keys = aim_keys(self.keyboard);
        let mut taps = Taps::new(TapTrail::CAPACITY);
        let mut results = vec![Corrections::default(); self.windows.len()];
        let half = self.stream.len() / 2;
        for (i, sentence) in self.stream.iter().enumerate() {
            let phase = usize::from(self.user.second.is_some() && i >= half);
            let (hand, aims) = (grips[phase].hand, &aims[phase]);
            for w in 0..sentence.words.len() {
                let mut rng = Rng::for_item(self.seed, i as u64, w as u64);
                let word = sentence.word(w);
                let previous = PreviousWords::from(sentence.before(w));
                let mut typed: Vec<u16> = Vec::with_capacity(word.len());
                for &unit in word {
                    let at = AtTap {
                        aim_keys: &aim_keys,
                        previous: &previous,
                        typed: &typed,
                    };
                    let tap = self.tap(unit, (hand, aims), at, (&mut taps, &engine), &mut rng);
                    typed.push(tap.unit);
                }
                let offsets = taps.offsets_for(&typed, self.mode != Mode::Off);
                let strip = engine.strip(sentence.before(w), &typed, &offsets, 1);
                let committed = strip
                    .into_iter()
                    .find(|candidate| candidate.is_auto_correction)
                    .map_or_else(|| typed.clone(), |candidate| candidate.text);
                let corrected = Corrected {
                    // This run compares ways of using the aim, not against the plain hit test.
                    plain: typed == word,
                    plain_right: chars::equals_ignore_case(&committed, word),
                    clean: typed == word,
                    changed: committed != typed,
                    right: chars::equals_ignore_case(&committed, word),
                };
                for (window, result) in self.windows.iter().zip(&mut results) {
                    if window.contains(&i) {
                        result.add(corrected);
                    }
                }
                if self.mode != Mode::Off {
                    self.learn(&mut taps, (&typed, &committed), corrected, &mut rng);
                }
            }
        }
        results
    }

    /// Types `unit` as this user, through the keyboard's hit test.
    fn tap(
        &self,
        unit: u16,
        (hand, aims): (Hand, &[(f32, f32)]),
        at: AtTap,
        (taps, engine): (&mut Taps, &Engine),
        rng: &mut Rng,
    ) -> Typed {
        let keyboard = self.keyboard;
        let target = keyboard.index_for(i32::from(chars::to_lower(unit)));
        let aim = target.map_or((0.0, 0.0), |index| aims[index]);
        let Some((hit, x, y)) = touch::land(keyboard, unit, hand, aim, rng) else {
            taps.typed(&[unit]);
            return Typed { unit, offset: None };
        };
        let orientation = taps.orientation();
        let shift = |index: usize| {
            let key = at
                .aim_keys
                .get(index)
                .filter(|_| self.mode == Mode::DecoderAndKeys)?;
            let bias = taps.aim.bias(orientation, key.code)?;
            Some((bias.dx * key.width, bias.dy * key.height))
        };
        let resize = self.resize.as_ref();
        let odds = resize
            .and(engine.session().language_model())
            .and_then(|model| model.letter_odds(at.previous, at.typed));
        let hit = touch::key_for(keyboard, hit, (x, y), shift, odds.as_ref(), resize);
        let key = &keyboard.keys[hit];
        let typed = touch::typed(key, unit, x, y);
        taps.trail.pressed(key.code, typed.offset);
        taps.typed(&[typed.unit]);
        typed
    }

    /// What the keyboard learns from the word just committed, and the user's fix if it was wrong.
    fn learn(
        &self,
        taps: &mut Taps,
        (typed, committed): (&[u16], &[u16]),
        corrected: Corrected,
        rng: &mut Rng,
    ) {
        let fixed = !corrected.right && rng.uniform() < FIXES;
        if corrected.changed {
            taps.picked(true);
            if !fixed && self.from_corrected != FromCorrected::Nothing {
                let rebase = self.from_corrected == FromCorrected::Rebased;
                learn_corrected(taps, typed, committed, rebase);
            }
        } else {
            taps.kept(typed);
        }
        if fixed {
            taps.backspace();
        }
    }
}

fn aim_keys(keyboard: &Keyboard) -> Vec<AimKey> {
    keyboard
        .keys
        .iter()
        .map(|key| AimKey {
            code: key.code,
            width: key.visible.width(),
            height: key.visible.height(),
        })
        .collect()
}

/// Learns the taps of `typed`, which auto-correction made `committed`: the letters it kept, and
/// if `rebase`, each letter it changed to a neighbouring key, as a tap on that key.
fn learn_corrected(taps: &mut Taps, typed: &[u16], committed: &[u16], rebase: bool) {
    let Some(trail) = taps.trail.typed_for(typed) else {
        return;
    };
    let meant = java::code_points(committed);
    if meant.len() != trail.len() {
        return;
    }
    let samples: Vec<(i32, TapOffset)> = trail
        .into_iter()
        .zip(meant)
        .filter_map(|((tapped, offset), meant)| {
            let offset = offset?;
            if java::to_lower(tapped) == java::to_lower(meant) {
                return Some((meant, offset));
            }
            let (tx, ty) = qwerty::key_center(tapped).filter(|_| rebase)?;
            let (mx, my) = qwerty::key_center(meant)?;
            let dx = offset.dx + (tx - mx) as f32 / qwerty::KEY_WIDTH as f32;
            let dy = offset.dy + (ty - my) as f32 / qwerty::KEY_HEIGHT as f32;
            (dx.abs() <= 1.0 && dy.abs() <= 1.0).then_some((meant, TapOffset { dx, dy }))
        })
        .collect();
    let orientation = taps.orientation();
    taps.aim.learn(orientation, &samples);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    #[test]
    fn mixes_the_sources_evenly() {
        let a = vec![Sentence::new("one."), Sentence::new("two.")];
        let b = vec![Sentence::new("three.")];
        let sources = [a, b];
        let stream = stream(&sources);
        let first: Vec<String> = stream
            .iter()
            .map(|s| String::from_utf16_lossy(s.word(0)))
            .collect();
        assert_eq!(first, ["one", "three", "two"]);
    }

    #[test]
    fn a_changing_user_is_scored_before_and_after_the_change() {
        let [neutral, _, _, changing, _] = users();
        assert_eq!(neutral.windows(100, 10), [("after warm-up", 10..100)]);
        let windows = changing.windows(2000, 100);
        assert_eq!(windows[0].1, 100..1000);
        assert_eq!(windows[1].1, 1000..1500);
        assert_eq!(windows[2].1, 1500..2000);
    }

    #[test]
    fn a_corrected_word_teaches_its_kept_letters_and_maybe_its_neighbours() {
        let typed = utf16("thw");
        // W tapped on its right half, towards E, or on its left half, away from it.
        for (w_dx, rebase, e) in [
            (0.4, false, None),
            (0.4, true, Some(0.4 - 1.0)),
            (-0.4, true, None),
        ] {
            let mut taps = Taps::new(TapTrail::CAPACITY);
            for (&unit, dx) in typed.iter().zip([-0.2, -0.2, w_dx]) {
                taps.trail
                    .pressed(i32::from(unit), Some(TapOffset { dx, dy: 0.0 }));
                taps.typed(&[unit]);
            }
            learn_corrected(&mut taps, &typed, &utf16("the"), rebase);
            let orientation = taps.orientation();
            let key = |c: char| taps.aim.key(orientation, c as i32).map(|aim| aim.dx);
            assert_eq!(key('t'), Some(-0.2));
            assert_eq!(key('w'), None);
            assert_eq!(key('e'), e, "{w_dx} {rebase}");
        }
    }
}
