// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// The accent, container, error and fixed colours `ColorSpec2025` adds for
// PHONE; the surfaces and shared helpers are in the parent module.

use super::*;

/// The `RELATIVE_LIGHTER`, `FARTHER` pair of a container and its accent.
const fn farther(role_a: Role, role_b: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair {
        role_a,
        role_b,
        delta: 5.0,
        polarity: Polarity::RelativeLighter,
        stay_together: true,
        constraint: Constraint::Farther,
    })
}

/// The `DARKER`, `EXACT` pair of a fixed dim and its fixed colour.
const fn exact(role_a: Role, role_b: Role) -> Option<ToneDeltaPair> {
    Some(ToneDeltaPair {
        role_a,
        role_b,
        delta: 5.0,
        polarity: Polarity::Darker,
        stay_together: true,
        constraint: Constraint::Exact,
    })
}

/// An accent: a background over the highest surface, paired with its
/// container.
const fn accent(
    name: &'static str,
    palette: fn(&DynScheme) -> TonalPalette,
    tone: fn(&DynScheme) -> f64,
    tone_delta_pair: fn(&DynScheme) -> Option<ToneDeltaPair>,
) -> ColorDef {
    ColorDef {
        tone: Some(tone),
        is_background: true,
        background: Some(highest),
        contrast_curve: Some(|_| cc(4.5)),
        tone_delta_pair: Some(tone_delta_pair),
        ..ColorDef::new(name, palette)
    }
}

/// A container: a background whose curve applies only at raised contrast.
const fn container(
    name: &'static str,
    palette: fn(&DynScheme) -> TonalPalette,
    tone: fn(&DynScheme) -> f64,
) -> ColorDef {
    ColorDef {
        tone: Some(tone),
        is_background: true,
        background: Some(highest),
        contrast_curve: Some(container_curve),
        ..ColorDef::new(name, palette)
    }
}

/// A foreground with the builder's default tone, taken from its background.
const fn on(
    name: &'static str,
    palette: fn(&DynScheme) -> TonalPalette,
    background: fn(&DynScheme) -> Option<Role>,
    contrast_curve: fn(&DynScheme) -> Option<ContrastCurve>,
) -> ColorDef {
    ColorDef {
        background: Some(background),
        contrast_curve: Some(contrast_curve),
        ..ColorDef::new(name, palette)
    }
}

fn cyan_or(p: &TonalPalette, other: f64) -> f64 {
    if p.key_color.is_cyan() { 88.0 } else { other }
}

