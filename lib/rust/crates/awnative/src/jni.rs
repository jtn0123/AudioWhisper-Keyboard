//! Just enough JNI for the entry points: copying Java arrays and strings into Rust, writing
//! results back, calling a Kotlin method back, and throwing. Everything is copied in and out, so
//! no Java memory is held while Rust works on it.

use std::ffi::{CStr, CString};
use std::ptr;

use jni_sys::{JNI_FALSE, JNI_TRUE, JNIEnv, jarray, jboolean, jmethodID, jobject, jobjectArray};
use jni_sys::{jsize, jstring, jvalue};

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

/// A Rust `bool` as a Java `boolean`.
pub fn boolean(value: bool) -> jboolean {
    if value { JNI_TRUE } else { JNI_FALSE }
}

/// `text` cut into pieces of these `lengths`, in order: how a long list of strings crosses JNI as
/// one string, rather than several calls a string. The lengths must cover `text` exactly.
pub fn cut<'a>(text: &'a [u16], lengths: &[i32]) -> Result<Vec<&'a [u16]>> {
    let mut rest = text;
    let mut pieces = Vec::with_capacity(lengths.len());
    for &length in lengths {
        let length = usize::try_from(length).map_err(|_| illegal("a negative length"))?;
        let (piece, after) = rest
            .split_at_checked(length)
            .ok_or_else(|| illegal("the lengths run past the text"))?;
        pieces.push(piece);
        rest = after;
    }
    if !rest.is_empty() {
        return Err(illegal("the lengths stop short of the text"));
    }
    Ok(pieces)
}

/// A Kotlin object's method, found by [`Env::method`], to call back during the native call that
/// found it.
#[derive(Clone, Copy, Debug)]
pub struct Method {
    object: jobject,
    id: jmethodID,
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

element!(
    jboolean,
    GetBooleanArrayRegion,
    NewBooleanArray,
    SetBooleanArrayRegion
);
element!(u16, GetCharArrayRegion, NewCharArray, SetCharArrayRegion);
element!(i8, GetByteArrayRegion, NewByteArray, SetByteArrayRegion);
element!(i32, GetIntArrayRegion, NewIntArray, SetIntArrayRegion);
element!(f32, GetFloatArrayRegion, NewFloatArray, SetFloatArrayRegion);
element!(i64, GetLongArrayRegion, NewLongArray, SetLongArrayRegion);

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

