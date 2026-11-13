//! Smartbar packets and the font-independent JVM differential-test boundary.
use crate::{
    handle,
    jni::{self, illegal},
};
use jni_sys::{JNIEnv, jclass, jfloatArray, jintArray, jlong, jobjectArray};
use render::{
    smartbar::{self, Config},
    smartbar_scene::{Action, Smartbar},
};
use std::ptr;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_smartbar(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    config: jfloatArray,
    codes: jintArray,
    labels: jobjectArray,
) {
    // SAFETY: synchronized Kotlin owner keeps the handle and local arrays alive.
    unsafe {
        handle::with_locked::<crate::render::Session, _>(env, native, "renderer", (), |s, env| {
            let config = Config::decode(&env.read::<f32>(config, "config")?)
                .ok_or_else(|| illegal("invalid smartbar config"))?;
            let codes = env.read::<i32>(codes, "codes")?;
            let labels = crate::render::strings(env, labels)?;
            if codes.len() % 2 != 0
                || labels.len() < codes.len()
                || (labels.len() - codes.len()) % 2 != 0
                || codes.len() / 2 != 5 + config.dynamic
            {
                return Err(illegal("invalid smartbar actions"));
            }
            let actions = codes
                .as_chunks::<2>()
                .0
                .iter()
                .enumerate()
                .map(|(i, c)| Action {
                    code: c[0],
                    enabled: c[1] & 1 != 0,
                    lit: if c[1] & 2 != 0 {
                        1
                    } else if c[1] & 4 != 0 {
                        2
                    } else {
                        0
                    },
                    level: (c[1] >> 8 & 255) as f32 / 255.0,
                    warming: c[1] & 8 != 0,
                    finishing: c[1] & 16 != 0,
                    icon: labels[i * 2].clone(),
                    label: labels[i * 2 + 1].clone(),
                    styles: Default::default(),
                })
                .collect();
            let mut bar = Smartbar::new(config, actions);
            bar.aux = labels[codes.len()..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|a| (a[0].clone(), a[1].clone()))
                .collect();
            if let Some(old) = &s.scene.smartbar {
                bar.pressed = old.pressed.clone();
                bar.scroll = old.scroll;
                bar.phase = old.phase;
            }
            s.scene.smartbar = Some(bar);
            Ok(())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SmartbarRules_layout(
    env: *mut JNIEnv,
    _: jclass,
    config: jfloatArray,
    width: f32,
    intrinsic: jfloatArray,
) -> jfloatArray {
    // SAFETY: JVM local arrays are valid for this invocation and copied before use.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let c = Config::decode(&env.read::<f32>(config, "config")?)
                .ok_or_else(|| illegal("invalid smartbar config"))?;
            let intrinsic = env.read::<f32>(intrinsic, "intrinsic")?;
            if !width.is_finite()
                || width <= 0.0
                || intrinsic.iter().any(|v| !v.is_finite() || *v < 0.0)
            {
                return Err(illegal("invalid smartbar measurement"));
            }
            env.make(&smartbar::layout(&c, width, &intrinsic, 0.0).packed(&c))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SmartbarRules_ellipsis(
    env: *mut JNIEnv,
    _: jclass,
    advances: jfloatArray,
    width: f32,
) -> i32 {
    // SAFETY: JVM local array is copied within the invocation.
    unsafe {
        jni::run(env, 0, |env| {
            Ok(smartbar::ellipsis(&env.read::<f32>(advances, "advances")?, width) as i32)
        })
    }
}
