//! What the scorer adds to FlorisBoard's classifier, and how much of it.
//!
//! [`Tuning::ORIGINAL`] is the classifier exactly as ported: the same words at the same weights,
//! bit for bit, which the keyboard's reference tests hold it to. [`Tuning::DEFAULT`] is what the
//! keyboard uses: the additions that measured better on both of the scoreboard's swipe simulators
//! (docs/swipe-accuracy.md).
//!
//! Distances are in key radii, the classifier's unit, which is a key's shorter side (see
//! [`crate::Layout`]).

use std::cmp::Ordering;

use crate::gesture::{Gesture, SAMPLES, distance};

/// How a swipe is scored beyond FlorisBoard's classifier. Each field is how strongly one addition
/// counts; 0 leaves it out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tuning {
    /// How much the swipe's first and last points must match the word's first and last keys, as
    /// gesture decoders weigh them (SHARK², Kristensson & Zhai 2004): a word is less likely by
    /// `ends` times the squared distances from them, in [`ENDS_SPREAD`]s of a key, halved.
    pub ends: f32,
    /// How much more the swipe's slow stretches count in its location distance than its fast
    /// ones: a finger slows at the letters it means. Each resampled point counts its slowness
    /// over the swipe's mean slowness, raised to `dwell`. Needs the points' times.
    pub dwell: f32,
    /// How much a word is held to the swipe's reach: the swipe straying from the word's path
    /// (a key it turns at that the word lacks) and the word's keys lying off the swipe (a key it
    /// needs that the swipe never nears) each make it less likely, by `reach` times the squared
    /// distance beyond half a key radius, in key radii, halved.
    pub reach: f32,
}

/// The spread of a swipe's ends around its word's end keys, in key radii.
pub const ENDS_SPREAD: f32 = 0.5;

/// How far either side of a point its slowness is measured, in key radii.
const DWELL_WINDOW: f32 = 0.5;

/// The most a point's dwell weight can be, either way: a long pause shouldn't count for all.
const DWELL_LIMIT: f32 = 4.0;

/// How far, in key radii, a swipe may stray from a word's path, or a word's key from the swipe,
/// before [`Tuning::reach`] counts it.
const REACH_SLACK: f32 = 0.5;

impl Tuning {
    /// FlorisBoard's classifier, unchanged.
    pub const ORIGINAL: Tuning = Tuning {
        ends: 0.0,
        dwell: 0.0,
        reach: 0.0,
    };

    /// What the keyboard uses: each addition at the strength that measured best on the
    /// scoreboard's held-out tuning sentences, on both swipe simulators.
    pub const DEFAULT: Tuning = Tuning {
        ends: 0.4,
        dwell: 1.5,
        reach: 0.5,
    };

    pub(crate) fn is_original(&self) -> bool {
        *self == Tuning::ORIGINAL
    }
}

impl Default for Tuning {
    fn default() -> Self {
        Tuning::DEFAULT
    }
}

/// How likely a word whose path starts at `first` and ends at `last` is, by its ends alone, for
/// `swipe` (as drawn), keys of `radius`.
pub(crate) fn ends_probability(
    ends: f32,
    swipe: &Gesture,
    first: (f32, f32),
    last: (f32, f32),
    radius: f32,
) -> f32 {
    if ends == 0.0 || swipe.is_empty() {
        return 1.0;
    }
    let end = swipe.len() - 1;
    let spread = ENDS_SPREAD * radius;
    let d0 = distance(swipe.x(0), swipe.y(0), first.0, first.1) / spread;
    let d1 = distance(swipe.x(end), swipe.y(end), last.0, last.1) / spread;
    (-(ends * (d0 * d0 + d1 * d1) / 2.0)).exp()
}

