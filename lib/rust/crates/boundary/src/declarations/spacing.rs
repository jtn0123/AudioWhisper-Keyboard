//! `SpacingNative`, the spaces the keyboard types by itself, answered by awnative's `spacing`
//! module and the spacing crate, and `SpacingRules`, the Kotlin over it.

use crate::{
    API, Api, Arg, ArgKind, BIT, Bit, BitKind, CLASS, Class, Kind, MEMBER, METHOD, Member, Method,
    Out, Param, Return, Type, arg,
};

/// What is known before any text is looked up, as `SpacingRules.Typing`'s fields.
const TYPING: [(&str, Param); 6] = [
    ("autoSpacePunctuation", Param::Boolean),
    ("rawEditor", Param::Boolean),
    ("keyVariation", Param::Int),
    ("selectionValid", Param::Boolean),
    ("selectionStart", Param::Int),
    ("inputType", Param::Int),
];

const EDITOR: (&str, Param) = ("editor", Param::Object("SpacingRules.Editor"));

/// `typing`'s fields, passed one by one.
const TYPING_ARGS: [Arg; 6] = [
    arg(
        "autoSpacePunctuation",
        ArgKind::As("typing: Typing", "typing.autoSpacePunctuation"),
    ),
    arg("rawEditor", ArgKind::Fixed("typing.rawEditor")),
    arg("keyVariation", ArgKind::Fixed("typing.keyVariation")),
    arg("selectionValid", ArgKind::Fixed("typing.selectionValid")),
    arg("selectionStart", ArgKind::Fixed("typing.selectionStart")),
    arg("inputType", ArgKind::Fixed("typing.inputType")),
];

/// A state function of the spacing crate.
const fn state(
    name: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
    rust: &'static str,
) -> Method {
    Method {
        name,
        params,
        returns,
        rust,
        ..METHOD
    }
}

pub const SPACING_NATIVE: Class = Class {
    name: "SpacingNative",
    doc: "The spacing rules' entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/spacing.rs and the spacing crate. [SpacingRules] wraps \
          them.",
    module: "spacing",
    methods: &[
        Method {
            name: "commitChar",
            params: &[
                ("text", Param::String),
                TYPING[0],
                TYPING[1],
                TYPING[2],
                TYPING[3],
                TYPING[4],
                TYPING[5],
                ("autoSpace", Param::Int),
                ("phantomSpace", Param::Int),
                EDITOR,
            ],
            returns: Return::Int,
            ..METHOD
        },
        Method {
            name: "phantomSpaceBefore",
            params: &[
                ("text", Param::String),
                TYPING[0],
                TYPING[1],
                TYPING[2],
                TYPING[3],
                TYPING[4],
                TYPING[5],
                ("phantomSpace", Param::Int),
                ("forceActive", Param::Boolean),
                EDITOR,
            ],
            returns: Return::Boolean,
            ..METHOD
        },
        state(
            "supportsAutoSpace",
            &[("language", Param::String)],
            Return::Boolean,
            "::spacing::spaces::supports_auto_space",
        ),
        state("fallbackRule", &[], Return::Strings, ""),
        state("inactive", &[], Return::Int, ""),
        state(
            "autoSpaceActive",
            &[("stayActiveNextUpdate", Param::Boolean)],
            Return::Int,
            "::spacing::state::auto_space_active",
        ),
        state(
            "phantomSpaceActive",
            &[
                ("showComposingRegion", Param::Boolean),
                ("stayActiveNextUpdate", Param::Boolean),
            ],
            Return::Int,
            "::spacing::state::phantom_space_active",
        ),
        state(
            "isActive",
            &[("state", Param::Int)],
            Return::Boolean,
            "::spacing::state::is_active",
        ),
        state(
            "showsComposingRegion",
            &[("state", Param::Int)],
            Return::Boolean,
            "::spacing::state::shows_composing_region",
        ),
        state(
            "afterUpdate",
            &[("state", Param::Int)],
            Return::Int,
            "::spacing::state::after_update",
        ),
        state(
            "keepsCandidate",
            &[("state", Param::Int)],
            Return::Boolean,
            "::spacing::state::keeps_candidate",
        ),
    ],
    ..CLASS
};

