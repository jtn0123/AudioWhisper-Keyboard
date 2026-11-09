//! `TouchPositionCorrectionUtils` (`suggest/core/layout/touch_position_correction_utils.h`).

const A: f32 = 0.0;
const B: f32 = 0.24;
const C: f32 = 1.20;
const R0: f32 = 0.0;
/// The sweet spot.
const R1: f32 = 0.25;
const R2: f32 = 1.0;

/// `TouchPositionCorrectionUtils::getSweetSpotFactor`: how much a tap's normalised squared
/// distance from a key costs. Typing calls it with `is_touch_position_correction_enabled` false
/// (a keyboard with no sweet spots), which is `std::min(C, x)`.
///
/// The enabled branch is written with `mul_add`: clang's default `-ffp-contract=on` fuses the
/// two products and the sum on arm64 (host and Android), and the test shows the bits differ
/// without it. Typing never reaches this branch.
pub fn get_sweet_spot_factor(
    is_touch_position_correction_enabled: bool,
    normalized_squared_distance: f32,
) -> f32 {
    let x = normalized_squared_distance;
    if !is_touch_position_correction_enabled {
        // std::min(C, x) is `(x < C) ? x : C`, but clang lowers it to a minNum (`fminnm` on
        // arm64): C for a quiet NaN, and the quieted NaN for a signalling one. No tap's distance
        // is NaN, so this only matters to the differential test.
        const QUIET_BIT: u32 = 0x0040_0000;
        if x.is_nan() {
            return if x.to_bits() & QUIET_BIT == 0 {
                f32::from_bits(x.to_bits() | QUIET_BIT)
            } else {
                C
            };
        }
        return if x < C { x } else { C };
    }
    if x < R0 {
        A
    } else if x < R1 {
        A.mul_add(R1 - x, B * (x - R0)) / (R1 - R0)
    } else if x < R2 {
        B.mul_add(R2 - x, C * (x - R1)) / (R2 - R1)
    } else {
        C
    }
}
