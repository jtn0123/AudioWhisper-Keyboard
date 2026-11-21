//! The types a method takes and returns: for each, the Kotlin type, the JNI type its entry point
//! declares, and how the generated Rust hands it to the hand-written function or takes it back.

/// The element type of a primitive array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Element {
    Boolean,
    Byte,
    Char,
    Int,
    Long,
    Float,
}

impl Element {
    /// The Kotlin array type, such as `IntArray`.
    fn kotlin(self) -> &'static str {
        match self {
            Element::Boolean => "BooleanArray",
            Element::Byte => "ByteArray",
            Element::Char => "CharArray",
            Element::Int => "IntArray",
            Element::Long => "LongArray",
            Element::Float => "FloatArray",
        }
    }

    /// The JNI array type, such as `jintArray`.
    fn jni(self) -> &'static str {
        match self {
            Element::Boolean => "jbooleanArray",
            Element::Byte => "jbyteArray",
            Element::Char => "jcharArray",
            Element::Int => "jintArray",
            Element::Long => "jlongArray",
            Element::Float => "jfloatArray",
        }
    }

    /// The Rust element type, as awnative's `jni::Element` implements it.
    fn rust(self) -> &'static str {
        match self {
            Element::Boolean => "u8",
            Element::Byte => "i8",
            Element::Char => "u16",
            Element::Int => "i32",
            Element::Long => "i64",
            Element::Float => "f32",
        }
    }
}

/// A parameter's type. Unless it says otherwise, the hand-written function takes the value
/// itself, a string's UTF-16 units as `&[u16]` with lone surrogates kept, and an array's
/// contents as a slice; a null where none is allowed throws `NullPointerException`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Param {
    /// `Boolean`, crossing as `jboolean`; the function takes a `bool`.
    Boolean,
    /// `Char`, crossing as `jchar`; the function takes a `u16`.
    Char,
    /// `Int`, crossing as `jint`; the function takes an `i32`.
    Int,
    /// `Long`, crossing as `jlong`; the function takes an `i64`.
    Long,
    /// `Float`, crossing as `jfloat`; the function takes an `f32`.
    Float,
    /// `String`, crossing as `jstring`; the function takes `&[u16]`.
    String,
    /// `String?`; the function takes `Option<&[u16]>`.
    NullableString,
    /// A primitive array, such as `IntArray`; the function takes `&[i32]`.
    Array(Element),
    /// A nullable primitive array, such as `IntArray?`; the function takes `Option<&[i32]>`.
    NullableArray(Element),
    /// `Array<String>`; the function takes `&[Vec<u16>]`. A null element throws.
    Strings,
    /// `Array<out String>`, which a `vararg` passes on; otherwise as [`Param::Strings`].
    StringVararg,
    /// `Array<String?>`; the function takes `&[Option<Vec<u16>>]`.
    NullableStrings,
    /// A primitive array the function fills in for the caller to read, such as `IntArray`; the
    /// function takes the call's environment and the array as its JNI reference, to write with
    /// `Env::write`.
    Out(Element),
    /// Any other object, of the given Kotlin type; a type from another package is written
    /// qualified, and imported. The function takes the call's environment and the object as a
    /// `jobject`, to call back or read itself.
    Object(&'static str),
}

/// A return type. The hand-written function returns it inside a `jni::Result`; an error throws
/// in Java instead, and the Kotlin caller sees no value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Return {
    /// Nothing; the function returns `()`.
    Unit,
    /// `Boolean`; the function returns a `bool`.
    Boolean,
    /// `Int`; the function returns an `i32`.
    Int,
    /// `Long`; the function returns an `i64`.
    Long,
    /// `Float`; the function returns an `f32`.
    Float,
    /// `Double`; the function returns an `f64`.
    Double,
    /// `String`; the function returns its UTF-16 units as a `Vec<u16>`.
    String,
    /// `String?`; the function returns an `Option<Vec<u16>>`.
    NullableString,
    /// A primitive array, such as `IntArray`; the function returns a `Vec<i32>`, or a
    /// `boundary::Chars` for a `CharArray`, whose `Vec<u16>` would be a string.
    Array(Element),
    /// A nullable primitive array, such as `IntArray?`; the function returns an `Option`.
    NullableArray(Element),
    /// `Array<String>`; the function returns a `Vec<Vec<u16>>`.
    Strings,
    /// Any other object, of the given Kotlin type and JNI type; the function takes the call's
    /// environment and returns the JNI reference it made.
    Object(&'static str, &'static str),
}

