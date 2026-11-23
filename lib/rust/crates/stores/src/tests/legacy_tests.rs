//! Importing the old databases: once, whatever crash comes at whatever step.
//!
//! Each run reads the old rows as Kotlin does: a database whose main file is gone has no rows,
//! even if its journals are still there.

use std::path::{Path, PathBuf};

use crate::clips::{Clip, Clips, FileInfo, IMAGE, Limit, TEXT};
use crate::legacy::{self, ClipRows, Imported, WordRows};
use crate::store::{self, Opened, Store};
use crate::words::{Word, Words};

use super::fault::{CRASHES, FaultDisk};
use super::golden::{check, clip, units};

const STORE: &str = "files/clipboard.store";
const HISTORY: &str = "databases/clipboard_history";
const FILES: &str = "databases/clipboard_files";
const SIDECARS: [&str; 3] = ["-wal", "-shm", "-journal"];

/// Each database's files, main file first.
fn files_of(database: &str) -> Vec<String> {
    let mut files = vec![database.to_owned()];
    files.extend(SIDECARS.iter().map(|end| format!("{database}{end}")));
    files
}

fn legacy_paths() -> Vec<PathBuf> {
    [HISTORY, FILES]
        .iter()
        .flat_map(|db| files_of(db))
        .map(PathBuf::from)
        .collect()
}

fn old_rows() -> ClipRows {
    let image = Clip {
        pinned: true,
        uri: Some(units("content://synthetic/clips/images/77")),
        ..clip(4, IMAGE, 1_600_000_000_400, None)
    };
    ClipRows {
        items: vec![
            clip(2, TEXT, 1_600_000_000_200, Some("synthetic old")),
            image,
            Clip {
                sensitive: true,
                remote: true,
                ..clip(9, TEXT, 1_600_000_000_900, Some("s"))
            },
        ],
        files: vec![FileInfo {
            id: 77,
            name: units("Image"),
            size: 3,
            orientation: 0,
            mime_types: units("image/png"),
        }],
        sequence: 12,
    }
}

/// A disk with both old databases, every journal, and `store` if given.
fn old_disk(store: Option<&Clips>) -> FaultDisk {
    let bytes = store.map(store::encode);
    let mut files: Vec<(String, Vec<u8>)> = legacy_paths()
        .iter()
        .map(|p| (p.to_string_lossy().into_owned(), b"sqlite".to_vec()))
        .collect();
    if let Some(bytes) = bytes {
        files.push((STORE.to_owned(), bytes));
    }
    let refs: Vec<(&str, &[u8])> = files
        .iter()
        .map(|(p, b)| (p.as_str(), b.as_slice()))
        .collect();
    FaultDisk::with(&refs)
}

/// The rows Kotlin would read from what is left on `disk`.
fn rows_on(disk: &FaultDisk) -> ClipRows {
    let all = old_rows();
    let history = disk.exists(HISTORY);
    ClipRows {
        items: if history { all.items } else { Vec::new() },
        files: if disk.exists(FILES) {
            all.files
        } else {
            Vec::new()
        },
        sequence: if history { all.sequence } else { 0 },
    }
}

fn open(disk: &FaultDisk) -> Store<Clips> {
    Store::open(Path::new(STORE), disk.boxed())
}

fn run(disk: &FaultDisk) -> std::io::Result<Imported> {
    let mut store = open(disk);
    legacy::import(&mut store, rows_on(disk), &legacy_paths())
}

fn assert_imported_once(disk: &FaultDisk, expected: &Clips, what: &str) {
    let store = open(disk);
    assert_eq!(store.opened(), Opened::Loaded, "{what}");
    check(&store.value().items, &expected.items, what);
    check(&store.value().files, &expected.files, what);
    assert_eq!(store.value().next_id, expected.next_id, "{what}");
    for path in legacy_paths() {
        assert!(
            !disk.exists(&path.to_string_lossy()),
            "{what}: an old file is left"
        );
    }
}

