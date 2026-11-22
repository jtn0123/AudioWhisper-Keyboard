//! Owned C recognizers and voice detectors (sherpa-ffi), for `SherpaNative`, whose generated
//! entry points call these. No audio or model policy here.

use std::{ptr, sync::Mutex};

use jni_sys::{jobject, jobjectArray};
use sherpa_ffi::{Batch, Recognizer, Vad};

use crate::handle::{self, into_handle, locked};
use crate::jni::{Env, Result, illegal};

/// A model path as a Rust string.
fn path(units: &[u16]) -> Result<String> {
    String::from_utf16(units).map_err(|_| illegal("invalid path"))
}

/// Dictation's sample rate and chunk size (`dictation::segmentation`).
pub fn audio_config() -> Vec<i32> {
    vec![
        dictation::segmentation::SAMPLE_RATE as i32,
        dictation::segmentation::CHUNK_SAMPLES as i32,
    ]
}

/// A recognizer for the model in `directory`.
pub fn create_recognizer(directory: &[u16]) -> Result<i64> {
    Ok(into_handle(
        Recognizer::new(&path(directory)?).map_err(illegal)?,
    ))
}

pub fn free_recognizer(recognizer: i64) -> Result<()> {
    // SAFETY: the wrapper's write lock waits for every decode and cancel; it frees once.
    unsafe { handle::free::<Recognizer>(recognizer) };
    Ok(())
}

/// The recognizer behind `handle`.
fn recognizer<'a>(handle: i64) -> Result<&'a Recognizer> {
    // SAFETY: the wrapper's read lock keeps the recognizer alive while it calls.
    unsafe { handle::object(handle, "ASR") }
}

/// Stops the decode in progress, if any; only the recognizer's atomic is touched.
pub fn cancel(handle: i64) -> Result<()> {
    recognizer(handle)?.cancel();
    Ok(())
}

/// The text in `samples` of `pcm`. The recognizer's own mutex serializes decodes.
pub fn transcribe(env: &Env, handle: i64, pcm: jobject, samples: i32) -> Result<Vec<u16>> {
    let recognizer = recognizer(handle)?;
    env.with_direct_f32(pcm, samples, |audio| {
        recognizer.transcribe(audio).map_err(illegal)
    })
}

/// A voice detector for the model at `model`.
pub fn create_vad(model: &[u16]) -> Result<i64> {
    Ok(into_handle(Mutex::new(
        Vad::new(&path(model)?).map_err(illegal)?,
    )))
}

pub fn free_vad(vad: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper frees the detector exactly once.
    unsafe { handle::free::<Mutex<Vad>>(vad) };
    Ok(())
}

/// A batch as Kotlin's `SherpaVad` reads it: speech flag, end, segment count, each segment's
/// start and raw length, and the transition, as one `LongArray`; then each phrase's samples;
/// then the preview's, or null.
fn batch(env: &Env, batch: Batch) -> Result<jobjectArray> {
    let mut metadata = vec![
        i64::from(batch.detected),
        batch.end,
        batch.segments.len() as i64,
    ];
    for segment in &batch.segments {
        metadata.extend([segment.start, segment.raw_len as i64]);
    }
    metadata.push(i64::from(batch.transition));
    let meta = || env.make(&metadata);
    let audio: Vec<_> = (batch.segments.iter())
        .map(|segment| move || env.make(&segment.samples))
        .collect();
    let preview =
        || (batch.preview.as_ref()).map_or(Ok(ptr::null_mut()), |samples| env.make(samples));
    let mut makers: Vec<&dyn Fn() -> Result<jobject>> = vec![&meta];
    for make in &audio {
        makers.push(make);
    }
    makers.push(&preview);
    env.make_objects(&makers)
}

/// Mode 0 pushes `samples` of `pcm`; 1 stops keeping what is pending, and 2 stops dropping it.
/// The PCM is borrowed only until this returns.
pub fn vad(env: &Env, handle: i64, pcm: jobject, samples: i32, mode: i32) -> Result<jobjectArray> {
    // SAFETY: the synchronized wrapper keeps the detector alive and serializes stop and close.
    let mut vad = unsafe { locked::<Vad>(handle, "VAD")? };
    let result = match mode {
        0 => env.with_direct_f32(pcm, samples, |samples| vad.push(samples).map_err(illegal))?,
        1 | 2 => vad.stop(mode == 1).map_err(illegal)?,
        _ => return Err(illegal("unknown VAD mode")),
    };
    batch(env, result)
}
