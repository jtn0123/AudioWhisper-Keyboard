# Stage 4: extensions

This page covers step 10 of [the Compose removal plan](compose-removal.md): the extension
screens in `FB/app/ext` move from Compose to the Rust settings renderer. The owner's rule is
that they are "ported to Rust and improved, not cut, and the port names each improvement it
makes" ([decision](decisions.md#everything-moves-to-rust-2026-11-21)).

Step 10 has three parts. **Parts 1 and 2 are done.** Part 1 is the extension home, the three
extension lists, an extension's page, its not-found page and *Search for updates*, with their
chips and boxes. Part 2 is the importer and *Export*. Part 3 is the extension editor, its files
and its component view; [What remains](#what-remains) lists it.

| Route | Drawn by | Notes |
| --- | --- | --- |
| `ext` (ExtensionHomeScreen) | Rust | New in part 1. |
| `ext/list/EXT_THEME`, `ext/list/EXT_KEYBOARD`, `ext/list/EXT_LANGUAGEPACK` (each with `?showUpdate=`) | Rust | New in part 1. The Theme screen's addons box opens the theme list with `showUpdate=true`. |
| `ext/view/<id>` (ExtensionViewScreen, or ExtensionNotFoundScreen) | Rust | New in part 1. |
| `ext/check-updates` (CheckUpdatesScreen) | Rust | New in part 1. |
| `ext/import/<type>` (ExtensionImportScreen), with `?files=`, `?uuid=` or `?document=` | Rust | New in part 2. A file opened in or shared with the app starts it too. |
| `ext/export/<id>` (ExtensionExportScreen) | — | Deleted in part 2. *Export* asks for its document from the page. |
| `ext/edit/<id>`, `ext/edit/null?create=…` | Compose | Part 3. *Edit* and *Create extension* hand off to it. |

The Compose routes `Ext.Home`, `Ext.List`, `Ext.View` and `Ext.CheckUpdates` stay as empty
entries that hand back to Rust, as in steps 2 and 4 to 9, so a deep link or a Compose screen
that still navigates to one reaches the Rust screen. `ExtensionListScreenType` moved into
`Routes.Ext`, since the route still names its values.

## The mutation script split

Before step 10 added cases, `scripts/settings-ui-mutations.py` was 1,016 counted lines
(non-blank, non-comment), over the 500-line file limit. It is now a runner of 84 counted lines,
and the cases live in `scripts/settings_ui_mutation_cases/`, one module per screen:

| Module | Cases | Counted lines |
| --- | --- | --- |
| `general.py` (the renderer, groups 1 and 2, Jetpref writes, third-party licences) | 47 | 100 |
| `other_theme.py` | 22 | 45 |
| `languages.py` | 15 | 34 |
| `words.py` | 26 | 55 |
| `project_licence.py` | 36 | 88 |
| `themes.py` (the theme manager) | 21 | 45 |
| `setup.py` | 26 | 59 |
| `theme_editor.py` (part 2) | 38 | 88 |
| `property_editor.py` (part 3) | 46 | 101 |
| `property_values.py` (part 3: colours, swatches, icons, contrast) | 57 | 116 |
| `theme_codes.py` (part 3: the code editor, and the fallback crash fix) | 49 | 94 |

`paths.py` (17) holds the paths the cases name and `__init__.py` (7) joins the modules in their old
order. Each module keeps its own Rust targets (`RUST`) beside its cases. The runner's command line
and output are unchanged: `python3 -I scripts/settings-ui-mutations.py <output> [case …]`. `-I`
leaves the script's folder off the import path, so the runner puts it back itself, and it writes no
`__pycache__`.

**Nothing was lost.** A dump of every case's name, file, original text and mutated text in run
order, with the Rust target map and the suites, was taken from the old script and from the split
one. The two dumps are byte-identical (383 cases, 255 Rust targets; SHA-256
`71b977d8…3c55`). The dump script was a scratch file and is not committed. The split was made by
slicing the old file's source lines, so each case's text is the same characters, comments
included.

**Every module still runs.** One case from each module went through the split runner, in a clean
worktree at the split commit:

| Module | Case | Runs | Result |
| --- | --- | --- | --- |
| `general.py` | `toggle` | JVM | caught (4 assertions) |
| `other_theme.py` | `time-drag-rounds-down` | Rust | caught |
| `languages.py` | `search-tag-anywhere` | Rust | caught |
| `words.py` | `word-list-reorders` | Rust | caught |
| `project_licence.py` | `licence-asked-every-frame` | Rust | caught |
| `themes.py` | `theme-sort-by-code-point` | JVM | caught |
| `setup.py` | `setup-unasked-inverted` | Rust | caught (8 panics) |
| `theme_editor.py` | `theme-editor-order-reversed` | Rust | caught (57 panics) |
| `property_editor.py` | `theme-editor-corners-key-unlisted` | Rust | caught |
| `property_values.py` | `colour-alpha-hundredths-truncated` | Rust | caught |
| `theme_codes.py` | `theme-code-string-id-dropped` | JVM | caught |

The restored run passed (exit 0), and every source's SHA-256 after the run matched the commit's.

## Part 1: the extension home, lists and page (done)

### What the screens show

**The home** (`ext`): ImportExtensionBox (where extensions come from, *Visit Addons Store* and
*Import*), UpdateBox (*Search for updates* and its note), then one row per list: themes,
keyboards and language packs, as Compose had them.

**A list** (`ext/list/<type>`): the boxes above when its route says `showUpdate=true`, then one
outlined box per installed extension of its type, in the index's order. A box is titled with
the extension's name in its border, then its id, its description in `bodySmall`, *View details*
and, for an extension made or imported here, *Edit*. The theme list has *Create extension* in a
box above the extensions.

**An extension's page** (`ext/view/<id>`): its description in `bodyLarge`, then rows of a label
and its value with a divider between them: Maintainers (a chip each), ID, Version, Keywords (a
chip each), Homepage and Issue tracker (links), License. Then *Delete* (for an extension made or
imported here) and *Export*. Under them, a theme or language pack extension lists what it
bundles, each in ExtensionComponentView's box, or says it bundles nothing. With no extension of
that id, the page is titled *Extension not found* and says so.

**Search for updates** (`ext/check-updates`): UpdateBox alone. Its button opens the store's
`check-updates` page with every installed extension's id and version.

### Data path

```
ExtensionManager.keyboardExtensions / themes / languagePacks (Kotlin LiveData)
  └─ SettingsExtensions.rows() ─ EXTENSIONS op, rows of wire::settings_ui::extensions ─▶ App::set_extensions
                                                                                           │ extensions::parse
App::ext_part (boxes.rs, page.rs, chips.rs) ◀────────────────────────────────────────────────┘
  Delete ─▶ the delete question ─▶ ["extensionDelete", id] ─▶ SettingsExtensions.delete
                                   EXTENSION_DONE (work, id, done) ◀─┘ ─▶ App::extension_done: the page closes
  Store, Search for updates, a link, an address ─▶ ["url", …] (the existing effect)
  Import, Export, Edit, Create ─▶ navigate ─▶ ["compose", route, native routes] (parts 2 and 3)
```

* **Rust asks, Kotlin answers.** Opening any extension screen emits `["extensions"]` once.
  `SettingsExtensions` replies with the whole index as one `EXTENSIONS` report, in
  `combinedExtensionList()` order (keyboards, themes, language packs): an `extension` row (id,
  serial type, title, version, licence, whether it can be deleted), then `description`,
  `maintainer` (each followed by its `email` and `url`), `keyword`, `homepage`,
  `issueTracker`, and `component` rows (each followed by its `author` rows). A null value
  sends no row. After the first ask, Kotlin re-sends the report whenever any index changes, so
  an extension imported, made or deleted anywhere shows at once.
* **Rust decides everything shown**: which extensions a screen lists, whether it shows the
  store and update boxes (the route's `showUpdate`), the update link, which rows the page
  leaves out (an empty keyword list, a blank link), each component's text, the not-found
  message, the delete question and what each control does.
* **Deleting** is the one action Kotlin takes. The page's *Delete* asks first (the settings
  dialog, `Lang::DeleteExtension`, naming the extension), then emits `extensionDelete`. Kotlin
  deletes through the extension manager, shows a long toast on failure as Compose did, and
  answers `EXTENSION_DONE`. On success the page closes, as ExtensionViewScreen popped itself.
* **Saved state.** The settings state saves the stack, scroll and open dialog as before; a
  restored extension screen asks for the index again, since Kotlin's side is new.

### What Kotlin does

Two files are new. Both must be Kotlin because only Kotlin can reach what they read:

* **`settingsui/SettingsExtensions.kt`** (74 counted lines): reads the three extension indexes
  (`LiveData` owned by `ExtensionManager`), each extension's meta and components (Kotlin
  objects), `canDelete`, and a link's host through `FlorisRef`; deletes an extension through the
  extension manager and shows the error toast. It builds no screen: `extensionRows` only
  flattens what an extension holds into wire rows.
