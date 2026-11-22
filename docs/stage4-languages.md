# Stage 4: Languages and layouts

This page covers step 6 of [the Compose removal plan](compose-removal.md). Rust now draws the
four language screens, and their Compose versions are deleted. One thing they needed did not
exist in Rust before: an **editable list**, with items the user adds, opens, moves and deletes,
and an editor screen for one item. It is new here, and step 7 reuses it
([below](#how-step-7-reuses-it)).

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/localization` | Rust | New here. Languages & Layouts. |
| `settings/localization/select-locale` | Rust | New here. Select language. |
| `settings/localization/language-pack-manage` | Rust | New here. Language packs. |
| `settings/localization/subtype/add` | Rust | New here. Add subtype. |
| `settings/localization/subtype/edit/<id>` | Rust | New here. Edit subtype. |

The settings list opens all four in Rust, with no Compose fallback. Their Compose routes stay
as empty entries that hand back to Rust, as in steps 2, 4 and 5. The language pack manager's
*Import* and a pack's page are still Compose's extension screens (step 10), and Back from them
returns to Rust.

## What each screen shows

**Languages & Layouts**, as LocalizationScreen had it:

1. *Display language names in*: the app's language, or each language's own.
2. *Display keyboard labels in subtype language*.
3. *Language packs*, which opens the pack manager.
4. Under *Subtypes*, the editable list. Each subtype is titled with its language and
   summarised by its characters layout, symbols layout and currency set. Each has *Move up*,
   *Move down* and *Delete* buttons. A tap on the subtype opens its editor.
5. With no subtypes, Compose's warning that English/QWERTY is used instead.
6. *Add subtype* at the end of the list.

**Select language**, as SelectLocaleScreen had it: a search field, then every locale the
keyboard knows, sorted by its shown name. The search matches any of a locale's three names, or
the start of its tag. When nothing matches, it says so with the search term. With no search,
the device's languages come first under *Suggested languages*, then *All languages*.

**Language packs**, as LanguagePackManagerScreen had it: *Import*, then a box for each pack
with its title and id (both open its page), its items sorted by label, and *Delete* when the
extension manager allows it. *Delete* asks first, naming the pack.

**Add subtype** and **Edit subtype**, as SubtypeEditorScreen had them:

1. When adding, the suggested presets card: a preset for each device language the keyboard has
   one for, or a message that there is none, then *Show all*. Add subtype opens the list of
   every preset first, as Compose did.
2. Language, popup mapping and characters layout.
3. Suggestion provider.
4. Symbols and symbols 2 layouts, composer and currency set.
5. Numeric, numeric advanced and numeric row layouts.
6. Phone and phone 2 layouts.
7. *Cancel* and *Save* in a bottom bar. When editing, *Delete* in the top bar.

Each field shows "‒ select ‒" until it has a value. *Language* opens Select language; the
others open a list of what the keyboard offers. *Save* checks every field. If one is unset, the
unset fields are outlined in red and a dialog says so. Adding a subtype that exists already
says that. Otherwise the list is written and the editor closes.

## The editable list

`settings_ui/items.rs` draws a list from data. A screen names a **source**, and the list asks
the source for its items and hands it each tap.

* **An item** is a title, a summary and a key. The key stays the same while the item moves.
* **A tap** becomes an `Op`: `Open(i)`, `Up(i)`, `Down(i)`, `Delete(i)`, `Add` or `Undo`. The
  source carries it out. For subtypes that is `languages/list.rs`, which runs SubtypeManager's
  own `reorder` and `remove` plans and writes the list.
* **Ids.** Item `slot` has node ids `8000 + 4 × slot` (the item) and the next three (its
  buttons). A slot belongs to an item's key, not its place, so an item keeps its ids while it
  moves, and accessibility focus moves with it.
* **Reordering without dragging.** Each item has *Move up* and *Move down* buttons, off at the
  ends. Its node also offers them, with *Delete*, as **custom accessibility actions**, so
  TalkBack's actions menu moves an item without finding its buttons. Each action's id is the id
  of the button it presses. None of those ids is a single bit, so none clashes with a standard
  action's.
* **Undo.** After a delete, an undo bar stays under the list: "Deleted German (Germany)" and
  *Undo*. Focus moves to *Undo*, and the message is a status node, so TalkBack reads it. There
  is no timer to beat. The bar goes when the user leaves the screen or changes the list again.
  Undo puts the item back where it stood, unless one with its key came back meanwhile.
* **The empty state** is the source's message, and *Add* stays under it.
* **The saved state** keeps each slot's key and the delete to undo.

**The editor** is a column of rows from the catalogue (`Kind::Pick(Field)` and
`Kind::Lang(…)`), over a **draft** the screen owns. The draft starts blank when adding, and as
the stored item when editing. *Save* checks it, runs the source's `add` or `modify` plan and
closes. *Delete* in the top bar removes the stored item and returns to the list with the undo
bar. The draft and its "checked" flag are in the saved state.

### How step 7 reuses it

The user dictionary is a list of words, each with a shortcut and a language.

* **The list.** Step 7 adds `Source::Words` and a `words.rs` with its items, its ops and its
  `Words` (the empty message and the add label), as `languages/list.rs` has for subtypes. The
  ids, buttons, custom actions, undo bar, empty state and saved slots come with it unchanged.
  Three `match`es in `items.rs` route by source, and each grows by one arm.
* **Moving.** A dictionary has no order. Step 7 adds a flag to `Words` that leaves out *Move
  up* and *Move down*, their custom actions and their focus stops.
* **The editor.** A word, its shortcut and its frequency are fields; the language is a
  `Field::Locale`-like row that opens Select language. The draft, the checks on *Save*, the red
  outlines, the message dialog and *Delete* with undo are reused.
* **Kotlin.** Words live in a database, not a preference. Step 7 adds a report like
  `LANGUAGES` and effects that add, change and delete a word, in `crates/wire`.

## The boundary

Rust edits and writes the subtype list itself. Kotlin reads what Rust cannot: the keyboard's
resources and Java's locale names.

1. **Rust asks.** A language screen that needs the report adds a `languages` effect. The list
   and the pack manager ask each time they open; the others reuse the report, and ask only
   when there is none. A second screen does not ask again while an answer is on its way.
2. **Kotlin reports.** `SettingsLanguages` sends a `LANGUAGES` report (op 23): rows of eight
   fields, each with a kind. The kinds are the subtype list as Kotlin read it, the locales
   Select language offers, the device's languages, the names of each locale in use, layouts,
   composers, currency sets, popup mappings, presets, packs and pack items.
3. **Rust writes.** A change writes `localization__subtypes` through `settings::subtype::plan`
   and `codec::encode("subtypes")`. These are the same Rust functions behind Kotlin's
   `SettingsSubtypes.plan` and `SettingsCodecs.subtypes`, which SubtypeManager used.
   SubtypeManager sees the preference change and loads the list, as it always has, and picks
   the active subtype again.
4. **Kotlin deletes a pack.** *Delete* on a pack adds `deleteLanguagePack` with its id. Kotlin
   asks the extension manager, shows Compose's toast if that fails, and reports again.

Once asked, Kotlin reports again when the keyboard's resources change or a pack is imported or
deleted. Every code is declared once in `crates/wire` (`settings_ui::languages`), and Kotlin
is generated: `SettingsUiOp.LANGUAGES`, `SettingsUiLanguageEffect`, `SettingsUiLanguageKind`
and `SettingsUiLanguage`.

**The list Kotlin read.** Kotlin reads a stored locale through `FlorisLocale`, which can tidy
it (`en_US` becomes `en-US`). The report sends the raw text it read and the list as read. While
the preference still holds that raw text, Rust shows and writes the list as read, as
SubtypeManager did.

## What Kotlin does

* `SettingsLanguages` (87 counted lines) builds the report and deletes a pack. Its row
  functions are top-level, so JVM tests build reports with them.
* `RustSettingsView` makes it before the saved state comes back, so a reopened language screen
  finds it, and hands it both effects.
* `RendererAccessibility` adds each node's custom actions, and performs one by pressing its
  control.
* `SubtypeManager` lost `addSubtype`, `modifySubtypeWithSameId`, `removeSubtype` and
  `getSubtypePresetForLocale`, which only the Compose screens used, and `reorderSubtypes`,
  which nothing called. It
  still loads the list, picks the active subtype and switches subtypes.
* `SettingsIcons` gained add, delete, the two arrows, the dropdown arrow, search and import.
* `EnumDisplayEntries` lost the *Display language names in* entries. Only LocalizationScreen
  used them. Rust reads the same strings through `SettingsLanguageStringIds`.

## Taken from Gboard

Gboard's language screens were looked at on the emulator, and put back as they were.

1. **Suggested languages first.** Gboard's *Add keyboard* lists the device's languages under
   *Suggested languages*, then *All languages*. Select language now does the same.
2. **A picked language fills the form.** Gboard opens a new language with its layout already
   chosen. When adding, picking a language now fills every unset field from that language's
   preset. Compose set only the popup mapping.
3. **Delete without a question.** Gboard's edit mode deletes at once. Delete on a subtype now
   does too, where Compose asked first, but it can be undone.
4. **What Gboard lacks.** Gboard reorders by dragging an unlabelled handle, so TalkBack cannot
   reorder at all, and it has no undo. Here, *Move up*, *Move down* and the custom actions
   reorder, and the undo bar brings a deleted subtype back.

## Improvements

Each one is new against the Compose screens.

1. **Reordering.** Compose could not reorder subtypes. The order is the order the globe key
   cycles through.
2. **Reordering with TalkBack**, through custom actions, and focus stays on the moved item.
3. **Undo** after deleting from the list or from the editor.
4. **Delete is a button.** Compose deleted on a long press, which nothing on screen told the
   user about.
5. **Suggested languages** in Select language (from Gboard).
6. **A picked language fills the form** when adding (from Gboard).
7. **The search survives recreation.** Compose kept the draft but lost the search.
8. **An unknown layout or currency set shows its name.** Compose's summary showed "null".
9. **A suggested preset shows its layout's label** ("QWERTY"), where Compose showed the
   component id ("qwerty").
10. **TalkBack reads the editor's errors and the pack delete question.** Compose's dialogs
    were drawn but their text was not a node.

## Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| Deleting a subtype does not ask first; Compose asked on a long press. | Undo is safer than a question, and a long press is hidden. Gboard deletes at once too. |
| *Move up*, *Move down* and *Delete* buttons on each subtype. | Compose had no way to reorder. |
| The editor's *Delete* returns to the list with the undo bar. | Compose's deleted for good. |
| A picked language fills every unset field when adding. Editing still sets only the popup mapping. | An edit should not change what the user chose. |
| Select language puts the device's languages first. | From Gboard. With a search, the list is Compose's. |
| The add button is at the end of the list. Compose's was a floating button. | Rust has no floating button. The label and what it does are the same. |
| The field lists are Rust's list dialog. Compose had dropdown menus. | Rust has no dropdown menu. The choices, their order and the "‒ select ‒" value are the same. |
| The pack boxes are headed groups. | Rust has no outlined box, as in step 5. |
| Two strings are gone: the subtype delete confirmation's title and warning. | Nothing asks any more. |

## Data safety

**Subtypes are written exactly as before.** Rust writes with the same plans and codec Kotlin
called, and JVM tests prove the bytes against the original kotlinx codec
(`ReferenceSettingsCodecs`), which no code under test uses.

* `theStoredFormatIsTheOriginalKotlinxOutput` freezes the stored text of a subtype, as a
  literal, and checks both codecs write it.
* `movesDeletesEditsAndAddsWriteWhatSubtypeManagerWrote` drives the real native screens and
  strings over a stored list of three subtypes. Moving, deleting, undoing, editing one field,
  adding a preset that exists and adding a new one each write what the removed SubtypeManager
  function wrote for the same edit, byte for byte. A new subtype's id is the time it was
  added, as before. Nothing writes the active subtype; SubtypeManager still does that.
* `theEditorAcceptsAndRefusesWhatSubtypeEditorStateDid` puts 5,000 drafts into the native
  editor and presses *Save*. Each is accepted or refused as the frozen `SubtypeEditorState`
  copy decides, and an accepted one is written as that copy built it.
* The cargo tests check each tap's write against the stored fixtures, and that a list Kotlin
  tidied is written tidied.

**Nothing is written by looking.** Opening a screen, cancelling the editor, Back, and a
malformed report write nothing. A report that does not parse is refused whole, and the screens
keep the last good one.

## Accessibility

The screens use the same nodes as the other Rust settings screens: buttons, headings, edit
fields, radio choices, and status text. On top of that:

* each subtype's node offers *Move up*, *Move down* and *Delete* as custom actions, and only
  the ones that apply;
* each button says which subtype it moves (its summary);
* the undo message is a status node, and focus moves to *Undo*;
* the editor's error and the pack delete question are text nodes;
* the focus order walks each subtype, then its buttons, then *Add*.

## Tests

* **Rust** (29 new):
  * `settings_ui_languages.rs` (10): the screens are native and ask for the report once;
    titles and summaries; moving writes the list byte for byte; delete and undo; how long undo
    lasts; custom actions; focus order; the empty list and *Add*; the list Kotlin tidied; a
    malformed report.
  * `settings_ui_languages_editor.rs` (11): adding a preset; suggested presets and *Show all*;
    unset fields; a subtype that exists; editing one field; cancel and Back; a picked
    language filling the form; editing the language; *Delete* and undo; the checks; focus order.
  * `settings_ui_languages_picker.rs` (8): sorting by shown name with suggestions first;
    the search; nothing found; the pack boxes; delete asks first; *Import* and a pack's page;
    the saved state, with bad state refused.
* **JVM** (4 new), in `SettingsLanguagesMatchTest`: the three in [Data safety](#data-safety),
  and each item's custom actions with the real strings.
* **Updated:** the three route-list tests expect the five new native routes.
  `SettingsDerivationMatchTest` lost its editor half, now in `SettingsLanguagesMatchTest`, and
  calls the choice index operation directly. `SettingsResourcesTest` reads icon names from a
  list constant too.

## Mutations

`scripts/settings-ui-mutations.py` has 15 new ones for these screens; each must make a test
fail, and the code is put back after each one.

**All 15 were caught**, each by a fresh assertion, and the restored source passed
(`restored_exit` 0).

| Mutation | What it breaks | Caught by |
| --- | --- | --- |
| `move-up-goes-down` | *Move up* moves the item down | `SettingsLanguagesMatchTest` |
| `undo-appends` | Undo puts the subtype at the end, not back in its place | `SettingsLanguagesMatchTest` |
| `editor-checks-characters-only` | Save checks only the characters layout is set | `SettingsLanguagesMatchTest` |
| `editor-adds-as-modify` | A new subtype is saved as an edit | `SettingsLanguagesMatchTest` (2) |
| `exists-closes-silently` | Adding a subtype that exists closes without the message | `SettingsLanguagesMatchTest` |
| `custom-actions-include-off` | A disabled button is still a custom action | `SettingsLanguagesMatchTest` |
| `language-route-dropped` | The pack manager's route is not native | the route-list tests (4) |
| `language-message-not-read` | The editor's error is not a text node | `SettingsLanguagesMatchTest` (2) |
| `language-string-id-missing` | *Move up* has no string | `SettingsResourcesTest` |
| `report-names-swapped` | The report swaps native and English names | `SettingsLanguagesMatchTest` |
| `search-tag-anywhere` | A tag matches anywhere, not at its start | `settings_ui_languages_picker` |
| `native-names-ignored` | *Display language names in* is ignored | `settings_ui_languages` |
| `pack-deleted-without-asking` | A pack is deleted without the question | `settings_ui_languages_picker` |
| `report-asked-every-screen` | The report is asked for on every screen | `settings_ui_languages` |
| `normalised-list-ignored` | A list Kotlin tidied is written untidied | `settings_ui_languages` |

## Emulator check

On `emulator-5558`, with the final build installed by `scripts/install-dev.sh`. The emulator
started with no subtypes. Each step read the stored preferences.

* **Empty list.** Compose's warning and *Add subtype* show.
* **Add.** *Add subtype* opens the presets. English (United States) fills the form, and Save
  writes one subtype in the fixture's format. SubtypeManager makes it the active subtype.
* **Add a second language.** German (Germany) from *Show all* fills every field: popup
  mapping `de`, QWERTZ, Latin. Save stores it after English. The English entry is byte for byte
  what it was.
* **Switch with the globe.** The globe key next to space shows only when the utility key is
  set to *Switch language* or *Dynamic*. It is *Switch to emojis* by default, so the check set
  *Dynamic* and put it back at the end. The globe switched to German, and the keyboard showed
  QWERTZ with € on the d key.
* **Edit.** German's characters layout changed to AZERTY. Only that one value changed, under
  the same id.
* **Reorder.** *Move up* on German put it first. Both entries stayed byte for byte the same.
* **Delete and undo.** Deleting English showed "Deleted English (United States)" and *Undo*.
  Undo put it back in its place. The stored list was byte for byte what it was before.
* **The new layout types.** With German active the keyboard showed AZERTY. The globe went back
  to English and QWERTY.
* **Remove all.** Deleting both brought the warning back. The list was stored as `[]`, and the
  active id went back to -1, as at the start.
* **Select language** shows the search field, *Suggested languages* (English, United States)
  and *All languages*.
* **Language packs** shows *Import* and the built-in pack with its two items, and no *Delete*.
* **Put back.** Every preference matched the start except the build's own last-used version,
  and the subtype list, now stored as `[]` where it had been absent. Both mean no subtypes.

**Typing could not be checked on the device.** No Rust settings field took text on this
emulator: not the search field, and not the home screen's preview field, which this step did
not change. Our keyboard's keys and agent-device's key events both typed nothing. The same
keyboard typed into the system Settings search. Earlier stages typed into Rust fields only
through the instrumented device tests, which this check could not run. The search is covered by
the cargo tests and the `search-tag-anywhere` mutation. See [Owed](#owed).

**Gboard was shown once by accident.** The Smartbar's *Switch language* action, with the
utility key at its default, moves to the next keyboard app, not the next subtype. That is the
keyboard's existing rule. It opened Gboard. Nothing was typed into it, and our keyboard was
selected again at once.

## Deleted

Counted as non-blank lines against `0775f6a1d`.

| Area | Deleted | Added |
| --- | ---: | ---: |
| Kotlin production | 1,239 | 177 |
| Rust production (`render`, `wire`, `awnative`) | 40 | 2,734 |
| Rust tests | 1 | 1,060 |
| Kotlin tests | 37 | 217 |
| String resources | 2 | 6 |
| Mutation script | 3 | 36 |

The Kotlin deletions are the five screens (1,078), FlorisDropdownLikeButton (68),
FlorisWarningCard (25), SubtypeManager's functions (24), the language names display entry (17),
SettingsSelection (6), and Routes and the string ids (21).

## Owed

* **No phone check.** Everything above ran on the emulator only.
* **Typing into a Rust settings field on a device.** On `emulator-5558` no Rust field took
  text, the home screen's preview field included. The fault is older than this step. The
  search field needs a device check once typing works.
* **`SettingsRules.editorValid` is now unused.** It is in the generated-bindings area another
  branch is changing, so it was left for that branch.
* **The choice index operation has no production caller.** Only `SettingsDerivationMatchTest`
  calls it. It can go with `editorValid`.
* **`SettingsLanguages` runs only on a device.** It needs the activity and the keyboard's
  managers. Its row functions run in JVM tests, and the emulator check covers the rest.
* **Language pack import and a pack's page are Compose** until step 10.
