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
