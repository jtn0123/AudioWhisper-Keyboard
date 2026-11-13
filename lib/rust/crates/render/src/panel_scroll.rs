//! Android/Compose spline decay, independent of a platform clock or view.
use std::sync::OnceLock;

/// The AndroidFlingSpline table used by Compose's default scrollable fling.
pub fn fraction(time: f32) -> f32 {
    static POSITIONS: OnceLock<[f32; 101]> = OnceLock::new();
    let table = POSITIONS.get_or_init(|| {
        let mut positions = [1.0; 101];
        let mut low = 0.0;
        for (i, position) in positions.iter_mut().enumerate().take(100) {
            let alpha = i as f32 / 100.0;
            let mut high = 1.0;
            loop {
                let x = low + (high - low) / 2.0;
                let coefficient = 3.0 * x * (1.0 - x);
                let estimate = coefficient * ((1.0 - x) * 0.175 + x * 0.350_000_02) + x * x * x;
                if (estimate - alpha).abs() < 0.00001 {
                    *position = coefficient * ((1.0 - x) * 0.5 + x) + x * x * x;
                    break;
                }
                if estimate > alpha {
                    high = x;
                } else {
                    low = x;
                }
            }
        }
        positions
    });
    let time = time.clamp(0.0, 1.0);
    let index = (time * 100.0) as usize;
    if index >= 100 {
        return 1.0;
    }
    table[index]
        + (time - index as f32 / 100.0) * (table[index + 1] - table[index])
            / ((index + 1) as f32 / 100.0 - index as f32 / 100.0)
}

/// Duration in milliseconds and unsigned distance; same float/double boundaries as Compose.
pub fn decay(density: f32, velocity: f32) -> (i64, f32) {
    if !density.is_finite() || density <= 0.0 || !velocity.is_finite() || velocity == 0.0 {
        return (0, 0.0);
    }
    let coefficient = 0.015 * (386.0878 * density * 160.0 * 0.84);
    let rate = (0.78_f64.ln() / 0.9_f64.ln()) as f32 as f64;
    let logarithm = ((0.35 * velocity.abs()) as f64 / coefficient as f64).ln();
    (
        (1000.0 * (logarithm / (rate - 1.0)).exp()) as i64,
        (coefficient as f64 * (rate / (rate - 1.0) * logarithm).exp()) as f32,
    )
}

#[derive(Debug)]
pub struct Fling {
    pub start: i64,
    pub origin: f32,
    pub duration: i64,
    pub distance: f32,
}
impl Fling {
    pub fn new(start: i64, origin: f32, density: f32, velocity: f32) -> Option<Self> {
        let (duration, distance) = decay(density, velocity);
        (duration > 0 && velocity.abs() > 1.0).then_some(Self {
            start,
            origin,
            duration: duration * 1_000_000,
            distance: distance * velocity.signum(),
        })
    }
    pub fn value(&self, now: i64) -> f32 {
        self.origin + self.distance * fraction((now - self.start) as f32 / self.duration as f32)
    }
}
