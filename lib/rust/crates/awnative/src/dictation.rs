//! JNI entry points for dictation's text rules and its mic key, called by `DictationNative` in
//! lib/rust's Kotlin.
//!
//! Results that aren't a boolean come back as strings: a typing as one string whose first
//! character's code is how many characters to delete, followed by the text to type; the parts of
//! a phrase as one string each, whose first character says what the part is.

use std::ptr;
use std::sync::Mutex;

use dictation::mic_key::{self, Action, MicKey, Status};
use dictation::{Spoken, Typing};
use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jchar, jclass, jint, jlong, jobject};
use jni_sys::{jobjectArray, jstring, jvalue};

use crate::handle::{free, into_handle, with_locked};
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
            Ok(jni::boolean(reverts))
        })
    }
}

/// A press this long, in milliseconds, is a hold (`mic_key::HOLD_MS`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_holdMs(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    mic_key::HOLD_MS
}

/// A new mic key, not pressed.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_newMicKey(
    _: *mut JNIEnv,
    _: jclass,
) -> jlong {
    into_handle(Mutex::new(MicKey::default()))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_freeMicKey(
    _: *mut JNIEnv,
    _: jclass,
    key: jlong,
) {
    // SAFETY: the Kotlin side frees each key once, when nothing can reach it.
    unsafe { free::<Mutex<MicKey>>(key) }
}

/// An action's number, as `MicKeyState.Action` orders them.
fn action(action: Action) -> jint {
    match action {
        Action::Nothing => 0,
        Action::Start => 1,
        Action::Finish => 2,
        Action::EndHold => 3,
        Action::Toggle => 4,
    }
}

/// The status numbered as `MicKeyState.Status` orders them.
fn status(number: jint) -> Result<Status> {
    match number {
        0 => Ok(Status::Idle),
        1 => Ok(Status::Listening),
        2 => Ok(Status::Finishing),
        _ => Err(jni::illegal("status")),
    }
}

/// The key behind `key` went down at `now` while dictation was `status` (`MicKey::press`): what
/// that asks for.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_micKeyPress(
    env: *mut JNIEnv,
    _: jclass,
    key: jlong,
    status_number: jint,
    now: jlong,
) -> jint {
    // SAFETY: the JVM passes its environment; Kotlin holds the key while it calls.
    unsafe {
        with_locked(env, key, "mic key", 0, |key: &mut MicKey, _| {
            Ok(action(key.press(status(status_number)?, now)))
        })
    }
}

/// The key behind `key` came up at `now` (`MicKey::release`): what that asks for.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationNative_micKeyRelease(
    env: *mut JNIEnv,
    _: jclass,
    key: jlong,
    now: jlong,
) -> jint {
    // SAFETY: the JVM passes its environment; Kotlin holds the key while it calls.
    unsafe {
        with_locked(env, key, "mic key", 0, |key: &mut MicKey, _| {
            Ok(action(key.release(now)))
        })
    }
}
