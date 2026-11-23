// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// The accent, container, error and fixed colours of `ColorSpec2021`; the
// surfaces and shared helpers are in the parent module.

use super::*;

/// An accent (`primary`, `secondary`, ...) or its container: a background
/// paired with its partner by the deprecated `NEARER` polarity.
const fn accent(
    name: &'static str,
    palette: fn(&DynScheme) -> crate::tonal::TonalPalette,
    tone: fn(&DynScheme) -> f64,
    contrast_curve: fn(&DynScheme) -> Option<ContrastCurve>,
    tone_delta_pair: fn(&DynScheme) -> Option<ToneDeltaPair>,
) -> ColorDef {
    ColorDef {
        tone: Some(tone),
        is_background: true,
        background: Some(highest),
        contrast_curve: Some(contrast_curve),
        tone_delta_pair: Some(tone_delta_pair),
        ..ColorDef::new(name, palette)
    }
}

/// A foreground over `background` with the given tone and contrast curve.
const fn on(
    name: &'static str,
    palette: fn(&DynScheme) -> crate::tonal::TonalPalette,
    tone: fn(&DynScheme) -> f64,
    background: fn(&DynScheme) -> Option<Role>,
    contrast_curve: fn(&DynScheme) -> Option<ContrastCurve>,
) -> ColorDef {
    ColorDef {
        tone: Some(tone),
        background: Some(background),
        contrast_curve: Some(contrast_curve),
        ..ColorDef::new(name, palette)
    }
}

