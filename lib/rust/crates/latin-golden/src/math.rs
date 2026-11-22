//! Pre-A1 Kotlin rounding and character model, kept independent of suggest, as the questions used it.
use chars::code_point;
pub fn round_to_int(value: f32) -> Option<i32> {
    if value.is_nan() {
        return None;
    }
    const SIGNIFICAND_WIDTH: i32 = 24;
    const EXP_BIAS: i32 = 127;
    const EXP_BIT_MASK: i32 = 0x7F80_0000;
    const SIGNIF_BIT_MASK: i32 = 0x007F_FFFF;
    let bits = value.to_bits() as i32;
    let biased_exponent = (bits & EXP_BIT_MASK) >> (SIGNIFICAND_WIDTH - 1);
    let shift = (SIGNIFICAND_WIDTH - 2 + EXP_BIAS) - biased_exponent;
    if shift & -32 == 0 {
        let magnitude = (bits & SIGNIF_BIT_MASK) | (SIGNIF_BIT_MASK + 1);
        let signed = if bits < 0 { -magnitude } else { magnitude };
        Some(((signed >> shift) + 1) >> 1)
    } else {
        // Too small to round up, or a whole number already: Java's (int) cast, which saturates.
        Some(value as i32)
    }
}

pub fn to_lower(code_point: i32) -> i32 {
    // A code point is at most 0x10FFFF, so its lower case fits.
    u32::try_from(code_point).map_or(code_point, |cp| code_point::to_lower(cp) as i32)
}
