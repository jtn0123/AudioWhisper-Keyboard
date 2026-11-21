//! `WordStoresNative`, the words the keyboard keeps for the user, answered by awnative's
//! `suggest::word_stores` module, and `WordStoreRules`, `LearnedWordStore`, `KeptWordStore` and
//! `PersonalWordStore`, the Kotlin over it.

use crate::{
    API, Api, ArgKind, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method,
    Out, Param, Return, Shape, arg, default,
};

const STORE: (&str, Param) = ("store", Param::Long);
const WORDS: (&str, Param) = ("words", Param::Long);
const WORD: (&str, Param) = ("word", Param::String);
const TEXT: (&str, Param) = ("text", Param::String);
const TYPED: [(&str, Param); 2] = [("typed", Param::String), ("maxCount", Param::Int)];
const PREVIOUS: [(&str, Param); 2] = [
    ("firstWord", Param::NullableString),
    ("isBeginningOfSentence", Param::Boolean),
];

/// A method of `WordStoresNative`.
const fn method(
    name: &'static str,
    params: &'static [(&'static str, Param)],
    returns: Return,
) -> Method {
    Method {
        name,
        params,
        returns,
        ..METHOD
    }
}

/// A member of a word store's class or object.
const fn member(name: &'static str, doc: &'static str, call: &'static str) -> Member {
    Member {
        name,
        doc,
        call,
        ..MEMBER
    }
}

/// A member of a word store that returns a list of words.
const fn words(name: &'static str, doc: &'static str, call: &'static str) -> Member {
    Member {
        result: Out::List,
        ..member(name, doc, call)
    }
}

/// A value of `WordStoreRules`.
const fn value(name: &'static str, doc: &'static str) -> Member {
    Member {
        kind: Kind::Val,
        ..member(name, doc, "")
    }
}

pub const WORD_STORES_NATIVE: Class = Class {
    name: "WordStoresNative",
    doc: "The word stores' entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/suggest/word_stores.rs. [WordStoreRules], \
          [LearnedWordStore], [KeptWordStore] and [PersonalWordStore] wrap them.",
    module: "suggest::word_stores",
    file: "word_stores",
    methods: &[
        method("sentenceStart", &[], Return::String),
        method("knownAfter", &[], Return::Int),
        method("learnedWordCapacity", &[], Return::Int),
        method("learnedPairCapacity", &[], Return::Int),
        method("maxWordLength", &[], Return::Int),
        method("keptCapacity", &[], Return::Int),
        Method {
            rust: "::suggest::learned_words::is_storable",
            ..method("isStorable", &[WORD], Return::Boolean)
        },
        method("previousKey", &PREVIOUS, Return::NullableString),
        Method {
            rust: "::suggest::personal_words::applies_to",
            ..method(
                "appliesTo",
                &[("tag", Param::NullableString), ("language", Param::String)],
                Return::Boolean,
            )
        },
        method(
            "newLearnedWords",
            &[("wordCapacity", Param::Int), ("pairCapacity", Param::Int)],
            Return::Long,
        ),
        method("freeLearnedWords", &[STORE], Return::Unit),
        method(
            "learnedAdd",
            &[
                STORE,
                WORD,
                ("previous", Param::NullableString),
                ("keepSpelling", Param::Boolean),
            ],
            Return::Unit,
        ),
        method(
            "learnedRemove",
            &[STORE, WORD, ("previous", Param::NullableString)],
            Return::Unit,
        ),
        method("learnedClear", &[STORE], Return::Unit),
        method("learnedKnownSpellings", &[STORE], Return::Strings),
        method("learnedSpellingOf", &[STORE, WORD], Return::NullableString),
        method(
            "learnedSwipeBoosts",
            &[STORE, ("words", Param::Strings)],
            Return::Array(Element::Int),
        ),
        method(
            "learnedCompletions",
            &[STORE, TYPED[0], TYPED[1]],
            Return::Strings,
        ),
        method(
            "learnedFollowers",
            &[
                STORE,
                PREVIOUS[0],
                PREVIOUS[1],
                ("minCount", Param::Int),
                ("maxCount", Param::Int),
            ],
            Return::Strings,
        ),
        method(
            "learnedPairCount",
            &[STORE, PREVIOUS[0], PREVIOUS[1], WORD],
            Return::Int,
        ),
        method("learnedChanges", &[STORE], Return::Long),
        method("learnedMerge", &[STORE, TEXT], Return::Unit),
        method("learnedSavedText", &[STORE], Return::String),
        method("newKeptWords", &[("capacity", Param::Int)], Return::Long),
        method("freeKeptWords", &[STORE], Return::Unit),
        method("keptContains", &[STORE, WORD], Return::Boolean),
        method(
            "keptAdd",
            &[STORE, WORD, ("remember", Param::Boolean)],
            Return::Boolean,
        ),
        method("keptClear", &[STORE], Return::Unit),
        method("keptMerge", &[STORE, TEXT], Return::Unit),
        method("keptSavedText", &[STORE], Return::String),
        method(
            "newPersonalWords",
            &[
                ("words", Param::Strings),
                ("frequencies", Param::Array(Element::Int)),
                ("shortcuts", Param::NullableStrings),
            ],
            Return::Long,
        ),
        method("freePersonalWords", &[WORDS], Return::Unit),
        method("personalOrder", &[WORDS], Return::Array(Element::Int)),
        method("personalSpellingOf", &[WORDS, WORD], Return::NullableString),
        method(
            "personalCompletions",
            &[WORDS, TYPED[0], TYPED[1]],
            Return::Strings,
        ),
    ],
    ..CLASS
};

