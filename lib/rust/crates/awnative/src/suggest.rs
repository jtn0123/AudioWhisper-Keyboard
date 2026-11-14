//! JNI entry points for the typing suggestions' rules (the `suggest` crate), called by
//! `SuggestNative` in lib/rust's Kotlin. Results that aren't one value come back packed, as each
//! function says; `SuggestRules` unpacks them.

mod kept_words;
mod learned_words;
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
mod legacy_suggester;
mod native_glide;
mod personal_words;
mod session;
mod suggester;
mod swipe;
mod tap;
mod word_learner;

use std::ptr;

use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jclass, jfloat, jint, jintArray};
use jni_sys::{jobjectArray, jstring};
use suggest::learned_words::NegativeCount;
use suggest::raw::RawSuggestion;
use suggest::word_case::{self, WordCase};
use suggest::{auto_correction, previous_words, undo, word_characters};

use crate::jni::{self, Error, Result, boolean, illegal};

/// The strings of a `String[]` that may hold no null.
fn non_null(strings: Vec<Option<Vec<u16>>>) -> Result<Vec<Vec<u16>>> {
    strings
        .into_iter()
        .map(|string| string.ok_or(Error::NullPointer("a word")))
        .collect()
}

/// Kotlin's `take` of a negative count throws this.
impl From<NegativeCount> for Error {
    fn from(NegativeCount(count): NegativeCount) -> Error {
        illegal(format!(
            "Requested element count {count} is less than zero."
        ))
    }
}

/// The cases' numbers, as `SuggestRules.Case` orders them.
const CASES: [WordCase; 3] = [WordCase::AsTyped, WordCase::Capitalized, WordCase::AllCaps];

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_isCursorMidWord(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jboolean {
    // SAFETY: the JVM provides this call's environment and string.
    unsafe {
        jni::run(env, 0, |env| {
            Ok(boolean(word_characters::cursor_mid_word(
                &env.read_string(text, "textAfterCursor")?,
            )))
        })
    }
}

/// Whether typing `codePoint` after a word continues it (`word_characters::continues_word`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_continuesWord(
    _: *mut JNIEnv,
    _: jclass,
    code_point: jint,
) -> jboolean {
    boolean(word_characters::continues_word(code_point))
}

/// The case of `typed`, as its number in [`CASES`].
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_wordCase(
    env: *mut JNIEnv,
    _: jclass,
    typed: jstring,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let case = WordCase::of(&env.read_string(typed, "typed")?);
            // There are three cases.
            Ok(CASES.iter().position(|&c| c == case).unwrap_or(0) as jint)
        })
    }
}

/// `word` in the case numbered `case`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_applyCase(
    env: *mut JNIEnv,
    _: jclass,
    case: jint,
    word: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let case = usize::try_from(case).ok().and_then(|i| CASES.get(i));
            let case = case.ok_or_else(|| illegal("no such case"))?;
            env.make_string(&case.apply(&env.read_string(word, "word")?))
        })
    }
}

/// Whether `typed` has more than one capital (`word_case::is_mostly_caps`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_isMostlyCaps(
    env: *mut JNIEnv,
    _: jclass,
    typed: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            Ok(boolean(word_case::is_mostly_caps(
                &env.read_string(typed, "typed")?,
            )))
        })
    }
}

/// `typed` as the decoder is given it (`word_case::decoder_input`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_decoderInput(
    env: *mut JNIEnv,
    _: jclass,
    typed: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make_string(&word_case::decoder_input(&env.read_string(typed, "typed")?))
        })
    }
}

/// The context before a word that starts after `text`: a first string that is "." if a sentence
/// starts there and empty if not, then the previous words, most recent first.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_previousWords(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let previous = previous_words::PreviousWords::from(&env.read_string(text, "text")?);
            let start = if previous.is_beginning_of_sentence {
                vec![u16::from(b'.')]
            } else {
                Vec::new()
            };
            env.make_strings(&[vec![start], previous.words].concat())
        })
    }
}

/// AOSP's "modest" auto-correction threshold (`auto_correction::MODEST_THRESHOLD`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_modestThreshold(
    _: *mut JNIEnv,
    _: jclass,
) -> jfloat {
    auto_correction::MODEST_THRESHOLD
}

/// The most previous words a context holds (`previous_words::MAX_COUNT`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_maxPreviousWords(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    previous_words::MAX_COUNT as jint
}

/// Whether auto-correcting `typed` to the decoder's `word` (with its `score` and `kindAndFlags`)
/// passes `threshold` (`auto_correction::suggestion_exceeds_threshold`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_exceedsThreshold(
    env: *mut JNIEnv,
    _: jclass,
    word: jstring,
    score: jint,
    kind_and_flags: jint,
    typed: jstring,
    threshold: jfloat,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            let word = env.read_string(word, "word")?;
            let suggestion = RawSuggestion {
                word,
                score,
                kind_and_flags,
            };
            let typed = env.read_string(typed, "typed")?;
            Ok(boolean(auto_correction::suggestion_exceeds_threshold(
                &suggestion,
                &typed,
                threshold,
            )))
        })
    }
}

/// How much of `after` is right, weighted by `score` (`auto_correction::normalized_score`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_normalizedScore(
    env: *mut JNIEnv,
    _: jclass,
    before: jstring,
    after: jstring,
    score: jint,
) -> jfloat {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0.0, |env| {
            let (before, after) = (
                env.read_string(before, "before")?,
                env.read_string(after, "after")?,
            );
            Ok(auto_correction::normalized_score(&before, &after, score))
        })
    }
}

/// The edits from `before` to `after`, both code points (`auto_correction::edit_distance`).
/// Kotlin threw for a negative one, which isn't a code point; so does this.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_editDistance(
    env: *mut JNIEnv,
    _: jclass,
    before: jintArray,
    after: jintArray,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let before = code_points(env.read::<i32>(before, "before")?)?;
            let after = code_points(env.read::<i32>(after, "after")?)?;
            Ok(auto_correction::edit_distance(&before, &after))
        })
    }
}

fn code_points(values: Vec<i32>) -> Result<Vec<u32>> {
    values
        .into_iter()
        .map(|v| {
            u32::try_from(v)
                .map_err(|_| illegal(format!("Not a valid Unicode code point: 0x{v:X}")))
        })
        .collect()
}

/// How to undo auto-correcting `typed` to `corrected` at `start` (`undo::revert_at`), or null: the
/// length to replace, as two units (high half first), then 1 for a phantom space or 0, then the
/// text to put back.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_revertAt(
    env: *mut JNIEnv,
    _: jclass,
    typed: jstring,
    corrected: jstring,
    start: jint,
    cursor: jint,
    text_before_cursor: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (typed, corrected) = (
                env.read_string(typed, "typed")?,
                env.read_string(corrected, "corrected")?,
            );
            let text = env.read_string(text_before_cursor, "textBeforeCursor")?;
            let Some(revert) = undo::revert_at((&typed, &corrected, start), cursor, &text) else {
                return Ok(ptr::null_mut());
            };
            let length = revert.length as u32;
            let head = [
                (length >> 16) as u16,
                length as u16,
                u16::from(revert.phantom_space),
            ];
            env.make_string(&[&head[..], &revert.text].concat())
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SuggestNative_previousSlotCount(
    _: *mut JNIEnv,
    _: jclass,
    word_count: jint,
    sentence_start: jboolean,
) -> jint {
    previous_words::slot_count(
        usize::try_from(word_count).unwrap_or(0),
        sentence_start != 0,
    ) as jint
}
