//! Java's arithmetic, where Rust's differs.

use chars::code_point;

/// Kotlin's `Float.roundToInt()`: `Math.round`, which rounds halves up and saturates, or `None`
/// for NaN, where Kotlin throws. Written as the JDK writes it, from the float's bits, so that
/// 0.49999997 rounds to 0 (adding 0.5 would make it 1).
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

/// `Character.toLowerCase(codePoint)` of any `Int`: one that isn't a code point stays as it is.
pub fn to_lower(code_point: i32) -> i32 {
    // A code point is at most 0x10FFFF, so its lower case fits.
    u32::try_from(code_point).map_or(code_point, |cp| code_point::to_lower(cp) as i32)
}

/// Java's `codePoints()` of a string as `Int`s: a lone surrogate is a code point of its own.
pub fn code_points(text: &[u16]) -> Vec<i32> {
    // As above, every code point fits.
    chars::text::code_points(text)
        .map(|(_, cp)| cp as i32)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_as_java_does() {
        for (value, rounded) in [
            (0.0, 0),
            (-0.0, 0),
            (0.5, 1),
            (-0.5, 0),
            (1.5, 2),
            (-1.5, -1),
            (2.5, 3),
            (0.499_999_97, 0),
            (-2.4, -2),
            (-2.6, -3),
            (1e-30, 0),
            (8_388_609.0, 8_388_609),
            (3e9, i32::MAX),
            (-3e9, i32::MIN),
            (f32::INFINITY, i32::MAX),
            (f32::NEG_INFINITY, i32::MIN),
        ] {
            assert_eq!(round_to_int(value), Some(rounded), "{value}");
        }
        assert_eq!(round_to_int(f32::NAN), None);
    }

    #[test]
    fn lower_cases_any_int() {
        assert_eq!(to_lower('A' as i32), 'a' as i32);
        assert_eq!(to_lower(-1), -1);
        assert_eq!(to_lower(i32::MIN), i32::MIN);
        assert_eq!(to_lower(0x11_0000), 0x11_0000);
    }

    #[test]
    fn reads_code_points_with_lone_surrogates() {
        let text: Vec<u16> = "a😀".encode_utf16().chain([0xD800]).collect();
        assert_eq!(code_points(&text), ['a' as i32, 0x1F600, 0xD800]);
    }
}
