// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// Java-compatible maths for the palette port.
//
// MaterialKolor's `kotlin.math` calls `java.lang.Math`. Here `pow`, `exp`,
// `cbrt`, `hypot` and `atan2` are fdlibm 5.3, as `StrictMath` and a desktop
// JVM's `Math` are, and `sin` and `cos` are Android's libm (bionic), which
// ART's `Math` calls: fdlibm as FreeBSD revised it. On ART the other five
// come from bionic too (Arm's optimized routines for `pow` and `exp`,
// FreeBSD's revisions for the rest), which can round a last bit differently;
// docs/decisions.md records what that reached on ART. Rust's `f64` methods
// call the host's libm instead, which disagrees in the last place for
// roughly one call in ten, so this module carries ports of these
// algorithms. See the per-file headers for provenance.

mod atan;
mod misc;
mod pow;
mod trig;

pub(crate) use atan::atan2;
pub(crate) use misc::{cbrt, exp, hypot};
pub(crate) use pow::pow;
pub(crate) use trig::{cos, sin};

/// High 32 bits of a double, as fdlibm's `__HI`.
#[inline]
pub(crate) fn hi(x: f64) -> i32 {
    (x.to_bits() >> 32) as u32 as i32
}

/// Low 32 bits of a double, as fdlibm's `__LO`.
#[inline]
pub(crate) fn lo(x: f64) -> u32 {
    x.to_bits() as u32
}

/// Builds a double from its high and low words.
#[inline]
pub(crate) fn from_words(hi: i32, lo: u32) -> f64 {
    f64::from_bits((u64::from(hi as u32) << 32) | u64::from(lo))
}

/// `x` with its high word replaced, as `__HI(x) = hi`.
#[inline]
pub(crate) fn with_hi(x: f64, hi: i32) -> f64 {
    from_words(hi, lo(x))
}

/// `x` with its low word replaced, as `__LO(x) = lo`.
#[inline]
pub(crate) fn with_lo(x: f64, lo: u32) -> f64 {
    from_words(hi(x), lo)
}

/// `Math.rint`, which is what Kotlin's `round(Double)` compiles to: round
/// half to even.
#[inline]
pub(crate) fn rint(x: f64) -> f64 {
    x.round_ties_even()
}

/// `Math.max(double, double)`: NaN wins, and +0.0 beats -0.0.
#[inline]
pub(crate) fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.to_bits() == (-0.0f64).to_bits() {
        return b;
    }
    if a >= b { a } else { b }
}

/// `Math.min(double, double)`: NaN wins, and -0.0 beats +0.0.
#[inline]
pub(crate) fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && b.to_bits() == (-0.0f64).to_bits() {
        return b;
    }
    if a <= b { a } else { b }
}

/// Kotlin's `Double.coerceIn(min, max)`. Kotlin throws when `min > max`; no
/// input to the scheme maths reaches that, and this returns `min` there
/// instead of panicking.
#[inline]
pub(crate) fn coerce_in(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

/// Kotlin's `Double.toInt()` (JVM `d2i`): truncate toward zero, saturate,
/// NaN becomes 0. Rust's `as` cast has exactly these semantics.
#[inline]
pub(crate) fn to_int(x: f64) -> i32 {
    x as i32
}

/// Kotlin's `Float.roundToInt()` (`Math.round(float)`): floor of `x + 0.5`
/// computed without double rounding. Widening to f64 makes the addition
/// exact for every float, so the floor is exact too.
#[inline]
pub(crate) fn round_float_to_int(x: f32) -> i32 {
    (f64::from(x) + 0.5).floor() as i32
}
