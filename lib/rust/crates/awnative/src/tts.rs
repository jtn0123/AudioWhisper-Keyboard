//! The read-aloud voice (sherpa-ffi's `Tts`), for `TtsNative`, whose generated entry points call
//! these. A handle is a token in a registry of leases, which keep a voice alive across its
//! callbacks without a Java or native lock: closing removes the token, and a call in progress
//! keeps its own lease.
use crate::jni::{Env, Result, illegal};
use jni_sys::{jobject, jvalue};
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
fn owner(id: i64) -> Result<SharedTts> {
    owners()
        .lock()
        .map_err(|_| illegal("TTS registry poisoned"))?
        .get(&id)
        .cloned()
        .ok_or_else(|| illegal("TTS closed"))
}
/// A voice for the model in `directory`, speaking like `reference_wav`, a 16-bit WAV file.
pub fn create(directory: &[u16], reference_wav: &[i8]) -> Result<i64> {
    let directory = String::from_utf16(directory).map_err(|_| illegal("Invalid model path"))?;
    let bytes: Vec<u8> = reference_wav.iter().map(|&byte| byte as u8).collect();
    let (pcm, rate) = reader::wave::pcm16(&bytes).map_err(illegal)?;
    let tts = Arc::new(Tts::new(&directory, pcm, rate).map_err(illegal)?);
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    owners()
        .lock()
        .map_err(|_| illegal("TTS registry poisoned"))?
        .insert(id, tts);
    Ok(id)
}

/// Samples a second in what the voice makes.
pub fn rate(id: i64) -> Result<i32> {
    Ok(owner(id)?.sample_rate())
}

/// Stops what the voice is speaking; nothing for a closed one.
pub fn cancel(id: i64) -> Result<()> {
    let tts = (owners().lock())
        .map_err(|_| illegal("TTS registry poisoned"))?
        .get(&id)
        .cloned();
    if let Some(tts) = tts {
        tts.cancel();
    }
    Ok(())
}

/// Removes the token; a call in progress keeps its lease until it returns.
pub fn close(id: i64) -> Result<()> {
    let owned = (owners().lock())
        .map_err(|_| illegal("TTS registry poisoned"))?
        .remove(&id);
    if let Some(tts) = owned {
        tts.close();
    }
    Ok(())
}

/// Speaks `text`, handing `callback` each piece of audio until it returns false.
pub fn generate(env: &Env, id: i64, text: &[u16], callback: jobject) -> Result<()> {
    speak(env, id, text, callback, |tts, text, audio| {
        tts.generate(text, audio)
    })
}

/// Present only in the comparison APK; the normal JNI generator never selects a seed.
#[cfg(feature = "tts-fixture")]
// SAFETY: the test APK declares this exact JNI signature and uses the same validated bridge.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_reader_TtsComparison_generate(
    env: *mut jni_sys::JNIEnv,
    _: jni_sys::jclass,
    id: jni_sys::jlong,
    text: jni_sys::jstring,
    seed: jni_sys::jint,
    callback: jobject,
) {
    // SAFETY: the environment and references belong to this JNI call; the text is copied.
    unsafe {
        crate::jni::run(env, (), |env| {
            let text = env.read_string(text, "utterance")?;
            speak(env, id, &text, callback, |tts, text, audio| {
                tts.generate_fixture(text, seed, audio)
            })
        })
    }
}

/// Runs `synthesize` on the voice, handing `callback` each piece of audio in a direct buffer over
/// the samples, synchronously on this JNI thread. The lease outlives every callback, including
/// Android's blocking playback writes.
fn speak(
    env: &Env,
    id: i64,
    text: &[u16],
    callback: jobject,
    synthesize: impl FnOnce(
        &Tts,
        &str,
        &mut dyn FnMut(&[f32]) -> bool,
    ) -> std::result::Result<(), String>,
) -> Result<()> {
    let tts = owner(id)?;
    let text = String::from_utf16_lossy(text);
    let method = env.method(
        callback,
        "audio callback",
        (c"audio", c"(Ljava/nio/ByteBuffer;)Z"),
    )?;
    let mut error = None;
    let mut audio = |samples: &[f32]| match env.with_pcm_buffer(samples, |buffer| {
        env.call_boolean(method, &[jvalue { l: buffer }])
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
}
