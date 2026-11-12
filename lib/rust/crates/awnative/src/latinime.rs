//! JNI entry points for LatinIME's dictionary engine, ported to Rust (the `latin` crate). They
//! stand in for libjni_latinime.so's: the same Java classes (`BinaryDictionary`,
//! `DicTraverseSession` and `ProximityInfo` in lib/latinime) and the same answers for valid calls.
//! Closed (0) dictionaries and sessions and malformed arrays throw Java exceptions instead.

mod suggestions;

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex};

use jni_sys::{JNI_TRUE, JNIEnv, jboolean, jbooleanArray, jclass, jint, jintArray, jlong};
use jni_sys::{jobjectArray, jstring};
use latin::defines::{MAX_WORD_LENGTH, NOT_A_PROBABILITY};
use latin::layout::{KeyboardGeometry, ProximityInfo};
use latin::search::DicTraverseSession;
use latin::trie::dictionary::Dictionary;
use latin::trie::jni::get_words;
use latin::trie::ngram::NgramContext;

use crate::handle::{free, into_handle, object};
use crate::jni::{self, Env, Result};

// Kotlin may call from any thread: the dictionary and the keyboard are only read, and a session
// is used under its lock.
const _: fn() = || {
    fn shared<T: Send + Sync>() {}
    shared::<Dictionary>();
    shared::<ProximityInfo>();
    shared::<Mutex<DicTraverseSession>>();
};

/// The bytes `[offset, offset + size)` of the file at `path`, which may be an uncompressed entry
/// inside an APK.
fn read_range(path: &str, offset: u64, size: usize) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; size];
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// `BinaryDictionary.openNative`: the dictionary at `[offset, offset + size)` of `sourceDir`, or 0
/// if it can't be read or isn't a dictionary of a ported format. The bytes are read into memory
/// where the C++ maps them. `isUpdatable` is ignored: the app opens only the read-only bundled
/// dictionary, and the version 2 format the port reads is never updated in place.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_BinaryDictionary_openNative(
    env: *mut JNIEnv,
    _: jclass,
    source_dir: jstring,
    offset: jlong,
    size: jlong,
    _is_updatable: jboolean,
) -> jlong {
    // SAFETY: the JVM passes this call's environment.
    unsafe {
        jni::run(env, 0, |env| {
            let Some(path) = env.read_nullable_string(source_dir)? else {
                return Ok(0);
            };
            let (Ok(offset), Ok(size)) = (u64::try_from(offset), usize::try_from(size)) else {
                return Ok(0);
            };
            let Ok(bytes) = read_range(&String::from_utf16_lossy(&path), offset, size) else {
                return Ok(0);
            };
            Ok(Dictionary::open(Arc::from(bytes)).map_or(0, into_handle))
        })
    }
}

/// `BinaryDictionary.closeNative`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_BinaryDictionary_closeNative(
    _: *mut JNIEnv,
    _: jclass,
    dict: jlong,
) {
    // SAFETY: the handle came from `openNative`, and `BinaryDictionary.close` frees it once.
    unsafe { free::<Dictionary>(dict) }
}

/// Runs `body` on the dictionary behind `dict`, throwing for a closed handle. On an error,
/// `fallback` is returned and Java observes the exception instead.
///
/// # Safety
/// `env` must be this call's environment, and `dict` 0 or a live handle from `openNative`.
unsafe fn with_dictionary<R: Copy>(
    env: *mut JNIEnv,
    dict: jlong,
    fallback: R,
    body: impl FnOnce(&Dictionary, &Env) -> Result<R>,
) -> R {
    // SAFETY: as the caller promises.
    unsafe { jni::run(env, fallback, |env| body(object(dict, "dictionary")?, env)) }
}

/// `BinaryDictionary.getProbabilityNative`: the word's probability, or `NOT_A_PROBABILITY`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_BinaryDictionary_getProbabilityNative(
    env: *mut JNIEnv,
    _: jclass,
    dict: jlong,
    word: jintArray,
) -> jint {
    // SAFETY: the JVM passes this call's environment, and `dict` comes from `openNative`.
    unsafe {
        with_dictionary(env, dict, NOT_A_PROBABILITY, |dictionary, env| {
            Ok(dictionary.probability(&env.read::<i32>(word, "word")?))
        })
    }
}

/// `BinaryDictionary.getNgramProbabilityNative`: the word's probability after the previous words
/// (most recent first, one per element of the arrays).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_BinaryDictionary_getNgramProbabilityNative(
    env: *mut JNIEnv,
    _: jclass,
    dict: jlong,
    prev_words: jobjectArray,
    beginning_of_sentence: jbooleanArray,
    word: jintArray,
) -> jint {
    // SAFETY: the JVM passes this call's environment, and `dict` comes from `openNative`.
    unsafe {
        with_dictionary(env, dict, 0, |dictionary, env| {
            let word = env.read::<i32>(word, "word")?;
            let prev_words = env.read_arrays::<i32>(prev_words, "prevWordCodePointArrays")?;
            let context = ngram_context(env, &prev_words, beginning_of_sentence)?;
            Ok(dictionary.ngram_probability(Some(&context), &word))
        })
    }
}

/// `JniDataUtils::constructNgramContext` for every previous word given: a flag is read only for a
/// word that is there, so the flags array may be null when there are none.
fn ngram_context(
    env: &Env,
    prev_words: &[Option<Vec<i32>>],
    beginning_of_sentence: jbooleanArray,
) -> Result<NgramContext> {
    let words: Vec<Option<&[i32]>> = prev_words.iter().map(Option::as_deref).collect();
    let flags = flags(env, prev_words, beginning_of_sentence)?;
    Ok(NgramContext::new(&words, &flags))
}

