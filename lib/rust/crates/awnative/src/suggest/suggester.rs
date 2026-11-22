//! Shared store/result transport for the native session. No managed dictionary callbacks.
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use crate::jni::Env;
use crate::{handle::object, jni::Result, jni::decimal};
use jni_sys::jlong;
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
use jni_sys::{jintArray, jobjectArray};
use std::sync::Mutex;
use suggest::{learned_words::LearnedWords, personal_words::PersonalWords, suggester::Arranged};

/// The words behind the handles: the personal words, and the learned words unless `learned` is 0.
///
/// # Safety
/// `personal` must be a handle to a `PersonalWords` and `learned` 0 or one to a
/// `Mutex<LearnedWords>`, neither freed: Kotlin holds both stores while it calls.
pub(super) unsafe fn stores<'a>(
    personal: jlong,
    learned: jlong,
) -> Result<(&'a PersonalWords, Option<&'a Mutex<LearnedWords>>)> {
    // SAFETY: as the caller promises.
    let personal = unsafe { object::<PersonalWords>(personal, "personal words")? };
    let learned = match learned {
        0 => None,
        // SAFETY: as the caller promises.
        handle => Some(unsafe { object::<Mutex<LearnedWords>>(handle, "learned words")? }),
    };
    Ok((personal, learned))
}

/// The strip as a row of strings: the indexes of its auto-correction and typed word, in decimal,
/// then its words.
pub(super) fn strip_row(strip: &[Arranged]) -> Vec<Vec<u16>> {
    let mut row = vec![
        decimal(mark(strip, |it| it.is_auto_correction)),
        decimal(mark(strip, |it| it.is_typed_word)),
    ];
    row.extend(strip.iter().map(|it| it.text.clone()));
    row
}

/// The strip's words, with the indexes of its auto-correction and typed word written to `marks`.
#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
pub(super) fn write_strip(env: &Env, strip: &[Arranged], marks: jintArray) -> Result<jobjectArray> {
    let auto_correction = mark(strip, |it| it.is_auto_correction);
    env.write(
        marks,
        &[auto_correction, mark(strip, |it| it.is_typed_word)],
        "marks",
    )?;
    let texts: Vec<&[u16]> = strip.iter().map(|it| &it.text[..]).collect();
    env.make_strings(&texts)
}

/// The index of the strip's first word that `is`, or -1.
fn mark(strip: &[Arranged], is: impl Fn(&Arranged) -> bool) -> i32 {
    // The strip is no longer than a Java array.
    strip.iter().position(is).map_or(-1, |index| index as i32)
}
