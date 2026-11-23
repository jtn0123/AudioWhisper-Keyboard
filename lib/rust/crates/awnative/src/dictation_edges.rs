//! The microphone's audio rules and a model's file checks (`dictation::audio`,
//! `dictation::model_check`) for `DictationEdgesNative`, whose generated entry points call these.

use dictation::{audio, model_check};

use crate::jni::{Result, illegal};

/// How loud the first `count` samples are (`audio::level`).
pub fn level(samples: &[f32], count: i32) -> Result<f32> {
    let count = usize::try_from(count)
        .ok()
        .filter(|&n| n <= samples.len())
        .ok_or_else(|| illegal(format!("{count} of {} samples", samples.len())))?;
    Ok(audio::level(&samples[..count]))
}

/// What a read of `count` means (`audio::read`).
pub fn read(count: i32, running: bool) -> Result<i32> {
    Ok(audio::read(count, running))
}

/// A WAV file's samples (`audio::pcm16`); its error throws as Kotlin's did.
pub fn pcm16(wav: &[i8]) -> Result<Vec<f32>> {
    let bytes: Vec<u8> = wav.iter().map(|&b| b as u8).collect();
    audio::pcm16(&bytes).map_err(|e| illegal(e.message()))
}

/// The mic's buffer (`audio::buffer_bytes`).
pub fn buffer_bytes(minimum: i32) -> Result<i32> {
    Ok(audio::buffer_bytes(minimum))
}

/// A `BooleanArray` as flags.
fn flags(values: &[u8]) -> Vec<bool> {
    values.iter().map(|&v| v != 0).collect()
}

/// Fails unless every array is as long as the first.
fn same_length(lengths: &[usize]) -> Result<()> {
    if lengths.iter().all(|&n| n == lengths[0]) {
        Ok(())
    } else {
        Err(illegal(format!("arrays of lengths {lengths:?}")))
    }
}

/// A quick check (`model_check::quick`).
pub fn quick(expected: &[i64], present: &[u8], lengths: &[i64]) -> Result<Vec<i32>> {
    same_length(&[expected.len(), present.len(), lengths.len()])?;
    Ok(model_check::quick(expected, &flags(present), lengths).row())
}

/// What each file is remembered by (`model_check::key`).
pub fn keys(paths: &[Vec<u16>], lengths: &[i64], modified: &[i64]) -> Result<Vec<Vec<u16>>> {
    same_length(&[paths.len(), lengths.len(), modified.len()])?;
    Ok((0..paths.len())
        .map(|i| model_check::key(&paths[i], lengths[i], modified[i]))
        .collect())
}

/// The files to hash (`model_check::to_hash`).
pub fn to_hash(known: &[u8]) -> Result<Vec<i32>> {
    Ok(model_check::to_hash(&flags(known))
        .into_iter()
        .map(|i| i as i32)
        .collect())
}

/// A full check's verdict (`model_check::verdict`): the state answer, then the files to remember.
pub fn verdict(known: &[u8], expected: &[Vec<u16>], sums: &[Option<Vec<u16>>]) -> Result<Vec<i32>> {
    same_length(&[known.len(), expected.len(), sums.len()])?;
    let (state, passed) = model_check::verdict(&flags(known), expected, sums);
    Ok(state
        .row()
        .into_iter()
        .chain(passed.into_iter().map(|i| i as i32))
        .collect())
}
