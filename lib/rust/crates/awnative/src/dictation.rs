//! JNI entry points for dictation's text rules, called by `DictationNative` in lib/rust's Kotlin.
//!
//! Results that aren't a boolean come back as strings: a typing as one string whose first
//! character's code is how many characters to delete, followed by the text to type; the parts of
//! a phrase as one string each, whose first character says what the part is.

use std::ptr;

use dictation::{Spoken, Typing};
use jni_sys::{JNI_FALSE, JNI_TRUE, JNIEnv, jboolean, jchar, jclass, jint, jobject};
use jni_sys::{jobjectArray, jstring, jvalue};

use crate::jni::{self, Env, Result};

/// The part tags `DictationNative` reads.
const WORDS: u16 = b'w' as u16;
const MARK: u16 = b'm' as u16;
const BREAK: u16 = b'b' as u16;
const SCRATCH_THAT: u16 = b's' as u16;

fn tagged(part: &Spoken) -> Vec<u16> {
    let mut tagged = Vec::new();
    match part {
        Spoken::Words(words) => {
            tagged.push(WORDS);
            tagged.extend(words);
        }
        Spoken::Mark(mark) => tagged.extend([MARK, *mark]),
        Spoken::Break(text) => {
            tagged.push(BREAK);
            tagged.extend(text.encode_utf16());
        }
        Spoken::ScratchThat => tagged.push(SCRATCH_THAT),
    }
    tagged
}

fn typing(env: &Env, typing: &Typing) -> Result<jstring> {
    // Replacing is 0 or 1; the most a typing can replace is the one full stop or mark.
    let replacing = u16::try_from(typing.replacing).map_err(|_| jni::illegal("replacing"))?;
    env.make_string(&[&[replacing], typing.text.as_slice()].concat())
}

/// The parts of `phrase`: words, a mark, line breaks or "scratch that" (see `dictation::parse`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_parse(
    env: *mut JNIEnv,
    _: jclass,
    phrase: jstring,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let phrase = env.read_string(phrase, "phrase")?;
            let parts: Vec<Vec<u16>> = dictation::parse(&phrase).iter().map(tagged).collect();
            env.make_strings(&parts)
        })
    }
}

/// How to type `phrase` after `before` (see `dictation::join`), asking `names.isName(word)` if
/// the phrase's first word might be a name.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_join(
    env: *mut JNIEnv,
    _: jclass,
    before: jstring,
    last_phrase: jstring,
    phrase: jstring,
    names: jobject,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let before = env.read_string(before, "before")?;
            let last_phrase = env.read_nullable_string(last_phrase)?;
            let phrase = env.read_string(phrase, "phrase")?;
            let mut failed = None;
            let joined = dictation::join(&before, last_phrase.as_deref(), &phrase, |word| {
                is_name(env, names, word).unwrap_or_else(|error| {
                    failed = Some(error);
                    false
                })
            });
            match failed {
                Some(error) => Err(error),
                None => typing(env, &joined),
            }
        })
    }
}

/// Asks Kotlin whether `word` is a name.
fn is_name(env: &Env, names: jobject, word: &[u16]) -> Result<bool> {
    let is_name = env.method(names, "names", (c"isName", c"(Ljava/lang/String;)Z"))?;
    env.with_string(word, |word| {
        env.call_boolean(is_name, &[jvalue { l: word }])
    })
}

/// How to type the spoken `mark` after `before` (see `dictation::punctuate`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_punctuate(
    env: *mut JNIEnv,
    _: jclass,
    before: jstring,
    last_phrase: jstring,
    mark: jchar,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let before = env.read_string(before, "before")?;
            let last_phrase = env.read_nullable_string(last_phrase)?;
            typing(
                env,
                &dictation::punctuate(&before, last_phrase.as_deref(), mark),
            )
        })
    }
}

/// Whether a backspace can take back `typed` (see `dictation::reverts_at`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_revertsAt(
    env: *mut JNIEnv,
    _: jclass,
    typed: jstring,
    start: jint,
    cursor: jint,
    text_before_cursor: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            let typed = env.read_string(typed, "typed")?;
            let text = env.read_string(text_before_cursor, "textBeforeCursor")?;
            let reverts = dictation::reverts_at(&typed, start, cursor, &text);
            Ok(if reverts { JNI_TRUE } else { JNI_FALSE })
        })
    }
}
