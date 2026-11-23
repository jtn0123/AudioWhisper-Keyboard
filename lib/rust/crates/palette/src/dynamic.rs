// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// DynamicScheme, DynamicColor and ToneDeltaPair, reshaped for Rust.
//
// In Kotlin every colour the app reads comes from a `ColorSpec2025`
// instance, which builds each role as the 2021 colour "extended" with a
// 2025 one: every lambda of the result picks the 2025 or 2021 definition by
// the scheme's spec version, and every lambda is non-null even when the
// chosen definition has none. Colours a lambda refers to (a background, a
// tone-delta partner) are those extended colours too. Here a role carries
// both definitions as tables of function pointers; `Role::def` makes the
// choice, and `Role::extended` records whether the lambdas are the
// always-non-null kind, which `ColorSpec2021.getTone` can observe. Only the
// PHONE platform is modelled; every WATCH branch of the Kotlin is dropped.

use std::cell::{Cell, RefCell};

use crate::Spec;
use crate::Style;
use crate::contrast::ContrastCurve;
use crate::hct::Hct;
use crate::spec2021;
use crate::spec2025;
use crate::tonal::TonalPalette;

/// Answers already computed for one scheme. Every solver here is a pure
/// function of the scheme, so remembering them changes no result; the
/// Kotlin recomputes them (its `hctCache` lives on `DynamicColor` objects
/// that `MaterialDynamicColors` rebuilds on every call).
#[derive(Debug)]
pub(crate) struct Memo {
    tone: [Cell<Option<f64>>; 48],
    raw_tone: [Cell<Option<f64>>; 48],
    /// `findBestToneForChroma` keyed by the bits of its arguments.
    pub(crate) best_tone: RefCell<Vec<([u64; 3], bool, f64)>>,
}

impl Memo {
    pub(crate) fn new() -> Memo {
        Memo {
            tone: std::array::from_fn(|_| Cell::new(None)),
            raw_tone: std::array::from_fn(|_| Cell::new(None)),
            best_tone: RefCell::new(Vec::new()),
        }
    }
}

/// Looks `slot` up, or fills it with `compute`.
fn memoised(slot: &Cell<Option<f64>>, compute: impl FnOnce() -> f64) -> f64 {
    if let Some(value) = slot.get() {
        return value;
    }
    let value = compute();
    slot.set(Some(value));
    value
}

/// `DynamicScheme` with `Platform.PHONE`.
#[derive(Debug)]
pub(crate) struct DynScheme {
    pub(crate) source: Hct,
    pub(crate) variant: Style,
    pub(crate) is_dark: bool,
    pub(crate) contrast_level: f64,
    pub(crate) primary: TonalPalette,
    pub(crate) secondary: TonalPalette,
    pub(crate) tertiary: TonalPalette,
    pub(crate) neutral: TonalPalette,
    pub(crate) neutral_variant: TonalPalette,
    pub(crate) error: TonalPalette,
    /// The spec after `maybeFallbackSpecVersion`.
    pub(crate) spec: Spec,
    pub(crate) memo: Memo,
}

impl DynScheme {
    /// `DynamicScheme.from(other, isDark, contrastLevel)`.
    pub(crate) fn with(&self, is_dark: bool, contrast_level: f64) -> DynScheme {
        DynScheme {
            source: self.source,
            variant: self.variant,
            is_dark,
            contrast_level,
            primary: self.primary,
            secondary: self.secondary,
            tertiary: self.tertiary,
            neutral: self.neutral,
            neutral_variant: self.neutral_variant,
            error: self.error,
            spec: self.spec,
            memo: Memo::new(),
        }
    }
}

/// `TonePolarity`. No PHONE colour uses `RELATIVE_DARKER`, so it is left
/// out, along with the solver branches that test for it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Polarity {
    Darker,
    Lighter,
    RelativeLighter,
    Nearer,
}

/// `ToneDeltaPair.DeltaConstraint`. No PHONE colour uses `NEARER`, so it is
/// left out, along with its solver branch.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Constraint {
    Exact,
    Farther,
}

/// `ToneDeltaPair`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ToneDeltaPair {
    pub(crate) role_a: Role,
    pub(crate) role_b: Role,
    pub(crate) delta: f64,
    pub(crate) polarity: Polarity,
    pub(crate) stay_together: bool,
    pub(crate) constraint: Constraint,
}

/// A scheme-dependent value, as Kotlin's `(DynamicScheme) -> T` lambdas.
pub(crate) type Lambda<T> = fn(&DynScheme) -> T;

/// One definition of a colour: the fields of a Kotlin `DynamicColor`.
#[derive(Clone, Copy)]
pub(crate) struct ColorDef {
    pub(crate) name: &'static str,
    pub(crate) palette: Lambda<TonalPalette>,
    /// `None` is the builder's default, `getInitialToneFromBackground`.
    pub(crate) tone: Option<Lambda<f64>>,
    pub(crate) is_background: bool,
    pub(crate) chroma_multiplier: Option<Lambda<f64>>,
    pub(crate) background: Option<Lambda<Option<Role>>>,
    pub(crate) second_background: Option<Lambda<Option<Role>>>,
    pub(crate) contrast_curve: Option<Lambda<Option<ContrastCurve>>>,
    pub(crate) tone_delta_pair: Option<Lambda<Option<ToneDeltaPair>>>,
}

