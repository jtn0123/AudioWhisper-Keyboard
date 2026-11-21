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
        ..METHOD
    }],
    ..CLASS
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
            ..METHOD
        }],
        ..CLASS
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
                ..METHOD
            },
            Method {
                name: "aMethodWhoseNameIsLongEnoughToWrap",
                doc: "Narrow arguments on a line too long, together on the next.",
                params: &[("a", Param::Int), ("b", Param::Long)],
                returns: Return::Boolean,
                ..METHOD
            },
            Method {
                name: "aMethodTakingFlagsLongEnoughToWrap",
                doc: "Narrow arguments that are expressions, one to a line all the same.",
                params: &[("first", Param::Boolean), ("second", Param::Boolean)],
                returns: Return::Boolean,
                ..METHOD
            },
            Method {
                name: "readsStrings",
                doc: "A string read that fits on its line and one that does not.",
                params: &[
                    ("text", Param::String),
                    ("textBeforeTheCursorHere", Param::String),
                ],
                returns: Return::Boolean,
                ..METHOD
            },
        ],
        ..CLASS
    },
    Class {
        name: "EveryType",
        doc: "Every parameter and return type, an instance method, an internal one, a declared \
              Rust path and lib/kotlin's package.",
        module: "every_type",
        kotlin: Module::Kotlin,
        public: true,
        methods: &[
            Method {
                name: "scalars",
                params: &[
                    ("flag", Param::Boolean),
                    ("unit", Param::Char),
                    ("scale", Param::Float),
                ],
                returns: Return::Double,
                instance: true,
                ..METHOD
            },
            Method {
                name: "texts",
                params: &[
                    ("maybe", Param::NullableString),
                    ("words", Param::Strings),
                    ("fields", Param::StringVararg),
                    ("holes", Param::NullableStrings),
                ],
                returns: Return::NullableString,
                internal: true,
                ..METHOD
            },
            Method {
                name: "arrays",
                params: &[
                    ("flags", Param::Array(Element::Boolean)),
                    ("bytes", Param::Array(Element::Byte)),
                    ("units", Param::Array(Element::Char)),
                    ("longs", Param::Array(Element::Long)),
                    ("floats", Param::NullableArray(Element::Float)),
                ],
                returns: Return::Array(Element::Char),
                rust: "arrays::read",
                ..METHOD
            },
            Method {
                name: "raw",
                doc: "An array to fill and an object of another package.",
                params: &[
                    ("out", Param::Out(Element::Int)),
                    ("buffer", Param::Object("java.nio.ByteBuffer?")),
                ],
                returns: Return::Object("Array<Any>", "jobjectArray"),
                ..METHOD
            },
            Method {
                name: "type",
                params: &[("match", Param::Long)],
                returns: Return::Strings,
                ..METHOD
            },
            Method {
                name: "nothing",
                params: &[("store", Param::Long)],
                returns: Return::Unit,
                env: true,
                ..METHOD
            },
            Method {
                name: "maybeInts",
                params: &[],
                returns: Return::NullableArray(Element::Int),
                ..METHOD
            },
            Method {
                name: "numbers",
                params: &[],
                returns: Return::Long,
                ..METHOD
            },
            Method {
                name: "fraction",
                params: &[],
                returns: Return::Float,
                ..METHOD
            },
            Method {
                name: "text",
                params: &[],
                returns: Return::String,
                ..METHOD
            },
            Method {
                name: "manyNumbers",
                params: &[
                    ("a", Param::Int),
                    ("b", Param::Int),
                    ("c", Param::Int),
                    ("d", Param::Int),
                    ("e", Param::Int),
                    ("f", Param::Int),
                    ("g", Param::Int),
                    ("h", Param::Boolean),
                ],
                returns: Return::Long,
                ..METHOD
            },
            Method {
                name: "manyThings",
                params: &[
                    ("store", Param::Long),
                    ("word", Param::String),
                    ("previous", Param::Strings),
                    ("taps", Param::Array(Element::Float)),
                    ("marks", Param::Out(Element::Int)),
                    ("count", Param::Int),
                    ("strength", Param::Int),
                ],
                returns: Return::Strings,
                ..METHOD
            },
            Method {
                name: "rows",
                params: &[("text", Param::String)],
                defaults: &[("text", "\"\"")],
                returns: Return::IntArrays,
                ..METHOD
            },
        ],
        ..CLASS
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
        .map(|(_, it)| it.path)
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
        for (_, file) in kotlin_files(classes) {
            assert!(file.text.ends_with("}\n") && !file.text.contains("\n\n\n"));
            for line in file.text.lines() {
                assert!(line.len() <= 120, "{}: {line}", file.path);
                assert_eq!(line.trim_end(), line, "{}", file.path);
                assert!(!line.contains('\t'), "{}", file.path);
            }
        }
    }
    let wrapped = &kotlin_files(STRESS)[1].1.text;
    assert!(
        wrapped
            .contains("    @JvmStatic external fun takesManyNumbers(\n        firstNumber: Int,\n")
    );
    assert!(wrapped.contains("        sixthNumber: Long,\n    ): Int\n"));
    let every = &kotlin_files(STRESS)[2].1.text;
    assert!(
        every.contains("    @JvmStatic external fun rows(text: String = \"\"): Array<IntArray>\n")
    );
    assert!(every.contains("    @JvmStatic @JvmName(\"texts\") internal external fun texts("));
}

