//! Compose's default pointer velocity tracker, as androidx.compose.ui 1.12.1 ships it (Apache-2.0).
//!
//! `PlatformVelocityTracker()` is a `Lsq2VelocityTracker` because
//! `AndroidComposeUiFlags.isFrameworkVelocityTrackerEnabled` defaults to false, so Compose does
//! not use the framework tracker that `velocity.rs` ports for View-style panels. Its
//! `VelocityTracker1D` keeps a 20-sample ring of (uptime ms, position), fits the samples no older
//! than 100 ms and no further than 40 ms apart with an unweighted quadratic, and reports the
//! linear coefficient. The fit is a Gram–Schmidt QR in f32, step for step as the bytecode does
//! it; 1.12.1 clamps a degenerate column's norm to 1e-6 instead of throwing.
const HISTORY: usize = 20;
const HORIZON_MS: f32 = 100.0;
const STOPPED_MS: f32 = 40.0;
const MIN_SAMPLES: usize = 3;

#[derive(Clone, Debug, Default)]
pub struct ComposeVelocity {
    samples: [Option<(i64, f32)>; HISTORY],
    index: usize,
    last_change: i64,
}

impl ComposeVelocity {
    /// `addPointerInputChange` for a down change: forget the old gesture, then record it.
    pub fn down(&mut self, ms: i64, position: f32) {
        self.reset();
        self.change(&[(ms, position)]);
    }
    /// A move change: its historical samples, then its current one (last).
    pub fn change(&mut self, samples: &[(i64, f32)]) {
        for &(ms, position) in samples {
            self.index = (self.index + 1) % HISTORY;
            self.samples[self.index] = Some((ms, position));
        }
        if let Some(&(ms, _)) = samples.last() {
            self.last_change = ms;
        }
    }
    /// An up change records nothing, but a pause over 40 ms since the last change means the
    /// pointer had stopped.
    pub fn up(&mut self, ms: i64) {
        if ms - self.last_change > STOPPED_MS as i64 {
            self.reset();
        }
        self.last_change = ms;
    }
    pub fn reset(&mut self) {
        self.samples = [None; HISTORY];
        self.index = 0;
    }
    /// `calculateVelocity(maximum)` in units per second: zero or NaN is 0, otherwise clamped.
    pub fn velocity(&self, maximum: f32) -> f32 {
        let velocity = self.unclamped();
        if velocity == 0.0 || velocity.is_nan() {
            0.0
        } else if velocity > 0.0 {
            velocity.min(maximum)
        } else {
            velocity.max(-maximum)
        }
    }
    fn unclamped(&self) -> f32 {
        let Some(newest) = self.samples[self.index] else {
            return 0.0;
        };
        let (mut points, mut times) = ([0.0; HISTORY], [0.0; HISTORY]);
        let (mut count, mut index, mut previous) = (0, self.index, newest);
        while count < HISTORY {
            let Some(sample) = self.samples[index] else {
                break;
            };
            let age = (newest.0 - sample.0) as f32;
            let gap = (sample.0 - previous.0).abs() as f32;
            previous = sample;
            if age > HORIZON_MS || gap > STOPPED_MS {
                break;
            }
            points[count] = sample.1;
            times[count] = -age;
            index = if index == 0 { HISTORY } else { index } - 1;
            count += 1;
        }
        if count < MIN_SAMPLES {
            return 0.0;
        }
        poly_fit(&times[..count], &points[..count], 2)[1] * 1000.0
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0;
    for i in 0..a.len() {
        sum += a[i] * b[i];
    }
    sum
}

/// `polyFitLeastSquares(x, y, x.len(), degree)`: polynomial coefficients, constant first.
pub fn poly_fit(x: &[f32], y: &[f32], degree: usize) -> Vec<f32> {
    let m = x.len();
    let n = degree.min(m.saturating_sub(1)) + 1;
    let mut a = vec![vec![0.0f32; m]; n];
    for h in 0..m {
        a[0][h] = 1.0;
        for i in 1..n {
            a[i][h] = a[i - 1][h] * x[h];
        }
    }
    let mut q = vec![vec![0.0f32; m]; n];
    let mut r = vec![vec![0.0f32; n]; n];
    for j in 0..n {
        let mut w = a[j].clone();
        for z in &q[..j] {
            let along = dot(&w, z);
            for h in 0..m {
                w[h] -= along * z[h];
            }
        }
        let norm = (f64::from(dot(&w, &w)).sqrt()) as f32;
        let inverse = 1.0 / norm.max(0.000_001);
        for value in &mut w {
            *value *= inverse;
        }
        for i in 0..n {
            r[j][i] = if i < j { 0.0 } else { dot(&w, &a[i]) };
        }
        q[j] = w;
    }
    let mut coefficients = vec![0.0f32; n];
    for i in (0..n).rev() {
        let mut value = dot(&q[i], y);
        for j in (i + 1..n).rev() {
            value -= r[i][j] * coefficients[j];
        }
        coefficients[i] = value / r[i][i];
    }
    coefficients
}
