# Stage 4: User dictionaries

This page covers step 7 of [the Compose removal plan](compose-removal.md). Rust now draws the
user dictionary screens, for the keyboard's own (Floris) dictionary and for Android's (System)
dictionary. UserDictionaryScreen is deleted. The screens reuse step 6's
[editable list](stage4-languages.md#the-editable-list) and item editor. The words stay where
they were: the Floris dictionary in Room, and the System dictionary in Android's provider.

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/dictionary/user-dictionary/FLORIS` | Rust | New here. The languages, then the actions. |
| `settings/dictionary/user-dictionary/FLORIS/words` | Rust | New here. Words for all languages. |
| `settings/dictionary/user-dictionary/FLORIS/words/<language>` | Rust | New here. One language's words. |
| `settings/dictionary/user-dictionary/FLORIS/word/add` | Rust | New here. Add word. |
| `settings/dictionary/user-dictionary/FLORIS/word/edit/<id>` | Rust | New here. Edit word. |
| `settings/dictionary/user-dictionary/SYSTEM` … | Rust | New here. The same five, for the System dictionary. |

The Dictionary screen opens both dictionaries in Rust, with no Compose fallback. The Compose
route `Settings.UserDictionary` stays as an empty entry that hands back to Rust, as in steps 2
and 4 to 6.

## What each screen shows

**A dictionary**, the old screen before a language was picked:

1. Each language the words are stored under, once. *For all languages* comes first, then the
   others sorted by language, as `queryLanguageList()` sorted them. A tap opens that
   language's words.
2. With no words, the old message "No words in this dictionary" as a card.
3. *Add word*, *Import* and *Export*. Compose had these in the overflow menu and the floating
   button. *Export* is off while the dictionary is empty. All three are off while the
   dictionary is switched off in Dictionary settings.
4. On the System dictionary only, *Open system manager UI*.

**A language's words**, titled by the language:

1. A search field that matches a word or its shortcut.
2. When nothing matches, "No word found for …" with the search.
3. The editable list. Each word is summarised by its frequency, and its shortcut if it has one,
   with the old strings. Each has a *Delete* button. A tap opens the editor.
4. *Add word* at the end, and the undo bar after a delete.

**Add word** and **Edit word**, the old dialog's four fields on their own screen: *Word*,
*Frequency (between 1 and 255)*, *Shortcut (optional)* and *Locale (optional)*. Then *Cancel*,
and *Add* or *Apply*. Editing puts *Delete* in the top bar, as Languages & Layouts does. The
old error messages show under each field after the first *Save*, then follow the typing.

## How the list and editor were reused

**The list** (`settings_ui/items.rs`) gained a second source, `Source::Words`, as step 6
planned. `words/list.rs` gives it items, ops and its `Words`: the empty message, the add label,
and one new flag.

* **`reorder: false`** hides *Move up* and *Move down*, their custom actions and their focus
  stops. A dictionary has no order. The item keeps only *Delete*. The flag is read in three
  places in `items.rs`: the buttons shown, the custom actions, and the focus order, which starts
  at the delete button.
* **The ids, undo bar, empty state and saved slots** are unchanged. An item's key is the word's
  row id, so a slot belongs to the stored row.
* **Undo** keeps step 6's rule: it lasts until the user leaves the list or changes it again.
  Undo stores the row again as it was, unless a row with its id came back meanwhile.

**The editor** reuses the item editor's parts.

* *Cancel* and *Save* are `Lang::Cancel` and `Lang::Save`, the subtype editor's footer.
* *Delete* is the subtype editor's top bar button (`languages::form::DELETE_ID`). Deleting from
  the editor closes onto the list with focus on *Undo*, as a delete in the list does.
* The draft, its "checked" flag, the red outlines and the error lines are the subtype editor's.
* The fields are new: four titled outlined fields (`words/form.rs`). Each is one line, Enter is
  *Done*, and the frequency field asks for the number keyboard. The single-line field flag that
  step 6 had for its search field was renamed `line_field` and is shared.

**The language stays a text field.** Step 6 planned a row that opens Select language. The
dialog let the user type any tag, and the stored words keep tags Select language cannot offer
(`en-us`, `all`, a language with no keyboard). A field keeps every stored word editable as it
is. A new word added from a language's list starts with that language filled in.

## The boundary

All of it is declared in `crates/wire` (`settings_ui/words.rs` and two job names in
`settings_ui/files.rs`). No `external fun` was added, and `lib/rust/src/main/kotlin` and
`crates/boundary` are unchanged. Everything goes through the settings op channel.

* **One new op, `WORDS` (24).** Kotlin's answer to every word effect: a report of
  `SettingsUiWord` rows. The first row says which dictionary, whether it is on, and what the
  effect did (`none`, `imported` or `failed`). Then every stored word in `queryAll()` order, then
  each locale once with its Java display name and display language.
* **Seven word effects** (`SettingsUiWordEffect`): `words` (read), `wordAdd`, `wordChange`,
  `wordDelete`, `wordRestore` (a delete undone), `wordImport` and `wordManager`. Each carries
  the dictionary and the rows it acts on.
* **Two file jobs** on step 5's picker: `wordsRead` reads the picked document as text, and
  `wordsWrite` writes an export's text to it. `work` and `done` each gained a `TEXT` field.

Rust does everything else: grouping by language, sorting, the search, the checks, the duplicate
check, reading an import, and writing an export's text.

## What Kotlin does

`SettingsWords.kt` is the adapter, 115 counted lines.

* It opens the DAO the old screen used: `florisUserDictionaryDao()` or
  `systemUserDictionaryDao()`, after `loadUserDictionariesIfNecessary()`.
* It runs one change at a time, in order, off the main thread, so a delete and its undo reach
  the database as they were asked.
* *Add* and *Apply* store the locale normalised by `FlorisLocale.fromTag(…).localeTag()`, as
  the dialog did. *Delete* deletes by row. *Undo* inserts the row as it was. In Room it keeps
  its id, which AUTOINCREMENT never hands out again. If the id is taken, it gets a new one.
* An import upserts each word in order, as `importCombinedList` did: `queryExact` with the word
  and its locale, then `update` with the found id or `insert`. In the Floris dictionary it is
  one Room transaction.
* The file work reads at most 4 MiB, and writes `dictionary=`, the document's last path segment,
  then Rust's text.
* A database error becomes the `failed` outcome. Nothing is logged: no word, shortcut or locale.

## Taken from Gboard

Gboard's Personal dictionary was the reference for behaviour, not for looks.

* The languages come first, then a language's words.
* A language's word list is titled by that language.
* A word added from a language's list belongs to that language.
* A word opens on a screen of its own, with *Delete* in the top bar.

## Improvements

1. **One entry per language.** `en-us`, `en_US` and `EN_us` are one language, as the keyboard
   reads them. Compose listed each spelling separately.
2. **Sorted and searchable.** A language's words are alphabetical, and the search matches a word
   or its shortcut. Compose listed them in database order with no search.
3. **Titled by language.** Compose's word list had the dictionary's title and a close button.
4. **A new word starts in its language.** Compose started every new word with no locale.
5. **Undo.** A delete hides the word at once and offers *Undo*, with focus on it, from the list
   or the editor. Compose deleted with no way back.
6. **Deleting a language's last word stays on its list**, still titled by the language, with
   *Undo*. Compose jumped back to the languages.
7. **Duplicates are refused.** Adding a word already stored for that language says so under the
   field. Compose stored a second copy. Its own edit is not a duplicate.
8. **A frequency with spaces no longer crashes.** The dialog accepted " 12" and then threw in
   `toInt`. It is now read trimmed.
9. **Single-line fields, and numbers for the frequency.** The dialog's fields took line breaks.
10. **All or nothing imports.** A file with a bad line imports nothing and names the line:
    "Line 5 of that file is not a word. Nothing was imported." Compose wrote the lines before
    it and showed the raw exception text. The Floris import is one transaction.
11. **Export reads the dictionary again** just before writing, so it is never stale.
12. **Plain errors.** A file that cannot be read or written, a database that refuses a change,
    and a device with no file picker each have a sentence, not the exception's message or
    "Database handle is null".
13. **A 4 MiB import limit.** Checking the whole file before writing means holding it, so
    its size is capped. An export takes about 30 bytes a word, so 4 MiB is over 100,000 words.
    Compose read any size, line by line.
14. **The actions are rows.** *Import*, *Export* and *Open system manager UI* were in an overflow
    menu, and *Add word* was a floating button over the list. They are now in the focus order
    with everything else.
15. **A switched-off dictionary's actions are off.** Compose showed it as empty, and adding a
    word there did nothing.

## Differences from Compose

* The editor is a screen, not a dialog. Back and *Cancel* both close it without a change.
* Nothing is exported from an empty dictionary. Compose wrote a file with only the header line.
* The pickers are unchanged. The import's is the `GetContent` intent for `*/*`, and the
  export's is `CreateDocument` with `my-personal-dictionary.clb` and `application/octet-stream`.
  Step 5's picker now sends both.
* The empty message is step 6's warning card. Compose showed a line of italic text.
* An import where the same word appears twice for a language still writes the later line over
  the earlier one, as Compose did.

## Data safety

**Rows are written exactly as before.** The JVM tests compare the DAO calls with a frozen copy
of the old code. `ReferenceUserDictionary` is UserDictionary's `importCombinedList` and
`exportCombinedList` and the dialog's checks and entry, as they were at `f5a6fc180`. Only the
document, the package name and the clock became parameters. Same calls on the same DAO means
the same Room rows and the same provider values. `RecordingUserDictionaryDao` records the calls
and behaves like Room (it keeps an inserted id and orders by id) or, for the System
dictionary, like the provider (it ignores the id and orders by frequency).

* `addingAndEditingWriteTheEntryTheDialogWrote` puts 2,500 random drafts into each dictionary's
  real native editor, about half adds and half edits, and presses *Save*. Each makes the call the
  dialog made, with the entry its `onConfirm` built, or none when the dialog refused it. The two
  intended differences are checked as such: a frequency with spaces is stored trimmed where the
  dialog threw, and a duplicate is refused where the dialog stored it. Every outcome is reached
  at least 50 times.
* `deleteRemovesTheStoredRowAsTheDialogDidAndUndoStoresItAgain`: the same `delete` as the
  dialog. Undo stores the row again: with its id in Room, and as a new row in the provider.
* `anImportMakesTheCallsImportCombinedListMadeAndAnOldExportImports`: an old export file, into
  a full and an empty dictionary, and a file with long and short keys, mixed case, spaces,
  Windows line ends, an unknown key and a repeated word, make the same calls in the same order
  and leave the same rows.
* `anImportWithABadLineWritesNothingWhereComposeWroteTheLinesBeforeIt` checks that intended
  difference.
* `anExportIsByteForByteWhatExportCombinedListWrote`, over the native screens and the picker.

**Stored formats are frozen as literals.** `SettingsWordsTest` freezes the export file of the
five sample words, the locale the dialog stored for each typed tag, and a report row. The cargo
tests freeze an old export file and check that it imports and that a new export is the same
bytes.

**Nothing is written by looking.** Opening a screen, the search, *Cancel*, Back and a malformed
report write nothing.

**No dictionary content is logged**, in code or tests. Every test word is made up, and the
frozen reference's error messages had the line's text taken out.

## Accessibility

The screens use the same nodes as the other Rust settings screens. On top of that:

* each word's node offers *Delete* as a custom action, and only that;
* each *Delete* button says which word it deletes (its summary);
* the undo message is a status node, and focus moves to *Undo*;
* a field's error is a text node under it, and also the field's summary;
* the editor's focus order is Back, *Delete*, the four fields, *Cancel*, *Save*.

## Tests

* **Rust** (32 new), in `tests/settings_ui_words/`:
  * `list.rs` (12): routes both ways; grouping by locale; the languages' order and names; the
    empty card and switched-off actions; the system manager; the words' order and summaries;
    only *Delete* where a subtype has *Move up* and *Move down*, with its custom actions and
    focus order; the search; delete and undo; the last word and its list's title; how long
    undo lasts; outcome
    messages and a malformed report.
  * `editor.rs` (10): where a new word starts; editing with *Delete*; the checks; errors after
    *Save*; the entry built; duplicates; *Apply* and *Delete*; *Cancel*; the fields; an editor
    filling in when the report comes.
  * `transfer.rs` (8): line ends as `readLine` had them; each line as the old import read it;
    the bad line's number; old files and the exact export bytes; the import and export round
    trips; a dismissed picker, no picker, a file error and lost work.
  * `saved.rs` (2): the draft, search, undo and file work come back; a malformed state changes
    nothing.
* **JVM** (11 new): `SettingsWordsMatchTest` (5), in [Data safety](#data-safety), and
  `SettingsWordsTest` (6): the frozen export and locale literals, the report's rows, a
  dictionary that is off, the import transaction, and the read limit.
* **Updated:** the four route-list tests expect the eight new native routes, and the group two
  test expects the words to be asked for as a dictionary opens. `settings_ui_screens` lists the
  new routes. `settings_ui_languages_picker` uses the renamed `line_field`.

## Mutations

`scripts/settings-ui-mutations.py` has 26 new ones for these screens (now in
`scripts/settings_ui_mutation_cases/words.py`). Each must make a test
fail, and the code is put back after each one.

**All 26 were caught**, each by a fresh assertion, and the restored source passed
(`restored_exit` 0). The last one, `word-title-forgets-name`, came with the emulator check's
fix and was run on its own.

| Mutation | What it breaks | Caught by |
| --- | --- | --- |
| `word-frequency-untrimmed` | *Save* reads the frequency untrimmed again | `SettingsWordsMatchTest` |
| `word-duplicate-allowed` | A word already in its language is saved again | `SettingsWordsMatchTest` |
| `word-locale-not-normalised` | *Add* and *Apply* store the locale as typed | `SettingsWordsMatchTest`, `SettingsWordsTest` |
| `word-import-not-upserted` | An import inserts a word that is already there | `SettingsWordsMatchTest`, `SettingsWordsTest` |
| `word-import-not-in-transaction` | The import runs outside Room's transaction | `SettingsWordsTest` |
| `word-undo-new-id` | Undo stores the row under a new id | `SettingsWordsMatchTest` |
| `word-import-null-locale-kept` | An import keeps `l=null` as the locale `null` | `SettingsWordsMatchTest` |
| `word-import-partial` | An import skips a bad line and writes the rest | `SettingsWordsMatchTest` |
| `word-import-line-off-by-one` | The bad line's number is one too low | `SettingsWordsMatchTest` |
| `word-export-null-locale-empty` | An export writes `l=` where it wrote `l=null` | `SettingsWordsMatchTest` |
| `word-export-not-reread` | Export writes the words on screen without reading them again | `SettingsWordsMatchTest` |
| `word-delete-sends-restore` | *Delete* asks Kotlin to store the word | `SettingsWordsMatchTest` |
| `word-files-to-backup` | A picked file goes to Back up's work, not the dictionary's | `SettingsWordsMatchTest` (3) |
| `word-string-id-missing` | "No app can choose a file" has no string | `SettingsResourcesTest` |
| `word-list-reorders` | Words get *Move up* and *Move down* | `settings_ui_words` (list) |
| `list-paints-move-for-words` | The word list paints the move buttons | `settings_ui_words` (list) |
| `list-focus-ignores-flag` | The move buttons stay in the focus order | `settings_ui_words` (list) |
| `word-search-ignores-shortcut` | The search matches the word only | `settings_ui_words` (list) |
| `word-empty-card-under-search` | A search that finds nothing shows the empty dictionary's card | `settings_ui_words` (list) |
| `word-added-without-language` | A word added from a language's list has no language | `settings_ui_words` (editor) |
| `word-title-not-language` | A language's words are not titled by the language | `settings_ui_words` (list, 3) |
| `word-title-forgets-name` | A list whose last word was deleted is titled by its tag | `settings_ui_words` (list) |
| `word-frequency-text-keyboard` | The frequency field asks for the text keyboard | `settings_ui_words` (editor) |
| `word-undo-outlives-list` | Undo is still offered after leaving the list | `settings_ui_words` (list) |
| `word-editor-delete-keeps-focus` | *Delete* in the editor leaves focus nowhere | `settings_ui_words` (editor) |
| `word-saved-search-dropped` | The search is lost when the screen comes back | `settings_ui_words` (saved) |

## Emulator check

The check ran on emulator-5558 (AVD `aw_keyboard`, Android 16), muted, with this branch's
build. It used a made-up word, with the shortcut `zx`. Every step was confirmed in the stored
state. The Room table was read with `run-as`, and the provider with `content query`. Each read
printed row counts and columns, never a word.

1. **Add.** The path was Typing → User dictionaries → Manage internal user dictionary →
   *Add word*. The word was stored as a new row: frequency 255, no locale, and the shortcut. It
   appeared under *For all languages* with "Frequency: 255 | Shortcut: zx" and only a *Delete*
   button.
2. **Edit.** Opening the word, typing `en-us` as its locale and pressing *Apply* changed the
   same row's locale to `en_US`, as the dialog stored it. The word moved to a new *English
   (United States)* list.
3. **Delete and undo.** *Delete* removed the row and showed "Deleted …" with *Undo*. *Undo*
   stored the row again with the same id, frequency, locale and shortcut.
4. **Export.** The system picker opened in Downloads with `my-personal-dictionary.clb` filled
   in. *Save* wrote the frozen format:
   * the header, `dictionary=` plus the document's last path segment, the date,
     `generated-by=` our package and `version=1`;
   * one line per row, in Room's order;
   * `l=null` for the row for all languages;
   * `;s=` only on the row with a shortcut;
   * a final newline.
5. **Import.** The word was deleted first, then *Import* picked the exported file. The word came
   back as a new row with its frequency, locale and shortcut. The row that was already there was
   updated with its own values, and its contents hashed the same as before.
6. **Suggestions.** Our keyboard was typing into system Settings' search, with keys tapped by
   coordinates. After `z`, and again after `zorb`, the strip offered the word.
7. **System dictionary.** *Manage system user dictionary* opens the Rust screen:
   * the empty card;
   * *Add word* and *Import*;
   * *Export*, off while the dictionary is empty;
   * *Open system manager UI*, which opened Android's Personal dictionary.

   The provider stayed empty throughout.

**Found and fixed:** after step 3's delete, the emptied list was retitled `en_US`. Its title
came from the report, and the report no longer named a language with no words. A list now keeps
the name a report last gave it (`words.names`). The commit adds an assertion to
`deleting_a_languages_last_word_stays_on_its_list_with_undo` and the mutation
`word-title-forgets-name`. Reinstalled, the title stays *English (United States)* after the
delete, and Undo brings the word back.

**Put back afterwards:**

* The Room files were restored from the copy taken first: 1 row, the same hash and the same
  AUTOINCREMENT sequence.
* The export was deleted through MediaStore.
* The provider is still empty.
* Our keyboard is the default again, and the enabled list is unchanged.
* The volumes are as found.
* Learned words and typing counters were not touched.

The emulator now runs this branch's build, so the prefs file's last-used version names it.

## Deleted

Counted as non-blank lines in this step's commits. They were written on `f5a6fc180` and
rebased onto `d85a94476`. The rebase changed one line: the `WORDS` op arm, put in the form
`d85a94476` gave `awnative`'s settings call.

| Area | Deleted | Added |
| --- | ---: | ---: |
| Kotlin production | 542 | 210 |
| Rust production (`render`, `wire`, `awnative`) | 83 | 1,648 |
| Rust tests | 2 | 1,120 |
| Kotlin tests | 7 | 451 |
| String resources | 1 | 7 |
| Mutation script | 4 | 66 |
| Cargo manifests | 0 | 5 |

The Kotlin deletions are UserDictionaryScreen (385), UserDictionaryValidation (62),
UserDictionary's import and export (87), and the route and adapters (8). The Kotlin additions
are SettingsWords (140), its string ids (36), UserDictionaryType in its own file (20, most of
it the licence header), and 14 lines in Routes and the adapters. The string resources lose the
dialog's locale error and gain the seven new messages. The manifests give `render` the
workspace's `chars` and `utilities` crates, for Kotlin's string rules and the dictionary's
checks. The ledger's changes are in `docs/jvm-ledger.tsv`'s notes.

## Owed

* **No phone check.** Everything above ran on the emulator only.
* **Compose's handback cannot tell the two dictionaries apart.** The empty
  `Settings.UserDictionary` entry hands back to Rust's dictionary list, whose rows lead to both.
  Nothing in the app opens that Compose route now.
* **Our keyboard typing into these fields.** The emulator check typed into them with
  agent-device's own keyboard, which agent-device needs: it refuses text entry into our app
  while our keyboard is selected. A person typing a word with our keyboard is owed, as E21 owes
  it for Select language's search. agent-device's `fill` also fails on Rust fields, which do
  not report focus; that is owed since [Other and Theme](stage4-other-theme.md#owed).
* **`SettingsWords` runs only on a device.** It needs the activity and DictionaryManager. Its
  functions run in JVM tests, and the emulator check covers the rest.
