//! What crosses the JNI boundary by number or name, declared once: the operation codes a `call`
//! entry point dispatches on, the positions of a packet's fields, and how a boolean is spelled.
//!
//! Each declaration is an ordinary Rust constant, which the crate that answers the call matches
//! on, and an entry in a [`Group`], from which crates/boundary writes the Kotlin side. Neither
//! side keeps a second copy of a number, so renumbering or renaming one here changes both, and
//! Kotlin that still uses an old name doesn't compile.
//!
//! This crate is a leaf: it depends on nothing, so the crates that answer calls build it for
//! Android as any other dependency, and the generator reads the same tables on the build computer
//! without depending on those crates. docs/rust.md says how to add an operation or a field.
//!
//! [`declare!`] checks each group as it compiles. Two operations of one group with the same code
//! fail the build:
//!
//! ```compile_fail
//! wire::declare! {
//!     /// Two operations that share a code.
//!     pub const OPS: enum ExampleOp(i32) {
//!         FIRST = 1,
//!         SECOND = 1,
//!     }
//! }
//! ```
//!
//! and so do two names that share a spelling:
//!
//! ```compile_fail
//! wire::declare! {
//!     /// Two operations that share a name on the wire.
//!     pub const OPS: enum ExampleOp(str) {
//!         FIRST = "first",
//!         SECOND = "first",
//!     }
//! }
//! ```
//!
//! A packet's fields are numbered in the order they are listed, from 0, so they cannot collide:
//!
//! ```
//! wire::declare! {
//!     /// A point.
//!     pub const POINT: fields ExamplePoint {
//!         X,
//!         Y,
//!     }
//! }
//! assert_eq!((X, Y, SIZE), (0, 1, 2));
//! ```

#![forbid(unsafe_code)]

pub mod backup;
pub mod crash;
pub mod dictation;
pub mod dictation_edges;
pub mod editor;
pub mod extensions;
pub mod flag;
pub mod gif;
pub mod ime_subtype;
pub mod ime_theme;
pub mod ime_window;
pub mod input;
pub mod key_graph;
pub mod nlp;
pub mod playback;
pub mod reading;
pub mod render;
pub mod settings;
pub mod settings_ui;
pub mod smartbar;
pub mod stores;
pub mod typing;
pub mod utilities;

#[cfg(test)]
mod tests;

/// A declared group: one operation set, packet or set of codes, and the Kotlin type it becomes.
#[derive(Debug)]
pub struct Group {
    /// The generated Kotlin type, such as `SettingsOp`. Groups of one name share one object.
    pub kotlin: &'static str,
    pub form: Form,
    /// The Rust module that declares it, such as `wire::settings`.
    pub path: &'static str,
    /// What it is, for the doc comment on both sides.
    pub doc: &'static str,
    pub constants: &'static [Constant],
}

/// The shape of the generated Kotlin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Form {
    /// Operations Kotlin sends: an `enum class` whose entries carry their `code`, so a call can't
    /// be given a bare number or another group's operation.
    Enum,
    /// Codes Kotlin reads or writes as plain values: an `object` of `const val`s.
    Object,
    /// A packet's field positions, from 0 in order: an `object` of `const val`s and `SIZE`.
    Fields,
    /// How a boolean is spelled: an `object` of `TRUE` and `FALSE`, with `encode` and `decode`.
    Flag,
}

/// One declared constant.
#[derive(Debug)]
pub struct Constant {
    /// Its name on both sides, in SCREAMING_SNAKE_CASE.
    pub name: &'static str,
    pub doc: &'static str,
    pub value: Value,
}

/// A constant's value: a code, a field position, or a name on the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Value {
    Int(i32),
    Text(&'static str),
}

/// Fails the build, as `declare!` runs it on every group, if two constants share a value or a
/// name on the wire holds a character a Kotlin string literal would need to escape.
pub const fn check(group: &Group) {
    let constants = group.constants;
    let mut i = 0;
    while i < constants.len() {
        if let Value::Text(text) = constants[i].value
            && !plain(text)
        {
            panic!("a name on the wire must be printable ASCII without quotes, `\\` or `$`");
        }
        let mut j = i + 1;
        while j < constants.len() {
            if same(constants[i].value, constants[j].value) {
                panic!("two constants of one group share a value");
            }
            j += 1;
        }
        i += 1;
    }
}

const fn plain(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let byte = bytes[i];
        if !byte.is_ascii_graphic() || matches!(byte, b'"' | b'\\' | b'$' | b'\'') {
            return false;
        }
        i += 1;
    }
    !bytes.is_empty()
}

