# Phase 5: the keyboard's stored data moves from Room to Rust

2026-11-22. This is [all-rust.md](all-rust.md) phase 5's first item, "the clipboard store moves
from Room to Rust", together with the keyboard's own user dictionary, and phase 4's "Room and KSP
leave the build" for Room's part of it. Branch `all-rust/stores`, from main 48b2e0018, rebased on
61f1513fb.

Rust now owns three sets of records that Room kept: the clipboard history, the records of its
media files, and the keyboard's own user dictionary (`floris_user_dictionary`). Rust owns their
format, reads and writes their files, and makes every change to them. Kotlin keeps only:

* the system `ClipboardManager` listener (`ClipboardManager.kt`);
* `ClipboardFileStorage`'s byte copies of media;
* `ClipboardMediaProvider`, the content provider that serves those copies, which asks Rust for each
  record;
* a one-time reader of main's Room databases (`LegacyRoom.kt`), on Android's own SQLite.

Android's system `UserDictionary` provider is unchanged.

## The store format

`crates/stores` keeps each store in one file. Every store shares one frame, little-endian
throughout (`codec.rs`):

| Bytes | What |
|---|---|
| 4 | `AWST`, the magic |
| 4 | the store's kind: `CLIP` or `WORD` |
| 4 | the payload's version, a `u32`; 1 for both stores |
| 8 | the payload's length in bytes, a `u64` |
| n | the payload |
| 4 | CRC-32 (IEEE, as zlib and `java.util.zip.CRC32` compute it) of every byte before it |

Text is a `u32` count of UTF-16 units followed by the units, so a lone surrogate survives, as it did
in Room. An absent text is a `0` byte; a present one is a `1` byte and then the text.

The clipboard payload (`clip_codec.rs`) holds, in order:

1. the next id;
2. the fingerprints of the databases already imported;
3. the clips by id, each with id, type, time, a flags byte (pinned, sensitive, from another
   device), optional text, optional URI, and MIME types;
4. the media files' records, each with id, size, orientation, display name and MIME types.

The dictionary payload (`word_codec.rs`) holds the next id, the imported fingerprints, and the
words by id. Each word has its id, frequency, the word, and an optional locale and shortcut.

A file is read only if it passes every check:

* The magic and kind match. Otherwise the file is not a store of this kind and opens as *Damaged*.
* The checksum matches, the length matches, nothing is left over, every count fits in the bytes
  left, ids are in order and below the next id, and flags and optional tags are known. Otherwise it
  opens as *Damaged* (`codec_tests.rs`: `every_cut_is_refused`,
  `payloads_that_break_their_rules_are_refused`, `clip_ids_must_rise_and_stay_below_the_next_id`,
  `unknown_flags_and_tags_are_refused`).
* The version is one this build reads. A higher version opens as *Newer*.

What each state means:

* **Damaged**: the store starts empty, and its first change replaces the file, as Room would have
  reset a corrupt database.
* **Newer** or **Unreadable** (an I/O error on read): the store reads as empty and refuses every
  change, so a downgrade, or a file this build can't read, loses nothing.

Golden files pin the format: `crates/stores/tests/fixtures/clips-v1.store`, `clips-v1-empty.store`,
`words-v1.store` and `words-v1-empty.store` must decode to the samples in `golden.rs`, and the
samples must encode to them byte for byte.

The files are `no_backup/clipboard.store` and `files/user-dictionary.store`. The first is outside
Auto Backup, as the clipboard's Room databases were. The second is inside it, as
`floris_user_dictionary` was (`data_extraction_rules.xml`, `backup_rules.xml`).

## Saves and the crash points

A change is made to a copy of the value. If the copy differs, it is saved, and only then replaces
the value (`store.rs`). So a change that can't be saved changes nothing. `disk::save`:

1. writes `<path>.tmp` and syncs it;
2. renames it over `<path>`;
3. syncs the directory, so the rename is on disk before the change counts as made.

