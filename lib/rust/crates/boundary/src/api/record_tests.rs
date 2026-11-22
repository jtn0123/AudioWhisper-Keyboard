//! The records, bit fields, pairs and interfaces members read and declare.

use super::tests::NATIVE;
use super::*;
use crate::{CLASS, METHOD, Method};

#[test]
fn rows_read_as_records_and_numbers_as_bits() {
    const ROWS: &[Class] = &[Class {
        name: "RowsNative",
        module: "rows",
        methods: &[
            Method {
                name: "parts",
                returns: Return::Strings,
                ..METHOD
            },
            Method {
                name: "context",
                returns: Return::Strings,
                ..METHOD
            },
            Method {
                name: "facts",
                returns: Return::NullableArray(crate::Element::Int),
                ..METHOD
            },
        ],
        ..CLASS
    }];
    const BITS: &[Bit] = &[
        Bit {
            name: "took",
            shift: 2,
            ..BIT
        },
        Bit {
            name: "count",
            kind: BitKind::Int(8),
            shift: 8,
            internal: true,
            ..BIT
        },
        Bit {
            name: "decision",
            kind: BitKind::Maybe,
            shift: 16,
            ..BIT
        },
        Bit {
            name: "upper",
            kind: BitKind::Char,
            shift: 18,
            ..BIT
        },
        Bit {
            name: "from",
            doc: "The first, or -1.",
            kind: BitKind::Signed,
            shift: 32,
            ..BIT
        },
    ];
    const ROWS_API: &[Api] = &[Api {
        name: "RowRules",
        doc: "Rows.",
        module: "rows",
        native: "RowsNative",
        nested: &[
            Type::Enum {
                name: "Kind",
                doc: "A kind.",
                entries: &["WORDS", "MARK"],
            },
            Type::Record {
                name: "Part",
                doc: "A part.",
                data: false,
                constants: &[],
                fields: &[("kind", "Kind"), ("text", "String"), ("final", "Boolean")],
            },
            Type::Record {
                name: "Context",
                doc: "A context.",
                data: false,
                constants: &[],
                fields: &[("start", "Boolean"), ("words", "List<String>")],
            },
            Type::Record {
                name: "Facts",
                doc: "Facts.",
                data: false,
                constants: &[],
                fields: &[("digit", "Boolean"), ("upper", "Char"), ("kind", "Kind")],
            },
            Type::Bits {
                name: "Outcome",
                doc: "An outcome.",
                long: true,
                internal: false,
                fields: BITS,
            },
        ],
        members: &[
            Member {
                name: "parts",
                result: Out::Records("Part"),
                ..MEMBER
            },
            Member {
                name: "context",
                result: Out::Record("Context"),
                ..MEMBER
            },
            Member {
                name: "facts",
                result: Out::Record("Facts"),
                ..MEMBER
            },
        ],
        ..API
    }];
    let text = &api_files(ROWS_API, ROWS)[0].1.text;
    let expected = [
        "    fun parts(): List<Part> =\n        RowsNative.parts().asList().chunked(3) { \
         Part(Kind.entries[it[0].toInt()], it[1], WireFlag.decode(it[2])) }\n",
        "    fun context(): Context =\n        RowsNative.context().let { \
         Context(WireFlag.decode(it[0]), it.asList().subList(1, it.size)) }\n",
        "    fun facts(): Facts? = RowsNative.facts()?.let { \
         Facts(it[0] != 0, it[1].toChar(), Kind.entries[it[2]]) }\n",
        "    @JvmInline\n    value class Outcome(private val bits: Long) {\n",
        "        val took: Boolean get() = (bits ushr 2) and 1L != 0L\n",
        "        internal val count: Int get() = ((bits ushr 8) and 0xFFL).toInt()\n",
        "        val decision: Boolean?\n            get() = when (((bits ushr 16) and 0x3L)\
         .toInt()) {\n                1 -> true\n",
        "        val upper: Char get() = ((bits ushr 18) and 0xFFFFL).toInt().toChar()\n",
        "        /** The first, or -1. */\n        val from: Int get() = (bits shr 32).toInt()\n",
    ];
    for line in expected {
        assert!(text.contains(line), "missing {line:?} in\n{text}");
    }
}

