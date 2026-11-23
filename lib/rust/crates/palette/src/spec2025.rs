// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// `ColorSpec2025`: its tone and HCT solvers, its PHONE palettes, and the
// lookup from a role to its 2025 definition (in `spec2025_colors`).

use crate::Style;
use crate::contrast::{
    darker, foreground_tone, lighter, ratio_of_tones, tone_prefers_light_foreground,
};
use crate::dynamic::{ColorDef, Constraint, DynScheme, Polarity, Role, ToneDeltaPair};
use crate::hct::{self, Hct, is_yellow};
use crate::jmath;
use crate::spec2021::{self, piecewise_value, rotated_hue};
use crate::spec2025_colors as c;
use crate::tonal::TonalPalette;

/// The 2025 definition of a role `ColorSpec2025` overrides.
pub(crate) fn def(role: Role) -> &'static ColorDef {
    match role {
        Role::Background => &c::BACKGROUND,
        Role::Error => &c::ERROR,
        Role::ErrorContainer => &c::ERROR_CONTAINER,
        Role::InverseOnSurface => &c::INVERSE_ON_SURFACE,
        Role::InversePrimary => &c::INVERSE_PRIMARY,
        Role::InverseSurface => &c::INVERSE_SURFACE,
        Role::OnBackground => &c::ON_BACKGROUND,
        Role::OnError => &c::ON_ERROR,
        Role::OnErrorContainer => &c::ON_ERROR_CONTAINER,
        Role::OnPrimary => &c::ON_PRIMARY,
        Role::OnPrimaryContainer => &c::ON_PRIMARY_CONTAINER,
        Role::OnSecondary => &c::ON_SECONDARY,
        Role::OnSecondaryContainer => &c::ON_SECONDARY_CONTAINER,
        Role::OnSurface => &c::ON_SURFACE,
        Role::OnSurfaceVariant => &c::ON_SURFACE_VARIANT,
        Role::OnTertiary => &c::ON_TERTIARY,
        Role::OnTertiaryContainer => &c::ON_TERTIARY_CONTAINER,
        Role::Outline => &c::OUTLINE,
        Role::OutlineVariant => &c::OUTLINE_VARIANT,
        Role::Primary => &c::PRIMARY,
        Role::PrimaryContainer => &c::PRIMARY_CONTAINER,
        // Not overridden: `Role::def` never asks for it.
        Role::Scrim => spec2021::def(Role::Scrim),
        Role::Secondary => &c::SECONDARY,
        Role::SecondaryContainer => &c::SECONDARY_CONTAINER,
        Role::Surface => &c::SURFACE,
        Role::SurfaceTint => &c::SURFACE_TINT,
        Role::SurfaceBright => &c::SURFACE_BRIGHT,
        Role::SurfaceDim => &c::SURFACE_DIM,
        Role::SurfaceContainer => &c::SURFACE_CONTAINER,
        Role::SurfaceContainerHigh => &c::SURFACE_CONTAINER_HIGH,
        Role::SurfaceContainerHighest => &c::SURFACE_CONTAINER_HIGHEST,
        Role::SurfaceContainerLow => &c::SURFACE_CONTAINER_LOW,
        Role::SurfaceContainerLowest => &c::SURFACE_CONTAINER_LOWEST,
        Role::SurfaceVariant => &c::SURFACE_VARIANT,
        Role::Tertiary => &c::TERTIARY,
        Role::TertiaryContainer => &c::TERTIARY_CONTAINER,
        Role::PrimaryFixed => &c::PRIMARY_FIXED,
        Role::PrimaryFixedDim => &c::PRIMARY_FIXED_DIM,
        Role::OnPrimaryFixed => &c::ON_PRIMARY_FIXED,
        Role::OnPrimaryFixedVariant => &c::ON_PRIMARY_FIXED_VARIANT,
        Role::SecondaryFixed => &c::SECONDARY_FIXED,
        Role::SecondaryFixedDim => &c::SECONDARY_FIXED_DIM,
        Role::OnSecondaryFixed => &c::ON_SECONDARY_FIXED,
        Role::OnSecondaryFixedVariant => &c::ON_SECONDARY_FIXED_VARIANT,
        Role::TertiaryFixed => &c::TERTIARY_FIXED,
        Role::TertiaryFixedDim => &c::TERTIARY_FIXED_DIM,
        Role::OnTertiaryFixed => &c::ON_TERTIARY_FIXED,
        Role::OnTertiaryFixedVariant => &c::ON_TERTIARY_FIXED_VARIANT,
    }
}

/// `ColorSpec2025.getHct(...).toInt()`.
pub(crate) fn get_argb(s: &DynScheme, color: Role) -> u32 {
    let def = color.def(s.spec);
    let palette = (def.palette)(s);
    let tone = get_tone(s, color);
    let chroma_multiplier = def.chroma_multiplier.map_or(1.0, |cm| cm(s));
    if chroma_multiplier == 1.0 {
        return palette.argb_at(tone);
    }
    let chroma = palette.chroma * chroma_multiplier;
    if tone == 99.0 && is_yellow(palette.hue) {
        return TonalPalette::from_hue_and_chroma(palette.hue, chroma).argb_at(tone);
    }
    hct::solve(palette.hue, chroma, tone)
}