* **`settingsui/SettingsExtensionStringIds.kt`** (25): the string resource ids of the screens'
  labels, in their own map so `SettingsStringIds` stays under the file limit.

`RustSettingsView` grows by 3 (272 to 275): it creates the adapter before the saved state comes
back and routes the two effects to it. `SettingsIcons` grows by 4 (67 to 71) for the
keyboard, file download, share and mail icons. `strings.xml` gains *Email* and *Website*, which
the maintainer chip and its dialog say.

Kotlin that went: the seven Compose files below (693 counted lines), `generateUpdateUrl`
(`Extension.kt`, 99 to 75), `getUpdateJsonPair` (`ExtensionMeta.kt`, 22 to 18), the four
routes' screen calls in `Routes.kt` (241 to 227), and the two `BuildConfig` fields only the
update link read (`FLADDONS_API_VERSION`, `FLADDONS_STORE_URL`). The store's address is now
Rust's (`extensions::STORE`).

### Improvements

Each was chosen for a person browsing, checking or removing their extensions. Gboard has no
extension pages, so the reference was what TalkBack and a phone-width screen need.

| # | Improvement | Why | Test |
| --- | --- | --- | --- |
| 1 | A row of chips wraps onto more lines. Compose scrolled it sideways. | On a phone, Compose showed the first two or three keywords and hid the rest with no sign there were more. | `many_keywords_wrap_and_stay_on_screen`, `chips_flow_into_lines_that_end_at_the_edge` |
| 2 | Keywords are in full ink and read as text. Compose drew them as disabled chips at 38 % ink. | TalkBack called each one a dimmed button, and they were hard to read. | `keywords_are_text_in_full_ink` |
| 3 | A maintainer chip tells TalkBack what its dialog offers (*Email, Website*); the dialog lists each address with its icon and what it is. | Compose's chip read as a bare name, and its dialog's address chips had no description. | `a_maintainer_chip_says_what_it_offers_and_lists_its_addresses`; JVM `aMaintainersDialogOffersWhatExtensionMaintainerChipsDialogOffered` |
| 4 | A maintainer with no address reads as text, and focus passes it by. | Compose disabled the chip, which TalkBack read as a dimmed button that could never work. | `a_maintainer_with_no_address_reads_as_text`; the role check in JVM `thePageShowsWhatExtensionViewScreenShowed` |
| 5 | The homepage and issue tracker are links that name their host, then where they go. | FlorisHyperlinkText was text that TalkBack read as the host alone, so a TalkBack user could not tell where it led. | `a_link_names_its_host_and_reads_where_it_goes`; JVM `thePageShowsWhatExtensionViewScreenShowed` |
| 6 | The ID, Version and License rows each read as one: the label, then the value. | Compose's label and value were two separate nodes. | `the_id_version_and_license_rows_read_as_one` |
| 7 | The delete question survives a rotation. | FlorisConfirmDeleteDialog was held in a plain `remember`, so rotating closed it. | `the_page_and_its_delete_question_survive_recreation`; JVM `theDeleteQuestionSurvivesRecreationAndDeletesWhatItNamed` |
| 8 | A built-in extension has no *Edit*. Compose showed it greyed out. | A button that can never work is noise, and TalkBack read it on every built-in box. | `a_built_in_extension_has_no_edit_button`; JVM `eachListShowsItsIndexWithTheBoxesItsRouteAsks` |
| 9 | A page opened before the index loads waits for it, then fills in; it updates when the index changes. | ExtensionViewScreen read the extension once, so a page opened early said the extension was not found. | `a_page_opened_before_the_index_comes_waits_for_it`, `kotlin_is_asked_once_and_every_report_redraws` |
| 10 | A box's name and id open the extension's page, as in the theme manager. | They were dead text in Compose; the whole box is now a way in. | `the_name_id_and_view_details_open_the_page_and_edit_opens_the_editor` |
| 11 | **Fix:** the update link always separates its pairs. | Compose wrote a comma after each extension not `equals` to the last. A keyboard extension is a data class, so one equal to the last lost its comma and the link's JSON broke. Rare, but real. | `the_update_link_separates_every_pair_even_when_an_earlier_one_equals_the_last`; JVM `twoAlikeKeyboardExtensionsStillGiveWellFormedJson` |
| 12 | **Fix:** an outlined box's title reads first, then what is in it. This applies to every outlined box: the extension lists and components, language packs, theme boxes and the theme editor's boxes. | The title is painted last so it sits over the border, and it was recorded last too, so TalkBack read a box's id and contents before its name. Compose read in on-screen order. The device check found it. | `a_box_reads_its_name_before_what_is_in_it` |

### Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| *Create extension* is an outlined box above the theme list's extensions, not a floating button. | It is where the theme manager has it (step 9), and a floating button covered the last box's buttons. |
| An empty description takes no space, in a box and on the page. Compose left a blank line. | Nothing to read; the blank line only pushed the content down. |
| The maintainer dialog lists the addresses as list items with an icon, the address and what it is, and has *Cancel*. Compose had chips and no button. | Improvement 3, and every settings dialog has a way out. |
| A component's box title and name are text, not buttons. | Nothing in ExtensionComponentView was tappable in Compose either; the Rust box title is a button elsewhere. |
| Keyboard and D-pad focus moves through each box's name, id, *View details* and *Edit*, then on the page through the maintainers that offer something, the links, *Delete* and *Export*. | Every Rust screen has a focus order; Compose's was whatever the layout gave. |

### Tests

* **Rust** (`lib/rust/crates/render/tests/settings_ui_extensions/`, 38 tests in `lists.rs`,
  `page.rs` and `saved.rs`): the home's boxes and order; the store and *Import*; each list
  showing its own type in index order; the boxes only when the route asks; the update link,
  exactly, for each screen, and with an earlier extension equal to the last; a box's title, id,
  description and buttons; no blank line for an empty description; no *Edit* for built-ins; each
  tap's destination; *Create extension*; the focus order; asking once and redrawing on every
  report; malformed reports refused; the titles; the page's rows in order and level with their
  labels; ID, Version and License read as one; rows with nothing to show left out (blank links
  included); the link's node; `chips::flow`; keywords wrapping at 720 px and drawn whole in full
  ink; the maintainer chip, its dialog and a maintainer with no address; the delete question,
  its Cancel and Delete and the page closing only when its own extension went; a page opened
  before the index; an unknown id; each component's text and *none found* in italics; *Export*;
  the page's focus order; each box's title read before its contents; every control labelled and on screen; and recreation with the page,
  its delete question, a maintainer's dialog and a list's query. Five existing files
  (`settings_ui_screens.rs`, `settings_ui_theme.rs`, `settings_ui_theme_manager.rs`,
  `settings_ui_setup.rs`, `settings_ui_languages_picker.rs`) now expect the extension routes
  native.
