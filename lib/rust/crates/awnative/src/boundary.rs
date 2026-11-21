//! The JNI entry points crates/boundary generates from its declarations, and what they share.
//! build.rs writes them into cargo's OUT_DIR. Each reads its arguments, calls the hand-written
//! function named after it in the module its class names (`ShiftNative.next` calls
//! `crate::shift::next`), or the function of another crate its declaration names, and hands the
//! result back through [`call`]. That function returns the value, or a `jni::Result` of it.

use std::ptr;

use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jdouble, jfloat, jint, jlong, jobject};

use crate::jni::{self, Env, Result};

/// What a hand-written function returns to Kotlin through a generated entry point: the JNI type
/// it crosses as, and what the entry point returns when it throws instead.
pub trait Returned {
    /// The JNI type of the declared return type.
    type Jni;
    /// What the entry point returns when it throws; Kotlin never sees it.
    const FALLBACK: Self::Jni;
    /// The value as JNI returns it, made in `env` if it is an object.
    fn into_jni(self, env: &Env) -> Result<Self::Jni>;
}

impl Returned for () {
    type Jni = ();
    const FALLBACK: () = ();
    fn into_jni(self, _: &Env) -> Result<()> {
        Ok(())
    }
}

impl Returned for bool {
    type Jni = jboolean;
    const FALLBACK: jboolean = JNI_FALSE;
    fn into_jni(self, _: &Env) -> Result<jboolean> {
        Ok(jni::boolean(self))
    }
}

/// Numbers cross as themselves.
macro_rules! number {
    ($($rust:ty => $jni:ty = $fallback:literal),*) => {$(
        impl Returned for $rust {
            type Jni = $jni;
            const FALLBACK: $jni = $fallback;
            fn into_jni(self, _: &Env) -> Result<$jni> {
                Ok(self)
            }
        }
    )*};
}

number!(i32 => jint = 0, i64 => jlong = 0, f32 => jfloat = 0.0, f64 => jdouble = 0.0);

/// Primitive arrays cross as new Java arrays of the same values.
macro_rules! array {
    ($($element:ty),*) => {$(
        impl Returned for Vec<$element> {
            type Jni = jobject;
            const FALLBACK: jobject = ptr::null_mut();
            fn into_jni(self, env: &Env) -> Result<jobject> {
                env.make(&self)
            }
        }
    )*};
}

array!(u8, i8, i32, i64, f32);

/// A string, as its UTF-16 units.
impl Returned for Vec<u16> {
    type Jni = jobject;
    const FALLBACK: jobject = ptr::null_mut();
    fn into_jni(self, env: &Env) -> Result<jobject> {
        env.make_string(&self)
    }
}

/// An `Array<String>`.
impl Returned for Vec<Vec<u16>> {
    type Jni = jobject;
    const FALLBACK: jobject = ptr::null_mut();
    fn into_jni(self, env: &Env) -> Result<jobject> {
        env.make_strings(&self)
    }
}

/// An object the hand-written function made itself.
impl Returned for jobject {
    type Jni = jobject;
    const FALLBACK: jobject = ptr::null_mut();
    fn into_jni(self, _: &Env) -> Result<jobject> {
        Ok(self)
    }
}

/// A nullable object: `None` crosses as `null`.
impl<T: Returned<Jni = jobject>> Returned for Option<T> {
    type Jni = jobject;
    const FALLBACK: jobject = ptr::null_mut();
    fn into_jni(self, env: &Env) -> Result<jobject> {
        self.map_or(Ok(ptr::null_mut()), |value| value.into_jni(env))
    }
}

/// What a hand-written function gives back: its value, or a `jni::Result` of it if it can fail.
/// The generated entry points call `result` on it, so either compiles.
pub trait Outcome<T> {
    /// The value, or the error to throw.
    fn result(self) -> Result<T>;
}

impl<T: Returned> Outcome<T> for T {
    fn result(self) -> Result<T> {
        Ok(self)
    }
}

impl<T: Returned> Outcome<T> for Result<T> {
    fn result(self) -> Result<T> {
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
    unsafe { jni::run(raw, T::FALLBACK, |env| body(env)?.into_jni(env)) }
}

/// The generated entry points.
mod generated {
    include!(concat!(env!("OUT_DIR"), "/boundary.rs"));
}
