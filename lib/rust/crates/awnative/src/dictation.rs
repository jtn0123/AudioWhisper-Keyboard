//! Dictation's text rules and its mic key, for `DictationNative`'s generated entry points.
//!
//! A phrase's parts come back as rows of two strings, the part's kind (as `DictationRules.Kind`
//! orders them) and its text; a typing as how many characters to delete, then the text to type.

use std::sync::Mutex;

use dictation::Spoken;
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use dictation::Typing;
use dictation::mic_key::{self, Action, MicKey, Status};
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use jni_sys::{JNIEnv, jclass, jobject, jstring, jvalue};

use crate::handle::{free, into_handle, locked};
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use crate::jni::{self, Env};
use crate::jni::{Result, decimal, illegal};

/// The parts of `phrase`, two strings each: words, a mark, line breaks or "scratch that" (see
/// `dictation::parse`).
pub fn parse(phrase: &[u16]) -> Vec<Vec<u16>> {
    let mut rows = Vec::new();
    for part in dictation::parse(phrase) {
        let (kind, text) = match part {
            Spoken::Words(words) => (0, words),
            Spoken::Mark(mark) => (1, vec![mark]),
            Spoken::Break(text) => (2, text.encode_utf16().collect()),
            Spoken::ScratchThat => (3, Vec::new()),
        };
        rows.extend([decimal(kind), text]);
    }
    rows
}

/// How to type the spoken `mark` after `before` (see `dictation::punctuate`).
pub fn punctuate(before: &[u16], last_phrase: Option<&[u16]>, mark: u16) -> Vec<Vec<u16>> {
    let typing = dictation::punctuate(before, last_phrase, mark);
    vec![decimal(typing.replacing), typing.text]
}

/// A press this long, in milliseconds, is a hold (`mic_key::HOLD_MS`).
pub fn hold_ms() -> i64 {
    mic_key::HOLD_MS
}

/// A new mic key, not pressed.
pub fn new_mic_key() -> i64 {
    into_handle(Mutex::new(MicKey::default()))
}

pub fn free_mic_key(key: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each key once, when nothing can reach it.
    unsafe { free::<Mutex<MicKey>>(key) };
    Ok(())
}

/// An action's number, as `MicKeyState.Action` orders them.
fn action(action: Action) -> i32 {
    match action {
        Action::Nothing => 0,
        Action::Start => 1,
        Action::Finish => 2,
        Action::EndHold => 3,
        Action::Toggle => 4,
    }
}

/// The status numbered as `MicKeyState.Status` orders them.
fn status(number: i32) -> Result<Status> {
    match number {
        0 => Ok(Status::Idle),
        1 => Ok(Status::Listening),
        2 => Ok(Status::Finishing),
        _ => Err(illegal("status")),
    }
}

/// The key behind `key` went down at `now` while dictation was `status` (`MicKey::press`): what
/// that asks for.
pub fn mic_key_press(key: i64, status_number: i32, now: i64) -> Result<i32> {
    // SAFETY: Kotlin holds the key while it calls, so it hasn't been freed.
    let mut key = unsafe { locked::<MicKey>(key, "mic key")? };
    Ok(action(key.press(status(status_number)?, now)))
}

/// The key behind `key` came up at `now` (`MicKey::release`): what that asks for.
pub fn mic_key_release(key: i64, now: i64) -> Result<i32> {
    // SAFETY: Kotlin holds the key while it calls, so it hasn't been freed.
    let mut key = unsafe { locked::<MicKey>(key, "mic key")? };
    Ok(action(key.release(now)))
}

/// `phrase` with filler words removed (when `remove_fillers`) and `vocabulary` applied: the Mac
/// app's rules, from AudioWhisper-Core's `transcript`. `vocabulary` is its text, one entry per
/// line (see `transcript::vocabulary`). With nothing to remove or apply, or a phrase that isn't
/// valid UTF-16, the phrase comes back as it was.
pub fn format(phrase: &[u16], vocabulary: &[u16], remove_fillers: bool) -> Vec<u16> {
    if !remove_fillers && chars::is_blank(vocabulary) {
        return phrase.to_vec();
    }
    let (Ok(text), Ok(vocabulary)) = (String::from_utf16(phrase), String::from_utf16(vocabulary))
    else {
        return phrase.to_vec();
    };
    let options = transcript::Options {
        vocabulary: transcript::Vocabulary::parse(&vocabulary),
        remove_fillers,
        spoken_punctuation: false,
    };
    transcript::format(&text, &options).encode_utf16().collect()
}

/// A typing as the test-only join returns it: one string whose first character's code is how
/// many characters to delete, followed by the text to type.
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
fn typing(env: &Env, typing: &Typing) -> jni::Result<jstring> {
    // Replacing is 0 or 1; the most a typing can replace is the one full stop or mark.
    let replacing = u16::try_from(typing.replacing).map_err(|_| illegal("replacing"))?;
    env.make_string(&[&[replacing], typing.text.as_slice()].concat())
}

/// How to type `phrase` after `before` (see `dictation::join`), asking `names.isName(word)` if
/// the phrase's first word might be a name.
#[unsafe(no_mangle)]
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LegacyFixtureNative_join(
    env: *mut JNIEnv,
    _: jclass,
    before: jstring,
    last_phrase: jstring,
    phrase: jstring,
    names: jobject,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, std::ptr::null_mut(), |env| {
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
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
fn is_name(env: &Env, names: jobject, word: &[u16]) -> Result<bool> {
    let is_name = env.method(names, "names", (c"isName", c"(Ljava/lang/String;)Z"))?;
    env.with_string(word, |word| {
        env.call_boolean(is_name, &[jvalue { l: word }])
    })
}
