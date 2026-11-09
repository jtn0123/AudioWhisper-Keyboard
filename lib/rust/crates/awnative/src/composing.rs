//! JNI entry points for the text composers (the `composing` crate), called by `ComposingNative`
//! in lib/rust's Kotlin.
//!
//! An action comes back as one string: null for none (Kotlin's `0 to toInsert`), otherwise the
//! number of units to delete as its first two units (the high 16 bits, then the low; it can be
//! -1), followed by the text to type. A rule table crosses once, as its endings in one string and
//! their texts in another, cut by their lengths (`jni::cut`), and is kept behind a handle.

use std::ptr;

use composing::rules::Rules;
use composing::{Action, hangul, kana};
use jni_sys::{JNIEnv, jboolean, jclass, jintArray, jlong, jstring};

use crate::handle::{free, into_handle, object};
use crate::jni::{self, Env, Result, illegal};

/// An action as `ComposingNative` reads it, or null for none.
fn packed(env: &Env, action: Option<Action>) -> Result<jstring> {
    let Some(action) = action else {
        return Ok(ptr::null_mut());
    };
    // The count's two halves, as units.
    let delete = action.delete as u32;
    let mut units = vec![(delete >> 16) as u16, delete as u16];
    units.extend(action.text);
    env.make_string(&units)
}

/// Reads the two strings every composer takes and asks `compose` with them.
fn with_text(
    env: &Env,
    preceding: jstring,
    to_insert: jstring,
    compose: impl FnOnce(&[u16], &[u16]) -> Option<Action>,
) -> Result<jstring> {
    let preceding = env.read_string(preceding, "precedingText")?;
    let to_insert = env.read_string(to_insert, "toInsert")?;
    packed(env, compose(&preceding, &to_insert))
}

/// Typing `toInsert` after `precedingText` in Korean (`hangul::actions`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ComposingNative_hangul(
    env: *mut JNIEnv,
    _: jclass,
    preceding: jstring,
    to_insert: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            with_text(env, preceding, to_insert, hangul::actions)
        })
    }
}

/// Typing `toInsert` after `precedingText` in kana (`kana::actions`), `sticky` or not.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ComposingNative_kana(
    env: *mut JNIEnv,
    _: jclass,
    preceding: jstring,
    to_insert: jstring,
    sticky: jboolean,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            with_text(env, preceding, to_insert, |preceding, to_insert| {
                kana::actions(preceding, to_insert, sticky != 0)
            })
        })
    }
}

/// Rules to try in order: ending `i` is the `i`th piece of `keys`, cut by `keyLengths`, and its
/// text the `i`th of `values`, cut by `valueLengths`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ComposingNative_newRules(
    env: *mut JNIEnv,
    _: jclass,
    keys: jstring,
    key_lengths: jintArray,
    values: jstring,
    value_lengths: jintArray,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let keys = env.read_string(keys, "keys")?;
            let key_lengths = env.read::<i32>(key_lengths, "keyLengths")?;
            let values = env.read_string(values, "values")?;
            let value_lengths = env.read::<i32>(value_lengths, "valueLengths")?;
            let keys = jni::cut(&keys, &key_lengths)?;
            let values = jni::cut(&values, &value_lengths)?;
            if keys.len() != values.len() {
                return Err(illegal("as many texts as endings"));
            }
            let rules = keys.iter().zip(&values);
            let rules = rules.map(|(key, value)| (key.to_vec(), value.to_vec()));
            Ok(into_handle(Rules::new(rules.collect())))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ComposingNative_freeRules(
    _: *mut JNIEnv,
    _: jclass,
    rules: jlong,
) {
    // SAFETY: the Kotlin side frees each rule table once, when nothing can reach it.
    unsafe { free::<Rules>(rules) }
}

/// Typing `toInsert` after `precedingText` with the rules behind `rules` (`Rules::actions`).
/// The rules never change, so calls from several threads may share them unlocked.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ComposingNative_rules(
    env: *mut JNIEnv,
    _: jclass,
    rules: jlong,
    preceding: jstring,
    to_insert: jstring,
) -> jstring {
    // SAFETY: the JVM passes its environment for this call, and Kotlin holds the rules while it
    // calls (see `object`).
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let rules = object::<Rules>(rules, "rules")?;
            with_text(env, preceding, to_insert, |preceding, to_insert| {
                rules.actions(preceding, to_insert)
            })
        })
    }
}
