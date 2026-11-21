//! The Kotlin–Rust boundary, generated from typed declarations. Each declared class becomes a
//! Kotlin `object` of `external fun`s and, for each of them, a Rust `extern "system"` function
//! that converts the JNI values and calls a hand-written function in awnative. Both sides come
//! from the one table in `declarations`, so their names and types cannot drift apart.
//!
//! It also writes the Kotlin side of crates/wire: the operation codes, packet fields and boolean
//! spelling those calls share, which the Rust side matches on directly.
//!
//! `cargo run -p boundary -- RUST KOTLIN` writes the Kotlin under the source roots of lib/rust
//! and lib/kotlin; Gradle's `generateNativeBoundary` task runs it before Kotlin compiles.
//! awnative's build.rs writes the Rust into cargo's OUT_DIR. Neither is checked in. docs/rust.md
//! says how to add a method, an operation or a field.
//!
//! The generator runs on the build computer only. It writes `unsafe` code as text; the code
//! itself compiles in awnative.

#![forbid(unsafe_code)]

mod constants;
mod declarations;
mod format;
mod kotlin;
mod rust;
#[cfg(test)]
mod tests;

pub use constants::{Module, Wire, wire_files};
pub use declarations::{CLASSES, WIRE};
pub use kotlin::{KotlinFile, kotlin_files};
pub use rust::rust_file;

/// The package of every generated class; lib/rust's `RustLibrary` loads the library for them.
pub const PACKAGE: &str = "com.jtn0123.audiowhisper.rust";

/// A Kotlin `object` of `external fun`s, and the awnative module that answers them.
#[derive(Debug)]
pub struct Class {
    /// The object's name, such as `ShiftNative`.
    pub name: &'static str,
    /// Its KDoc.
    pub doc: &'static str,
    /// The awnative module with a hand-written function for each method, named as the method is
    /// in snake case. The class is declared in `src/declarations/<module>.rs`.
    pub module: &'static str,
    pub methods: &'static [Method],
}

/// One `external fun`, and the Rust entry point behind it.
#[derive(Debug)]
pub struct Method {
    /// The Kotlin name. The hand-written Rust function's name is the same in snake case.
    pub name: &'static str,
    /// What it does, for the doc comment on both sides.
    pub doc: &'static str,
    /// Each parameter's Kotlin name and type. The Rust name is the same in snake case.
    pub params: &'static [(&'static str, Param)],
    pub returns: Return,
}

/// A parameter's type: the Kotlin type, its JNI type, and what the Rust function takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Param {
    /// `Int`, crossing as `jint`; the Rust function takes an `i32`.
    Int,
    /// `Long`, crossing as `jlong`; the Rust function takes an `i64`.
    Long,
    /// `String`, crossing as `jstring`; the Rust function takes its UTF-16 units as `&[u16]`,
    /// lone surrogates kept. A null throws `NullPointerException`.
    String,
}

/// A return type: the Kotlin type, its JNI type, and what the Rust function returns inside its
/// `jni::Result`. An error throws in Java instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Return {
    /// `Boolean`, crossing as `jboolean`; the Rust function returns a `bool`.
    Boolean,
    /// `Int`, crossing as `jint`; the Rust function returns an `i32`.
    Int,
}

impl Param {
    fn kotlin(self) -> &'static str {
        match self {
            Param::Int => "Int",
            Param::Long => "Long",
            Param::String => "String",
        }
    }

    fn jni(self) -> &'static str {
        match self {
            Param::Int => "jint",
            Param::Long => "jlong",
            Param::String => "jstring",
        }
    }
}

impl Return {
    fn kotlin(self) -> &'static str {
        match self {
            Return::Boolean => "Boolean",
            Return::Int => "Int",
        }
    }

    fn jni(self) -> &'static str {
        match self {
            Return::Boolean => "jboolean",
            Return::Int => "jint",
        }
    }
}

/// `name`, a Kotlin camelCase name, in Rust's snake case: `capsMode` is `caps_mode`.
fn snake_case(name: &str) -> String {
    let mut snake = String::with_capacity(name.len() + 4);
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            snake.push('_');
            snake.push(c.to_ascii_lowercase());
        } else {
            snake.push(c);
        }
    }
    snake
}

/// The JNI symbol the JVM looks up for `method` of `class`: `Java_`, the class's binary name and
/// the method's, with `_` escaped as `_1` and the package's dots as `_`.
fn jni_name(class: &Class, method: &Method) -> String {
    let escape = |name: &str| name.replace('_', "_1").replace('.', "_");
    format!(
        "Java_{}_{}_{}",
        escape(PACKAGE),
        escape(class.name),
        escape(method.name)
    )
}