pub(crate) const PRIMARY: ColorDef = accent(
    "primary",
    |s| s.primary,
    |s| {
        if mono(s) {
            pick(s, 100.0, 0.0)
        } else {
            pick(s, 80.0, 40.0)
        }
    },
    |_| curve(3.0, 4.5, 7.0, 7.0),
    |_| nearer(Role::PrimaryContainer, Role::Primary),
);
pub(crate) const ON_PRIMARY: ColorDef = on(
    "on_primary",
    |s| s.primary,
    |s| {
        if mono(s) {
            pick(s, 10.0, 90.0)
        } else {
            pick(s, 20.0, 100.0)
        }
    },
    |_| Some(Role::Primary),
    |_| curve(4.5, 7.0, 11.0, 21.0),
);
pub(crate) const PRIMARY_CONTAINER: ColorDef = accent(
    "primary_container",
    |s| s.primary,
    |s| {
        if fidelity(s) {
            s.source.tone
        } else if mono(s) {
            pick(s, 85.0, 25.0)
        } else {
            pick(s, 30.0, 90.0)
        }
    },
    |_| curve(1.0, 1.0, 3.0, 4.5),
    |_| nearer(Role::PrimaryContainer, Role::Primary),
);
pub(crate) const ON_PRIMARY_CONTAINER: ColorDef = on(
    "on_primary_container",
    |s| s.primary,
    |s| {
        if fidelity(s) {
            foreground_tone(Role::PrimaryContainer.raw_tone(s), 4.5)
        } else if mono(s) {
            pick(s, 0.0, 100.0)
        } else {
            pick(s, 90.0, 30.0)
        }
    },
    |_| Some(Role::PrimaryContainer),
    |_| curve(3.0, 4.5, 7.0, 11.0),
);
pub(crate) const INVERSE_PRIMARY: ColorDef = on(
    "inverse_primary",
    |s| s.primary,
    |s| pick(s, 40.0, 80.0),
    |_| Some(Role::InverseSurface),
    |_| curve(3.0, 4.5, 7.0, 7.0),
);
pub(crate) const SECONDARY: ColorDef = accent(
    "secondary",
    |s| s.secondary,
    |s| pick(s, 80.0, 40.0),
    |_| curve(3.0, 4.5, 7.0, 7.0),
    |_| nearer(Role::SecondaryContainer, Role::Secondary),
);
pub(crate) const ON_SECONDARY: ColorDef = on(
    "on_secondary",
    |s| s.secondary,
    |s| {
        if mono(s) {
            pick(s, 10.0, 100.0)
        } else {
            pick(s, 20.0, 100.0)
        }
    },
    |_| Some(Role::Secondary),
    |_| curve(4.5, 7.0, 11.0, 21.0),
);
pub(crate) const SECONDARY_CONTAINER: ColorDef = accent(
    "secondary_container",
    |s| s.secondary,
    |s| {
        let initial_tone = pick(s, 30.0, 90.0);
        if mono(s) {
            pick(s, 30.0, 85.0)
        } else if !fidelity(s) {
            initial_tone
        } else {
            let p = s.secondary;
            find_desired_chroma_by_tone(p.hue, p.chroma, initial_tone, !s.is_dark)
        }
    },
    |_| curve(1.0, 1.0, 3.0, 4.5),
    |_| nearer(Role::SecondaryContainer, Role::Secondary),
);
pub(crate) const ON_SECONDARY_CONTAINER: ColorDef = on(
    "on_secondary_container",
    |s| s.secondary,
    |s| {
        if mono(s) {
            pick(s, 90.0, 10.0)
        } else if !fidelity(s) {
            pick(s, 90.0, 30.0)
        } else {
            foreground_tone(Role::SecondaryContainer.raw_tone(s), 4.5)
        }
    },
    |_| Some(Role::SecondaryContainer),
    |_| curve(3.0, 4.5, 7.0, 11.0),
);
pub(crate) const TERTIARY: ColorDef = accent(
    "tertiary",
    |s| s.tertiary,
    |s| {
        if mono(s) {
            pick(s, 90.0, 25.0)
        } else {
            pick(s, 80.0, 40.0)
        }
    },
    |_| curve(3.0, 4.5, 7.0, 7.0),
    |_| nearer(Role::TertiaryContainer, Role::Tertiary),
);
pub(crate) const ON_TERTIARY: ColorDef = on(
    "on_tertiary",
    |s| s.tertiary,
    |s| {
        if mono(s) {
            pick(s, 10.0, 90.0)
        } else {
            pick(s, 20.0, 100.0)
        }
    },
    |_| Some(Role::Tertiary),
    |_| curve(4.5, 7.0, 11.0, 21.0),
);
pub(crate) const TERTIARY_CONTAINER: ColorDef = accent(
    "tertiary_container",
    |s| s.tertiary,
    |s| {
        if mono(s) {
            pick(s, 60.0, 49.0)
        } else if !fidelity(s) {
            pick(s, 30.0, 90.0)
        } else {
            fix_if_disliked(s.tertiary.get_hct(s.source.tone)).tone
        }
    },
    |_| curve(1.0, 1.0, 3.0, 4.5),
    |_| nearer(Role::TertiaryContainer, Role::Tertiary),
);
pub(crate) const ON_TERTIARY_CONTAINER: ColorDef = on(
    "on_tertiary_container",
    |s| s.tertiary,
    |s| {
        if mono(s) {
            pick(s, 0.0, 100.0)
        } else if !fidelity(s) {
            pick(s, 90.0, 30.0)
        } else {
            foreground_tone(Role::TertiaryContainer.raw_tone(s), 4.5)
        }
    },
    |_| Some(Role::TertiaryContainer),
    |_| curve(3.0, 4.5, 7.0, 11.0),
);
pub(crate) const ERROR: ColorDef = accent(
    "error",
    |s| s.error,
    |s| pick(s, 80.0, 40.0),
    |_| curve(3.0, 4.5, 7.0, 7.0),
    |_| nearer(Role::ErrorContainer, Role::Error),
);
pub(crate) const ON_ERROR: ColorDef = on(
    "on_error",
    |s| s.error,
    |s| pick(s, 20.0, 100.0),
    |_| Some(Role::Error),
    |_| curve(4.5, 7.0, 11.0, 21.0),
);
pub(crate) const ERROR_CONTAINER: ColorDef = accent(
    "error_container",
    |s| s.error,
    |s| pick(s, 30.0, 90.0),
    |_| curve(1.0, 1.0, 3.0, 4.5),
    |_| nearer(Role::ErrorContainer, Role::Error),
);
pub(crate) const ON_ERROR_CONTAINER: ColorDef = on(
    "on_error_container",
    |s| s.error,
    |s| {
        if mono(s) {
            pick(s, 90.0, 10.0)
        } else {
            pick(s, 90.0, 30.0)
        }
    },
    |_| Some(Role::ErrorContainer),
    |_| curve(3.0, 4.5, 7.0, 11.0),
);

/// A fixed colour or its dim, which ignore the scheme's darkness.
const fn fixed(
    name: &'static str,
    palette: fn(&DynScheme) -> crate::tonal::TonalPalette,
    tone: fn(&DynScheme) -> f64,
    tone_delta_pair: fn(&DynScheme) -> Option<ToneDeltaPair>,
) -> ColorDef {
    accent(
        name,
        palette,
        tone,
        |_| curve(1.0, 1.0, 3.0, 4.5),
        tone_delta_pair,
    )
}

