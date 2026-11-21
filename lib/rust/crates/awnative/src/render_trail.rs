//! Font-independent geometry oracle for the same trail used by the render session.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jfloatArray, jlong, jlongArray, jobject};
use render::{protocol::*, scene::Scene, touch::Action, trail::Trail};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideTrailRules_circles(
    env: *mut JNIEnv,
    _: jobject,
    times: jlongArray,
    coords: jfloatArray,
    finish: jlong,
    now: jlong,
    duration: jlong,
    strip: f32,
) -> jfloatArray {
    // SAFETY: all arrays are local JVM references, copied before the pure geometry calculation.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let times = env.read::<i64>(times, "trail times")?;
            let coords = env.read::<f32>(coords, "trail points")?;
            if coords.len() != times.len() * 2
                || coords.iter().any(|v| !v.is_finite())
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
                        coords[i * 2].to_bits() as i32,
                        coords[i * 2 + 1].to_bits() as i32,
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
            env.make(&out)
        })
    }
}
