//! `ClipboardStoreNative`, the clipboard history's store, answered by awnative's
//! `clipboard_store` module, and `ClipboardStore` with the records it returns, the Kotlin over it.

use crate::{
    API, Api, CLASS, Class, Element, HANDLE, Handle, Kind, MEMBER, METHOD, Member, Method, Module,
    Out, Param, Return, Shape, Type,
};

const STORE: (&str, Param) = ("store", Param::Long);
const NOW: (&str, Param) = ("now", Param::Long);
const LIMIT_ENABLED: (&str, Param) = ("limitEnabled", Param::Boolean);
const LIMIT: (&str, Param) = ("limit", Param::Int);
const LONGS: Param = Param::Array(Element::Long);
const INTS: Param = Param::Array(Element::Int);

/// A method of `ClipboardStoreNative`.
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

pub const CLIPBOARD_STORE_NATIVE: Class = Class {
    name: "ClipboardStoreNative",
    doc: "The clipboard store's entry points in libawnative, answered by \
          lib/rust/crates/awnative/src/clipboard_store.rs. [ClipboardStore] wraps them.",
    module: "clipboard_store",
    methods: &[
        method("openClipboard", &[("path", Param::String)], Return::Long),
        method("freeClipboard", &[STORE], Return::Unit),
        method("clipboardOpened", &[STORE], Return::Int),
        method("clipboardItems", &[STORE], Return::StringArrays),
        method(
            "clipboardAdd",
            &[
                STORE,
                ("kind", Param::Int),
                ("created", Param::Long),
                ("flags", Param::Int),
                ("text", Param::NullableString),
                ("uri", Param::NullableString),
                ("mimeTypes", Param::String),
                NOW,
                LIMIT_ENABLED,
                LIMIT,
            ],
            Return::Long,
        ),
        method(
            "clipboardRestore",
            &[
                STORE,
                ("kinds", INTS),
                ("created", LONGS),
                ("flags", INTS),
                ("texts", Param::NullableStrings),
                ("uris", Param::NullableStrings),
                ("mimeTypes", Param::Strings),
                NOW,
                LIMIT_ENABLED,
                LIMIT,
            ],
            Return::Int,
        ),
        method(
            "clipboardPin",
            &[
                STORE,
                ("id", Param::Long),
                ("pinned", Param::Boolean),
                NOW,
                LIMIT_ENABLED,
                LIMIT,
            ],
            Return::Unit,
        ),
        method(
            "clipboardDelete",
            &[
                STORE,
                ("ids", LONGS),
                ("onlyIfUnpinned", Param::Boolean),
                NOW,
                LIMIT_ENABLED,
                LIMIT,
            ],
            Return::Strings,
        ),
        method(
            "clipboardClear",
            &[
                STORE,
                ("unpinnedOnly", Param::Boolean),
                NOW,
                LIMIT_ENABLED,
                LIMIT,
            ],
            Return::Strings,
        ),
        method(
            "clipboardExpire",
            &[
                STORE,
                NOW,
                ("old", Param::Boolean),
                ("minutes", Param::Int),
                ("sensitive", Param::Boolean),
                ("seconds", Param::Int),
                LIMIT_ENABLED,
                LIMIT,
            ],
            Return::Unit,
        ),
        method(
            "clipboardEnforce",
            &[STORE, NOW, LIMIT_ENABLED, LIMIT],
            Return::Unit,
        ),
        method(
            "clipboardFile",
            &[STORE, ("id", Param::Long)],
            Return::StringArrays,
        ),
        method(
            "clipboardAddFile",
            &[
                STORE,
                ("id", Param::Long),
                ("name", Param::String),
                ("size", Param::Long),
                ("orientation", Param::Int),
                ("mimeTypes", Param::String),
            ],
            Return::Unit,
        ),
        method(
            "clipboardRemoveFile",
            &[STORE, ("id", Param::Long)],
            Return::Unit,
        ),
        method(
            "clipboardImport",
            &[
                STORE,
                ("ids", LONGS),
                ("kinds", INTS),
                ("created", LONGS),
                ("flags", INTS),
                ("texts", Param::NullableStrings),
                ("uris", Param::NullableStrings),
                ("mimeTypes", Param::Strings),
                ("fileIds", LONGS),
                ("fileNames", Param::Strings),
                ("fileSizes", LONGS),
                ("orientations", INTS),
                ("fileMimeTypes", Param::Strings),
                ("sequence", Param::Long),
                ("paths", Param::Strings),
            ],
            Return::Int,
        ),
    ],
    ..CLASS
};

/// A member of `ClipboardStore`.
const fn member(name: &'static str, doc: &'static str, call: &'static str) -> Member {
    Member {
        name,
        doc,
        call,
        ..MEMBER
    }
}

