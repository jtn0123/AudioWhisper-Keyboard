//! Layout JNI transports primitive metadata and indices; keys never cross into Rust.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jboolean, jclass, jint, jintArray, jlong, jlongArray};
use keys::assembly;
use std::ptr;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LayoutRules_plan(
    env: *mut JNIEnv,
    _: jclass,
    mode: jint,
    number_row: jboolean,
) -> jintArray {
    // SAFETY: the JVM supplies this thread's environment.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make(&assembly::plan(mode, number_row != 0))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LayoutRules_modifierType(
    _: *mut JNIEnv,
    _: jclass,
    main_type: jint,
) -> jint {
    assembly::modifier_type(main_type)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LayoutRules_mergeRows(
    env: *mut JNIEnv,
    _: jclass,
    main: jintArray,
    modifier: jintArray,
    extension: jintArray,
    codes: jintArray,
) -> jintArray {
    // SAFETY: the JVM supplies the environment and live primitive array references.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let rows = assembly::merge(
                &env.read::<i32>(main, "main")?,
                &env.read::<i32>(modifier, "modifier")?,
                &env.read::<i32>(extension, "extension")?,
                &env.read::<i32>(codes, "modCodes")?,
            )
            .map_err(illegal)?;
            env.make(&rows)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LayoutRules_hints(
    env: *mut JNIEnv,
    _: jclass,
    mode: jint,
    number_hints: jboolean,
    main: jintArray,
    symbols: jintArray,
    types: jintArray,
) -> jintArray {
    // SAFETY: the JVM supplies the environment and live primitive array references.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let hints = assembly::hints(
                mode,
                number_hints != 0,
                &env.read::<i32>(main, "main")?,
                &env.read::<i32>(symbols, "symbols")?,
                &env.read::<i32>(types, "types")?,
            )
            .map_err(illegal)?;
            env.make(&hints)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LayoutRules_cacheKey(
    _: *mut JNIEnv,
    _: jclass,
    mode: jint,
    subtype_hash: jint,
) -> jlong {
    assembly::cache_key(mode, subtype_hash)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LayoutRules_invalidated(
    env: *mut JNIEnv,
    _: jclass,
    keys: jlongArray,
    mode: jint,
    subtype_hash: jint,
    filter: jint,
) -> jlongArray {
    // SAFETY: the JVM supplies the environment and live primitive array reference.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            if !(0..=3).contains(&filter) {
                return Err(illegal("unknown cache filter"));
            }
            let removed = assembly::invalidated(
                &env.read::<i64>(keys, "keys")?,
                (filter & 1 != 0).then_some(mode),
                (filter & 2 != 0).then_some(subtype_hash),
            );
            env.make(&removed)
        })
    }
}
