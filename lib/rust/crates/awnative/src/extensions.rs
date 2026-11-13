//! Data conversion only; extension decisions and error descriptions live in `extensions`.
use crate::jni::{self, Error, illegal};
use jni_sys::{JNIEnv, jclass, jint, jobjectArray};
use std::ptr;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ExtensionPolicyNative_call(
    env: *mut JNIEnv,
    _: jclass,
    operation: jint,
    input: jobjectArray,
) -> jobjectArray {
    // SAFETY: the JVM supplies this call's environment and array; all strings are copied before use.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let input = env
                .read_strings(input, "extension input")?
                .into_iter()
                .map(|v| v.ok_or(Error::NullPointer("extension field")))
                .collect::<jni::Result<Vec<_>>>()?;
            let output = extensions::call(operation, &input).map_err(|e| illegal(e.0))?;
            env.make_strings(&output)
        })
    }
}