/// A foreground over a fixed dim, with the fixed colour as its second
/// background.
const fn on_fixed(
    name: &'static str,
    palette: fn(&DynScheme) -> crate::tonal::TonalPalette,
    tone: fn(&DynScheme) -> f64,
    background: fn(&DynScheme) -> Option<Role>,
    second_background: fn(&DynScheme) -> Option<Role>,
    contrast_curve: fn(&DynScheme) -> Option<ContrastCurve>,
) -> ColorDef {
    ColorDef {
        second_background: Some(second_background),
        ..on(name, palette, tone, background, contrast_curve)
    }
}

pub(crate) const PRIMARY_FIXED: ColorDef = fixed(
    "primary_fixed",
    |s| s.primary,
    |s| if mono(s) { 40.0 } else { 90.0 },
    |_| lighter(Role::PrimaryFixed, Role::PrimaryFixedDim),
);
pub(crate) const PRIMARY_FIXED_DIM: ColorDef = fixed(
    "primary_fixed_dim",
    |s| s.primary,
    |s| if mono(s) { 30.0 } else { 80.0 },
    |_| lighter(Role::PrimaryFixed, Role::PrimaryFixedDim),
);
pub(crate) const ON_PRIMARY_FIXED: ColorDef = on_fixed(
    "on_primary_fixed",
    |s| s.primary,
    |s| if mono(s) { 100.0 } else { 10.0 },
    |_| Some(Role::PrimaryFixedDim),
    |_| Some(Role::PrimaryFixed),
    |_| curve(4.5, 7.0, 11.0, 21.0),
);
pub(crate) const ON_PRIMARY_FIXED_VARIANT: ColorDef = on_fixed(
    "on_primary_fixed_variant",
    |s| s.primary,
    |s| if mono(s) { 90.0 } else { 30.0 },
    |_| Some(Role::PrimaryFixedDim),
    |_| Some(Role::PrimaryFixed),
    |_| curve(3.0, 4.5, 7.0, 11.0),
);
pub(crate) const SECONDARY_FIXED: ColorDef = fixed(
    "secondary_fixed",
    |s| s.secondary,
    |s| if mono(s) { 80.0 } else { 90.0 },
    |_| lighter(Role::SecondaryFixed, Role::SecondaryFixedDim),
);
pub(crate) const SECONDARY_FIXED_DIM: ColorDef = fixed(
    "secondary_fixed_dim",
    |s| s.secondary,
    |s| if mono(s) { 70.0 } else { 80.0 },
    |_| lighter(Role::SecondaryFixed, Role::SecondaryFixedDim),
);
pub(crate) const ON_SECONDARY_FIXED: ColorDef = on_fixed(
    "on_secondary_fixed",
    |s| s.secondary,
    |_| 10.0,
    |_| Some(Role::SecondaryFixedDim),
    |_| Some(Role::SecondaryFixed),
    |_| curve(4.5, 7.0, 11.0, 21.0),
);
pub(crate) const ON_SECONDARY_FIXED_VARIANT: ColorDef = on_fixed(
    "on_secondary_fixed_variant",
    |s| s.secondary,
    |s| if mono(s) { 25.0 } else { 30.0 },
    |_| Some(Role::SecondaryFixedDim),
    |_| Some(Role::SecondaryFixed),
    |_| curve(3.0, 4.5, 7.0, 11.0),
);
pub(crate) const TERTIARY_FIXED: ColorDef = fixed(
    "tertiary_fixed",
    |s| s.tertiary,
    |s| if mono(s) { 40.0 } else { 90.0 },
    |_| lighter(Role::TertiaryFixed, Role::TertiaryFixedDim),
);
pub(crate) const TERTIARY_FIXED_DIM: ColorDef = fixed(
    "tertiary_fixed_dim",
    |s| s.tertiary,
    |s| if mono(s) { 30.0 } else { 80.0 },
    |_| lighter(Role::TertiaryFixed, Role::TertiaryFixedDim),
);
pub(crate) const ON_TERTIARY_FIXED: ColorDef = on_fixed(
    "on_tertiary_fixed",
    |s| s.tertiary,
    |s| if mono(s) { 100.0 } else { 10.0 },
    |_| Some(Role::TertiaryFixedDim),
    |_| Some(Role::TertiaryFixed),
    |_| curve(4.5, 7.0, 11.0, 21.0),
);
pub(crate) const ON_TERTIARY_FIXED_VARIANT: ColorDef = on_fixed(
    "on_tertiary_fixed_variant",
    |s| s.tertiary,
    |s| if mono(s) { 90.0 } else { 30.0 },
    |_| Some(Role::TertiaryFixedDim),
    |_| Some(Role::TertiaryFixed),
    |_| curve(3.0, 4.5, 7.0, 11.0),
);
