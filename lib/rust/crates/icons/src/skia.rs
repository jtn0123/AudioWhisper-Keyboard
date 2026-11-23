//! The path Compose built from an icon's nodes: `List<PathNode>.toPath` (ui-graphics 1.12.1)
//! calling Android's `android.graphics.Path`, which is Skia's `SkPath`. Both halves are kept, as
//! each tracks its own current point: Compose's for `H`, `V` and the reflective curves, Skia's for
//! the relative commands it is handed.

use crate::arc::draw_arc;
use crate::nodes::Node;

/// A point, in Skia's single precision.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Point {
    pub x: f32,
    pub y: f32,
}

/// What `SkPath` records for each command.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Verb {
    Move,
    Line,
    Quad,
    Cubic,
    Close,
}

/// `SkPath`'s verbs and points, and its `fLastMoveToIndex`: the index of the contour's move, or
/// its complement once the contour is closed.
#[derive(Debug)]
pub(crate) struct SkPath {
    pub verbs: Vec<Verb>,
    pub points: Vec<Point>,
    last_move_to: i32,
}

impl SkPath {
    fn new() -> Self {
        Self {
            verbs: vec![],
            points: vec![],
            last_move_to: !0,
        }
    }

    fn last_pt(&self) -> Point {
        self.points.last().copied().unwrap_or_default()
    }

    /// After a close, the next segment starts from the closed contour's move.
    fn inject_move_to_if_needed(&mut self) {
        if self.last_move_to < 0 {
            let at = if self.verbs.is_empty() {
                Point::default()
            } else {
                self.points[!self.last_move_to as usize]
            };
            self.move_to(at.x, at.y);
        }
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.last_move_to = self.points.len() as i32;
        self.verbs.push(Verb::Move);
        self.points.push(Point { x, y });
    }

    fn r_move_to(&mut self, dx: f32, dy: f32) {
        let from = match self.points.len() {
            0 => Point::default(),
            n if self.last_move_to >= 0 => self.points[n - 1],
            _ => self.points[!self.last_move_to as usize],
        };
        self.move_to(from.x + dx, from.y + dy);
    }

    fn segment(&mut self, verb: Verb, points: &[f32]) {
        self.inject_move_to_if_needed();
        self.verbs.push(verb);
        for [x, y] in points.as_chunks::<2>().0 {
            self.points.push(Point { x: *x, y: *y });
        }
    }

    fn relative(&mut self, verb: Verb, deltas: &[f32]) {
        self.inject_move_to_if_needed();
        let from = self.last_pt();
        let mut absolute = [0f32; 6];
        for (i, d) in deltas.iter().enumerate() {
            absolute[i] = if i % 2 == 0 { from.x } else { from.y } + d;
        }
        self.segment(verb, &absolute[..deltas.len()]);
    }

    pub(crate) fn cubic_to(&mut self, points: [f32; 6]) {
        self.segment(Verb::Cubic, &points);
    }

    fn close(&mut self) {
        if self.verbs.last().is_some_and(|&v| v != Verb::Close) {
            self.verbs.push(Verb::Close);
        }
        self.last_move_to ^= (!self.last_move_to) >> 31;
    }
}

fn is_curve(command: u8) -> bool {
    matches!(command, b'C' | b'c' | b'S' | b's')
}

fn is_quad(command: u8) -> bool {
    matches!(command, b'Q' | b'q' | b'T' | b't')
}

