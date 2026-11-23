//! `UserWordsNative`, the keyboard's own user dictionary, answered by awnative's `user_words`
//! module, and `UserWordStore` with the record it returns, the Kotlin over it.

use crate::{
    API, Api, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method, Module,
    Out, Param, Return, Shape, Type,
};

const STORE: (&str, Param) = ("store", Param::Long);
const ROW: [(&str, Param); 5] = [
    ("id", Param::Long),
    ("word", Param::String),
    ("frequency", Param::Int),
    ("locale", Param::NullableString),
    ("shortcut", Param::NullableString),
];

/// A method of `UserWordsNative`.
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

pub const USER_WORDS_NATIVE: Class = Class {
    name: "UserWordsNative",
    doc: "The user dictionary store's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/user_words.rs. [UserWordStore] wraps them.",
    module: "user_words",
    methods: &[
        method("openUserWords", &[("path", Param::String)], Return::Long),
        method("freeUserWords", &[STORE], Return::Unit),
        method("userWordsOpened", &[STORE], Return::Int),
        method("userWordsAll", &[STORE], Return::StringArrays),
        method(
            "userWordsExact",
            &[
                STORE,
                ("word", Param::String),
                ("locale", Param::NullableString),
            ],
            Return::StringArrays,
        ),
        method(
            "userWordsInsert",
            &[STORE, ROW[0], ROW[1], ROW[2], ROW[3], ROW[4]],
            Return::Long,
        ),
        method(
            "userWordsUpdate",
            &[STORE, ROW[0], ROW[1], ROW[2], ROW[3], ROW[4]],
            Return::Unit,
        ),
        method("userWordsDelete", &[STORE, ROW[0]], Return::Unit),
        method("userWordsBegin", &[STORE], Return::Unit),
        method("userWordsCommit", &[STORE], Return::Unit),
        method("userWordsRollback", &[STORE], Return::Unit),
        method(
            "userWordsImport",
            &[
                STORE,
                ("ids", Param::Array(Element::Long)),
                ("words", Param::Strings),
                ("frequencies", Param::Array(Element::Int)),
                ("locales", Param::NullableStrings),
                ("shortcuts", Param::NullableStrings),
                ("sequence", Param::Long),
                ("paths", Param::Strings),
            ],
            Return::Int,
        ),
    ],
    ..CLASS
};

/// A member of `UserWordStore`.
const fn member(name: &'static str, doc: &'static str, call: &'static str) -> Member {
    Member {
        name,
        doc,
        call,
        ..MEMBER
    }
}

/// A member that returns rows of words.
const fn rows(name: &'static str, doc: &'static str, call: &'static str) -> Member {
    Member {
        result: Out::Records("StoredWord"),
        ..member(name, doc, call)
    }
}

pub const USER_WORD_STORE: Api = Api {
    name: "UserWordStore",
    doc: "The keyboard's own user dictionary, kept by Rust (crates/stores) in the file at \
          [path], which every change replaces crash-safely, as the Room table it replaces \
          behaved: rows by id, an id of 0 taking the next one. Safe to use from any thread; a \
          file it could not read is kept as found, and every change then throws. Its native \
          memory is freed once nothing can reach it.",
    module: "user_words",
    native: "UserWordsNative",
    shape: Shape::Handle(Handle {
        create: "openUserWords",
        free: "freeUserWords",
        ..HANDLE
    }),
    members: &[
        Member {
            kind: Kind::Val,
            result: Out::Entry("StoreOpened"),
            ..member("opened", "How the file was found.", "userWordsOpened")
        },
        rows("all", "Every word, by id.", "userWordsAll"),
        rows(
            "exact",
            "The words that are [word] exactly, in [locale] exactly (null matching null), by id.",
            "userWordsExact",
        ),
        member(
            "insert",
            "Adds a word with its [id], or the next one if [id] is 0; the id it got. Throws if \
             a word has that id.",
            "userWordsInsert",
        ),
        member(
            "update",
            "Replaces the word with this [id], if there is one.",
            "userWordsUpdate",
        ),
        member(
            "delete",
            "Removes the word [id], if there is one.",
            "userWordsDelete",
        ),
        member(
            "begin",
            "Starts a transaction: changes are kept in memory until [commit] saves them once, or \
             [rollback] drops them.",
            "userWordsBegin",
        ),
        member(
            "commit",
            "Saves the transaction's changes; if they cannot be saved, they are dropped and it \
             throws.",
            "userWordsCommit",
        ),
        member(
            "rollback",
            "Drops the transaction's changes.",
            "userWordsRollback",
        ),
        Member {
            result: Out::Entry("StoreImport"),
            ..member(
                "import",
                "Imports the old Room database's rows once ([ids] i, [words] i, [frequencies] i, \
                 [locales] i, [shortcuts] i, by id, and its `AUTOINCREMENT` [sequence]), then \
                 deletes its [paths], main file first.",
                "userWordsImport",
            )
        },
    ],
    ..API
};

pub const STORED_WORD: (&str, Module, Type) = (
    "user_words",
    Module::Rust,
    Type::Record {
        name: "StoredWord",
        doc: "A word of the user dictionary: its [flags] are [WordFlag] codes, and its [locale] \
              and [shortcut] are empty unless [WordFlag.HAS_LOCALE] and [WordFlag.HAS_SHORTCUT] \
              say it has them.",
        data: true,
        fields: &[
            ("id", "Long"),
            ("word", "String"),
            ("frequency", "Int"),
            ("flags", "Int"),
            ("locale", "String"),
            ("shortcut", "String"),
        ],
        constants: &[],
    },
);
