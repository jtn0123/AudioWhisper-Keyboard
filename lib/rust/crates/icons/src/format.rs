//! Kotlin's `iconPath` text: each path's points in a 24-unit box as `M x y` and `L x y`, a `Z`
//! before each new contour and after each path, every number as Kotlin wrote a `Float`.
//! The renderer reads the text back as doubles, so the digits matter, not only the values.

use crate::approximate::approximate;
use crate::nodes::parse;
use crate::skia::build;
use crate::sources::Source;

/// `source` as `iconPath` wrote it: a point starts a contour when it is the path's first or its
/// fraction of the length repeats the one before (a move).
pub(crate) fn icon_path(source: &Source) -> String {
    let mut out = String::new();
    for svg in &source.paths {
        let nodes = parse(svg).unwrap_or_default();
        let points = approximate(&build(&nodes), 0.25);
        let mut fraction = -1f32;
        for (i, point) in points.as_chunks::<3>().0.iter().enumerate() {
            let start = i == 0 || point[0] == fraction;
            if start && i > 0 {
                out.push_str("Z ");
            }
            out.push_str(if start { "M " } else { "L " });
            out.push_str(&java_float(point[1] * 24.0 / source.width));
            out.push(' ');
            out.push_str(&java_float(point[2] * 24.0 / source.height));
            out.push(' ');
            fraction = point[0];
        }
        out.push_str("Z ");
    }
    out
}

/// `Float.toString`: the shortest digits that read back as `value`, written plainly from 10⁻³
/// up to 10⁷ with at least one digit after the point, and as `d.dddE±n` outside that.
pub(crate) fn java_float(value: f32) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    let sign = if value.is_sign_negative() { "-" } else { "" };
    if value.is_infinite() {
        return format!("{sign}Infinity");
    }
    if value == 0.0 {
        return format!("{sign}0.0");
    }
    let scientific = format!("{:e}", value.abs());
    let (mantissa, exponent) = scientific.split_once('e').unwrap_or((&scientific, "0"));
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let exponent: i32 = exponent.parse().unwrap_or(0);
    let magnitude = value.abs() as f64;
    if !(1e-3..1e7).contains(&magnitude) {
        let rest = if digits.len() > 1 { &digits[1..] } else { "0" };
        return format!("{sign}{}.{rest}E{exponent}", &digits[..1]);
    }
    if exponent < 0 {
        let zeros = "0".repeat((-exponent - 1) as usize);
        return format!("{sign}0.{zeros}{digits}");
    }
    let whole = exponent as usize + 1;
    let (int, frac) = if digits.len() > whole {
        (digits[..whole].to_string(), digits[whole..].to_string())
    } else {
        (format!("{digits:0<whole$}"), "0".to_string())
    };
    format!("{sign}{int}.{frac}")
}

#[cfg(test)]
mod tests {
    use super::java_float;

    #[test]
    fn writes_floats_as_kotlin_does() {
        let cases: [(f32, &str); 14] = [
            (0.0, "0.0"),
            (-0.0, "-0.0"),
            (12.0, "12.0"),
            (1.5, "1.5"),
            (-3.25, "-3.25"),
            (0.1, "0.1"),
            (0.001, "0.001"),
            (0.000_999_999_9, "9.999999E-4"),
            (2.384_185_8e-7, "2.3841858E-7"),
            (1e7, "1.0E7"),
            (9_999_999.0, "9999999.0"),
            (123_456.79, "123456.79"),
            (20.270_834, "20.270834"),
            (1.0e-4, "1.0E-4"),
        ];
        for (value, text) in cases {
            assert_eq!(java_float(value), text, "{value:e}");
        }
    }
}