* **JVM** (`SettingsExtensionsMatchTest`, 7 tests), through the real native library with the
  report built by `SettingsExtensions`' own `extensionRows`, against test-only copies of what the
  deleted Compose screens decided (`ReferenceExtensionScreens`): the update link for every screen
  over 30 random indexes; two alike keyboard extensions; each list's extensions and boxes for
  every route and query; the page's rows, links, chips and buttons over 40 random indexes (empty,
  blank and missing values, accents, an emoji); each maintainer's dialog; each component's text;
  and the delete question across recreation. `SettingsGroupOneMatchTest`,
  `SettingsGroupTwoMatchTest`, `SettingsUiMatchTest` and `SettingsOtherThemeMatchTest` now count
  the extension routes as native, and `SettingsResourcesTest` checks the new strings and icons.
* **Gate** (rebased on main b10bee9b1): 1,000 JVM tests in 244 files (main had 993 in 243) and
  1,363 cargo tests (main 1,325), none failing or skipped; 391 native declarations, none hand
  written; the ledger matches 422 files (main 427); lint shows only the allowed issues.

### Mutations

`scripts/settings_ui_mutation_cases/extensions.py` has 45 new cases: 15 must fail a JVM test
(`SettingsExtensionsMatchTest` or `SettingsResourcesTest`), 30 the Rust tests in
`settings_ui_extensions`. The runner now also runs `SettingsExtensionsMatchTest`. **All 45 were
caught**, and the restored code passed. They ran in five goes, kept in
`build/mutations-extensions/` (not committed):

* `run2`: the first four caught; then `ext-blank-link-shown` **survived**. A blank link's value
  has no width, so its node vanished either way, but its *Homepage* label still drew. Both tests
  now check the label too (`rows_with_nothing_to_show_are_left_out` and the JVM page test).
* `run3`: that case and the next caught; then the Gradle daemon died mid-build with no test run,
  which is not a result.
* `run4`: 21 caught; then `ext-box-description-blank` **survived**: no test gave a box an empty
  description, only none. `a_box_without_a_description_takes_no_blank_line` now does.
* `run5`: the last 16 caught.
* `run6`: after the device check's fix (improvement 12), its two new cases and
  `ext-component-name-button`, whose code moved, all caught.

Before these runs, a first attempt was stopped: three settings route-list tests
(`SettingsGroupTwoMatchTest`, `SettingsUiMatchTest`, `SettingsOtherThemeMatchTest`) still
expected the extension routes in Compose, so every JVM case would have looked caught. They were
fixed first, and the whole app test task passed before the runs above.

| Mutation | What it breaks | Caught by |
| --- | --- | --- |
| ext-update-pairs-unjoined | The update link's pairs run together | JVM match test |
| ext-list-every-type | A list shows every type of extension | JVM match test |
| ext-update-any-query | Any query shows the store and update boxes, `showUpdate=false` included | JVM match test |
| ext-updates-screen-no-box | *Search for updates* has no button | JVM match test |
| ext-blank-link-shown | A blank homepage or issue tracker gets a row | JVM match test |
| ext-page-empty-description | An empty description takes a line on the page | JVM match test |
| ext-quiet-maintainer-button | A maintainer with no address is a button | JVM match test |
| ext-addresses-website-first | The dialog lists the website before the email | JVM match test |
| ext-night-said-day | Every theme component says `isNightTheme = false` | JVM match test |
| ext-delete-names-id | The delete question names the id, not the title | JVM match test |
| ext-kotlin-stylesheet-unsent | Kotlin's report drops a theme's stylesheet path | JVM match test |
| ext-kotlin-deletable-ignored | Kotlin's report calls every extension deletable | JVM match test |
| ext-kotlin-email-as-url | Kotlin's report sends a maintainer's email as their website | JVM match test |
| ext-icon-unmapped | The mail icon has no Android icon | `SettingsResourcesTest` |
| ext-string-id-dropped | *Website* has no string id, so it shows its key | `SettingsResourcesTest` |
| ext-asked-every-open | The index asked for again on every screen | Rust test |
| ext-restore-not-asked | A restored extension screen never asks for the index | Rust test |
| ext-title-before-index | A page opened before the index is titled *Extension not found* | Rust test |
| ext-page-not-waiting | A page opened before the index says no extension has the id | Rust test |
| ext-page-untitled | The page is titled *Extension not found* even when the extension is there | Rust test |
| ext-done-closes-on-failure | A failed delete closes the page | Rust test |
| ext-unknown-kind-accepted | A report row of unknown kind is accepted | Rust test |
| ext-owner-unchecked | A row under the wrong extension is accepted | Rust test |
| ext-store-imports | *Visit Addons Store* opens *Import* | Rust test |
| ext-create-imports | *Create extension* opens *Import* | Rust test |
| ext-export-edits | *Export* opens the editor | Rust test |
| ext-edit-for-built-ins | Built-in extensions get an *Edit* | Rust test |
| ext-box-description-blank | An empty description takes a line in a box | Rust test |
| ext-focus-skips-id | Keyboard focus skips a box's id | Rust test |
| ext-delete-for-built-ins | A built-in extension's page offers *Delete* | Rust test |
| ext-not-found-unnamed | The not-found message leaves out the id | Rust test |
| ext-meta-row-unvoiced | TalkBack hears the ID, Version and License labels without their values | Rust test |
| ext-link-reads-host | TalkBack hears a link's host twice, never where it goes | Rust test |
| ext-none-found-upright | *No bundled components* is not in italics | Rust test |
| ext-component-name-button | A component's full name is a button that does nothing | Rust test |
| ext-component-title-button | A component's title is a button that does nothing | Rust test |
| ext-box-title-read-last | TalkBack reads a box's title after its contents | Rust test |
| ext-quiet-maintainer-focused | Focus stops on a maintainer with no address | Rust test |
| ext-chips-one-line | Chips never wrap and run off the screen | Rust test |
| ext-keywords-faded | Keywords drawn in faded ink | Rust test |
| ext-chip-always-button | Keywords and quiet maintainers are buttons | Rust test |
| ext-maintainer-offers-unsaid | A maintainer chip does not say what it offers | Rust test |
| ext-chip-label-cut | A chip's label is measured the old way and cut short | Rust test |
| ext-address-keeps-mailto | The dialog shows `mailto:` before the email | Rust test |
| ext-address-kinds-swapped | The dialog calls an email a website | Rust test |