fn crash_everywhere(existing: Option<&Clips>, expected: &Clips) {
    for crash in CRASHES {
        for at in 0..30 {
            let disk = old_disk(existing);
            let _store = open(&disk);
            disk.fail_at(at);
            let first = run(&disk);
            let next = disk.after(crash);
            let second = run(&next);
            assert!(second.is_ok(), "{crash:?} at {at}: the next run failed");
            let what = format!("{crash:?} at {at} ({first:?}, then {second:?})");
            assert_imported_once(&next, expected, &what);
        }
    }
}

#[test]
fn an_empty_store_takes_the_rows_with_their_ids() {
    let expected = Clips {
        next_id: 13,
        items: old_rows().items,
        files: old_rows().files,
        ..Clips::default()
    };
    crash_everywhere(None, &expected);
}

#[test]
fn a_store_with_clips_takes_the_rows_after_its_own() {
    let mut existing = Clips::default();
    existing.add(
        clip(0, TEXT, 1_700_000_000_000, Some("synthetic new")),
        Limit {
            now: 0,
            enabled: false,
            maximum: 0,
        },
    );
    let mut expected = existing.clone();
    for (n, item) in old_rows().items.into_iter().enumerate() {
        expected.items.push(Clip {
            id: 2 + n as i64,
            ..item
        });
    }
    expected.next_id = 5;
    expected.files = old_rows().files;
    crash_everywhere(Some(&existing), &expected);
}

#[test]
fn rows_imported_before_a_crash_are_not_imported_again() {
    let disk = old_disk(None);
    let mut store = open(&disk);
    disk.fail_at(4); // saved and read back; the first removal fails
    assert!(legacy::import(&mut store, rows_on(&disk), &legacy_paths()).is_err());
    let next = disk.after(super::fault::Crash::Death);
    assert_eq!(run(&next).ok(), Some(Imported::Already));
    assert_eq!(open(&next).value().items.len(), 3);
}

#[test]
fn no_rows_only_removes_the_files() {
    let disk = FaultDisk::with(&[
        (HISTORY, b"sqlite"),
        ("databases/clipboard_history-wal", b"wal"),
    ]);
    let mut store = open(&disk);
    assert_eq!(
        legacy::import(&mut store, ClipRows::default(), &legacy_paths()).ok(),
        Some(Imported::Nothing)
    );
    assert!(
        disk.names().is_empty(),
        "no store written, no old file left"
    );
}

#[test]
fn a_store_kept_as_found_takes_nothing_and_deletes_nothing() {
    let newer = crate::codec::frame(*b"CLIP", 2, &[]);
    let disk = FaultDisk::with(&[(STORE, &newer), (HISTORY, b"sqlite")]);
    let mut store = open(&disk);
    assert_eq!(
        legacy::import(&mut store, old_rows(), &legacy_paths()).ok(),
        Some(Imported::Kept)
    );
    assert!(disk.exists(HISTORY));
    check(&disk.file(STORE), &Some(newer), "kept store");
}

#[test]
fn a_dictionary_imports_once_and_renumbers_after_its_own_words() {
    let path = Path::new("files/user-dictionary.store");
    let word = |id, text: &str| Word {
        id,
        word: units(text),
        frequency: 1,
        locale: Some(units("en")),
        shortcut: None,
    };
    let rows = || WordRows {
        words: vec![word(3, "b"), word(1, "a")],
        sequence: 6,
    };
    let files = [PathBuf::from("databases/floris_user_dictionary")];
    let disk = FaultDisk::with(&[("databases/floris_user_dictionary", b"sqlite")]);
    let mut store: Store<Words> = Store::open(path, disk.boxed());
    assert_eq!(
        legacy::import(&mut store, rows(), &files).ok(),
        Some(Imported::Rows)
    );
    check(
        &store.value().words,
        &vec![word(1, "a"), word(3, "b")],
        "kept ids, by id",
    );
    assert_eq!(store.value().next_id, 7);
    assert_eq!(
        legacy::import(&mut store, rows(), &files).ok(),
        Some(Imported::Already)
    );
    let later = WordRows {
        words: vec![word(1, "c")],
        sequence: 1,
    };
    assert_eq!(
        legacy::import(&mut store, later, &files).ok(),
        Some(Imported::Rows)
    );
    check(
        &store.value().words.last().cloned(),
        &Some(word(7, "c")),
        "renumbered",
    );
}
