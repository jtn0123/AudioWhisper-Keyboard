//! Input-only JNI boundary. Native owns all pointer, swipe and glide recognition state.
use crate::{
    handle,
    jni::illegal,
    render::{Session, strings},
};
use jni_sys::{
    JNIEnv, jboolean, jclass, jfloatArray, jint, jintArray, jlong, jlongArray, jobjectArray,
};
use render::{gesture_config::GestureConfig, touch::Motion};
use std::ptr;
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_gestures(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    values: jfloatArray,
    actions: jobjectArray,
) {
    // SAFETY: the synchronized owner keeps the session and copied local arrays alive.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", (), |s, env| {
            let config = GestureConfig::decode(
                &env.read::<f32>(values, "gesture config")?,
                strings(env, actions)?,
            )
            .ok_or_else(|| illegal("invalid gesture settings"))?;
            s.touch.configure(config);
            Ok(())
        });
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_longPress(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    pointer: jint,
    popup: jboolean,
    uninterrupted: jboolean,
) -> jintArray {
    // SAFETY: the synchronized owner keeps the session alive and owns the returned local array.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let actions: Vec<_> = s
                .touch
                .long_press(&s.scene, pointer, popup != 0, uninterrupted != 0)
                .into_iter()
                .flat_map(|a| [a.0, a.1, a.2])
                .collect();
            s.prepare_panel();
            env.make(&actions)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_RenderNative_touch(
    env: *mut JNIEnv,
    _: jclass,
    native: jlong,
    metadata: jintArray,
    times: jlongArray,
    coords: jfloatArray,
    aim: jlong,
) -> jintArray {
    // SAFETY: the wrapper owns the live handle and JVM arrays for this call; `aim` is 0 or a tap
    // trail its Kotlin owner keeps alive while it calls.
    unsafe {
        handle::with_locked::<Session, _>(env, native, "renderer", ptr::null_mut(), |s, env| {
            let meta = env.read::<i32>(metadata, "metadata")?;
            if meta.len() < 2 {
                return Err(illegal("missing action/index"));
            }
            // A key goes down: where this user aims, as of now (`aimed_key_at`).
            if matches!(meta[0], 0 | 5) {
                s.scene.aims = crate::suggest::tap_aim::key_shifts(aim, &s.scene.keys)?;
            }
            let event = Motion {
                action: meta[0],
                index: meta[1] as usize,
                ids: meta[2..].to_vec(),
                times: env.read(times, "times")?,
                coords: env.read(coords, "coords")?,
            };
            s.touch_time = event.times.last().copied().unwrap_or(0);
            let actions: Vec<_> = s
                .touch
                .event(&mut s.scene, &event)
                .into_iter()
                .flat_map(|a| [a.0, a.1, a.2])
                .collect();
            s.prepare_panel();
            env.make(&actions)
        })
    }
}
