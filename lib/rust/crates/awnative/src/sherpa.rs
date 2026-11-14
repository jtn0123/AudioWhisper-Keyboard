//! The one JVM boundary for owned C recognizers and VADs. No audio/model policy here.
use crate::{
    handle,
    jni::{self, Env, illegal},
};
use jni_sys::{JNIEnv, jclass, jint, jlong, jobject, jobjectArray, jstring};
use sherpa_ffi::{Batch, Recognizer, Vad};
use std::{ptr, sync::Mutex};
fn string(e: &Env, s: jstring) -> jni::Result<String> {
    String::from_utf16(&e.read_string(s, "model path")?).map_err(|_| illegal("invalid path"))
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_createRecognizer(
    env: *mut JNIEnv,
    _: jclass,
    dir: jstring,
) -> jlong {
    // SAFETY: the environment and strings belong to this JNI call; Kotlin owns the new allocation.
    unsafe {
        jni::run(env, 0, |e| {
            Ok(handle::into_handle(
                Recognizer::new(&string(e, dir)?).map_err(illegal)?,
            ))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_freeRecognizer(
    _: *mut JNIEnv,
    _: jclass,
    h: jlong,
) {
    // SAFETY: the wrapper's write lock waits for every decode and cancel; Cleaner frees once.
    unsafe { handle::free::<Recognizer>(h) }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_cancel(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
) {
    // SAFETY: wrapper read lock keeps owner live; only the native atomic is accessed.
    unsafe {
        jni::run(env, (), |_| {
            handle::object::<Recognizer>(h, "ASR")?.cancel();
            Ok(())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_transcribe(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    pcm: jobject,
    n: jint,
) -> jstring {
    // SAFETY: wrapper read lock keeps recognizer alive; native mutex serializes streams/decodes.
    unsafe {
        jni::run(env, ptr::null_mut(), |e| {
            let owner = handle::object::<Recognizer>(h, "ASR")?;
            e.with_direct_f32(pcm, n, |audio| {
                e.make_string(&owner.transcribe(audio).map_err(illegal)?)
            })
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_createVad(
    env: *mut JNIEnv,
    _: jclass,
    model: jstring,
) -> jlong {
    // SAFETY: JNI strings are copied; the synchronized wrapper owns the returned allocation.
    unsafe {
        jni::run(env, 0, |e| {
            Ok(handle::into_handle(Mutex::new(
                Vad::new(&string(e, model)?).map_err(illegal)?,
            )))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_freeVad(
    _: *mut JNIEnv,
    _: jclass,
    h: jlong,
) {
    // SAFETY: synchronized Cleaner/producer calls free the allocation exactly once.
    unsafe { handle::free::<Mutex<Vad>>(h) }
}
fn batch(e: &Env, b: Batch) -> jni::Result<jobjectArray> {
    let mut metadata = vec![i64::from(b.detected), b.end, b.segments.len() as i64];
    for segment in &b.segments {
        metadata.extend([segment.start, segment.raw_len as i64]);
    }
    metadata.push(i64::from(b.transition));
    let meta = || e.make(&metadata);
    let audio: Vec<_> = b
        .segments
        .iter()
        .map(|s| move || e.make(&s.samples))
        .collect();
    let preview = || {
        b.preview
            .as_ref()
            .map_or(Ok(ptr::null_mut()), |p| e.make(p))
    };
    let mut makers: Vec<&dyn Fn() -> jni::Result<jobject>> = vec![&meta];
    for make in &audio {
        makers.push(make);
    }
    makers.push(&preview);
    e.make_objects(&makers)
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_vad(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    pcm: jobject,
    n: jint,
    mode: jint,
) -> jobjectArray {
    // SAFETY: wrapper serializes stop/close with push. Direct PCM is borrowed only until return.
    unsafe {
        handle::with_locked::<Vad, _>(env, h, "VAD", ptr::null_mut(), |vad, e| {
            let b = match mode {
                0 => e.with_direct_f32(pcm, n, |samples| vad.push(samples).map_err(illegal))?,
                1 | 2 => vad.stop(mode == 1).map_err(illegal)?,
                _ => return Err(illegal("unknown VAD mode")),
            };
            batch(e, b)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SherpaNative_audioConfig(
    env: *mut JNIEnv,
    _: jclass,
) -> jni_sys::jintArray {
    // SAFETY: JVM environment for this call; the returned array owns its values.
    unsafe {
        jni::run(env, ptr::null_mut(), |e| {
            e.make(&[
                dictation::segmentation::SAMPLE_RATE as i32,
                dictation::segmentation::CHUNK_SAMPLES as i32,
            ])
        })
    }
}
