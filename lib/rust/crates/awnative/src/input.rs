//! Input JNI transport. No Java objects survive a call and no callbacks hold a native lock.
use crate::{
    handle,
    jni::{self, illegal},
};
use jni_sys::{JNIEnv, jboolean, jclass, jfloatArray, jint, jlong, jlongArray};
use std::{ptr, sync::Mutex};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputNative_create(
    env: *mut JNIEnv,
    _: jclass,
) -> jlong {
    // SAFETY: this environment belongs to the current JVM call; its wrapper owns the allocation.
    unsafe {
        jni::run(env, 0, |_| {
            Ok(handle::into_handle(Mutex::new(
                input::lifecycle::Dispatcher::default(),
            )))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputNative_free(
    _: *mut JNIEnv,
    _: jclass,
    value: jlong,
) {
    // SAFETY: the synchronized Cleaner wrapper zeros and frees its owned handle exactly once.
    unsafe { handle::free::<Mutex<input::lifecycle::Dispatcher>>(value) }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputNative_call(
    env: *mut JNIEnv,
    _: jclass,
    value: jlong,
    operation: jint,
    fields: jlongArray,
) -> jlongArray {
    // SAFETY: fields are copied; the wrapper synchronizes calls and close. No callback enters Java.
    unsafe {
        handle::with_locked::<input::lifecycle::Dispatcher, _>(
            env,
            value,
            "input dispatcher",
            ptr::null_mut(),
            |session, env| {
                let fields = env.read::<i64>(fields, "input fields")?;
                env.make(&session.call(operation, &fields).map_err(illegal)?)
            },
        )
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputRules_route(
    env: *mut JNIEnv,
    _: jclass,
    fields: jlongArray,
) -> jlongArray {
    // SAFETY: only copied primitive values cross this JNI call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make(
                &input::routing::route(&env.read::<i64>(fields, "routing fields")?)
                    .map_err(illegal)?,
            )
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputRules_feedback(
    env: *mut JNIEnv,
    _: jclass,
    fields: jlongArray,
) -> jlongArray {
    // SAFETY: only copied primitive values cross this JNI call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make(
                &input::feedback::plan(&env.read::<i64>(fields, "feedback fields")?)
                    .map_err(illegal)?,
            )
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputRules_pointer(
    env: *mut JNIEnv,
    _: jclass,
    fields: jfloatArray,
) -> jboolean {
    // SAFETY: bounds and coordinates are copied and their length checked before indexing.
    unsafe {
        jni::run(env, 0, |env| {
            let v = env.read::<f32>(fields, "pointer fields")?;
            Ok(jni::boolean(match v.as_slice() {
                [code] => input::pointer::roll_release(*code as i32),
                [code, still] => input::pointer::rolls(*code as i32, *still != 0.0),
                [x, y, left, top, right, bottom] => {
                    input::pointer::slide(*x, *y, *left, *top, *right, *bottom)
                }
                _ => return Err(illegal("wrong pointer field count")),
            }))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputRules_visual(
    env: *mut JNIEnv,
    _: jclass,
    fields: jlongArray,
) -> jint {
    // SAFETY: the copied packet is checked before indexing; no JVM objects are retained.
    unsafe {
        jni::run(env, 0, |env| {
            let v = env.read::<i64>(fields, "visual fields")?;
            match v.as_slice() {
                [kind, code, mode] => Ok(input::visual::label(
                    *kind as i32,
                    *code as i32,
                    *mode as i32,
                )),
                [code, raw, input, options, mode] => Ok(input::visual::icon(
                    *code as i32,
                    *raw as u64,
                    *input as i32,
                    *options as i32,
                    *mode as i32,
                )),
                _ => Err(illegal("wrong visual field count")),
            }
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputRules_hold(
    _: *mut JNIEnv,
    _: jclass,
    code: jint,
    popup: jboolean,
    uninterrupted: jboolean,
    space_action: jboolean,
) -> jint {
    input::pointer::hold(code, popup != 0, uninterrupted != 0, space_action != 0)
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_InputRules_control(
    env: *mut JNIEnv,
    _: jclass,
    fields: jlongArray,
    text: jni_sys::jstring,
) -> jlongArray {
    // SAFETY: strings and numbers are copied and validated by the logic crate; nothing is retained.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            env.make(
                &input::control::plan(
                    &env.read::<i64>(fields, "control fields")?,
                    &env.read_string(text, "control text")?,
                )
                .map_err(illegal)?,
            )
        })
    }
}
