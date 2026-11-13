//! JNI transport only. No editor semantics or retained JVM references.
use crate::{
    handle,
    jni::{self, Error, illegal},
};
use jni_sys::{JNIEnv, jclass, jlong, jobjectArray, jstring};
use std::{ptr, sync::Mutex};

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EditorNative_create(
    env: *mut JNIEnv,
    _: jclass,
) -> jlong {
    // SAFETY: the environment belongs to this JVM call; the returned allocation is owned by its wrapper.
    unsafe {
        jni::run(env, 0, |_| {
            Ok(handle::into_handle(Mutex::new(editor::Session::new())))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EditorNative_free(
    _: *mut JNIEnv,
    _: jclass,
    value: jlong,
) {
    // SAFETY: the wrapper synchronizes close with calls, zeros its handle and frees exactly once.
    unsafe { handle::free::<Mutex<editor::Session>>(value) }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EditorNative_call(
    env: *mut JNIEnv,
    _: jclass,
    value: jlong,
    operation: jstring,
    fields: jobjectArray,
) -> jobjectArray {
    // SAFETY: all JVM arrays are copied; the wrapper keeps the handle live and serializes close.
    unsafe {
        handle::with_locked::<editor::Session, _>(
            env,
            value,
            "editor",
            ptr::null_mut(),
            |session, env| {
                let operation = String::from_utf16(&env.read_string(operation, "operation")?)
                    .map_err(|_| illegal("invalid editor operation"))?;
                let fields = env
                    .read_strings(fields, "fields")?
                    .into_iter()
                    .map(|s| s.ok_or(Error::NullPointer("editor field")))
                    .collect::<jni::Result<Vec<_>>>()?;
                env.make_strings(&session.checked_call(&operation, &fields).map_err(illegal)?)
            },
        )
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_EditorNative_attributes(
    env: *mut JNIEnv,
    _: jclass,
    fields: jobjectArray,
) -> jobjectArray {
    // SAFETY: this call's environment and array are JVM-owned; strings are copied before evaluation.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let fields = env
                .read_strings(fields, "fields")?
                .into_iter()
                .map(|s| s.ok_or(Error::NullPointer("attribute field")))
                .collect::<jni::Result<Vec<_>>>()?;
            if fields.len() < 2 || (editor::integer(&fields, 0) == 6 && fields.len() < 4) {
                return Err(illegal("wrong editor attribute field count"));
            }
            env.make_strings(&editor::attributes::call(&fields))
        })
    }
}
