//! The first swipe simulator ("trace"): a path drawn through the word's keys by rule.
//!
//! A swipe aims at each key of the word in turn, missing its visible centre by a Gaussian of a
//! fifth of the key across and down. Corners are cut: each turn is pulled towards the midpoint of
//! its neighbours by up to a third of a key (uniformly drawn, and never past the midpoint). The
//! finger's path between them is drawn as a Catmull-Rom curve sampled every 4 px, with a little
//! tremor.
//!
//! **Timing.** The path's geometry has no speed of its own, so its clock is the two-thirds power
//! law of drawing movements (Lacquaniti, Terzuolo & Viviani 1983, "The law relating the kinematic
//! and figural aspects of drawing movements", Acta Psychologica 54, 115–130): speed falls as the
//! cube root of curvature, `v = K κ^(-1/3)`. The law has no limit on a straight run, so speed is
//! capped there: `v = V (1 + κ / κ₀)^(-1/3)`, at most [`SPEED`], with κ₀ a quarter of a key's
//! curvature. The finger slows at turns but not at a letter it passes straight through; the
//! second simulator ([`super::lognormal`]) slows at every letter, so a change that needs either is
//! checked against both.
//!
//! **Left thumb.** Each aim falls short of far keys and scatters more there, as the taps do
//! ([`crate::touch::aim`]); the rest is as above.

use super::{Board, Sample};
use crate::rng::Rng;
use crate::touch::Hand;

/// How far an aim misses a key's centre, in visible key widths and heights.
const AIM_SPREAD: f32 = 0.2;
/// How far turns are pulled in towards their neighbours' midpoint at most, in key widths.
const CUT_MAX: f32 = 0.33;
/// The spacing of the drawn path, and the tremor along it, in pixels.
const STEP: f32 = 4.0;
const TREMOR: f32 = 1.5;
/// The speed on a straight run, in pixels a millisecond (about fourteen keys a second).
const SPEED: f32 = 2.0;

/// The finger's samples for a swipe along `path` (key indexes) on `board`.
pub fn draw(board: &Board, path: &[usize], hand: Hand, rng: &mut Rng) -> Vec<Sample> {
    let keys = &board.keys;
    let mut aims: Vec<(f32, f32)> = path
        .iter()
        .map(|&k| {
            let key = &keys[k];
            let aim = board.aim(hand, k);
            let spread = AIM_SPREAD * aim.spread;
            (
                key.center_x + aim.bias_x + rng.gaussian() as f32 * spread * key.width,
                key.center_y + aim.bias_y + rng.gaussian() as f32 * spread * key.height,
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
            let width = keys[path[i]].width;
            let cut = (rng.uniform() as f32 * CUT_MAX * width).min(distance) / distance;
            aims[i].0 += dx * cut;
            aims[i].1 += dy * cut;
        }
    }
    let smooth = curve(&aims);
    let times = clock(&smooth, keys[0].width);
    smooth
        .into_iter()
        .zip(times)
        .map(|((x, y), t)| Sample {
            x: x + rng.gaussian() as f32 * TREMOR,
            y: y + rng.gaussian() as f32 * TREMOR,
            t,
        })
        .collect()
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

/// When the finger reaches each of `points`, in milliseconds, moving at the capped power law's
/// speed for keys `width` wide.
fn clock(points: &[(f32, f32)], width: f32) -> Vec<f32> {
    let kappa_0 = 1.0 / (4.0 * width);
    let speed = |i: usize| {
        if i == 0 || i + 1 >= points.len() {
            return SPEED;
        }
        let kappa = curvature(points[i - 1], points[i], points[i + 1]);
        SPEED * (1.0 + kappa / kappa_0).powf(-1.0 / 3.0)
    };
    let mut times = Vec::with_capacity(points.len());
    let mut t = 0.0f32;
    times.push(t);
    for i in 1..points.len() {
        let (a, b) = (points[i - 1], points[i]);
        let step = (b.0 - a.0).hypot(b.1 - a.1);
        t += step / ((speed(i - 1) + speed(i)) / 2.0);
        times.push(t);
    }
    times
}

/// The curvature of the circle through three points (Menger's): 0 for points in a line.
fn curvature(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    let (ab, bc, ca) = (
        (b.0 - a.0).hypot(b.1 - a.1),
        (c.0 - b.0).hypot(c.1 - b.1),
        (a.0 - c.0).hypot(a.1 - c.1),
    );
    let twice_area = ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)).abs();
    let product = ab * bc * ca;
    if product > 0.0 {
        2.0 * twice_area / product
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curvature_is_the_inverse_radius() {
        let r = 50.0f32;
        let on = |deg: f32| (r * deg.to_radians().cos(), r * deg.to_radians().sin());
        assert!((curvature(on(0.0), on(30.0), on(70.0)) - 1.0 / r).abs() < 1e-4);
        assert_eq!(curvature((0.0, 0.0), (5.0, 0.0), (9.0, 0.0)), 0.0);
        assert_eq!(curvature((1.0, 1.0), (1.0, 1.0), (4.0, 0.0)), 0.0);
    }

    #[test]
    fn the_clock_runs_at_full_speed_on_straights_and_slower_round_turns() {
        let straight: Vec<(f32, f32)> = (0..50).map(|i| (i as f32 * 4.0, 0.0)).collect();
        let times = clock(&straight, 140.0);
        assert!((times[49] - 98.0).abs() < 1e-3, "{}", times[49]);
        // A quarter turn of radius 20 px: its 31 px take far longer than 31 ms.
        let turn: Vec<(f32, f32)> = (0..=10)
            .map(|i| {
                let a = std::f32::consts::FRAC_PI_2 * i as f32 / 10.0;
                (20.0 * a.cos(), 20.0 * a.sin())
            })
            .collect();
        let times = clock(&turn, 140.0);
        let length: f32 = turn
            .windows(2)
            .map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1))
            .sum();
        assert!(times[10] > 2.0 * length / SPEED, "{} {length}", times[10]);
        assert!(times.windows(2).all(|w| w[1] > w[0]));
    }
}
