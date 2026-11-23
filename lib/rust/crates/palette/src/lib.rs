// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).

//! Material You colour schemes, bit-exact with MaterialKolor 5.0.1.
//!
//! [`scheme`] reproduces MaterialKolor's
//! `dynamicColorScheme(primary = seed, isDark, isAmoled, style, contrastLevel, specVersion)`
//! on the PHONE platform, as computed on a JVM: every role of the returned
//! [`Scheme`] is the ARGB word `toColorScheme` would have produced.
//!
//! The arithmetic follows the Kotlin expression by expression, and the
//! `java.lang.Math` functions it relies on are reimplemented so that results
//! match to the last bit: fdlibm 5.3 for `pow`, `cbrt`, `exp`, `hypot` and
//! `atan2`, and Android's libm (fdlibm as FreeBSD revised it, built without
//! fused operations) for `sin` and `cos`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod color;
mod contrast;
mod dynamic;
mod hct;
mod jmath;
mod scheme;
mod spec2021;
mod spec2021_colors;
mod spec2025;
mod spec2025_colors;
mod temperature;
mod tonal;

use dynamic::Role;

/// MaterialKolor's `PaletteStyle`, in declaration order.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Style {
    /// `TonalSpot`: the Material default, a calm primary with muted accents.
    TonalSpot,
    /// `Neutral`: close to greyscale with a hint of the seed.
    Neutral,
    /// `Vibrant`: maximum chroma for the primary.
    Vibrant,
    /// `Expressive`: hues rotated away from the seed.
    Expressive,
    /// `Rainbow`: a chromatic primary over greyscale neutrals.
    Rainbow,
    /// `FruitSalad`: a playful rotation of the seed's hue.
    FruitSalad,
    /// `Monochrome`: pure greyscale.
    Monochrome,
    /// `Fidelity`: keeps the seed's own chroma in the primary container.
    Fidelity,
    /// `Content`: like `Fidelity`, with an analogous tertiary.
    Content,
}

impl Style {
    /// Every style, in `PaletteStyle` order.
    pub const ALL: [Style; 9] = [
        Style::TonalSpot,
        Style::Neutral,
        Style::Vibrant,
        Style::Expressive,
        Style::Rainbow,
        Style::FruitSalad,
        Style::Monochrome,
        Style::Fidelity,
        Style::Content,
    ];

    /// The Kotlin enum name, such as `"TonalSpot"`.
    pub fn name(self) -> &'static str {
        match self {
            Style::TonalSpot => "TonalSpot",
            Style::Neutral => "Neutral",
            Style::Vibrant => "Vibrant",
            Style::Expressive => "Expressive",
            Style::Rainbow => "Rainbow",
            Style::FruitSalad => "FruitSalad",
            Style::Monochrome => "Monochrome",
            Style::Fidelity => "Fidelity",
            Style::Content => "Content",
        }
    }

    /// The style with the given Kotlin enum name, if any.
    pub fn from_name(name: &str) -> Option<Style> {
        Style::ALL.into_iter().find(|style| style.name() == name)
    }
}

/// `ColorSpec.SpecVersion`: which revision of the Material colour rules
/// to apply. Only `TonalSpot`, `Neutral`, `Vibrant` and `Expressive` have
/// 2025 rules; every other style is computed with 2021 rules regardless.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Spec {
    /// `SPEC_2021`.
    Spec2021,
    /// `SPEC_2025`.
    Spec2025,
}

impl Spec {
    /// Both versions.
    pub const ALL: [Spec; 2] = [Spec::Spec2021, Spec::Spec2025];

    /// The Kotlin enum name, `"SPEC_2021"` or `"SPEC_2025"`.
    pub fn name(self) -> &'static str {
        match self {
            Spec::Spec2021 => "SPEC_2021",
            Spec::Spec2025 => "SPEC_2025",
        }
    }

    /// The version with the given Kotlin enum name, if any.
    pub fn from_name(name: &str) -> Option<Spec> {
        Spec::ALL.into_iter().find(|spec| spec.name() == name)
    }
}

/// The level of MaterialKolor's `Contrast` entry with the given name
/// (`Default` 0, `Medium` 0.5, `High` 1, `Reduced` -1).
pub fn contrast_level(name: &str) -> Option<f64> {
    match name {
        "Default" => Some(0.0),
        "Medium" => Some(0.5),
        "High" => Some(1.0),
        "Reduced" => Some(-1.0),
        _ => None,
    }
}

macro_rules! scheme_struct {
    ($($field:ident = $role:ident / $kotlin:literal),* $(,)?) => {
        /// A Compose `ColorScheme` as MaterialKolor fills it: every role as
        /// an ARGB word.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct Scheme {
            $(
                #[doc = concat!("`", $kotlin, "`.")]
                pub $field: u32,
            )*
        }

        fn fill(s: &dynamic::DynScheme, amoled: bool) -> Scheme {
            Scheme { $($field: scheme::argb(s, amoled, Role::$role),)* }
        }
    };
}

