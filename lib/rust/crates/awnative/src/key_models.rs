//! Key-domain JNI. Logic remains Java-free in keys; state values cross without native handles.

use crate::jni::{self, boolean, illegal};
use jni_sys::{JNIEnv, jboolean, jclass, jint, jintArray, jlong, jobjectArray, jstring};
use keys::models;
use std::ptr;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_read(
    env: *mut JNIEnv,
    _: jclass,
    raw: jlong,
    field: jint,
) -> jint {
    // SAFETY: the JVM supplies this thread's environment.
    unsafe {
        jni::run(env, 0, |_| {
            models::read(raw as u64, field as usize).ok_or_else(|| illegal("unknown state field"))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_write(
    env: *mut JNIEnv,
    _: jclass,
    raw: jlong,
    field: jint,
    value: jint,
) -> jlong {
    // SAFETY: the JVM supplies this thread's environment.
    unsafe {
        jni::run(env, raw, |_| {
            models::write(raw as u64, field as usize, value)
                .map(|v| v as i64)
                .ok_or_else(|| illegal("unknown state field"))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_select(
    _: *mut JNIEnv,
    _: jclass,
    kind: jint,
    raw: jlong,
    present: jint,
) -> jint {
    models::select(kind, raw as u64, present as u32)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_isCurrency(
    _: *mut JNIEnv,
    _: jclass,
    code: jint,
) -> jboolean {
    boolean(models::is_currency(code))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_currencySlot(
    _: *mut JNIEnv,
    _: jclass,
    code: jint,
) -> jint {
    models::currency_slot(code)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_currencyDefaults(
    env: *mut JNIEnv,
    _: jclass,
) -> jintArray {
    // SAFETY: the JVM supplies this thread's environment.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make(models::CURRENCY_DEFAULTS)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_isSpace(
    _: *mut JNIEnv,
    _: jclass,
    kind: jint,
    code: jint,
) -> jboolean {
    boolean(models::is_space(kind, code))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_predefined(
    env: *mut JNIEnv,
    _: jclass,
) -> jobjectArray {
    let rows: Vec<Vec<u16>> = keys::model_data::PREDEFINED
        .iter()
        .map(|key| {
            format!("{},{},{}", key.code, key.kind, key.label)
                .encode_utf16()
                .collect()
        })
        .collect();
    // SAFETY: the JVM supplies this thread's environment.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make_strings(&rows)) }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_codeKind(
    _: *mut JNIEnv,
    _: jclass,
    code: jint,
) -> jint {
    models::code_kind(code)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_text(
    env: *mut JNIEnv,
    _: jclass,
    code: jint,
    label: jstring,
    display: jboolean,
    non_spacing: jboolean,
) -> jstring {
    // SAFETY: the JVM supplies this thread's environment and string reference.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make_string(&models::text(
                code,
                &env.read_string(label, "label")?,
                display != 0,
                non_spacing != 0,
            ))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_multiText(
    env: *mut JNIEnv,
    _: jclass,
    codes: jintArray,
    label: jstring,
    display: jboolean,
) -> jstring {
    // SAFETY: the JVM supplies this thread's environment and array/string references.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make_string(&models::multi_text(
                &env.read::<i32>(codes, "codes")?,
                &env.read_string(label, "label")?,
                display != 0,
            ))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_popupMode(
    _: *mut JNIEnv,
    _: jclass,
    mode: jint,
) -> jboolean {
    boolean(models::popup_mode(mode))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_popupGroup(
    env: *mut JNIEnv,
    _: jclass,
    group: jint,
) -> jstring {
    // SAFETY: the JVM supplies this thread's environment.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            match models::popup_group(group) {
                Some(s) => env.make_string(&s.encode_utf16().collect::<Vec<_>>()),
                None => Ok(ptr::null_mut()),
            }
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_popupSource(
    _: *mut JNIEnv,
    _: jclass,
    variation: jint,
    present: jint,
) -> jint {
    models::popup_source(variation, present as u32)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_phoneHint(
    env: *mut JNIEnv,
    _: jclass,
    code: jint,
) -> jstring {
    // SAFETY: the JVM supplies this thread's environment.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| match models::phone_hint(code) {
            Some(s) => env.make_string(&s.encode_utf16().collect::<Vec<_>>()),
            None => Ok(ptr::null_mut()),
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_codes(
    env: *mut JNIEnv,
    _: jclass,
) -> jintArray {
    // SAFETY: the JVM supplies this thread's environment.
    unsafe { jni::run(env, ptr::null_mut(), |env| env.make(keys::codes::VALUES)) }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_enabled(
    _: *mut JNIEnv,
    _: jclass,
    code: jint,
    a: jint,
    b: jint,
) -> jboolean {
    boolean(models::enabled(code, a, b))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_KeyModels_visible(
    _: *mut JNIEnv,
    _: jclass,
    code: jint,
    utility: jboolean,
    action: jint,
    language: jboolean,
) -> jboolean {
    boolean(models::visible(code, utility != 0, action, language != 0))
}
