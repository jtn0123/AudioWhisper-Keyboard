// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// Contrast, ContrastCurve, and the contrast helpers on DynamicColor's
// companion (foregroundTone and friends).

use crate::color::{lerp, lstar_from_y, y_from_lstar};
use crate::jmath;

const CONTRAST_RATIO_EPSILON: f64 = 0.04;
const LUMINANCE_GAMUT_MAP_TOLERANCE: f64 = 0.4;

/// `Contrast.ratioOfYs`.
fn ratio_of_ys(y1: f64, y2: f64) -> f64 {
    let lighter = jmath::max(y1, y2);
    let darker = if lighter == y2 { y1 } else { y2 };
    (lighter + 5.0) / (darker + 5.0)
}

/// `Contrast.ratioOfTones`.
pub(crate) fn ratio_of_tones(tone1: f64, tone2: f64) -> f64 {
    ratio_of_ys(y_from_lstar(tone1), y_from_lstar(tone2))
}

/// `Contrast.lighter`: a tone at least `ratio` lighter, or -1.
pub(crate) fn lighter(tone: f64, ratio: f64) -> f64 {
    if tone < 0.0 || tone > 100.0 {
        return -1.0;
    }
    let dark_y = y_from_lstar(tone);
    let light_y = ratio * (dark_y + 5.0) - 5.0;
    if light_y < 0.0 || light_y > 100.0 {
        return -1.0;
    }
    let real_contrast = ratio_of_ys(light_y, dark_y);
    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > CONTRAST_RATIO_EPSILON {
        return -1.0;
    }
    let return_value = lstar_from_y(light_y) + LUMINANCE_GAMUT_MAP_TOLERANCE;
    if return_value < 0.0 || return_value > 100.0 {
        -1.0
    } else {
        return_value
    }
}

/// `Contrast.lighterUnsafe`.
fn lighter_unsafe(tone: f64, ratio: f64) -> f64 {
    let lighter_safe = lighter(tone, ratio);
    if lighter_safe < 0.0 {
        100.0
    } else {
        lighter_safe
    }
}

/// `Contrast.darker`: a tone at least `ratio` darker, or -1.
pub(crate) fn darker(tone: f64, ratio: f64) -> f64 {
    if tone < 0.0 || tone > 100.0 {
        return -1.0;
    }
    let light_y = y_from_lstar(tone);
    let dark_y = (light_y + 5.0) / ratio - 5.0;
    if dark_y < 0.0 || dark_y > 100.0 {
        return -1.0;
    }
    let real_contrast = ratio_of_ys(light_y, dark_y);
    let delta = (real_contrast - ratio).abs();
    if real_contrast < ratio && delta > CONTRAST_RATIO_EPSILON {
        return -1.0;
    }
    let return_value = lstar_from_y(dark_y) - LUMINANCE_GAMUT_MAP_TOLERANCE;
    if return_value < 0.0 || return_value > 100.0 {
        -1.0
    } else {
        return_value
    }
}

/// `Contrast.darkerUnsafe`.
fn darker_unsafe(tone: f64, ratio: f64) -> f64 {
    jmath::max(0.0, darker(tone, ratio))
}

/// `DynamicColor.foregroundTone`: the tone that best contrasts `bg_tone`.
pub(crate) fn foreground_tone(bg_tone: f64, ratio: f64) -> f64 {
    let lighter_tone = lighter_unsafe(bg_tone, ratio);
    let darker_tone = darker_unsafe(bg_tone, ratio);
    let lighter_ratio = ratio_of_tones(lighter_tone, bg_tone);
    let darker_ratio = ratio_of_tones(darker_tone, bg_tone);
    if tone_prefers_light_foreground(bg_tone) {
        let negligible_difference = (lighter_ratio - darker_ratio).abs() < 0.1
            && lighter_ratio < ratio
            && darker_ratio < ratio;
        if lighter_ratio >= ratio || lighter_ratio >= darker_ratio || negligible_difference {
            lighter_tone
        } else {
            darker_tone
        }
    } else if darker_ratio >= ratio || darker_ratio >= lighter_ratio {
        darker_tone
    } else {
        lighter_tone
    }
}

/// `DynamicColor.tonePrefersLightForeground`.
pub(crate) fn tone_prefers_light_foreground(tone: f64) -> bool {
    jmath::rint(tone) < 60.0
}

/// `ContrastCurve`: a contrast ratio for each contrast level.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ContrastCurve {
    low: f64,
    normal: f64,
    medium: f64,
    high: f64,
}

impl ContrastCurve {
    /// `ContrastCurve(low, normal, medium, high)`.
    pub(crate) const fn new(low: f64, normal: f64, medium: f64, high: f64) -> ContrastCurve {
        ContrastCurve {
            low,
            normal,
            medium,
            high,
        }
    }

    /// `ContrastCurve.get`.
    pub(crate) fn get(&self, contrast_level: f64) -> f64 {
        if contrast_level <= -1.0 {
            self.low
        } else if contrast_level < 0.0 {
            lerp(self.low, self.normal, (contrast_level - -1.0) / 1.0)
        } else if contrast_level < 0.5 {
            lerp(self.normal, self.medium, (contrast_level - 0.0) / 0.5)
        } else if contrast_level < 1.0 {
            lerp(self.medium, self.high, (contrast_level - 0.5) / 0.5)
        } else {
            self.high
        }
    }
}
