// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// The colour definitions of `ColorSpec2021`, one constant per role.

use crate::Style;
use crate::contrast::{ContrastCurve, foreground_tone};
use crate::dynamic::{ColorDef, Constraint, DynScheme, Polarity, Role, ToneDeltaPair};
use crate::hct::Hct;
use crate::jmath;
use crate::temperature::fix_if_disliked;

mod accents;
pub(crate) use accents::*;

fn mono(s: &DynScheme) -> bool {
    s.variant == Style::Monochrome
}

fn fidelity(s: &DynScheme) -> bool {
    s.variant == Style::Fidelity || s.variant == Style::Content
}

/// `highestSurface`.
fn highest(s: &DynScheme) -> Option<Role> {
    Some(if s.is_dark {
        Role::SurfaceBright
    } else {
        Role::SurfaceDim
    })
}

/// `if (s.isDark) dark else light`.
fn pick(s: &DynScheme, dark: f64, light: f64) -> f64 {
    if s.is_dark { dark } else { light }
}

/// A `ContrastCurve` evaluated at the scheme's contrast level.
fn at(s: &DynScheme, low: f64, normal: f64, medium: f64, high: f64) -> f64 {
    ContrastCurve::new(low, normal, medium, high).get(s.contrast_level)
}

const fn curve(low: f64, normal: f64, medium: f64, high: f64) -> Option<ContrastCurve> {
    Some(ContrastCurve::new(low, normal, medium, high))
}

/// The deprecated `NEARER` pair between a container and its accent.
const fn nearer(role_a: Role, role_b: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair {
        role_a,
        role_b,
        delta: 10.0,
        polarity: Polarity::Nearer,
        stay_together: false,
        constraint: Constraint::Exact,
    })
}

/// The `LIGHTER`, stay-together pair between a fixed colour and its dim.
const fn lighter(role_a: Role, role_b: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair {
        role_a,
        role_b,
        delta: 10.0,
        polarity: Polarity::Lighter,
        stay_together: true,
        constraint: Constraint::Exact,
    })
}

/// `ColorSpec2021.findDesiredChromaByTone`.
fn find_desired_chroma_by_tone(hue: f64, chroma: f64, tone: f64, by_decreasing: bool) -> f64 {
    let mut answer = tone;
    let mut closest_to_chroma = Hct::from(hue, chroma, tone);
    if closest_to_chroma.chroma < chroma {
        let mut chroma_peak = closest_to_chroma.chroma;
        while closest_to_chroma.chroma < chroma {
            answer += if by_decreasing { -1.0 } else { 1.0 };
            let potential_solution = Hct::from(hue, chroma, answer);
            if chroma_peak > potential_solution.chroma {
                break;
            }
            if (potential_solution.chroma - chroma).abs() < 0.4 {
                break;
            }
            let potential_delta = (potential_solution.chroma - chroma).abs();
            let current_delta = (closest_to_chroma.chroma - chroma).abs();
            if potential_delta < current_delta {
                closest_to_chroma = potential_solution;
            }
            chroma_peak = jmath::max(chroma_peak, potential_solution.chroma);
        }
    }
    answer
}

/// A neutral-palette surface with a fixed tone per contrast level.
const fn surface(name: &'static str, tone: fn(&DynScheme) -> f64) -> ColorDef {
    ColorDef {
        tone: Some(tone),
        is_background: true,
        ..ColorDef::new(name, |s| s.neutral)
    }
}

