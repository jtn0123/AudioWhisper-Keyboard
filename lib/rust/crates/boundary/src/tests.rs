use std::collections::HashSet;
use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use super::*;

/// A small class whose generated code these tests spell out.
const EXAMPLE: Class = Class {
    name: "ExampleNative",
    doc: "An example.",
    module: "example",
    methods: &[Method {
        name: "isWord",
        doc: "Whether `text` is a word.",
        params: &[("text", Param::String), ("limit", Param::Int)],
        returns: Return::Boolean,
    }],
};

/// Classes that take each layout rustfmt chooses between: a signature on one line, a call on
/// one line, with its arguments together on the next and one to a line, and a string read on one
/// line and wrapped after its `=`.
const STRESS: &[Class] = &[
    Class {
        name: "X",
        doc: "The shortest name, whose signature fits on one line.",
        module: "x",
        methods: &[Method {
            name: "y",
            doc: "Nothing in, a number out.",
            params: &[],
            returns: Return::Int,
        }],
    },
    Class {
        name: "AnUnusuallyLongNativeClassName",
        doc: "Long names, to wrap every line that can wrap. This doc is long enough that its \
              KDoc wraps over more than one line, as the class docs of the real bindings do.",
        module: "an_unusually_long_module_name",
        methods: &[
            Method {
                name: "takesManyNumbers",
                doc: "Arguments wider than rustfmt's call width, one to a line.",
                params: &[
                    ("firstNumber", Param::Int),
                    ("secondNumber", Param::Long),
                    ("thirdNumber", Param::Int),
                    ("fourthNumber", Param::Long),
                    ("fifthNumber", Param::Int),
                    ("sixthNumber", Param::Long),
                ],
                returns: Return::Int,
            },
            Method {
                name: "aMethodWhoseNameIsLongEnoughToWrap",
                doc: "Narrow arguments on a line too long, together on the next.",
                params: &[("a", Param::Int), ("b", Param::Long)],
                returns: Return::Boolean,
            },
            Method {
                name: "readsStrings",
                doc: "A string read that fits on its line and one that does not.",
                params: &[
                    ("text", Param::String),
                    ("textBeforeTheCursorHere", Param::String),
                ],
                returns: Return::Boolean,
            },
        ],
    },
];

/// `source` as the toolchain's rustfmt formats it, which `cargo fmt` would run.
fn rustfmt(source: &str) -> String {
    // rustup's rustfmt sits beside the cargo running these tests.
    let cargo = env::var_os("CARGO").expect("cargo sets CARGO for its tests");
    let mut child = Command::new(Path::new(&cargo).with_file_name("rustfmt"))
        .args(["--edition", "2024"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("rust-toolchain.toml installs rustfmt");
    let mut input = child.stdin.take().expect("piped stdin");
    input
        .write_all(source.as_bytes())
        .expect("rustfmt reads its input");
    drop(input);
    let output = child.wait_with_output().expect("rustfmt runs");
    let errors = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "rustfmt failed: {errors}");
    String::from_utf8(output.stdout).expect("rustfmt writes UTF-8")
}

#[test]
fn generation_is_deterministic() {
    assert_eq!(rust_file(CLASSES), rust_file(CLASSES));
    assert_eq!(kotlin_files(CLASSES), kotlin_files(CLASSES));
    let paths: HashSet<_> = kotlin_files(CLASSES)
        .into_iter()
        .map(|it| it.path)
        .collect();
    assert_eq!(paths.len(), CLASSES.len(), "one file per class");
}

#[test]
fn the_rust_is_formatted_as_rustfmt_would() {
    for classes in [CLASSES, STRESS, &[EXAMPLE]] {
        let source = rust_file(classes);
        assert_eq!(rustfmt(&source), source);
    }
}

#[test]
fn the_kotlin_fits_the_style() {
    for classes in [CLASSES, STRESS] {
        for file in kotlin_files(classes) {
            assert!(file.text.ends_with("}\n") && !file.text.contains("\n\n\n"));
            for line in file.text.lines() {
                assert!(line.len() <= 120, "{}: {line}", file.path);
                assert_eq!(line.trim_end(), line, "{}", file.path);
                assert!(!line.contains('\t'), "{}", file.path);
            }
        }
    }
    let wrapped = &kotlin_files(STRESS)[1].text;
    assert!(
        wrapped
            .contains("    @JvmStatic external fun takesManyNumbers(\n        firstNumber: Int,\n")
    );
    assert!(wrapped.contains("        sixthNumber: Long,\n    ): Int\n"));
}

/// Kotlin's and Rust's keywords, which would not compile as a name on one side or the other.
const KEYWORDS: &[&str] = &[
    "abstract",
    "as",
    "async",
    "await",
    "become",
    "box",
    "break",
    "class",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "final",
    "fn",
    "for",
    "fun",
    "gen",
    "if",
    "impl",
    "in",
    "interface",
    "is",
    "let",
    "loop",
    "macro",
    "match",
    "mod",
    "move",
    "mut",
    "null",
    "object",
    "override",
    "package",
    "priv",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "this",
    "throw",
    "trait",
    "true",
    "try",
    "type",
    "typealias",
    "typeof",
    "unsafe",
    "unsized",
    "use",
    "val",
    "var",
    "virtual",
    "when",
    "where",
    "while",
    "yield",
];

/// Whether `name` is a plain ASCII identifier starting with a letter of the given case.
fn plain(name: &str, upper: bool) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| {
        c.is_ascii_alphabetic() && c.is_ascii_uppercase() == upper && !KEYWORDS.contains(&name)
    }) && chars.all(|c| c.is_ascii_alphanumeric())
}

