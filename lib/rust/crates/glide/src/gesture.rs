//! A gesture's points, and the two transforms scoring compares them through: resampling to evenly
//! spaced points, and scaling into a unit box.
//!
//! Every float operation follows the Kotlin it replaces exactly. Rust never fuses a multiply and an
//! add on its own and `as i32` on a float already follows Java's rules (truncate, NaN is 0,
//! saturate), so only Java's `Math.max` and `Math.min` need writing out.

/// The most points a gesture keeps; later ones are dropped.
pub const MAX_POINTS: usize = 500;

/// How many evenly spaced points a gesture is resampled to.
pub const SAMPLES: usize = 200;

/// Java's `Math.max` for floats: NaN wins, and +0 beats -0.
pub(crate) fn java_max(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && a.is_sign_negative() {
        return b;
    }
    if a >= b { a } else { b }
}

/// Java's `Math.min` for floats: NaN wins, and -0 beats +0.
pub(crate) fn java_min(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return a;
    }
    if a == 0.0 && b == 0.0 && b.is_sign_negative() {
        return b;
    }
    if a <= b { a } else { b }
}

pub(crate) fn distance(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    (dx * dx + dy * dy).sqrt()
}

/// Up to [`MAX_POINTS`] points, in fixed storage so one gesture can be refilled for every word.
#[derive(Clone, Debug)]
pub struct Gesture {
    xs: [f32; MAX_POINTS],
    ys: [f32; MAX_POINTS],
    len: usize,
}

impl Default for Gesture {
    fn default() -> Self {
        Self::new()
    }
}

impl Gesture {
    pub const fn new() -> Self {
        Gesture {
            xs: [0.0; MAX_POINTS],
            ys: [0.0; MAX_POINTS],
            len: 0,
        }
    }