pub(crate) const PRIMARY: ColorDef = accent(
    "primary",
    |s| s.primary,
    |s| {
        let p = &s.primary;
        match s.variant {
            Style::Neutral => pick(s, 80.0, 40.0),
            Style::TonalSpot => {
                if s.is_dark {
                    80.0
                } else {
                    t_max_c(s, p, 0.0, 100.0)
                }
            }
            Style::Expressive => {
                let upper = if p.key_color.is_yellow() {
                    25.0
                } else {
                    cyan_or(p, 98.0)
                };
                t_max_c(s, p, 0.0, upper)
            }
            _ => t_max_c(s, p, 0.0, cyan_or(p, 98.0)),
        }
    },
    |_| farther(Role::PrimaryContainer, Role::Primary),
);
pub(crate) const SURFACE_TINT: ColorDef = ColorDef {
    name: "surface_tint",
    ..PRIMARY
};
pub(crate) const ON_PRIMARY: ColorDef = on(
    "on_primary",
    |s| s.primary,
    |_| Some(Role::Primary),
    |_| cc(6.0),
);
pub(crate) const PRIMARY_CONTAINER: ColorDef = container(
    "primary_container",
    |s| s.primary,
    |s| {
        let p = &s.primary;
        match (s.variant, s.is_dark) {
            (Style::Neutral, _) => pick(s, 30.0, 90.0),
            (Style::TonalSpot, true) => t_min_c(s, p, 35.0, 93.0),
            (Style::TonalSpot, false) => t_max_c(s, p, 0.0, 90.0),
            (Style::Expressive, true) => t_max_c(s, p, 30.0, 93.0),
            (Style::Expressive, false) => t_max_c(s, p, 78.0, cyan_or(p, 90.0)),
            (_, true) => t_min_c(s, p, 66.0, 93.0),
            (_, false) => t_max_c(s, p, 66.0, cyan_or(p, 93.0)),
        }
    },
);
pub(crate) const ON_PRIMARY_CONTAINER: ColorDef = on(
    "on_primary_container",
    |s| s.primary,
    |_| Some(Role::PrimaryContainer),
    |_| cc(6.0),
);
pub(crate) const INVERSE_PRIMARY: ColorDef = ColorDef {
    tone: Some(|s| t_max_c(s, &s.primary, 0.0, 100.0)),
    ..on(
        "inverse_primary",
        |s| s.primary,
        |_| Some(Role::InverseSurface),
        |_| cc(6.0),
    )
};
pub(crate) const SECONDARY: ColorDef = accent(
    "secondary",
    |s| s.secondary,
    |s| {
        let p = &s.secondary;
        match s.variant {
            Style::Neutral => {
                if s.is_dark {
                    t_min_c(s, p, 0.0, 98.0)
                } else {
                    t_max_c(s, p, 0.0, 100.0)
                }
            }
            Style::Vibrant => t_max_c(s, p, 0.0, pick(s, 90.0, 98.0)),
            _ => {
                if s.is_dark {
                    80.0
                } else {
                    t_max_c(s, p, 0.0, 100.0)
                }
            }
        }
    },
    |_| farther(Role::SecondaryContainer, Role::Secondary),
);
pub(crate) const ON_SECONDARY: ColorDef = on(
    "on_secondary",
    |s| s.secondary,
    |_| Some(Role::Secondary),
    |_| cc(6.0),
);
pub(crate) const SECONDARY_CONTAINER: ColorDef = container(
    "secondary_container",
    |s| s.secondary,
    |s| {
        let p = &s.secondary;
        match (s.variant, s.is_dark) {
            (Style::Vibrant, true) => t_min_c(s, p, 30.0, 40.0),
            (Style::Vibrant, false) => t_max_c(s, p, 84.0, 90.0),
            (Style::Expressive, true) => 15.0,
            (Style::Expressive, false) => t_max_c(s, p, 90.0, 95.0),
            _ => pick(s, 25.0, 90.0),
        }
    },
);
pub(crate) const ON_SECONDARY_CONTAINER: ColorDef = on(
    "on_secondary_container",
    |s| s.secondary,
    |_| Some(Role::SecondaryContainer),
    |_| cc(6.0),
);
pub(crate) const TERTIARY: ColorDef = accent(
    "tertiary",
    |s| s.tertiary,
    |s| {
        let p = &s.tertiary;
        if s.variant == Style::Expressive || s.variant == Style::Vibrant {
            t_max_c(s, p, 0.0, cyan_or(p, pick(s, 98.0, 100.0)))
        } else if s.is_dark {
            t_max_c(s, p, 0.0, 98.0)
        } else {
            t_max_c(s, p, 0.0, 100.0)
        }
    },
    |_| farther(Role::TertiaryContainer, Role::Tertiary),
);
pub(crate) const ON_TERTIARY: ColorDef = on(
    "on_tertiary",
    |s| s.tertiary,
    |_| Some(Role::Tertiary),
    |_| cc(6.0),
);
pub(crate) const TERTIARY_CONTAINER: ColorDef = container(
    "tertiary_container",
    |s| s.tertiary,
    |s| {
        let p = &s.tertiary;
        match (s.variant, s.is_dark) {
            (Style::Neutral, true) => t_max_c(s, p, 0.0, 93.0),
            (Style::Neutral, false) => t_max_c(s, p, 0.0, 96.0),
            (Style::TonalSpot, _) => t_max_c(s, p, 0.0, pick(s, 93.0, 100.0)),
            (Style::Expressive, _) => t_max_c(s, p, 75.0, cyan_or(p, pick(s, 93.0, 100.0))),
            (_, true) => t_max_c(s, p, 0.0, 93.0),
            (_, false) => t_max_c(s, p, 72.0, 100.0),
        }
    },
);
pub(crate) const ON_TERTIARY_CONTAINER: ColorDef = on(
    "on_tertiary_container",
    |s| s.tertiary,
    |_| Some(Role::TertiaryContainer),
    |_| cc(6.0),
);
pub(crate) const ERROR: ColorDef = accent(
    "error",
    |s| s.error,
    |s| {
        if s.is_dark {
            t_min_c(s, &s.error, 0.0, 98.0)
        } else {
            t_max_c(s, &s.error, 0.0, 100.0)
        }
    },
    |_| farther(Role::ErrorContainer, Role::Error),
);
pub(crate) const ON_ERROR: ColorDef =
    on("on_error", |s| s.error, |_| Some(Role::Error), |_| cc(6.0));
