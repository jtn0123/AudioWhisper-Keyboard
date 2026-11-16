//! Thin JNI for the in-process Latin session. No callbacks run while the session is locked.
use super::{
    non_null,
    suggester::{stores, write_strip},
};
use crate::{
    files::read_range,
    handle::{free, into_handle, locked},
    jni::{self, Env, Result, illegal},
};
use jni_sys::{
    JNIEnv, jboolean, jbyteArray, jclass, jfloat, jfloatArray, jint, jintArray, jlong,
    jobjectArray, jstring,
};
use std::{
    ptr,
    sync::{Arc, Mutex},
};
use suggest::{
    auto_correction::Strength,
    previous_words::PreviousWords,
    raw::RawSuggestion,
    session::{Error as SessionError, Session},
    suggester::{Suggester, TypingState},
    tap::TapOffset,
};

impl From<SessionError> for jni::Error {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::CannotRoundNaN => illegal("Cannot round NaN value."),
            SessionError::NegativeCount(n) => suggest::learned_words::NegativeCount(n).into(),
        }
    }
}

fn previous(env: &Env, words: jobjectArray, bos: jboolean) -> Result<PreviousWords> {
    Ok(PreviousWords {
        words: non_null(env.read_strings(words, "previousWords")?)?,
        is_beginning_of_sentence: bos != 0,
    })
}
fn taps(env: &Env, array: jfloatArray) -> Result<Vec<Option<TapOffset>>> {
    let values = env.read::<f32>(array, "taps")?;
    let (chunks, rest) = values.as_chunks::<3>();
    if !rest.is_empty() {
        return Err(illegal("three values per tap"));
    }
    Ok(chunks
        .iter()
        .map(|v| (v[0] != 0.0).then_some(TapOffset { dx: v[1], dy: v[2] }))
        .collect())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_open(
    env: *mut JNIEnv,
    _: jclass,
    bytes: jbyteArray,
) -> jlong {
    // SAFETY: the JVM provides the call's environment and array; bytes are copied before use.
    unsafe {
        jni::run(env, 0, |env| {
            let bytes: Vec<u8> = env
                .read::<i8>(bytes, "dictionary")?
                .into_iter()
                .map(|b| b as u8)
                .collect();
            Ok(Session::open(Arc::from(bytes)).map_or(0, |s| into_handle(Mutex::new(s))))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_free(
    _: *mut JNIEnv,
    _: jclass,
    handle: jlong,
) {
    // SAFETY: the Kotlin cleaner owns this allocation and releases it exactly once under its wrapper lock.
    unsafe { free::<Mutex<Session>>(handle) }
}
/// Reads the n-gram model at `[offset, offset + length)` of the file at `path` (an uncompressed
/// APK entry) into the session. False, with the session left as it was, if it can't be read or
/// isn't a model.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_attachModel(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
    path: jstring,
    offset: jlong,
    length: jlong,
) -> jboolean {
    // SAFETY: the wrapper holds its live handle under the same monitor as close; the JVM provides env.
    unsafe {
        jni::run(env, 0, |env| {
            let path = String::from_utf16_lossy(&env.read_string(path, "model path")?);
            let (Ok(offset), Ok(length)) = (u64::try_from(offset), usize::try_from(length)) else {
                return Ok(0);
            };
            let Ok(bytes) = read_range(&path, offset, length) else {
                return Ok(0);
            };
            let Some(model) = ngram::Model::open(bytes) else {
                return Ok(0);
            };
            locked::<Session>(handle, "session")?.attach(Arc::new(model));
            Ok(1)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_probability(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
    word: jstring,
) -> jint {
    // SAFETY: the wrapper holds its live handle under the same monitor as close; the JVM provides env.
    unsafe {
        jni::run(env, -1, |env| {
            Ok(locked::<Session>(handle, "session")?.probability(&env.read_string(word, "word")?))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_ngram(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
    word: jstring,
    words: jobjectArray,
    bos: jboolean,
) -> jint {
    // SAFETY: the wrapper holds its live handle under the same monitor as close; the JVM provides env.
    unsafe {
        jni::run(env, -1, |env| {
            Ok(locked::<Session>(handle, "session")?
                .ngram_probability(&previous(env, words, bos)?, &env.read_string(word, "word")?))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_words(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
) -> jstring {
    // SAFETY: the wrapper holds its live handle under the same monitor as close; the JVM provides env.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let words = locked::<Session>(handle, "session")?.words();
            env.make_string(&words.concat())
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_wordLengths(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
) -> jintArray {
    // SAFETY: the wrapper holds its live handle under the same monitor as close; the JVM provides env.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let sizes: Vec<i32> = locked::<Session>(handle, "session")?
                .words()
                .iter()
                .map(|w| w.len() as i32)
                .collect();
            env.make(&sizes)
        })
    }
}

fn pack(results: &[RawSuggestion]) -> Vec<i32> {
    let mut packed = Vec::new();
    for r in results {
        packed.extend([r.score, r.kind_and_flags, r.word.len() as i32]);
        packed.extend(r.word.iter().map(|&u| i32::from(u)));
    }
    packed
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_decode(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
    input: jstring,
    words: jobjectArray,
    bos: jboolean,
    block: jboolean,
    offsets: jfloatArray,
) -> jintArray {
    // SAFETY: the wrapper holds its live handle under the same monitor as close; the JVM provides env.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let results = locked::<Session>(handle, "session")?.decode(
                &env.read_string(input, "input")?,
                &previous(env, words, bos)?,
                block != 0,
                &taps(env, offsets)?,
            )?;
            env.make(&pack(&results))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_suggest(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
    personal: jlong,
    learned: jlong,
    typed: jstring,
    words: jobjectArray,
    bos: jboolean,
    mid_word: jboolean,
    block: jboolean,
    offsets: jfloatArray,
    max_count: jint,
    strength: jint,
    marks: jintArray,
) -> jobjectArray {
    // SAFETY: Kotlin holds the session and both stores for the call, serializing close with the wrapper monitor.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (personal, learned) = stores(personal, learned)?;
            let state = TypingState {
                typed: env.read_string(typed, "typed")?,
                previous_words: previous(env, words, bos)?,
                is_cursor_mid_word: mid_word != 0,
            };
            let offsets = taps(env, offsets)?;
            let mut session = locked::<Session>(handle, "session")?;
            let mut dictionary = session.request(&state.previous_words, block != 0, &offsets);
            let strip = Suggester {
                dictionary: &mut dictionary,
                personal,
                learned,
                threshold: Strength::from_ordinal(strength).threshold(),
            }
            .suggest(&state, max_count)?;
            write_strip(env, &strip, marks)
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LatinSessionNative_spell(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
    personal: jlong,
    learned: jlong,
    word: jstring,
    block: jboolean,
    max_count: jint,
    threshold: jfloat,
    verdict: jintArray,
) -> jobjectArray {
    // SAFETY: Kotlin holds the session and both stores for the call, serializing close with the wrapper monitor.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (personal, learned) = stores(personal, learned)?;
            let word = env.read_string(word, "word")?;
            let mut session = locked::<Session>(handle, "session")?;
            let previous = PreviousWords::NONE;
            let mut dictionary = session.request(&previous, block != 0, &[]);
            let check = Suggester {
                dictionary: &mut dictionary,
                personal,
                learned,
                threshold: Some(threshold),
            }
            .spell(&word, max_count)?;
            env.write(
                verdict,
                &[
                    i32::from(check.is_valid),
                    i32::from(check.is_high_confidence),
                ],
                "verdict",
            )?;
            env.make_strings(&check.suggestions)
        })
    }
}

/// Test APKs/host tests alone can inspect untouched native slots. This symbol is absent from shipped Android libraries.
#[cfg(any(not(target_os = "android"), feature = "latin-fixture"))]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_suggest_A1Probe_snapshot(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
) -> jintArray {
    // SAFETY: the test holds a live session wrapper while reading its handle, with no concurrent close.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let session = locked::<Session>(handle, "session")?;
            let o = session.output();
            let mut result = vec![o.count];
            for array in [&o.code_points, &o.scores, &o.space_indices, &o.types] {
                result.extend(array);
            }
            result.extend([o.auto_commit_confidence, o.weight.to_bits() as i32]);
            env.make(&result)
        })
    }
}