    /// A gesture through `xs` and `ys`, pairwise; points past [`MAX_POINTS`] are dropped.
    pub fn from_points(xs: &[f32], ys: &[f32]) -> Self {
        let mut gesture = Gesture::new();
        for (&x, &y) in xs.iter().zip(ys) {
            gesture.push(x, y);
        }
        gesture
    }

    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Adds a point, unless the gesture is full.
    pub fn push(&mut self, x: f32, y: f32) {
        if self.len >= MAX_POINTS {
            return;
        }
        self.xs[self.len] = x;
        self.ys[self.len] = y;
        self.len += 1;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// The point's x, or 0 past the end, as the Kotlin gesture's empty storage read.
    pub fn x(&self, i: usize) -> f32 {
        if i < self.len { self.xs[i] } else { 0.0 }
    }

    /// The point's y, or 0 past the end.
    pub fn y(&self, i: usize) -> f32 {
        if i < self.len { self.ys[i] } else { 0.0 }
    }

    /// The length of the path through every point.
    pub fn length(&self) -> f32 {
        let mut length = 0.0f32;
        for i in 1..self.len {
            length += distance(self.xs[i - 1], self.ys[i - 1], self.xs[i], self.ys[i]);
        }
        length
    }

    /// Writes into `out` this gesture as [`SAMPLES`] evenly spaced steps along its path, after its
    /// first point. Rounding can leave a step or two out; a single point is repeated instead, and an
    /// empty gesture stays empty.
    pub fn resample_into(&self, out: &mut Gesture) {
        out.clear();
        if self.len == 0 {
            return;
        }
        out.push(self.xs[0], self.ys[0]);
        if self.len == 1 {
            for _ in 0..SAMPLES {
                out.push(self.xs[0], self.ys[0]);
            }
            return;
        }
        let interpoint = self.length() / SAMPLES as f32;
        let (mut last_x, mut last_y) = (self.xs[0], self.ys[0]);
        let mut cumulative_error = 0.0f32;
        for i in 0..self.len - 1 {
            let mut dx = self.xs[i + 1] - self.xs[i];
            let mut dy = self.ys[i + 1] - self.ys[i];
            let norm = (dx * dx + dy * dy).sqrt();
            dx /= norm;
            dy /= norm;
            // The steps that fit between these two points are rarely a whole number. The fractions
            // left over add up, and once they make a whole step it is added here.
            let mut steps = norm / interpoint;
            cumulative_error += steps - (steps as i32) as f32;
            if cumulative_error > 1.0 {
                steps = (steps as i32).wrapping_add(cumulative_error as i32) as f32;
                cumulative_error %= 1.0;
            }
            // Kotlin went on stepping once the gesture was full, adding nothing; stopping is the same.
            let mut step = 0;
            while step < steps as i32 && out.len < MAX_POINTS {
                last_x += dx * interpoint;
                last_y += dy * interpoint;
                out.push(last_x, last_y);
                step += 1;
            }
        }
    }

    /// Writes into `out` this gesture scaled so its longer side is 1, centred on the origin.
    pub fn normalize_into(&self, out: &mut Gesture) {
        out.clear();
        let (mut max_x, mut max_y) = (-1.0f32, -1.0f32);
        let (mut min_x, mut min_y) = (10000.0f32, 10000.0f32);
        for i in 0..self.len {
            max_x = java_max(self.xs[i], max_x);
            max_y = java_max(self.ys[i], max_y);
            min_x = java_min(self.xs[i], min_x);
            min_y = java_min(self.ys[i], min_y);
        }
        let width = max_x - min_x;
        let height = max_y - min_y;
        let longest_side = java_max(java_max(width, height), 0.00001);
        let centroid_x = (width / 2.0 + min_x) / longest_side;
        let centroid_y = (height / 2.0 + min_y) / longest_side;
        for i in 0..self.len {
            out.push(
                self.xs[i] / longest_side - centroid_x,
                self.ys[i] / longest_side - centroid_y,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zigzag(points: usize) -> Gesture {
        let mut gesture = Gesture::new();
        for i in 0..points {
            gesture.push(i as f32 * 37.0, if i % 2 == 0 { 100.0 } else { 260.0 });
        }
        gesture
    }

    #[test]
    fn points_past_the_end_read_as_zero() {
        let mut gesture = Gesture::from_points(&[1.0, 3.0, 5.0], &[2.0, 4.0, 6.0]);
        gesture.clear();
        gesture.push(7.0, 8.0);

        assert_eq!(gesture.x(0), 7.0);
        assert_eq!(gesture.x(1), 0.0);
        assert_eq!(gesture.y(2), 0.0);
    }

    #[test]
    fn stops_at_500_points() {
        let mut gesture = Gesture::new();
        for i in 0..600 {
            gesture.push(i as f32, 0.0);
        }

        assert_eq!(gesture.len(), 500);
        assert_eq!(gesture.x(499), 499.0);
        assert_eq!(gesture.length(), 499.0);
        assert_eq!(gesture.x(500), 0.0);
    }

    #[test]
    fn a_single_point_resamples_to_itself_repeated() {
        let mut resampled = Gesture::new();
        Gesture::from_points(&[5.0], &[9.0]).resample_into(&mut resampled);

        // The point, then the 200 samples of it.
        assert_eq!(resampled.len(), 201);
        assert!((0..=200).all(|i| resampled.x(i) == 5.0 && resampled.y(i) == 9.0));
        assert_eq!(resampled.x(201), 0.0);
    }

    #[test]
    fn an_empty_gesture_resamples_to_nothing() {
        let mut resampled = zigzag(5);
        Gesture::new().resample_into(&mut resampled);

        assert!(resampled.is_empty());
    }

    #[test]
    fn resampling_spaces_points_evenly_along_the_path() {
        let mut resampled = Gesture::new();
        Gesture::from_points(&[0.0, 200.0], &[0.0, 0.0]).resample_into(&mut resampled);

        assert_eq!(resampled.len(), 201);
        for i in 0..=200 {
            assert_eq!(resampled.x(i), i as f32, "x at {i}");
        }
    }

    #[test]
    fn a_reused_target_matches_a_new_one() {
        let word = zigzag(6);
        let mut reused = zigzag(300);
        word.resample_into(&mut reused);
        let mut fresh = Gesture::new();
        word.resample_into(&mut fresh);

        assert_eq!(reused.len(), fresh.len());
        for i in 0..310 {
            assert_eq!(reused.x(i), fresh.x(i), "x at {i}");
            assert_eq!(reused.y(i), fresh.y(i), "y at {i}");
        }
    }

    #[test]
    fn normalizing_fits_the_longer_side_to_one_around_the_origin() {
        let mut normalized = zigzag(300);
        Gesture::from_points(&[10.0, 30.0, 30.0], &[50.0, 50.0, 60.0])
            .normalize_into(&mut normalized);

        assert_eq!(normalized.len(), 3);
        assert_eq!((normalized.x(0), normalized.y(0)), (-0.5, -0.25));
        assert_eq!((normalized.x(1), normalized.y(1)), (0.5, -0.25));
        assert_eq!((normalized.x(2), normalized.y(2)), (0.5, 0.25));
        assert_eq!(normalized.x(250), 0.0);
    }

    #[test]
    fn java_max_and_min_keep_nan_and_signed_zeros() {
        assert!(java_max(f32::NAN, 1.0).is_nan());
        assert!(java_max(1.0, f32::NAN).is_nan());
        assert!(java_min(f32::NAN, 1.0).is_nan());
        assert!(java_min(1.0, f32::NAN).is_nan());
        assert!(java_max(-0.0, 0.0).is_sign_positive());
        assert!(java_max(0.0, -0.0).is_sign_positive());
        assert!(java_min(0.0, -0.0).is_sign_negative());
        assert!(java_min(-0.0, 0.0).is_sign_negative());
    }
}
