//! Stateless, copied UTF-16 packets; no handles, callbacks or I/O.
use crate::jni::{self, Error, illegal};
use jni_sys::{JNIEnv, jclass, jint, jobjectArray};
use std::ptr;

type PolicyCall = fn(i32, &[Vec<u16>]) -> Result<Vec<Vec<u16>>, &'static str>;

fn transfer(
    env: *mut JNIEnv,
    operation: jint,
    input: jobjectArray,
    call: PolicyCall,
) -> jobjectArray {
    // SAFETY: the JVM supplies this call's environment/array; inputs are copied and
    // outputs allocated within this frame. No JNI references escape.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let input = env
                .read_strings(input, "policy input")?
                .into_iter()
                .map(|v| v.ok_or(Error::NullPointer("policy field")))
                .collect::<jni::Result<Vec<_>>>()?;
            let output = call(operation, &input).map_err(illegal)?;
            env.make_strings(&output)
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_BackupNative_call(
    env: *mut JNIEnv,
    _: jclass,
    operation: jint,
    input: jobjectArray,
) -> jobjectArray {
    transfer(env, operation, input, backup::call)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_org_florisboard_lib_kotlin_UtilityNative_call(
    env: *mut JNIEnv,
    _: jclass,
    operation: jint,
    input: jobjectArray,
) -> jobjectArray {
    transfer(env, operation, input, utilities::call)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SettingsNative_call(
    env: *mut JNIEnv,
    _: jclass,
    operation: jint,
    input: jobjectArray,
) -> jobjectArray {
    transfer(env, operation, input, settings::call)
}
