//! Android's `Path.approximate`: hwui's `SkPathGlue::approximate` (`libs/hwui/jni/Path.cpp`)
//! walking the path with `SkPath::Iter(path, false)`. AOSP compiles both with
//! `-ffp-contract=off`, so plain single-precision arithmetic in the same order matches it.

use crate::skia::{Point, SkPath, Verb};

/// What `SkPath::Iter` hands back: a verb and its points, the first being where it starts.
enum Step {
    Move(Point),
    Line(Point),
    Quad([Point; 3]),
    Cubic([Point; 4]),
    Close(Point),
}

/// The segments of `path` as `SkPath::Iter` without forced closing walks them: a trailing move is
/// dropped, and a close first draws the line back to the contour's move if it ends elsewhere.
fn steps(path: &SkPath) -> Vec<Step> {
    let (mut steps, mut at, mut move_to, mut last) =
        (vec![], 0, Point::default(), Point::default());
    for (i, verb) in path.verbs.iter().enumerate() {
        let pts = &path.points;
        match verb {
            Verb::Move if i + 1 == path.verbs.len() => break,
            Verb::Move => {
                (move_to, last) = (pts[at], pts[at]);
                steps.push(Step::Move(move_to));
                at += 1;
            }
            Verb::Line => {
                last = pts[at];
                steps.push(Step::Line(last));
                at += 1;
            }
            Verb::Quad => {
                steps.push(Step::Quad([last, pts[at], pts[at + 1]]));
                last = pts[at + 1];
                at += 2;
            }
            Verb::Cubic => {
                steps.push(Step::Cubic([last, pts[at], pts[at + 1], pts[at + 2]]));
                last = pts[at + 2];
                at += 3;
            }
            Verb::Close => {
                let nan = [last.x, last.y, move_to.x, move_to.y]
                    .iter()
                    .any(|v| v.is_nan());
                if last != move_to && !nan {
                    steps.push(Step::Line(move_to));
                }
                steps.push(Step::Close(move_to));
                last = move_to;
            }
        }
    }
    steps
}

/// The points hwui collects, with the length along the path at each.
#[derive(Default)]
struct Segments {
    points: Vec<Point>,
    lengths: Vec<f32>,
}

impl Segments {
    fn add_move(&mut self, point: Point) {
        let length = self.lengths.last().copied().unwrap_or(0.0);
        self.points.push(point);
        self.lengths.push(length);
    }

    fn add_line(&mut self, to: Point) {
        let Some(&back) = self.points.last() else {
            self.points.push(Point::default());
            self.lengths.push(0.0);
            return self.add_line_from(Point::default(), to);
        };
        if back != to {
            self.add_line_from(back, to);
        }
    }

    fn add_line_from(&mut self, back: Point, to: Point) {
        let length = self.lengths[self.lengths.len() - 1] + distance(back, to);
        self.points.push(to);
        self.lengths.push(length);
    }
}

/// `SkPoint::Distance`, which falls back to double precision only when the square overflows.
fn distance(a: Point, b: Point) -> f32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    let mag2 = dx * dx + dy * dy;
    if mag2.is_finite() {
        mag2.sqrt()
    } else {
        let (x, y) = (dx as f64, dy as f64);
        (x * x + y * y).sqrt() as f32
    }
}

fn cubic(t: f32, p: &[Point]) -> Point {
    let at = |p0: f32, p1: f32, p2: f32, p3: f32| {
        let one_minus_t = 1.0 - t;
        let omt2 = one_minus_t * one_minus_t;
        let omt3 = omt2 * one_minus_t;
        let (t2, t3) = (t * t, t * t * t);
        (omt3 * p0) + (3.0 * omt2 * t * p1) + (3.0 * one_minus_t * t2 * p2) + (t3 * p3)
    };
    Point {
        x: at(p[0].x, p[1].x, p[2].x, p[3].x),
        y: at(p[0].y, p[1].y, p[2].y, p[3].y),
    }
}

fn quad(t: f32, p: &[Point]) -> Point {
    let at = |p0: f32, p1: f32, p2: f32| {
        let one_minus_t = 1.0 - t;
        one_minus_t * ((one_minus_t * p0) + (t * p1)) + t * ((one_minus_t * p1) + (t * p2))
    };
    Point {
        x: at(p[0].x, p[1].x, p[2].x),
        y: at(p[0].y, p[1].y, p[2].y),
    }
}

type Curve = fn(f32, &[Point]) -> Point;

