# Stage 4: the theme manager and theme editor

This page covers step 9 of [the Compose removal plan](compose-removal.md): the theme manager and
the theme editor move from Compose to the Rust settings renderer. The owner's rule is that they
are "ported to Rust and improved, not cut, and the port names each improvement it makes"
([decision](decisions.md#everything-moves-to-rust-2026-11-21)).

Step 9 has three parts. **Part 1, the theme manager, and part 2, the theme editor's shell, are
done**, and this page records both. Part 3, editing a property's value, is not started;
[What remains](#what-remains) says exactly what it needs, and what deleting the Compose editor
then takes.

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/theme/manage/SELECT_DAY` | Rust | New here: the day theme manager. |
| `settings/theme/manage/SELECT_NIGHT` | Rust | New here: the night theme manager. |
| `ext/edit/null?create=ime.extension.theme` (create a theme) | Compose | Step 10. The manager hands off to it. |
| `ext/edit/<id>` (the extension editor) | Compose | Step 10. Its *Manage component* opens the Rust theme editor. |
| `settings/theme/edit/<workspace>/<component>` | Rust | New in part 2: the theme editor, in `ThemeEditorActivity`. |
| `settings/theme/editor/rule/add`, `.../rule`, `.../meta`, `.../tune` | Rust | New in part 2: the rule editor, the meta editor and fine-tune. |
| `ext/view/<id>` (an extension's page: export, delete) | Compose | Step 10. |
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
`scripts/settings-ui-mutations.py` has 21 new ones for the manager; each must make a test fail,
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
| 2 | **Duplicate rule.** Opens the rule editor with the rule's element, selector and attributes; the rule it adds takes the original's properties. | The usual way to make a pressed or a numeric-mode variant of a key's style. Compose had you add the rule, then every property, by hand. | `duplicate_adds_a_rule_like_another_with_its_properties` |
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

`scripts/settings-ui-mutations.py` has 54 new cases for the editor: 43 in Rust and 11 in Kotlin.
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
    manager's *Edit* had stacked it twice, the deep link fault under [Owed](#owed). *Back* from
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
* **E25:** no instrumented test drives the Rust theme editor. The device check was by hand
  through agent-device. A `SettingsOtherThemeDevice` step that opens *Manage component*, deletes a
  property, saves, and reads the workspace's stylesheet file back would hold it.
* **Not a device check, and older than this page:** the theme manager's *Create* hands off to
  Compose's *Create theme extension* by deep link, and the screen ends up on the stack twice.
  Its *Edit* does the same to the extension editor (step 10 of part 2's device check). The
  likely cause is that `NavController` handles the activity's deep link when its graph is set,
  and `FlorisAppActivity`'s `LaunchedEffect(intentToBeHandled)` calls `handleDeepLink` again.
  *Save* closes the top copy and its workspace, the copy under it stays, and a second *Save*
  there crashes with `FileNotFoundException` on the closed workspace's `saver/extension.json`
  (`ExtensionEditScreen`'s `handleSave`). Repro: *Select day theme*, *Create theme extension*,
  *Save*, *Save* again. Found on the emulator while checking part 2. It goes with step 10, which
  ports the extension editor, unless it is fixed sooner as a crash fix to frozen code.

## What remains

Part 3, the property editor, and then deleting the Compose editor. The Compose editor is 14
files, 3,549 non-blank lines: `EditPropertyDialog` (1,200), `ThemeEditorScreen` (473),
`ThemeEditorComponents` (344), `SnyggValueIcon` (320), `EditCodeValueDialog` (312),
`EditRuleDialog` (293), `ThemeTranslations` (177), `TextKeyDataPreviewBox` (114),
`EnumLikeAttributeBox` (113), `ThemeEditorLoading` (70), `FineTuneDialog` (54), `SnyggLevel` (29),
`DisplayKbdAfterDialog` (26) and `Color` (24). Part 2 ported what `ThemeEditorScreen`,
`ThemeEditorLoading`, `EditRuleDialog` (but its code dialog), `EnumLikeAttributeBox`,
`FineTuneDialog`, `SnyggLevel`, `DisplayKbdAfterDialog` and `ThemeTranslations` do. None of the
14 is routed to now, but all stay until part 3 is done.

### Part 3: the property editor

`EditPropertyDialog` and its helpers, built on `snygg::value`, `snygg::value_codec` and
`snygg::value_spec`, reusing the colour picker from step 4 and the existing list, field and
dialog widgets. It opens from two places part 2 left empty: tapping a property in a rule's box,
and a rule's *Add property* (or, for `@font`, *Add source set*, as Compose's button did), which
belongs on the rule's title line beside *Duplicate* and *Edit*.

* the property picker, filtered by the editor level;
* each value kind: defined variables (with the variable picker), static colours (the step 4
  colour picker, shown in fine-tune's colour representation, which part 2 already stores),
  dynamic light and dark colours (the Material You colour slots), font families (generic and
  custom, from the workspace's fonts, which the report already carries), font style and weight,
  padding (four sides), shapes (rectangle, circle, cut and rounded corners in dp and in percent,
  each corner), dp, sp and percentage sizes, content scale, text align, decoration, max lines
  and overflow, and URIs (the workspace's images, also in the report);
* switching a value between the kinds its property supports (the encoder picker);
* each value's small preview icon (`SnyggValueIcon`) beside it in the rules list, where part 2
  shows the value as the stylesheet writes it;
* the "Errors" box for a set missing a required property (only a font set's `src`), which part 2
  leaves to *Save*'s refusal;
* the code value editor (`EditCodeValueDialog`): recording a key's code, or typing one, with the
  key data preview (`TextKeyDataPreviewBox`). In part 2 the rule editor takes typed codes in one
  comma-separated field; part 3 adds *record a key* to it.

The draft, Save, Back, undo and the preview are already in place: a property edit changes
`App::theme_editor`'s rules like a delete does, and everything after follows.

### Then the deletion

Delete the 14 files. `ExtensionEditScreen` no longer imports `ThemeEditorScreen`. Before that,
move the things other code still imports from the package, which are mechanical changes to
frozen code:

* `PrettyPrintConfig` and `DialogProperty`, used by `ExtensionEditScreen` (step 10).
  `SettingsThemeEditor` and `SettingsThemeEditorMatchTest` use `PrettyPrintConfig` too;
* `CustomRuleComparator` (`ThemeEditorComponents.kt`), which `SettingsThemeEditorMatchTest` uses
  to build the editor Compose built (`SnyggStylesheetEditor` itself is in `lib/snygg` and
  stays);
* `ColorPreferenceSerializer` (`Color.kt`), used by `AppPrefs` for both accent colours;
* `DisplayKbdAfterDialogs` and `SnyggLevel`, used by `AppPrefs` and `EnumDisplayEntries`.

Then remove their rows from `docs/jvm-ledger.tsv`, mark them deleted in
`docs/stage5-inventory.md`, drop their lines from `app/src/main/baseline-prof.txt`, and record the
step as done in [compose-removal.md](compose-removal.md).

### Improvements proposed for part 3

Each needs its own test when it is built.

* **A live preview of the value being edited.** The keyboard already shows the draft as it
  changes; part 3 should send the preview as a value changes in the property editor, before it
  is confirmed.
* **Contrast warning** when a key's text colour is hard to read on its background.
* **Search** in the property picker, which lists dozens of properties at the developer level.
* **Filter rules** by element, if stylesheets grow past what one scroll shows. Part 2 left it
  out: the shipped stylesheets have tens of rules, already grouped by element.