const fn same(a: Value, b: Value) -> bool {
    match (a, b) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Text(a), Value::Text(b)) => {
            let (a, b) = (a.as_bytes(), b.as_bytes());
            if a.len() != b.len() {
                return false;
            }
            let mut i = 0;
            while i < a.len() {
                if a[i] != b[i] {
                    return false;
                }
                i += 1;
            }
            true
        }
        _ => false,
    }
}

/// Declares one [`Group`] as constants in the current module and a `Group` named `$group`:
///
/// - `pub const OPS: enum SettingsOp(i32) { CATALOGUE = 0, … }`: operations Kotlin sends, as
///   `i32` codes or, with `(str)`, names. Kotlin gets an `enum class`.
/// - `pub const CODES: object RenderProtocol(i32) { KEY_DOWN = 0, … }`: plain codes. Kotlin gets
///   an `object` of `const val`s.
/// - `pub const CONTEXT: fields NlpContext { INPUT_TYPE, … }`: a packet's field positions, as
///   `usize` from 0 in order, and `SIZE`, how many there are.
/// - `pub const FLAG: flag WireFlag { TRUE = "1", FALSE = "0" }`: how a boolean is spelled.
///
/// `///` comments on the group and on each constant become doc comments on both sides. Every
/// group is checked as it compiles: no two of its constants may share a value.
#[macro_export]
macro_rules! declare {
    (
        $(#[doc = $doc:literal])*
        pub const $group:ident: enum $kotlin:ident($ty:ident) { $($body:tt)* }
    ) => {
        $crate::declare!(@values Enum, [$($doc)*], $group, $kotlin, $ty, $($body)*);
    };
    (
        $(#[doc = $doc:literal])*
        pub const $group:ident: object $kotlin:ident($ty:ident) { $($body:tt)* }
    ) => {
        $crate::declare!(@values Object, [$($doc)*], $group, $kotlin, $ty, $($body)*);
    };
    (
        $(#[doc = $doc:literal])*
        pub const $group:ident: flag $kotlin:ident {
            $(#[doc = $yes_doc:literal])* TRUE = $yes:literal,
            $(#[doc = $no_doc:literal])* FALSE = $no:literal $(,)?
        }
    ) => {
        $crate::declare!(
            @values Flag, [$($doc)*], $group, $kotlin, str,
            $(#[doc = $yes_doc])* TRUE = $yes,
            $(#[doc = $no_doc])* FALSE = $no,
        );
    };
    (
        $(#[doc = $doc:literal])*
        pub const $group:ident: fields $kotlin:ident {
            $($(#[doc = $field_doc:literal])* $name:ident),+ $(,)?
        }
    ) => {
        $crate::declare!(@fields [] $($(#[doc = $field_doc])* $name,)+);
        /// How many fields the packet has.
        pub const SIZE: usize = <[&str]>::len(&[$(stringify!($name)),+]);
        $(#[doc = $doc])*
        pub const $group: $crate::Group = $crate::Group {
            kotlin: stringify!($kotlin),
            form: $crate::Form::Fields,
            path: module_path!(),
            doc: concat!($($doc, "\n",)*),
            constants: &[$($crate::Constant {
                name: stringify!($name),
                doc: concat!($($field_doc, "\n",)*),
                value: $crate::Value::Int($name as i32),
            }),+],
        };
        const _: () = $crate::check(&$group);
    };
    (@fields [$($done:ident)*]) => {};
    (
        @fields [$($done:ident)*] $(#[doc = $field_doc:literal])* $name:ident, $($rest:tt)*
    ) => {
        $(#[doc = $field_doc])*
        pub const $name: usize = <[&str]>::len(&[$(stringify!($done)),*]);
        $crate::declare!(@fields [$($done)* $name] $($rest)*);
    };
    (
        @values $form:ident, [$($doc:literal)*], $group:ident, $kotlin:ident, $ty:ident,
        $($(#[doc = $value_doc:literal])* $name:ident = $value:expr),+ $(,)?
    ) => {
        $($(#[doc = $value_doc])* pub const $name: $crate::declare!(@type $ty) = $value;)+
        $(#[doc = $doc])*
        pub const $group: $crate::Group = $crate::Group {
            kotlin: stringify!($kotlin),
            form: $crate::Form::$form,
            path: module_path!(),
            doc: concat!($($doc, "\n",)*),
            constants: &[$($crate::Constant {
                name: stringify!($name),
                doc: concat!($($value_doc, "\n",)*),
                value: $crate::declare!(@value $ty, $name),
            }),+],
        };
        const _: () = $crate::check(&$group);
    };
    (@type i32) => { i32 };
    (@type str) => { &'static str };
    (@value i32, $name:ident) => { $crate::Value::Int($name) };
    (@value str, $name:ident) => { $crate::Value::Text($name) };
}
