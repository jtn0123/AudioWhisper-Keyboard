// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// TonalPalette and its KeyColor search. The Kotlin class memoises `tone()`;
// the function is pure, so this port recomputes instead.

use crate::color;
use crate::hct::{self, Hct, is_yellow};
use crate::jmath;

/// `TonalPalette`: a hue and chroma, from which every tone is derived.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TonalPalette {
    pub(crate) hue: f64,
    pub(crate) chroma: f64,
    pub(crate) key_color: Hct,
}

impl TonalPalette {
    /// `TonalPalette.fromInt`.
    pub(crate) fn from_int(argb: u32) -> TonalPalette {
        TonalPalette::from_hct(Hct::from_int(argb))
    }

    /// `TonalPalette.fromHct`.
    pub(crate) fn from_hct(hct: Hct) -> TonalPalette {
        TonalPalette {
            hue: hct.hue,
            chroma: hct.chroma,
            key_color: hct,
        }
    }

    /// `TonalPalette.fromHueAndChroma`, which searches for the key colour.
    pub(crate) fn from_hue_and_chroma(hue: f64, chroma: f64) -> TonalPalette {
        let key_color = key_color(hue, chroma);
        TonalPalette {
            hue,
            chroma,
            key_color,
        }
    }

    /// `TonalPalette.tone(Int)`.
    pub(crate) fn tone(&self, tone: i32) -> u32 {
        if tone == 99 && is_yellow(self.hue) {
            average_argb(self.tone(98), self.tone(100))
        } else {
            hct::solve(self.hue, self.chroma, f64::from(tone))
        }
    }

    /// `getHct(tone).toInt()`.
    pub(crate) fn argb_at(&self, tone: f64) -> u32 {
        if tone == 99.0 && is_yellow(self.hue) {
            return self.tone(99);
        }
        hct::solve(self.hue, self.chroma, tone)
    }

    /// `TonalPalette.getHct(Double)`.
    pub(crate) fn get_hct(&self, tone: f64) -> Hct {
        if tone == 99.0 && is_yellow(self.hue) {
            return Hct::from_int(self.tone(99));
        }
        Hct::from(self.hue, self.chroma, tone)
    }
}

/// `TonalPalette.averageArgb`: channel means, rounded as `Float.roundToInt`.
fn average_argb(argb1: u32, argb2: u32) -> u32 {
    let mean = |a: i32, b: i32| jmath::round_float_to_int((a + b) as f32 / 2.0);
    let red = mean(color::red(argb1), color::red(argb2));
    let green = mean(color::green(argb1), color::green(argb2));
    let blue = mean(color::blue(argb1), color::blue(argb2));
    color::argb_from_rgb(red, green, blue)
}

/// `KeyColor.create()`: the tone nearest 50 that can hold the chroma.
fn key_color(hue: f64, requested_chroma: f64) -> Hct {
    const PIVOT_TONE: i32 = 50;
    const TONE_STEP_SIZE: i32 = 1;
    const EPSILON: f64 = 0.01;
    let mut cache = [None; 102];
    let mut max_chroma = |tone: i32| -> f64 {
        *cache[tone as usize].get_or_insert_with(|| Hct::from(hue, 200.0, f64::from(tone)).chroma)
    };
    let mut lower_tone = 0;
    let mut upper_tone = 100;
    while lower_tone < upper_tone {
        let mid_tone = (lower_tone + upper_tone) / 2;
        let is_ascending = max_chroma(mid_tone) < max_chroma(mid_tone + TONE_STEP_SIZE);
        let sufficient_chroma = max_chroma(mid_tone) >= requested_chroma - EPSILON;
        if sufficient_chroma {
            if (lower_tone - PIVOT_TONE).abs() < (upper_tone - PIVOT_TONE).abs() {
                upper_tone = mid_tone;
            } else {
                if lower_tone == mid_tone {
                    return Hct::from(hue, requested_chroma, f64::from(lower_tone));
                }
                lower_tone = mid_tone;
            }
        } else if is_ascending {
            lower_tone = mid_tone + TONE_STEP_SIZE;
        } else {
            upper_tone = mid_tone;
        }
    }
    Hct::from(hue, requested_chroma, f64::from(lower_tone))
}