/// `ColorSpec2025.getTone`.
pub(crate) fn get_tone(s: &DynScheme, color: Role) -> f64 {
    match color.tone_delta_pair(s) {
        Some(pair) => paired_tone(s, color, pair),
        None => contrasted_tone(s, color),
    }
}

/// Kotlin's `isBackground && !name.endsWith("_fixed_dim")` clamp, which
/// keeps background tones out of the 50-60 band.
fn clamp_background(color: Role, tone: f64) -> f64 {
    if !color.is_background() || color.name().ends_with("_fixed_dim") {
        tone
    } else if tone >= 57.0 {
        jmath::coerce_in(tone, 65.0, 100.0)
    } else {
        jmath::coerce_in(tone, 0.0, 49.0)
    }
}

/// The tone-delta-pair branch of `ColorSpec2025.getTone`.
fn paired_tone(s: &DynScheme, color: Role, pair: ToneDeltaPair) -> f64 {
    // Kotlin also negates for `RELATIVE_DARKER && !isDark`; see `Polarity`.
    let absolute_delta = if pair.polarity == Polarity::Darker
        || (pair.polarity == Polarity::RelativeLighter && s.is_dark)
    {
        -pair.delta
    } else {
        pair.delta
    };
    let am_role_a = color.name() == pair.role_a.name();
    let self_role = if am_role_a { pair.role_a } else { pair.role_b };
    let reference_role = if am_role_a { pair.role_b } else { pair.role_a };
    let mut self_tone = self_role.raw_tone(s);
    let reference_tone = reference_role.tone(s);
    let relative_delta = absolute_delta * if am_role_a { 1.0 } else { -1.0 };
    self_tone = match pair.constraint {
        Constraint::Exact => jmath::coerce_in(reference_tone + relative_delta, 0.0, 100.0),
        Constraint::Farther if relative_delta > 0.0 => {
            jmath::coerce_in(self_tone, reference_tone + relative_delta, 100.0)
        }
        Constraint::Farther => jmath::coerce_in(self_tone, 0.0, reference_tone + relative_delta),
    };
    if color.has_background(s.spec)
        && color.has_contrast_curve(s.spec)
        && let (Some(background), Some(curve)) = (color.background(s), color.contrast_curve(s))
    {
        let bg_tone = background.tone(s);
        let self_contrast = curve.get(s.contrast_level);
        let ratio = ratio_of_tones(bg_tone, self_tone);
        if !(ratio >= self_contrast && s.contrast_level >= 0.0) {
            self_tone = foreground_tone(bg_tone, self_contrast);
        }
    }
    clamp_background(color, self_tone)
}

/// The single-colour branch of `ColorSpec2025.getTone`.
fn contrasted_tone(s: &DynScheme, color: Role) -> f64 {
    let mut answer = color.raw_tone(s);
    let bg_tone = color.background(s).map(|bg| bg.tone(s));
    let desired_ratio = color
        .contrast_curve(s)
        .map(|curve| curve.get(s.contrast_level));
    let (Some(bg_tone), Some(desired_ratio)) = (bg_tone, desired_ratio) else {
        return answer;
    };
    let ratio = ratio_of_tones(bg_tone, answer);
    if !(ratio >= desired_ratio && s.contrast_level >= 0.0) {
        answer = foreground_tone(bg_tone, desired_ratio);
    }
    answer = clamp_background(color, answer);
    let Some(bg_tone2) = color.second_background(s).map(|bg| bg.tone(s)) else {
        return answer;
    };
    let bg_tone1 = bg_tone;
    let upper = jmath::max(bg_tone1, bg_tone2);
    let lower = jmath::min(bg_tone1, bg_tone2);
    if ratio_of_tones(upper, answer) >= desired_ratio
        && ratio_of_tones(lower, answer) >= desired_ratio
    {
        return answer;
    }
    let light_option = lighter(upper, desired_ratio);
    let dark_option = darker(lower, desired_ratio);
    let available = usize::from(light_option != -1.0) + usize::from(dark_option != -1.0);
    let prefers_light =
        tone_prefers_light_foreground(bg_tone1) || tone_prefers_light_foreground(bg_tone2);
    if prefers_light {
        return if light_option < 0.0 {
            100.0
        } else {
            light_option
        };
    }
    if available == 1 {
        return if light_option != -1.0 {
            light_option
        } else {
            dark_option
        };
    }
    if dark_option < 0.0 { 0.0 } else { dark_option }
}

fn hc(hue: f64, chroma: f64) -> TonalPalette {
    TonalPalette::from_hue_and_chroma(hue, chroma)
}

const EXPRESSIVE_BREAKPOINTS: [f64; 9] =
    [0.0, 105.0, 140.0, 204.0, 253.0, 278.0, 300.0, 333.0, 360.0];
