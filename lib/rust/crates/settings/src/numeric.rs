//! kotlinx.serialization 1.12 AbstractJsonLexer.consumeNumericLiteral, including quoted
//! integers, exponent arithmetic, wrapping accumulation and final JVM Long conversion.
//! This deliberately differs from Kotlin's stored scalar toLongOrNull grammar.
use crate::Result;
pub fn decode(input: &[u16]) -> Result<i64> {
    let input = if input.first() == Some(&34) {
        if input.last() != Some(&34) || input.len() < 2 {
            return Err("invalid numeric quote");
        }
        &input[1..input.len() - 1]
    } else {
        input
    };
    let mut value = 0i64;
    let mut exponent = 0i64;
    let mut negative = false;
    let mut positive_exponent = false;
    let mut has_exponent = false;
    for (i, &unit) in input.iter().enumerate() {
        if matches!(unit, 69 | 101) && !has_exponent {
            if i == 0 {
                return Err("invalid exponent");
            }
            positive_exponent = true;
            has_exponent = true;
        } else if unit == 45 && has_exponent {
            positive_exponent = false;
        } else if unit == 43 && has_exponent {
            positive_exponent = true;
        } else if unit == 45 {
            if i != 0 {
                return Err("invalid numeric sign");
            }
            negative = true;
        } else if (48..=57).contains(&unit) {
            let digit = i64::from(unit - 48);
            if has_exponent {
                exponent = exponent.wrapping_mul(10).wrapping_add(digit);
            } else {
                value = value.wrapping_mul(10).wrapping_sub(digit);
                if value > 0 {
                    return Err("numeric overflow");
                }
            }
        } else {
            return Err("invalid numeric literal");
        }
    }
    if input.is_empty() || (negative && input.len() == 1) {
        return Err("empty numeric literal");
    }
    if has_exponent {
        let power = if positive_exponent {
            exponent as f64
        } else {
            -(exponent as f64)
        };
        let result = value as f64 * 10.0f64.powf(power);
        let limit = 9_223_372_036_854_776_000.0;
        if result > limit || result < -limit || result.floor() != result {
            return Err("numeric exponent overflow");
        }
        value = result as i64;
    }
    if negative {
        Ok(value)
    } else if value == i64::MIN {
        Err("numeric overflow")
    } else {
        Ok(-value)
    }
}