`disk::load` first removes a `.tmp` a crash left: it was never renamed, so `<path>` is still the
last whole file. Transactions (the dictionary's `runInTransaction`) keep changes in memory and save
once at commit. If the commit can't be saved, they are dropped; if the block throws, they are rolled
back.

Every step goes through the `Disk` trait. The tests swap in `FaultDisk`, which can stop the process
at any step. It keeps two views of the files: what the process saw, and what survives a power cut,
where only synced renames and removals count. The next "run" starts from either view.

| Crash point | What survives | Test |
|---|---|---|
| Any step of a save, killed or power cut | the old value or the new, never a mix | `a_crash_at_any_step_of_a_save_leaves_the_old_value_or_the_new` |
| A save that fails | the value as it was | `a_transaction_that_cannot_be_saved_is_dropped`, `a_change_that_changes_nothing_writes_nothing` |
| A transaction | saved once at commit, or not at all | `a_transaction_saves_once_or_not_at_all` |
| A stale `.tmp` | removed at the next open | `a_store_opens_from_every_state_of_its_file`; on Android `aCrashMidSaveImportsOnTheNextStart` |
| A file this build can't read | never overwritten | `a_file_it_could_not_read_is_never_overwritten`; on Android `aStoreFromANewerBuildKeepsTheDatabases` |

## Migration from main

On first open, main's Room databases are imported into each store once, and then deleted. Kotlin
reads them, because it has Android's SQLite and Rust takes no SQLite dependency. Rust decides
everything else. Each database is opened read-only (read-write only if that fails, which recovers
what an interrupted write left, as Room's own open would) and read by column name. So every schema
main shipped reads:

* the history at versions 2, 3 and 4;
* the media records at versions 1 and 2;
* the dictionary at version 1.

`legacy::import` then:

1. returns *Kept*, and touches nothing, if the store is Newer or Unreadable. Kotlin checks this
   before opening a database at all, because opening a WAL database creates files beside it,
   even read-only.
2. fingerprints the rows: FNV-1a over their encoding, the `AUTOINCREMENT` sequence included.
3. if the store holds that fingerprint already, the rows went in on a run that stopped before the
   files were gone, so it only deletes the files (*Already*).
4. otherwise adds the rows and the fingerprint in one saved change. It reads the file back and
   checks that it holds exactly that value (*Rows*). An empty store keeps the rows' ids, with the
   next id after both the largest id and the sequence, as `AUTOINCREMENT` gives. A store with rows
   of its own puts the imported rows after them.
5. only then deletes the files in this order, syncing the directory after each: history, files, the
   history's `-wal`, `-shm` and `-journal`, then the files database's. A crash between steps
   leaves journals with no database, which the next run deletes as holding no rows.

| Crash point | Next start | Test (cargo, then on Android's SQLite) |
|---|---|---|
| Before the store's rename | the old store, every database: imports again | `an_empty_store_takes_the_rows_with_their_ids`, `a_store_with_clips_takes_the_rows_after_its_own` (each crashes at every step, killed and power cut) |
| A power cut after the import returned | nothing comes back: each removal was synced | the same two tests, which cut the power after the next run too |
| After the rename, before any database went | the fingerprint matches: deletes only | `rows_imported_before_a_crash_are_not_imported_again`; `aCrashBeforeAnyDatabaseWentImportsNothingTwice` |
| Between the two clipboard databases | the media records and the history's WAL: imports the records, no duplicate | `aCrashBetweenTheDatabasesLeavesNoDuplicate` |
| Only WAL files left | nothing to import; they are deleted | `aCrashWithOnlyTheWalLeftImportsNothing` |
| Mid-save of the store | the `.tmp` is removed, the import runs again | `aCrashMidSaveImportsOnTheNextStart` |
| A damaged database | imported as empty and deleted, as Room would have reset it | `aDamagedDatabaseImportsAsEmptyAndIsDeleted` |
| A store from a newer build | the databases are kept as they are | `a_store_kept_as_found_takes_nothing_and_deletes_nothing`; `aStoreFromANewerBuildKeepsTheDatabases` |
| No rows | only the files are deleted | `no_rows_only_removes_the_files`; `anEmptyHistoryImportsNothingAndIsDeleted` |

A database that came back without its WAL would read as older rows, with a different fingerprint,
and be imported a second time. That is why each removal is synced before the next.

Where this differs from main's own Room:

* **Media records at version 1.** Main's Room dropped them on upgrade (destructive migration).
  The import keeps them with orientation 0, the value a record without one already reads as.
* **The sequence.** Main's 3-to-4 auto-migration rebuilt the table and reset `sqlite_sequence` to
  the largest id (13 to 12 in the fixture). The import keeps the file's own counter, which is never
  lower. So an id is never handed out twice either way.
* **A database missing a column the reader needs** (history version 1, which main never exported a
  schema for and Room would reset) is kept, not deleted. It is logged as "lacks a column: kept" and
  read again at the next start. This loses no data the reader can't see.
* **Going back to main's build.** Main's build doesn't read the stores. It starts with empty Room
  databases, and the store files wait beside them. Installing this build again imports whatever
  main's build stored, after the store's own rows. So nothing is lost either way, but main's build
  shows none of the words or clips kept since. An Auto Backup made by this build and restored onto
  main's build is the same case: the words come back with the next upgrade, not before.

Only counts and outcomes are logged (`AwStores`), never a row.

## Room and KSP leave the build

* Room's runtime and compiler are gone from `app/build.gradle.kts` and `gradle/libs.versions.toml`.
* So are the `ksp { room.* }` options and the six schema files in `app/schemas`.
* `ClipboardDatabase.kt` is now `ClipboardItem.kt`: the clip model and its backup JSON, with no
  DAOs or databases.
* `UserDictionary.kt` lost its Room DAO and database; `FlorisUserDictionaryDatabase.kt` is a
  Room-shaped DAO over the Rust store, so its callers are unchanged.

**KSP stays**: Jetpref's `@Preferences` processor still generates `FlorisPreferenceModelImpl` from
`AppPrefs`. It goes when Rust owns the preferences file and Jetpref leaves (phase 4).

## Proof

* **Cargo**, 41 tests in `crates/stores` with no dependency outside the standard library: the
  golden files both ways, every way a file can be damaged, every crash point above (each step,
  killed and power cut), the history's and the dictionary's rules, and the import.
* **Match tests against main's Room** (JVM, `app/src/test/.../stores`). Main's own build wrote
  the expected results: 40 seeded sequences of 30 operations on its clipboard DAOs, driven as its
  `ClipboardManager` drove them, and 24 sequences of 25 on its dictionary
  (`app/src/test/resources/stores/*-match.json`). The branch replays all 1,200 and 600 steps
  through the production bridge. After each step its history, or its words, must hash the same as
  main's, and so must a store reopened from its file every seventh step, as after the process
  died.
* **Device tests** (instrumented, on Android's own SQLite), 11:
  * `StoresMigrationDeviceTest` (9) imports every database fixture and checks the rows against
    what main's DAOs read from the same file, migrations and all, then the crash points in the
    table above.
  * `StoresBackupDeviceTest` (2) reads each of main's clipboard backups into the same clips and
    writes them back byte for byte, then restores them into the store.

### Fixtures

All synthetic, made fresh by main's own build (48b2e0018, the last with Room) with the harness in
`scripts/stores-fixtures/` (its README says how to make them again):

* `history-v2.db`, `history-v3.db`, `files-v1.db`: raw SQLite from main's exported schemas, as Room
  made them.
* `history-v4.db`, `history-v4-empty.db`, `files-v2.db`, `words-v1.db`: main's Room databases.
* `history-v4-wal.db` with its `-wal` and `-shm`: copied while open, its rows still in the WAL.
* `*.expected.json`: what main's DAOs read from each, as row hashes.
* `backup-clips-{TEXT,IMAGE,VIDEO}.json`: main's `BackupIO` clipboard backups, with what main read
  back.
* `clipboard-match.json`, `dictionary-match.json`: the match sequences.
* `crates/stores/tests/fixtures/*.store`: the four golden store files.

That covers every schema main shipped: history 2, 3 and 4, media records 1 and 2, dictionary 1.

### Mutations

`scripts/store-mutations.py` makes 67 faults in the stores' Rust, the bridge and the Kotlin
around it. Each names the targets that must catch it: `cargo test -p stores` (54 cases), the JVM
match tests (22) and the device tests (20). A fault counts as caught only by a fresh assertion,
never by a compile failure.

* **Cargo and JVM**, on the rebased branch: all 58 faults with one of those targets caught (36
  by cargo alone, 16 by both, 6 by the match tests alone), and the restored source passes both.
* **Device**, on `emulator-5556`, each fault built into the app and installed: the device tests
  caught 18 of their 20, and the restored source passes. The other two, a stale `.tmp` left by
  the open and a change let through to a newer store's file, were caught by cargo. The device
  tests now check both too (96eaeec3e). A second device pass caught both, and the restored
  source passed again.

The first runs found two faults no test caught, both now caught (c81044718): clips stored out of
id order, and database removals a power cut could undo. Before that, six faults looked like
survivors only because the scratch disk is HFS+, which keeps whole seconds: a mutant written in
the same second as the last build left Cargo's binary in place. The script now stamps each
write a second later.

## Measurements

On this Mac (18 cores), against main 48b2e0018 in a scratch worktree, while other jobs ran: the
load average was 24.6 to 95.4 during the builds.

| | main | branch |
|---|---|---|
| `:app:assembleDebug` after `:app:clean`, `--no-build-cache` (3 runs) | 36.1, 65.4, 34.9 s; median 36.1 | 29.0, 32.6, 32.9 s; median 32.6 |
| The same after a one-line Kotlin change (3 runs) | 38.4, 36.9, 33.6 s; median 36.9 | 41.5, 31.6, 27.7 s; median 31.6 |
| `:app:assemblePersonal` (1 run) | 80.3 s | 61.0 s |

The branch's medians are 3.5 s (clean) and 5.3 s (incremental) faster, smaller than the spread of
main's own runs. KSP still runs, for Jetpref, but no longer processes Room's DAOs and databases.

APK sizes are compared as the sum of the zip's compressed entries, not file sizes: AGP packages
debug APKs incrementally and leaves gaps in the zip, so two debug files differ by whatever the last
builds left behind.

| | debug | personal (the shipped build) |
|---|---|---|
| Compressed entries | −176,290 bytes | +19,579 bytes |
| Dex | −270,148 | −73,212 |
| `libawnative.so` | +97,552 | +97,552 |
| File size | (gaps) | +28,411 (55,662,803 to 55,691,214) |

The shipped APK grows by 28 KB: R8 had already stripped most of Room from it, so its dex shrinks by
only 73 KB, less than the store code adds to the native library.

**Keyboard-service cold start**, on `emulator-5556` with the Rust renderer: the keyboard's process
killed, then the render probe's empty editor focused (`scripts/render-benchmark`'s harness, the
`aw-host` timestamps), from the process's start (`/proc/<pid>/stat`, 10 ms ticks) to the input
view's first layout. Rounds alternated branch, main, branch, main, 11 starts each with the first
dropped, so 20 per build. Host load 7 to 12.

| Median of 20 (min to max), ms | main | branch |
|---|---|---|
| Process start to `onCreateInputView` | 344.0 (277.6 to 381.9) | 330.3 (255.2 to 398.2) |
| `onCreateInputView` to first layout | 582.2 (393.9 to 655.2) | 517.6 (417.8 to 628.4) |
| Process start to first layout | 923.6 (752.8 to 1002.8) | 838.5 (746.2 to 940.7) |

The branch was 85 ms faster at the median, inside both builds' spread, so no gain is claimed. The
stores open off the main thread on both builds.

## On the emulator

`emulator-5556`, screenshots in `build/stores-check/` (not committed):

1. **Main's build with data.** Synthetic history: four text clips, one pinned, and an image shared
   from Files; four dictionary words imported from a synthetic `.clb`
   (`01-main-history.png`, `02-main-dictionary-imported.png`).
2. **The branch installed over it** with `scripts/install-dev.sh`. `AwStores` logged "clipboard: 5
   clips, 1 files: ROWS" and "dictionary: 4 words: ROWS", and no database file is left. The
   stores decoded off the device match main's Room rows by hash: clips, media records and words all
   equal, with next ids 6 and 5 following main's sequences of 5 and 4
   (`03-branch-dictionary-migrated.png`, `04-branch-history-migrated.png`).
3. **Changes.** Pin, unpin and delete were each saved. After the process was killed the history
   was as left, and new clips took new ids (`05-branch-image-pinned.png`,
   `06-branch-after-process-death.png`). Clear history kept the pinned clips
   (`07-branch-cleared-keeps-pinned.png`). Deleting the image clip removed its record and its media
   file.
4. **Backup and restore.** A clipboard backup made on the branch, restored after clearing, brought
   back the same clips by hash, ids aside, as main's restore does (`08-branch-restored.png`).
5. **The Compose fallback.** With the Rust renderer off, the Compose clipboard read and changed the
   same store; then the Rust renderer was switched back on (`09-compose-fallback-history.png`,
   `10-rust-renderer-back-on.png`).
6. **Down and up again.** For the cold-start rounds, main's build went on twice and the branch back
   over it twice. Main's build made fresh empty databases each time; the branch imported nothing
   from them ("0 clips, 0 files: NOTHING", "0 words: NOTHING"), deleted them, and kept its store
   unchanged by hash.

## Kotlin files and the ledger

Counted lines, as `scripts/jvm-ledger.py` counts them:

| File | main | branch |
|---|---:|---:|
| `com/jtn0123/audiowhisper/stores/LegacyRoom.kt` (added) | | 129 |
| `FB/ime/clipboard/provider/ClipboardStoreFile.kt` (added) | | 34 |
| `FB/ime/dictionary/FlorisUserDictionaryDatabase.kt` (added) | | 51 |
| `FB/ime/clipboard/provider/ClipboardDatabase.kt`, now `ClipboardItem.kt` | 298 | 137 |
| `FB/ime/clipboard/ClipboardManager.kt` | 258 | 216 |
| `FB/ime/clipboard/provider/ClipboardMediaProvider.kt` | 154 | 147 |
| `FB/ime/dictionary/DictionaryManager.kt` | 138 | 79 |
| `FB/ime/dictionary/UserDictionary.kt` | 320 | 118 |
| **These eight** | **1,168** | **911** |

`FB` is `app/src/main/kotlin/dev/patrickgold/florisboard`. Six more production files changed only
in comments or within a line, 474 counted lines each side (`FlorisApplication.kt`, `BackupIO.kt`,
`ClipboardPolicy.kt`, `FlorisLocale.kt`, `RendererClipboardActions.kt`, `SettingsWords.kt`). With
them, the production Kotlin this branch touched went from 1,642 to 1,385 counted lines (−257).
`app/build.gradle.kts` went from 321 to 316.

Tests and harness: `StoresMigrationDeviceTest.kt` 172, `StoresBackupDeviceTest.kt` 54,
`ClipboardStoreMatchTest.kt` 67, `UserWordStoreMatchTest.kt` 76 and `shared/StoreKeys.kt` 49 were
added; `AcceptanceProbe.kt` went from 98 to 94 and `ReferenceUserDictionary.kt` from 131 to 121.
`scripts/stores-fixtures/MatchWriter.kt` (242) and `StoresFixtureWriter.kt` (261) run only inside
main's build, to make the fixtures.

The ledger has 404 rows (main 401): three added, `ClipboardDatabase.kt`'s row renamed to
`ClipboardItem.kt` and lowered, and four rows lowered. `--tighten` finds nothing to lower, and no
row was raised.

## Owed

In [owed checks](owed-checks.md):

* **P29**, the migration on the phone with its real data, compared by counts only, and the
  keyboard's cold start there.
* **E45**, Android's Auto Backup with the stores, both ways (`bmgr`). The in-app backup was run
  both ways; Auto Backup was not.
* The `nlp-unlearn` fixture op, which throws since ae452c49b removed the call it made. This branch
  found it because main's androidTest Java no longer compiled (fixed here: e4c699fec), not because
  of the stores.
* A clipboard history database at version 1 is kept and logged, never imported or deleted.