/// The path `nodes` make, as `toPath` made it.
pub(crate) fn build(nodes: &[Node]) -> SkPath {
    let mut p = SkPath::new();
    let (mut cx, mut cy, mut kx, mut ky, mut sx, mut sy) = (0f32, 0f32, 0f32, 0f32, 0f32, 0f32);
    let mut previous = nodes.first().map_or(b'Z', |n| n.command);
    for node in nodes {
        let a = node.args;
        match node.command {
            b'm' => {
                (cx, cy) = (cx + a[0], cy + a[1]);
                p.r_move_to(a[0], a[1]);
                (sx, sy) = (cx, cy);
            }
            b'M' => {
                (cx, cy) = (a[0], a[1]);
                p.move_to(a[0], a[1]);
                (sx, sy) = (cx, cy);
            }
            b'l' => {
                p.relative(Verb::Line, &a[..2]);
                (cx, cy) = (cx + a[0], cy + a[1]);
            }
            b'L' => {
                p.segment(Verb::Line, &a[..2]);
                (cx, cy) = (a[0], a[1]);
            }
            b'h' => {
                p.relative(Verb::Line, &[a[0], 0.0]);
                cx += a[0];
            }
            b'H' => {
                p.segment(Verb::Line, &[a[0], cy]);
                cx = a[0];
            }
            b'v' => {
                p.relative(Verb::Line, &[0.0, a[0]]);
                cy += a[0];
            }
            b'V' => {
                p.segment(Verb::Line, &[cx, a[0]]);
                cy = a[0];
            }
            b'c' => {
                p.relative(Verb::Cubic, &a[..6]);
                (kx, ky) = (cx + a[2], cy + a[3]);
                (cx, cy) = (cx + a[4], cy + a[5]);
            }
            b'C' => {
                p.segment(Verb::Cubic, &a[..6]);
                (kx, ky, cx, cy) = (a[2], a[3], a[4], a[5]);
            }
            b's' => {
                let (rx, ry) = if is_curve(previous) {
                    (cx - kx, cy - ky)
                } else {
                    (0.0, 0.0)
                };
                p.relative(Verb::Cubic, &[rx, ry, a[0], a[1], a[2], a[3]]);
                (kx, ky) = (cx + a[0], cy + a[1]);
                (cx, cy) = (cx + a[2], cy + a[3]);
            }
            b'S' => {
                let (rx, ry) = if is_curve(previous) {
                    (2.0 * cx - kx, 2.0 * cy - ky)
                } else {
                    (cx, cy)
                };
                p.segment(Verb::Cubic, &[rx, ry, a[0], a[1], a[2], a[3]]);
                (kx, ky, cx, cy) = (a[0], a[1], a[2], a[3]);
            }
            b'q' => {
                p.relative(Verb::Quad, &a[..4]);
                (kx, ky) = (cx + a[0], cy + a[1]);
                (cx, cy) = (cx + a[2], cy + a[3]);
            }
            b'Q' => {
                p.segment(Verb::Quad, &a[..4]);
                (kx, ky, cx, cy) = (a[0], a[1], a[2], a[3]);
            }
            b't' => {
                let (rx, ry) = if is_quad(previous) {
                    (cx - kx, cy - ky)
                } else {
                    (0.0, 0.0)
                };
                p.relative(Verb::Quad, &[rx, ry, a[0], a[1]]);
                (kx, ky) = (cx + rx, cy + ry);
                (cx, cy) = (cx + a[0], cy + a[1]);
            }
            b'T' => {
                let (rx, ry) = if is_quad(previous) {
                    (2.0 * cx - kx, 2.0 * cy - ky)
                } else {
                    (cx, cy)
                };
                p.segment(Verb::Quad, &[rx, ry, a[0], a[1]]);
                (kx, ky, cx, cy) = (rx, ry, a[0], a[1]);
            }
            b'a' | b'A' => {
                let (ex, ey) = if node.command == b'a' {
                    (a[5] + cx, a[6] + cy)
                } else {
                    (a[5], a[6])
                };
                let (from, to) = ([cx as f64, cy as f64], [ex as f64, ey as f64]);
                let radii = [a[0] as f64, a[1] as f64];
                draw_arc(
                    &mut p,
                    from,
                    to,
                    radii,
                    a[2] as f64,
                    a[3] != 0.0,
                    a[4] != 0.0,
                );
                (cx, cy, kx, ky) = (ex, ey, ex, ey);
            }
            _ => {
                (cx, cy, kx, ky) = (sx, sy, sx, sy);
                p.close();
            }
        }
        previous = node.command;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nodes::parse;

    fn verbs(svg: &str) -> (Vec<Verb>, Vec<(f32, f32)>) {
        let p = build(&parse(svg).unwrap());
        (p.verbs, p.points.iter().map(|p| (p.x, p.y)).collect())
    }

    #[test]
    fn relative_commands_follow_skias_last_point() {
        let (v, pts) = verbs("M 1.0 1.0 l 2.0 0.0 Z l 1.0 1.0 m 1.0 1.0 h 1.0");
        use Verb::*;
        assert_eq!(v, [Move, Line, Close, Move, Line, Move, Line]);
        // After the close, the line starts again from the contour's move, which a close adds no
        // point for; the next move is relative to the line's end.
        assert_eq!(pts[2..], [(1.0, 1.0), (2.0, 2.0), (3.0, 3.0), (4.0, 3.0)]);
    }

    #[test]
    fn closes_once_and_reflects_only_after_a_curve() {
        let (v, _) = verbs("M 0.0 0.0 Z Z");
        assert_eq!(v, [Verb::Move, Verb::Close]);
        let (_, pts) = verbs("M 0.0 0.0 C 1.0 1.0 2.0 1.0 3.0 0.0 S 5.0 -1.0 6.0 0.0");
        assert_eq!(pts[4], (4.0, -1.0));
        let (_, pts) = verbs("M 0.0 0.0 L 3.0 0.0 S 5.0 -1.0 6.0 0.0");
        assert_eq!(pts[2], (3.0, 0.0));
    }
}
