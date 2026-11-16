//! The second swipe simulator ("lognormal"): the finger as a published model of human movement
//! moves it, so that nothing tuned to the first simulator ([`super::trace`]) passes for free.
//!
//! **The model.** The kinematic theory of rapid human movements in its sigma-lognormal form
//! (R. Plamondon, "A kinematic theory of rapid human movements, Part I", Biological Cybernetics
//! 72, 295–307, 1995; R. Plamondon & M. Djioua, "A multi-level representation paradigm for
//! handwriting stroke generation", Human Movement Science 25, 586–607, 2006), used as
//! L. A. Leiva, D. Martín-Albo & R. Plamondon use it to synthesise human-like stroke gestures
//! ("Gestures à Go Go", ACM TIST 7(2), 2015, doi:10.1145/2799648). A movement is a sum of
//! strokes, one towards each "virtual target". Stroke j's speed is a lognormal of time,
//!
//! ```text
//! |v_j(t)| = D_j / (σ_j √(2π) (t − t0_j)) · exp(−(ln(t − t0_j) − μ_j)² / (2σ_j²)),
//! ```
//!
//! its direction turns steadily from θs_j to θe_j as it goes, and the finger's velocity is the
//! sum of the strokes'. Each starts before the last has ended, so the finger slows at every key
//! but doesn't stop, and turns before it gets there: corners are cut by the overlap, not by a
//! rule, and it slows at a letter it passes straight through as well as at a turn.
//!
//! **Parameters.** One stroke per pair of the word's keys in turn (a doubled letter once):
//!  - each virtual target is the key's centre missed by a Gaussian of 1.2 mm (28 px) both ways;
//!  - the lognormal's median time e^μ is 100 ms + 50 ms · log₂(1 + C/W) (Fitts' law; C the
//!    stroke's chord, W a key's width), times e^N(0, 0.1); σ is uniform in 0.20–0.35, within the
//!    range those papers fit to fast handwriting and gestures;
//!  - each stroke starts 0.8–1.1 of the last one's median after it;
//!  - each stroke bows: θe − θs is drawn from N(0, 0.15 rad).
//!
//! **Left thumb.** Aims fall short of far keys and scatter more there, as taps do
//! ([`crate::touch::aim`]), and each stroke bows as the thumb turns about its base at the
//! keyboard's bottom left: its θe − θs gains half the angle it sweeps about that pivot.
//!
//! Sampled at 240 Hz, the S24 Ultra's touch rate, with 1 px of sensor noise, until every stroke
//! is 99.9% done.

use super::{Board, Sample};
use crate::rng::Rng;
use crate::touch::Hand;
use std::f64::consts::{PI, SQRT_2};

/// How far a virtual target misses a key's centre, in pixels (1.2 mm at 3.75 px/dp).
const AIM_SPREAD: f64 = 28.0;
/// Fitts' law for a stroke's median time, in milliseconds.
const FITTS_A: f64 = 100.0;
const FITTS_B: f64 = 50.0;
const MEDIAN_SPREAD: f64 = 0.1;
const SIGMA: (f64, f64) = (0.20, 0.35);
/// When the next stroke starts, as a share of the last one's median time.
const ONSET: (f64, f64) = (0.8, 1.1);
const BOW_SPREAD: f64 = 0.15;
/// How much of the angle the thumb sweeps about its pivot a stroke turns through.
const THUMB_BOW: f64 = 0.5;
const SAMPLE_MS: f64 = 1000.0 / 240.0;
const NOISE: f64 = 1.0;
const DONE: f64 = 0.999;

/// One lognormal stroke.
#[derive(Clone, Copy, Debug)]
struct Stroke {
    t0: f64,
    mu: f64,
    sigma: f64,
    /// Its length along its arc, its starting direction and how far it turns.
    length: f64,
    theta: f64,
    turn: f64,
}

impl Stroke {
    /// How much of the stroke is done by time `t`: the lognormal's distribution function.
    fn done(&self, t: f64) -> f64 {
        if t <= self.t0 {
            return 0.0;
        }
        0.5 * (1.0 + erf(((t - self.t0).ln() - self.mu) / (self.sigma * SQRT_2)))
    }

