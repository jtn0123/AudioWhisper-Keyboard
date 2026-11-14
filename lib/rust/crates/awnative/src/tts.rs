//! Registry leases keep generator owners live across callbacks without a Java/native lock.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jbyteArray, jclass, jint, jlong, jobject, jstring, jvalue};
use sherpa_ffi::{SharedTts, Tts};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicI64, Ordering},
    },
};
static OWNERS: OnceLock<Mutex<HashMap<i64, SharedTts>>> = OnceLock::new();
static NEXT: AtomicI64 = AtomicI64::new(1);
fn owners() -> &'static Mutex<HashMap<i64, SharedTts>> {
    OWNERS.get_or_init(Mutex::default)
}
fn owner(id: i64) -> jni::Result<SharedTts> {
    owners()
        .lock()
        .map_err(|_| illegal("TTS registry poisoned"))?
        .get(&id)
        .cloned()
        .ok_or_else(|| illegal("TTS closed"))
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_TtsNative_create(
    env: *mut JNIEnv,
    _: jclass,
    dir: jstring,
    wav: jbyteArray,
) -> jlong {
    // SAFETY: Java arrays/strings are copied for this call; registry owns the returned token.
    unsafe {
        jni::run(env, 0, |e| {
            let dir = String::from_utf16(&e.read_string(dir, "model path")?)
                .map_err(|_| illegal("Invalid model path"))?;
            let bytes = e
                .read::<i8>(wav, "reference WAV")?
                .into_iter()
                .map(|v| v as u8)
                .collect::<Vec<_>>();
            let (pcm, rate) = reader::wave::pcm16(&bytes).map_err(illegal)?;
            let tts = Arc::new(Tts::new(&dir, pcm, rate).map_err(illegal)?);
            let id = NEXT.fetch_add(1, Ordering::Relaxed);
            owners()
                .lock()
                .map_err(|_| illegal("TTS registry poisoned"))?
                .insert(id, tts);
            Ok(id)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_TtsNative_rate(
    env: *mut JNIEnv,
    _: jclass,
    id: jlong,
) -> jint {
    // SAFETY: the call-local Arc retains the owner independently of concurrent close.
    unsafe { jni::run(env, 0, |_| Ok(owner(id)?.sample_rate())) }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_TtsNative_cancel(
    env: *mut JNIEnv,
    _: jclass,
    id: jlong,
) {
    // SAFETY: no raw handle is dereferenced; only a call-local Arc's atomic is changed.
    unsafe {
        jni::run(env, (), |_| {
            if let Some(tts) = owners()
                .lock()
                .map_err(|_| illegal("TTS registry poisoned"))?
                .get(&id)
                .cloned()
            {
                tts.cancel();
            }
            Ok(())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_TtsNative_close(
    env: *mut JNIEnv,
    _: jclass,
    id: jlong,
) {
    // SAFETY: removing a token is idempotent; active calls retain an Arc and model lease.
    unsafe {
        jni::run(env, (), |_| {
            let owned = owners()
                .lock()
                .map_err(|_| illegal("TTS registry poisoned"))?
                .remove(&id);
            if let Some(tts) = owned {
                tts.close();
            }
            Ok(())
        })
    }
}
// SAFETY: the exported signature matches TtsNative; the synchronous bridge validates every reference.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_TtsNative_generate(
    env: *mut JNIEnv,
    _: jclass,
    id: jlong,
    text: jstring,
    callback: jobject,
) {
    generate(env, id, text, callback, |tts, text, audio| {
        tts.generate(text, audio)
    })
}

/// Present only in the comparison APK; the normal JNI generator never selects a seed.
#[cfg(feature = "tts-fixture")]
// SAFETY: the test APK declares this exact JNI signature and uses the same validated bridge.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_reader_TtsComparison_generate(
    env: *mut JNIEnv,
    _: jclass,
    id: jlong,
    text: jstring,
    seed: jint,
    callback: jobject,
) {
    generate(env, id, text, callback, |tts, text, audio| {
        tts.generate_fixture(text, seed, audio)
    })
}

fn generate(
    env: *mut JNIEnv,
    id: jlong,
    text: jstring,
    callback: jobject,
    synthesize: impl FnOnce(&Tts, &str, &mut dyn FnMut(&[f32]) -> bool) -> Result<(), String>,
) {
    // SAFETY: callback/local references and Env are used only synchronously on this JNI thread.
    // Rust model/PCM leases outlive the callback, including blocking Android playback writes.
    unsafe {
        jni::run(env, (), |e| {
            let tts = owner(id)?;
            let text = String::from_utf16_lossy(&e.read_string(text, "utterance")?);
            let method = e.method(
                callback,
                "audio callback",
                (c"audio", c"(Ljava/nio/ByteBuffer;)Z"),
            )?;
            let mut error = None;
            let mut audio = |samples: &[f32]| match e.with_pcm_buffer(samples, |buffer| {
                e.call_boolean(method, &[jvalue { l: buffer }])
            }) {
                Ok(keep) => keep,
                Err(failure) => {
                    error = Some(failure);
                    false
                }
            };
            let result = synthesize(&tts, &text, &mut audio);
            if let Some(error) = error {
                return Err(error);
            }
            result.map_err(illegal)
        })
    }
}
