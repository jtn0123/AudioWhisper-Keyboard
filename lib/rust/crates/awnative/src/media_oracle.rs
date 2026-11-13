//! Differential probes use the same layout, scheduler and budget code as the live surface.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jclass, jfloatArray, jintArray, jlongArray};
use render::media_cache::{Frame, MediaCache};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_MediaRules_trace(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
    ops: jlongArray,
) -> jlongArray {
    // SAFETY: local JVM arrays are copied; all generated frame buffers are owned by this probe.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<i32>(config, "cache bounds")?;
            let ops = env.read::<i64>(ops, "cache trace")?;
            if c.len() != 2 || c.iter().any(|v| *v < 0) || ops.len() % 8 != 0 {
                return Err(illegal("media trace"));
            }
            let mut cache = MediaCache::default();
            cache.budget = c[0] as usize;
            cache.per_item = c[1] as usize;
            let mut out = vec![];
            for o in ops.as_chunks::<8>().0 {
                let mut accepted = false;
                match o[0] {
                    0 => {
                        if !(1..=32).contains(&o[4]) || !(1..=32).contains(&o[5]) || o[2] < 0 {
                            return Err(illegal("trace dimensions"));
                        }
                        let f = Frame::argb(
                            o[4] as u16,
                            o[5] as u16,
                            &vec![0; o[4] as usize * o[5] as usize],
                            o[3],
                        )
                        .unwrap();
                        accepted = cache.upload(o[1], o[2] as usize, f, o[6] != 0);
                    }
                    1 => cache.visibility(
                        if o[1] == 0 {
                            Default::default()
                        } else {
                            [o[1]].into()
                        },
                        o[7],
                    ),
                    2 => {
                        accepted = cache.tick(o[7]);
                    }
                    3 => cache = MediaCache::default(),
                    _ => return Err(illegal("trace operation")),
                }
                out.extend([
                    cache.entries.get(&o[1]).map_or(-1, |e| e.frame as i64),
                    cache.deadline().unwrap_or(-1),
                    cache.bytes as i64,
                    cache.entries.len() as i64,
                    i64::from(accepted),
                ]);
            }
            env.make(&out)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_MediaRules_grid(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM owns the local array; the result copies pure grid coordinates.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "gif grid")?;
            if c.len() < 2 || c.iter().any(|v| !v.is_finite()) || c[0] <= 0.0 || c[1] <= 0.0 {
                return Err(illegal("gif grid"));
            }
            let rects = render::gif_panel::grid(c[0], c[1], &c[2..]);
            env.make(
                &rects
                    .iter()
                    .flat_map(|r| [r.left, r.top, r.right, r.bottom])
                    .collect::<Vec<_>>(),
            )
        })
    }
}