/// Kotlin's hard keywords, which would not compile as a name there. Rust's are escaped.
const KEYWORDS: &[&str] = &[
    "as",
    "break",
    "class",
    "continue",
    "do",
    "else",
    "false",
    "for",
    "fun",
    "if",
    "in",
    "interface",
    "is",
    "null",
    "object",
    "package",
    "return",
    "super",
    "this",
    "throw",
    "true",
    "try",
    "typealias",
    "typeof",
    "val",
    "var",
    "when",
    "while",
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
            declarations.join(format!("{}.rs", class.file())).is_file(),
            "{}",
            class.file()
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
    assert_eq!(snake_case("isURL"), "is_url");
    assert_eq!(snake_case("parseHTMLText"), "parse_html_text");
    assert_eq!(snake_case("utf16Length"), "utf16_length");
    assert_eq!(snake_case("toUTF16"), "to_utf16");
    assert_eq!(rust_param("match"), "match_");
    assert_eq!(rust_param("lastCode"), "last_code");
    let keyword = Method {
        name: "type",
        ..METHOD
    };
    assert_eq!(callee(&EXAMPLE, &keyword), "crate::example::r#type");
    let elsewhere = Method {
        rust: "::shift::is_double_space_period",
        ..METHOD
    };
    assert_eq!(
        callee(&EXAMPLE, &elsewhere),
        "::shift::is_double_space_period"
    );
    let method = Method {
        name: "a_b",
        returns: Return::Int,
        ..METHOD
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
        files[0].1.path,
        "com/jtn0123/audiowhisper/rust/ExampleNative.kt"
    );
    assert_eq!(files[0].1.text, kotlin);
    let rust = "\
// Generated by lib/rust/crates/boundary from its declarations, src/declarations. Do not edit:
// awnative's build.rs writes this file into cargo's OUT_DIR, and src/boundary.rs includes it.

use jni_sys::{JNIEnv, jboolean, jclass, jint, jstring};

use crate::boundary::{Outcome, call};

/// The arguments of the methods that take more than a Rust function should one by one.
pub mod args {}

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
        call::<bool>(env, |env| {
            let text = env.read_string(text, \"text\")?;
            crate::example::is_word(&text, limit).result()
        })
    }
}
";
    assert_eq!(rust_file(&[EXAMPLE]), rust);
}

#[test]
fn docs_keep_their_paragraphs() {
    assert_eq!(
        crate::format::kdoc("One.\n\nTwo.", "    "),
        "    /**\n     * One.\n     *\n     * Two.\n     */\n"
    );
    assert_eq!(
        crate::format::rust_doc("One.\n\nTwo.", ""),
        "/// One.\n///\n/// Two.\n"
    );
}
