// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// ColorUtils and MathUtils: sRGB/XYZ/L* conversions and angle helpers.

use crate::jmath;

const SRGB_TO_XYZ: [[f64; 3]; 3] = [
    [0.41233895, 0.35762064, 0.18051042],
    [0.2126, 0.7152, 0.0722],
    [0.01932141, 0.11916382, 0.95034478],
];

pub(crate) const WHITE_POINT_D65: [f64; 3] = [95.047, 100.0, 108.883];

/// `ColorUtils.argbFromRgb`: opaque colour from 8-bit channels.
pub(crate) fn argb_from_rgb(red: i32, green: i32, blue: i32) -> u32 {
    (255u32 << 24)
        | (((red & 255) as u32) << 16)
        | (((green & 255) as u32) << 8)
        | (blue & 255) as u32
}

/// `ColorUtils.argbFromLinrgb`.
pub(crate) fn argb_from_linrgb(linrgb: [f64; 3]) -> u32 {
    let r = delinearized(linrgb[0]);
    let g = delinearized(linrgb[1]);
    let b = delinearized(linrgb[2]);
    argb_from_rgb(r, g, b)
}

/// `ColorUtils.redFromArgb`.
pub(crate) fn red(argb: u32) -> i32 {
    ((argb >> 16) & 255) as i32
}

/// `ColorUtils.greenFromArgb`.
pub(crate) fn green(argb: u32) -> i32 {
    ((argb >> 8) & 255) as i32
}

/// `ColorUtils.blueFromArgb`.
pub(crate) fn blue(argb: u32) -> i32 {
    (argb & 255) as i32
}

/// `ColorUtils.xyzFromArgb`.
fn xyz_from_argb(argb: u32) -> [f64; 3] {
    let r = linearized(red(argb));
    let g = linearized(green(argb));
    let b = linearized(blue(argb));
    matrix_multiply([r, g, b], &SRGB_TO_XYZ)
}

/// `ColorUtils.labFromArgb`.
pub(crate) fn lab_from_argb(argb: u32) -> [f64; 3] {
    let linear_r = linearized(red(argb));
    let linear_g = linearized(green(argb));
    let linear_b = linearized(blue(argb));
    let m = &SRGB_TO_XYZ;
    let x = m[0][0] * linear_r + m[0][1] * linear_g + m[0][2] * linear_b;
    let y = m[1][0] * linear_r + m[1][1] * linear_g + m[1][2] * linear_b;
    let z = m[2][0] * linear_r + m[2][1] * linear_g + m[2][2] * linear_b;
    let x_normalized = x / WHITE_POINT_D65[0];
    let y_normalized = y / WHITE_POINT_D65[1];
    let z_normalized = z / WHITE_POINT_D65[2];
    let fx = lab_f(x_normalized);
    let fy = lab_f(y_normalized);
    let fz = lab_f(z_normalized);
    let l = 116.0 * fy - 16.0;
    let a = 500.0 * (fx - fy);
    let b = 200.0 * (fy - fz);
    [l, a, b]
}

/// `ColorUtils.argbFromLstar`: the grey with the given L*.
pub(crate) fn argb_from_lstar(lstar: f64) -> u32 {
    let y = y_from_lstar(lstar);
    let component = delinearized(y);
    argb_from_rgb(component, component, component)
}

/// `ColorUtils.lstarFromArgb`.
pub(crate) fn lstar_from_argb(argb: u32) -> f64 {
    let y = xyz_from_argb(argb)[1];
    116.0 * lab_f(y / 100.0) - 16.0
}

/// `ColorUtils.yFromLstar`.
pub(crate) fn y_from_lstar(lstar: f64) -> f64 {
    100.0 * lab_invf((lstar + 16.0) / 116.0)
}

/// `ColorUtils.lstarFromY`.
pub(crate) fn lstar_from_y(y: f64) -> f64 {
    lab_f(y / 100.0) * 116.0 - 16.0
}

/// `ColorUtils.linearized`: 8-bit sRGB channel to linear 0..100.
pub(crate) fn linearized(rgb_component: i32) -> f64 {
    let normalized = f64::from(rgb_component) / 255.0;
    if normalized <= 0.040449936 {
        normalized / 12.92 * 100.0
    } else {
        jmath::pow((normalized + 0.055) / 1.055, 2.4) * 100.0
    }
}

/// `ColorUtils.delinearized`: linear 0..100 to an 8-bit sRGB channel.
pub(crate) fn delinearized(rgb_component: f64) -> i32 {
    let normalized = rgb_component / 100.0;
    let delinearized = if normalized <= 0.0031308 {
        normalized * 12.92
    } else {
        1.055 * jmath::pow(normalized, 1.0 / 2.4) - 0.055
    };
    jmath::to_int(jmath::coerce_in(
        jmath::rint(delinearized * 255.0),
        0.0,
        255.0,
    ))
}

/// `ColorUtils.labF`. Note: `pow(t, 1/3)`, not `cbrt`, as the Kotlin has it.
fn lab_f(t: f64) -> f64 {
    let e = 216.0 / 24389.0;
    let kappa = 24389.0 / 27.0;
    if t > e {
        jmath::pow(t, 1.0 / 3.0)
    } else {
        (kappa * t + 16.0) / 116.0
    }
}

/// `ColorUtils.labInvf`.
fn lab_invf(ft: f64) -> f64 {
    let e = 216.0 / 24389.0;
    let kappa = 24389.0 / 27.0;
    let ft3 = ft * ft * ft;
    if ft3 > e {
        ft3
    } else {
        (116.0 * ft - 16.0) / kappa
    }
}

/// `MathUtils.signum`, which returns an `Int` (NaN gives 1).
pub(crate) fn signum(num: f64) -> f64 {
    if num < 0.0 {
        -1.0
    } else if num == 0.0 {
        0.0
    } else {
        1.0
    }
}

/// `MathUtils.lerp`.
pub(crate) fn lerp(start: f64, stop: f64, amount: f64) -> f64 {
    (1.0 - amount) * start + amount * stop
}

/// `MathUtils.sanitizeDegrees(Int)`.
pub(crate) fn sanitize_degrees_int(degrees: i32) -> i32 {
    let mut sanitized = degrees % 360;
    if sanitized < 0 {
        sanitized += 360;
    }
    sanitized
}

/// `MathUtils.sanitizeDegrees(Double)`.
pub(crate) fn sanitize_degrees(degrees: f64) -> f64 {
    let mut sanitized = degrees % 360.0;
    if sanitized < 0.0 {
        sanitized += 360.0;
    }
    sanitized
}

/// `MathUtils.matrixMultiply`.
pub(crate) fn matrix_multiply(row: [f64; 3], m: &[[f64; 3]; 3]) -> [f64; 3] {
    let a = row[0] * m[0][0] + row[1] * m[0][1] + row[2] * m[0][2];
    let b = row[0] * m[1][0] + row[1] * m[1][1] + row[2] * m[1][2];
    let c = row[0] * m[2][0] + row[1] * m[2][1] + row[2] * m[2][2];
    [a, b, c]
}

/// `MathUtils.toDegrees`.
pub(crate) fn to_degrees(angrad: f64) -> f64 {
    angrad * 57.29577951308232
}

/// `MathUtils.toRadians`.
pub(crate) fn to_radians(angdeg: f64) -> f64 {
    angdeg * 0.017453292519943295
}