#[test]
fn text_arguments_take_any_char_sequence() {
    const TEXT: &[Class] = &[Class {
        name: "TextNative",
        module: "text",
        methods: &[Method {
            name: "lastWord",
            params: &[("text", Param::NullableString), ("query", Param::String)],
            returns: Return::NullableString,
            ..METHOD
        }],
        ..CLASS
    }];
    const TEXT_API: &[Api] = &[Api {
        name: "TextRules",
        doc: "Text.",
        module: "text",
        native: "TextNative",
        members: &[Member {
            name: "lastWord",
            args: &[arg("text", ArgKind::Text), arg("query", ArgKind::Text)],
            ..MEMBER
        }],
        ..API
    }];
    let text = &api_files(TEXT_API, TEXT)[0].1.text;
    let expected = "    fun lastWord(text: CharSequence?, query: CharSequence): String? =\n        \
                    TextNative.lastWord(text?.toString(), query.toString())\n";
    assert!(text.contains(expected), "{text}");
}

#[test]
fn two_elements_read_as_a_pair() {
    const PAIRS: &[Class] = &[Class {
        name: "PairNative",
        module: "pairs",
        methods: &[
            Method {
                name: "action",
                returns: Return::Strings,
                ..METHOD
            },
            Method {
                name: "span",
                returns: Return::Array(crate::Element::Int),
                ..METHOD
            },
        ],
        ..CLASS
    }];
    const PAIR_API: &[Api] = &[Api {
        name: "PairRules",
        doc: "Pairs.",
        module: "pairs",
        native: "PairNative",
        members: &[
            Member {
                name: "action",
                result: Out::Pair("Int", "String"),
                ..MEMBER
            },
            Member {
                name: "span",
                result: Out::Pair("Int", "Boolean"),
                ..MEMBER
            },
        ],
        ..API
    }];
    let text = &api_files(PAIR_API, PAIRS)[0].1.text;
    for expected in [
        "    fun action(): Pair<Int, String> = PairNative.action().let { it[0].toInt() to it[1] }\n",
        "    fun span(): Pair<Int, Boolean> = PairNative.span().let { it[0] to (it[1] != 0) }\n",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn a_parameter_can_be_declared_as_another() {
    const URLS: &[Class] = &[Class {
        name: "UrlNative",
        module: "urls",
        methods: &[Method {
            name: "url",
            params: &[
                ("key", Param::String),
                ("names", Param::Strings),
                ("values", Param::Strings),
            ],
            returns: Return::String,
            ..METHOD
        }],
        ..CLASS
    }];
    const URL_API: &[Api] = &[Api {
        name: "UrlRules",
        doc: "URLs.",
        module: "urls",
        native: "UrlNative",
        members: &[Member {
            name: "url",
            args: &[
                Arg {
                    default: "emptyMap()",
                    ..arg(
                        "names",
                        ArgKind::As("params: Map<String, String>", "params.keys.toTypedArray()"),
                    )
                },
                arg("values", ArgKind::Fixed("params.values.toTypedArray()")),
            ],
            ..MEMBER
        }],
        ..API
    }];
    let text = &api_files(URL_API, URLS)[0].1.text;
    let expected = "    fun url(key: String, params: Map<String, String> = emptyMap()): String =\n        \
                    UrlNative.url(key, params.keys.toTypedArray(), params.values.toTypedArray())\n";
    assert!(text.contains(expected), "{text}");
}

#[test]
fn string_rows_read_as_records_each() {
    const ROWS: &[Class] = &[Class {
        name: "EntryNative",
        module: "entries",
        methods: &[
            Method {
                name: "entries",
                returns: Return::StringArrays,
                ..METHOD
            },
            Method {
                name: "lists",
                returns: Return::IntArrays,
                ..METHOD
            },
        ],
        ..CLASS
    }];
    const ENTRIES: &[Api] = &[Api {
        name: "EntryRules",
        doc: "Entries.",
        module: "entries",
        native: "EntryNative",
        nested: &[Type::Record {
            name: "Entry",
            doc: "An entry.",
            data: false,
            constants: &[],
            fields: &[
                ("value", "String"),
                ("count", "Int"),
                ("words", "List<String>"),
            ],
        }],
        members: &[
            Member {
                name: "entries",
                result: Out::Records("Entry"),
                ..MEMBER
            },
            Member {
                name: "lists",
                result: Out::List,
                ..MEMBER
            },
        ],
        ..API
    }];
    let text = &api_files(ENTRIES, ROWS)[0].1.text;
    for expected in [
        "    fun entries(): List<Entry> =\n        EntryNative.entries().map { \
         Entry(it[0], it[1].toInt(), it.asList().subList(2, it.size)) }\n",
        "    fun lists(): List<IntArray> = EntryNative.lists().asList()\n",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn arrays_read_as_a_record_with_constants() {
    const SCORES: &[Class] = &[Class {
        name: "ScoreNative",
        module: "scores",
        methods: &[
            Method {
                name: "scores",
                returns: crate::OBJECTS,
                ..METHOD
            },
            Method {
                name: "layout",
                returns: Return::NullableArray(crate::Element::Int),
                ..METHOD
            },
            Method {
                name: "best",
                returns: crate::NULLABLE_OBJECTS,
                ..METHOD
            },
        ],
        ..CLASS
    }];
    const SCORE_API: &[Api] = &[Api {
        name: "ScoreRules",
        doc: "Scores.",
        module: "scores",
        native: "ScoreNative",
        nested: &[Type::Record {
            name: "Scores",
            doc: "",
            data: false,
            fields: &[
                ("words", "IntArray"),
                ("weights", "FloatArray"),
                ("best", "Int"),
            ],
            constants: &[("NONE", "", "-1", "No word.")],
        }],
        members: &[
            Member {
                name: "scores",
                result: Out::Record("Scores"),
                ..MEMBER
            },
            Member {
                name: "layout",
                result: Out::NullableWrap("Layout"),
                ..MEMBER
            },
            Member {
                name: "best",
                result: Out::Record("Scores"),
                ..MEMBER
            },
        ],
        ..API
    }];
    let text = &api_files(SCORE_API, SCORES)[0].1.text;
    for expected in [
        "    class Scores(val words: IntArray, val weights: FloatArray, val best: Int) {\n        \
         companion object {\n            /** No word. */\n            const val NONE = -1\n        \
         }\n    }\n",
        "    fun scores(): Scores =\n        ScoreNative.scores().let { Scores(it[0] as IntArray, \
         it[1] as FloatArray, (it[2] as IntArray)[0]) }\n",
        "    fun layout(): Layout? = ScoreNative.layout()?.let { Layout(it) }\n",
        "    fun best(): Scores? =\n        ScoreNative.best()?.let { Scores(it[0] as IntArray, \
         it[1] as FloatArray, (it[2] as IntArray)[0]) }\n",
    ] {
        assert!(text.contains(expected), "{text}");
    }
}

#[test]
fn an_interface_lists_what_rust_calls_back() {
    const CALLBACKS: &[Api] = &[Api {
        name: "CallbackRules",
        doc: "Callbacks.",
        module: "callbacks",
        native: "ExampleNative",
        nested: &[Type::Interface {
            name: "Editor",
            doc: "What Rust asks.",
            members: &[
                ("val rule: String", ""),
                ("val language: String", ""),
                ("fun before(n: Int): String", "The last [n] characters."),
            ],
        }],
        ..API
    }];
    let text = &api_files(CALLBACKS, NATIVE)[0].1.text;
    let expected = "    /** What Rust asks. */\n    interface Editor {\n        val rule: String\n        \
                    val language: String\n\n        /** The last [n] characters. */\n        \
                    fun before(n: Int): String\n    }\n";
    assert!(text.contains(expected), "{text}");
}

#[test]
fn an_interface_of_one_function_takes_a_lambda() {
    const CALLBACKS: &[Api] = &[Api {
        name: "CallbackRules",
        doc: "Callbacks.",
        module: "callbacks",
        native: "ExampleNative",
        nested: &[Type::Interface {
            name: "Measure",
            doc: "",
            members: &[("fun measure(scale: Float): Float", "")],
        }],
        ..API
    }];
    let text = &api_files(CALLBACKS, NATIVE)[0].1.text;
    let expected = "    fun interface Measure {\n        fun measure(scale: Float): Float\n    }\n";
    assert!(text.contains(expected), "{text}");
}

#[test]
fn chained_calls_split_outside_their_lambdas() {
    let after = ".let { Strip(it[0], it.size) }?.also { fence(a); fence(b) }.toList()";
    assert_eq!(
        super::expr::chain(after),
        [
            ".let { Strip(it[0], it.size) }",
            "?.also { fence(a); fence(b) }",
            ".toList()"
        ]
    );
}