const VIBRANT_BREAKPOINTS: [f64; 6] = [0.0, 38.0, 105.0, 140.0, 333.0, 360.0];
const VIBRANT_ROTATIONS: [f64; 5] = [-14.0, 10.0, -14.0, 10.0, -14.0];

fn expressive_neutral_hue(src: &Hct) -> f64 {
    rotated_hue(
        src,
        &[0.0, 71.0, 124.0, 253.0, 278.0, 300.0, 360.0],
        &[10.0, 0.0, 10.0, 0.0, 10.0, 0.0],
    )
}

fn expressive_neutral_chroma(src: &Hct, is_dark: bool) -> f64 {
    let neutral_hue = expressive_neutral_hue(src);
    if !is_dark {
        18.0
    } else if is_yellow(neutral_hue) {
        6.0
    } else {
        14.0
    }
}

fn vibrant_neutral_hue(src: &Hct) -> f64 {
    rotated_hue(src, &VIBRANT_BREAKPOINTS, &VIBRANT_ROTATIONS)
}

/// The palettes `SchemeX` builds for every role but primary, which
/// MaterialKolor replaces with the seed's own palette.
pub(crate) struct Palettes {
    pub(crate) secondary: TonalPalette,
    pub(crate) tertiary: TonalPalette,
    pub(crate) neutral: TonalPalette,
    pub(crate) neutral_variant: TonalPalette,
    pub(crate) error: TonalPalette,
}

/// `ColorSpec2025`'s palettes for a 2025 variant, `None` for the others
/// (which `ColorSpec2025` hands to `ColorSpec2021`).
pub(crate) fn palettes(variant: Style, src: &Hct, is_dark: bool) -> Option<Palettes> {
    let error_hue = piecewise_value(
        src,
        &[0.0, 3.0, 13.0, 23.0, 33.0, 43.0, 153.0, 273.0, 360.0],
        &[12.0, 22.0, 32.0, 12.0, 22.0, 32.0, 22.0, 12.0],
    );
    let palettes = match variant {
        Style::Neutral => Palettes {
            secondary: hc(src.hue, if src.is_blue() { 6.0 } else { 4.0 }),
            tertiary: hc(
                rotated_hue(
                    src,
                    &[0.0, 38.0, 105.0, 161.0, 204.0, 278.0, 333.0, 360.0],
                    &[-32.0, 26.0, 10.0, -39.0, 24.0, -15.0, -32.0],
                ),
                20.0,
            ),
            neutral: hc(src.hue, 1.4),
            neutral_variant: hc(src.hue, 1.4 * 2.2),
            error: hc(error_hue, 50.0),
        },
        Style::TonalSpot => Palettes {
            secondary: hc(src.hue, 16.0),
            tertiary: hc(
                rotated_hue(
                    src,
                    &[0.0, 20.0, 71.0, 161.0, 333.0, 360.0],
                    &[-40.0, 48.0, -32.0, 40.0, -32.0],
                ),
                28.0,
            ),
            neutral: hc(src.hue, 5.0),
            neutral_variant: hc(src.hue, 5.0 * 1.7),
            error: hc(error_hue, 60.0),
        },
        Style::Expressive => {
            let neutral_hue = expressive_neutral_hue(src);
            let neutral_chroma = expressive_neutral_chroma(src, is_dark);
            let d = if (105.0..125.0).contains(&neutral_hue) {
                1.6
            } else {
                2.3
            };
            Palettes {
                secondary: hc(
                    rotated_hue(
                        src,
                        &EXPRESSIVE_BREAKPOINTS,
                        &[-160.0, 155.0, -100.0, 96.0, -96.0, -156.0, -165.0, -160.0],
                    ),
                    if is_dark { 16.0 } else { 24.0 },
                ),
                tertiary: hc(
                    rotated_hue(
                        src,
                        &EXPRESSIVE_BREAKPOINTS,
                        &[-165.0, 160.0, -105.0, 101.0, -101.0, -160.0, -170.0, -165.0],
                    ),
                    48.0,
                ),
                neutral: hc(neutral_hue, neutral_chroma),
                neutral_variant: hc(neutral_hue, neutral_chroma * d),
                error: hc(error_hue, 64.0),
            }
        }
        Style::Vibrant => {
            let neutral_hue = vibrant_neutral_hue(src);
            Palettes {
                secondary: hc(
                    rotated_hue(src, &VIBRANT_BREAKPOINTS, &VIBRANT_ROTATIONS),
                    56.0,
                ),
                tertiary: hc(
                    rotated_hue(
                        src,
                        &[0.0, 38.0, 71.0, 105.0, 140.0, 161.0, 253.0, 333.0, 360.0],
                        &[-72.0, 35.0, 24.0, -24.0, 62.0, 50.0, 62.0, -72.0],
                    ),
                    56.0,
                ),
                neutral: hc(neutral_hue, 28.0),
                neutral_variant: hc(neutral_hue, 28.0 * 1.29),
                error: hc(error_hue, 80.0),
            }
        }
        _ => return None,
    };
    Some(palettes)
}
