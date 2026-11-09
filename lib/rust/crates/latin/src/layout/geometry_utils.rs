//! `GeometryUtils` (`suggest/core/layout/geometry_utils.h`): only what typing reaches.
//!
//! Skipped, because only gesture input or the key-to-key distances typing never reads use them:
//! `getAngle` (`atan2f`), `getAngleDiff` (`ROUND_FLOAT_10000`, `floorf`) and `getDistanceInt`
//! (`hypotf`). The only caller of the first two for typing is `updateTouchPoints`' running sum of
//! angles, which `pushTouchPoint` ignores unless it is sampling a gesture; `getDistanceInt` fills
//! the sampled length cache and the key-to-key distances, which nothing in typing reads. So the
//! port calls no `libm` function at all.

/// `GeometryUtils::SQUARE_FLOAT`.
pub fn square_float(x: f32) -> f32 {
    x * x
}