impl ColorDef {
    /// A definition with only the builder's defaults filled in.
    pub(crate) const fn new(name: &'static str, palette: Lambda<TonalPalette>) -> ColorDef {
        ColorDef {
            name,
            palette,
            tone: None,
            is_background: false,
            chroma_multiplier: None,
            background: None,
            second_background: None,
            contrast_curve: None,
            tone_delta_pair: None,
        }
    }
}

/// The colour roles a `ColorScheme` reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Role {
    Background,
    Error,
    ErrorContainer,
    InverseOnSurface,
    InversePrimary,
    InverseSurface,
    OnBackground,
    OnError,
    OnErrorContainer,
    OnPrimary,
    OnPrimaryContainer,
    OnSecondary,
    OnSecondaryContainer,
    OnSurface,
    OnSurfaceVariant,
    OnTertiary,
    OnTertiaryContainer,
    Outline,
    OutlineVariant,
    Primary,
    PrimaryContainer,
    Scrim,
    Secondary,
    SecondaryContainer,
    Surface,
    SurfaceTint,
    SurfaceBright,
    SurfaceDim,
    SurfaceContainer,
    SurfaceContainerHigh,
    SurfaceContainerHighest,
    SurfaceContainerLow,
    SurfaceContainerLowest,
    SurfaceVariant,
    Tertiary,
    TertiaryContainer,
    PrimaryFixed,
    PrimaryFixedDim,
    OnPrimaryFixed,
    OnPrimaryFixedVariant,
    SecondaryFixed,
    SecondaryFixedDim,
    OnSecondaryFixed,
    OnSecondaryFixedVariant,
    TertiaryFixed,
    TertiaryFixedDim,
    OnTertiaryFixed,
    OnTertiaryFixedVariant,
}

impl Role {
    /// Whether `ColorSpec2025` overrides this role, making it an extended
    /// colour. `scrim` is the only role here it leaves as plain 2021.
    pub(crate) fn extended(self) -> bool {
        self != Role::Scrim
    }

    /// The definition this role uses for a scheme of the given spec.
    pub(crate) fn def(self, spec: Spec) -> &'static ColorDef {
        match spec {
            Spec::Spec2025 if self.extended() => spec2025::def(self),
            _ => spec2021::def(self),
        }
    }

    /// `color.name`, which for an extended colour is the 2021 name.
    pub(crate) fn name(self) -> &'static str {
        spec2021::def(self).name
    }

    /// `color.isBackground`, which for an extended colour is the 2021 flag.
    pub(crate) fn is_background(self) -> bool {
        spec2021::def(self).is_background
    }

    /// `color.background != null`: always true for an extended colour.
    pub(crate) fn has_background(self, spec: Spec) -> bool {
        self.extended() || self.def(spec).background.is_some()
    }

    /// `color.contrastCurve != null`: always true for an extended colour.
    pub(crate) fn has_contrast_curve(self, spec: Spec) -> bool {
        self.extended() || self.def(spec).contrast_curve.is_some()
    }

    /// `color.background?.invoke(s)`.
    pub(crate) fn background(self, s: &DynScheme) -> Option<Role> {
        self.def(s.spec)
            .background
            .and_then(|background| background(s))
    }

    /// `color.secondBackground?.invoke(s)`.
    pub(crate) fn second_background(self, s: &DynScheme) -> Option<Role> {
        self.def(s.spec)
            .second_background
            .and_then(|background| background(s))
    }

    /// `color.contrastCurve?.invoke(s)`.
    pub(crate) fn contrast_curve(self, s: &DynScheme) -> Option<ContrastCurve> {
        self.def(s.spec).contrast_curve.and_then(|curve| curve(s))
    }

    /// `color.toneDeltaPair?.invoke(s)`.
    pub(crate) fn tone_delta_pair(self, s: &DynScheme) -> Option<ToneDeltaPair> {
        self.def(s.spec).tone_delta_pair.and_then(|pair| pair(s))
    }

    /// `color.tone(s)`: the raw tone lambda, before any contrast solving.
    pub(crate) fn raw_tone(self, s: &DynScheme) -> f64 {
        memoised(&s.memo.raw_tone[self as usize], || {
            match self.def(s.spec).tone {
                Some(tone) => tone(s),
                None => self.background(s).map_or(50.0, |bg| bg.tone(s)),
            }
        })
    }

    /// `DynamicColor.getTone(scheme)`.
    pub(crate) fn tone(self, s: &DynScheme) -> f64 {
        memoised(&s.memo.tone[self as usize], || match s.spec {
            Spec::Spec2021 => spec2021::get_tone(s, self),
            Spec::Spec2025 => spec2025::get_tone(s, self),
        })
    }

    /// `DynamicColor.getArgb(scheme)`; no role here has an opacity.
    pub(crate) fn argb(self, s: &DynScheme) -> u32 {
        match s.spec {
            Spec::Spec2021 => {
                let tone = self.tone(s);
                (self.def(s.spec).palette)(s).argb_at(tone)
            }
            Spec::Spec2025 => spec2025::get_argb(s, self),
        }
    }
}