### Device check

This ran on `emulator-5556` (Android 16, 1080×2400) with the gate's build installed by
`scripts/install-dev.sh`. TalkBack's labels were read from the accessibility tree. The
screenshots are in `build/extensions/` and are not committed.

1. `01-home.png`: the home, with the store and update boxes and the three lists.
2. `02-theme-list.png`: *Create extension* and the two built-in theme boxes, with no *Edit*.
3. `03-page.png`: *FlorisBoard default themes*. It shows the description, then two maintainer
   chips with their mail icons. ID, Version and License follow, then *Export* (no *Delete*) and
   the bundled themes. The tree reads the chips as buttons named by the maintainer, and the
   component text as text.
4. `04-maintainer-dialog.png`: patrickgold's dialog. It lists the address with its icon and
   *Email*, without `mailto:`.
5. `05-dialog-rotated.png`: the dialog is still open after rotating to landscape. *Cancel*
   closed it after rotating back.
6. Up on the page went to the theme list, and system Back from there went to the home.
7. `06-keyboard-list.png` and `07-layouts-page.png`: the keyboard list. A keyboard extension's
   page has no components section; nor did Compose's.
8. `08-pack-page.png`: the language pack's page, with its two bundled packs.
9. `09-not-found.png`: `ui://florisboard/ext/view/nope` says no extension with ID `nope` could
   be found.
10. `10-check-updates.png`: *Check for Updates*, with its note and button.
11. `11-theme-addons.png`: `ext/list/EXT_THEME?showUpdate=true` shows the store, import and
    update boxes above the list.
12. `12-user-box.png` and `13-user-page.png`: an extension made with *Create extension* and
    saved at once has *Edit* in its box. Its page shows the maintainer *Local* as text, leaves
    out the empty description, and offers *Delete*. It also says it has no bundled components.
13. `14-delete-rotated.png`: the delete question named *My themes* and survived a rotation.
    *Delete* removed the extension and closed the page, back to the list.
14. The tree showed every outlined box read its id and contents before its name, which became
    improvement 12. After reinstalling, `15-page-title-first.png`: the tree lists each
    component's name, then its id, then its text, and the keyboard list's boxes start with their
    name.

agent-device has no rotation command, so rotating went through `settings put system
user_rotation`, with auto-rotate left off as it was. Afterwards, the test extension was deleted,
the rotation was back at 0 with auto-rotate off, our IME was restored, and the volume was at 0.
The selected theme was never changed. The agent-device session was closed.

### Deleted

`ExtensionHomeScreen` (48 counted lines), `ExtensionListScreen` (161), `ExtensionViewScreen`
(264), `CheckUpdatesScreen` (17), `AddonBox` (87), `ExtensionKeywordChip` (18) and
`ExtensionMaintainerChip` (98): 693 counted lines. `ExtensionNotFoundScreen` stays: Compose's
editor screen still uses it (part 2 deleted the export screen, its other user).

## Part 2: import and export (done)

### What the screens show

**The importer** (`ext/import/<type>`, one each for any extension, keyboards, themes and
language packs, titled as Compose titled them): *Select files* as an outlined button centred
with 16 dp above and below, then one box per file read, as FileInfoView drew it, or *No files
selected* in italics. A file's box is titled with its name over its media type
(*application/unknown* when Android names none). In it are the size, id and version in grey,
the title, the description in italics, who made it and its components. A file the importer
skips ends with a thin error bar, *File can not be imported. Reason:* and why, in italics, in
the error colour. The bottom bar has *Cancel* and *Import*.