scheme_struct!(
    primary = Primary / "primary",
    on_primary = OnPrimary / "onPrimary",
    primary_container = PrimaryContainer / "primaryContainer",
    on_primary_container = OnPrimaryContainer / "onPrimaryContainer",
    inverse_primary = InversePrimary / "inversePrimary",
    secondary = Secondary / "secondary",
    on_secondary = OnSecondary / "onSecondary",
    secondary_container = SecondaryContainer / "secondaryContainer",
    on_secondary_container = OnSecondaryContainer / "onSecondaryContainer",
    tertiary = Tertiary / "tertiary",
    on_tertiary = OnTertiary / "onTertiary",
    tertiary_container = TertiaryContainer / "tertiaryContainer",
    on_tertiary_container = OnTertiaryContainer / "onTertiaryContainer",
    background = Background / "background",
    on_background = OnBackground / "onBackground",
    surface = Surface / "surface",
    on_surface = OnSurface / "onSurface",
    surface_variant = SurfaceVariant / "surfaceVariant",
    on_surface_variant = OnSurfaceVariant / "onSurfaceVariant",
    surface_tint = SurfaceTint / "surfaceTint",
    inverse_surface = InverseSurface / "inverseSurface",
    inverse_on_surface = InverseOnSurface / "inverseOnSurface",
    error = Error / "error",
    on_error = OnError / "onError",
    error_container = ErrorContainer / "errorContainer",
    on_error_container = OnErrorContainer / "onErrorContainer",
    outline = Outline / "outline",
    outline_variant = OutlineVariant / "outlineVariant",
    scrim = Scrim / "scrim",
    surface_bright = SurfaceBright / "surfaceBright",
    surface_dim = SurfaceDim / "surfaceDim",
    surface_container = SurfaceContainer / "surfaceContainer",
    surface_container_high = SurfaceContainerHigh / "surfaceContainerHigh",
    surface_container_highest = SurfaceContainerHighest / "surfaceContainerHighest",
    surface_container_low = SurfaceContainerLow / "surfaceContainerLow",
    surface_container_lowest = SurfaceContainerLowest / "surfaceContainerLowest",
    primary_fixed = PrimaryFixed / "primaryFixed",
    primary_fixed_dim = PrimaryFixedDim / "primaryFixedDim",
    on_primary_fixed = OnPrimaryFixed / "onPrimaryFixed",
    on_primary_fixed_variant = OnPrimaryFixedVariant / "onPrimaryFixedVariant",
    secondary_fixed = SecondaryFixed / "secondaryFixed",
    secondary_fixed_dim = SecondaryFixedDim / "secondaryFixedDim",
    on_secondary_fixed = OnSecondaryFixed / "onSecondaryFixed",
    on_secondary_fixed_variant = OnSecondaryFixedVariant / "onSecondaryFixedVariant",
    tertiary_fixed = TertiaryFixed / "tertiaryFixed",
    tertiary_fixed_dim = TertiaryFixedDim / "tertiaryFixedDim",
    on_tertiary_fixed = OnTertiaryFixed / "onTertiaryFixed",
    on_tertiary_fixed_variant = OnTertiaryFixedVariant / "onTertiaryFixedVariant",
);

/// The Kotlin names of the 35 roles [`roles`] returns, in its order.
pub const NAMES: [&str; 35] = [
    "primary",
    "onPrimary",
    "primaryContainer",
    "onPrimaryContainer",
    "inversePrimary",
    "secondary",
    "onSecondary",
    "secondaryContainer",
    "onSecondaryContainer",
    "tertiary",
    "onTertiary",
    "tertiaryContainer",
    "onTertiaryContainer",
    "background",
    "onBackground",
    "onSurface",
    "surfaceVariant",
    "onSurfaceVariant",
    "surfaceTint",
    "inverseSurface",
    "inverseOnSurface",
    "error",
    "onError",
    "errorContainer",
    "onErrorContainer",
    "outline",
    "outlineVariant",
    "scrim",
    "surfaceBright",
    "surfaceDim",
    "surfaceContainer",
    "surfaceContainerHigh",
    "surfaceContainerHighest",
    "surfaceContainerLow",
    "surfaceContainerLowest",
];

/// The 35 roles named by [`NAMES`], in that order.
pub fn roles(s: &Scheme) -> [u32; 35] {
    [
        s.primary,
        s.on_primary,
        s.primary_container,
        s.on_primary_container,
        s.inverse_primary,
        s.secondary,
        s.on_secondary,
        s.secondary_container,
        s.on_secondary_container,
        s.tertiary,
        s.on_tertiary,
        s.tertiary_container,
        s.on_tertiary_container,
        s.background,
        s.on_background,
        s.on_surface,
        s.surface_variant,
        s.on_surface_variant,
        s.surface_tint,
        s.inverse_surface,
        s.inverse_on_surface,
        s.error,
        s.on_error,
        s.error_container,
        s.on_error_container,
        s.outline,
        s.outline_variant,
        s.scrim,
        s.surface_bright,
        s.surface_dim,
        s.surface_container,
        s.surface_container_high,
        s.surface_container_highest,
        s.surface_container_low,
        s.surface_container_lowest,
    ]
}

/// MaterialKolor's `dynamicColorScheme(primary = seed, isDark = dark,
/// isAmoled = amoled, style, contrastLevel = contrast, specVersion = spec)`
/// on the PHONE platform.
///
/// `seed` is an ARGB word; its alpha has no effect. `contrast` is
/// normally one of the [`contrast_level`] values, from -1 to 1.
pub fn scheme(
    seed: u32,
    dark: bool,
    amoled: bool,
    style: Style,
    contrast: f64,
    spec: Spec,
) -> Scheme {
    let s = scheme::dynamic_scheme(seed, dark, style, contrast, spec);
    fill(&s, amoled)
}
