//! Rust objects that Kotlin holds: each crosses JNI as a `Long`, the address of a box. The Kotlin
//! wrapper frees it once, with a `java.lang.ref.Cleaner` when nothing can reach the wrapper, or
//! on closing it.

use jni_sys::jlong;

use crate::jni::{Result, illegal};

/// Boxes `value` for Kotlin to hold.
pub fn into_handle<T>(value: T) -> jlong {
    Box::into_raw(Box::new(value)) as jlong
}

/// The object behind a handle.
///
/// # Safety
/// `handle` must be 0 or come from [`into_handle`] of a `T` that hasn't been freed. The Kotlin
/// side frees an object only once nothing can reach it, so no call is still using it, or once it
/// is closed, which its other calls check under the same lock.
pub unsafe fn object<'a, T>(handle: jlong, what: &str) -> Result<&'a T> {
    // SAFETY: as the caller promises, a non-zero handle points to a live `T`.
    unsafe { (handle as *const T).as_ref() }.ok_or_else(|| illegal(format!("the {what} is closed")))
}

/// Frees what a handle from [`into_handle`] holds.
///
/// # Safety
/// `handle` must be 0, or come from [`into_handle`] of a `T`, and not be used again.
pub unsafe fn free<T>(handle: jlong) {
    if handle != 0 {
        // SAFETY: as the caller promises.
        drop(unsafe { Box::from_raw(handle as *mut T) });
    }
}
