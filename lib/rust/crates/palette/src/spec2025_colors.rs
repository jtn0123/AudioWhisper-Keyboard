// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// The colour definitions `ColorSpec2025` extends its 2021 colours with, for
// the PHONE platform. Roles the Kotlin copies from another 2025 colour
// (`background` from `surface`, `surfaceTint` from `primary`, ...) are
// struct updates of that colour's constant.

use crate::Style;
use crate::contrast::ContrastCurve;
use crate::dynamic::{ColorDef, Constraint, DynScheme, Polarity, Role, ToneDeltaPair};
use crate::hct::Hct;
use crate::jmath;
use crate::tonal::TonalPalette;

mod accents;
pub(crate) use accents::*;

/// `findBestToneForChroma`, remembered per scheme.
fn best_tone(s: &DynScheme, hue: f64, chroma: f64, tone: f64, by_decreasing: bool) -> f64 {
    let key = [hue.to_bits(), chroma.to_bits(), tone.to_bits()];
    let memo = &s.memo.best_tone;
    if let Some(&(_, _, answer)) = memo
        .borrow()
        .iter()
        .find(|(k, d, _)| *k == key && *d == by_decreasing)
    {
        return answer;
    }
    let answer = find_best_tone_for_chroma(hue, chroma, tone, by_decreasing);
    memo.borrow_mut().push((key, by_decreasing, answer));
    answer
}

/// `ColorSpec2025.findBestToneForChroma`.
fn find_best_tone_for_chroma(hue: f64, chroma: f64, tone: f64, by_decreasing: bool) -> f64 {
    let mut tone = tone;
    let mut answer = tone;
    let mut best_candidate = Hct::from(hue, chroma, answer);
    while best_candidate.chroma < chroma {
        if !(0.0..=100.0).contains(&tone) {
            break;
        }
        tone += if by_decreasing { -1.0 } else { 1.0 };
        let new_candidate = Hct::from(hue, chroma, tone);
        if best_candidate.chroma < new_candidate.chroma {
            best_candidate = new_candidate;
            answer = tone;
        }
    }
    answer
}

/// `tMaxC(palette, lowerBound, upperBound, chromaMultiplier)`.
fn t_max_c_by(s: &DynScheme, p: &TonalPalette, lower: f64, upper: f64, multiplier: f64) -> f64 {
    let answer = best_tone(s, p.hue, p.chroma * multiplier, 100.0, true);
    jmath::coerce_in(answer, lower, upper)
}

/// `tMaxC(palette, lowerBound, upperBound)`.
fn t_max_c(s: &DynScheme, p: &TonalPalette, lower: f64, upper: f64) -> f64 {
    t_max_c_by(s, p, lower, upper, 1.0)
}

/// `tMinC(palette, lowerBound, upperBound)`.
fn t_min_c(s: &DynScheme, p: &TonalPalette, lower: f64, upper: f64) -> f64 {
    let answer = best_tone(s, p.hue, p.chroma, 0.0, false);
    jmath::coerce_in(answer, lower, upper)
}

/// `ColorSpec2025.getContrastCurve`.
const fn cc(default_contrast: f64) -> Option<ContrastCurve> {
    let (medium, high) = match default_contrast {
        1.5 => (3.0, 5.5),
        3.0 => (4.5, 7.0),
        4.5 => (7.0, 11.0),
        6.0 => (7.0, 11.0),
        7.0 => (11.0, 21.0),
        9.0 => (11.0, 21.0),
        11.0 => (21.0, 21.0),
        21.0 => (21.0, 21.0),
        _ => (7.0, 21.0),
    };
    Some(ContrastCurve::new(
        default_contrast,
        default_contrast,
        medium,
        high,
    ))
}

/// The `contrastLevel > 0` curve every container and fixed colour uses.
fn container_curve(s: &DynScheme) -> Option<ContrastCurve> {
    if s.contrast_level > 0.0 {
        cc(1.5)
    } else {
        None
    }
}

fn pick(s: &DynScheme, dark: f64, light: f64) -> f64 {
    if s.is_dark { dark } else { light }
}

/// The PHONE background of most colours: `surfaceBright` or `surfaceDim`.
fn highest(s: &DynScheme) -> Option<Role> {
    Some(if s.is_dark {
        Role::SurfaceBright
    } else {
        Role::SurfaceDim
    })
}

fn yellow_neutral(s: &DynScheme) -> bool {
    s.neutral.key_color.is_yellow()
}

/// A light surface tone: yellow neutral, vibrant, or otherwise.
fn light_tone(s: &DynScheme, yellow: f64, vibrant: f64, other: f64) -> f64 {
    if yellow_neutral(s) {
        yellow
    } else if s.variant == Style::Vibrant {
        vibrant
    } else {
        other
    }
}

/// A surface chroma multiplier per variant (1 for any other variant).
fn surface_cm(s: &DynScheme, neutral: f64, tonal: f64, expr: [f64; 2], vibrant: f64) -> f64 {
    match s.variant {
        Style::Neutral => neutral,
        Style::TonalSpot => tonal,
        Style::Expressive if yellow_neutral(s) => expr[0],
        Style::Expressive => expr[1],
        Style::Vibrant => vibrant,
        _ => 1.0,
    }
}

/// The chroma multiplier of `onSurface`, `onSurfaceVariant` and `outline`.
fn text_cm(s: &DynScheme) -> f64 {
    match s.variant {
        Style::Neutral => 2.2,
        Style::TonalSpot => 1.7,
        Style::Expressive if yellow_neutral(s) => pick(s, 3.0, 2.3),
        Style::Expressive => 1.6,
        _ => 1.0,
    }
}

