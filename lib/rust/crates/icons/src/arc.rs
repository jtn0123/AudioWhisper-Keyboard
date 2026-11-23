//! An elliptical arc as cubic Béziers: Compose's `drawArc` and `arcToBezier` (ui-graphics
//! 1.12.1, `PathParser.kt`), in the same double precision and the same order of operations, so
//! each control point rounds to the same float.

use crate::skia::SkPath;
use std::f64::consts::PI;

/// The arc from `from` to `to` with `radii`, turned `theta` degrees, as cubics on `p`.
pub(crate) fn draw_arc(
    p: &mut SkPath,
    from: [f64; 2],
    to: [f64; 2],
    radii: [f64; 2],
    theta: f64,
    more_than_half: bool,
    positive: bool,
) {
    let ([x0, y0], [x1, y1], [a, b]) = (from, to, radii);
    let theta_d = theta / 180.0 * PI;
    let (cos_t, sin_t) = (theta_d.cos(), theta_d.sin());
    // Into unit space: the inverse rotation, then the inverse scale.
    let x0p = (x0 * cos_t + y0 * sin_t) / a;
    let y0p = (-x0 * sin_t + y0 * cos_t) / b;
    let x1p = (x1 * cos_t + y1 * sin_t) / a;
    let y1p = (-x1 * sin_t + y1 * cos_t) / b;
    let (dx, dy) = (x0p - x1p, y0p - y1p);
    let (xm, ym) = ((x0p + x1p) / 2.0, (y0p + y1p) / 2.0);
    let dsq = dx * dx + dy * dy;
    if dsq == 0.0 {
        return;
    }
    let disc = 1.0 / dsq - 1.0 / 4.0;
    if disc < 0.0 {
        // The points are too far apart for these radii: grow them, as Compose does.
        let adjust = (dsq.sqrt() / 1.99999) as f32 as f64;
        draw_arc(
            p,
            from,
            to,
            [a * adjust, b * adjust],
            theta,
            more_than_half,
            positive,
        );
        return;
    }
    let s = disc.sqrt();
    let (sdx, sdy) = (s * dx, s * dy);
    let (mut cx, mut cy) = if more_than_half == positive {
        (xm - sdy, ym + sdx)
    } else {
        (xm + sdy, ym - sdx)
    };
    let eta0 = (y0p - cy).atan2(x0p - cx);
    let eta1 = (y1p - cy).atan2(x1p - cx);
    let mut sweep = eta1 - eta0;
    if positive != (sweep >= 0.0) {
        if sweep > 0.0 {
            sweep -= 2.0 * PI;
        } else {
            sweep += 2.0 * PI;
        }
    }
    cx *= a;
    cy *= b;
    let tcx = cx;
    cx = cx * cos_t - cy * sin_t;
    cy = tcx * sin_t + cy * cos_t;
    let ellipse = Ellipse {
        cx,
        cy,
        a,
        b,
        theta: theta_d,
    };
    arc_to_bezier(p, &ellipse, [x0, y0], eta0, sweep);
}

/// An ellipse centred at `cx`, `cy` with radii `a` and `b`, turned `theta` radians.
struct Ellipse {
    cx: f64,
    cy: f64,
    a: f64,
    b: f64,
    theta: f64,
}

/// The arc of `e` from `start` (the point at angle `eta`) sweeping `sweep`, a cubic per quarter
/// turn or part of one.
fn arc_to_bezier(p: &mut SkPath, e: &Ellipse, start: [f64; 2], eta: f64, sweep: f64) {
    let Ellipse { cx, cy, a, b, .. } = *e;
    let [mut eta1x, mut eta1y] = start;
    let segments = (sweep * 4.0 / PI).abs().ceil() as i32;
    let mut eta1 = eta;
    let (cos_t, sin_t) = (e.theta.cos(), e.theta.sin());
    let (cos_eta1, sin_eta1) = (eta1.cos(), eta1.sin());
    let mut ep1x = (-a * cos_t * sin_eta1) - (b * sin_t * cos_eta1);
    let mut ep1y = (-a * sin_t * sin_eta1) + (b * cos_t * cos_eta1);
    let per_segment = sweep / segments as f64;
    for _ in 0..segments {
        let eta2 = eta1 + per_segment;
        let (sin_eta2, cos_eta2) = (eta2.sin(), eta2.cos());
        let e2x = cx + (a * cos_t * cos_eta2) - (b * sin_t * sin_eta2);
        let e2y = cy + (a * sin_t * cos_eta2) + (b * cos_t * sin_eta2);
        let ep2x = (-a * cos_t * sin_eta2) - (b * sin_t * cos_eta2);
        let ep2y = (-a * sin_t * sin_eta2) + (b * cos_t * cos_eta2);
        let tan_diff2 = ((eta2 - eta1) / 2.0).tan();
        let alpha = (eta2 - eta1).sin() * ((4.0 + 3.0 * tan_diff2 * tan_diff2).sqrt() - 1.0) / 3.0;
        let (q1x, q1y) = (eta1x + alpha * ep1x, eta1y + alpha * ep1y);
        let (q2x, q2y) = (e2x - alpha * ep2x, e2y - alpha * ep2y);
        p.cubic_to([q1x, q1y, q2x, q2y, e2x, e2y].map(|v| v as f32));
        (eta1, eta1x, eta1y, ep1x, ep1y) = (eta2, e2x, e2y, ep2x, ep2y);
    }
}

#[cfg(test)]
mod tests {
    use crate::nodes::parse;
    use crate::skia::{Verb, build};

    #[test]
    fn a_half_circle_is_four_eighth_turn_cubics_ending_on_the_far_point() {
        let p = build(&parse("M 0.0 0.0 A 1.0 1.0 0.0 0 1 2.0 0.0").unwrap());
        assert_eq!(
            p.verbs,
            [
                Verb::Move,
                Verb::Cubic,
                Verb::Cubic,
                Verb::Cubic,
                Verb::Cubic
            ]
        );
        let end = p.points[12];
        assert!((end.x - 2.0).abs() < 1e-6 && end.y.abs() < 1e-6);
        // A positive arc from the left runs through the top, which is negative y on screen.
        assert!(p.points[6].y < -0.99);
    }

    #[test]
    fn coincident_ends_draw_nothing_and_short_radii_grow() {
        let p = build(&parse("M 1.0 1.0 a 2.0 2.0 0.0 0 0 0.0 0.0").unwrap());
        assert_eq!(p.verbs, [Verb::Move]);
        let p = build(&parse("M 0.0 0.0 A 0.1 0.1 0.0 0 1 4.0 0.0").unwrap());
        assert_eq!(p.verbs.len(), 5);
    }
}