    /// Borrow direct native-order Float32 PCM for a synchronous call. The app's producer
    /// owns the buffer and must not mutate it until return; the closure cannot retain it.
    pub fn with_direct_f32<T>(
        &self,
        buffer: jobject,
        count: i32,
        body: impl FnOnce(&[f32]) -> Result<T>,
    ) -> Result<T> {
        if buffer.is_null() || count < 0 {
            return Err(illegal("invalid PCM buffer"));
        }
        let capacity = jni_call!(self, GetDirectBufferCapacity, buffer);
        if capacity < i64::from(count) * 4 {
            return Err(illegal("PCM capacity"));
        }
        let address = jni_call!(self, GetDirectBufferAddress, buffer).cast::<f32>();
        if count == 0 {
            return body(&[]);
        }
        if address.is_null() || !(address as usize).is_multiple_of(align_of::<f32>()) {
            return Err(illegal("unaligned PCM"));
        }
        // SAFETY: JNI pins the direct allocation for this call; capacity and alignment were
        // checked. The producer does not write while the synchronous consumer borrows it.
        body(unsafe { std::slice::from_raw_parts(address, count as usize) })
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

    /// The strings in a `String[]`, `None` for each null; `name` says which argument was null, if
    /// the array was.
    pub fn read_strings(
        &self,
        array: jobjectArray,
        name: &'static str,
    ) -> Result<Vec<Option<Vec<u16>>>> {
        let len = self.length(array, name)?;
        let mut strings = Vec::with_capacity(len);
        // The index fits in a jsize, as the length does.
        for i in 0..len {
            let string = jni_call!(self, GetObjectArrayElement, array, i as jsize);
            self.check()?;
            let units = self.read_nullable_string(string);
            if !string.is_null() {
                self.delete_local(string);
            }
            strings.push(units?);
        }
        Ok(strings)
    }

    /// The arrays in an array of arrays such as `int[][]`, `None` for each null; `name` says which
    /// argument was null, if the outer array was.
    #[cfg(not(target_os = "android"))]
    pub fn read_arrays<T: Element>(
        &self,
        array: jobjectArray,
        name: &'static str,
    ) -> Result<Vec<Option<Vec<T>>>> {
        let len = self.length(array, name)?;
        let mut arrays = Vec::with_capacity(len);
        // The index fits in a jsize, as the length does.
        for i in 0..len {
            let inner = jni_call!(self, GetObjectArrayElement, array, i as jsize);
            self.check()?;
            if inner.is_null() {
                arrays.push(None);
                continue;
            }
            let values = self.read(inner, name);
            self.delete_local(inner);
            arrays.push(Some(values?));
        }
        Ok(arrays)
    }

    /// A new Java string of these UTF-16 units, or null for `None`.
    pub fn make_nullable_string(&self, units: Option<&[u16]>) -> Result<jstring> {
        units.map_or(Ok(ptr::null_mut()), |units| self.make_string(units))
    }

    /// A new array of `class` (a JNI class name) holding what `make` makes of each item.
    fn make_array<T>(
        &self,
        class: &CStr,
        items: &[T],
        make: impl Fn(&T) -> Result<jobject>,
    ) -> Result<jobjectArray> {
        let len = jsize::try_from(items.len()).map_err(|_| illegal("array too long"))?;
        let class = jni_call!(self, FindClass, class.as_ptr());
        if class.is_null() {
            return Err(Error::Pending);
        }
        let array = jni_call!(self, NewObjectArray, len, class, ptr::null_mut());
        self.delete_local(class);
        if array.is_null() {
            return Err(Error::Pending);
        }
        // The index fits in a jsize, as the length does.
        for (i, item) in items.iter().enumerate() {
            let object = make(item)?;
            jni_call!(self, SetObjectArrayElement, array, i as jsize, object);
            self.delete_local(object);
            self.check()?;
        }
        Ok(array)
    }

    /// A new `String[]` of these strings.
    pub fn make_strings<S: AsRef<[u16]>>(&self, strings: &[S]) -> Result<jobjectArray> {
        self.make_array(c"java/lang/String", strings, |it| {
            self.make_string(it.as_ref())
        })
    }

    /// A new `Object[]` of what each of `makers` makes, such as arrays of different types.
    pub fn make_objects(&self, makers: &[&dyn Fn() -> Result<jobject>]) -> Result<jobjectArray> {
        self.make_array(c"java/lang/Object", makers, |make| make())
    }

    /// `object`'s method `name`, whose JNI `signature` is this, to call back; `what` says which
    /// argument `object` is, if it is null. R8 must keep the method's name (see
    /// lib/rust/consumer-rules.pro).
    pub fn method(
        &self,
        object: jobject,
        what: &'static str,
        (name, signature): (&CStr, &CStr),
    ) -> Result<Method> {
        if object.is_null() {
            return Err(Error::NullPointer(what));
        }
        let class = jni_call!(self, GetObjectClass, object);
        let id = jni_call!(self, GetMethodID, class, name.as_ptr(), signature.as_ptr());
        self.delete_local(class);
        if id.is_null() {
            return Err(Error::Pending); // GetMethodID raised NoSuchMethodError.
        }
        Ok(Method { object, id })
    }

    /// Calls `method`, which returns a boolean, with `arguments`, as its signature has them.
    pub fn call_boolean(&self, method: Method, arguments: &[jvalue]) -> Result<bool> {
        let (object, id) = (method.object, method.id);
        let result = jni_call!(self, CallBooleanMethodA, object, id, arguments.as_ptr());
        self.check()?;
        Ok(result != 0)
    }

    /// Calls `method`, which returns an int, with `arguments`, as its signature has them.
    pub fn call_int(&self, method: Method, arguments: &[jvalue]) -> Result<i32> {
        let (object, id) = (method.object, method.id);
        let result = jni_call!(self, CallIntMethodA, object, id, arguments.as_ptr());
        self.check()?;
        Ok(result)
    }

    /// Calls `method`, which returns an object, with `arguments`, as its signature has them: a
    /// local reference for the caller to free, or null.
    pub fn call_object(&self, method: Method, arguments: &[jvalue]) -> Result<jobject> {
        let (object, id) = (method.object, method.id);
        let result = jni_call!(self, CallObjectMethodA, object, id, arguments.as_ptr());
        if let Err(error) = self.check() {
            if !result.is_null() {
                self.delete_local(result);
            }
            return Err(error);
        }
        Ok(result)
    }

    /// What `call` gives a new Java string of these UTF-16 units, freeing the string after.
    pub fn with_string<R>(
        &self,
        units: &[u16],
        call: impl FnOnce(jstring) -> Result<R>,
    ) -> Result<R> {
        let string = self.make_string(units)?;
        let result = call(string);
        self.delete_local(string);
        result
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cuts_text_at_the_lengths() {
        let text: Vec<u16> = "toofab".encode_utf16().collect();
        let pieces = cut(&text, &[3, 0, 3]).ok();
        assert_eq!(pieces, Some(vec![&text[..3], &text[3..3], &text[3..]]));
        assert_eq!(cut(&[], &[]).ok(), Some(vec![]));
        assert!(cut(&text, &[3, 2]).is_err());
        assert!(cut(&text, &[3, 4]).is_err());
        assert!(cut(&text, &[7, -1]).is_err());
    }
}