const fn surface(name: &'static str, tone: fn(&DynScheme) -> f64) -> ColorDef {
    ColorDef {
        tone: Some(tone),
        is_background: true,
        ..ColorDef::new(name, |s| s.neutral)
    }
}

pub(crate) const SURFACE: ColorDef = surface("surface", |s| {
    if s.is_dark {
        4.0
    } else {
        light_tone(s, 99.0, 97.0, 98.0)
    }
});
pub(crate) const BACKGROUND: ColorDef = ColorDef {
    name: "background",
    ..SURFACE
};
pub(crate) const SURFACE_DIM: ColorDef = ColorDef {
    chroma_multiplier: Some(|s| {
        if s.is_dark {
            1.0
        } else {
            surface_cm(s, 2.5, 1.7, [2.7, 1.75], 1.36)
        }
    }),
    ..surface("surface_dim", |s| {
        if s.is_dark {
            4.0
        } else {
            light_tone(s, 90.0, 85.0, 87.0)
        }
    })
};
pub(crate) const SURFACE_BRIGHT: ColorDef = ColorDef {
    chroma_multiplier: Some(|s| {
        if s.is_dark {
            surface_cm(s, 2.5, 1.7, [2.7, 1.75], 1.36)
        } else {
            1.0
        }
    }),
    ..surface("surface_bright", |s| {
        if s.is_dark {
            18.0
        } else {
            light_tone(s, 99.0, 97.0, 98.0)
        }
    })
};
pub(crate) const SURFACE_CONTAINER_LOWEST: ColorDef =
    surface("surface_container_lowest", |s| pick(s, 0.0, 100.0));
pub(crate) const SURFACE_CONTAINER_LOW: ColorDef = ColorDef {
    chroma_multiplier: Some(|s| surface_cm(s, 1.3, 1.25, [1.3, 1.15], 1.08)),
    ..surface("surface_container_low", |s| {
        if s.is_dark {
            6.0
        } else {
            light_tone(s, 98.0, 95.0, 96.0)
        }
    })
};
pub(crate) const SURFACE_CONTAINER: ColorDef = ColorDef {
    chroma_multiplier: Some(|s| surface_cm(s, 1.6, 1.4, [1.6, 1.3], 1.15)),
    ..surface("surface_container", |s| {
        if s.is_dark {
            9.0
        } else {
            light_tone(s, 96.0, 92.0, 94.0)
        }
    })
};
pub(crate) const SURFACE_CONTAINER_HIGH: ColorDef = ColorDef {
    chroma_multiplier: Some(|s| surface_cm(s, 1.9, 1.5, [1.95, 1.45], 1.22)),
    ..surface("surface_container_high", |s| {
        if s.is_dark {
            12.0
        } else {
            light_tone(s, 94.0, 90.0, 92.0)
        }
    })
};
pub(crate) const SURFACE_CONTAINER_HIGHEST: ColorDef = ColorDef {
    chroma_multiplier: Some(|s| surface_cm(s, 2.2, 1.7, [2.3, 1.6], 1.29)),
    ..surface("surface_container_highest", |s| {
        if s.is_dark {
            15.0
        } else {
            light_tone(s, 92.0, 88.0, 90.0)
        }
    })
};
pub(crate) const SURFACE_VARIANT: ColorDef = ColorDef {
    name: "surface_variant",
    ..SURFACE_CONTAINER_HIGHEST
};
pub(crate) const ON_SURFACE: ColorDef = ColorDef {
    tone: Some(|s| {
        if s.variant == Style::Vibrant {
            t_max_c_by(s, &s.neutral, 0.0, 100.0, 1.1)
        } else {
            highest(s).map_or(50.0, |bg| bg.tone(s))
        }
    }),
    chroma_multiplier: Some(text_cm),
    background: Some(highest),
    contrast_curve: Some(|s| if s.is_dark { cc(11.0) } else { cc(9.0) }),
    ..ColorDef::new("on_surface", |s| s.neutral)
};
pub(crate) const ON_BACKGROUND: ColorDef = ColorDef {
    name: "on_background",
    tone: Some(|s| Role::OnSurface.tone(s)),
    ..ON_SURFACE
};
pub(crate) const ON_SURFACE_VARIANT: ColorDef = ColorDef {
    chroma_multiplier: Some(text_cm),
    background: Some(highest),
    contrast_curve: Some(|s| if s.is_dark { cc(6.0) } else { cc(4.5) }),
    ..ColorDef::new("on_surface_variant", |s| s.neutral)
};
pub(crate) const INVERSE_SURFACE: ColorDef = surface("inverse_surface", |s| pick(s, 98.0, 4.0));
pub(crate) const INVERSE_ON_SURFACE: ColorDef = ColorDef {
    background: Some(|_| Some(Role::InverseSurface)),
    contrast_curve: Some(|_| cc(7.0)),
    ..ColorDef::new("inverse_on_surface", |s| s.neutral)
};
pub(crate) const OUTLINE: ColorDef = ColorDef {
    chroma_multiplier: Some(text_cm),
    background: Some(highest),
    contrast_curve: Some(|_| cc(3.0)),
    ..ColorDef::new("outline", |s| s.neutral)
};
pub(crate) const OUTLINE_VARIANT: ColorDef = ColorDef {
    chroma_multiplier: Some(|s| {
        if s.variant == Style::Vibrant {
            1.29
        } else {
            text_cm(s)
        }
    }),
    background: Some(highest),
    contrast_curve: Some(|_| cc(1.5)),
    ..ColorDef::new("outline_variant", |s| s.neutral)
};