/// A member that returns the media no clip refers to any more.
const fn orphans(name: &'static str, doc: &'static str, call: &'static str) -> Member {
    Member {
        result: Out::List,
        ..member(name, doc, call)
    }
}

pub const CLIPBOARD_STORE: Api = Api {
    name: "ClipboardStore",
    doc: "The clipboard history and its media files' records, kept by Rust (crates/stores) in \
          the file at [path], which every change replaces crash-safely. Each change applies the \
          history's size limit after it: [limitEnabled], [limit], and [now] in milliseconds. \
          Safe to use from any thread; a file it could not read is kept as found, and every \
          change then throws. Its native memory is freed once nothing can reach it.",
    module: "clipboard_store",
    native: "ClipboardStoreNative",
    shape: Shape::Handle(Handle {
        create: "openClipboard",
        free: "freeClipboard",
        ..HANDLE
    }),
    members: &[
        Member {
            kind: Kind::Val,
            result: Out::Entry("StoreOpened"),
            ..member("opened", "How the file was found.", "clipboardOpened")
        },
        Member {
            result: Out::Records("StoredClip"),
            ..member(
                "items",
                "The history as shown: newest first, equal times oldest row first.",
                "clipboardItems",
            )
        },
        member(
            "add",
            "Copies a clip into the history; a text it has already moves to the top instead, \
             pinned if either was. The new row's id.",
            "clipboardAdd",
        ),
        member(
            "restore",
            "Adds each clip the history doesn't have yet, in order, as restoring a backup does. \
             Row i is [kinds] i, [created] i, [flags] i, [texts] i, [uris] i and [mimeTypes] i. \
             How many it added.",
            "clipboardRestore",
        ),
        member("pin", "Pins or unpins the clip [id].", "clipboardPin"),
        orphans(
            "delete",
            "Removes the clips [ids], or only the unpinned ones; the media no clip refers to any \
             more, for the caller to delete.",
            "clipboardDelete",
        ),
        orphans(
            "clear",
            "Removes every clip, or every unpinned one; the media no clip refers to any more.",
            "clipboardClear",
        ),
        member(
            "expire",
            "Removes the unpinned clips older than [minutes] if [old], and the sensitive ones \
             older than [seconds] if [sensitive].",
            "clipboardExpire",
        ),
        member("enforce", "Applies the size limit.", "clipboardEnforce"),
        Member {
            result: Out::Records("StoredFile"),
            ..member(
                "fileRecords",
                "The record of the media file [id]: one, or none.",
                "clipboardFile",
            )
        },
        member(
            "addFile",
            "Keeps a media file's record, replacing any with its id.",
            "clipboardAddFile",
        ),
        member(
            "removeFile",
            "Forgets the media file [id]'s record.",
            "clipboardRemoveFile",
        ),
        Member {
            result: Out::Entry("StoreImport"),
            ..member(
                "import",
                "Imports the old Room databases' rows once, then deletes their [paths], each \
                 database's main file first: the history's rows by id ([ids] i, [kinds] i, \
                 [created] i, [flags] i, [texts] i, [uris] i, [mimeTypes] i), its \
                 `AUTOINCREMENT` [sequence], and the media files' records ([fileIds] i, \
                 [fileNames] i, [fileSizes] i, [orientations] i, [fileMimeTypes] i).",
                "clipboardImport",
            )
        },
    ],
    ..API
};

/// A top-level record of this file.
const fn record(
    name: &'static str,
    doc: &'static str,
    fields: &'static [(&'static str, &'static str)],
) -> (&'static str, Module, Type) {
    (
        "clipboard_store",
        Module::Rust,
        Type::Record {
            name,
            doc,
            data: true,
            fields,
            constants: &[],
        },
    )
}

pub const STORED_CLIP: (&str, Module, Type) = record(
    "StoredClip",
    "A clip of the history: its [flags] are [ClipFlag] codes, and its [text] and [uri] are \
     empty unless [ClipFlag.HAS_TEXT] and [ClipFlag.HAS_URI] say it has them. Its [mimeTypes] \
     are joined by commas.",
    &[
        ("id", "Long"),
        ("kind", "Int"),
        ("created", "Long"),
        ("flags", "Int"),
        ("text", "String"),
        ("uri", "String"),
        ("mimeTypes", "String"),
    ],
);

pub const STORED_FILE: (&str, Module, Type) = record(
    "StoredFile",
    "The record of a media file the clipboard keeps: its [mimeTypes] are joined by commas.",
    &[
        ("id", "Long"),
        ("name", "String"),
        ("size", "Long"),
        ("orientation", "Int"),
        ("mimeTypes", "String"),
    ],
);