/// The beginning-of-sentence flag of each previous word: false for a missing or too long one, whose
/// flag the C++ doesn't read (so the array is read only if some word needs it).
fn flags(
    env: &Env,
    prev_words: &[Option<Vec<i32>>],
    beginning_of_sentence: jbooleanArray,
) -> Result<Vec<bool>> {
    let needs_flag =
        |word: &Option<Vec<i32>>| word.as_ref().is_some_and(|w| w.len() <= MAX_WORD_LENGTH);
    if !prev_words.iter().any(needs_flag) {
        return Ok(vec![false; prev_words.len()]);
    }
    let read = env.read::<jboolean>(beginning_of_sentence, "isBeginningOfSentenceArray")?;
    prev_words
        .iter()
        .enumerate()
        .map(|(i, word)| match (needs_flag(word), read.get(i)) {
            (false, _) => Ok(false),
            (true, Some(&flag)) => Ok(flag == JNI_TRUE),
            (true, None) => Err(jni::illegal("isBeginningOfSentenceArray is too short")),
        })
        .collect()
}

/// `BinaryDictionary.getWordsNative`: every word, each followed by a newline.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_BinaryDictionary_getWordsNative(
    env: *mut JNIEnv,
    _: jclass,
    dict: jlong,
) -> jstring {
    // SAFETY: the JVM passes this call's environment, and `dict` comes from `openNative`.
    unsafe {
        with_dictionary(env, dict, std::ptr::null_mut(), |dictionary, env| {
            env.make_string(&get_words(dictionary))
        })
    }
}

/// `DicTraverseSession.setDicTraverseSessionNative`: a new session for a dictionary of
/// `dictSize` bytes. The locale is ignored: the search takes it from the dictionary's header.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_DicTraverseSession_setDicTraverseSessionNative(
    _: *mut JNIEnv,
    _: jclass,
    _locale: jstring,
    dict_size: jlong,
) -> jlong {
    let size = usize::try_from(dict_size).unwrap_or(0);
    into_handle(Mutex::new(DicTraverseSession::new(size)))
}

/// `DicTraverseSession.initDicTraverseSessionNative`: nothing to do, as every search starts by
/// initialising its session for the dictionary and the previous words it is given.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_DicTraverseSession_initDicTraverseSessionNative(
    _: *mut JNIEnv,
    _: jclass,
    _session: jlong,
    _dict: jlong,
    _previous_word: jintArray,
    _previous_word_length: jint,
) {
}

/// `DicTraverseSession.releaseDicTraverseSessionNative`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_DicTraverseSession_releaseDicTraverseSessionNative(
    _: *mut JNIEnv,
    _: jclass,
    session: jlong,
) {
    // SAFETY: the handle came from `setDicTraverseSessionNative`, and `close` frees it once.
    unsafe { free::<Mutex<DicTraverseSession>>(session) }
}

/// `ProximityInfo.setProximityInfoNative`: the keyboard, or 0 if its grid or arrays don't fit
/// together. The sweet spots, which the app passes as null, are ignored.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_keyboard_ProximityInfo_setProximityInfoNative(
    env: *mut JNIEnv,
    _: jclass,
    keyboard_width: jint,
    keyboard_height: jint,
    grid_width: jint,
    grid_height: jint,
    most_common_key_width: jint,
    most_common_key_height: jint,
    proximity_chars: jintArray,
    key_count: jint,
    key_xs: jintArray,
    key_ys: jintArray,
    key_widths: jintArray,
    key_heights: jintArray,
    key_char_codes: jintArray,
    _sweet_spot_xs: jni_sys::jfloatArray,
    _sweet_spot_ys: jni_sys::jfloatArray,
    _sweet_spot_radii: jni_sys::jfloatArray,
) -> jlong {
    // SAFETY: the JVM passes this call's environment.
    unsafe {
        jni::run(env, 0, |env| {
            let proximity_chars = env.read::<i32>(proximity_chars, "proximityCharsArray")?;
            let key_xs = env.read::<i32>(key_xs, "keyXCoordinates")?;
            let key_ys = env.read::<i32>(key_ys, "keyYCoordinates")?;
            let key_widths = env.read::<i32>(key_widths, "keyWidths")?;
            let key_heights = env.read::<i32>(key_heights, "keyHeights")?;
            let key_char_codes = env.read::<i32>(key_char_codes, "keyCharCodes")?;
            let geometry = KeyboardGeometry {
                keyboard_width,
                keyboard_height,
                grid_width,
                grid_height,
                most_common_key_width,
                most_common_key_height,
                proximity_chars: &proximity_chars,
                key_count,
                key_x_coordinates: &key_xs,
                key_y_coordinates: &key_ys,
                key_widths: &key_widths,
                key_heights: &key_heights,
                key_char_codes: &key_char_codes,
            };
            Ok(ProximityInfo::new(&geometry).map_or(0, into_handle))
        })
    }
}

/// `ProximityInfo.releaseProximityInfoNative`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_keyboard_ProximityInfo_releaseProximityInfoNative(
    _: *mut JNIEnv,
    _: jclass,
    info: jlong,
) {
    // SAFETY: the handle came from `setProximityInfoNative`, and `close` frees it once.
    unsafe { free::<ProximityInfo>(info) }
}
