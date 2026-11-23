// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// MaterialKolor's `dynamicColorScheme(primary = seed, ...)`: the `SchemeX`
// defaults for the style, rebuilt around the seed's own primary palette,
// then read role by role as `toColorScheme` does.

use crate::dynamic::{DynScheme, Memo, Role};
use crate::hct::Hct;
use crate::spec2021;
use crate::spec2025::{self, Palettes};
use crate::tonal::TonalPalette;
use crate::{Spec, Style};

/// `DynamicScheme.maybeFallbackSpecVersion`: only four styles have a 2025
/// definition; the rest always use 2021.
fn effective_spec(spec: Spec, style: Style) -> Spec {
    match style {
        Style::Expressive | Style::Vibrant | Style::TonalSpot | Style::Neutral => spec,
        _ => Spec::Spec2021,
    }
}

/// The non-primary palettes of `SchemeX(sourceHct, isDark, contrast, spec)`.
fn default_palettes(style: Style, source: &Hct, is_dark: bool, spec: Spec) -> Palettes {
    let modern = match spec {
        Spec::Spec2025 => spec2025::palettes(style, source, is_dark),
        Spec::Spec2021 => None,
    };
    modern.unwrap_or_else(|| Palettes {
        secondary: spec2021::secondary_palette(style, source),
        tertiary: spec2021::tertiary_palette(style, source),
        neutral: spec2021::neutral_palette(style, source),
        neutral_variant: spec2021::neutral_variant_palette(style, source),
        // `getErrorPalette` is null; `DynamicScheme` substitutes this.
        error: TonalPalette::from_hue_and_chroma(25.0, 84.0),
    })
}

/// The `DynamicScheme` MaterialKolor builds for `primary = seed`.
pub(crate) fn dynamic_scheme(
    seed: u32,
    is_dark: bool,
    style: Style,
    contrast_level: f64,
    spec: Spec,
) -> DynScheme {
    let source = Hct::from_int(seed);
    let spec = effective_spec(spec, style);
    let palettes = default_palettes(style, &source, is_dark, spec);
    DynScheme {
        source,
        variant: style,
        is_dark,
        contrast_level,
        primary: TonalPalette::from_int(seed),
        secondary: palettes.secondary,
        tertiary: palettes.tertiary,
        neutral: palettes.neutral,
        neutral_variant: palettes.neutral_variant,
        error: palettes.error,
        spec,
        memo: Memo::new(),
    }
}

/// One role as `MaterialKolors` reads it: AMOLED dark schemes force pure
/// black backgrounds and pure white text on them.
pub(crate) fn argb(s: &DynScheme, amoled: bool, role: Role) -> u32 {
    if amoled && s.is_dark {
        match role {
            Role::Background | Role::Surface => return 0xFF00_0000,
            Role::OnBackground | Role::OnSurface => return 0xFFFF_FFFF,
            _ => {}
        }
    }
    role.argb(s)
}
