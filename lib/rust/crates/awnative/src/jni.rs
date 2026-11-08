//! Just enough JNI for the entry points: copying Java arrays and strings into Rust, writing
//! results back, calling a Kotlin method back, and throwing. Everything is copied in and out, so
//! no Java memory is held while Rust works on it.

use std::ffi::{CStr, CString};
use std::ptr;

use jni_sys::{JNIEnv, jarray, jobject, jobjectArray, jsize, jstring, jvalue};

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

    /// The string's UTF-16 units; `name` says which argument was null, if it was.
    pub fn read_string(&self, string: jstring, name: &'static str) -> Result<Vec<u16>> {
        self.read_nullable_string(string)?
            .ok_or(Error::NullPointer(name))
    }

    /// The string's UTF-16 units, or `None` if it is null.
    pub fn read_nullable_string(&self, string: jstring) -> Result<Option<Vec<u16>>> {
        if string.is_null() {
            return Ok(None);
        }
        let len = jni_call!(self, GetStringLength, string);
        let mut units = vec![0; len.max(0) as usize];
        jni_call!(self, GetStringRegion, string, 0, len, units.as_mut_ptr());
        self.check()?;
        Ok(Some(units))
    }

    /// A new Java string of these UTF-16 units, as they are: they needn't be valid UTF-16, as a
    /// Kotlin String needn't.
    pub fn make_string(&self, units: &[u16]) -> Result<jstring> {
        let len = jsize::try_from(units.len()).map_err(|_| illegal("string too long"))?;
        let string = jni_call!(self, NewString, units.as_ptr(), len);
        if string.is_null() {
            return Err(Error::Pending);
        }
        Ok(string)
    }

    /// A new `String[]` of these strings.
    pub fn make_strings(&self, strings: &[Vec<u16>]) -> Result<jobjectArray> {
        let len = jsize::try_from(strings.len()).map_err(|_| illegal("array too long"))?;
        let class = jni_call!(self, FindClass, c"java/lang/String".as_ptr());
        if class.is_null() {
            return Err(Error::Pending);
        }
        let array = jni_call!(self, NewObjectArray, len, class, ptr::null_mut());
        self.delete_local(class);
        if array.is_null() {
            return Err(Error::Pending);
        }
        // The index fits in a jsize, as the length does.
        for (i, units) in strings.iter().enumerate() {
            let string = self.make_string(units)?;
            jni_call!(self, SetObjectArrayElement, array, i as jsize, string);
            self.delete_local(string);
            self.check()?;
        }
        Ok(array)
    }

    /// Calls `object`'s method `name`, whose `signature` takes one object and returns a boolean,
    /// with `argument`; `what` says which argument `object` is, if it is null. R8 must keep the
    /// method's name (see lib/rust/consumer-rules.pro).
    pub fn call_boolean(
        &self,
        object: jobject,
        what: &'static str,
        (name, signature): (&CStr, &CStr),
        argument: jobject,
    ) -> Result<bool> {
        if object.is_null() {
            return Err(Error::NullPointer(what));
        }
        let class = jni_call!(self, GetObjectClass, object);
        let method = jni_call!(self, GetMethodID, class, name.as_ptr(), signature.as_ptr());
        self.delete_local(class);
        if method.is_null() {
            return Err(Error::Pending); // GetMethodID raised NoSuchMethodError.
        }
        let arguments = [jvalue { l: argument }];
        let result = jni_call!(self, CallBooleanMethodA, object, method, arguments.as_ptr());
        self.check()?;
        Ok(result != 0)
    }

    /// Frees a local reference this call made, before it returns.
    pub fn delete_local(&self, object: jobject) {
        jni_call!(self, DeleteLocalRef, object);
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
        self.delete_local(class);
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