/// `subdividePoints`: the curve's point midway in t between two, and whether it lies further
/// than the error allows from the chord's midpoint.
fn subdivide(
    curve: Curve,
    p: &[Point],
    a: (f32, Point),
    b: (f32, Point),
    error2: f32,
) -> (bool, (f32, Point)) {
    let mid_t = (b.0 + a.0) / 2.0;
    let (mid_x, mid_y) = ((b.1.x + a.1.x) / 2.0, (b.1.y + a.1.y) / 2.0);
    let mid = curve(mid_t, p);
    let (ex, ey) = (mid.x - mid_x, mid.y - mid_y);
    ((ex * ex) + (ey * ey) > error2, (mid_t, mid))
}

/// `addBezier`: halves the curve's t intervals, in order, until each chord is within the error.
/// A cubic double-checks each split at its quarter until one needs it, for an S-shaped curve
/// whose midpoint happens to sit on the chord.
fn add_bezier(s: &mut Segments, curve: Curve, p: &[Point], error2: f32, mut double_check: bool) {
    let mut ts = vec![(0.0, curve(0.0, p)), (1.0, curve(1.0, p))];
    let (mut iter, mut next) = (0, 1);
    while next < ts.len() {
        loop {
            let (mut needs, mid) = subdivide(curve, p, ts[iter], ts[next], error2);
            if !needs && double_check {
                needs = subdivide(curve, p, ts[iter], mid, error2).0;
                double_check &= !needs;
            }
            // Android's map would take a t equal to one it holds as that one and never finish;
            // these icons never come near it, but this stops there rather than spinning.
            if !needs || mid.0 == ts[iter].0 || mid.0 == ts[next].0 {
                break;
            }
            ts.insert(iter + 1, mid);
            next = iter + 1;
        }
        iter = next;
        next += 1;
    }
    for (_, point) in ts {
        s.add_line(point);
    }
}

/// `path.approximate(error)`: for each point, the fraction of the path's length at it, then its
/// x and y.
pub(crate) fn approximate(path: &SkPath, error: f32) -> Vec<f32> {
    let mut s = Segments::default();
    let error2 = error * error;
    for step in steps(path) {
        match step {
            Step::Move(p) => s.add_move(p),
            Step::Line(p) | Step::Close(p) => s.add_line(p),
            Step::Quad(p) => add_bezier(&mut s, quad, &p, error2, false),
            Step::Cubic(p) => add_bezier(&mut s, cubic, &p, error2, true),
        }
    }
    if s.points.is_empty() {
        let lone = if path.verbs.len() == 1 {
            path.points[0]
        } else {
            Point::default()
        };
        s.add_move(lone);
    }
    let mut total = s.lengths[s.lengths.len() - 1];
    if total == 0.0 {
        s.points.push(s.points[s.points.len() - 1]);
        s.lengths.push(1.0);
        total = 1.0;
    }
    s.points
        .iter()
        .zip(&s.lengths)
        .flat_map(|(p, length)| [length / total, p.x, p.y])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::approximate;
    use crate::{nodes::parse, skia::build};

    fn flat(svg: &str) -> Vec<f32> {
        approximate(&build(&parse(svg).unwrap()), 0.25)
    }

    #[test]
    fn a_closed_square_measures_its_closing_side() {
        let points = flat("M 0.0 0.0 L 4.0 0.0 L 4.0 4.0 L 0.0 4.0 Z");
        let fractions: Vec<f32> = points.chunks(3).map(|p| p[0]).collect();
        assert_eq!(fractions, [0.0, 0.25, 0.5, 0.75, 1.0]);
        assert_eq!(points[12..], [1.0, 0.0, 0.0]);
    }

    #[test]
    fn moves_share_a_length_and_a_lone_move_still_has_two_points() {
        let points = flat("M 0.0 0.0 L 2.0 0.0 M 5.0 5.0 L 5.0 7.0");
        let fractions: Vec<f32> = points.chunks(3).map(|p| p[0]).collect();
        assert_eq!(fractions, [0.0, 0.5, 0.5, 1.0]);
        // The walk drops a trailing move, so a lone one stands in as the path's only point.
        assert_eq!(flat("M 3.0 3.0"), [0.0, 3.0, 3.0, 1.0, 3.0, 3.0]);
        assert_eq!(flat(""), [0.0, 0.0, 0.0, 1.0, 0.0, 0.0]);
    }

    #[test]
    fn a_straight_cubic_is_one_chord_and_a_bent_one_is_split() {
        assert_eq!(flat("M 0.0 0.0 C 1.0 0.0 2.0 0.0 3.0 0.0").len(), 6);
        assert!(flat("M 0.0 0.0 C 0.0 10.0 10.0 10.0 10.0 0.0").len() > 12);
    }
}