/// The weight of each of the [`SAMPLES`] resampled points of `swipe`, drawn at `times` (one per
/// point, in milliseconds), for `dwell`: none (all alike) without usable times.
pub(crate) fn dwell_weights(dwell: f32, swipe: &Gesture, times: &[f32], radius: f32) -> Vec<f32> {
    let mut weights = vec![1.0f32; SAMPLES];
    if dwell == 0.0 || times.len() != swipe.len() || swipe.len() < 2 {
        return weights;
    }
    // Arc length at each point, and time there; time must not run backwards.
    let mut along = Vec::with_capacity(swipe.len());
    let mut total = 0.0f32;
    along.push(0.0f32);
    for i in 1..swipe.len() {
        total += distance(swipe.x(i - 1), swipe.y(i - 1), swipe.x(i), swipe.y(i));
        along.push(total);
        let ordered = times[i].partial_cmp(&times[i - 1]);
        if !matches!(ordered, Some(Ordering::Greater | Ordering::Equal)) {
            return weights;
        }
    }
    let duration = times[swipe.len() - 1] - times[0];
    if !(total > 0.0 && duration > 0.0 && duration.is_finite()) {
        return weights;
    }
    let mean = duration / total;
    let window = DWELL_WINDOW * radius;
    for (i, weight) in weights.iter_mut().enumerate() {
        let at = total * i as f32 / SAMPLES as f32;
        let (from, to) = ((at - window).max(0.0), (at + window).min(total));
        if to <= from {
            continue;
        }
        let slowness = (time_at(&along, times, to) - time_at(&along, times, from)) / (to - from);
        *weight = (slowness / mean)
            .powf(dwell)
            .clamp(1.0 / DWELL_LIMIT, DWELL_LIMIT);
    }
    weights
}

/// The time the swipe reached `at` along its path: linear between points.
fn time_at(along: &[f32], times: &[f32], at: f32) -> f32 {
    let i = along.partition_point(|&s| s < at);
    if i == 0 {
        return times[0];
    }
    if i >= along.len() {
        return times[along.len() - 1];
    }
    let span = along[i] - along[i - 1];
    if span <= 0.0 {
        return times[i];
    }
    times[i - 1] + (times[i] - times[i - 1]) * (at - along[i - 1]) / span
}

/// How likely a word whose path is `path` (its keys' centres) is, by reach alone, for `swipe` as
/// drawn, keys of `radius`.
pub(crate) fn reach_probability(reach: f32, swipe: &Gesture, path: &Gesture, radius: f32) -> f32 {
    if reach == 0.0 || swipe.is_empty() || path.is_empty() {
        return 1.0;
    }
    let beyond = |d: f32| (d / radius - REACH_SLACK).max(0.0);
    let strays = beyond(farthest(swipe, path));
    let off = beyond(farthest(path, swipe));
    (-(reach * (strays * strays + off * off) / 2.0)).exp()
}

/// How far the point of `points` farthest from the line through `line`'s points is from it.
pub(crate) fn farthest(points: &Gesture, line: &Gesture) -> f32 {
    let mut farthest = 0.0f32;
    for i in 0..points.len() {
        let (x, y) = (points.x(i), points.y(i));
        let mut nearest = distance(x, y, line.x(0), line.y(0));
        for j in 1..line.len() {
            let segment = to_segment(x, y, (line.x(j - 1), line.y(j - 1)), (line.x(j), line.y(j)));
            nearest = nearest.min(segment);
        }
        farthest = farthest.max(nearest);
    }
    farthest
}

/// The distance from (`x`, `y`) to the segment from `a` to `b`.
fn to_segment(x: f32, y: f32, a: (f32, f32), b: (f32, f32)) -> f32 {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let squared = dx * dx + dy * dy;
    let t = if squared > 0.0 {
        (((x - a.0) * dx + (y - a.1) * dy) / squared).clamp(0.0, 1.0)
    } else {
        0.0
    };
    distance(x, y, a.0 + t * dx, a.1 + t * dy)
}

/// The location distance FlorisBoard's classifier uses, with each point counted at its weight.
pub(crate) fn weighted_location_distance(a: &Gesture, b: &Gesture, weights: &[f32]) -> f32 {
    let mut total = 0.0f32;
    let mut weight = 0.0f32;
    for (i, &w) in weights.iter().enumerate().take(SAMPLES) {
        total += w * ((a.x(i) - b.x(i)).abs() + (a.y(i) - b.y(i)).abs());
        weight += w;
    }
    total / weight / 2.0
}

#[cfg(test)]
#[path = "tuning_tests.rs"]
mod tests;
