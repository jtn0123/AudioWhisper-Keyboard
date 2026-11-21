//! The Kotlin–Rust boundary, generated from typed declarations. Each declared class becomes a
//! Kotlin `object` of `external fun`s and, for each of them, a Rust `extern "system"` function
//! that converts the JNI values and calls a hand-written function in awnative. Both sides come
//! from the one table in `declarations`, so their names and types cannot drift apart.
//!
//! Over those objects it writes the public Kotlin the bindings used to write by hand: objects
//! whose members forward to native methods, classes that own a native handle, and the records
//! and enums they use (see `api`). What is left to write by hand is logic, as extensions.
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

mod api;
mod constants;
mod declarations;
mod format;
mod kotlin;
mod rust;
#[cfg(test)]
mod tests;
mod types;

pub use api::{
    API, Api, Arg, ArgKind, HANDLE, Handle, Kind, MEMBER, Member, Out, Shape, Type, api_files, arg,
    default, type_files,
};
pub use constants::{Module, Wire, wire_files};
pub use declarations::{APIS, CLASSES, TYPES, WIRE};
pub use kotlin::{KotlinFile, kotlin_files};
pub use rust::rust_file;
pub use types::{Element, Param, Return};

/// The package of every class generated in lib/rust; its `RustLibrary` loads the library for them.
pub const PACKAGE: &str = "com.jtn0123.audiowhisper.rust";

/// A Kotlin `object` of `external fun`s, and the awnative module that answers them.
#[derive(Debug)]
pub struct Class {
    /// The object's name, such as `ShiftNative`, and so its JNI symbols'.
    pub name: &'static str,
    /// Its KDoc.
    pub doc: &'static str,
    /// The awnative module with a hand-written function for each method, named as the method is
    /// in snake case.
    pub module: &'static str,
    /// The file the class is declared in, `src/declarations/<file>.rs`, when not the module's
    /// name.
    pub file: &'static str,
    /// The Gradle module the object is generated in, whose package its JNI symbols name.
    pub kotlin: Module,
    /// Whether the object is public, for one whose methods are themselves the Kotlin API, rather
    /// than `internal` behind a wrapper.
    pub public: bool,
    pub methods: &'static [Method],
}

/// A class's defaults, for the fields a declaration leaves out with `..CLASS`.
pub const CLASS: Class = Class {
    name: "",
    doc: "",
    module: "",
    file: "",
    kotlin: Module::Rust,
    public: false,
    methods: &[],
};

impl Class {
    /// The declarations file's name.
    pub fn file(&self) -> &'static str {
        if self.file.is_empty() {
            self.module
        } else {
            self.file
        }
    }
}

/// One `external fun`, and the Rust entry point behind it.
#[derive(Debug)]
pub struct Method {
    /// The Kotlin name. The hand-written Rust function's name is the same in snake case.
    pub name: &'static str,
    /// What it does, for the doc comment on both sides; empty when the hand-written function's
    /// own doc says it.
    pub doc: &'static str,
    /// Each parameter's Kotlin name and type. The Rust name is the same in snake case.
    pub params: &'static [(&'static str, Param)],
    pub returns: Return,
    /// Whether it is a member of the object rather than `@JvmStatic`, so that its entry point
    /// takes the object, a `jobject`, rather than the class.
    pub instance: bool,
    /// Default values of parameters, as Kotlin: (name, value).
    pub defaults: &'static [(&'static str, &'static str)],
    /// Whether the `external fun` is `internal`, for a hand-written extension in its module to
    /// call rather than the module's users. It keeps its JVM name, and so its JNI symbol, with
    /// `@JvmName`.
    pub internal: bool,
    /// The function's path under the module, when it is not the method's name in snake case; or,
    /// starting `crate::`, one in another of awnative's modules, or `::`, a function of another
    /// crate, called directly.
    pub rust: &'static str,
    /// Whether the hand-written function takes the call's `&Env` first. A parameter or result
    /// it handles as a JNI reference implies it.
    pub env: bool,
}

/// A method's defaults, for the fields a declaration leaves out with `..METHOD`.
pub const METHOD: Method = Method {
    name: "",
    doc: "",
    params: &[],
    returns: Return::Unit,
    instance: false,
    defaults: &[],
    internal: false,
    rust: "",
    env: false,
};

impl Method {
    /// Whether the hand-written function takes the call's environment.
    fn takes_env(&self) -> bool {
        self.env || self.returns.raw() || self.params.iter().any(|(_, param)| param.raw())
    }
}

/// Rust's keywords. A Rust name that is one is escaped: a function's raw, a parameter's with a
/// trailing underscore, as the entry point's parameter list must stay plain words.
const RUST_KEYWORDS: &[&str] = &[
    "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
    "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "gen", "if", "impl",
    "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
    "return", "self", "static", "struct", "super", "trait", "true", "try", "type", "typeof",
    "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

/// `name`, a Kotlin camelCase name, in Rust's snake case, reading an acronym as one word:
/// `capsMode` is `caps_mode`, `isURL` is `is_url` and `parseHTMLText` is `parse_html_text`.
fn snake_case(name: &str) -> String {
    let chars: Vec<char> = name.chars().collect();
    let mut snake = String::with_capacity(name.len() + 4);
    for (i, &c) in chars.iter().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            let before = chars[i - 1];
            let next_lower = chars.get(i + 1).is_some_and(char::is_ascii_lowercase);
            if !before.is_ascii_uppercase() || next_lower {
                snake.push('_');
            }
        }
        snake.push(c.to_ascii_lowercase());
    }
    snake
}

/// The Rust name of the parameter `name`: in snake case, with a trailing underscore if that is
/// a keyword.
fn rust_param(name: &str) -> String {
    let snake = snake_case(name);
    if RUST_KEYWORDS.contains(&snake.as_str()) {
        snake + "_"
    } else {
        snake
    }
}

/// The function `method` of `class` calls: its declared path, or its name in snake case, raw if
/// that is a keyword, under the class's module; or a declared path from `crate::` or `::`.
fn callee(class: &Class, method: &Method) -> String {
    let name = if method.rust.is_empty() {
        let snake = snake_case(method.name);
        if RUST_KEYWORDS.contains(&snake.as_str()) {
            format!("r#{snake}")
        } else {
            snake
        }
    } else if method.rust.starts_with("::") || method.rust.starts_with("crate::") {
        return method.rust.to_owned();
    } else {
        method.rust.to_owned()
    };
    format!("crate::{}::{name}", class.module)
}

/// The JNI symbol the JVM looks up for `method` of `class`: `Java_`, the class's binary name and
/// the method's, with `_` escaped as `_1` and the package's dots as `_`.
fn jni_name(class: &Class, method: &Method) -> String {
    let escape = |name: &str| name.replace('_', "_1").replace('.', "_");
    format!(
        "Java_{}_{}_{}",
        escape(class.kotlin.package()),
        escape(class.name),
        escape(method.name)
    )
}
