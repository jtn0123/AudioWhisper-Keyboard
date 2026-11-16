//! Simulated taps: where a thumb lands when it aims at a key, and what the keyboard makes of it.
//!
//! A tap lands around the centre of the key aimed at, scattered as a Gaussian with a spread of a
//! quarter of the key pitch across (36 px, about 1.8 mm on the S24 Ultra) and a fifth of the row
//! pitch down (40 px). The keyboard finds the key under it by its touch bounds and works out the
//! tap's offset from that key's visible centre, as it does on the phone.
//!
//! [`Hand::LeftThumb`] types one-handed with the left thumb, pivoting near the keyboard's bottom
//! left corner: the further a key is from the pivot, the more the thumb falls short of it (up to
//! 0.15 of a key towards the left and 0.1 of a row down at the far right) and the wider it
//! scatters (up to 15% more).
//!
//! A tap that lands on a key that types no letter (space, shift, delete, the comma) or off the
//! keys is drawn again: those are other mistakes (a stray space, a lost letter) than the typos
//! auto-correction is for, and the keyboard would not see them as a word to correct.

use crate::geometry::{Key, Keyboard};
use crate::rng::Rng;
use suggest::tap::TapOffset;

/// The spread of a tap, in key pitches across and row pitches down.
const SPREAD_X: f32 = 0.25;
const SPREAD_Y: f32 = 0.2;
/// The left thumb's shortfall and extra spread at the far side of the keyboard.
const THUMB_SHORT_X: f32 = 0.15;
const THUMB_SHORT_Y: f32 = 0.1;
const THUMB_SPREAD: f32 = 0.15;
/// How many times a tap is drawn again before it is put on the key's centre.
const REDRAWS: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hand {
    /// Taps scatter evenly around the key aimed at.
    Neutral,
    /// One-handed, left thumb.
    LeftThumb,
}

impl Hand {
    pub fn name(self) -> &'static str {
        match self {
            Hand::Neutral => "neutral",
            Hand::LeftThumb => "left thumb",
        }
    }
}

/// One typed character: what the keyboard typed and where in its key the tap was, if it came from
/// a tap.
#[derive(Clone, Copy, Debug)]
pub struct Typed {
    pub unit: u16,
    pub offset: Option<TapOffset>,
}

/// Types `unit` aiming at its key. A character with no key of its own (an accented letter, an
/// apostrophe) is typed as it is, with no position, as from a long-press or the symbols page.
pub fn tap(keyboard: &Keyboard, unit: u16, hand: Hand, rng: &mut Rng) -> Typed {
    let lower = chars::to_lower(unit);
    let Some(target) = keyboard
        .key_for(i32::from(lower))
        .filter(|key| key.is_letter())
    else {
        return Typed { unit, offset: None };
    };
    let (pitch_x, pitch_y) = keyboard.pitch();
    let (cx, cy) = target.center();
    let Aim {
        bias_x,
        bias_y,
        spread,
    } = aim(keyboard, hand, cx, cy);
    for _ in 0..REDRAWS {
        let x = cx + bias_x + rng.gaussian() as f32 * SPREAD_X * spread * pitch_x;
        let y = cy + bias_y + rng.gaussian() as f32 * SPREAD_Y * spread * pitch_y;
        if let Some(hit) = keyboard.key_at(x, y).filter(|key| key.is_letter()) {
            return typed(hit, unit, x, y);
        }
    }
    typed(target, unit, cx, cy)
}

/// Where a thumb aiming at (cx, cy) lands on average, and how much more it scatters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aim {
    /// How far it falls from the point aimed at, in pixels.
    pub bias_x: f32,
    pub bias_y: f32,
    /// Its scatter as a multiple of a neutral tap's.
    pub spread: f32,
}

/// How `hand` aims at (cx, cy) on `keyboard`: the left thumb falls short of far keys and scatters
/// more there, as the module's comment says; a neutral hand aims true.
pub fn aim(keyboard: &Keyboard, hand: Hand, cx: f32, cy: f32) -> Aim {
    let (mut bias_x, mut bias_y, mut spread) = (0.0, 0.0, 1.0);
    if hand == Hand::LeftThumb {
        let (pitch_x, pitch_y) = keyboard.pitch();
        let reach = thumb_reach(keyboard, cx, cy);
        bias_x = -THUMB_SHORT_X * reach * pitch_x;
        bias_y = THUMB_SHORT_Y * reach * pitch_y;
        spread += THUMB_SPREAD * reach;
    }
    Aim {
        bias_x,
        bias_y,
        spread,
    }
}

/// Where the left thumb pivots: the keyboard's bottom left corner.
pub fn thumb_pivot(keyboard: &Keyboard) -> (f32, f32) {
    (0.0, keyboard.height)
}

/// How far (cx, cy) is from the left thumb's pivot, as a share of the keyboard's width.
fn thumb_reach(keyboard: &Keyboard, cx: f32, cy: f32) -> f32 {
    let (px, py) = thumb_pivot(keyboard);
    ((cx - px).hypot(cy - py) / keyboard.width).min(1.0)
}

/// What a tap at (x, y) on `hit` types when the user meant `unit`: `hit`'s letter in `unit`'s case.
fn typed(hit: &Key, unit: u16, x: f32, y: f32) -> Typed {
    let letter = hit.code as u16;
    let unit = if chars::is_upper_case(unit) {
        chars::to_upper(letter)
    } else {
        letter
    };
    let (cx, cy) = hit.center();
    Typed {
        unit,
        offset: TapOffset::of(x, y, cx, cy, hit.visible.width(), hit.visible.height()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn keyboard() -> Keyboard {
        let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main/assets");
        Keyboard::phone(&assets).unwrap()
    }

    fn miss_rate(keyboard: &Keyboard, unit: u16, hand: Hand) -> (f64, f32) {
        let mut rng = Rng::new(1);
        let n = 4000;
        let mut misses = 0;
        let mut dx = 0.0;
        for _ in 0..n {
            let typed = tap(keyboard, unit, hand, &mut rng);
            misses += usize::from(typed.unit != unit);
            if typed.unit == unit {
                dx += typed.offset.unwrap().dx;
            }
        }
        (misses as f64 / n as f64, dx / (n - misses) as f32)
    }

    #[test]
    fn taps_mostly_hit_the_key_aimed_at() {
        let keyboard = keyboard();
        let (misses, dx) = miss_rate(&keyboard, u16::from(b'g'), Hand::Neutral);
        assert!((0.03..0.10).contains(&misses), "{misses}");
        assert!(dx.abs() < 0.02, "{dx}");
    }

    #[test]
    fn the_left_thumb_falls_short_of_the_far_keys() {
        let keyboard = keyboard();
        let (near, _) = miss_rate(&keyboard, u16::from(b'z'), Hand::LeftThumb);
        let (far, dx) = miss_rate(&keyboard, u16::from(b'p'), Hand::LeftThumb);
        assert!(far > near, "{far} {near}");
        assert!(dx < -0.05, "{dx}");
    }

    #[test]
    fn keeps_the_case_and_types_keyless_characters_as_they_are() {
        let keyboard = keyboard();
        let mut rng = Rng::new(3);
        let typed = tap(&keyboard, u16::from(b'Q'), Hand::Neutral, &mut rng);
        assert!(chars::is_upper_case(typed.unit));
        let typed = tap(&keyboard, 0xE9, Hand::Neutral, &mut rng);
        assert_eq!((typed.unit, typed.offset), (0xE9, None));
    }
}
