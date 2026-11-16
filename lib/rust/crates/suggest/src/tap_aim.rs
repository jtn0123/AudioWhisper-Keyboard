//! Where this user's taps land in each key: a running mean and spread of their tap offsets per
//! key, learned only from words they typed and kept (see [`crate::tap_learning`]), so a thumb that
//! always falls a little short of P can be corrected for. Gboard learns the same thing.
//!
//! The model applies in two places, each only once a key has enough samples:
//!
//! * [`TapAim::aimed`]: the offset the decoder hears for a tap is moved by the key's learned bias,
//!   so a tap where this user usually hits P reads as a centred P.
//! * [`TapAim::retarget`]: which key an ambiguous tap near a border types. A tap moves to the
//!   neighbour only when both keys' learned aims agree that it was meant for the neighbour.
//!
//! Each key's estimate decays: its mean follows its last [`WINDOW`] samples, and every sample of
//! any key fades every other key's weight, so a key the user stopped hitting that way falls back
//! below [`MIN_SAMPLES`] and stops applying. Portrait and landscape are learned apart: the grip and
//! the keys' shape both change with the orientation.
//!
//! The saved text holds numbers only: the key's code point and its statistics, never a tap, a
//! position in the text or a word.

use std::collections::BTreeMap;

use crate::java;
use crate::tap::TapOffset;

/// Samples a key needs before its aim applies.
pub const MIN_SAMPLES: f32 = 16.0;
/// A key's mean follows about this many of its latest samples.
pub const WINDOW: f32 = 200.0;
/// Every sample fades every key's weight by `1 - 1 / FADE`: a key unused for that many samples
/// of the others keeps about a third of its weight.
pub const FADE: f32 = 20_000.0;
/// The most an aim moves a tap, in key widths and heights.
pub const CAP: f32 = 0.25;
/// A key's mean applies only beyond this many standard errors of it, so the noise in a few
/// samples (or an unbiased typist's) is not mistaken for a bias.
pub const CONFIDENCE: f32 = 1.0;
/// A bias smaller than this, in key widths or heights, is not applied. An unbiased typist's taps
/// show biases this small where the keyboard's edge or a neighbouring control cuts their scatter
/// off on one side; applied, they moved 2 of 400,000 neutral taps the wrong way in the scoreboard,
/// and this keeps all but 0.02 points of the left-thumb typist's gain (docs/tap-aim-results.md).
pub const MIN_BIAS: f32 = 0.03;
/// A sample is held to this far from the key's centre. A tap on the key itself is never further
/// out than [`TapOffset::LIMIT`]; one that landed on a neighbour and was corrected (see
/// [`crate::tap_learning`]) can be, by a little.
pub const SAMPLE_LIMIT: f32 = 0.75;
/// At most this many keys per orientation are kept.
pub const MAX_KEYS: usize = 64;

const HEADER: &str = "tap-aim 1";

/// The screen's orientation, which a learned aim belongs to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    #[default]
    Portrait,
    Landscape,
}

impl Orientation {
    pub fn of(landscape: bool) -> Self {
        if landscape {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        }
    }

    fn index(self) -> usize {
        match self {
            Orientation::Portrait => 0,
            Orientation::Landscape => 1,
        }
    }
}

/// One key's learned aim, in key widths (`dx`) and heights (`dy`) from its centre.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KeyAim {
    /// How many samples the estimate stands on, faded and at most [`WINDOW`].
    pub weight: f32,
    pub dx: f32,
    pub dy: f32,
    /// The spread: the variance of the samples about the mean, across and down.
    pub var_x: f32,
    pub var_y: f32,
}

impl KeyAim {
    fn add(&mut self, x: f32, y: f32) {
        self.weight = (self.weight + 1.0).min(WINDOW);
        let alpha = 1.0 / self.weight;
        let (dx, dy) = (x - self.dx, y - self.dy);
        self.dx += alpha * dx;
        self.dy += alpha * dy;
        self.var_x = (1.0 - alpha) * (self.var_x + alpha * dx * dx);
        self.var_y = (1.0 - alpha) * (self.var_y + alpha * dy * dy);
    }

    /// The part of the mean that applies: what is beyond [`CONFIDENCE`] standard errors of it,
    /// at most [`CAP`], and nothing unless that is at least [`MIN_BIAS`]. `None` below
    /// [`MIN_SAMPLES`].
    pub fn bias(&self) -> Option<TapOffset> {
        if self.weight.is_nan() || self.weight < MIN_SAMPLES {
            return None;
        }
        let along = |mean: f32, variance: f32| {
            let error = CONFIDENCE * (variance / self.weight).sqrt();
            let beyond = (mean.abs() - error).max(0.0);
            if beyond < MIN_BIAS {
                0.0
            } else {
                (beyond * mean.signum()).clamp(-CAP, CAP)
            }
        };
        Some(TapOffset {
            dx: along(self.dx, self.var_x),
            dy: along(self.dy, self.var_y),
        })
    }

    fn valid(&self) -> bool {
        let unit = |v: f32| v.is_finite() && (0.0..=1.0).contains(&v);
        let offset = |v: f32| v.is_finite() && v.abs() <= SAMPLE_LIMIT;
        self.weight.is_finite()
            && self.weight > 0.0
            && self.weight <= WINDOW
            && offset(self.dx)
            && offset(self.dy)
            && unit(self.var_x)
            && unit(self.var_y)
    }
}

/// A key as the keyboard lays it out, for [`TapAim::retarget`]: what it types and the size it is
/// drawn at, which a tap's offset is measured in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AimKey {
    pub code: i32,
    pub width: f32,
    pub height: f32,
}

