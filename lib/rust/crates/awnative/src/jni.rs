//! Just enough JNI for the entry points: copying Java arrays into Rust, writing results back, and
//! throwing. Everything is copied in and out, so no Java memory is held while Rust works on it.

use std::ffi::{CStr, CString};

use jni_sys::{JNIEnv, jarray, jsize};

/// Why an entry point gave up: a Java exception to throw, or one the JVM already raised.
#[derive(Debug)]
pub enum Error {
    /// The JVM raised an exception, which is thrown when the entry point returns.
    Pending,
    IllegalArgument(String),
    NullPointer(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Shorthand for an [`Error::IllegalArgument`].
pub fn illegal(message: impl Into<String>) -> Error {
    Error::IllegalArgument(message.into())
}

/// The JNI environment of the current native call.
#[derive(Debug)]
pub struct Env {
    raw: *mut JNIEnv,
}

/// Calls a JNI function through the environment's function table.
macro_rules! jni_call {
    ($env:expr, $name:ident $(, $arg:expr)*) => {{
        let raw = $env.raw;
        // SAFETY: `raw` is the JVM's environment for this thread (see `run`), whose function
        // table every JVM fills in; the arguments are checked by each caller.
        unsafe { ((**raw).$name.expect(concat!("JNI has no ", stringify!($name))))(raw $(, $arg)*) }
    }};
}

/// A Java primitive type whose arrays an entry point reads or writes.
pub trait Element: Copy + Default {
    /// Copies `len` values from the start of `array` into `values`.
    fn get(env: &Env, array: jarray, len: jsize, values: *mut Self);
    /// A new array of `len` values, or null with an exception raised.
    fn new_array(env: &Env, len: jsize) -> jarray;
    /// Copies `len` values from `values` to the start of `array`.
    fn set(env: &Env, array: jarray, len: jsize, values: *const Self);
}

macro_rules! element {
    ($elem:ty, $get:ident, $new:ident, $set:ident) => {
        impl Element for $elem {
            fn get(env: &Env, array: jarray, len: jsize, values: *mut Self) {
                jni_call!(env, $get, array, 0, len, values);
            }

            fn new_array(env: &Env, len: jsize) -> jarray {
                jni_call!(env, $new, len)
            }

            fn set(env: &Env, array: jarray, len: jsize, values: *const Self) {
                jni_call!(env, $set, array, 0, len, values);
            }
        }
    };
}

element!(u16, GetCharArrayRegion, NewCharArray, SetCharArrayRegion);
element!(i32, GetIntArrayRegion, NewIntArray, SetIntArrayRegion);
element!(f32, GetFloatArrayRegion, NewFloatArray, SetFloatArrayRegion);

impl Env {
    fn length(&self, array: jarray, name: &'static str) -> Result<usize> {
        if array.is_null() {
            return Err(Error::NullPointer(name));
        }
        Ok(jni_call!(self, GetArrayLength, array).max(0) as usize)
    }

    /// Fails if the JVM raised an exception during the last call.
    fn check(&self) -> Result<()> {
        if jni_call!(self, ExceptionCheck) != 0 {
            Err(Error::Pending)
        } else {
            Ok(())
        }
    }

    /// The array's contents; `name` says which argument was null, if it was.
    pub fn read<T: Element>(&self, array: jarray, name: &'static str) -> Result<Vec<T>> {
        let len = self.length(array, name)?;
        let mut values = vec![T::default(); len];
        T::get(self, array, len as jsize, values.as_mut_ptr());
        self.check()?;
        Ok(values)
    }

    /// A new Java array holding `values`.
    pub fn make<T: Element>(&self, values: &[T]) -> Result<jarray> {
        let len = jsize::try_from(values.len()).map_err(|_| illegal("array too long"))?;
        let array = T::new_array(self, len);
        if array.is_null() {
            return Err(Error::Pending);
        }
        T::set(self, array, len, values.as_ptr());
        self.check()?;
        Ok(array)
    }

    /// Writes `values` to the start of `array`, which must be long enough.
    pub fn write<T: Element>(&self, array: jarray, values: &[T], name: &'static str) -> Result<()> {
        if self.length(array, name)? < values.len() {
            return Err(illegal(format!("{name} is too short")));
        }
        T::set(self, array, values.len() as jsize, values.as_ptr());
        self.check()
    }

    fn throw(&self, error: Error) {
        let (class, message) = match error {
            Error::Pending => return,
            Error::IllegalArgument(message) => (c"java/lang/IllegalArgumentException", message),
            Error::NullPointer(name) => {
                (c"java/lang/NullPointerException", format!("{name} is null"))
            }
        };
        self.throw_new(class, &message);
    }

    fn throw_new(&self, class: &CStr, message: &str) {
        let class = jni_call!(self, FindClass, class.as_ptr());
        if class.is_null() {
            return; // FindClass raised NoClassDefFoundError, which is thrown instead.
        }
        let message = CString::new(message.replace('\0', "")).unwrap_or_default();
        jni_call!(self, ThrowNew, class, message.as_ptr());
        jni_call!(self, DeleteLocalRef, class);
    }
}

/// Runs an entry point's `body`, throwing its error in Java and returning `fallback` if it fails.
///
/// # Safety
/// `raw` must be the environment the JVM passed to the native call this runs in.
pub unsafe fn run<T>(raw: *mut JNIEnv, fallback: T, body: impl FnOnce(&Env) -> Result<T>) -> T {
    let env = Env { raw };
    match body(&env) {
        Ok(value) => value,
        Err(error) => {
            env.throw(error);
            fallback
        }
    }
}
