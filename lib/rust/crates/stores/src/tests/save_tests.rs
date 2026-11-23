//! Opening a store from every state its file can be in, and a crash at every step of a save.

use std::path::Path;

use crate::clips::{Clips, Limit, TEXT};
use crate::codec;
use crate::store::{self, Opened, Payload, Store};
use crate::words::{Word, Words};

use super::fault::{CRASHES, FaultDisk};
use super::golden::{check, clip, sample_clips, sample_words, units};

const PATH: &str = "files/clipboard.store";
const NO_LIMIT: Limit = Limit {
    now: 0,
    enabled: false,
    maximum: 0,
};

fn open(disk: &FaultDisk) -> Store<Clips> {
    Store::open(Path::new(PATH), disk.boxed())
}

#[test]
fn a_store_opens_from_every_state_of_its_file() {
    let missing = open(&FaultDisk::default());
    assert_eq!(missing.opened(), Opened::Missing);
    let saved = store::encode(&sample_clips());
    let loaded = open(&FaultDisk::with(&[(PATH, &saved)]));
    assert_eq!(loaded.opened(), Opened::Loaded);
    check(loaded.value(), &sample_clips(), "loaded clips");
    let damaged = open(&FaultDisk::with(&[(PATH, &saved[..saved.len() - 1])]));
    assert_eq!(damaged.opened(), Opened::Damaged);
    check(damaged.value(), &Clips::default(), "damaged clips");
    let newer = codec::frame(Clips::KIND, Clips::VERSION + 1, &[]);
    let newer = open(&FaultDisk::with(&[(PATH, &newer)]));
    assert_eq!(newer.opened(), Opened::Newer);
}

#[test]
fn a_file_it_could_not_read_is_never_overwritten() {
    for (file, opened) in [
        (
            codec::frame(Clips::KIND, 9, b"from the future"),
            Opened::Newer,
        ),
        (store::encode(&sample_clips()), Opened::Unreadable),
    ] {
        let disk = FaultDisk::with(&[(PATH, &file)]);
        if opened == Opened::Unreadable {
            disk.fail_at(1); // the stale temporary file's removal works; the read fails
        }
        let mut store = open(&disk);
        assert_eq!(store.opened(), opened);
        assert!(
            store
                .change(|clips| clips.add(clip(0, TEXT, 1, Some("x")), NO_LIMIT))
                .is_err()
        );
        check(&disk.file(PATH), &Some(file), "kept file");
    }
}

#[test]
fn a_damaged_file_is_replaced_by_the_first_change() {
    let disk = FaultDisk::with(&[(PATH, b"AWSTCLIP garbage")]);
    let mut store = open(&disk);
    store
        .change(|clips| clips.add(clip(0, TEXT, 1, Some("x")), NO_LIMIT))
        .expect("saved");
    let reopened = open(&disk);
    assert_eq!(reopened.opened(), Opened::Loaded);
    assert_eq!(reopened.value().items.len(), 1);
}

#[test]
fn a_change_that_changes_nothing_writes_nothing() {
    let disk = FaultDisk::with(&[(PATH, &store::encode(&sample_clips()))]);
    let mut store = open(&disk);
    let before = disk.steps();
    store
        .change(|clips| clips.set_pinned(404, true, NO_LIMIT))
        .expect("no change");
    assert_eq!(disk.steps(), before);
}

#[test]
fn a_crash_at_any_step_of_a_save_leaves_the_old_value_or_the_new() {
    let old = sample_clips();
    let mut new = old.clone();
    new.add(clip(0, TEXT, 9, Some("synthetic new")), NO_LIMIT);
    let steps = 3; // write the temporary file, rename it, sync the directory
    for crash in CRASHES {
        for at in 0..=steps {
            let disk = FaultDisk::with(&[
                (PATH, &store::encode(&old)),
                ("files/clipboard.store.tmp", b"stale"),
            ]);
            let mut store = open(&disk);
            disk.fail_at(at);
            let saved =
                store.change(|clips| clips.add(clip(0, TEXT, 9, Some("synthetic new")), NO_LIMIT));
            check(
                store.value(),
                if saved.is_ok() { &new } else { &old },
                "value in memory",
            );
            let next = disk.after(crash);
            let reopened = open(&next);
            assert_eq!(reopened.opened(), Opened::Loaded, "{crash:?} at {at}");
            let value = reopened.value();
            assert!(value == &old || value == &new, "{crash:?} at {at}: neither");
            if saved.is_ok() && matches!(crash, super::fault::Crash::Death) {
                check(value, &new, "a saved change");
            }
            assert!(
                !next.exists("files/clipboard.store.tmp"),
                "{crash:?} at {at}: stale file"
            );
        }
    }
}

#[test]
fn a_transaction_saves_once_or_not_at_all() {
    let path = Path::new("files/user-dictionary.store");
    let disk = FaultDisk::with(&[]);
    let mut store: Store<Words> = Store::open(path, disk.boxed());
    let word = |text: &str| Word {
        word: units(text),
        ..Word::default()
    };
    store.begin();
    let before = disk.steps();
    for text in ["one", "two", "three"] {
        store
            .change(|words| words.insert(word(text)))
            .expect("kept")
            .expect("an id");
    }
    assert_eq!(disk.steps(), before, "nothing written inside a transaction");
    store.commit().expect("saved");
    assert_eq!(disk.steps(), before + 3, "one save");
    store.begin();
    store.change(|words| words.delete(1)).expect("kept");
    store.rollback();
    assert_eq!(store.value().words.len(), 3);
    let reopened: Store<Words> = Store::open(path, disk.boxed());
    check(reopened.value(), store.value(), "committed words");
}

#[test]
fn a_transaction_that_cannot_be_saved_is_dropped() {
    let path = Path::new("files/user-dictionary.store");
    let disk = FaultDisk::with(&[]);
    let mut store: Store<Words> = Store::open(path, disk.boxed());
    store
        .change(|words| *words = sample_words())
        .expect("saved");
    store.begin();
    store.change(|words| words.delete(1)).expect("kept");
    disk.fail_at(0);
    assert!(store.commit().is_err());
    check(
        store.value(),
        &sample_words(),
        "words after a failed commit",
    );
}
