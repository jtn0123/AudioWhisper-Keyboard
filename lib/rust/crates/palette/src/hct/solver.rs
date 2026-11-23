// Ported from material-color-utilities (Copyright 2021-2025 Google LLC,
// Apache-2.0) by way of MaterialKolor 5.0.1's Kotlin port (MIT licence, as
// declared in its published Maven POM; <https://github.com/jordond/materialkolor>).
//
// HctSolver: finds the sRGB colour with a requested hue, chroma and tone.

use std::f64::consts::PI;

use super::default_viewing_conditions;
use super::planes::CRITICAL_PLANES;
use crate::color::{
    argb_from_linrgb, argb_from_lstar, matrix_multiply, sanitize_degrees, signum, y_from_lstar,
};
use crate::jmath;

const SCALED_DISCOUNT_FROM_LINRGB: [[f64; 3]; 3] = [
    [
        0.001200833568784504,
        0.002389694492170889,
        0.0002795742885861124,
    ],
    [
        0.0005891086651375999,
        0.0029785502573438758,
        0.0003270666104008398,
    ],
    [
        0.00010146692491640572,
        0.0005364214359186694,
        0.0032979401770712076,
    ],
];
const LINRGB_FROM_SCALED_DISCOUNT: [[f64; 3]; 3] = [
    [1373.2198709594231, -1100.4251190754821, -7.278681089101213],
    [-271.815969077903, 559.6580465940733, -32.46047482791194],
    [1.9622899599665666, -57.173814538844006, 308.7233197812385],
];
const Y_FROM_LINRGB: [f64; 3] = [0.2126, 0.7152, 0.0722];
const NO_VERTEX: [f64; 3] = [-1.0, -1.0, -1.0];

fn sanitize_radians(angle: f64) -> f64 {
    (angle + PI * 8.0) % (PI * 2.0)
}

fn true_delinearized(rgb_component: f64) -> f64 {
    let normalized = rgb_component / 100.0;
    let delinearized = if normalized <= 0.0031308 {
        normalized * 12.92
    } else {
        1.055 * jmath::pow(normalized, 1.0 / 2.4) - 0.055
    };
    delinearized * 255.0
}

fn chromatic_adaptation(component: f64) -> f64 {
    let af = jmath::pow(component.abs(), 0.42);
    signum(component) * 400.0 * af / (af + 27.13)
}

fn hue_of(linrgb: [f64; 3]) -> f64 {
    let scaled = matrix_multiply(linrgb, &SCALED_DISCOUNT_FROM_LINRGB);
    let r_a = chromatic_adaptation(scaled[0]);
    let g_a = chromatic_adaptation(scaled[1]);
    let b_a = chromatic_adaptation(scaled[2]);
    let a = (11.0 * r_a + -12.0 * g_a + b_a) / 11.0;
    let b = (r_a + g_a - 2.0 * b_a) / 9.0;
    jmath::atan2(b, a)
}

fn are_in_cyclic_order(a: f64, b: f64, c: f64) -> bool {
    sanitize_radians(b - a) < sanitize_radians(c - a)
}

fn intercept(source: f64, mid: f64, target: f64) -> f64 {
    (mid - source) / (target - source)
}

fn lerp_point(source: [f64; 3], t: f64, target: [f64; 3]) -> [f64; 3] {
    [
        source[0] + (target[0] - source[0]) * t,
        source[1] + (target[1] - source[1]) * t,
        source[2] + (target[2] - source[2]) * t,
    ]
}

fn set_coordinate(source: [f64; 3], coordinate: f64, target: [f64; 3], axis: usize) -> [f64; 3] {
    let t = intercept(source[axis], coordinate, target[axis]);
    lerp_point(source, t, target)
}

fn is_bounded(x: f64) -> bool {
    (0.0..=100.0).contains(&x)
}

fn nth_vertex(y: f64, n: i32) -> [f64; 3] {
    let [k_r, k_g, k_b] = Y_FROM_LINRGB;
    let coord_a = if n % 4 <= 1 { 0.0 } else { 100.0 };
    let coord_b = if n % 2 == 0 { 0.0 } else { 100.0 };
    if n < 4 {
        let r = (y - coord_a * k_g - coord_b * k_b) / k_r;
        if is_bounded(r) {
            [r, coord_a, coord_b]
        } else {
            NO_VERTEX
        }
    } else if n < 8 {
        let g = (y - coord_b * k_r - coord_a * k_b) / k_g;
        if is_bounded(g) {
            [coord_b, g, coord_a]
        } else {
            NO_VERTEX
        }
    } else {
        let b = (y - coord_a * k_r - coord_b * k_g) / k_b;
        if is_bounded(b) {
            [coord_a, coord_b, b]
        } else {
            NO_VERTEX
        }
    }
}

fn bisect_to_segment(y: f64, target_hue: f64) -> ([f64; 3], [f64; 3]) {
    let mut left = NO_VERTEX;
    let mut right = left;
    let mut left_hue = 0.0;
    let mut right_hue = 0.0;
    let mut initialized = false;
    let mut uncut = true;
    for n in 0..=11 {
        let mid = nth_vertex(y, n);
        if mid[0] < 0.0 {
            continue;
        }
        let mid_hue = hue_of(mid);
        if !initialized {
            left = mid;
            right = mid;
            left_hue = mid_hue;
            right_hue = mid_hue;
            initialized = true;
            continue;
        }
        if uncut || are_in_cyclic_order(left_hue, mid_hue, right_hue) {
            uncut = false;
            if are_in_cyclic_order(left_hue, target_hue, mid_hue) {
                right = mid;
                right_hue = mid_hue;
            } else {
                left = mid;
                left_hue = mid_hue;
            }
        }
    }
    (left, right)
}

