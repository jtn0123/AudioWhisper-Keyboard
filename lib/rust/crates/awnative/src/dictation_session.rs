//! Session transport. Effects are executed after returning to Kotlin, outside the lock.
use crate::{
    handle,
    jni::{self, illegal},
};
use jni_sys::{JNIEnv, jboolean, jclass, jint, jlong, jobjectArray, jstring};
use std::{ptr, sync::Mutex};
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_create(
    env: *mut JNIEnv,
    _: jclass,
) -> jlong {
    // SAFETY: the environment is valid for this call and the wrapper owns the allocation.
    unsafe {
        jni::run(env, 0, |_| {
            Ok(handle::into_handle(Mutex::new(
                dictation::session::Session::new(),
            )))
        })
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_free(
    _: *mut JNIEnv,
    _: jclass,
    h: jlong,
) {
    // SAFETY: Cleaner synchronizes calls and frees once, after zeroing its handle.
    unsafe { handle::free::<Mutex<dictation::session::Session>>(h) }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_call(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    op: jstring,
    now: jlong,
    id: jint,
    fields: jobjectArray,
) -> jobjectArray {
    // SAFETY: the wrapper synchronizes close; all JVM values are copied in this call.
    unsafe {
        handle::with_locked::<dictation::session::Session, _>(
            env,
            h,
            "dictation",
            ptr::null_mut(),
            |s, e| {
                let op = String::from_utf16(&e.read_string(op, "operation")?)
                    .map_err(|_| illegal("invalid operation"))?;
                let fields = e
                    .read_strings(fields, "fields")?
                    .into_iter()
                    .map(|s| s.ok_or(illegal("null field")))
                    .collect::<jni::Result<Vec<_>>>()?;
                e.make_strings(&s.call(&op, now, id, &fields).map_err(illegal)?)
            },
        )
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_releaseAt(
    _: *mut JNIEnv,
    _: jclass,
    idle: jlong,
    now: jlong,
    shown: jboolean,
) -> jlong {
    dictation::session::release_at(idle, now, shown != 0)
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_ending(
    env: *mut JNIEnv,
    _: jclass,
    typed: jboolean,
    failures: jint,
) -> jstring {
    // SAFETY: the JVM environment is valid for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |e| {
            e.make_string(
                &dictation::session::ending(typed != 0, failures)
                    .encode_utf16()
                    .collect::<Vec<_>>(),
            )
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_preview(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    claim: jboolean,
) -> jboolean {
    // SAFETY: wrapper keeps owner alive; only its gate is changed under the session mutex.
    unsafe {
        handle::with_locked::<dictation::session::Session, _>(env, h, "dictation", 0, |s, _| {
            Ok(jni::boolean(if claim != 0 {
                s.claim_preview()
            } else {
                s.complete_preview();
                false
            }))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_model(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    op: jstring,
) -> jint {
    // SAFETY: wrapper retains the owner; only cache policy is changed under the native mutex.
    unsafe {
        handle::with_locked::<dictation::session::Session, _>(env, h, "dictation", 0, |s, e| {
            let op = String::from_utf16(&e.read_string(op, "model operation")?)
                .map_err(|_| illegal("invalid model operation"))?;
            s.model(&op).map_err(illegal)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_DictationSessionNative_speech(
    env: *mut JNIEnv,
    _: jclass,
    h: jlong,
    id: jint,
    detected: jboolean,
) {
    // SAFETY: synchronized wrapper retains the owner. A mic notification only updates native
    // phase under its mutex; it does not publish UI or add work to either executor.
    unsafe {
        handle::with_locked::<dictation::session::Session, _>(env, h, "dictation", (), |s, _| {
            s.speech(id, detected != 0);
            Ok(())
        })
    }
}
