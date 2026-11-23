// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// `ColorSpec2021`: its tone solver, its palettes, and the lookup from a role
// to its 2021 definition. The definitions themselves are in
// `spec2021_colors`.

use crate::Style;
use crate::color::sanitize_degrees;
use crate::contrast::{
    darker, foreground_tone, lighter, ratio_of_tones, tone_prefers_light_foreground,
};
use crate::dynamic::{ColorDef, DynScheme, Polarity, Role};
use crate::hct::Hct;
use crate::jmath;
use crate::spec2021_colors as c;
use crate::temperature::{TemperatureCache, fix_if_disliked};
use crate::tonal::TonalPalette;

/// The 2021 definition of a role.
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
        Role::Scrim => &c::SCRIM,
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

/// `ColorSpec2021.getTone`.
pub(crate) fn get_tone(s: &DynScheme, color: Role) -> f64 {
    match color.tone_delta_pair(s) {
        Some(pair) => paired_tone(s, color, pair),
        None => contrasted_tone(s, color),
    }
}

/// The tone-delta-pair branch of `ColorSpec2021.getTone`.
fn paired_tone(s: &DynScheme, color: Role, pair: crate::dynamic::ToneDeltaPair) -> f64 {
    let decreasing_contrast = s.contrast_level < 0.0;
    let delta = pair.delta;
    let a_is_nearer = pair.polarity == Polarity::Nearer
        || (pair.polarity == Polarity::Lighter && !s.is_dark)
        || (pair.polarity == Polarity::Darker && !s.is_dark);
    let nearer = if a_is_nearer {
        pair.role_a
    } else {
        pair.role_b
    };
    let farther = if a_is_nearer {
        pair.role_b
    } else {
        pair.role_a
    };
    let am_nearer = color.name() == nearer.name();
    let expansion_dir = if s.is_dark { 1.0 } else { -1.0 };
    let mut n_tone = nearer.raw_tone(s);
    let mut f_tone = farther.raw_tone(s);
    if color.has_background(s.spec)
        && nearer.has_contrast_curve(s.spec)
        && farther.has_contrast_curve(s.spec)
    {
        let bg = color.background(s);
        let n_curve = nearer.contrast_curve(s);
        let f_curve = farther.contrast_curve(s);
        if let (Some(bg), Some(n_curve), Some(f_curve)) = (bg, n_curve, f_curve) {
            let n_contrast = n_curve.get(s.contrast_level);
            let f_contrast = f_curve.get(s.contrast_level);
            let bg_tone = bg.tone(s);
            if ratio_of_tones(bg_tone, n_tone) < n_contrast {
                n_tone = foreground_tone(bg_tone, n_contrast);
            }
            if ratio_of_tones(bg_tone, f_tone) < f_contrast {
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
            if decreasing_contrast {
                n_tone = foreground_tone(bg_tone, n_contrast);
                f_tone = foreground_tone(bg_tone, f_contrast);
            }
        }
    }
    if (f_tone - n_tone) * expansion_dir < delta {
        f_tone = jmath::coerce_in(n_tone + delta * expansion_dir, 0.0, 100.0);
        if (f_tone - n_tone) * expansion_dir < delta {
            n_tone = jmath::coerce_in(f_tone - delta * expansion_dir, 0.0, 100.0);
        }
    }
    let n_in_band = (50.0..60.0).contains(&n_tone);
    if n_in_band || ((50.0..60.0).contains(&f_tone) && pair.stay_together) {
        if expansion_dir > 0.0 {
            n_tone = 60.0;
            f_tone = jmath::max(f_tone, n_tone + delta * expansion_dir);
        } else {
            n_tone = 49.0;
            f_tone = jmath::min(f_tone, n_tone + delta * expansion_dir);
        }
    } else if (50.0..60.0).contains(&f_tone) {
        f_tone = if expansion_dir > 0.0 { 60.0 } else { 49.0 };
    }
    if am_nearer { n_tone } else { f_tone }
}

/// The single-colour branch of `ColorSpec2021.getTone`.
///
/// An extended colour's `background` and `contrastCurve` lambdas are never
/// null, so a 2021 surface (which has neither) falls through to a
/// background tone and ratio of 0; at a negative contrast level that solves
/// to `foregroundTone(0, 0)`, which is 100.
fn contrasted_tone(s: &DynScheme, color: Role) -> f64 {
    let mut answer = color.raw_tone(s);
    if !color.has_background(s.spec) || !color.has_contrast_curve(s.spec) {
        return answer;
    }
    let bg_tone = color.background(s).map_or(0.0, |bg| bg.tone(s));
    let desired_ratio = color
        .contrast_curve(s)
        .map_or(0.0, |curve| curve.get(s.contrast_level));
    if ratio_of_tones(bg_tone, answer) < desired_ratio {
        answer = foreground_tone(bg_tone, desired_ratio);
    }
    if s.contrast_level < 0.0 {
        answer = foreground_tone(bg_tone, desired_ratio);
    }
    if color.is_background() && (50.0..60.0).contains(&answer) {
        answer = if ratio_of_tones(49.0, bg_tone) >= desired_ratio {
            49.0
        } else {
            60.0
        };
    }
    let Some(second_background) = color.second_background(s).map(|bg| bg.tone(s)) else {
        return answer;
    };
    let bg_tone1 = color.background(s).map_or(0.0, |bg| bg.tone(s));
    let bg_tone2 = second_background;
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
        return if light_option == -1.0 {
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
    if dark_option == -1.0 {
        0.0
    } else {
        dark_option
    }
}

/// `DynamicScheme.getPiecewiseValue`.
pub(crate) fn piecewise_value(source: &Hct, breakpoints: &[f64], values: &[f64]) -> f64 {
    let size = (breakpoints.len() - 1).min(values.len());
    let source_hue = source.hue;
    for i in 0..size {
        if source_hue >= breakpoints[i] && source_hue < breakpoints[i + 1] {
            return sanitize_degrees(values[i]);
        }
    }
    source_hue
}

/// `DynamicScheme.getRotatedHue`.
pub(crate) fn rotated_hue(source: &Hct, breakpoints: &[f64], rotations: &[f64]) -> f64 {
    let mut rotation = piecewise_value(source, breakpoints, rotations);
    if (breakpoints.len() - 1).min(rotations.len()) == 0 {
        rotation = 0.0;
    }
    sanitize_degrees(source.hue + rotation)
}

fn hc(hue: f64, chroma: f64) -> TonalPalette {
    TonalPalette::from_hue_and_chroma(hue, chroma)
}

/// `ColorSpec2021.getSecondaryPalette`.
pub(crate) fn secondary_palette(variant: Style, src: &Hct) -> TonalPalette {
    match variant {
        Style::Content | Style::Fidelity => {
            hc(src.hue, jmath::max(src.chroma - 32.0, src.chroma * 0.5))
        }
        Style::FruitSalad => hc(sanitize_degrees(src.hue - 50.0), 36.0),
        Style::Monochrome => hc(src.hue, 0.0),
        Style::Neutral => hc(src.hue, 8.0),
        Style::Rainbow | Style::TonalSpot => hc(src.hue, 16.0),
        Style::Expressive => hc(
            rotated_hue(
                src,
                &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                &[45.0, 95.0, 45.0, 20.0, 45.0, 90.0, 45.0, 45.0, 45.0],
            ),
            24.0,
        ),
        Style::Vibrant => hc(
            rotated_hue(
                src,
                &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                &[18.0, 15.0, 10.0, 12.0, 15.0, 18.0, 15.0, 12.0, 12.0],
            ),
            24.0,
        ),
    }
}

/// `ColorSpec2021.getTertiaryPalette`.
pub(crate) fn tertiary_palette(variant: Style, src: &Hct) -> TonalPalette {
    match variant {
        Style::Content => {
            let analogous = TemperatureCache::new(*src).analogous_colors(3, 6);
            TonalPalette::from_hct(fix_if_disliked(analogous[2]))
        }
        Style::Fidelity => {
            TonalPalette::from_hct(fix_if_disliked(TemperatureCache::new(*src).complement()))
        }
        Style::FruitSalad => hc(src.hue, 36.0),
        Style::Monochrome => hc(src.hue, 0.0),
        Style::Neutral => hc(src.hue, 16.0),
        Style::Rainbow | Style::TonalSpot => hc(sanitize_degrees(src.hue + 60.0), 24.0),
        Style::Expressive => hc(
            rotated_hue(
                src,
                &[0.0, 21.0, 51.0, 121.0, 151.0, 191.0, 271.0, 321.0, 360.0],
                &[120.0, 120.0, 20.0, 45.0, 20.0, 15.0, 20.0, 120.0, 120.0],
            ),
            32.0,
        ),
        Style::Vibrant => hc(
            rotated_hue(
                src,
                &[0.0, 41.0, 61.0, 101.0, 131.0, 181.0, 251.0, 301.0, 360.0],
                &[35.0, 30.0, 20.0, 25.0, 30.0, 35.0, 30.0, 25.0, 25.0],
            ),
            32.0,
        ),
    }
}

/// `ColorSpec2021.getNeutralPalette`.
pub(crate) fn neutral_palette(variant: Style, src: &Hct) -> TonalPalette {
    match variant {
        Style::Content | Style::Fidelity => hc(src.hue, src.chroma / 8.0),
        Style::FruitSalad | Style::Vibrant => hc(src.hue, 10.0),
        Style::Monochrome | Style::Rainbow => hc(src.hue, 0.0),
        Style::Neutral => hc(src.hue, 2.0),
        Style::TonalSpot => hc(src.hue, 6.0),
        Style::Expressive => hc(sanitize_degrees(src.hue + 15.0), 8.0),
    }
}

/// `ColorSpec2021.getNeutralVariantPalette`.
pub(crate) fn neutral_variant_palette(variant: Style, src: &Hct) -> TonalPalette {
    match variant {
        Style::Content | Style::Fidelity => hc(src.hue, (src.chroma / 8.0) + 4.0),
        Style::FruitSalad => hc(src.hue, 16.0),
        Style::Monochrome | Style::Rainbow => hc(src.hue, 0.0),
        Style::Neutral => hc(src.hue, 2.0),
        Style::TonalSpot => hc(src.hue, 8.0),
        Style::Expressive => hc(sanitize_degrees(src.hue + 15.0), 12.0),
        Style::Vibrant => hc(src.hue, 12.0),
    }
}
