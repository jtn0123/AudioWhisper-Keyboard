//! JNI entry points for Kotlin's character rules as the `chars` crate has them, called by
//! `JavaTextNative` in lib/rust's Kotlin. Nothing in the keyboard calls these; they are there so
//! tests can hold `chars` up against `java.lang.String` and `Character`, on a computer and on a
//! phone.

use std::ptr;

use jni_sys::jstring;
use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jchar, jclass, jint, jintArray, jlong};

use crate::jni::{self, Env, Result, boolean};

// The bits of `unitFacts`' first value, which `JavaTextNative` reads.
const LETTER: jint = 1;
const DIGIT: jint = 2;
const LETTER_OR_DIGIT: jint = 4;
const UPPER_CASE: jint = 8;
const LOWER_CASE: jint = 16;
const TITLE_CASE: jint = 32;
const WHITESPACE: jint = 64;
const REGEX_SPACE: jint = 128;

/// What `toIntOrNull` returns for null: no `Int` is `Long.MIN_VALUE`.
const NO_INT: jlong = jlong::MIN;

/// `unit`'s `Char` predicates, as bits, then its `uppercaseChar()`, its `lowercaseChar()` and
/// the first code point of its NFD.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_unitFacts(
    env: *mut JNIEnv,
    _: jclass,
    unit: jchar,
) -> jintArray {
    let predicates = [
        (chars::is_letter as fn(u16) -> bool, LETTER),
        (chars::is_digit, DIGIT),
        (chars::is_letter_or_digit, LETTER_OR_DIGIT),
        (chars::is_upper_case, UPPER_CASE),
        (chars::is_lower_case, LOWER_CASE),
        (chars::is_title_case, TITLE_CASE),
        (chars::is_whitespace, WHITESPACE),
        (chars::is_regex_space, REGEX_SPACE),
    ];
    let bits = predicates
        .iter()
        .filter(|(test, _)| test(unit))
        .map(|(_, bit)| bit)
        .sum();
    let decomposed = chars::code_point::decomposed_first(unit);
    // A code point is at most 0x10FFFF, so it fits in an Int.
    let facts = [
        bits,
        chars::to_upper(unit).into(),
        chars::to_lower(unit).into(),
        decomposed as jint,
    ];
    // SAFETY: the JVM passes its environment for this call.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make(&facts)) }
}

/// `Character.isDigit(codePoint)` as 0 or 1, then `Character.digit(codePoint, 10)`,
/// `Character.toUpperCase(codePoint)` and `Character.toLowerCase(codePoint)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_codePointFacts(
    env: *mut JNIEnv,
    _: jclass,
    code_point: jint,
) -> jintArray {
    use chars::code_point::{digit, is_digit, to_lower, to_upper};
    // Java answers a negative Int as no character at all.
    let facts = match u32::try_from(code_point) {
        Ok(cp) => [
            jint::from(is_digit(cp)),
            digit(cp).map_or(-1, jint::from),
            to_upper(cp) as jint,
            to_lower(cp) as jint,
        ],
        Err(_) => [0, -1, code_point, code_point],
    };
    // SAFETY: the JVM passes its environment for this call.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make(&facts)) }
}

/// Reads `text` and makes a string of what `map` makes of it.
///
/// # Safety
/// `env` must be the environment the JVM passed to the native call this runs in.
unsafe fn map_string(
    env: *mut JNIEnv,
    text: jstring,
    map: impl FnOnce(&[u16]) -> Vec<u16>,
) -> jstring {
    // SAFETY: as the caller promises.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let text = env.read_string(text, "text")?;
            env.make_string(&map(&text))
        })
    }
}

/// Reads two strings and gives what `test` says of them.
///
/// # Safety
/// `env` must be the environment the JVM passed to the native call this runs in.
unsafe fn test_strings(
    env: *mut JNIEnv,
    (a, b): (jstring, jstring),
    test: fn(&[u16], &[u16]) -> bool,
) -> jboolean {
    // SAFETY: as the caller promises.
    unsafe {
        jni::run(env, JNI_FALSE, |env: &Env| -> Result<jboolean> {
            let (a, b) = (env.read_string(a, "a")?, env.read_string(b, "b")?);
            Ok(boolean(test(&a, &b)))
        })
    }
}

/// Kotlin's `text.lowercase()`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_lowercase(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe { map_string(env, text, chars::lowercase) }
}

/// Kotlin's `text.uppercase()`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_uppercase(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe { map_string(env, text, chars::uppercase) }
}

/// Kotlin's `text.trim()`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_trim(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe { map_string(env, text, |text| chars::trim(text).to_vec()) }
}

/// Kotlin's `a.equals(b, ignoreCase = true)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_equalsIgnoreCase(
    env: *mut JNIEnv,
    _: jclass,
    a: jstring,
    b: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe { test_strings(env, (a, b), chars::equals_ignore_case) }
}

/// Kotlin's `a.startsWith(b, ignoreCase = true)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_startsWithIgnoreCase(
    env: *mut JNIEnv,
    _: jclass,
    a: jstring,
    b: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe { test_strings(env, (a, b), chars::starts_with_ignore_case) }
}

/// Kotlin's `text.isBlank()`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_isBlank(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            Ok(boolean(chars::is_blank(&env.read_string(text, "text")?)))
        })
    }
}

/// Kotlin's `text.codePointCount(0, text.length)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_codePointCount(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let count = chars::text::code_point_count(&env.read_string(text, "text")?);
            // There are no more code points than units, whose count is an Int.
            Ok(count as jint)
        })
    }
}

/// Kotlin's `text.toIntOrNull()`, with `Long.MIN_VALUE` for null.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_JavaTextNative_toIntOrNull(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, NO_INT, |env| {
            let value = chars::text::to_int_or_null(&env.read_string(text, "text")?);
            Ok(value.map_or(NO_INT, jlong::from))
        })
    }
}
