//! Keys resized by what comes next, invisibly, as iOS and Gboard do: after "th" a tap on the E/R
//! border is more likely an E, so near that border E takes a little of R's room, though both keys
//! look the same. Both renderers' hit tests go through here, after [`crate::aim::retarget`].
//!
//! The rule is Bayes' with a Gaussian tap: a tap meant for a key lands around its centre with a
//! spread σ, so the border between two neighbours where both are equally likely moves towards the
//! less likely one by (σ / key)² × ln(odds) key widths (or heights, between rows). `Resize` holds
//! that factor and how far a border may move at most. It is conservative on purpose:
//!
//! * a border moves at most [`Resize::cap`] of a key, so a tap further inside its key than that
//!   always types it;
//! * only a tap's own key and the neighbour just across the nearest borders compete;
//! * keys without odds (not letters, or nothing known of the word being typed) never move.

use crate::Rect;

/// How strongly the letter odds move the borders.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Resize {
    /// How far a border between keys side by side moves per nat of their letters' log odds, in
    /// key widths: (σ / key width)² for a tap's spread σ across.
    pub across: f32,
    /// The same between rows, in key heights.
    pub down: f32,
    /// The most a border moves, in key widths or heights.
    pub cap: f32,
}

impl Resize {
    /// What the keyboard uses, tuned on held-out text (docs/key-resize-results.md): a tap spread
    /// of 0.2 key across and 0.16 row down, a little tighter than the scoreboard's typists, so
    /// borders move less than the odds alone would move them; and at most 0.15 key.
    pub const SHIPPED: Resize = Resize {
        across: 0.04,
        down: 0.025,
        cap: 0.15,
    };

    /// Never moves a border.
    pub const OFF: Resize = Resize {
        across: 0.0,
        down: 0.0,
        cap: 0.0,
    };
}

/// One side of a key: which way it faces, across (`true`) or down.
const SIDES: [(f32, f32, bool); 4] = [
    (-1.0, 0.0, true),
    (1.0, 0.0, true),
    (0.0, -1.0, false),
    (0.0, 1.0, false),
];
/// How far past a border, in pixels, the neighbour across it is looked for.
const PAST: f32 = 0.5;

/// The key a tap at (`x`, `y`) types when the plain hit test found `hit`, given `prior(i)`, the
/// natural log of how likely key `i`'s letter is to come next (none for a key that has no odds),
/// `bounds(i)`, key `i`'s touch bounds, and `key_at`, the plain hit test.
///
/// The tap moves to the neighbour across one of `hit`'s sides when it lands within that border's
/// move of it; of two such neighbours, the one it is further over the moved border for.
pub fn retarget(
    hit: usize,
    (x, y): (f32, f32),
    tuning: &Resize,
    prior: impl Fn(usize) -> Option<f32>,
    bounds: impl Fn(usize) -> Option<Rect>,
    key_at: impl Fn(f32, f32) -> Option<usize>,
) -> usize {
    let (Some(own), Some(r)) = (prior(hit), bounds(hit)) else {
        return hit;
    };
    if tuning.cap <= 0.0 || r.is_empty() {
        return hit;
    }
    let mut best = (hit, 0.0f32);
    for (sx, sy, across) in SIDES {
        let (size, inside, edge) = if across {
            let edge = if sx < 0.0 { r.left } else { r.right };
            (r.width(), (x - edge).abs(), (edge + sx * PAST, y))
        } else {
            let edge = if sy < 0.0 { r.top } else { r.bottom };
            (r.height(), (y - edge).abs(), (x, edge + sy * PAST))
        };
        let inside = inside / size;
        let Some(other) = key_at(edge.0, edge.1).filter(|&k| k != hit) else {
            continue;
        };
        let Some(theirs) = prior(other) else {
            continue;
        };
        let factor = if across { tuning.across } else { tuning.down };
        let moved = (factor * (theirs - own)).min(tuning.cap);
        let over = moved - inside;
        if over > best.1 {
            best = (other, over);
        }
    }
    best.0
}

/// The key a tap types with both what this user's aim has taught (`shift`, as in
/// [`crate::aim::retarget`]) and the letter odds (`prior`, as in [`retarget`]).
///
/// The learned aim goes first: where it moves the tap, that is the key, and the odds never move
/// it again or back. Where it leaves the tap on `hit`, the odds may move it, measured from where
/// this user meant it: the tap taken back by `hit`'s own learned shift, if known, and no further
/// than `hit`'s border, so the odds never act on a move the aim declined beyond what they would
/// do for a tap on that border.
pub fn key_for(
    hit: usize,
    (x, y): (f32, f32),
    tuning: &Resize,
    shift: impl Fn(usize) -> Option<(f32, f32)>,
    prior: impl Fn(usize) -> Option<f32>,
    bounds: impl Fn(usize) -> Option<Rect>,
    key_at: impl Fn(f32, f32) -> Option<usize>,
) -> usize {
    let aimed = crate::aim::retarget(hit, (x, y), &shift, &key_at);
    if aimed != hit {
        return aimed;
    }
    let (mx, my) = match (shift(hit), bounds(hit)) {
        (Some((dx, dy)), Some(r)) => (
            (x - dx).clamp(r.left, r.right),
            (y - dy).clamp(r.top, r.bottom),
        ),
        _ => (x, y),
    };
    retarget(hit, (mx, my), tuning, prior, bounds, key_at)
}

#[cfg(test)]
mod tests;