pub const WORD_STORE_RULES: Api = Api {
    name: "WordStoreRules",
    doc: "The rules and numbers of the words the keyboard keeps for the user, in Rust \
          (lib/rust/crates/suggest): what it learns from typing, the words kept as typed, and \
          the personal dictionaries. The keyboard's `suggest` package wraps these and documents \
          them.",
    module: "word_stores",
    native: "WordStoresNative",
    members: &[
        value(
            "sentenceStart",
            "The previous \"word\" of a sentence's first word.",
        ),
        value(
            "knownAfter",
            "A word typed this many times is one of the user's words.",
        ),
        value(
            "learnedWordCapacity",
            "How many words a [LearnedWordStore] keeps unless told otherwise.",
        ),
        value(
            "learnedPairCapacity",
            "How many word pairs a [LearnedWordStore] keeps unless told otherwise.",
        ),
        value(
            "maxWordLength",
            "The longest word a [LearnedWordStore] keeps, in UTF-16 units.",
        ),
        value(
            "keptCapacity",
            "How many words a [KeptWordStore] remembers unless told otherwise.",
        ),
        member(
            "isStorable",
            "Whether [word] can be learned: letters, with apostrophes or hyphens inside.",
            "",
        ),
        member(
            "previousKey",
            "The word before the next one: [firstWord], the most recent word before it, else \
             [sentenceStart] if [isBeginningOfSentence], else null, as what came before is \
             unknown.",
            "",
        ),
        member(
            "appliesTo",
            "Whether a personal dictionary row tagged [tag] (null for every language) applies \
             to [language].",
            "",
        ),
    ],
    ..API
};