pub const SPACING_RULES: Api = Api {
    name: "SpacingRules",
    doc: "The spaces the keyboard types by itself, in Rust (lib/rust/crates/spacing): a phantom \
          space owed after a picked or swiped word, auto-spacing after punctuation, and the two \
          flags' states, which the caller keeps as plain `Int`s.\n\n\
          The rules look text up only once they get far enough to need it, as the Kotlin they \
          replace did: Rust asks [Editor] for the rule, the text before the cursor and the \
          language only then.",
    module: "spacing",
    native: "SpacingNative",
    nested: &[
        Type::Record {
            name: "Rule",
            doc: "The symbols a punctuation rule lists for each kind of space.",
            data: false,
            constants: &[],
            fields: &[
                ("precedingAutoSpace", "String"),
                ("followingAutoSpace", "String"),
                ("precedingPhantomSpace", "String"),
                ("followingPhantomSpace", "String"),
            ],
        },
        Type::Interface {
            name: "Editor",
            doc: "What the rules look up when they need it.",
            members: &[
                ("val rule: Rule", ""),
                (
                    "val language: String",
                    "The subtype's primary language, as `Locale.getLanguage` gives it.",
                ),
                (
                    "fun textBeforeCursor(n: Int): String",
                    "The last [n] characters before the cursor.",
                ),
                ("val currentWordText: String", ""),
            ],
        },
        Type::Record {
            name: "Typing",
            doc: "What is known before any text is looked up; [inputType] is the field's raw \
                  `EditorInfo.inputType`.",
            data: false,
            constants: &[],
            fields: &[
                ("autoSpacePunctuation", "Boolean"),
                ("rawEditor", "Boolean"),
                ("keyVariation", "Int"),
                ("selectionValid", "Boolean"),
                ("selectionStart", "Int"),
                ("inputType", "Int"),
            ],
        },
        Type::Bits {
            name: "Spaces",
            doc: "What typing a character does with spaces, and the flags after it.",
            long: false,
            internal: true,
            fields: &[
                Bit {
                    name: "deletePreviousSpace",
                    ..BIT
                },
                Bit {
                    name: "spaceBefore",
                    shift: 1,
                    ..BIT
                },
                Bit {
                    name: "spaceAfter",
                    shift: 2,
                    ..BIT
                },
                Bit {
                    name: "autoSpace",
                    kind: BitKind::Int(8),
                    shift: 8,
                    ..BIT
                },
                Bit {
                    name: "phantomSpace",
                    kind: BitKind::Int(8),
                    shift: 16,
                    ..BIT
                },
            ],
        },
    ],
    members: &[
        Member {
            name: "commitChar",
            doc: "What typing the character [char] does, with the flags [autoSpace] and \
                  [phantomSpace].",
            args: &[
                arg("text", ArgKind::As("char: String", "char")),
                TYPING_ARGS[0],
                TYPING_ARGS[1],
                TYPING_ARGS[2],
                TYPING_ARGS[3],
                TYPING_ARGS[4],
                TYPING_ARGS[5],
            ],
            result: Out::Wrap("Spaces"),
            ..MEMBER
        },
        Member {
            name: "phantomSpaceBefore",
            doc: "Whether to type the owed phantom space ([phantomSpace]), or one [forceActive], \
                  before [text].",
            args: &TYPING_ARGS,
            ..MEMBER
        },
        Member {
            name: "supportsAutoSpace",
            doc: "Whether a word in [language] is followed by a space \
                  (`FlorisLocale.supportsAutoSpace`).",
            ..MEMBER
        },
        Member {
            name: "fallbackRule",
            doc: "`PunctuationRule.Fallback`'s id, label and five symbol strings, in its \
                  constructor's order.",
            kind: Kind::Val,
            result: Out::List,
            ..MEMBER
        },
        Member {
            name: "INACTIVE",
            doc: "Neither flag set.",
            kind: Kind::Val,
            call: "inactive",
            ..MEMBER
        },
        Member {
            name: "autoSpaceActive",
            doc: "An active auto-space.",
            ..MEMBER
        },
        Member {
            name: "phantomSpaceActive",
            doc: "An active phantom space.",
            ..MEMBER
        },
        Member {
            name: "isActive",
            doc: "Whether the space is owed.",
            ..MEMBER
        },
        Member {
            name: "showsComposingRegion",
            doc: "Whether the word before a phantom space shows as being composed.",
            ..MEMBER
        },
        Member {
            name: "afterUpdate",
            doc: "The state after the editor's selection changed: a mark to stay active is used \
                  up, else it clears.",
            ..MEMBER
        },
        Member {
            name: "keepsCandidate",
            doc: "Whether a phantom space in [state] before an update keeps the suggestion it \
                  came from.",
            ..MEMBER
        },
    ],
    ..API
};