pub(crate) const BACKGROUND: ColorDef = surface("background", |s| pick(s, 6.0, 98.0));
pub(crate) const ON_BACKGROUND: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 90.0, 10.0)),
    background: Some(|_| Some(Role::Background)),
    contrast_curve: Some(|_| curve(3.0, 3.0, 4.5, 7.0)),
    ..ColorDef::new("on_background", |s| s.neutral)
};
pub(crate) const SURFACE: ColorDef = surface("surface", |s| pick(s, 6.0, 98.0));
pub(crate) const SURFACE_DIM: ColorDef = surface("surface_dim", |s| {
    if s.is_dark {
        6.0
    } else {
        at(s, 87.0, 87.0, 80.0, 75.0)
    }
});
pub(crate) const SURFACE_BRIGHT: ColorDef = surface("surface_bright", |s| {
    if s.is_dark {
        at(s, 24.0, 24.0, 29.0, 34.0)
    } else {
        98.0
    }
});
pub(crate) const SURFACE_CONTAINER_LOWEST: ColorDef = surface("surface_container_lowest", |s| {
    if s.is_dark {
        at(s, 4.0, 4.0, 2.0, 0.0)
    } else {
        100.0
    }
});
pub(crate) const SURFACE_CONTAINER_LOW: ColorDef = surface("surface_container_low", |s| {
    if s.is_dark {
        at(s, 10.0, 10.0, 11.0, 12.0)
    } else {
        at(s, 96.0, 96.0, 96.0, 95.0)
    }
});
pub(crate) const SURFACE_CONTAINER: ColorDef = surface("surface_container", |s| {
    if s.is_dark {
        at(s, 12.0, 12.0, 16.0, 20.0)
    } else {
        at(s, 94.0, 94.0, 92.0, 90.0)
    }
});
pub(crate) const SURFACE_CONTAINER_HIGH: ColorDef = surface("surface_container_high", |s| {
    if s.is_dark {
        at(s, 17.0, 17.0, 21.0, 25.0)
    } else {
        at(s, 92.0, 92.0, 88.0, 85.0)
    }
});
pub(crate) const SURFACE_CONTAINER_HIGHEST: ColorDef = surface("surface_container_highest", |s| {
    if s.is_dark {
        at(s, 22.0, 22.0, 26.0, 30.0)
    } else {
        at(s, 90.0, 90.0, 84.0, 80.0)
    }
});
pub(crate) const ON_SURFACE: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 90.0, 10.0)),
    background: Some(highest),
    contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
    ..ColorDef::new("on_surface", |s| s.neutral)
};
pub(crate) const SURFACE_VARIANT: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 30.0, 90.0)),
    is_background: true,
    ..ColorDef::new("surface_variant", |s| s.neutral_variant)
};
pub(crate) const ON_SURFACE_VARIANT: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 80.0, 30.0)),
    background: Some(highest),
    contrast_curve: Some(|_| curve(3.0, 4.5, 7.0, 11.0)),
    ..ColorDef::new("on_surface_variant", |s| s.neutral_variant)
};
pub(crate) const INVERSE_SURFACE: ColorDef = surface("inverse_surface", |s| pick(s, 90.0, 20.0));
pub(crate) const INVERSE_ON_SURFACE: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 20.0, 95.0)),
    background: Some(|_| Some(Role::InverseSurface)),
    contrast_curve: Some(|_| curve(4.5, 7.0, 11.0, 21.0)),
    ..ColorDef::new("inverse_on_surface", |s| s.neutral)
};
pub(crate) const OUTLINE: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 60.0, 50.0)),
    background: Some(highest),
    contrast_curve: Some(|_| curve(1.5, 3.0, 4.5, 7.0)),
    ..ColorDef::new("outline", |s| s.neutral_variant)
};
pub(crate) const OUTLINE_VARIANT: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 30.0, 80.0)),
    background: Some(highest),
    contrast_curve: Some(|_| curve(1.0, 1.0, 3.0, 4.5)),
    ..ColorDef::new("outline_variant", |s| s.neutral_variant)
};
pub(crate) const SCRIM: ColorDef = ColorDef {
    tone: Some(|_| 0.0),
    ..ColorDef::new("scrim", |s| s.neutral)
};
pub(crate) const SURFACE_TINT: ColorDef = ColorDef {
    tone: Some(|s| pick(s, 80.0, 40.0)),
    is_background: true,
    ..ColorDef::new("surface_tint", |s| s.primary)
};
