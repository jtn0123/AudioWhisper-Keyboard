//! Shared preference policy JNI. All returned ids refer to the caller's value pool.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jclass, jintArray, jobjectArray};
use render::arrangement::{Arrangement, Editor};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelPolicyNative_history(
    env: *mut JNIEnv,
    _: jclass,
    pinned: jintArray,
    recent: jintArray,
    config: jintArray,
) -> jobjectArray {
    // SAFETY: JVM local arrays are copied and outputs remain local references for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<i32>(config, "history policy")?;
            let [enabled, ps, rs, pl, rl, op, value, offset] = c.as_slice() else {
                return Err(illegal("history policy length"));
            };
            if !(0..4).contains(ps) || !(0..4).contains(rs) || *pl < 0 || *rl < 0 {
                return Err(illegal("history policy range"));
            }
            let mut h = emoji::history::History {
                pinned: env.read(pinned, "pinned")?,
                recent: env.read(recent, "recent")?,
            };
            h.apply(
                emoji::history::Policy {
                    enabled: *enabled != 0,
                    pinned: *ps,
                    recent: *rs,
                    pinned_limit: *pl,
                    recent_limit: *rl,
                },
                *op,
                *value,
                *offset,
            );
            env.make_objects(&[&|| env.make(&h.pinned), &|| env.make(&h.recent)])
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelPolicyNative_arrange(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
    dynamic: jintArray,
    hidden: jintArray,
) -> jobjectArray {
    // SAFETY: JVM supplies local arrays, copied before running policy; no native handle escapes.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<i32>(config, "arrangement operations")?;
            if c.is_empty() || (c.len() - 1) % 3 != 0 {
                return Err(illegal("arrangement operations length"));
            }
            let mut a = Arrangement {
                sticky: c[0],
                dynamic: env.read(dynamic, "dynamic")?,
                hidden: env.read(hidden, "hidden")?,
            };
            let mut e = Editor::new(a.clone());
            for op in c[1..].as_chunks::<3>().0 {
                match op[0] {
                    0 => {
                        a.distinct();
                        e = Editor::new(a.clone());
                    }
                    1 => e.begin(op[1], op[2] as usize),
                    2 => e.over(op[1], op[2] as usize),
                    3 => e.complete(),
                    _ => return Err(illegal("unknown arrangement operation")),
                }
            }
            let saved = e.saved();
            env.make_objects(&[
                &|| env.make(&[saved.sticky]),
                &|| env.make(&saved.dynamic),
                &|| env.make(&saved.hidden),
            ])
        })
    }
}
