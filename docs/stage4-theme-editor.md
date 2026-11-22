# Stage 4: the theme manager and theme editor

This page covers step 9 of [the Compose removal plan](compose-removal.md): the theme manager and
the theme editor move from Compose to the Rust settings renderer. The owner's rule is that they
are "ported to Rust and improved, not cut, and the port names each improvement it makes"
([decision](decisions.md#everything-moves-to-rust-2026-11-21)).

Step 9 has three parts, and **all three are done**: part 1, the theme manager; part 2, the theme
editor's shell; and part 3, the property editor, after which the Compose editor was deleted.
This page records each. [What remains](#what-remains) lists what is owed.

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/theme/manage/SELECT_DAY` | Rust | New here: the day theme manager. |
| `settings/theme/manage/SELECT_NIGHT` | Rust | New here: the night theme manager. |
| `ext/edit/null?create=ime.extension.theme` (create a theme) | Compose | Step 10. The manager hands off to it. |
| `ext/edit/<id>` (the extension editor) | Compose | Step 10. Its *Manage component* opens the Rust theme editor. |
| `settings/theme/edit/<workspace>/<component>` | Rust | New in part 2: the theme editor, in `ThemeEditorActivity`. |
| `settings/theme/editor/rule/add`, `.../rule`, `.../meta`, `.../tune` | Rust | New in part 2: the rule editor, the meta editor and fine-tune. |
| `settings/theme/editor/property/add`, `.../property`, `.../code/add`, `.../code` | Rust | New in part 3: the property editor and the code editor. |
| `ext/view/<id>` (an extension's page: export, delete) | Rust | Step 10, part 1 ([stage 4: extensions](stage4-extensions.md)). |
| `ext/import/EXT_THEME` | Compose | Step 10. |

The Compose route `Settings.ThemeManager` stays as an empty entry that hands back to Rust, as in
step 2. Its argument is now a plain string, since nothing in Compose reads it.

## Part 1: the theme manager (done)

### What the manager shows

The day and night managers are the same screen; the route decides which preference a tap writes
(`theme__day_theme_id` or `theme__night_theme_id`). Both have the preview field, so the keyboard
can be tried with the theme on show.

1. **A box with *Create theme extension* and *Import*.** New here (improvement 1).
2. **One outlined box per installed theme extension**, in the index's order, the language pack
   manager's box: the extension's name in the border, its id under it, then its themes sorted by
   label. Each theme is a Material list item: a radio button (on for the chosen theme), the label,
   and a small sun or moon icon at the end in the faded `onSurfaceVariant` Compose used.
3. **Under each extension's themes,** *View details*, and *Edit* for an extension made or imported
   on this phone (improvements 2 and 3).

While a manager is on top, the keyboard shows the chosen theme, as Compose's `DisposableEffect`
did. Choosing another theme switches the keyboard at once. Leaving the manager, or the settings
screen closing, gives the keyboard its own theme back.

### Data path

```
ExtensionManager.themes (Kotlin LiveData)
  └─ SettingsThemes.rows()  ─ THEMES op, rows of wire::settings_ui::themes ─▶  App::set_themes
                                                                                  │ themes::parse: group, sort
App::themes_part (cards) ◀─────────────────────────────────────────────────────────┘
  tap ─▶ App::pick_theme ─▶ write "ext:comp" to the day or night pref (the normal write effect)
       └▶ sync_theme_preview ─▶ ["themePreview", "ext:comp"] ─▶ SettingsThemes ─▶ ThemeManager.previewThemeId
```

* **Rust asks, Kotlin answers.** Opening a manager emits `["themes"]` once. `SettingsThemes`
  replies with one `extension` row per installed theme extension (id, title, whether it can be
  deleted) followed by one `theme` row per theme (id, label, night). After that it re-sends
  whenever the extension index changes, so a theme imported or deleted elsewhere redraws the
  boxes.
* **Rust does the logic.** It groups the rows by extension as `buildMap` did (a repeated id keeps
  its first place and takes the later extension's themes), sorts each extension's themes by label
  in Kotlin's `String` order (UTF-16 units, stable), works out which theme is chosen from the
  stored preference, and writes the choice as `SettingsComponentSerializer` stores it
  (`<extension>:<component>`). A malformed report is refused and the last good one kept.
* **The preview** is the only state Kotlin keeps. Rust sends `themePreview` with the chosen theme
  when a manager comes on top or the choice changes, and an empty name when no manager is on top.
  It sends only changes. Kotlin sets `ThemeManager.previewThemeId` and clears it when the view is
  released.
* **A restored screen** (after a rotation or process death) asks for the report again and
  re-sends the preview.

The wire rows are `wire::settings_ui::themes` (op `THEMES` = 27, effects `themes` and
`themePreview`), generated for Kotlin as `SettingsUiTheme`, `SettingsUiThemeKind` and
`SettingsUiThemeEffect`.

### What Kotlin does

Two files are new. Both must be Kotlin because only Kotlin can reach what they read:

* **`settingsui/SettingsThemes.kt`** (36 counted lines): reads `ExtensionManager.themes`, the
  extensions' titles and `canDelete` (whether the extension came from the phone's storage, not
  the app's assets), and sets `ThemeManager.previewThemeId`. These are Kotlin objects owned by
  the app's extension and theme managers.
* **`settingsui/SettingsThemeStringIds.kt`** (9): the string resource ids of the manager's new
  labels. Android string resources have to be looked up from Kotlin; the map sits in its own file
  so `SettingsStringIds` stays under the file limit.

`RustSettingsView` gains four lines: it creates the adapter before the saved state comes back,
routes the two effects to it, and releases it with the view.

### Improvements

Each was chosen for a person choosing or making a keyboard theme. Gboard's theme page was the
reference: making your own theme is the big **+** at the top of it.

| # | Improvement | Why | Test |
| --- | --- | --- | --- |
| 1 | *Create theme extension* and *Import* at the top of the manager. Compose had neither: you went back to Theme, then to *Theme extensions*, to find them. | Gboard puts making your own theme first. Making and importing a theme is what a person on this page is likely to want next. | `create_and_import_hand_off_to_the_compose_extension_screens` |
| 2 | An extension's name, its id and *View details* open its page, where it is exported or deleted. | Compose's manager offered no way to reach an extension from the theme list. | `an_extensions_name_id_and_view_details_open_its_page_and_edit_its_editor` |
| 3 | *Edit*, for an extension made or imported here, opens the extension editor directly. | The way to change a theme you made is two screens shorter. Built-in themes cannot be edited, so they get no *Edit*. | same test |
| 4 | TalkBack says whether each theme is a day or a night theme. | Compose's sun and moon icon had no content description, so a TalkBack user could not tell them apart. | `talkback_hears_whether_each_theme_is_a_day_or_a_night_theme`; JVM `eachThemeTellsTalkBackWhetherItIsADayOrNightTheme` |
| 5 | TalkBack hears which theme is chosen. | Compose's radio button had no click handler, so its row reported no selected state. | the `selected` checks in `each_extension_is_a_box_titled_with_its_name_over_its_id_listing_its_themes` and `a_tap_writes_the_day_theme_and_previews_it`; JVM `aTapWritesWhatTheComponentSerializerStoresAndThePreviewNamesIt` |

### Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| *Create* and *Import* sit in a box at the top of the manager, not in the extension list's floating button. | Improvement 1. The extension list itself is step 10 and keeps its button. |
| A built-in extension has no *Edit* button. Compose's extension list shows it greyed out. | A button that can never work is noise; TalkBack would read it on every built-in box. |
| The preview is also cleared when the settings view is released, not only when the manager leaves the screen. | A settings screen killed with a manager on top would otherwise leave the keyboard showing a theme that is not the chosen one. |
| If the day or night preference changes elsewhere while a manager is open, the keyboard's preview is not updated until the manager is reopened. The radio buttons do update. | Nothing else writes these preferences while the manager is on top. |
| A repeated extension id in the index is handled. Compose's `!!` lookup could crash on one. | It costs one branch. |
| Keyboard and D-pad focus moves through *Create*, *Import*, then each box's name, id, themes, *View details* and *Edit*. | Every Rust screen has a focus order; Compose's was whatever the layout gave. |

### Tests

* **Rust** (`lib/rust/crates/render/tests/settings_ui_theme_manager.rs`, 13 tests): the
  grouping and sorting, including non-ASCII labels and a repeated id; malformed reports refused;
  opening each manager natively, with the preview field, asking once for the report and
  previewing the chosen theme; Back clearing the preview; the boxes, titles, ids and their taps;
  TalkBack's day or night state; the faded icon; a tap writing the day or night preference and
  previewing it once; a new report redrawing an open manager; *Create*, *Import*, *View details*
  and *Edit* handing off, and no *Edit* for built-ins; the focus order; and a restored screen
  asking again. `settings_ui_theme.rs` now checks that the Theme screen's day and night rows open
  the native manager, and `settings_ui_screens.rs` lists the two new native routes.
* **JVM** (`SettingsThemesMatchTest`, 3 tests), through the real native library with the report
  built by `SettingsThemes`' own row builder: 40 random indexes grouped and sorted as the frozen
  `ThemeManagerScreen` did (ids repeat; labels include accents, an emoji, a ligature and the empty
  string); a tap writing exactly what `SettingsComponentSerializer` stores and a preview that
  `ExtensionComponentName.from` reads back, for both managers; and TalkBack's day or night state.
  The settings route lists in `SettingsGroupOneMatchTest`, `SettingsGroupTwoMatchTest` and
  `SettingsUiMatchTest` include the two routes.
* **Device** (`SettingsOtherThemeDevice.manager()`, run by `scripts/settings-device-tests.py`):
  picks a theme on the Rust manager, waits for the preference and for the keyboard's preview to
  start, goes back, and checks the Theme row's TalkBack state and that the preview ended. It
  compiles; running it is owed (E23).
* **The gate**, on the code of 709b3ebab: `test`, `lintDebug`, `:lib:rust:check`,
  `oracleCheck`, `checkNativeDeclarations`, `checkJvmLedger`, `assembleDebug` and the
  instrumented tests' compile all pass. 950 JVM tests in 232 files (947 in 231 before), 0
  failures; 2,159 cargo test results passed, 0 failed, 0 ignored (2,133 before); 389 natives, 0
  hand-written, 0 mismatched (the manager reuses the settings native, so the count is
  unchanged); the ledger matches 437 files and 437 rows. Lint reports only the four allowed IDs.

### Mutations

A mutation is a small deliberate mistake put into the code to check that some test notices it.
`scripts/settings-ui-mutations.py` has 21 new ones for the manager (now in
`scripts/settings_ui_mutation_cases/themes.py`); each must make a test fail,
and the code is put back after each one. **All 21 were caught**, and the restored code passed
(`build/mutations-themes/` for the first four; that run then stopped when the Gradle daemon was
killed mid-build, with no test run, so the other 17 ran again in `build/mutations-themes2/`,
whose `result.json` also records the restored runs).

| Mutation | What it breaks | Caught by |
| --- | --- | --- |
| theme-sort-by-code-point | Themes sorted by code point, not Kotlin's UTF-16 order | JVM match test |
| theme-repeat-keeps-old-themes | A repeated extension id keeps the earlier extension's themes | JVM match test |
| theme-day-writes-night | The day manager writes the night theme | JVM match test |
| theme-value-slash | The choice stored as `ext/comp`, which the serializer cannot read | JVM match test |
| theme-pick-not-previewed | A tap writes the theme but the keyboard keeps the old preview | JVM match test |
| theme-night-dropped | Every theme read as a day theme | JVM match test |
| theme-chosen-not-selected | TalkBack hears no theme as chosen | JVM match test |
| theme-row-night-unsent | Kotlin's report sends every theme as a day theme | JVM match test |
| theme-string-id-dropped | *View details* has no string id, so it shows its key | `SettingsResourcesTest` |
| theme-asked-every-open | The index asked for again while a report is on its way | Rust manager test |
| theme-report-keeps-asked | A report leaves the request open, so the next manager never asks | Rust Theme test |
| theme-preview-every-sync | The preview re-sent with no change | Rust manager test |
| theme-back-keeps-preview | Back leaves the keyboard showing the manager's theme | Rust manager test |
| theme-restore-not-asked | A restored manager never asks for the index | Rust manager test |
| theme-edit-for-built-ins | Built-in extensions get an *Edit* that can never work | Rust manager test |
| theme-view-opens-editor | *View details* opens the editor | Rust manager test |
| theme-import-creates | *Import* opens *Create* | Rust manager test |
| theme-icon-unfaded | The day or night icon drawn at full strength | Rust manager test |
| theme-focus-skips-themes | Keyboard focus skips the themes | Rust manager test |
| theme-unknown-kind-accepted | A report row of unknown kind is accepted | Rust manager test |
| theme-extension-unchecked | A theme row under the wrong extension is accepted | Rust manager test |

### Device check

On `emulator-5556` (`aw_keyboard_ssd`, Android 16), installed with `scripts/install-dev.sh` from
the build the gate made, driven through agent-device. Screenshots are in `build/theme-editor/`
(not committed). **Every step passed:**

1. `ui://florisboard/settings/theme/manage/SELECT_DAY` opens the Rust day manager: *Create* and
   *Import* at the top, then the built-in extensions' boxes with their ids, radio buttons, the
   faded sun and moon icons and *View details*, and no *Edit* (`01-day-manager.png`). The
   accessibility tree reports the chosen theme as selected and checked.
2. Choosing Floris Pure Night as the day theme moves the radio button, and the preview field's
   keyboard shows it at once (`02-day-picked-preview.png`). The Theme screen's *Day theme* row
   then names it (`03-theme-screen-day-row.png`).
3. In the night manager, choosing Floris Day (Borderless) shows that light theme on the keyboard
   while the manager is on top, although the theme mode is *Always day*
   (`04-night-picked-preview.png`). Back on the Theme screen the keyboard shows its own day theme
   again and *Night theme* names the new choice (`05-left-manager-own-theme.png`).
4. *Create theme extension* opens Compose's *Create theme extension* (`06-create-handoff.png`).
   An extension made there from Floris Night and saved comes back to the Rust manager, which
   lists it in its own box with *Edit* (`07-created-theme-in-manager.png`); choosing its theme
   works like any other.
5. *Edit* opens Compose's extension editor for it (`08-edit-handoff.png`), whose theme editor
   still opens (`09-compose-theme-editor.png`); Back returns to the manager.
6. *View details* opens its page (`10-view-details-handoff.png`); deleting it there removes its
   box from the manager without reopening it (`11-deleted-gone.png`). With the chosen day theme
   deleted, no radio button is on, as in Compose, which compared the stored id the same way.

The day and night themes were set back to Floris Day and Floris Night afterwards.

Not checked on the device: TalkBack speaking the day or night state (the tests cover the node's
state description; P15 owes hearing it), and the editor itself, which is still Compose.

### Deleted

`ThemeManagerScreen` (135 non-blank lines), and its route's enum argument in `Routes.kt`.
The remaining 14 files of `FB/app/settings/theme` are deleted after part 3.

## Part 2: the theme editor shell (done)

Rust now draws the theme editor that the extension editor's *Manage component* opens. Every part
of it is ported except adding a property and editing a property's value, which are part 3.
Properties are listed with their values, and tapping one does nothing yet. `ThemeEditorScreen`
and the other Compose files are no longer routed to. They are not deleted: part 3 needs them as
its reference, and other code still imports from their package.

### How the editor is reached

The extension editor (`ext/edit/<id>`, Compose until step 10) keeps the extension unpacked in a
`CacheManager.ExtEditorWorkspace` that lives as long as the process. *Manage component* used to
show `ThemeEditorScreen(workspace, editor)` in its place. It now starts `ThemeEditorActivity` on
`ui://florisboard/settings/theme/edit/<workspace uuid>/<component index>`, and clears its own
action, so Back from the Rust editor returns to the extension editor.

The editor needs its own activity. `FlorisAppActivity` is `singleTask`, so starting it would
clear the fallback activity that holds the extension editor, and the workspace with it.
`ThemeEditorActivity` is an empty subclass with a standard launch mode. Rust draws everything in
it.

The route holds the workspace and the component's index, not the component's id, so renaming
the component does not break it. A workspace the process no longer has (after the process was
killed, say) gives a page saying the theme is no longer being edited.

| Route | What it is |
| --- | --- |
| `settings/theme/edit/<workspace>/<index>` | The editor: the component box, fine-tune, *Add rule* and the rules. |
| `settings/theme/editor/rule/add` and `settings/theme/editor/rule` | The rule editor, adding or editing (`EditRuleDialog`). |
| `settings/theme/editor/meta` | The component meta editor (`ComponentMetaEditorDialog`). |
| `settings/theme/editor/tune` | Fine-tune (`FineTuneDialog`): the editor level, the colour representation and whether the keyboard shows after a dialog. |

### What it shows

1. **Loading.** *Loading the stylesheet…* until Kotlin's report arrives.
2. **A stylesheet that does not parse** shows `ThemeEditorScreen`'s question as a page: the
   error, then *Yes* to load what can be read (leniently, as `SnyggStylesheet.fromJson` did) and
   *No* to start empty. With no file at all, the editor starts empty, as
   `loadThemeEditorStylesheet` did.
3. **The component box**, `ExtensionComponentView`: the label in the border, `extension:id`
   under it, the authors, night flag and stylesheet path, and *Edit*.
4. ***Fine-tune*** and ***Add rule***, as list items where Compose had a top bar button and a
   floating button.
5. **The rules** in `compare_order`, each in an outlined box drawn as `SnyggRuleRow`: the element
   by its translated name, written as-is at the developer level; its selector; then each
   attribute as `key = [values]`. Each rule but `@defines` has *Duplicate* and *Edit*, where
   Compose had *Edit* and *Add property*. Under it come the properties, each a name over its
   value with *Delete*. A rule with several property sets (`@font`) heads each set with "Source
   set". `@defines` has its note.
6. **The rule editor**: the element (a dropdown while adding, fixed while editing), the font's
   name for `@font`, the selector chips, the key codes as one comma-separated field, and the
   mode and shift state chips with *Add*. *Add* or *Apply* refuses a rule that already exists, a
   bad code (with its range), adding with no element, and a font name the stylesheet cannot read
   back. Each says why. Editing has *Delete* in the top bar, which asks first.
7. **The meta editor**, in Compose's order: the id, the label, the authors (one comma-separated
   line), the night switch, the stylesheet path, and Material You's palette style, contrast level
   and spec version as dropdowns of their enums' names. An empty stylesheet path shows, faded,
   where the stylesheet goes by the id being edited, as Compose's placeholder did. *Apply* checks each field with the
   extension validation rules, filling in the pattern a message names as Compose did. It also
   refuses an id another component of the extension has, and shows why under each field.
8. **Fine-tune**: its three preferences on the existing choice rows. The keyboard comes back
   after a dialog as *Display keyboard after dialogs* says.
9. **Save** in the top bar writes the stylesheet and the meta back and closes. A stylesheet that
   cannot be written (a font set without `src`, read leniently) stays open with the reason.
   **Back** with unsaved changes asks whether to discard them; without changes it closes.

While the editor is on top, the keyboard shows the draft through `ThemeManager.previewThemeInfo`,
as Compose did, and redraws as the draft changes. Leaving the editor, or the view being
released, gives the keyboard its own theme back.

### Data path

```
ExtEditorWorkspace (Kotlin, CacheManager)
  └─ SettingsThemeEditor.load ─ THEME_EDITOR op (28), a wire::settings_ui::theme_editor report ─▶ App::set_theme_editor
                                                                                  │ snygg::json::parse, strict, then as asked
theme_editor::paint (component box, rules) ◀────────────────────────────────────────┘
  edits ─▶ the draft (rules and meta) ─▶ ["themeEditorPreview", ws, index, json] ─▶ ThemeManager.previewThemeInfo
  Save  ─▶ snygg::json_write (Compose's PrettyPrintConfig) ─▶ ["themeEditorSave", ws, index, json, meta…]
        ─▶ SettingsThemeEditor.save: the file in the workspace, a new component in the extension
        ─▶ the extension editor's own Save zips the workspace, as before
```

* **Rust asks, Kotlin answers.** Opening the editor emits `["themeEditor", workspace, index]`.
  `SettingsThemeEditor` replies with one report: the outcome (`loaded` or `missing`), the
  stylesheet's text (a Compose editor's draft if one is held, else the file), the extension's id,
  the component's id, label, authors, night flag and own stylesheet path, the workspace's font and
  image file names, the Material You flags by name with the names each can take, and the other
  components' ids. A report for another component than the one on screen is ignored.
* **Rust does the logic.** It parses the stylesheet with `snygg`, orders and translates the rules,
  edits the draft, checks the rule and meta editors, and writes the stylesheet with a new
  serializer (`snygg::json_write`) that matches Compose's `PrettyPrintConfig` exactly: key
  order, two-space indent, escapes and number formats. `SettingsThemeEditorMatchTest` holds it to
  `SnyggStylesheet.toJson` on every shipped stylesheet, so a theme saved by Rust opens unchanged in
  an older build.
* **Save** sends the stylesheet and the meta. Kotlin writes the stylesheet where the new
  component keeps it, which is where `handleSave` copies it from when no Compose editor holds
  one. It then puts a new `ThemeExtensionComponentEditor` with the meta at the same index. It must
  be a new one: the extension editor stays composed under `ThemeEditorActivity` and skips a
  component it has already drawn, so changing the old one in place left the old label on screen.
* **The draft is in the saved state**, as its last field (version 8, 21 fields). Rotating or
  recreating the activity brings back the unsaved rules and meta, and an open rule or meta editor
  with what it holds. Kotlin is asked for the report again, and the draft replaces what it
  brings. A draft for another component, or one that does not parse, is dropped. The editor is
  restored only as the root of `ThemeEditorActivity`, never above another screen.

The wire is `wire::settings_ui::theme_editor`: op `THEME_EDITOR` = 28, effects `themeEditor`,
`themeEditorSave` and `themeEditorPreview`, outcomes `loaded` and `missing`. It is generated for
Kotlin as `SettingsUiThemeEditor`, `SettingsUiThemeEditorEffect` and
`SettingsUiThemeEditorOutcome`. No native was added: the editor uses the settings native's op
channel.

### What Kotlin does

Three files are new. Each must be Kotlin because only Kotlin can reach what it touches:

* **`settingsui/SettingsThemeEditor.kt`** (95 counted lines). It reads the extension editor's
  workspace from `CacheManager`, the stylesheet file and the font and image directories. It reads
  materialkolor's `PaletteStyle`, `Contrast` and `ColorSpec.SpecVersion` entries. On Save it
  writes the file and the new component, and it sets `ThemeManager.previewThemeInfo`. These are
  Kotlin objects owned by the app's cache and theme managers, and a Kotlin library's enums. Rust
  parses, edits, checks and serializes.
* **`settingsui/SettingsThemeEditorStringIds.kt`** (204). The string resource ids of the editor's
  labels and of the element, property, selector and value names it translates. Android string
  resources must be looked up from Kotlin; the map is its own file so `SettingsStringIds` stays
  under the file limit.
* **`app/ThemeEditorActivity.kt`** (18). An `Activity` subclass has to be Kotlin. It exists for the
  launch mode explained above.

Four files grow:

* `RustSettingsView` (+4): creates the adapter before the saved state comes back, routes its three
  effects to it, and releases it with the view.
* `SettingsIcons` (+1): the *Fine-tune* and *Duplicate* icons.
* `SettingsStringIds` (+1): joins the editor's map to the others.
* `ExtensionEditScreen` (+5, frozen Compose): *Manage component* opens the new activity instead of
  `ThemeEditorScreen`. This is the routing change the brief asked for.

The manifest declares the activity, and `strings.xml` gains six strings: *Duplicate rule*, the
font name and duplicate id messages, the delete question, *Loading the stylesheet…* and the
missing workspace page.

### Improvements

Each was chosen for a person making a keyboard theme.

| # | Improvement | Why | Test |
| --- | --- | --- | --- |
| 1 | **Undo in place.** Deleting a rule or a property leaves an undo bar where it was, inside the rule's box, and *Undo* puts it back in the same place. | Compose deleted at once, with no way back. A theme author tries things out. | `a_deleted_property_leaves_undo_where_it_was_and_undo_puts_it_back`, `deleting_a_rule_asks_first_and_undo_puts_it_back_in_its_place`, `the_defines_note_and_an_undo_bar_keep_inside_the_rule_box` |
| 2 | **Duplicate rule.** Opens the rule editor with the rule's element, selector and attributes; the rule it adds takes the original's properties, unless its element was changed to one taking the other kind of set (fixed in part 3, [below](#fixed-on-the-way)). | The usual way to make a pressed or a numeric-mode variant of a key's style. Compose had you add the rule, then every property, by hand. | `duplicate_adds_a_rule_like_another_with_its_properties`, `a_duplicate_takes_its_sources_sets_only_where_its_element_takes_that_kind` |
| 3 | **No *Delete* on a font's `src`.** | A font set without `src` cannot be written: Compose let you delete it, and the extension then failed to save. | `a_fonts_source_has_no_delete_and_its_other_properties_do`; snygg `a_font_set_without_its_src_fails_as_to_json_does` |
| 4 | **A font name the stylesheet cannot read back is refused**, with why. | Compose accepted one, and the saved stylesheet then failed to load. | `a_font_needs_a_name_the_stylesheet_can_read_back` |
| 5 | **Edits are a draft.** *Save* keeps them; *Back* with changes asks whether to discard them. | Compose wrote each edit straight into the workspace and had no *Save*, so the only way to drop a mistake was to discard the whole extension's changes. | `save_writes_the_stylesheet_rust_built_and_the_meta_then_closes`, `back_with_unsaved_changes_asks_to_discard_and_without_them_closes` |
| 6 | **Attributes are listed in the order the file writes them.** | Compose showed them in the order they were added, so a rule's lines changed order after Save and reopen. | the end of `a_new_element_rule_takes_its_selector_codes_modes_and_shift_states` |
| 7 | **A repeated component id is refused under its field.** | Compose said so in an English-only toast that went as soon as it came. | `a_bad_or_taken_id_or_a_bad_path_refuses_apply_and_says_why` |
| 8 | **The extension editor shows the new meta at once after *Save*.** | Found on the emulator: changing the component in place left the old label and night flag on screen. | JVM `theReportCarriesTheFileTheMetaAndTheNamesAndSaveWritesThemBack` (`assertNotSame`) |

The *Delete* question for a rule now says it can be undone. Rule filtering was considered and
left out: a stylesheet has tens of rules, and the list is already grouped by element.

### Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| The rule editor, the meta editor and fine-tune are screens, not dialogs. | The settings renderer's editors are screens with a bottom bar. Back closes them as it closed a dialog. |
| The stylesheet error is a page with *Yes* and *No*, not a dialog over an empty editor. | Nothing behind it can be used until it is answered. |
| The error's text is `snygg`'s, not kotlinx.serialization's. | Rust parses the stylesheet. A syntax error gives `serde_json`'s line and column. |
| *Fine-tune* and *Add rule* are list items. | The settings renderer has no top bar actions or floating button. |
| The authors are one comma-separated line, where Compose took one per line. | The settings renderer's fields are single-line. Blank authors are dropped, as before. |
| The key codes are one comma-separated field. Compose added each code in its own dialog, with *record a key*. | Recording a key is `EditCodeValueDialog`, part 3. Typed codes are checked against the same ranges. |
| A property's value shows as the stylesheet writes it. | Value previews (`SnyggValueIcon`) and the colour representation are part 3. Fine-tune's colour representation is stored now and used then. |
| Tapping a property does nothing, and a rule has no *Add property*. | Both open `EditPropertyDialog`, part 3. |
| A set missing a required property shows no "Errors" box under its rule. *Save* refuses it with the reason instead. | The box belongs with *Add property*, which fixes what it reports. Only a stylesheet read leniently can lack one: the editor cannot delete a required property. |
| The draft lives in the saved state, not the workspace. | That is what makes improvement 5 possible. Rotation keeps the draft and any open editor, as Compose's `rememberSaveable` and the workspace did. |
| The undo bar does not survive rotation. | As the user dictionary's: it is a moment's offer, not part of the draft. |
| The editor has its own activity. | The launch mode, above. |
| No italics or monospace in the rules list. | The settings renderer draws one face. |

### Tests

* **Rust, `settings_ui_theme_editor`** (28 tests in `rules.rs`, `form.rs`, `meta.rs` and
  `saved.rs`): loading, a missing workspace and the error page's two answers; the rules' order,
  names and lines at each editor level; the preview; deleting with undo in place and inside the
  box; no *Delete* on `src`; *Duplicate*; deleting a rule with its question; fine-tune and the
  keyboard after it; *Save*'s packet; *Back* asking; the rule editor's elements, selectors,
  codes, modes, shift states and every refusal; *Apply* renaming in place; the meta editor's
  every field, its checks and the id another component has; and the draft, an open editor and
  the meta across recreation, with a draft for another component or a malformed one refused.
  `settings_ui_screens.rs` lists the new routes.
* **Rust, `snygg/tests/json_write.rs`** (5 tests): every shipped stylesheet round-trips; the
  layout and escapes of kotlinx's pretty printer; a font set without `src` fails as `toJson`
  does; and lenient parsing keeps what it can.
* **JVM, `SettingsThemeEditorMatchTest`** (5 tests): every shipped stylesheet written by Rust
  exactly as Compose's `toJson(PrettyPrintConfig)` wrote it; a stylesheet a Compose editor
  changed read back and written unchanged; properties deleted in Rust written as Compose would
  write them without; and the element order and the element and property strings held to
  FlorisImeUi's and ThemeTranslations'.
* **JVM, `SettingsThemeEditorGlueMatchTest`** (4 tests), with the glue's own functions and the
  real natives: the report's every field and Save's packet round-tripping through Rust; Save's
  new component, what it keeps, where the file goes, and a flag name no enum has; a Compose
  editor's stylesheet reported over the file; a component that keeps its stylesheet by its id
  reporting no path of its own; a missing component; and the draft across recreation, restored
  only as the activity's root.
* `SettingsGroupOneMatchTest`, `SettingsGroupTwoMatchTest`, `SettingsUiMatchTest`,
  `SettingsSetupMatchTest` and `SettingsProjectLicenceMatchTest` count the new routes and saved
  state version 8. `SettingsResourcesTest` checks every new string key has its id.
* **The gate**, on the code of 3cc7ba633, after `git rebase origin/main` (40231dc9f): `test`,
  `lintDebug`, `:lib:rust:check` (clippy, fmt and the rustdoc check), `checkNativeDeclarations`,
  `checkJvmLedger`, `assembleDebug` and the instrumented tests' compile all pass. 965 JVM tests
  in 235 files (956 in 233 before), 0 failures; 1,164 cargo test results passed, 0 failed, 0
  ignored (1,131 before); 389 natives, 0 hand-written, 0 mismatched (the editor uses the
  settings native, so the count is unchanged); the ledger matches 439 files and 439 rows (436
  before). Lint reports only the four allowed IDs. The first run failed
  `ComposeStabilityLinkageTest` on stale `$stable` fields of `FlorisPreferenceModel` classes this
  work does not touch; it passed after `./gradlew :app:clean`.

### Mutations

`scripts/settings-ui-mutations.py` has 54 new cases for the editor (now in
`scripts/settings_ui_mutation_cases/theme_editor.py`): 43 in Rust and 11 in Kotlin.
35 of the Rust ones are checked by the Rust editor tests. The other 8 (the element names, the
serializer, and the native saved state and *Back*) and all the Kotlin ones are checked by the
JVM match tests, since those compare against Compose. Each case's text matches its source
exactly once. **All 54 were caught by a failing assertion**, and the restored code passed.

The first 45 ran in `build/mutations-theme-editor-4/`. That run stopped at
`theme-editor-save-old-path`: the glue test failed, but with a `FileNotFoundException`, not an
assertion, so the script did not count it. The test now asserts the file is there. That case and
the six after it ran again in `build/mutations-theme-editor-5/`, whose `result.json` also records
the restored run of the JVM suites. Run 4 stopped before its own restored run, so the restored
Rust code is held by the gate below, which runs every cargo test. The last two cases, for the
stylesheet path's hint found missing on the emulator, ran in `build/mutations-theme-editor-6/`,
whose restored run passed too.

| Mutation | What it breaks | Caught by |
| --- | --- | --- |
| theme-editor-order-reversed | The rules listed in reverse of `compare_order` | `a_new_element_rule_takes_its_selector_codes_modes_and_shift_states` and 8 more |
| theme-editor-developer-ignored | Elements written as-is at the advanced level, translated at the developer level | `a_new_element_rule_takes_its_selector_codes_modes_and_shift_states` and 7 more |
| theme-editor-selector-untranslated | Selectors shown by their raw names | `deleting_a_rule_asks_first_and_undo_puts_it_back_in_its_place` and 2 more |
| theme-editor-property-undo-at-end | *Undo* puts a property back at the end of its set | `a_deleted_property_leaves_undo_where_it_was_and_undo_puts_it_back` |
| theme-editor-src-deletable | A font's `src` gets a *Delete* | `a_fonts_source_has_no_delete_and_its_other_properties_do` |
| theme-editor-undo-not-in-place | The undo bar drawn at the end of the set, not where the property was | `a_deleted_property_leaves_undo_where_it_was_and_undo_puts_it_back` |
| theme-editor-exists-unchecked | A rule that already exists is added | `a_rule_that_exists_or_a_bad_code_is_refused_with_why` and 1 more |
| theme-editor-apply-clashes-itself | Applying a rule unchanged is refused as a clash with itself | `apply_renames_a_rule_in_place_and_keeps_its_properties` |
| theme-editor-internal-codes-refused | Internal key codes (negative) refused | `a_new_element_rule_takes_its_selector_codes_modes_and_shift_states` |
| theme-editor-font-not-read-back | A font name the stylesheet cannot read back accepted | `a_font_needs_a_name_the_stylesheet_can_read_back` |
| theme-editor-duplicate-empty | *Duplicate* adds the rule without the original's properties | `duplicate_adds_a_rule_like_another_with_its_properties` |
| theme-editor-new-font-single | A new `@font` rule gets one property set, not a list of them | `a_font_needs_a_name_the_stylesheet_can_read_back` |
| theme-editor-element-not-fixed | The element can be changed while editing a rule | `apply_renames_a_rule_in_place_and_keeps_its_properties` |
| theme-editor-meta-label-untrimmed | The label kept with its surrounding spaces | `edit_shows_the_meta_and_apply_changes_it_for_save_to_write` |
| theme-editor-meta-unchecked | *Apply* checks only the label | `a_bad_or_taken_id_or_a_bad_path_refuses_apply_and_says_why` |
| theme-editor-id-unchecked | The id checked with the label's rule | `a_bad_or_taken_id_or_a_bad_path_refuses_apply_and_says_why` |
| theme-editor-id-taken-unchecked | An id another component has is accepted | `a_bad_or_taken_id_or_a_bad_path_refuses_apply_and_says_why` |
| theme-editor-pattern-unfilled | A message's `{pattern}` left unfilled | `a_bad_or_taken_id_or_a_bad_path_refuses_apply_and_says_why` |
| theme-editor-path-ignores-id | The default stylesheet path ignores the id being edited | `edit_offers_the_id_the_path_and_material_you_and_save_writes_them` |
| theme-editor-save-flags-dropped | *Save* leaves out the Material You flags | `edit_offers_the_id_the_path_and_material_you_and_save_writes_them` and 1 more |
| theme-editor-flag-choice-ignored | Choosing a Material You flag changes nothing | `edit_offers_the_id_the_path_and_material_you_and_save_writes_them` and 1 more |
| theme-editor-saved-flags-dropped | The saved state loses the Material You flags | `the_meta_editor_and_the_meta_draft_survive_recreation` |
| theme-editor-blank-author-kept | Blank authors kept | `edit_shows_the_meta_and_apply_changes_it_for_save_to_write` |
| theme-editor-changed-ignores-meta | A meta change alone is not unsaved, so *Back* does not ask | `edit_shows_the_meta_and_apply_changes_it_for_save_to_write` and 1 more |
| theme-editor-keyboard-always-ignored | *Always* does not bring the keyboard back after a dialog | `fine_tune_lists_its_three_preferences_and_the_keyboard_comes_back_as_told` |
| theme-editor-stale-report-used | A report for another component used | `the_editor_says_it_is_loading_until_kotlin_answers_then_shows_the_component` |
| theme-editor-error-not-asked | A stylesheet that does not parse loaded leniently without asking | `a_stylesheet_that_fails_asks_and_yes_loads_it_leniently_and_no_starts_empty` |
| theme-editor-saved-editor-unheld | A rule or meta editor restored without what it held | `an_editor_saved_before_its_stylesheet_loaded_comes_back_on_the_rules_only` |
| theme-editor-saved-other-component | A draft for another component restored | `a_draft_for_another_component_or_a_malformed_part_is_not_used` |
| theme-editor-saved-form-dropped | An open rule editor loses what it holds across recreation | `unsaved_rules_and_an_open_rule_editor_survive_recreation` |
| theme-editor-note-past-box | `@defines`' note runs past the rule's box | `the_defines_note_and_an_undo_bar_keep_inside_the_rule_box` |
| theme-editor-undo-past-box | The undo bar runs past the rule's box | `the_defines_note_and_an_undo_bar_keep_inside_the_rule_box` |
| theme-editor-attributes-unsorted | Attributes listed in the order they were added | `a_new_element_rule_takes_its_selector_codes_modes_and_shift_states` |
| theme-editor-names-reordered | The element order differs from FlorisImeUi's | SettingsThemeEditorMatchTest |
| theme-editor-property-string-wrong | A property's name string differs from ThemeTranslations' | SettingsThemeEditorMatchTest, SettingsResourcesTest |
| theme-json-indent-tab | The stylesheet indented with tabs | SettingsThemeEditorGlueMatchTest, SettingsThemeEditorMatchTest |
| theme-json-tight-colon | No space after a key's colon | SettingsThemeEditorGlueMatchTest, SettingsThemeEditorMatchTest |
| theme-json-empty-object-opened | An empty object written across lines | SettingsThemeEditorMatchTest |
| theme-editor-back-not-held | A draft does not hold *Back*, so it leaves without asking | SettingsThemeEditorGlueMatchTest |
| theme-editor-saved-not-written | The draft left out of the saved state | SettingsThemeEditorGlueMatchTest |
| theme-editor-not-a-root | On restore, the editor refused as a back stack's root and every other route accepted as one | SettingsThemeEditorGlueMatchTest, SettingsGroupOneMatchTest and 5 other settings match tests |
| theme-editor-report-file-first | The file reported over a Compose editor's changes | SettingsThemeEditorGlueMatchTest |
| theme-editor-save-keeps-compose-editor | *Save* keeps a Compose editor, so `handleSave` writes its stylesheet over Rust's | SettingsThemeEditorGlueMatchTest |
| theme-editor-save-blank-authors | Kotlin's *Save* keeps blank authors | SettingsThemeEditorGlueMatchTest |
| theme-editor-save-in-place | *Save* changes the component in place, so the extension editor shows the old meta | SettingsThemeEditorGlueMatchTest |
| theme-editor-save-old-path | The stylesheet written where the old id kept it | SettingsThemeEditorGlueMatchTest |
| theme-editor-save-flags-kept | The palette style chosen is ignored | SettingsThemeEditorGlueMatchTest |
| theme-editor-report-path-effective | The default path reported as the component's own | SettingsThemeEditorGlueMatchTest |
| theme-editor-id-taken-string-dropped | The taken id message has no string id | SettingsResourcesTest |
| theme-editor-save-load-path-dropped | The new component loses the old one's `stylesheetPathOnLoad` | SettingsThemeEditorGlueMatchTest |
| theme-editor-string-id-dropped | *Duplicate rule* has no string id | SettingsResourcesTest |
| theme-editor-icon-unmapped | The *Duplicate* icon unmapped | SettingsResourcesTest |
| theme-editor-hint-ignores-edit | The path hint names the saved id, not the one being edited | `edit_offers_the_id_the_path_and_material_you_and_save_writes_them` |
| theme-editor-hint-untrimmed | The path hint keeps the id's spaces | `edit_offers_the_id_the_path_and_material_you_and_save_writes_them` |

### Device check

On `emulator-5556` (`aw_keyboard_ssd`, Android 16), installed with `scripts/install-dev.sh` from
the gate's build of 3cc7ba633, driven through agent-device, on an extension made earlier from
Floris Day. Screenshots are in `build/theme-editor-check2/` (not committed). **Every step
passed:**

1. The day manager's *Edit* opens Compose's extension editor, and the component's *Edit* opens
   the Rust editor: the component box (label, `extension:id`, authors, night flag, path, *Edit*),
   then *Fine tune editor*, *Add rule*, and `@defines` with its note inside its box
   (`01-opened.png`).
2. The meta editor shows its fields in Compose's order. The empty stylesheet path shows
   `stylesheets/floris_day.json`, faded (`02-meta-path-hint.png`). The first look found this
   hint missing; it is the one fix the device check made (4a84245a8). Typing `2` after the id
   changes the hint to `stylesheets/floris_day2.json` before *Apply*
   (`03-meta-hint-follows-id.png`). *Palette style* lists the nine styles with the current one
   chosen (`04-meta-palette-dropdown.png`). After *Apply* the box shows the new label, id and
   path (`05-meta-applied.png`).
3. Deleting a `@defines` property leaves *Deleted Primary color (variant)* and *Undo* where it
   was, inside the box (`06-undo-bar-in-place.png`). *Undo* puts it back in the same place
   (`07-undo-restored.png`).
4. *Add rule* with Key, *Pressed*, codes `32, 10`, modes Symbols then Characters, and shift state
   Caps lock (`08-add-rule-filled.png`) adds a box listing `code = [10, 32]`,
   `mode = [characters, symbols]` and `shiftstate = [caps_lock]`: sorted as the file writes
   them, not in the order they were added (`09-added-rule-attribute-order.png`).
5. *Duplicate* on `Key` *Pressed* `code = [10]` opens *Add rule* with its element, selector and
   code (`10-duplicate-prefilled.png`). *Add* unchanged says *This stylesheet rule is already
   defined.* (`11-duplicate-exists-refused.png`). With code 13 added, it adds
   `code = [10, 13]` with the original's Background and Foreground
   (`12-duplicate-added-with-properties.png`).
6. With the keyboard up on *Try out your setup*, the keyboard shows the draft
   (`13-preview-before-delete.png`). Deleting the space key's *Font size* redraws it at once with
   the larger label of the plain key rule (`14-preview-space-font-size-deleted.png`).
7. Rotating to landscape keeps the draft on the keyboard (`15-rotated-landscape-draft.png`).
   Back in portrait, *Back* asks *Unsaved changes* with *Cancel* and *Discard*
   (`16-back-asks-to-discard.png`). After *Cancel*, the meta, both new rules and the deleted font
   size are as they were, and the undo bar is gone, as designed
   (`17-after-rotation-draft-kept.png`, `18-after-rotation-rules-kept.png`).
8. *Fine tune editor* lists the editing level, the colour representation and *Display keyboard
   after dialogs* (`19-fine-tune.png`). They were left as they were.
9. *Save* closes the editor, and the extension editor shows *Floris Day Edited* and
   `floris_day2` at once (`20-saved-extension-editor-shows-meta.png`). Its *Edit* reopens the
   saved rules from `stylesheets/floris_day2.json` (`21-reopened-saved-rules.png`), and *Back*
   with no changes closes without asking (`22-back-unchanged-closes.png`).
10. The extension's *Save* left an extension editor on screen (`23-extension-saved.png`): the
    popped screen stayed drawn, the fault [fixed before part 3](#the-managers-create-and-edit-crash-fixed). *Back* from
    the copy left behind reached the manager, which lists *Floris Day Edited*
    (`24-after-back-from-stale-copy.png`). Chosen as the day theme, it gives the keyboard the
    larger space label, so the saved extension carries the stylesheet Rust wrote
    (`25-saved-theme-on-keyboard.png`).

Afterwards Floris Day was chosen again (`26-floris-day-restored.png`), and the three test
extensions were deleted through *View details*; their files are gone. The Theme screen shows
*Always day*, Floris Day and Floris Night (`27-theme-screen-restored.png`). Auto-rotate is back
on. agent-device's typing had switched the keyboard to its helper input method; it was set back
to AudioWhisper Keyboard. Media volume stayed at 0.

Not checked on the device, and held by the tests instead: the stylesheet error page, a workspace
the process no longer has, the developer editing level, the rule editor's *Delete*, and
rotating with the rule or meta editor open. P17 owes the phone, TalkBack included.

Found on the device, and not the editor's: with no font size of its own, the space key's label
was wider than the key and was drawn past its left edge, over the emoji key; the right side was
cut at the key's edge (`14-preview-space-font-size-deleted.png`, `25-saved-theme-on-keyboard.png`).
The keyboard drew any stylesheet that way. Fixed on 2026-11-22: the label now ends in an
ellipsis inside the key, as Compose drew it
([rust.md, a label too big for its key](rust.md#a-label-too-big-for-its-key-fixed-2026-11-22)).

## The manager's Create and Edit crash, fixed

Found while checking part 2 and fixed before part 3, on its own. Repro: *Select day theme*,
*Create theme extension*, *Save*, *Save* again; the second *Save* crashed with
`FileNotFoundException` on the closed workspace's `saver/extension.json`.

The guess written down here before was that the screen was stacked twice. It was not: a probe
of the back stack showed one extension editor. The manager opens the Compose fallback with a
browsable link, and Navigation's link handling in the same task clears the stack to the graph
and opens only the link's screen. So *Save* (and *Cancel*, and back from the editor's arrow)
popped the only screen. With the back stack empty no destination change is dispatched, so the
fallback's listener that hands native routes back never ran, and the `NavHost` kept drawing
the popped screen, whose workspace `handleSave` had just closed. That stale screen was the
"twice"; its *Save* was the crash.

The fix is in `FlorisAppActivity`, not in the frozen Compose screen:

* The fallback watches the lifecycle of its top back stack entry and finishes once that entry
  has stopped with no entry left (`currentBackStackEntry` is null) after its links were handled
  (`fallbackDone`). It cannot collect `NavController.currentBackStack`, which is restricted to
  Navigation's own library (lint's RestrictedApi), nor `visibleEntries`, which keeps the popped
  entry for good because the `NavHost` never completes its exit when nothing replaces it. The manager under it refreshes on the result, so a
  saved extension shows at once.
* The fallback no longer hands the browsable link it started with to `handleDeepLink` a second
  time (`fallbackHandles`). The `NavController` handles `activity.intent` when its graph is
  set and saves that it did, so the second pass built a second editor on a second workspace
  (an *Edit* unzipped the extension twice, and the first copy was never closed). After a
  rotation the second pass also replaced the restored editor with a fresh one; that loss of
  unsaved work follows from Navigation's code and was not reproduced on the old build. Files
  and shares the fallback starts with, and every later link, are taken as before.

Kotlin, because it is the activity's own intent and Compose navigation: the two decisions are
plain top-level functions so the JVM can test them. `FlorisAppActivity.kt` grew from 249 to 254
counted lines; the ledger's budget was raised by hand with that reason.

Tests: `ComposeFallbackLinksTest` (5) checks that every later intent is taken exactly as the old
`onNewIntent` took it, that the starting link is left to the `NavController`, that starting
files and shares are still imported, and that the fallback leaves only when its links are
handled and its stack is empty. Mutations, each caught by it: `fallback-relinks-start`,
`fallback-stays-after-save`, `fallback-leaves-before-graph`. They test the decisions, not their
call sites; dropping the effect that calls `fallbackDone` is caught only on the device.

Device check on `emulator-5556`: *Create theme extension* then *Save* goes back to *Select day
theme*, so there is no second *Save* (`01-create-save-returns.png`); *Edit* opens one editor;
with a bundled theme added, a rotation to landscape keeps it (`02-edit-rotated-keeps-theme.png`);
*Cancel* and then *Edit*, *Save* each go back to the manager (`03-edit-save-returns.png`). No
fatal exception in the log. Screenshots are under `build/part3-device/`, not committed. The
phone check is P20.

## Part 3: the property editor (done)

Rust now edits every property value in the theme editor. It also adds and moves a font's source
sets, and edits a key rule's codes one at a time, with *record a key*. With that, nothing routes
to the Compose editor or imports from it, and its 14 files are deleted (below).

### How it is reached

| Route | What it is |
| --- | --- |
| `settings/theme/editor/property/add` | *Add property* (**+**), the last button on a rule's title line. On `@font` it is *Add source set*, and each source set's heading has its own **+**. |
| `settings/theme/editor/property` | A property tapped in the rules list (`EditPropertyDialog`). |
| `settings/theme/editor/code/add` and `settings/theme/editor/code` | The code editor (`EditCodeValueDialog`), from the rule editor's *Add* chip or a code's chip. |

### What it shows

1. **The rules list** shows each value as Compose did. A static colour is written in fine-tune's
   colour representation (hex, rgb or hsv, with its alpha), as `formatColor(withAlpha = true)`
   wrote it. Before *Delete* each value has its preview icon, as `SnyggValueIcon` drew it:
   * a swatch for a colour, checkered under a translucent one, and a Material You colour's
     swatch;
   * a shape's outline;
   * the Material icon for a font, a style, a weight, padding, a dp or sp size, text alignment,
     decoration, overflow, an image, content scale or yes/no;
   * for a variable, its link over the small icon of the value it names.

   Tapping the icon opens the row, as tapping the row does.
2. **The property editor**, top to bottom:
   * the error line;
   * *Search properties*, only while adding to an element rule;
   * *Name*: the properties the element takes at the editor level, fixed while editing. In
     `@defines` it is a field, checked as Compose checked a variable's name;
   * *Value encoder*: the kinds the property takes;
   * the value;
   * *Show keyboard*;
   * *Cancel*, then *Add* or *Apply*.

   Editing has *Delete* in the top bar, and the deleted property leaves part 2's undo bar. A
   font's `src` has no *Delete*.
3. **Each value kind:**
   * a variable, chosen from `@defines`' names;
   * a static colour: its swatch and text. Tapping it opens step 4's colour picker with an alpha
     bar, and *OK* sets the value;
   * a Material You light or dark colour, chosen by name, each name with its swatch in this
     device's palette;
   * a font family: a generic family or one of the stylesheet's `@font` names. Font style and
     font weight (nine weights) are chosen too;
   * padding: four sides, with *Apply for all sides*;
   * a shape: rectangle, circle, or cut or rounded corners in dp or in whole percent, each corner
     on its own;
   * a dp, sp or percentage size, or a line count, typed beside its unit;
   * the keyword kinds (content scale, text alignment, decoration, overflow and the rest), chosen
     from their keywords;
   * an image or a font file, chosen from the workspace's files by URI. A note says when there
     are none.
4. **The contrast note.** While a rule's `foreground` or `background` is edited, the editor
   checks the pair before *Apply*. Under 4.5:1 a note shows under the value: *Low contrast:
   2.1:1, below 4.5:1*. It is a note, not an error, so *Apply* still works.
5. **Source sets** (`@font`). Each *Source set* heading has *Move up*, *Move down*, *Delete* and
   **+**. Moving stops at the ends, and a disabled button is skipped in the focus order. *Delete*
   leaves an undo bar where the set was. **+** adds a property to that set. A set without `src`
   shows the *Errors* box, *Required property "src" does not exist*, under its heading.
6. **The code editor.** The rule editor shows a key rule's codes as chips, followed by an *Add*
   chip. A code's chip opens the code editor on it, and *Add* opens it empty. Its rows:
   * the key data preview, as `TextKeyDataPreviewBox` drew it: the key's icon or label in a
     36 dp box, then its Unicode or key name over its kind;
   * the code field, with *Record a key* at its end and the recorder's message under it;
   * the help text and its two links.

   The field takes several comma-separated codes at once. *Apply* refuses an empty code, a code
   out of range, or one the rule already has, and says why. *Delete* in the top bar removes the
   code.

### Data path

```
property editor ─ each name, encoder or value change the stylesheet can hold
  ─▶ ["themeEditorPreview", ws, index, the draft with that value] ─▶ ThemeManager.previewThemeInfo
  Apply ─▶ the draft's rules, as part 2's edits;  Cancel, Back ─▶ the draft previewed again
SettingsThemeEditor ─ THEME_EDITOR report (op 28) + COLOR_NAMES, LIGHT_COLORS, DARK_COLORS ─▶ swatches, contrast
code editor ─ Record a key ─▶ ["themeCodeRecord"] ─▶ SettingsKeyRecorder takes one key-up
                                                  ─▶ THEME_CODE (op 33) key | stopped ─▶ the field
             preview      ─▶ ["themeCodeDescribe", code] ─▶ ICU's name, mark flag, the key's icon
                                                  ─▶ THEME_CODE (op 33) described ─▶ the preview
```

* **Rust does the logic.** It works out:
  * the properties an element takes at each level, and the kinds each property takes, from
    `snygg::value_spec`;
  * every value's editing, checks and text, through `snygg::value_codec`. A value must read back
    as it was written;
  * the colour formats, the icon each value gets, and the contrast ratio;
  * the source sets, the codes' ranges and refusals, and the preview's label and kind, from the
    `keys` crate.

  A static colour is written as `rgba(r,g,b,a)`, as Compose's editor and `snygg` wrote it.
* **The palette** is the one the keyboard draws the component in. Kotlin builds it from the
  keyboard's accent (the chosen one, else the system's) under the component's own palette style,
  contrast and spec. It uses the same `SnyggPalette` call as the keyboard, and sends each colour
  name with its light and dark ARGB in the report. A report without a palette gives no swatches.
* **The contrast check** looks at the property being edited against the other one of the pair,
  in the same rule and property set. Nothing across rules, nothing inherited, no borders or
  shadows. Variables are followed one level through `@defines`, as the keyboard does, and dynamic
  colours come from the palette. A translucent foreground is laid over the background. A
  translucent background is laid over the plain `keyboard` rule's background, when that is an
  opaque colour. Otherwise, or in the `keyboard` rule itself, the check lays it over black and
  over white and keeps the worse. The ratio is rounded down to one decimal, so `#777777` on white
  (4.478) shows 4.4:1, never 4.5. The threshold is WCAG AA for normal text: key labels and hints
  are drawn below WCAG's large-text size, so 3:1 would pass hint labels that fail.
* **Codes** use op 33, `THEME_CODE`. Its effects are `themeCodeRecord`, `themeCodeStop` and
  `themeCodeDescribe`, and its reports `key`, `stopped` and `described`. They are generated for
  Kotlin as `SettingsUiThemeCode*`. Recording follows Compose's rule: it needs this app's keyboard
  enabled and selected. When the keyboard is not enabled, the message stays under the field and
  the input method settings open. When it is enabled but not selected, the input method picker
  opens. Rust asks Kotlin to describe each code once. It never asks for the unspecified key, and it
  ignores an answer about a code no longer shown.
* **The saved state** is version 10 (8 after part 2). The open property editor and code editor
  come back with what they hold: name, encoder, value, search, the code being typed and its
  error. Recording does not come back. Compose held it in plain `remember`, nothing listens after
  recreation, and a key pressed later is not the one that was asked for.

### What Kotlin does

One file is new, and it must be Kotlin:

* **`settingsui/SettingsKeyRecorder.kt`** (90 counted lines). The key-up exists only in the
  running input method's `InputEventDispatcher.keyEventReceiver`, in this process. Unicode
  names need `android.icu`, a key's icon needs the keyboard's `ComputingEvaluator` and icon
  paths, and stopping needs the activity's lifecycle. `KeyRecording` swaps the receiver slot for
  one key-up, and gives back the previous receiver only while it still holds the slot. It stops
  and reports `stopped` on `ON_STOP`. It answers `DESCRIBE` for the one code shown. A code goes
  nowhere but back to Rust, and nothing logs it.

Two files hold what other code still imported from the deleted package, moved unchanged:

* **`app/ThemeEditorPrefs.kt`** (24): `PrettyPrintConfig`, `ColorPreferenceSerializer`,
  `DisplayKbdAfterDialogs` and `SnyggLevel`, which the Jetpref preference model, the extension
  editor and the editor's glue use.
* **`app/ext/ExtensionDialogProperty.kt`** (31, Compose): `DialogProperty`, which only the
  extension editor uses. It goes with that editor at step 10. It is its own file because
  `ExtensionEditScreen` is already over the file limit.

These files grow or shrink:

* `SettingsThemeEditor` (95 to 109): reads the keyboard's accent preference and the system
  accent, and builds the palette for the report.
* `lib/snygg` `SnyggPalette` (16 to 27) and `SnyggUi` (303 to 285): materialkolor's
  `dynamicColorScheme` is a Kotlin library. The keyboard and the editor now share one call.
* `SettingsIcons` (53 to 67): the 18 value icons and *Record a key*'s icon, mapped to Material
  vectors.
* `SettingsThemeEditorStringIds` (204 to 285): the property editor's, the code editor's and the
  contrast note's strings, and the corners chip's (below).
* `RustSettingsView` (268 to 272): routes the three code effects to the recorder, sends its
  reports back through op 33, and releases it with the view.

`strings.xml` gains 14 strings: the padding sides and *Apply for all sides*, *Show keyboard*,
*Search properties*, *Add source set*, *Move up* and *Move down*, the Errors box's title and
message, the contrast note, and *Record a key* for TalkBack.

### Improvements

Each one is new over Compose and has its own test.

| # | Improvement | Why | Test |
| --- | --- | --- | --- |
| 1 | **A live preview.** The keyboard shows the value being edited before *Apply*. *Cancel* and *Back* put the draft back. | Compose's preview followed the workspace, which changed only once the dialog was confirmed. | `each_value_that_can_be_written_is_previewed_before_apply`, `back_previews_the_draft_and_apply_previews_what_it_applied` |
| 2 | **A contrast warning** under 4.5:1, with the ratio, rounded down. | A theme author cannot always tell that hint labels will be hard to read, and Compose did not check. | `contrast.rs` (5 tests), among them `the_ratio_is_wcag_2s_and_shows_rounded_down` (21:1, 1:1, about 4.48:1) and `the_note_comes_and_goes_with_the_value_and_apply_still_takes_it` |
| 3 | **Search** in the property names, by label or written name, ignoring case. | At the developer level the list holds dozens of names. Compose had no search. | `search_narrows_the_names_by_label_or_name_ignoring_case` |
| 4 | **A name the set already has is refused**, with why. | Compose's check never fired, and adding overwrote the property silently. | `apply_refuses_a_missing_name_encoder_or_value_and_a_name_the_set_has` |
| 5 | **A value must read back as written.** A cut corner in fractional dp is refused. | Compose wrote it, and the saved stylesheet then read back as something else. | snygg `a_cut_corner_in_fractional_dp_is_written_but_does_not_read_back` |
| 6 | **A variable's name is trimmed.** | A stray space made a second variable. | `a_variable_is_named_in_a_field_checked_as_compose_checked_it` |
| 7 | **A deleted source set can be undone**, from where it was. | Compose deleted it at once. | `deleting_a_set_leaves_undo_where_it_was_and_undo_puts_it_back` |
| 8 | **Disabled *Move up* and *Move down* are skipped** in the focus order. | TalkBack and keyboard users no longer land on a button that does nothing. | `the_focus_order_takes_each_headings_enabled_buttons` |
| 9 | **An open property or code editor survives recreation**, with what it holds. | Rotating while editing a value lost it. | `an_open_property_editor_survives_recreation_with_what_it_holds`, `an_open_code_editor_survives_recreation_with_what_it_holds_but_not_recording` |
| 10 | **Material You swatches use the component's own palette flags**, not materialkolor's defaults. | They then match what the keyboard draws. | JVM `SettingsThemeEditorPaletteMatchTest` |
| 11 | **Yes shows a ticked box.** | Compose drew a bold-text icon for it. | `each_kind_has_the_icon_compose_drew` |
| 12 | **A variable that names itself ends.** | Compose's icon followed the chain without end on a loop. | `a_variable_naming_a_variable_shows_two_links_and_a_loop_ends` |
| 13 | **Several codes typed at once**, comma-separated. | Typing a rule's codes stays as fast as part 2's single field. | `several_codes_go_in_at_once_and_an_unchanged_code_just_closes` |
| 14 | **A recorded key replaces only the entry being typed.** | The other codes typed in the field stay. | `a_recorded_key_sets_the_code_being_typed_and_stops_recording` |
| 15 | **The "needs our keyboard" message stays under the field**, and the input method settings open when the keyboard is not enabled. | Compose toasted, and always opened the picker, which cannot enable a keyboard. | `recording_needs_this_apps_keyboard_enabled_and_selected` |
| 16 | ***Record a key* has a TalkBack label and a checked state**, and the placeholder is the field's TalkBack hint. | Compose's button was an unlabelled icon. | `a_recorded_key_sets_the_code_being_typed_and_stops_recording`, `a_rules_codes_are_chips_that_open_the_editor_and_add_opens_it_empty`, JVM `aRecordedKeyAndAStopCrossAsTheRendererReadsThem` |
| 17 | **Recording stops when the activity stops**, and a key receiver installed after the recorder's is left alone. | A key typed in another app is never taken, and the keyboard is never left without its receiver. | `recording_stops_when_tapped_again_typed_over_left_or_stopped_by_kotlin`, JVM `stoppingGivesTheReceiverBackOnlyWhileItIsHeld` |
| 18 | **A code is described once**, and an answer about a code no longer shown is ignored. | The preview never shows the previous key's icon. | `the_preview_of_internal_codes_with_an_icon_a_label_or_neither`, JVM `thePreviewShowsWhatTextKeyDataPreviewBoxShowedForEveryKeyAndSomeCharacters` |

Rule filtering, proposed after part 2, was left out again. The shipped stylesheets have tens of
rules, already grouped by element.

### Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| The property editor and the code editor are screens, not dialogs. | As part 2's editors: the settings renderer's editors are screens with a bottom bar. |
| The colour picker is an *OK*/*Cancel* dialog with only its *Custom* page (no presets), not inline. There is no separate text field, as in Compose. | It is step 4's picker, shared with the settings rows, which keep theirs without the alpha bar. |
| The contrast note does not show while dragging inside the picker. It shows once *OK* sets the value. | The picker is the shared dialog, and the note belongs to the editor. |
| The Material You lists show a swatch beside each name. | Compose showed swatches only after choosing. |
| A percentage is a whole percent. | The v2 spec cannot reach a fractional one. |
| Font weight lists its nine words once each. | The stylesheet always writes the word. Compose listed all 18 keys, each weight twice. |
| *Add property* is last on a rule's title line, in the primary colour. | It sits beside part 2's *Duplicate* and *Edit*. |
| A file is labelled by its URI. | The value the stylesheet holds. |
| The *Errors* box strings are new, and the source set heading is plain text. | Compose's box had no strings of its own to reuse. |
| A value's error shows as it is typed. The name's and encoder's show after *Apply*. | The value check is cheap and helps while typing. The others need a whole form. |
| *Show keyboard* focuses a hidden field. | The settings renderer has no preview field in the form. |
| No LRI/PDI isolates around inserted names. | The renderer lays out one direction. |
| In the rules list the icon sits in the slot before *Delete*, and a tap on it opens the row. | The renderer's list rows end in their buttons. |
| A variable's small icon stops at the link. | Compose followed a chain of variables. |
| Max lines, percentages, `inherit` and undefined show no icon. | Compose drew none for them. |
| Shape corners too big for the icon shrink together. | As Skia fits them. |
| After a meta change, the swatches use the report's palette until *Save*. | The palette comes from Kotlin with the report. |
| While recording, the field keeps its text and *Recording…* shows under it. There are no started or stopped toasts, and the record icon is a static red. | The renderer has no toasts or blinking icons, and the message stays where it is read. |
| The preview shows the last comma-separated entry. | That is the code being typed. |
| The code editor's help and links are always shown. | Compose hid them behind a title toggle. |
| A lone surrogate previews as U+FFFD. | It is not a character. |

### Tests

* **Rust, `settings_ui_theme_editor`**: 94 tests (28 after part 2), in `property.rs`,
  `values.rs`, `preview.rs`, `sets.rs`, `colour.rs`, `dynamic.rs`, `icons.rs`, `contrast.rs`,
  `codes.rs` and `saved.rs`, with the part 2 files. They cover:
  * *Add property* on every title line, and a tapped property opening with its name fixed;
  * every refusal, *Delete* with undo and *Cancel*;
  * search;
  * each value kind's editing and its written text;
  * the preview before *Apply*;
  * source sets, their moves, undo, **+** and the *Errors* box;
  * the picker with alpha, the representations and the swatches;
  * every kind's icon, shapes and variables;
  * the contrast measure and note;
  * the code editor's chips, refusals, recording, preview and links;
  * the saved state for each editor;
  * a duplicate's sets, below.
* **Rust, `settings_ui_color.rs` and `settings_ui_other_theme.rs`**: the settings rows' picker is
  unchanged.
* **Rust, `snygg/tests/value_edit.rs`** (7 tests): every encoder's default writes and reads back;
  chosen names, colours, line counts, weights, sizes, sides and corners write as Compose wrote
  them; and a fractional cut corner does not read back.
* **JVM**:
  * `SettingsThemeEditorMatchTest` (7 tests) adds two match tests: properties and codes edited in
    Rust are written exactly as Compose's dialogs wrote them. Its references to the deleted
    `CustomRuleComparator` and `PropertyNameMap` now come from `ReferenceThemeEditor`, a test-only
    transcription of them as they stood when they were deleted.
  * `SettingsThemeEditorColourMatchTest` (2): Rust's `withAlpha` text equals Compose's
    `formatColor` for a spread of colours and alphas in each representation, and the picker takes
    what Compose's parser took.
  * `SettingsThemeEditorPaletteMatchTest` (2): the report carries the keyboard's palette under the
    component's flags, and the accent is the chosen one, else the system's.
  * `SettingsThemeCodeMatchTest` (3): the preview shows what `TextKeyDataPreviewBox` showed for
    every predefined key and a spread of characters, and the recorder's reports cross as Rust
    reads them.
  * `SettingsKeyRecorderTest` (4): one key-up taken, a key-down, repeat or cancel not taken, the
    receiver given back only while held, and an empty slot left empty.
  * The settings match tests name saved state version 10. `SettingsResourcesTest` checks every
    new string and icon.
* **The gate**, on the code of f3121345c, rebased on origin/main f1fdf3f34 (main had not moved
  at the last `git fetch`): `test`, `lintDebug`, `:lib:rust:check` (clippy, fmt and the rustdoc
  check), `checkNativeDeclarations`, `checkJvmLedger`, `assembleDebug` and the instrumented
  tests' compile all pass. 985 JVM tests in 240 files, 0 failures, 0 skipped; 1,298 cargo test
  results passed, 0 failed, 0 ignored; 389 natives, 0 hand-written, 0 mismatched (op 33 rides the
  settings native, so the count is unchanged); the ledger matches 428 files and 428 rows (14
  deleted, 3 added). Lint reports only the four allowed IDs.

### Fixed on the way

Two faults in earlier work, found while finishing part 3. Each is its own commit with its own
test and mutation case.

* **The shape editor's *Apply for all corners* chip had no label** (b459379b7). Its string key
  was missing from the property editor's key list, so Kotlin was never asked for it. Lint found
  the string unused once the Compose editor, its only other user, was deleted. The key list's
  test in `settings_ui_screens.rs` read only the files directly in `src/settings_ui`; it now
  reads every file under it. Mutation `theme-editor-corners-key-unlisted`.
* **A duplicate changed to another kind of rule kept its source's sets** (f3121345c). Found on
  the emulator (below): *Duplicate* on a key rule, with the element changed to `@font` before
  *Add*, made an `@font` rule holding the key's properties as one set, with *Add property* where
  a font has *Add source set*, and the *Errors* box. Part 2's *Duplicate* copied the source's
  sets whatever rule it added. Now every new rule starts empty in the kind it takes, as
  `updateRule` added it, and a duplicate takes its source's sets only when they are that kind:
  a key duplicated as Window keeps its properties. Test
  `a_duplicate_takes_its_sources_sets_only_where_its_element_takes_that_kind`; mutation
  `theme-editor-duplicate-kind-ignored`.

### Mutations

`scripts/settings-ui-mutations.py` has 133 new cases for part 3 (now in `property_editor.py`,
`property_values.py` and `theme_codes.py` under `scripts/settings_ui_mutation_cases/`): 131 for the property and code
editors, and one for each fix above. 115 of the 131 are caught by the Rust editor tests. The
other 16 are caught by JVM tests: the Kotlin palette, recorder and preview, the string and icon
maps, and the written stylesheet compared with Compose's. Four of part 2's cases were
retargeted: `theme-editor-undo-not-in-place` and `theme-editor-internal-codes-refused` moved with
their code, and `theme-editor-duplicate-empty` and `theme-editor-new-font-single` now aim at the
duplicate fix's `new_sets`. Each case's text matches its source exactly once. **Every case was
caught by a failing assertion**, and the restored code passed.

The 131, the corners case, the two moved part 2 cases and the crash fix's three ran together on
the code of a186ed5cd, in `build/mutations-part3-final/` (137 cases, `restored_exit` 0). The
duplicate fix's case and the two it retargeted ran on f3121345c, in
`build/mutations-duplicate-kind/`, whose restored run passed too. The script was then 1,016 counted
lines (675 before part 3), over the 500-line file limit. Step 10 split it: the runner keeps
its command line and output, and the cases live in `scripts/settings_ui_mutation_cases/`, one
module per screen, each under the limit ([the split](stage4-extensions.md#the-mutation-script-split)).

| Case | What it breaks | Caught by |
| --- | --- | --- |
| `theme-editor-property-exists-unchecked` | A property whose name already exists in the set is added | `property::apply_refuses_a_missing_name_encoder_or_value_and_a_name_the_set_has`, `sets::a_sets_add_adds_a_property_to_that_set` |
| `theme-editor-property-edit-clashes-itself` | Editing a property without renaming it is refused as a clash with itself | `values::an_edited_value_shows_in_its_fields_as_written` |
| `theme-editor-value-not-read-back` | A value the stylesheet cannot read back is accepted | `values::a_custom_font_is_one_of_the_stylesheets_fonts` and 3 more |
| `theme-editor-property-name-untrimmed` | A property's name kept with its surrounding spaces | `property::a_variable_is_named_in_a_field_checked_as_compose_checked_it` |
| `theme-editor-rename-keeps-encoder` | Renaming a property keeps its encoder and value | `property::apply_refuses_a_missing_name_encoder_or_value_and_a_name_the_set_has` |
| `theme-editor-chip-copies-first` | *Apply for all sides* copies the first side, not the one typed last | `values::padding_takes_four_sides_and_the_chip_copies_the_last_to_all` |
| `theme-editor-search-when-editing` | The property search line shows while editing an existing property | `property::search_narrows_the_names_by_label_or_name_ignoring_case` |
| `theme-editor-search-by-label-only` | Property search matches labels only, not property names | `property::search_narrows_the_names_by_label_or_name_ignoring_case` |
| `theme-editor-search-untrimmed` | Spaces around a property search stop it finding anything | `property::search_narrows_the_names_by_label_or_name_ignoring_case` |
| `theme-editor-font-add-opens-editor` | *Add* on a font rule opens a property editor, not a new source set | `sets::a_sets_add_adds_a_property_to_that_set` and 5 more |
| `theme-editor-set-up-moves-down` | *Up* on a source set moves it down | `sets::a_sets_heading_moves_it_up_and_down_but_not_past_the_ends` |
| `theme-editor-property-delete-ignored` | *Delete* in the property editor closes it without removing the property | `property::delete_in_the_top_bar_goes_with_undo_and_a_fonts_src_has_none` |
| `theme-editor-preview-without-value` | The theme preview leaves out the property being edited | `colour::ok_sets_the_value_and_previews_it_and_apply_writes_it` and 3 more |
| `theme-editor-close-keeps-preview` | Closing the property editor keeps its value on the keyboard, not the draft | `preview::back_previews_the_draft_and_apply_previews_what_it_applied`, `preview::each_value_that_can_be_written_is_previewed_before_apply` |
| `theme-editor-src-deletable-in-editor` | A font's `src` gets a *Delete* in the property editor's top bar | `property::delete_in_the_top_bar_goes_with_undo_and_a_fonts_src_has_none` |
| `theme-editor-show-keyboard-inert` | The *Show keyboard* row does nothing when tapped | `preview::show_keyboard_focuses_the_hidden_editor_and_back_hides_it_first` |
| `theme-editor-font-weight-doubled` | The font weight dropdown lists each of its nine words twice | `values::keyword_kinds_are_chosen_from_their_keywords` |
| `theme-editor-uri-slash-encoded` | Characters a URI must keep, such as `/`, are percent-encoded | `sets::a_sets_add_adds_a_property_to_that_set` and 3 more |
| `theme-editor-percent-over-100` | A percentage from 101 to 1000 is accepted instead of refused | `values::corners_in_dp_or_whole_percents_are_checked_as_compose_checked_them` |
| `theme-editor-set-up-always-enabled` | The first source set's *Up* is never disabled | `sets::a_sets_heading_moves_it_up_and_down_but_not_past_the_ends` |
| `theme-editor-set-undo-at-end` | *Undo* of a deleted source set puts it at the end of its rule | `sets::deleting_a_set_leaves_undo_where_it_was_and_undo_puts_it_back` |
| `theme-editor-errors-box-inverted` | The errors box lists the required properties that are present | `sets::a_sets_add_adds_a_property_to_that_set` and 3 more |
| `theme-editor-set-focus-disabled-up` | The first source set's disabled *Up* stays in the focus order | `sets::the_focus_order_takes_each_headings_enabled_buttons` |
| `theme-editor-variable-value-written` | A variable in the rules list is written as the stylesheet writes it, not by its name | `rules::a_deleted_property_leaves_undo_where_it_was_and_undo_puts_it_back`, `rules::the_rules_come_in_compare_order_with_their_properties_named_and_written_out` |
| `theme-editor-saved-property-unheld` | A property editor comes back after recreation with no property held | `saved::a_property_editor_saved_without_its_form_comes_back_on_the_rules` |
| `theme-editor-saved-property-dropped` | A property editor open at save time does not come back | `saved::an_open_property_editor_survives_recreation_with_what_it_holds` |
| `theme-editor-property-string-id-dropped` | The *Apply for all sides* label is missing from the Kotlin string map | `SettingsResourcesTest.stringIdsAreExactlyTheNativeKeyList` |
| `theme-editor-cut-percent-rounded` | A cut-corner shape in percent is written as rounded corners | `SettingsThemeEditorMatchTest.propertiesEditedInRustAreWrittenAsComposesDialogWroteThem` |
| `colour-alpha-hundredths-truncated` | Alpha in colour text is truncated, not rounded, to hundredths | `formats_with_alpha_as_format_color_with_alpha_does` |
| `colour-alpha-zeros-kept` | Alpha in colour text keeps trailing zeros, such as `0.50` | `formats_with_alpha_as_format_color_with_alpha_does` |
| `colour-hex-alpha-first` | An 8-digit hex colour takes its first two digits as alpha | `parses_an_alpha_only_with_alpha` |
| `colour-alpha-pattern-loose` | Colour alpha accepts `1.00` and `0.`, which the pattern refuses | `parses_an_alpha_only_with_alpha` |
| `colour-alpha-without-flag` | A colour with an alpha value is accepted where the field takes none | `parses_an_alpha_only_with_alpha`, `parses_rgb_and_hsv_with_jetpref_number_rules` |
| `colour-accent-takes-alpha` | A colour picker without an alpha bar accepts alpha in typed text | `the_accent_picker_keeps_its_square_plane_and_takes_no_alpha` |
| `colour-picker-set-drops-alpha` | The alpha bar stays fully opaque when a colour is set into the picker | `colour::ok_sets_the_value_and_previews_it_and_apply_writes_it` |
| `colour-theme-ok-stores-pref` | *OK* in a theme colour's picker is ignored, while a plain picker takes it | `colour::a_colour_typed_with_its_alpha_in_any_form_is_written_as_compose_wrote_it` and 9 more |
| `colour-theme-plane-square` | The alpha colour plane is square, not 1.5 times as wide as tall | `colour::the_theme_picker_is_the_custom_page_with_a_wide_plane_and_no_page_switch` |
| `colour-alpha-bar-in-editor` | The alpha bar shows its thumb and touch area while the colour is typed | `colour::the_alpha_bar_follows_the_finger_outside_the_editor` |
| `colour-alpha-thumb-translucent` | The alpha bar's thumb is drawn translucent, not over white | `colour::the_alpha_bar_follows_the_finger_outside_the_editor` |
| `colour-theme-presets-button` | A theme colour's picker shows a *Presets* button it has no page for | `colour::a_colour_typed_with_its_alpha_in_any_form_is_written_as_compose_wrote_it` and 12 more |
| `colour-stale-theme-picker` | A stale alpha picker is taken for the property editor's while it edits a non-colour value | `colour::a_picked_colour_leaves_no_picker_on_the_next_value` |
| `colour-picked-not-previewed` | A colour picked for a property is not shown on the keyboard | `colour::ok_sets_the_value_and_previews_it_and_apply_writes_it`, `preview::each_value_that_can_be_written_is_previewed_before_apply` |
| `colour-representation-ignored` | The saved *RGB* colour representation falls back to hex | `colour::the_value_row_and_the_rules_list_show_each_representation` |
| `colour-rules-list-written` | A static colour in the rules list is written as the stylesheet writes it, not in the colour representation | `colour::the_value_row_and_the_rules_list_show_each_representation` and 4 more |
| `colour-swatch-always-checkered` | An opaque colour swatch is drawn over a checkerboard | `colour::the_value_row_shows_its_swatch_after_the_colour` |
| `dynamic-dark-from-light` | A dynamic colour's dark value is taken from the light palette | `dynamic::each_name_in_the_lists_has_its_swatch_from_the_palette` and 2 more |
| `dynamic-palette-lengths-unchecked` | A palette whose light or dark list has the wrong length is accepted | `dynamic::the_report_carries_the_palette_or_none` |
| `dynamic-list-without-swatch` | The rows of a dynamic colour's list lose their colour swatch | `dynamic::each_name_in_the_lists_has_its_swatch_from_the_palette` |
| `dynamic-list-name-under-swatch` | A dynamic colour's list name runs under its 24 dp swatch | `dynamic::each_name_in_the_lists_has_its_swatch_from_the_palette` |
| `dynamic-value-row-without-swatch` | A dynamic colour's value row loses its swatch and becomes a plain dropdown | `dynamic::the_value_row_and_the_rules_list_show_the_colour_the_palette_gives` |
| `dynamic-icon-without-palette` | A dynamic colour with no palette is given a preview icon | `dynamic::without_a_palette_a_dynamic_colour_has_no_swatch` |
| `rules-list-without-icon` | A property row in the rules list loses its value icon | `dynamic::the_value_row_and_the_rules_list_show_the_colour_the_palette_gives` and 3 more |
| `rules-list-icon-slot-unkept` | A property row keeps no button slot for its value icon | `dynamic::without_a_palette_a_dynamic_colour_has_no_swatch` |
| `rules-list-icon-untappable` | A property row's value icon is covered by the row and cannot be tapped | `dynamic::the_value_row_and_the_rules_list_show_the_colour_the_palette_gives` |
| `palette-default-style-in-kotlin` | The Material You palette ignores the chosen palette style | `SettingsThemeEditorPaletteMatchTest.theReportCarriesTheKeyboardsPaletteUnderTheComponentsFlags` |
| `palette-dark-as-light-in-kotlin` | The light colours are taken from the dark half of the palette | `SettingsThemeEditorPaletteMatchTest.theReportCarriesTheKeyboardsPaletteUnderTheComponentsFlags` |
| `accent-choice-ignored-in-kotlin` | The keyboard's palette ignores a chosen accent and uses the system's | `SettingsThemeEditorPaletteMatchTest.theAccentIsTheChosenOneOrTheSystemsWhenNoneIs` |
| `icon-yes-as-bold` | A *Yes* value's icon is drawn with the bold glyph | `icons::each_kind_has_the_icon_compose_drew`, `icons::the_rules_list_shows_each_kinds_icon_before_delete` |
| `icon-end-aligned-centre` | An end-aligned text value's icon shows centre alignment | `icons::each_kind_has_the_icon_compose_drew`, `icons::the_rules_list_shows_each_kinds_icon_before_delete` |
| `icon-glyph-ink` | Glyph value icons are inked with the wrong theme colour | `icons::the_rules_list_shows_each_kinds_icon_before_delete` |
| `shape-dp-not-tripled` | A corner in dp is drawn at a third of its size in the shape icon | `icons::a_shape_is_outlined_as_always_percent_shape_makes_it` |
| `shape-corners-unfitted` | A shape icon's corners overflow the icon when they add up past its side | `icons::a_shape_is_outlined_as_always_percent_shape_makes_it` |
| `shape-cut-as-round` | A *Cut* corner shape icon is drawn with rounded corners | `icons::a_shape_is_outlined_as_always_percent_shape_makes_it` |
| `shape-cut-line-thin` | The inner edge of a *Cut* corner shape icon is not inset to match its outline | `icons::a_shape_is_outlined_as_always_percent_shape_makes_it` |
| `shape-icon-uncentred` | A shape icon sits in the corner of its box, not centred | `icons::the_rules_list_shows_each_kinds_icon_before_delete` |
| `variable-link-not-raised` | A variable's link and linked value icon are not raised 2 dp | `icons::a_variable_naming_a_variable_shows_two_links_and_a_loop_ends`, `icons::a_variable_shows_its_link_over_the_small_icon_of_its_value` |
| `variable-link-unbacked` | A variable's link icon loses its rounded backing box | `icons::a_variable_shows_its_link_over_the_small_icon_of_its_value` |
| `variable-small-form-follows` | A variable's small icon shows the linked value, not just the link | `icons::a_variable_naming_a_variable_shows_two_links_and_a_loop_ends` |
| `variable-undefined-unlinked` | An undefined variable shows no link icon | `icons::a_variable_naming_a_variable_shows_two_links_and_a_loop_ends` and 2 more |
| `variable-first-define` | A variable's value comes from the first `@defines` property, not the one it names | `contrast::only_a_rules_own_pair_is_checked_and_variables_one_level` and 3 more |
| `value-row-variable-unlinked` | A variable value row with no value falls back to a plain dropdown | `icons::the_value_row_shows_a_variables_link_even_before_it_names_one` |
| `icon-unmapped-in-kotlin` | The *open in full* icon is missing from the Kotlin icon map | `SettingsResourcesTest.everyIconTheNativeScreensNameIsMapped` |
| `contrast-always-low` | Every colour pair gets a low-contrast note, whatever its ratio | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards`, `contrast::the_note_comes_and_goes_with_the_value_and_apply_still_takes_it` |
| `contrast-flare-changed` | Contrast ratios add 0.1 to each luminance, not WCAG's 0.05 | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards` and 4 more |
| `contrast-shown-rounded` | A ratio just under 4.5 is shown as 4.5 instead of rounded down | `contrast::the_note_comes_and_goes_with_the_value_and_apply_still_takes_it`, `contrast::the_ratio_is_wcag_2s_and_shows_rounded_down` |
| `contrast-foreground-unveiled` | A translucent foreground's contrast ignores the background showing through | `contrast::the_ratio_is_wcag_2s_and_shows_rounded_down` |
| `contrast-background-unveiled` | A translucent background's contrast ignores the colour beneath it | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards`, `contrast::the_ratio_is_wcag_2s_and_shows_rounded_down` |
| `contrast-keyboard-ignored` | A translucent colour's contrast ignores the keyboard background beneath it | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards` |
| `contrast-keyboard-own-known` | The keyboard's own colours are measured over the keyboard background | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards` |
| `contrast-sides-unswapped` | A foreground side's contrast swaps which colour is drawn on which | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards` |
| `contrast-variable-unread` | A variable colour gets no contrast note | `contrast::only_a_rules_own_pair_is_checked_and_variables_one_level` |
| `contrast-dynamic-unread` | A dynamic colour gets no contrast note | `contrast::a_dynamic_colour_is_checked_as_the_palette_has_it` |
| `contrast-note-as-error` | The low-contrast note is drawn in the error colour | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards` and 3 more |
| `contrast-note-unpainted` | The low-contrast note is not painted under the property's value | `contrast::a_background_is_checked_against_its_rules_foreground_over_the_keyboards` and 3 more |
| `contrast-string-unmapped-in-kotlin` | The low-contrast note text is missing from the Kotlin string map | `SettingsResourcesTest.stringIdsAreExactlyTheNativeKeyList` |
| `theme-code-exists-allowed` | A code already in the rule is accepted instead of refused as existing | `codes::every_refusal_keeps_the_editor_open_with_why`, `saved::an_open_code_editor_survives_recreation_with_what_it_holds_but_not_recording` |
| `theme-code-edit-clashes-itself` | Editing a code without changing it is refused as already existing | `codes::several_codes_go_in_at_once_and_an_unchanged_code_just_closes` |
| `theme-code-characters-unbounded` | A character code above `CHARACTERS_MAX` is accepted | `codes::every_refusal_keeps_the_editor_open_with_why`, `form::a_rule_that_exists_or_a_bad_code_is_refused_with_why` |
| `theme-code-internal-unbounded` | An internal code below `INTERNAL_MIN` is accepted | `codes::every_refusal_keeps_the_editor_open_with_why` |
| `theme-code-empty-accepted` | An empty code field is accepted instead of refused as invalid | `codes::every_refusal_keeps_the_editor_open_with_why` |
| `theme-code-one-of-several` | Only the first of several comma-separated codes is saved | `codes::a_recorded_key_sets_the_code_being_typed_and_stops_recording` and 2 more |
| `theme-code-edit-keeps-old` | Editing a code keeps the old code in the rule alongside the new one | `codes::codes_are_added_edited_and_removed_one_at_a_time`, `saved::an_open_code_editor_survives_recreation_with_what_it_holds_but_not_recording` |
| `theme-code-delete-ignored` | *Delete* in the code editor leaves the code in the rule | `codes::codes_are_added_edited_and_removed_one_at_a_time` |
| `theme-code-chip-opens-first` | Tapping a code chip opens the rule's first code, not its own | `codes::codes_are_added_edited_and_removed_one_at_a_time` and 3 more |
| `theme-code-opened-undescribed` | Opening a code editor does not describe the code's key | `codes::codes_are_added_edited_and_removed_one_at_a_time` |
| `theme-code-recorded-replaces-field` | A recorded key replaces the whole code list, not only the entry being typed | `codes::a_recorded_key_sets_the_code_being_typed_and_stops_recording` |
| `theme-code-key-while-not-recording` | A key pressed while not recording still replaces the typed code | `codes::a_recorded_key_sets_the_code_being_typed_and_stops_recording` |
| `theme-code-record-without-keyboard` | *Record a key* starts without the app's keyboard enabled and selected | `codes::recording_needs_this_apps_keyboard_enabled_and_selected` |
| `theme-code-picker-enable-swapped` | Recording opens the enable screen when the keyboard is already enabled, the picker when not | `codes::recording_needs_this_apps_keyboard_enabled_and_selected` |
| `theme-code-needs-keyboard-stays` | The keyboard-needed error stays shown after the keyboard is enabled | `codes::recording_needs_this_apps_keyboard_enabled_and_selected` |
| `theme-code-recording-unsaid` | The code field does not say it is recording a key | `codes::a_recorded_key_sets_the_code_being_typed_and_stops_recording` |
| `theme-code-record-no-keyboard-shown` | Recording does not bring up the keyboard for the code field | `codes::recording_needs_this_apps_keyboard_enabled_and_selected` |
| `theme-code-stop-unsent` | Stopping a recording does not tell the keyboard to stop listening | `codes::recording_stops_when_tapped_again_typed_over_left_or_stopped_by_kotlin` |
| `theme-code-left-keeps-recording` | Leaving the code editor keeps the key recording running | `codes::recording_stops_when_tapped_again_typed_over_left_or_stopped_by_kotlin` |
| `theme-code-typed-keeps-recording` | Typing a code keeps the key recording running | `codes::recording_stops_when_tapped_again_typed_over_left_or_stopped_by_kotlin` |
| `theme-code-stopped-ignored` | The code field keeps saying it is recording after the keyboard stops | `codes::recording_stops_when_tapped_again_typed_over_left_or_stopped_by_kotlin` |
| `theme-code-described-again` | A code the keyboard already described is described again | `codes::the_preview_of_a_character_with_its_unicode_name` |
| `theme-code-unspecified-described` | The unspecified code 0 is sent to the keyboard to describe | `codes::the_preview_of_internal_codes_with_an_icon_a_label_or_neither` |
| `theme-code-stale-description-taken` | A late description for an old code is applied to the current field | `codes::the_preview_of_a_character_with_its_unicode_name` |
| `theme-code-preview-first-entry` | The key preview shows a list's first code, not the one being typed | `codes::a_recorded_key_sets_the_code_being_typed_and_stops_recording`, `saved::an_open_code_editor_survives_recreation_with_what_it_holds_but_not_recording` |
| `theme-code-preview-said-unfiltered` | The key preview shows another key's description | `codes::the_preview_of_internal_codes_with_an_icon_a_label_or_neither` |
| `theme-code-mark-ignored` | The key preview ignores a non-spacing key's mark | `codes::the_preview_of_a_character_with_its_unicode_name` |
| `theme-code-icon-keeps-label` | A key with a described icon still shows its text label in the preview | `codes::the_preview_of_internal_codes_with_an_icon_a_label_or_neither` |
| `theme-code-label-resources-swapped` | The *Symbols* and *Symbols 2* key previews swap their labels | `codes::the_preview_of_internal_codes_with_an_icon_a_label_or_neither` |
| `theme-code-kinds-swapped` | The `enter_editing` and `function` key kinds swap their names | `codes::the_preview_of_internal_codes_with_an_icon_a_label_or_neither` |
| `theme-code-record-unchecked` | The *Record a key* button is never shown checked while recording | `codes::a_recorded_key_sets_the_code_being_typed_and_stops_recording` |
| `theme-code-hint-empty` | The empty code field shows no placeholder hint | `codes::a_rules_codes_are_chips_that_open_the_editor_and_add_opens_it_empty` |
| `theme-code-link-first` | Each code help link opens the first link's URL | `codes::the_help_links_open_their_pages_and_the_focus_order_reaches_every_control` |
| `theme-code-saved-dropped` | A code editor open at save time does not come back | `saved::an_open_code_editor_survives_recreation_with_what_it_holds_but_not_recording` |
| `theme-code-saved-unheld` | A code editor comes back with no code held | `saved::a_code_editor_saved_without_its_forms_comes_back_on_the_rules` |
| `theme-code-saved-error-dropped` | A code field's error is lost when the editor is restored | `saved::an_open_code_editor_survives_recreation_with_what_it_holds_but_not_recording` |
| `theme-code-saved-recording-kept` | A restored code editor comes back already recording a key | `saved::an_open_code_editor_survives_recreation_with_what_it_holds_but_not_recording` |
| `theme-code-string-id-dropped` | The *Record a key* label is missing from the Kotlin string map | `SettingsResourcesTest.stringIdsAreExactlyTheNativeKeyList` |
| `theme-code-icon-unmapped` | The `pageview` icon is missing from the Kotlin icon map | `SettingsResourcesTest.everyIconTheNativeScreensNameIsMapped` |
| `theme-code-recorder-keeps-slot` | Stopping a key recording leaves the recorder holding the keyboard's key input | `SettingsKeyRecorderTest.oneKeyUpIsTakenAndTheKeyboardGetsItsReceiverBack` and 2 more |
| `theme-code-recorder-overwrites-later` | A stop after another receiver took the key slot restores the old receiver over it | `SettingsKeyRecorderTest.stoppingGivesTheReceiverBackOnlyWhileItIsHeld` |
| `theme-code-recorder-restarts` | A second start while listening leaves the recorder stuck as the key receiver | `SettingsKeyRecorderTest.stoppingGivesTheReceiverBackOnlyWhileItIsHeld` |
| `theme-code-recorder-takes-key-down` | A recording takes its key on press-down rather than on release | `SettingsKeyRecorderTest.aKeyDownRepeatOrCancelIsNotTaken`, `SettingsKeyRecorderTest.oneKeyUpIsTakenAndTheKeyboardGetsItsReceiverBack` |
| `theme-code-described-mark-dropped` | A described key's non-spacing mark is set on function keys, not characters | `SettingsThemeCodeMatchTest.anIconCrossesAsPathDataAndTakesTheLabelsPlace`, `SettingsThemeCodeMatchTest.thePreviewShowsWhatTextKeyDataPreviewBoxShowedForEveryKeyAndSomeCharacters` |
| `theme-code-described-name-dropped` | A described character key's name is always blank | `SettingsThemeCodeMatchTest.anIconCrossesAsPathDataAndTakesTheLabelsPlace`, `SettingsThemeCodeMatchTest.thePreviewShowsWhatTextKeyDataPreviewBoxShowedForEveryKeyAndSomeCharacters` |
| `theme-code-described-icon-unspecified` | A described key's icon is always the unspecified one | `SettingsThemeCodeMatchTest.anIconCrossesAsPathDataAndTakesTheLabelsPlace` |
| `theme-editor-undo-not-in-place` | *Undo* for a deleted property appears one property too low | `property::delete_in_the_top_bar_goes_with_undo_and_a_fonts_src_has_none` and 2 more |
| `theme-editor-internal-codes-refused` | An internal key code is refused as invalid | `codes::a_recorded_key_sets_the_code_being_typed_and_stops_recording` and 4 more |
| `theme-editor-corners-key-unlisted` | The shape corners' *Apply for all* label is missing from the property editor's string keys | `every_resource_key_the_settings_code_names_is_in_the_key_list` |
| `theme-editor-duplicate-empty` | A duplicate takes none of its source's properties | `form::a_duplicate_takes_its_sources_sets_only_where_its_element_takes_that_kind`, `rules::duplicate_adds_a_rule_like_another_with_its_properties` |
| `theme-editor-new-font-single` | A new `@font` rule gets one property set, not a list of them | `form::a_duplicate_takes_its_sources_sets_only_where_its_element_takes_that_kind`, `form::a_font_needs_a_name_the_stylesheet_can_read_back` |
| `theme-editor-duplicate-kind-ignored` | A duplicate whose element takes the other kind of set keeps its source's sets, so a key's properties become a font's one set | `form::a_duplicate_takes_its_sources_sets_only_where_its_element_takes_that_kind` |

### Device check

On `emulator-5556` (`aw_keyboard_ssd`, Android 16), installed with `scripts/install-dev.sh` from
the gate's build of a186ed5cd, driven through agent-device. Screenshots are in
`build/part3-device/` (not committed). **Every planned step passed:**

1. *Create theme extension*, then *Save*, goes back to the manager in one *Save*
   (`10-create-save.png`). The new extension's component opens the Rust editor.
2. The rules list shows each value's icon before *Delete*: swatches, shape outlines, and a
   variable's link over its swatch (`11-rules-list-icons.png`).
3. A key rule's Background, as a static colour, opens the picker with its alpha bar. Its *OK*
   sets a translucent purple, written with its alpha (`12-colour-picker-alpha.png`). With *Show
   keyboard*, the keyboard draws the Enter key translucent purple before *Apply*
   (`13-live-preview.png`).
4. A rounded-corner shape shows its four corners and *Apply for all corners*, now labelled (the
   fix below). The chip copies the corner typed last, *Top start*, to every corner. An empty value copied gives each
   corner *Please enter a dp size* (`14-shape-corners.png`).
5. Font size 28 sp makes the preview's labels larger (`15-font-size.png`,
   `15b-font-size-preview.png`). A number too large for the stylesheet to read back says
   *Please enter a valid number*, as improvement 5 means it to.
6. Padding's *Apply for all sides* copies the side typed last to all four (`16-padding.png`).
7. A variable `--test` added in `@defines`, then Window's Foreground set to it with the
   *Variable* encoder: the row shows the link over the variable's swatch (`17-variable.png`).
8. *Search properties* with "border" leaves Border color, Border style and Border width
   (`18-add-search.png`). The *Material You light colour* encoder lists each name with its swatch
   (`18b-dynamic-swatches.png`).
9. A Foreground close to its rule's Background shows *Low contrast: 3.2:1, below 4.5:1* under the
   value, and *Apply* still works (`19-contrast-note.png`). Choosing *Foreground of background*
   clears it (`19b-contrast-cleared.png`).
10. *Record a key*, with this app's keyboard selected, shows *Recording…*. One press on the
    keyboard fills the code, stops recording, and the preview shows the key's Unicode name, its
    kind and its label (`20-record-key.png`).
11. An `@font` rule's *Add source set* adds a *Source set* heading with *Move up* and *Move down*
    disabled, *Delete* and **+**, and the *Errors* box says *Required property 'src' does not
    exist* (`21-source-set-errors.png`). Deleting the rule leaves its undo bar.
12. *Save*, then reopening the component, keeps every applied value (`22-reopened.png`). An
    unapplied edit in the open property editor survives a rotation to landscape
    (`23-rotated.png`).
13. Chosen as the day theme, the edited extension draws the translucent purple Enter key, 16 dp
    corners, the larger labels and the Material You border (`24-keyboard-theme.png`).

Three more checks:

* A recorded code applied as a chip on a rule (`25-code-applied.png`).
* With agent-device's input method selected instead of ours, *Record a key* shows *AudioWhisper
  Keyboard needs to be the default keyboard…* under the field and opens the input method picker
  (`26-record-not-selected.png`).
* *Home* while recording, then back from Recents: recording has stopped, as `ON_STOP` stops it
  (`27-recording-stopped-on-stop.png`).

**Found on the device:** *Duplicate* on a key rule, with its element changed to `@font` before
*Add*, made an `@font` rule holding the key's properties as a single set. It had *Add property*
where a font has *Add source set*, and the *Errors* box (`28-duplicate-as-font-bug.png`). Part 2's
*Duplicate* copied the source's sets whatever rule it added. Fixed in f3121345c ([above](#fixed-on-the-way)). On a build of
that commit, installed the same way, the same steps make `@font` `Dupfont` with *Add source set*,
no properties and no *Errors* box (`29-duplicate-as-font-fixed.png`). Two source sets added there
show *Move up* disabled on the first and *Move down* on the last, and *Delete* on the first
leaves *Deleted Source set* and *Undo* where it was, with the remaining set's moves both disabled
(`30-source-set-deleted-undo.png`). *Undo* brings back both sets with their buttons as before.
The extension was discarded unsaved.

**How the fields were typed.** agent-device's `fill` cannot type into the renderer's fields: it
fails with *Field focus was refused*. The values were typed by focusing the field with our
keyboard up and clearing it with our keyboard's Backspace, then selecting agent-device's helper
input method, tapping the field and using `type`. So our keyboard's Backspace was seen editing a
Rust field (a data point for E21), but its letters were not.

Afterwards the test extensions were deleted, Floris Day was chosen again, our keyboard was
selected again, the screen was back in portrait with auto-rotate off as it was before the check,
and media volume stayed at 0.

One slip to record: one snapshot printed the keyboard's suggestion strip, which showed the code
of a test key pressed for step 10. It was a letter key pressed for the check, not text anyone
typed. After that only the lines needed were printed.

### Deleted

The 14 files of the Compose theme editor, 3,549 non-blank lines: `EditPropertyDialog` (1,200),
`ThemeEditorScreen` (473), `ThemeEditorComponents` (344), `SnyggValueIcon` (320),
`EditCodeValueDialog` (312), `EditRuleDialog` (293), `ThemeTranslations` (177),
`TextKeyDataPreviewBox` (114), `EnumLikeAttributeBox` (113), `ThemeEditorLoading` (70),
`FineTuneDialog` (54), `SnyggLevel` (29), `DisplayKbdAfterDialog` (26) and `Color` (24). Their
ledger rows are gone, the inventory marks them deleted, and the baseline profile drops their
lines. `DisplayKbdAfterDialogs`' two lines are renamed for its new package, and the two for
`DisplayColorsAs`, a class that no longer existed, are dropped. Four strings only Compose used
are gone from `strings.xml`, each `settings__theme_editor__` followed by
`no_enum_value_to_add_anymore`, `code_recording_started`, `code_recording_stopped` or
`file_selector_dialog_title`. Nothing else changed in frozen
code: `ExtensionEditScreen`, `AppPrefs` and `EnumDisplayEntries` changed only their imports.

## Owed

* **E23:** run `scripts/settings-device-tests.py ... other-theme` again: `SettingsOtherThemeDevice`
  now picks the theme on the Rust manager. It compiles but has not run.
* **P15:** the theme manager on the phone: every step above ran on the emulator only. Choose a
  day and a night theme, check that the keyboard previews each and goes back to its own theme on
  leaving, and that TalkBack reads each theme as a day or night theme and says which is chosen.
* **P17:** the theme editor on the phone: every step of part 2's device check ran on the emulator
  only. On an extension made on the phone, open *Manage component*; add, duplicate and edit a
  rule; delete a property and undo it; change the meta; check the keyboard shows the draft; rotate
  with a draft and with the rule editor open; *Save*, then save the extension and choose its theme
  in the manager.
* **E25:** no instrumented test drives the Rust theme editor, part 3's editors included. The
  device checks were by hand through agent-device. A `SettingsOtherThemeDevice` step that opens *Manage component*, deletes a
  property, saves, and reads the workspace's stylesheet file back would hold it.
* **P20:** the theme manager's *Create* and *Edit* on the phone, after the crash fix above:
  *Save* and *Cancel* each go back to the manager, and an added bundled theme survives a
  rotation of the extension editor.
* **P21:** part 3 on the phone: every step of its device check ran on the emulator only. Set a
  colour with alpha, a shape's corners, a font size and a variable, checking the keyboard shows
  each before *Apply*; search; add a source set; record a key with a real press, and check that
  a key typed in another app after it stops is not taken; compare the Material You swatches with
  the phone's palette. With TalkBack: *Record a key*'s label and checked state, the contrast note
  read aloud, and disabled *Move up* and *Move down* skipped.
* **E32:** no device check chose a file. Pick a font from the archive files as an `@font` set's
  `src`, and an image, save, and check the keyboard draws them; move one of two source sets that
  differ and check the saved order.
* **E33:** agent-device's `fill` fails on the renderer's fields with *Field focus was refused*.
  Find what it asks of the field, and whether TalkBack or switch access meets the same refusal.

## What remains

Nothing of step 9. What is left is owed, or belongs to later steps:

* the phone and instrumented checks under [Owed](#owed);
* the extension editor, its import and its pages, which are Compose until step 10.
  `DialogProperty` moved out of the deleted package into `ExtensionDialogProperty.kt` for it, and
  goes with it;
* rule filtering, left out twice: no shipped stylesheet needs it yet.
