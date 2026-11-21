//! The JNI entry points crates/boundary generates from its declarations, and what they share.
//! build.rs writes them into cargo's OUT_DIR. Each reads its strings, calls the hand-written
//! function named after it in the module its class names (`ShiftNative.next` calls
//! `crate::shift::next`), and hands the result back through [`call`].

use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jint};

use crate::jni::{self, Env, Result};

/// What a hand-written function returns to Kotlin through a generated entry point: the JNI type
/// it crosses as, and what the entry point returns when it throws instead.
pub trait Returned {
    /// The JNI type of the declared return type.
    type Jni;
    /// What the entry point returns when it throws; Kotlin never sees it.
    const FALLBACK: Self::Jni;
    /// The value as JNI returns it.
    fn into_jni(self) -> Self::Jni;
}

impl Returned for bool {
    type Jni = jboolean;
    const FALLBACK: jboolean = JNI_FALSE;
    fn into_jni(self) -> jboolean {
        jni::boolean(self)
    }
}

impl Returned for i32 {
    type Jni = jint;
    const FALLBACK: jint = 0;
    fn into_jni(self) -> jint {
        self
    }
}

/// Runs a generated entry point's `body`, returning its value to Kotlin, or throwing its error in
/// Java and returning the fallback.
///
/// # Safety
/// `raw` must be the environment the JVM passed to the native call this runs in.
pub unsafe fn call<T: Returned>(raw: *mut JNIEnv, body: impl FnOnce(&Env) -> Result<T>) -> T::Jni {
    // SAFETY: the caller promises what `run` needs.
    unsafe { jni::run(raw, T::FALLBACK, |env| body(env).map(T::into_jni)) }
}

/// The generated entry points.
mod generated {
    include!(concat!(env!("OUT_DIR"), "/boundary.rs"));
}