pub(crate) const ERROR_CONTAINER: ColorDef = container(
    "error_container",
    |s| s.error,
    |s| {
        if s.is_dark {
            t_min_c(s, &s.error, 30.0, 93.0)
        } else {
            t_max_c(s, &s.error, 0.0, 90.0)
        }
    },
);
pub(crate) const ON_ERROR_CONTAINER: ColorDef = on(
    "on_error_container",
    |s| s.error,
    |_| Some(Role::ErrorContainer),
    |_| cc(4.5),
);

/// A fixed dim: a background with no contrast target, `delta` below its
/// fixed colour.
const fn fixed_dim(
    name: &'static str,
    palette: fn(&DynScheme) -> TonalPalette,
    tone: fn(&DynScheme) -> f64,
    tone_delta_pair: fn(&DynScheme) -> Option<ToneDeltaPair>,
) -> ColorDef {
    ColorDef {
        tone: Some(tone),
        is_background: true,
        tone_delta_pair: Some(tone_delta_pair),
        ..ColorDef::new(name, palette)
    }
}

pub(crate) const PRIMARY_FIXED: ColorDef = container(
    "primary_fixed",
    |s| s.primary,
    |s| Role::PrimaryContainer.tone(&s.with(false, 0.0)),
);
pub(crate) const PRIMARY_FIXED_DIM: ColorDef = fixed_dim(
    "primary_fixed_dim",
    |s| s.primary,
    |s| Role::PrimaryFixed.tone(s),
    |_| exact(Role::PrimaryFixedDim, Role::PrimaryFixed),
);
pub(crate) const ON_PRIMARY_FIXED: ColorDef = on(
    "on_primary_fixed",
    |s| s.primary,
    |_| Some(Role::PrimaryFixedDim),
    |_| cc(7.0),
);
pub(crate) const ON_PRIMARY_FIXED_VARIANT: ColorDef = on(
    "on_primary_fixed_variant",
    |s| s.primary,
    |_| Some(Role::PrimaryFixedDim),
    |_| cc(4.5),
);
pub(crate) const SECONDARY_FIXED: ColorDef = container(
    "secondary_fixed",
    |s| s.secondary,
    |s| Role::SecondaryContainer.tone(&s.with(false, 0.0)),
);
pub(crate) const SECONDARY_FIXED_DIM: ColorDef = fixed_dim(
    "secondary_fixed_dim",
    |s| s.secondary,
    |s| Role::SecondaryFixed.tone(s),
    |_| exact(Role::SecondaryFixedDim, Role::SecondaryFixed),
);
pub(crate) const ON_SECONDARY_FIXED: ColorDef = on(
    "on_secondary_fixed",
    |s| s.secondary,
    |_| Some(Role::SecondaryFixedDim),
    |_| cc(7.0),
);
pub(crate) const ON_SECONDARY_FIXED_VARIANT: ColorDef = on(
    "on_secondary_fixed_variant",
    |s| s.secondary,
    |_| Some(Role::SecondaryFixedDim),
    |_| cc(4.5),
);
pub(crate) const TERTIARY_FIXED: ColorDef = container(
    "tertiary_fixed",
    |s| s.tertiary,
    |s| Role::TertiaryContainer.tone(&s.with(false, 0.0)),
);
pub(crate) const TERTIARY_FIXED_DIM: ColorDef = fixed_dim(
    "tertiary_fixed_dim",
    |s| s.tertiary,
    |s| Role::TertiaryFixed.tone(s),
    |_| exact(Role::TertiaryFixedDim, Role::TertiaryFixed),
);
pub(crate) const ON_TERTIARY_FIXED: ColorDef = on(
    "on_tertiary_fixed",
    |s| s.tertiary,
    |_| Some(Role::TertiaryFixedDim),
    |_| cc(7.0),
);
pub(crate) const ON_TERTIARY_FIXED_VARIANT: ColorDef = on(
    "on_tertiary_fixed_variant",
    |s| s.tertiary,
    |_| Some(Role::TertiaryFixedDim),
    |_| cc(4.5),
);