    /// How far the stroke has moved the finger by time `t`, along its arc.
    fn moved(&self, t: f64) -> (f64, f64) {
        let done = self.done(t);
        if self.turn.abs() < 1e-9 {
            let s = self.length * done;
            return (s * self.theta.cos(), s * self.theta.sin());
        }
        let r = self.length / self.turn;
        let phi = self.theta + self.turn * done;
        (
            r * (phi.sin() - self.theta.sin()),
            r * (self.theta.cos() - phi.cos()),
        )
    }
}

/// The finger's samples for a swipe along `path` (key indexes) on `board`.
pub fn draw(board: &Board, path: &[usize], hand: Hand, rng: &mut Rng) -> Vec<Sample> {
    let targets: Vec<(f64, f64)> = path
        .iter()
        .map(|&k| {
            let key = &board.keys[k];
            let aim = board.aim(hand, k);
            let spread = AIM_SPREAD * f64::from(aim.spread);
            (
                f64::from(key.center_x + aim.bias_x) + rng.gaussian() * spread,
                f64::from(key.center_y + aim.bias_y) + rng.gaussian() * spread,
            )
        })
        .collect();
    let width = f64::from(board.keys[0].width);
    let pivot = (f64::from(board.pivot.0), f64::from(board.pivot.1));
    let mut strokes = Vec::with_capacity(targets.len());
    let mut t0 = 0.0;
    for pair in targets.windows(2) {
        let (from, to) = (pair[0], pair[1]);
        let chord = (to.0 - from.0).hypot(to.1 - from.1);
        let median = (FITTS_A + FITTS_B * (1.0 + chord / width).log2())
            * (rng.gaussian() * MEDIAN_SPREAD).exp();
        let sigma = SIGMA.0 + (SIGMA.1 - SIGMA.0) * rng.uniform();
        let mut turn = rng.gaussian() * BOW_SPREAD;
        if hand == Hand::LeftThumb {
            turn += THUMB_BOW * swept(pivot, from, to);
        }
        // An arc with this chord, turning through `turn`.
        let length = if turn.abs() < 1e-9 {
            chord
        } else {
            chord * (turn / 2.0) / (turn / 2.0).sin()
        };
        strokes.push(Stroke {
            t0,
            mu: median.ln(),
            sigma,
            length,
            theta: (to.1 - from.1).atan2(to.0 - from.0) - turn / 2.0,
            turn,
        });
        t0 += median * (ONSET.0 + (ONSET.1 - ONSET.0) * rng.uniform());
    }
    let start = targets[0];
    let mut samples = Vec::new();
    let mut t = 0.0;
    loop {
        let (mut x, mut y) = start;
        let mut finished = true;
        for stroke in &strokes {
            let (dx, dy) = stroke.moved(t);
            x += dx;
            y += dy;
            finished &= stroke.done(t) >= DONE;
        }
        samples.push(Sample {
            x: (x + rng.gaussian() * NOISE) as f32,
            y: (y + rng.gaussian() * NOISE) as f32,
            t: t as f32,
        });
        if finished {
            return samples;
        }
        t += SAMPLE_MS;
    }
}

/// The angle from `from` to `to` about `pivot`, between -π and π.
fn swept(pivot: (f64, f64), from: (f64, f64), to: (f64, f64)) -> f64 {
    let a = (from.1 - pivot.1).atan2(from.0 - pivot.0);
    let b = (to.1 - pivot.1).atan2(to.0 - pivot.0);
    let mut d = b - a;
    while d > PI {
        d -= 2.0 * PI;
    }
    while d < -PI {
        d += 2.0 * PI;
    }
    d
}

/// The error function, to within 1.5e-7 (Abramowitz & Stegun 7.1.26).
fn erf(x: f64) -> f64 {
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    sign * (1.0 - poly * (-x * x).exp())
}

#[cfg(test)]
#[path = "lognormal_tests.rs"]
mod tests;
