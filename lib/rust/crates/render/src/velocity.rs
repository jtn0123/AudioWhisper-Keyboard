//! Android 16 VelocityTracker's unweighted LSQ2 planar strategy, in MotionEvent uptime.
//! AOSP frameworks/native/libs/input/VelocityTracker.cpp (android16-release), Apache-2.0.
//! UP preserves the last velocity, a >40ms pause stops it; history is capped at 20/100ms.
use crate::touch::Motion;
use std::collections::{BTreeMap, VecDeque};
#[derive(Debug, Default)]
pub struct Velocity {
    samples: BTreeMap<i32, VecDeque<(i64, f32, f32)>>,
    last: i64,
}
impl Velocity {
    pub fn event(&mut self, event: &Motion) {
        if event.action == 0 || event.action == 3 {
            self.samples.clear();
        }
        if event.action == 5 {
            self.samples.remove(&event.ids[event.index]);
        }
        if event.action == 1 || event.action == 6 {
            if event.times.last().copied().unwrap_or(0) - self.last > 40_000_000 {
                self.samples.clear();
            }
            return;
        }
        if !matches!(event.action, 0 | 2 | 5) {
            return;
        }
        for (h, sample) in event.coords.chunks_exact(event.ids.len() * 2).enumerate() {
            let now = event.times[h + 1];
            for (p, &id) in event.ids.iter().enumerate() {
                if self.samples.contains_key(&id) && now - self.last > 40_000_000 {
                    self.samples.clear();
                }
                self.last = now;
                let trace = self.samples.entry(id).or_default();
                if trace.back().is_some_and(|p| p.0 == now) {
                    trace.pop_back();
                }
                trace.push_back((now, sample[p * 2], sample[p * 2 + 1]));
                while trace.len() > 20 || trace.front().is_some_and(|p| now - p.0 > 100_000_000) {
                    trace.pop_front();
                }
            }
        }
    }
    pub fn get(&self, id: i32) -> (f32, f32) {
        let Some(points) = self.samples.get(&id) else {
            return (0.0, 0.0);
        };
        (
            axis(points, false) * 1000.0 / 1000.0,
            axis(points, true) * 1000.0 / 1000.0,
        )
    }
}
fn axis(points: &VecDeque<(i64, f32, f32)>, y: bool) -> f32 {
    if points.len() < 2 {
        return 0.0;
    }
    let newest = points.back().expect("two samples").0;
    if points.len() == 2 {
        // AOSP's general Gram-Schmidt QR solver reduces to these two columns for LSQ1.
        let a = points[0];
        let b = points[1];
        let x = [0.0, -(b.0 - a.0) as f32 * 0.000000001];
        let q0 = 1.0 / 2.0_f32.sqrt();
        let dot = x[0] * q0 + x[1] * q0;
        let mut q = [x[0] - dot * q0, x[1] - dot * q0];
        let norm = (q[0] * q[0] + q[1] * q[1]).sqrt();
        if norm < 0.000001 {
            return 0.0;
        }
        let inv = 1.0 / norm;
        q[0] *= inv;
        q[1] *= inv;
        return (q[0] * (if y { b.2 } else { b.1 }) + q[1] * (if y { a.2 } else { a.1 }))
            / (q[0] * x[0] + q[1] * x[1]);
    }
    let (mut sx, mut sxy, mut sy, mut sx2, mut sx3, mut sx2y, mut sx4) =
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for &(time, px, py) in points {
        let x = -(newest - time) as f32 * 0.000000001;
        let y = if y { py } else { px };
        let x2 = x * x;
        let x3 = x2 * x;
        let x4 = x3 * x;
        sx += x;
        sx2 += x2;
        sxy += x * y;
        sx2y += x2 * y;
        sy += y;
        sx3 += x3;
        sx4 += x4;
    }
    let n = points.len() as f32;
    let xx = sx2 - sx * sx / n;
    let xy = sxy - sx * sy / n;
    let xx2 = sx3 - sx * sx2 / n;
    let x2y = sx2y - sx2 * sy / n;
    let x2x2 = sx4 - sx2 * sx2 / n;
    let denominator = xx * x2x2 - xx2 * xx2;
    if denominator == 0.0 {
        0.0
    } else {
        (xy * x2x2 - x2y * xx2) / denominator
    }
}
