//! Kotlin's `String.toFloatOrNull()` on the JVM and Android: Kotlin screens the text with its own
//! grammar, then `java.lang.Float.parseFloat` reads it, and anything either rejects is null.
//! Together they accept, after trimming units up to U+0020 from both ends:
//!
//! * `NaN` or `Infinity`, with an optional sign;
//! * a decimal: digits with an optional `.` and fraction (or just `.` and a fraction), an optional
//!   exponent `e`/`E`, sign and digits, and an optional last `f`, `F`, `d` or `D`;
//! * a hexadecimal: `0x` or `0X`, hex digits as a decimal's, a required exponent `p`/`P`, sign
//!   and decimal digits, and the same optional suffix.
//!
//! The value is the nearest `f32`, ties to even, as Java gives; an exponent too big for an `int`
//! gives infinity or zero.

/// Kotlin's `toFloatOrNull()` of `text`.
pub fn to_float_or_null(text: &[u16]) -> Option<f32> {
    let text = trim(text);
    let (negative, body) = match text.first() {
        Some(&unit) if unit == u16::from(b'+') || unit == u16::from(b'-') => {
            (unit == u16::from(b'-'), &text[1..])
        }
        _ => (false, text),
    };
    let ascii: Vec<u8> = body
        .iter()
        .map(|&it| u8::try_from(it).ok())
        .collect::<Option<_>>()?;
    let magnitude = match ascii.as_slice() {
        b"NaN" => return Some(f32::NAN),
        b"Infinity" => f32::INFINITY,
        [b'0', b'x' | b'X', rest @ ..] => hexadecimal(without_suffix(rest))?,
        _ => decimal(without_suffix(&ascii))?,
    };
    Some(if negative { -magnitude } else { magnitude })
}

/// `text` without the units up to U+0020 at either end, as both Kotlin and Java's `trim` drop.
fn trim(text: &[u16]) -> &[u16] {
    let start = text.iter().position(|&it| it > 0x20).unwrap_or(text.len());
    let end = text
        .iter()
        .rposition(|&it| it > 0x20)
        .map_or(start, |it| it + 1);
    &text[start..end]
}

/// `number` without a last `f`, `F`, `d` or `D`.
fn without_suffix(number: &[u8]) -> &[u8] {
    match number {
        [rest @ .., b'f' | b'F' | b'd' | b'D'] => rest,
        _ => number,
    }
}

/// The leading run of `number` whose units pass `test`, and the rest.
fn split(number: &[u8], test: impl Fn(u8) -> bool) -> (&[u8], &[u8]) {
    number.split_at(
        number
            .iter()
            .position(|&it| !test(it))
            .unwrap_or(number.len()),
    )
}

/// A significand: digits, then optionally `.` and more, with at least one digit in all.
fn significand(number: &[u8], test: impl Fn(u8) -> bool) -> Option<(&[u8], &[u8], &[u8])> {
    let (whole, rest) = split(number, &test);
    let (fraction, rest) = match rest {
        [b'.', after @ ..] => split(after, &test),
        _ => (&rest[..0], rest),
    };
    (!whole.is_empty() || !fraction.is_empty()).then_some((whole, fraction, rest))
}

/// An exponent's optional sign and decimal digits, which must be all of `number`; saturates far
/// past any float's range, as Java's does.
fn exponent(number: &[u8]) -> Option<i64> {
    let (negative, digits) = match number {
        [b'-', rest @ ..] => (true, rest),
        [b'+', rest @ ..] => (false, rest),
        _ => (false, number),
    };
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    let value = digits.iter().fold(0i64, |value, &it| {
        (value * 10 + i64::from(it - b'0')).min(LIMIT)
    });
    Some(if negative { -value } else { value })
}

/// Bigger than any exponent a float can use, however many digits come before it.
const LIMIT: i64 = 1 << 40;

fn decimal(number: &[u8]) -> Option<f32> {
    let (whole, fraction, rest) = significand(number, |it| it.is_ascii_digit())?;
    let power = match rest {
        [] => 0,
        [b'e' | b'E', exponent_part @ ..] => exponent(exponent_part)?,
        _ => return None,
    };
    let digits: Vec<u8> = whole.iter().chain(fraction).copied().collect();
    let Some(first) = digits.iter().position(|&it| it != b'0') else {
        return Some(0.0);
    };
    // The value is 0.DIGITS × 10^at.
    let at = (whole.len() as i64 - first as i64).saturating_add(power);
    if at > 40 {
        return Some(f32::INFINITY);
    }
    if at < -50 {
        return Some(0.0);
    }
    let digits = std::str::from_utf8(&digits[first..]).ok()?;
    format!("0.{digits}e{at}").parse().ok()
}

fn hexadecimal(number: &[u8]) -> Option<f32> {
    let (whole, fraction, rest) = significand(number, |it| it.is_ascii_hexdigit())?;
    let power = match rest {
        [b'p' | b'P', exponent_part @ ..] => exponent(exponent_part)?,
        _ => return None,
    };
    let digits: Vec<u8> = whole.iter().chain(fraction).copied().collect();
    let Some(first) = digits.iter().position(|&it| it != b'0') else {
        return Some(0.0);
    };
    let digits = &digits[first..];
    // The first 16 digits fill a u64; any others only say whether something is below it.
    let kept = &digits[..digits.len().min(16)];
    let below = digits[kept.len()..].iter().any(|&it| it != b'0');
    let bits = kept
        .iter()
        .fold(0u64, |bits, &it| bits << 4 | u64::from(hex_value(it)));
    let shifted = 4 * (digits.len() - kept.len()) as i64 - 4 * fraction.len() as i64;
    Some(round(bits, below, power.saturating_add(shifted)))
}

fn hex_value(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        _ => (digit | 0x20) - b'a' + 10,
    }
}

/// The `f32` nearest `(bits + a little more if below) × 2^power`, ties to even; `bits` isn't 0.
fn round(bits: u64, below: bool, power: i64) -> f32 {
    let top = power + i64::from(63 - bits.leading_zeros());
    if top > 127 {
        return f32::INFINITY;
    }
    // The power of two of the result's last bit: 24 bits down from the top, or a subnormal's.
    let last = (top - 23).max(-149);
    let shift = last - power;
    let mantissa = if shift <= 0 {
        u128::from(bits) << -shift
    } else if shift >= 128 {
        0
    } else {
        let bits = u128::from(bits);
        let kept = bits >> shift;
        let dropped = bits & ((1 << shift) - 1);
        let half = 1 << (shift - 1);
        let up = dropped > half || (dropped == half && (below || kept & 1 == 1));
        kept + u128::from(up)
    };
    from_parts(mantissa as u32, last)
}

/// The `f32` `mantissa × 2^last`, where `mantissa` has at most 24 bits, or 25 if rounding just
/// carried into a new one.
fn from_parts(mut mantissa: u32, mut last: i64) -> f32 {
    if mantissa >= 1 << 24 {
        mantissa >>= 1;
        last += 1;
    }
    if mantissa < 1 << 23 {
        return f32::from_bits(mantissa); // subnormal, or zero
    }
    let field = last + 23 + 127;
    if field > 254 {
        return f32::INFINITY;
    }
    f32::from_bits((field as u32) << 23 | (mantissa & 0x7F_FFFF))
}

#[cfg(test)]
mod tests;
