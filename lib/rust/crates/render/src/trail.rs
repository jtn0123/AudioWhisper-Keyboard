//! Compose's physical-pixel glide circles: 20px head, 3px spacing and 0.99 radius reduction.
//! Completed/cancelled trails use AccelerateInterpolator's quadratic 20->0 radius animation.
use crate::{
    protocol::*,
    scene::{Scene, union},
    touch::Action,
};
use keys::Rect;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}
#[derive(Clone, Copy, Debug)]
struct Point {
    x: f32,
    y: f32,
    time: i64,
}
#[derive(Debug, Default)]
pub struct Trail {
    active: Vec<Point>,
    fading: Vec<Point>,
    fade_start: i64,
}
impl Trail {
    pub fn actions(&mut self, actions: &[Action], now: i64) {
        for &Action(kind, x, y) in actions {
            match kind {
                GLIDE_POINT => self.active.push(Point {
                    x: f32::from_bits(x as u32),
                    y: f32::from_bits(y as u32),
                    time: now,
                }),
                GLIDE_COMPLETE | GLIDE_CANCEL if !self.active.is_empty() => {
                    self.fading = std::mem::take(&mut self.active);
                    self.fade_start = now;
                }
                _ => {}
            }
        }
    }
    pub fn frame(&mut self, scene: &mut Scene, now: i64, show: bool, duration: i64) -> bool {
        scene.trail.clear();
        if !show || duration <= 0 {
            self.active.clear();
            self.fading.clear();
            return false;
        }
        let fraction = ((now - self.fade_start).max(0) as f32 / duration as f32).clamp(0.0, 1.0);
        if fraction < 1.0 {
            circles(
                &self.fading,
                now,
                duration,
                20.0 + fraction * fraction * -20.0,
                scene.strip,
                &mut scene.trail,
            );
        } else {
            self.fading.clear();
        }
        circles(
            &self.active,
            now,
            duration,
            20.0,
            scene.strip,
            &mut scene.trail,
        );
        // Points age out even if a finger stops moving. Never poll after the final visible circle.
        !scene.trail.is_empty()
    }
}
fn circles(
    points: &[Point],
    now: i64,
    duration: i64,
    initial_radius: f32,
    strip: f32,
    out: &mut Vec<Circle>,
) {
    let mut radius = initial_radius;
    let (mut prev_x, mut prev_y) = points.last().map_or((0.0, 0.0), |p| (p.x, p.y));
    for i in (1..points.len()).rev() {
        if now - points[i - 1].time > duration {
            break;
        }
        let dx = prev_x - points[i - 1].x;
        let dy = prev_y - points[i - 1].y;
        let count = ((dx * dx + dy * dy).sqrt() / 3.0) as i32;
        for j in 0..count {
            radius *= 0.99;
            let t = j as f32 / count as f32;
            let x = points[i].x * (1.0 - t) + points[i - 1].x * t;
            let y = points[i].y * (1.0 - t) + points[i - 1].y * t;
            out.push(Circle {
                x,
                y: y + strip,
                radius,
            });
            prev_x = x;
            prev_y = y;
        }
    }
}
pub fn bounds(circles: &[Circle]) -> Option<Rect> {
    circles
        .iter()
        .map(|c| Rect {
            left: c.x - c.radius,
            top: c.y - c.radius,
            right: c.x + c.radius,
            bottom: c.y + c.radius,
        })
        .reduce(union)
}