fn midpoint(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        (a[0] + b[0]) / 2.0,
        (a[1] + b[1]) / 2.0,
        (a[2] + b[2]) / 2.0,
    ]
}

fn critical_plane_below(x: f64) -> i32 {
    jmath::to_int((x - 0.5).floor())
}

fn critical_plane_above(x: f64) -> i32 {
    jmath::to_int((x - 0.5).ceil())
}

fn bisect_to_limit(y: f64, target_hue: f64) -> [f64; 3] {
    let (mut left, mut right) = bisect_to_segment(y, target_hue);
    let mut left_hue = hue_of(left);
    for axis in 0..3 {
        if left[axis] == right[axis] {
            continue;
        }
        let (mut l_plane, mut r_plane) = if left[axis] < right[axis] {
            (
                critical_plane_below(true_delinearized(left[axis])),
                critical_plane_above(true_delinearized(right[axis])),
            )
        } else {
            (
                critical_plane_above(true_delinearized(left[axis])),
                critical_plane_below(true_delinearized(right[axis])),
            )
        };
        for _ in 0..=7 {
            if (r_plane - l_plane).abs() <= 1 {
                break;
            }
            let m_plane = jmath::to_int((f64::from(l_plane + r_plane) / 2.0).floor());
            let mid_plane_coordinate = CRITICAL_PLANES[m_plane as usize];
            let mid = set_coordinate(left, mid_plane_coordinate, right, axis);
            let mid_hue = hue_of(mid);
            if are_in_cyclic_order(left_hue, target_hue, mid_hue) {
                right = mid;
                r_plane = m_plane;
            } else {
                left = mid;
                left_hue = mid_hue;
                l_plane = m_plane;
            }
        }
    }
    midpoint(left, right)
}

fn inverse_chromatic_adaptation(adapted: f64) -> f64 {
    let adapted_abs = adapted.abs();
    let base = jmath::max(0.0, 27.13 * adapted_abs / (400.0 - adapted_abs));
    signum(adapted) * jmath::pow(base, 1.0 / 0.42)
}

fn find_result_by_j(hue_radians: f64, chroma: f64, y: f64) -> u32 {
    let mut j = y.sqrt() * 11.0;
    let vc = default_viewing_conditions();
    let t_inner_coeff = 1.0 / jmath::pow(1.64 - jmath::pow(0.29, vc.n), 0.73);
    let e_hue = 0.25 * (jmath::cos(hue_radians + 2.0) + 3.8);
    let p1 = e_hue * (50000.0 / 13.0) * vc.nc * vc.ncb;
    let h_sin = jmath::sin(hue_radians);
    let h_cos = jmath::cos(hue_radians);
    for iteration_round in 0..=4 {
        let j_normalized = j / 100.0;
        let alpha = if chroma == 0.0 || j == 0.0 {
            0.0
        } else {
            chroma / j_normalized.sqrt()
        };
        let t = jmath::pow(alpha * t_inner_coeff, 1.0 / 0.9);
        let ac = vc.aw * jmath::pow(j_normalized, 1.0 / vc.c / vc.z);
        let p2 = ac / vc.nbb;
        let gamma = 23.0 * (p2 + 0.305) * t / (23.0 * p1 + 11.0 * t * h_cos + 108.0 * t * h_sin);
        let a = gamma * h_cos;
        let b = gamma * h_sin;
        let r_a = (460.0 * p2 + 451.0 * a + 288.0 * b) / 1403.0;
        let g_a = (460.0 * p2 - 891.0 * a - 261.0 * b) / 1403.0;
        let b_a = (460.0 * p2 - 220.0 * a - 6300.0 * b) / 1403.0;
        let scaled = [
            inverse_chromatic_adaptation(r_a),
            inverse_chromatic_adaptation(g_a),
            inverse_chromatic_adaptation(b_a),
        ];
        let linrgb = matrix_multiply(scaled, &LINRGB_FROM_SCALED_DISCOUNT);
        if linrgb[0] < 0.0 || linrgb[1] < 0.0 || linrgb[2] < 0.0 {
            return 0;
        }
        let [k_r, k_g, k_b] = Y_FROM_LINRGB;
        let fnj = k_r * linrgb[0] + k_g * linrgb[1] + k_b * linrgb[2];
        if fnj <= 0.0 {
            return 0;
        }
        if iteration_round == 4 || (fnj - y).abs() < 0.002 {
            return if linrgb[0] > 100.01 || linrgb[1] > 100.01 || linrgb[2] > 100.01 {
                0
            } else {
                argb_from_linrgb(linrgb)
            };
        }
        j -= (fnj - y) * j / (2.0 * fnj);
    }
    0
}

/// `HctSolver.solveToInt`: the sRGB colour closest to the requested HCT.
pub(super) fn solve_to_int(hue_degrees: f64, chroma: f64, lstar: f64) -> u32 {
    let hue_degrees = sanitize_degrees(hue_degrees);
    if chroma < 0.0001 || lstar < 0.0001 || lstar > 99.9999 {
        return argb_from_lstar(lstar);
    }
    let hue_radians = hue_degrees / 180.0 * PI;
    let y = y_from_lstar(lstar);
    let exact_answer = find_result_by_j(hue_radians, chroma, y);
    if exact_answer != 0 {
        return exact_answer;
    }
    argb_from_linrgb(bisect_to_limit(y, hue_radians))
}