**Export** is the page's button. It opens the system's *Create document* picker for a `.flex`
named after the extension, writes the extension into the document picked, and says whether it
worked. It has no screen of its own.

A file opened in the app or shared with it (the manifest's two *Import Extension* filters)
starts the importer for any extension on it, without *Select files*, as Compose did.

### Data path

```
Select files ─▶ ["pick", "openMany", "*/*"] ─▶ SettingsFiles (GetMultipleContents) ─▶ PICKED (handles)
  └─▶ work extRead (handles) ─▶ ExtensionFileWork.read: CacheManager.readFromUriIntoCache, off the main thread
        DONE extRead (OK, the workspace's id) ◀─┘ ─▶ the route gains files=<id> (uuid=<id> for a shared file)
  └─▶ ["extensionFiles", id] ─▶ SettingsExtensions: importRows ─▶ EXTENSION_FILES ─▶ App::set_import_files
Import ─▶ work extImport (id, the indexes Rust chose) ─▶ ExtensionManager.import each ─▶ DONE ─▶ toast, close
Cancel, Back, Up, other files ─▶ ["extensionFilesClose", id] ─▶ the workspace is deleted
Export ─▶ ["pick", "create", FLEX_MIME, "<id>.flex"] ─▶ PICKED ─▶ work extExport ─▶ ExtensionManager.export
A shared or opened file ─▶ SettingsEntry.route ─▶ start or navigate ext/import/EXT_ANY?document=<handle> ─▶ extRead
```

* **Rust decides what each file is.** Kotlin sends, for each file read, its name, media type,
  size as Android words it, whether its name or type says it is an extension archive
  (`FileRegistry.matchesFileFilter`, unchanged), and the archive read from it, with whether an
  extension with that id ships with the app. Rust decides whether it can be imported and why
  not, in ExtensionImportScreen's order: not an archive, no manifest read from it, replacing a
  built-in extension, then (new) of a type this importer does not take. Rust picks the files
  Import imports and whether Import is on.
* **The workspace is in the route.** `files=<id>` for files picked here, `uuid=<id>` for a
  shared file (which hides *Select files*, as Compose's `initUuid` did), `document=<handle>`
  for a shared file not read yet. So a rotation or recreation keeps the selection, and a
  workspace Android has cleared is reported closed.
* **The file work runs off the main thread.** Compose read, imported and exported on the main
  thread. The jobs go through step 5's file work queue, one at a time, and their reports outlive
  the activity.
* **Failures are told in plain words.** Compose put the exception's message in the toast and
  showed a failed read's stack trace on screen; either can hold a file's address. Rust says
  what happened: the files could not be read, are gone, or the file could not be written.

### What Kotlin does

One file is new. It must be Kotlin because only Kotlin can reach what it uses:

* **`settingsui/ExtensionFileWork.kt`** (84 counted lines): the three jobs, on the IO
  dispatcher, through `ContentResolver`, `CacheManager`, `ExtensionManager` and
  `DocumentsContract`; and `importRows`, which flattens a workspace's `FileInfo` list into wire
  rows with `Formatter.formatShortFileSize`. It decides nothing that is shown.

Grown: `SettingsExtensions` 74 to 78 (the two effects), `SettingsExtensionStringIds` 25 to 45
(the importer's and Export's strings), `SettingsFiles` 109 to 110 (the jobs sent to their own
work), `RustSettingsView` 275 to 276 (the effects routed), `SettingsEntry` 7 to 17 (a shared or
opened file's route) and `CacheManager` 179 to 183 (the read waits for its workspace; see fix
9). Shrunk: `FlorisAppActivity` 254 to 233 (the fallback no longer reads files), `Routes.kt`
227 to 208, `ExtensionPolicy` 30 to 27 and `CacheManager` 182 to 179 (the one-file read and
`FileInfo.skipReason`). `strings_settings_renderer.xml` gains four plain-words messages.

`ExtensionOp` `ACCEPTS` (7) and `SKIP_REASON` (8), `ExtensionSkipReason` and their Kotlin
wrappers went: only ExtensionImportScreen called them, and Rust now decides skips in the
importer itself.

### Improvements

| # | Improvement | Why | Test |
| --- | --- | --- | --- |
| 1 | A file holding a type of extension the importer does not take is shown skipped, with why, and never imported; *Import* stays off when nothing else can be imported. | ExtensionImportScreen listed it as importable, turned *Import* on, left it out when tapped and still said *Imported*. | `a_file_of_another_type_is_shown_skipped_and_never_imported`; JVM `aFileOfAnotherTypeIsShownSkippedAndLeavesImportOff` |
| 2 | The selection survives a rotation or recreation. | Compose held the workspace in a plain `remember`: turning the phone lost the files and left them in the cache. | `the_selection_survives_recreation` |
| 3 | Leaving the importer any way, or choosing other files, deletes what was read. | Compose deleted it only on *Cancel*; Back and Up left it in the cache. | `leaving_the_importer_any_way_closes_its_workspace` |
| 4 | Failures are told in plain words. | Compose's toast held the exception's message and a failed read showed its stack trace, which can hold a file's address. | `import_failures_are_told_in_plain_words` |
| 5 | *Export* goes straight to the picker, with the file named after the extension. | ExtensionExportScreen was an empty screen that launched the picker and closed itself. | `export_asks_for_a_flex_named_after_the_extension_and_opens_no_screen` |
| 6 | Reading, importing and exporting run off the main thread; while files are read, the importer says nothing rather than *No files selected*. | Compose did all three on the main thread, so a large archive froze the screen. | `the_importer_waits_for_the_read_without_freezing` |
| 7 | When Android has cleared the selection (the app was ended in the background), the importer offers *Select files* again. | Compose showed *No files selected* with *Select files* hidden for a shared file: a dead end. | `a_workspace_android_cleared_offers_select_files_again`; JVM `aClosedWorkspaceOffersSelectFilesAgain` |
| 8 | An export that fails deletes the half-written document. | Compose left a broken archive where the user asked for it. | JVM `aFailedExportDeletesWhatItWrote` |
| 9 | **Fix:** the files read can be found as soon as the read reports, and a read that fails deletes what it copied. | `readFromUriIntoCache` added the workspace in a coroutine it did not wait for, so the lookup that follows at once could miss it; a failed read left its copies in the cache. | No JVM test: `CacheManager` needs an Android `Context` and the JVM tests have no Robolectric. The device check imports through it (owed E37). |

### Differences from Compose

| Difference | Why |
| --- | --- |
| A file opened in or shared with the app opens the importer in the settings activity. Compose started a second activity, the Compose fallback, read the file there on the main thread, then opened its importer. | The importer is native now, so there is nothing to hand off. |
| A file Android could not read is reported by the read job, and the importer says the files could not be read. Compose showed *An unexpected error occurred* over the stack trace. | Improvement 4. |
| *Import* is on only when a file can be imported; Compose's was on when any file passed the old checks. | Improvement 1. |

### Tests

* **Rust** (`settings_ui_extensions/import.rs`, `import_files.rs`, `import_common.rs`; 17 new
  tests, 55 in the binary): each importer's title and layout; *Select files* to the read, the
  files report and the boxes; waiting for the read; *Import* and what it sends; closing the
  workspace on *Cancel*, Back and other files; a dismissed picker and a missing one; a shared
  file started through `SettingsUiExtensionLink.SHARED`; recreation; a cleared workspace; each
  failure's words; work lost with the app; Export's pick and job; each box's lines, ink,
  italics, spacing and bar; the skip reasons and their order; another type; an italic line as tall
  as the text painted in it; and malformed reports. `page.rs`, `lists.rs`, `settings_ui_screens.rs`, `settings_ui_setup.rs`,
  `settings_ui_theme_manager.rs` and `settings_ui_languages_picker.rs` now expect the importers
  native, and the theme manager's and language packs' *Import* to open them.
* **JVM**: `SettingsExtensionImportMatchTest` (4 tests) runs the importer in the real native
  library with the report built by `ExtensionFileWork`'s own `importRows`, against test-only
  copies of ExtensionImportScreen's decisions (`ReferenceExtensionPolicy.skip` and `accepts`):
  over 40 random workspaces of up to five files and all four importers, each box's name, type
  and head line, each skip reason, whether *Import* is on and which files it imports. It also
  covers improvements 1, 7 and 8. `SettingsEntryTest` checks that an opened or shared file
  starts the importer on it, set up or not; `ComposeFallbackLinksTest` that the fallback takes
  only later browsable links; `SettingsResourcesTest` the new strings.
  `ExtensionMetadataMatchTest` lost the asserts of the two ops that went.
* **Gate** (rebased on main b10bee9b1): 1,005 JVM tests in 245 files (part 1 left 1,000 in
  244) and 1,380 cargo tests (part 1 left 1,363), none failing or skipped; 391 native
  declarations, none hand written; the ledger matches 421 files (part 1 left 422); lint shows
  only the allowed issues. The first run flagged `ext__import__error_unexpected_exception` as
  unused, the label over the stack trace Compose showed; it was deleted and the gate run again.

### Mutations

`scripts/settings_ui_mutation_cases/extensions.py` has 44 new cases for part 2, and one changed:
8 must fail a JVM test (`SettingsExtensionImportMatchTest`, `SettingsEntryTest`,
`ComposeFallbackLinksTest` or `SettingsResourcesTest`) and 36 the Rust tests. Of the Rust ones,
34 are in the importer's code, `ext-import-composed` puts an importer back on the Compose list in
`catalogue.rs`, and `ext-import-cancel-imports` makes the language pack editor's *Import* skip
the importer. `ext-export-edits`, from part 1, now breaks the Rust call to `export` instead of
the hand-off. The runner now also runs `SettingsExtensionImportMatchTest`. **All 45 were
caught**, and the restored code passed. They ran in three goes, kept in
`build/mutations-extensions/` (not committed):

* `mut-part2`: the first 40 caught; then `ext-import-read-gone-closed` **survived**. A read that
  finishes after the importer was left must delete what it read, and no test covered that.
  `leaving_the_importer_any_way_closes_its_workspace` now does, and checks that a failed read
  closes nothing.
* `mut-part2b`: that case and the last three caught.
* `mut-part2c`: `ext-import-italic-measured-upright`, added with the device check's fix below,
  caught.

### Device check

This ran on `emulator-5556` (Android 16, 1080×2400). The screenshots are in
`build/extensions/` and are not committed.

The first install showed a blank page after *Export* came back. It was the wrong build:
`scripts/install-dev.sh` installs the APK already built and does not build one, and that APK
dated from part 1, where *Export* still handed off to the Compose fallback. The checks below
ran on a build of this branch.

1. `16-export-picker.png`: *Export* on *FlorisBoard default themes* opened the system's
   *Create document* picker at once, with `org.florisboard.themes.flex` filled in. No other
   activity started (the event log shows only the picker).
2. `17-export-done.png`: *Successfully exported extension!*, with the page still showing.
3. `18-importer-empty.png`: the importer for any extension: *Select files*, *No files selected*,
   *Import* off.
4. `19-import-core-skipped.png`: the export read back, in its box as FileInfoView drew it, and
   skipped because it would replace a built-in extension. *Import* stayed off. Its description
   left a blank line under it, which led to the fix below.
5. `20-import-rotated.png`: rotating destroyed and recreated the activity both ways (the event
   log shows each relaunch), and the file stayed selected.
6. *Cancel* went back to the home. A test theme made with *Create extension* (still Compose
   until part 3), exported as `local.themes.<id>.flex`, then deleted, was read back:
   `21-import-ready.png` shows it importable with *Import* on.
7. `22-import-success.png`: *Import* said *Successfully imported extension!* and closed the
   importer; the theme was back in the theme list.
8. `23-import-other-type.png`: `ui://florisboard/ext/import/EXT_KEYBOARD` with the theme picked
   shows it skipped: *The file holds a different type of extension than this screen imports.*
   *Import* stayed off (improvement 1).
9. With that file shown, the app's cache held one import workspace; after system Back it held
   none (improvement 3).
10. `24-import-shared.png`: *Share* on the file in the Files app, then *AudioWhisper Keyboard,
    Import Extension*, opened the importer on it without *Select files*. *Cancel* left no
    workspace in the cache.
11. `25-import-two-core-fixed.png`: after the fix, two files picked at once each show their own
    box, and a description that fits on one italic line has no blank line under it.

**Found and fixed:** an italic line in a file's box was measured with Android's upright layout
but painted with the italic one. Where the two wrap apart, the box kept an empty second line.
`line` now measures italic text as it is painted
(`an_italic_description_is_as_tall_as_its_painted_text`, mutation
`ext-import-italic-measured-upright`).

The Files app's file rows ignored agent-device's taps, in the app and in the picker, so the files
were chosen by long-press and *Select*. Opening a file from the Files app (the `VIEW` filter)
was not tried: the app offers *Open with* only through a tap. Afterwards the test theme was
deleted, the three exports were removed from Downloads, no import workspace was left, the
rotation was back at 0 with auto-rotate off, our IME was the default and the volume was at 0.
The selected theme was never changed. The agent-device session was closed.

### Deleted

`ExtensionImportScreen` (294 counted lines, with `ExtensionImportScreenType`) and
`ExtensionExportScreen` (56): 350 counted lines; their routes, deep links and composables in
`Routes.kt`; the Compose fallback's file reading in `FlorisAppActivity`; `CacheManager`'s
one-file read and `FileInfo.skipReason`; `ExtensionPolicy.accepts` and `skipReason`; and the
`ext__import__error_unexpected_exception` string.

## Owed

* **P23.** Part 1 on a phone, with TalkBack heard. The emulator check read the accessibility
  tree, and nobody listened to it.
* **E36.** No instrumented test drives the Rust extension screens on a device. The JVM match
  tests run the real native library against the Compose reference, but the device check above
  was by hand.
* **P24.** Part 2 on a phone, with TalkBack heard: the importer's boxes, its skip reasons and
  *Export*'s picker.
* **E37.** No instrumented test drives the importer or *Export* on a device. In particular, fix
  9 in `CacheManager` (the read waits for its workspace, and a failed read deletes what it
  copied) has no test: the JVM tests have no Robolectric, and the device check imported through
  it by hand. Opening a `.flex` from the Files app (the `VIEW` filter) was not tried on the
  device; sharing one was.

## What remains

* **Part 3, the extension editor.** `ExtensionEditScreen`, `ExtensionEditFilesScreen` and
  `ExtensionComponentView`, writing `extension.json` with snygg's `json_write.rs` (it matches
  `PrettyPrintConfig` byte for byte). Then the theme manager's *Create* and *Edit* stay in
  Rust, `FlorisAppActivity`'s `fallbackHandles` and `fallbackDone` hand-off goes (owed P20),
  and `ExtensionNotFoundScreen`, `RadioListItem` and `ExtensionDialogProperty` are deleted with
  the rest of `FB/app/ext`.