pub const LEARNED_WORD_STORE: Api = Api {
    name: "LearnedWordStore",
    doc: "The words and word pairs the user has typed, with how often, in Rust. Safe to use from \
          any thread. Its native memory is freed once nothing can reach it. The previous word of \
          a pair is passed as the context's most recent word and whether a sentence starts there \
          (see [WordStoreRules.previousKey]).",
    module: "word_stores",
    native: "WordStoresNative",
    shape: Shape::Handle(Handle {
        params: &[
            default("wordCapacity", "WordStoreRules.learnedWordCapacity"),
            default("pairCapacity", "WordStoreRules.learnedPairCapacity"),
        ],
        create: "newLearnedWords",
        free: "freeLearnedWords",
        ..HANDLE
    }),
    members: &[
        member(
            "add",
            "One use of [word] after [previous]: a word, [WordStoreRules.sentenceStart], or null \
             when unknown.",
            "learnedAdd",
        ),
        member(
            "remove",
            "Takes back one [add] of [word] after [previous].",
            "learnedRemove",
        ),
        member("clear", "", "learnedClear"),
        words(
            "knownSpellings",
            "How the user spells each of their words, least recently used first.",
            "learnedKnownSpellings",
        ),
        Member {
            args: &[arg("words", ArgKind::List)],
            ..member(
                "swipeBoosts",
                "How many steps swipe typing raises each of [words] for the user's use of it \
                 ([SwipeRules.tunedFrequency]): more each time its uses double, up to a limit, \
                 and none for a word that isn't theirs as spelt. Cleared with the words.",
                "learnedSwipeBoosts",
            )
        },
        member(
            "spellingOf",
            "How the user spells [word], once it is one of theirs, or null.",
            "learnedSpellingOf",
        ),
        words(
            "completions",
            "Every completion of [typed], best first: none if [typed] is empty or [maxCount] \
             isn't positive. The caller keeps the first [maxCount] it wants.",
            "learnedCompletions",
        ),
        words(
            "followers",
            "Up to [maxCount] words typed after the previous word at least [minCount] times, \
             best first.",
            "learnedFollowers",
        ),
        member(
            "pairCount",
            "How many times [word] was typed after the previous word.",
            "learnedPairCount",
        ),
        member(
            "changes",
            "How many times the words have changed (learned, taken back, cleared or loaded). \
             Equal counts mean nothing changed in between; only equality means anything.",
            "learnedChanges",
        ),
        member(
            "merge",
            "Takes in a saved file's [text], keeping what was learned before it was read.",
            "learnedMerge",
        ),
        member(
            "savedText",
            "The text to save, for [merge] to read back.",
            "learnedSavedText",
        ),
    ],
    ..API
};

pub const KEPT_WORD_STORE: Api = Api {
    name: "KeptWordStore",
    doc: "The words whose auto-correction the user undid, in Rust. Safe to use from any thread. \
          Its native memory is freed once nothing can reach it.",
    module: "word_stores",
    native: "WordStoresNative",
    shape: Shape::Handle(Handle {
        params: &[default("capacity", "WordStoreRules.keptCapacity")],
        create: "newKeptWords",
        free: "freeKeptWords",
        ..HANDLE
    }),
    members: &[
        Member {
            operator: true,
            ..member(
                "contains",
                "Whether [word] is kept, in any case.",
                "keptContains",
            )
        },
        member(
            "add",
            "Keeps [word], saving it if [remember]; true when [savedText] changed.",
            "keptAdd",
        ),
        member(
            "clear",
            "Forgets every kept word, remembered or not.",
            "keptClear",
        ),
        member(
            "merge",
            "Takes in a saved file's [text], ahead of the words remembered before it was read.",
            "keptMerge",
        ),
        member(
            "savedText",
            "The text to save, for [merge] to read back.",
            "keptSavedText",
        ),
    ],
    ..API
};

pub const PERSONAL_WORD_STORE: Api = Api {
    name: "PersonalWordStore",
    doc: "A snapshot of the personal dictionaries' rows, in Rust: row i is [words] i, \
          [frequencies] i and [shortcuts] i. Never changes, so safe to use from any thread. Its \
          native memory is freed once nothing can reach it.",
    module: "word_stores",
    native: "WordStoresNative",
    shape: Shape::Handle(Handle {
        params: &[arg("words", ArgKind::List), arg("shortcuts", ArgKind::List)],
        create: "newPersonalWords",
        free: "freePersonalWords",
        ..HANDLE
    }),
    members: &[
        Member {
            kind: Kind::Val,
            ..member(
                "order",
                "The rows kept, most frequent first, by their index in the lists this was made \
                 from.",
                "personalOrder",
            )
        },
        member(
            "spellingOf",
            "How the user spells [word]: [word] if they wrote it so, else their spelling \
             ignoring case, else null.",
            "personalSpellingOf",
        ),
        words(
            "completions",
            "At most [maxCount] words for [typed], best first.",
            "personalCompletions",
        ),
    ],
    ..API
};
