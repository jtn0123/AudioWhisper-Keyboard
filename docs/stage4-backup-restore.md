# Stage 4: Back up and Restore

This page covers step 5 of [the Compose removal plan](compose-removal.md). Rust now draws the
**Back up** and **Restore** settings screens, and their Compose versions are deleted. One thing
they needed did not exist in Rust before: a way to use the **system file picker**. It is new
here, and steps 7 and 10 reuse it ([below](#how-steps-7-and-10-reuse-it)).

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/other/backup` | Rust | New here. |
| `settings/other/restore` | Rust | New here. |

Other opens both screens in Rust, with no Compose fallback. Their Compose routes stay as empty
entries that hand back to Rust, as in steps 2 and 4.

## What each screen shows

**Back up data:**

1. *Select backup destination*: *Local file system* or *Third-party app via share menu*.
2. *Select what to back up*: Preferences, Keyboard extensions and Theme extensions (ticked), then
   *Clipboard history* with Text items, Images and Videos (not ticked). The clipboard box is
   ticked, empty or partly ticked, and a tap on it ticks or clears all three.
3. A bottom bar with *Cancel* and *Back up*. *Back up* is off while nothing is ticked.

**Restore data:**

1. *Restore mode*: *Merge with current data* or *Erase and overwrite current data*.
2. *Select file*. Its summary is "No file selected", then the file's name.
3. Once a backup is read, *Selected backup archive*: when it was made, the app version and the
   app package, then a warning if there is one.
4. If it can be restored, *Select what to restore*, with the same groups as Back up. A group the
   backup lacks is off and says "Not in this backup". A clipboard file that will not read is off
   and says "Damaged in this backup, so it cannot be restored".
5. A bottom bar with *Cancel* and *Restore*. *Restore* asks first: the dialog names the groups
   and the date, and says what erasing does.

On both screens, a note above the bottom bar says what is happening ("Making the backup…") or
what went wrong. While work runs, only *Cancel* can be pressed.

## The file picker boundary

Rust asks for the picker; Kotlin opens it and says how it closed; Rust decides everything after.

1. **Rust asks.** `App::pick` adds a `pick` effect with a mode, the MIME types and a suggested
   name, and records a *purpose* in `Files.waiting`: which screen and step the answer is for.
   The modes are `create` (one new document), `open` (one document) and `openMany`.
2. **Kotlin opens the picker.** `SettingsFiles` builds the intent androidx's `CreateDocument`,
   `GetContent` and `GetMultipleContents` build (`PickRequest`), and launches it.
3. **Kotlin reports how it closed**: a `picked` report with `picked`, `cancelled` or `noPicker`
   (no app can show a picker), then a handle and display name for each document. The document's
   address stays in Kotlin under its handle. It never reaches Rust or the log, and each handle
   works once.
4. **Rust decides what follows.** Usually that is a `work` effect: a job, plus a group mask, an
   erase flag or a document handle. A `toast` effect shows a long toast.
5. **Kotlin runs the work** in `FileWorkQueue`. One job runs at a time, and a second is refused
   with `failed`. `discard` runs beside a job. Copying, zipping and unzipping run off the main
   thread. Each job ends with a `done` report: the result and any facts (a file name, a
   backup's details, how many restore steps ran).
6. **Reports outlive the activity.** The queue belongs to the process. A report goes to
   whichever screen is attached, or waits for the next one. When a screen attaches and no work
   runs, Kotlin sends `idle`. If Rust was still waiting for work, it knows the work was lost
   (Android closed the app), and says so.

The waiting purpose, the running job, both screens' choices, the backup read, the note and an
open confirmation are all in the saved state (version 3). A picker answer that arrives after
Android recreated the activity still finds its screen.

Every code is declared once in `crates/wire` (`settings_ui::files`), and Kotlin is generated:
the op `SettingsUiOp.FILES` (22), the effects `SettingsUiFileEffect`, the reports
`SettingsUiFiles`, `SettingsUiPickMode`, `SettingsUiPickOutcome`, `SettingsUiJob`,
`SettingsUiJobResult`, and the packets `SettingsUiPick`, `SettingsUiPicked`,
`SettingsUiDocument`, `SettingsUiWork` and `SettingsUiDone`.

### How steps 7 and 10 reuse it

They reuse, unchanged: the `pick` effect and `PickRequest`, handles in place of addresses,
`FileWorkQueue` (one job at a time, waiting reports, `idle`), the saved `Files` state and the
`toast` effect.

* **Step 7, user dictionaries.** Export is `pick(…, CREATE, &["text/plain"], name)`; import is
  `OPEN`. It adds its own purposes and `SettingsUiJob` values in `crates/wire`, its work in
  Kotlin, and its handlers in Rust.
* **Step 10, extensions.** Import is `OPEN_MANY` with several MIME types joined by `|`
  (`application/zip|…`). `SettingsFilesTest` already checks the intent that builds.

Two places route by owner and grow by one line each. `file_report` hands every answer and
report to the backup handlers today; with a second user, it picks the handler by purpose and by
job. `SettingsFiles.execute` hands every job to `BackupWork.run` today; it picks the runner by
job.

## What Kotlin does

Kotlin reads Android and does the file work. It decides nothing that is shown, and every
message is Rust's.

* `SettingsFiles` opens pickers, reads display names, keeps handles and shows toasts.
* `BackupWork` runs the jobs:
  * **prepare**: `BackupIO.backup`, then the zip;
  * **write**: copies the zip into the picked document, and deletes the document if that fails;
  * **share**: offers the zip through the share menu;
  * **read**: copies the picked file, unzips it, reads its details and checks each clipboard
    file decodes;
  * **apply**: `BackupIO.restore`, counting its steps;
  * **discard**: deletes archives no work needs.
* `BackupFiles` holds the archive steps with no Android in them, so JVM tests run them.
* `BackupIO` stays the zip and file adapter. It gained a per-step callback, so a restore that
  stops can say how far it got.
* `RendererAccessibility` gained a partly ticked checkbox state, a polite live region for status
  notes, and the CheckBox class for checkbox nodes.

## Improvements

Each one is new against the Compose screens.

1. **Restore shows the file name** and the backup's details before anything changes.
2. **Groups the backup lacks, or holds damaged, are off and say why.** Erase no longer empties a
   group the backup cannot refill.
3. **Restore asks first.** The dialog names the groups and the date. For erase, it warns that
   the whole clipboard history is cleared. TalkBack reads the message.
4. **Errors in plain language.** Compose showed the raw exception text in a toast. Each failure
   now has its own sentence, and it stays on screen.
5. **A newer backup gets its own warning.** It says settings this version lacks will not come
   back.
6. **No false warning on this app's own backups.** The vendor check knows only FlorisBoard's
   package, so Compose called every AudioWhisper backup "a third-party app".
7. **Progress notes.** While work runs, the buttons are off, so a job cannot start twice.
8. **Off the main thread:** copying, zipping and unzipping.
9. **Survives recreation:** picker answers, choices and running work.
10. **A fresh backup each time.** Compose could reuse a stale archive after a share.
11. **A failed save deletes the empty document** the picker made.
12. **Cancel says what happened**: "Backup cancelled. Nothing was saved."
13. **Clipboard files are checked before anything changes.** A damaged one is left out, so the
    restore cannot fail on it part-way.
14. **A restore that stops part-way says what came back**, what did not, and whether erasing
    had begun. It keeps the backup for another try.
15. **Leaving a screen deletes its cached archives.** Compose deleted them only on *Cancel*.
16. ***Restore* is off until something restorable is ticked.**
17. **Status notes are a live region**, so TalkBack reads them as they change.
18. **The clipboard box reports "partly ticked"** to TalkBack.

## Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| Restore offers only the groups the backup holds. Compose let any group be ticked, and erase then emptied a keyboard or theme folder the backup lacked. | That lost data and restored nothing ([decision](decisions.md#restore-offers-only-the-groups-a-backup-holds-2026-11-21)). With merge, or with groups the backup holds, the restore is Compose's exactly (see [Data safety](#data-safety)). |
| *Restore* asks first. Compose restored at once. | Erase deletes data. |
| Errors are a note on the screen. Compose showed a toast with the exception text. | A toast vanishes and the exception text is not plain language. Success is still Compose's toast, and the screen closes. |
| The backup's details read date, version, package. Compose read package, version, date. | The date is what tells one backup from another. |
| Compose's outlined boxes are headings. | Rust has no outlined box. The rows and bottom bar are the same. |
| The clipboard box on Restore counts only the clips the backup can restore. | A clip that cannot be ticked should not leave the box partly ticked. |
| This app's own package gets no vendor warning. FlorisBoard's still gets none. Any other still does. | See improvement 6. |
| A newer backup gets "made by a newer version". Compose showed the general "another version" warning. | It says what will be lost. |
| Restoring still runs on the main thread, as in Compose. | `BackupIO.restore` writes preferences and the clipboard database. Moving it is its own change. |

## Data safety

**A backup made by the Compose screen restores exactly as before.** `BackupFilesTest` makes
backups with Compose's own steps, copied from `BackupScreen`, and FlorisBoard's original zip
(`ReferenceZipUtils`), so no code under test makes the fixtures.

* Back up writes Compose's archive: the same file name and the same entries, byte for byte, for
  five group selections.
* For each of the 63 group combinations Compose could back up:
  * the backup reads with the right details;
  * it unpacks to the same files, byte for byte, as Compose's restore unpacked;
  * every selection, merging or erasing, leads to Compose's restore plan.

  The plan is what `BackupIO` carries out. There is one exception: erase no longer clears a
  keyboard or theme folder the backup lacks. The plan is exactly Compose's when merging, or when
  the ticked groups are in the backup.
* `SettingsBackupFlowTest` restores Compose backups through the real native screens and
  strings. It covers four group sets, merging and erasing, from picking the file to the success
  toast. The restore Rust asks for is the plan Compose ran for the same choices.

**Every failure leaves data no worse off and says what happened:**

| What happens | Your data | The screen says |
| --- | --- | --- |
| Back up: the save picker is cancelled | Unchanged. The archive is deleted. | Backup cancelled. Nothing was saved. |
| Back up: no app can show a picker | Unchanged | This device has no app for choosing where to save a file… |
| Back up: the backup cannot be made | Unchanged | The backup could not be made. Nothing was saved… |
| Back up: the file cannot be written | Unchanged. The empty document is deleted. | The backup could not be saved there. Try another place. |
| Restore: the picker is cancelled | Unchanged. A backup read before stays. | No file was chosen, so nothing changed. |
| Restore: the file is not a ZIP, is empty, or is cut short | Unchanged | That file could not be read. Nothing changed. |
| Restore: a ZIP with no backup details | Unchanged | That file is not a keyboard backup. Nothing changed. |
| Restore: the details do not read | Unchanged | The details in that backup could not be read… |
| Restore: version below 64 or no package | Unchanged. *Restore* is off. | Compose's "invalid metadata" error. |
| Restore: made by a newer version | Can be restored | This backup was made by a newer version… |
| Restore: a damaged clipboard file | That group cannot be ticked | Damaged in this backup, so it cannot be restored |
| Restore: stops part-way | The groups before the failure are back. The rest are as they were, unless erasing had begun. | The restore stopped part-way. Restored: … Not restored: … (and, if erasing had begun, that some old data is gone) |
| Android closes the app during work | As far as the work got | The app was closed before the work finished… |

`BackupFilesTest` pins the broken files. Raw bytes, an empty file, a ZIP cut to 40 or 200
bytes, and a missing file are all unreadable. A ZIP of photos is not a backup. `{`, `{}` and a
text version code are bad details.

## Tests

* **Rust** (16 new):
  * `settings_ui_backup.rs` (7): Other opens both screens; Back up's defaults and its tri-state
    box; prepare, pick and write, then the toast; share; cancel and failures; work lost with the
    app; leaving.
  * `settings_ui_restore.rs` (8): reading and showing a backup; every warning; the confirmation;
    part-way failures; read failures; the success toast; malformed reports; the saved state,
    with bad state refused.
  * The backup crate (1): the clipboard box counts only the clips offered.
* **JVM** (12 new):
  * `BackupFilesTest` (4): the fixtures above.
  * `SettingsBackupFlowTest` (2): the restore flow above, and what TalkBack hears (radio
    states, the partly ticked box, live notes, buttons off while busy).
  * `SettingsFilesTest` (6): the intents, the `picked` report, and the queue (waiting reports,
    `idle`, one job at a time, a job that throws, handles taken once).
* **Updated:** three route-list match tests now expect the two new native routes.
  `SettingsOtherThemeDevice` now checks both rows open Rust screens. It compiles, but it did not
  run here (see [Owed](#owed)).

## Emulator check

On `emulator-5558` (`aw_keyboard`, Android 16), with `42f209fca` installed by
`scripts/install-dev.sh`. Files were picked in the system picker (DocumentsUI), in Downloads.

* **Back up to a file.** Other opened Back up in Rust. The accessibility tree showed the
  destinations as radio buttons and the groups as checkboxes, each with its state. The save
  picker opened with the suggested name `backup_<package>_117_<time>.zip`. Closing it said
  "Backup cancelled. Nothing was saved." and wrote no file. Saving wrote a 1,400-byte archive
  holding the preferences file, `files/ime/keyboard/`, `files/ime/theme/` and
  `backup_metadata.json`, and the screen went back to Other.
* **Back up by sharing.** The share sheet opened with the archive. Closing it left the screen
  saying "Choose an app to send the backup to."
* **Restore the wrong thing.** Closing the open picker said "No file was chosen, so nothing
  changed." and Restore stayed off. A text file said "That file could not be read. Nothing
  changed." A ZIP with folders but no metadata said "That file is not a keyboard backup. Nothing
  changed."
* **A ZIP with a file in a folder but no entry for the folder** reads as "could not be read".
  `ZipUtils.unzip` does not make a missing parent folder. That code is unchanged, so Compose
  failed on it too. Every backup this app writes has its folder entries.
* **Restore the backup.** The screen showed its date, app version (`0.5.2-debug+42f209fca
  (117)`) and package, with no warning, and offered only the three groups it held. The dialog
  named those groups and the date. After it, "Successfully restored data!" and back to Other.
* **The theme came back.** The backup was made with the settings theme on System default, then
  the theme was set to Dark.
  * **Merge kept Dark.** This is correct. Jetpref saves only values that differ from the
    default, so the backup held no settings theme, and Merge keeps what a backup does not hold.
    Compose used the same import and did the same.
  * **Erase and overwrite brought System default back.** The dialog also warned that the whole
    clipboard history is cleared. The screen redrew light.
  * **Merge brings back a stored value.** With the theme on Light, a new backup (1,415 bytes)
    held `other__settings_theme`. After switching to Dark, a Merge restore set Light again.
* **Leaving forgets the choices.** After a restore, Restore opened again in Merge with no file.
* **Afterwards** the theme was set back to System default, the test files were deleted, and
  the keyboard was still the input method.

## Deleted

`BackupScreen` (362) and `RestoreScreen` (265), `CacheManager`'s Compose-only fields (4), 4
lines in `Routes.kt` rewritten as 2, and an import in `ExtensionEditScreen` (1): 636 non-blank
production lines deleted, 2 added. Compose's two failure strings go too. `RadioListItem` moved
out of `BackupScreen` into its own file (42 lines with its header), unchanged.
`ExtensionEditScreen` is its only user, and it goes at step 10.

Over the whole branch, against `88df513ed`, Kotlin gains 431 non-blank lines (the picker
adapter, the file work, `RadioListItem` and string ids) and loses 640. Rust gains 1,481 and
loses 32. String resources gain 33 and lose 2.

## Owed

* **No phone check.** Everything above ran on the emulator only.
* **Merge cannot bring back a default.** A backup holds no setting left at its default, so a
  Merge restore keeps whatever is set now. Erase does reset it. This is Jetpref's file format.
  It can change once Rust owns the preferences file ([all-rust.md](all-rust.md), phase 4).
* **A ZIP missing its folder entries reads as unreadable.** `ZipUtils.unzip` does not make a
  missing parent folder. It is unchanged from Compose and shared with extension import. The app
  never writes such a file, but another zip tool might.
* **The instrumented check did not run.** `SettingsOtherThemeDevice` is updated and compiles.
  Running it needs the connected test tasks, which this step did not use.
* **No mutations for these screens.** The existing `backup-opens-restore` mutation still
  applies, and a Rust test also catches it.
* **Restoring on the main thread**, as Compose did. A large clipboard could make the screen
  stall while it restores.
* **Erase clears the whole clipboard history**, even if only some clipboard groups are ticked.
  This is Compose's behaviour, kept so the plan matches. The dialog now warns about it.
* **A shared archive stays in the cache** until the next share or backup, because the app that
  took it may still be reading it.
