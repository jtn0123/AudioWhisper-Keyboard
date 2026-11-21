//! Font-independent geometry oracle for the same trail used by the render session.
use crate::jni::{Result, illegal};
use render::{protocol::*, scene::Scene, touch::Action, trail::Trail};

/// The circles a trail of `points` at `times`, finished at `finish` (or not, if negative), draws
/// at `now`, after a fade of `duration`; the first value is 1 while it still animates.
pub fn circles(
    times: &[i64],
    points: &[f32],
    finish: i64,
    now: i64,
    duration: i64,
    strip: f32,
) -> Result<Vec<f32>> {
    if points.len() != times.len() * 2
        || points.iter().any(|v| !v.is_finite())
        || !strip.is_finite()
        || duration < 0
        || times.windows(2).any(|p| p[0] > p[1])
    {
        return Err(illegal("invalid trail fixture"));
    }
    let mut trail = Trail::default();
    for (i, &time) in times.iter().enumerate() {
        trail.actions(
            &[Action(
                GLIDE_POINT,
                points[i * 2].to_bits() as i32,
                points[i * 2 + 1].to_bits() as i32,
            )],
            time,
        );
    }
    if finish >= 0 {
        trail.actions(&[Action(GLIDE_COMPLETE, 0, 0)], finish);
    }
    let mut scene = Scene {
        strip,
        ..Scene::default()
    };
    let animating = trail.frame(&mut scene, now, true, duration);
    let mut out = vec![if animating { 1.0 } else { 0.0 }];
    out.extend(scene.trail.iter().flat_map(|c| [c.x, c.y, c.radius]));
    Ok(out)
}
