// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// Hct, plus the parts of Cam16 and ViewingConditions that it uses. Only the
// default viewing conditions are ever needed, so they are built once.

mod planes;
mod solver;

use std::sync::OnceLock;

use crate::color::{
    self, lerp, linearized, lstar_from_argb, sanitize_degrees, signum, to_degrees, y_from_lstar,
};
use crate::jmath;

pub(crate) const XYZ_TO_CAM16RGB: [[f64; 3]; 3] = [
    [0.401288, 0.650173, -0.051461],
    [-0.250268, 1.204414, 0.045854],
    [-0.002079, 0.048952, 0.953127],
];

/// `ViewingConditions`, without the fields the palette maths never reads.
pub(crate) struct ViewingConditions {
    pub(crate) n: f64,
    pub(crate) aw: f64,
    pub(crate) nbb: f64,
    pub(crate) ncb: f64,
    pub(crate) c: f64,
    pub(crate) nc: f64,
    pub(crate) rgb_d: [f64; 3],
    pub(crate) fl: f64,
    pub(crate) z: f64,
}

/// `ViewingConditions.DEFAULT`.
pub(crate) fn default_viewing_conditions() -> &'static ViewingConditions {
    static DEFAULT: OnceLock<ViewingConditions> = OnceLock::new();
    DEFAULT.get_or_init(|| {
        make_viewing_conditions(
            color::WHITE_POINT_D65,
            200.0 / std::f64::consts::PI * y_from_lstar(50.0) / 100.0,
            50.0,
            2.0,
        )
    })
}

/// `ViewingConditions.make` with `discountingIlluminant = false`.
fn make_viewing_conditions(
    white_point: [f64; 3],
    adapting_luminance: f64,
    background_lstar: f64,
    surround: f64,
) -> ViewingConditions {
    let lstar = jmath::max(0.1, background_lstar);
    let m = &XYZ_TO_CAM16RGB;
    let wp = white_point;
    let r_w = wp[0] * m[0][0] + wp[1] * m[0][1] + wp[2] * m[0][2];
    let g_w = wp[0] * m[1][0] + wp[1] * m[1][1] + wp[2] * m[1][2];
    let b_w = wp[0] * m[2][0] + wp[1] * m[2][1] + wp[2] * m[2][2];
    let f = 0.8 + surround / 10.0;
    let c = if f >= 0.9 {
        lerp(0.59, 0.69, (f - 0.9) * 10.0)
    } else {
        lerp(0.525, 0.59, (f - 0.8) * 10.0)
    };
    let d = f * (1.0 - 1.0 / 3.6 * jmath::exp((-adapting_luminance - 42.0) / 92.0));
    let d = jmath::coerce_in(d, 0.0, 1.0);
    let rgb_d = [
        d * (100.0 / r_w) + 1.0 - d,
        d * (100.0 / g_w) + 1.0 - d,
        d * (100.0 / b_w) + 1.0 - d,
    ];
    let k = 1.0 / (5.0 * adapting_luminance + 1.0);
    let k4 = k * k * k * k;
    let k4f = 1.0 - k4;
    let fl = k4 * adapting_luminance + 0.1 * k4f * k4f * jmath::cbrt(5.0 * adapting_luminance);
    let n = y_from_lstar(lstar) / wp[1];
    let z = 1.48 + n.sqrt();
    let nbb = 0.725 / jmath::pow(n, 0.2);
    let factors = [
        jmath::pow(fl * rgb_d[0] * r_w / 100.0, 0.42),
        jmath::pow(fl * rgb_d[1] * g_w / 100.0, 0.42),
        jmath::pow(fl * rgb_d[2] * b_w / 100.0, 0.42),
    ];
    let rgb_a = factors.map(|f| 400.0 * f / (f + 27.13));
    let aw = (2.0 * rgb_a[0] + rgb_a[1] + 0.05 * rgb_a[2]) * nbb;
    ViewingConditions {
        n,
        aw,
        nbb,
        ncb: nbb,
        c,
        nc: f,
        rgb_d,
        fl,
        z,
    }
}

