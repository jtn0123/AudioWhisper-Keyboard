//! Resource/preview transport; never logs text or retains Java references.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{JNIEnv, jclass, jfloat, jintArray, jlong, jobjectArray};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_decorations(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    labels: jobjectArray,
    height: jfloat,
    flags: jintArray,
) {
    // SAFETY: synchronized wrapper owns the session and copied local strings for this call.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let labels = strings(env, labels)?;
            let flags = env.read::<i32>(flags, "decoration flags")?;
            if flags.len() != 2 {
                return Err(illegal("decoration flags"));
            }
            if labels.len() != 2 || !height.is_finite() || height <= 0.0 {
                return Err(illegal("decorations"));
            }
            let request = render::theme::Request {
                element: String::new(),
                mode: s.mode.clone(),
                shift: s.shift.clone(),
                code: 0,
                key_type: String::new(),
                selector: 0,
                auto_commit: false,
                popup_source: None,
            };
            if let (Some(theme), Some(text)) = (&mut s.theme, &mut s.text) {
                render::ime_decorations::prepare(
                    &mut s.scene,
                    theme,
                    text,
                    &request,
                    if render::ime_decorations::show_watermark(flags[0] != 0, flags[1] != 0) {
                        &labels[0]
                    } else {
                        ""
                    },
                    &labels[1],
                    height,
                );
            }
            Ok(())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ImeChromeRules_watermark(
    env: *mut JNIEnv,
    _: jclass,
    flags: jintArray,
) -> jni_sys::jboolean {
    // SAFETY: JVM local input is copied and the result is a primitive boolean.
    unsafe {
        crate::jni::run(env, 0, |env| {
            let flags = env.read::<i32>(flags, "watermark flags")?;
            if flags.len() != 2 {
                return Err(illegal("watermark flags"));
            }
            Ok(u8::from(render::ime_decorations::show_watermark(
                flags[0] != 0,
                flags[1] != 0,
            )))
        })
    }
}