#[test]
fn declared_names_are_plain_and_distinct() {
    let declarations = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/declarations");
    let mut classes = HashSet::new();
    for class in CLASSES {
        assert!(
            plain(class.name, true) && classes.insert(class.name),
            "{}",
            class.name
        );
        assert!(
            declarations.join(format!("{}.rs", class.module)).is_file(),
            "{}",
            class.module
        );
        assert!(!class.doc.trim().is_empty() && !class.methods.is_empty());
        // One method of a name: overloaded JNI symbols are not supported.
        let mut methods = HashSet::new();
        for method in class.methods {
            assert!(
                plain(method.name, false) && methods.insert(method.name),
                "{}",
                method.name
            );
            assert!(!method.doc.trim().is_empty(), "{}", method.name);
            let mut params = HashSet::new();
            for (name, _) in method.params {
                // `env` is the entry point's own JNI environment.
                let distinct = *name != "env" && params.insert(*name);
                assert!(plain(name, false) && distinct, "{}.{name}", method.name);
            }
        }
    }
}

#[test]
fn names_cross_as_jni_and_rust_spell_them() {
    assert_eq!(snake_case("isDoubleSpacePeriod"), "is_double_space_period");
    assert_eq!(snake_case("next"), "next");
    let method = Method {
        name: "a_b",
        doc: "",
        params: &[],
        returns: Return::Int,
    };
    assert_eq!(
        jni_name(&EXAMPLE, &method),
        "Java_com_jtn0123_audiowhisper_rust_ExampleNative_a_1b"
    );
}

#[test]
fn an_example_comes_out_as_written() {
    let kotlin = "\
// Generated by lib/rust/crates/boundary from src/declarations/example.rs. Do not edit.
package com.jtn0123.audiowhisper.rust

/** An example. */
internal object ExampleNative {
    init {
        RustLibrary.load()
    }

    /** Whether `text` is a word. */
    @JvmStatic external fun isWord(text: String, limit: Int): Boolean
}
";
    let files = kotlin_files(&[EXAMPLE]);
    assert_eq!(
        files[0].path,
        "com/jtn0123/audiowhisper/rust/ExampleNative.kt"
    );
    assert_eq!(files[0].text, kotlin);
    let rust = "\
// Generated by lib/rust/crates/boundary from its declarations, src/declarations. Do not edit:
// awnative's build.rs writes this file into cargo's OUT_DIR, and src/boundary.rs includes it.

use jni_sys::{JNIEnv, jboolean, jclass, jint, jstring};

use crate::boundary::call;

// ExampleNative, declared in src/declarations/example.rs, calls crate::example.

/// Whether `text` is a word.
#[unsafe(no_mangle)]
pub extern \"system\" fn Java_com_jtn0123_audiowhisper_rust_ExampleNative_isWord(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    limit: jint,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        call(env, |env| {
            let text = env.read_string(text, \"text\")?;
            crate::example::is_word(&text, limit)
        })
    }
}
";
    assert_eq!(rust_file(&[EXAMPLE]), rust);
}