impl Param {
    /// The Kotlin type, qualified if it is from another package.
    pub fn kotlin(self) -> String {
        match self {
            Param::Boolean => "Boolean".into(),
            Param::Char => "Char".into(),
            Param::Int => "Int".into(),
            Param::Long => "Long".into(),
            Param::Float => "Float".into(),
            Param::String => "String".into(),
            Param::NullableString => "String?".into(),
            Param::Array(element) | Param::Out(element) => element.kotlin().into(),
            Param::NullableArray(element) => format!("{}?", element.kotlin()),
            Param::Strings => "Array<String>".into(),
            Param::StringVararg => "Array<out String>".into(),
            Param::NullableStrings => "Array<String?>".into(),
            Param::Object(kotlin) => kotlin.into(),
        }
    }

    pub fn jni(self) -> &'static str {
        match self {
            Param::Boolean => "jboolean",
            Param::Char => "jchar",
            Param::Int => "jint",
            Param::Long => "jlong",
            Param::Float => "jfloat",
            Param::String | Param::NullableString => "jstring",
            Param::Array(element) | Param::NullableArray(element) | Param::Out(element) => {
                element.jni()
            }
            Param::Strings | Param::StringVararg | Param::NullableStrings => "jobjectArray",
            Param::Object(_) => "jobject",
        }
    }

    /// Whether the hand-written function needs the call's environment for this parameter.
    pub fn raw(self) -> bool {
        matches!(self, Param::Out(_) | Param::Object(_))
    }

    /// How the entry point reads the parameter `rust`, named `name` in Kotlin, before the call,
    /// if it must, and the argument it passes the hand-written function.
    pub fn read(self, rust: &str, name: &str) -> (Option<String>, String) {
        let array =
            |element: Element| format!("env.read::<{}>({rust}, \"{name}\")?", element.rust());
        match self {
            Param::Boolean => (None, format!("{rust} != 0")),
            Param::Char | Param::Int | Param::Long | Param::Float | Param::Out(_) => {
                (None, rust.to_owned())
            }
            Param::Object(_) => (None, rust.to_owned()),
            Param::String => (
                Some(format!("env.read_string({rust}, \"{name}\")?")),
                format!("&{rust}"),
            ),
            Param::NullableString => (
                Some(format!("env.read_nullable_string({rust})?")),
                format!("{rust}.as_deref()"),
            ),
            Param::Array(element) => (Some(array(element)), format!("&{rust}")),
            Param::NullableArray(element) => (
                Some(format!("env.read_nullable::<{}>({rust})?", element.rust())),
                format!("{rust}.as_deref()"),
            ),
            Param::Strings | Param::StringVararg => (
                Some(format!("env.read_texts({rust}, \"{name}\")?")),
                format!("&{rust}"),
            ),
            Param::NullableStrings => (
                Some(format!("env.read_strings({rust}, \"{name}\")?")),
                format!("&{rust}"),
            ),
        }
    }
}

impl Return {
    pub fn kotlin(self) -> String {
        match self {
            Return::Unit => "Unit".into(),
            Return::Boolean => "Boolean".into(),
            Return::Int => "Int".into(),
            Return::Long => "Long".into(),
            Return::Float => "Float".into(),
            Return::Double => "Double".into(),
            Return::String => "String".into(),
            Return::NullableString => "String?".into(),
            Return::Array(element) => element.kotlin().into(),
            Return::NullableArray(element) => format!("{}?", element.kotlin()),
            Return::Strings => "Array<String>".into(),
            Return::Object(kotlin, _) => kotlin.into(),
        }
    }

    /// The JNI type, or `()` for nothing.
    pub fn jni(self) -> &'static str {
        match self {
            Return::Unit => "()",
            Return::Boolean => "jboolean",
            Return::Int => "jint",
            Return::Long => "jlong",
            Return::Float => "jfloat",
            Return::Double => "jdouble",
            Return::String | Return::NullableString => "jstring",
            Return::Array(element) | Return::NullableArray(element) => element.jni(),
            Return::Strings => "jobjectArray",
            Return::Object(_, jni) => jni,
        }
    }

    /// The Rust type the hand-written function returns, which the entry point names so that one
    /// of another type cannot compile.
    pub fn rust(self) -> String {
        match self {
            Return::Unit => "()".into(),
            Return::Boolean => "bool".into(),
            Return::Int => "i32".into(),
            Return::Long => "i64".into(),
            Return::Float => "f32".into(),
            Return::Double => "f64".into(),
            Return::String => "Vec<u16>".into(),
            Return::NullableString => "Option<Vec<u16>>".into(),
            Return::Array(Element::Char) => "Chars".into(),
            Return::Array(element) => format!("Vec<{}>", element.rust()),
            Return::NullableArray(Element::Char) => "Option<Chars>".into(),
            Return::NullableArray(element) => format!("Option<Vec<{}>>", element.rust()),
            Return::Strings => "Vec<Vec<u16>>".into(),
            Return::Object(_, jni) => jni.into(),
        }
    }

    /// Whether the hand-written function needs the call's environment to make the result.
    pub fn raw(self) -> bool {
        matches!(self, Return::Object(..))
    }
}