/// `Cam16.fromInt`, returning only hue and chroma (all `Hct` keeps).
fn cam16_hue_chroma(argb: u32) -> (f64, f64) {
    let vc = default_viewing_conditions();
    let red_l = linearized(color::red(argb));
    let green_l = linearized(color::green(argb));
    let blue_l = linearized(color::blue(argb));
    let x = 0.41233895 * red_l + 0.35762064 * green_l + 0.18051042 * blue_l;
    let y = 0.2126 * red_l + 0.7152 * green_l + 0.0722 * blue_l;
    let z = 0.01932141 * red_l + 0.11916382 * green_l + 0.95034478 * blue_l;
    let m = &XYZ_TO_CAM16RGB;
    let r_t = x * m[0][0] + y * m[0][1] + z * m[0][2];
    let g_t = x * m[1][0] + y * m[1][1] + z * m[1][2];
    let b_t = x * m[2][0] + y * m[2][1] + z * m[2][2];
    let r_d = vc.rgb_d[0] * r_t;
    let g_d = vc.rgb_d[1] * g_t;
    let b_d = vc.rgb_d[2] * b_t;
    let r_af = jmath::pow(vc.fl * r_d.abs() / 100.0, 0.42);
    let g_af = jmath::pow(vc.fl * g_d.abs() / 100.0, 0.42);
    let b_af = jmath::pow(vc.fl * b_d.abs() / 100.0, 0.42);
    let r_a = signum(r_d) * 400.0 * r_af / (r_af + 27.13);
    let g_a = signum(g_d) * 400.0 * g_af / (g_af + 27.13);
    let b_a = signum(b_d) * 400.0 * b_af / (b_af + 27.13);
    let a = (11.0 * r_a + -12.0 * g_a + b_a) / 11.0;
    let b = (r_a + g_a - 2.0 * b_a) / 9.0;
    let u = (20.0 * r_a + 20.0 * g_a + 21.0 * b_a) / 20.0;
    let p2 = (40.0 * r_a + 20.0 * g_a + b_a) / 20.0;
    let hue = sanitize_degrees(to_degrees(jmath::atan2(b, a)));
    let ac = p2 * vc.nbb;
    let j = 100.0 * jmath::pow(ac / vc.aw, vc.c * vc.z);
    let hue_prime = if hue < 20.14 { hue + 360.0 } else { hue };
    let e_hue = 0.25 * (jmath::cos(color::to_radians(hue_prime) + 2.0) + 3.8);
    let p1 = 50000.0 / 13.0 * e_hue * vc.nc * vc.ncb;
    let t = p1 * jmath::hypot(a, b) / (u + 0.305);
    let alpha = jmath::pow(1.64 - jmath::pow(0.29, vc.n), 0.73) * jmath::pow(t, 0.9);
    let chroma = alpha * (j / 100.0).sqrt();
    (hue, chroma)
}

/// A colour in hue, chroma and tone. Equality is by `argb`, as in Kotlin.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Hct {
    pub(crate) argb: u32,
    pub(crate) hue: f64,
    pub(crate) chroma: f64,
    pub(crate) tone: f64,
}

impl PartialEq for Hct {
    fn eq(&self, other: &Self) -> bool {
        self.argb == other.argb
    }
}

impl Hct {
    /// `Hct.fromInt`.
    pub(crate) fn from_int(argb: u32) -> Hct {
        let (hue, chroma) = cam16_hue_chroma(argb);
        Hct {
            argb,
            hue,
            chroma,
            tone: lstar_from_argb(argb),
        }
    }

    /// `Hct.from(hue, chroma, tone)`.
    pub(crate) fn from(hue: f64, chroma: f64, tone: f64) -> Hct {
        Hct::from_int(solve(hue, chroma, tone))
    }

    /// `Hct.isYellow`.
    pub(crate) fn is_yellow(&self) -> bool {
        is_yellow(self.hue)
    }

    /// `Hct.isBlue`.
    pub(crate) fn is_blue(&self) -> bool {
        is_blue(self.hue)
    }

    /// `Hct.isCyan`.
    pub(crate) fn is_cyan(&self) -> bool {
        is_cyan(self.hue)
    }
}

/// `Hct.from(hue, chroma, tone).toInt()`, without the round trip through
/// CAM16 that only the other fields of the `Hct` need.
pub(crate) fn solve(hue: f64, chroma: f64, tone: f64) -> u32 {
    solver::solve_to_int(hue, chroma, tone)
}

/// `Hct.isBlue(hue)`.
pub(crate) fn is_blue(hue: f64) -> bool {
    (250.0..270.0).contains(&hue)
}

/// `Hct.isYellow(hue)`.
pub(crate) fn is_yellow(hue: f64) -> bool {
    (105.0..125.0).contains(&hue)
}

/// `Hct.isCyan(hue)`.
pub(crate) fn is_cyan(hue: f64) -> bool {
    (170.0..207.0).contains(&hue)
}
