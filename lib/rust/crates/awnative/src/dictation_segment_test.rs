//! Host-only differential transport for the policy independent of sherpa inference.
use crate::{
    handle,
    jni::{self, illegal},
};
use dictation::segmentation::Segmentation;
use jni_sys::{JNIEnv, jboolean, jclass, jfloatArray, jint, jlong};
use std::{ptr, sync::Mutex};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SegmentationNative_create(
    env: *mut JNIEnv,
    _: jclass,
) -> jlong {
    // SAFETY: JVM call environment; test wrapper owns the new allocation.
    unsafe {
        jni::run(env, 0, |_| {
            Ok(handle::into_handle(Mutex::new(Segmentation::default())))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SegmentationNative_free(
    _: *mut JNIEnv,
    _: jclass,
    h: jlong,
) {
    // SAFETY: test wrapper frees exactly once after its calls finish.
    unsafe { handle::free::<Mutex<Segmentation>>(h) }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SegmentationNative_call(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    op: jint,
    audio: jfloatArray,
    from: jlong,
    until: jlong,
    detected: jboolean,
) -> jfloatArray {
    // SAFETY: test wrapper keeps handle live; PCM is copied before applying policy.
    unsafe {
        jni::run(env, ptr::null_mut(), |e| {
            let mut s = handle::locked::<Segmentation>(h, "segmentation")?;
            let audio = e.read::<f32>(audio, "audio")?;
            let result = match op {
                0 => {
                    s.add(&audio);
                    Vec::new()
                }
                1 => s.phrase(from, &audio),
                2 => s.preview(detected != 0).unwrap_or_default(),
                3 => s.slice(from, until),
                _ => return Err(illegal("unknown segmentation event")),
            };
            e.make(&result)
        })
    }
}