/// The learned aim of every key, per orientation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TapAim {
    keys: [BTreeMap<i32, KeyAim>; 2],
}

/// The key a code point's taps are learned under: its lower case, if it is a letter.
fn key_of(code_point: i32) -> Option<i32> {
    let key = java::to_lower(code_point);
    u16::try_from(key)
        .ok()
        .filter(|&unit| chars::is_letter(unit))?;
    Some(key)
}

impl TapAim {
    pub fn is_empty(&self) -> bool {
        self.keys.iter().all(BTreeMap::is_empty)
    }

    pub fn clear(&mut self) {
        self.keys.iter_mut().for_each(BTreeMap::clear);
    }

    /// The learned aim of the key for `code_point` (either case).
    pub fn key(&self, orientation: Orientation, code_point: i32) -> Option<&KeyAim> {
        self.keys[orientation.index()].get(&key_of(code_point)?)
    }

    /// What applies for the key for `code_point`: see [`KeyAim::bias`].
    pub fn bias(&self, orientation: Orientation, code_point: i32) -> Option<TapOffset> {
        self.key(orientation, code_point)?.bias()
    }

    /// Learns that each key in `samples` was tapped at its offset, meaning that key.
    pub fn learn(&mut self, orientation: Orientation, samples: &[(i32, TapOffset)]) {
        let keys = &mut self.keys[orientation.index()];
        for &(code_point, offset) in samples {
            let Some(key) = key_of(code_point) else {
                continue;
            };
            if !offset.dx.is_finite() || !offset.dy.is_finite() {
                continue;
            }
            if !keys.contains_key(&key) && keys.len() >= MAX_KEYS {
                continue;
            }
            let fade = 1.0 - 1.0 / FADE;
            keys.iter_mut()
                .filter(|&(&other, _)| other != key)
                .for_each(|(_, aim)| aim.weight *= fade);
            let x = offset.dx.clamp(-SAMPLE_LIMIT, SAMPLE_LIMIT);
            let y = offset.dy.clamp(-SAMPLE_LIMIT, SAMPLE_LIMIT);
            keys.entry(key).or_default().add(x, y);
        }
    }

    /// A tap at `offset` on the key for `code_point`, as the decoder should hear it: moved by the
    /// key's learned bias, if one applies, and held to [`TapOffset::LIMIT`].
    pub fn aimed(&self, orientation: Orientation, code_point: i32, offset: TapOffset) -> TapOffset {
        match self.bias(orientation, code_point) {
            Some(bias) => TapOffset {
                dx: (offset.dx - bias.dx).clamp(-TapOffset::LIMIT, TapOffset::LIMIT),
                dy: (offset.dy - bias.dy).clamp(-TapOffset::LIMIT, TapOffset::LIMIT),
            },
            None => offset,
        }
    }

    /// Which of `keys` a tap at `at` types, given that the plain hit test (`key_at`) found `hit`
    /// (see [`keys::aim::retarget`]): each key's learned bias, in its own size, is how far this
    /// user's taps for it land off where they mean. Only letters with an aim that applies have one.
    pub fn retarget(
        &self,
        orientation: Orientation,
        keys: &[AimKey],
        hit: usize,
        at: (f32, f32),
        key_at: impl Fn(f32, f32) -> Option<usize>,
    ) -> usize {
        let shift = |index: usize| {
            let key = keys.get(index)?;
            let bias = self.bias(orientation, key.code)?;
            Some((bias.dx * key.width, bias.dy * key.height))
        };
        keys::aim::retarget(hit, at, shift, key_at)
    }

    /// The saved text: a header, then one line per key, numbers only.
    pub fn text(&self) -> Vec<u16> {
        let mut out = String::from(HEADER);
        out.push('\n');
        for (orientation, keys) in self.keys.iter().enumerate() {
            for (code, aim) in keys {
                out.push_str(&format!(
                    "{orientation} {code} {} {} {} {} {}\n",
                    aim.weight, aim.dx, aim.dy, aim.var_x, aim.var_y
                ));
            }
        }
        out.encode_utf16().collect()
    }

    /// Reads [`TapAim::text`]. A line that doesn't hold a key's numbers is skipped; text with
    /// another header gives an empty model.
    pub fn parse(text: &[u16]) -> TapAim {
        let mut aim = TapAim::default();
        let text = String::from_utf16_lossy(text);
        let mut lines = text.lines();
        if lines.next().map(str::trim_end) != Some(HEADER) {
            return aim;
        }
        for line in lines {
            if let Some((orientation, key, parsed)) = parse_line(line) {
                let keys = &mut aim.keys[orientation];
                if keys.len() < MAX_KEYS || keys.contains_key(&key) {
                    keys.insert(key, parsed);
                }
            }
        }
        aim
    }
}

fn parse_line(line: &str) -> Option<(usize, i32, KeyAim)> {
    let mut fields = line.split(' ');
    let orientation = match fields.next()? {
        "0" => 0,
        "1" => 1,
        _ => return None,
    };
    let code: i32 = fields.next()?.parse().ok()?;
    let mut number = || fields.next()?.parse::<f32>().ok();
    let aim = KeyAim {
        weight: number()?,
        dx: number()?,
        dy: number()?,
        var_x: number()?,
        var_y: number()?,
    };
    if fields.next().is_some() || key_of(code) != Some(code) || !aim.valid() {
        return None;
    }
    Some((orientation, code, aim))
}

#[cfg(test)]
mod tests;
