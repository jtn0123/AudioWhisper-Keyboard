//! Pure policy probes used by JVM differential tests; production uses the same functions.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jclass, jfloatArray, jintArray, jlong};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_editing(
    env: *mut JNIEnv,
    _: jclass,
    codes: jintArray,
    hold: jlong,
    repeat: jlong,
    words: jlong,
) -> jintArray {
    // SAFETY: JVM owns local arrays for the call; every output is copied into a local array.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let codes = env.read::<i32>(codes, "codes")?;
            let out: Vec<_> = codes
                .iter()
                .flat_map(|&code| {
                    let r = render::editing::repeat_code(code, words != 0);
                    [
                        i32::from(render::editing::repeatable(code)),
                        r,
                        render::editing::hold_delay(code, hold) as i32,
                        render::editing::repeat_delay(r, repeat) as i32,
                        i32::from(render::editing::closes(code)),
                    ]
                })
                .collect();
            env.make(&out)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_cells(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: local arrays are copied; the returned reference belongs to the JVM.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = env.read::<f32>(config, "grid")?;
            if c.len() != 2 || c.iter().any(|v| !v.is_finite() || *v <= 0.0) {
                return Err(illegal("grid config"));
            }
            env.make(
                &render::panel_geometry::cells(c[0], c[1])
                    .iter()
                    .flat_map(|(x, w)| [*x, *w])
                    .collect::<Vec<_>>(),
            )
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_emojiChoices(
    env: *mut JNIEnv,
    _: jclass,
    tones: jintArray,
    tone: i32,
) -> jintArray {
    // SAFETY: JVM supplies the local primitive array; no native object or borrowed memory escapes.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let tones = env.read::<i32>(tones, "tones")?;
            let mut out = vec![render::panel_geometry::base(&tones, tone) as i32];
            out.extend(
                render::panel_geometry::variants(&tones, tone)
                    .iter()
                    .map(|i| *i as i32),
            );
            env.make(&out)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_selection(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
) -> jintArray {
    // SAFETY: JVM owns the local arrays; Rust returns a copied editor-operation plan.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c: [i32; 6] = env
                .read::<i32>(config, "selection")?
                .try_into()
                .map_err(|_| illegal("selection config"))?;
            env.make(&render::editing::selection(c))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_scroll(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM owns the input array and receives a copied primitive result.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c: [f32; 3] = env
                .read::<f32>(config, "scroll")?
                .try_into()
                .map_err(|_| illegal("scroll config"))?;
            let (duration, distance) = render::panel_scroll::decay(c[0], c[1]);
            env.make(&[
                duration as f32,
                distance,
                distance * c[1].signum() * render::panel_scroll::fraction(c[2]),
            ])
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_PanelRules_overflowAndHistoryMoves(
    env: *mut JNIEnv,
    _: jclass,
    config: jintArray,
) -> jintArray {
    // SAFETY: the JVM owns local primitive arrays; only a copied result is returned.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c: [i32; 5] = env
                .read::<i32>(config, "panel policy")?
                .try_into()
                .map_err(|_| illegal("panel policy config"))?;
            let arrangement = render::arrangement::Arrangement {
                dynamic: (0..c[0].max(0)).collect(),
                ..Default::default()
            };
            env.make(&[
                arrangement.overflow(c[1].max(0) as usize).len() as i32,
                i32::from(render::panel_geometry::history_moves(c[2] != 0, c[3], c[4])),
            ])
        })
    }
}
