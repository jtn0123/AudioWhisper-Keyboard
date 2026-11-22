# Stage 4: the theme manager and theme editor

This page covers step 9 of [the Compose removal plan](compose-removal.md): the theme manager and
the theme editor move from Compose to the Rust settings renderer. The owner's rule is that they
are "ported to Rust and improved, not cut, and the port names each improvement it makes"
([decision](decisions.md#everything-moves-to-rust-2026-11-21)).

Step 9 has three parts. **Part 1, the theme manager, is done** and is what this page records.
Parts 2 and 3, the editor itself, are not started; [What remains](#what-remains) says exactly
what they need.

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/theme/manage/SELECT_DAY` | Rust | New here: the day theme manager. |
| `settings/theme/manage/SELECT_NIGHT` | Rust | New here: the night theme manager. |
| `ext/edit/null?create=ime.extension.theme` (create a theme) | Compose | Step 10. The manager hands off to it. |
| `ext/edit/<id>` (the extension editor and its theme editor) | Compose | Parts 2 and 3 of this step, then step 10. |
| `ext/view/<id>` (an extension's page: export, delete) | Compose | Step 10. |
| `ext/import/EXT_THEME` | Compose | Step 10. |

The Compose route `Settings.ThemeManager` stays as an empty entry that hands back to Rust, as in
step 2. Its argument is now a plain string, since nothing in Compose reads it.

## What the manager shows

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

## Data path

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

## What Kotlin does

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

## Improvements

Each was chosen for a person choosing or making a keyboard theme. Gboard's theme page was the
reference: making your own theme is the big **+** at the top of it.

| # | Improvement | Why | Test |
| --- | --- | --- | --- |
| 1 | *Create theme extension* and *Import* at the top of the manager. Compose had neither: you went back to Theme, then to *Theme extensions*, to find them. | Gboard puts making your own theme first. Making and importing a theme is what a person on this page is likely to want next. | `create_and_import_hand_off_to_the_compose_extension_screens` |
| 2 | An extension's name, its id and *View details* open its page, where it is exported or deleted. | Compose's manager offered no way to reach an extension from the theme list. | `an_extensions_name_id_and_view_details_open_its_page_and_edit_its_editor` |
| 3 | *Edit*, for an extension made or imported here, opens the extension editor directly. | The way to change a theme you made is two screens shorter. Built-in themes cannot be edited, so they get no *Edit*. | same test |
| 4 | TalkBack says whether each theme is a day or a night theme. | Compose's sun and moon icon had no content description, so a TalkBack user could not tell them apart. | `talkback_hears_whether_each_theme_is_a_day_or_a_night_theme`; JVM `eachThemeTellsTalkBackWhetherItIsADayOrNightTheme` |
| 5 | TalkBack hears which theme is chosen. | Compose's radio button had no click handler, so its row reported no selected state. | the `selected` checks in `each_extension_is_a_box_titled_with_its_name_over_its_id_listing_its_themes` and `a_tap_writes_the_day_theme_and_previews_it`; JVM `aTapWritesWhatTheComponentSerializerStoresAndThePreviewNamesIt` |

## Differences from Compose

Each one is deliberate.

| Difference | Why |
| --- | --- |
| *Create* and *Import* sit in a box at the top of the manager, not in the extension list's floating button. | Improvement 1. The extension list itself is step 10 and keeps its button. |
| A built-in extension has no *Edit* button. Compose's extension list shows it greyed out. | A button that can never work is noise; TalkBack would read it on every built-in box. |
| The preview is also cleared when the settings view is released, not only when the manager leaves the screen. | A settings screen killed with a manager on top would otherwise leave the keyboard showing a theme that is not the chosen one. |
| If the day or night preference changes elsewhere while a manager is open, the keyboard's preview is not updated until the manager is reopened. The radio buttons do update. | Nothing else writes these preferences while the manager is on top. |
| A repeated extension id in the index is handled. Compose's `!!` lookup could crash on one. | It costs one branch. |
| Keyboard and D-pad focus moves through *Create*, *Import*, then each box's name, id, themes, *View details* and *Edit*. | Every Rust screen has a focus order; Compose's was whatever the layout gave. |

## Tests

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

## Mutations

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

## Device check

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

## Deleted

`ThemeManagerScreen` (135 non-blank lines), and its route's enum argument in `Routes.kt`.
The remaining 14 files of `FB/app/settings/theme` are deleted with parts 2 and 3.

## Owed

* **E23:** run `scripts/settings-device-tests.py ... other-theme` again: `SettingsOtherThemeDevice`
  now picks the theme on the Rust manager. It compiles but has not run.
* **P15:** the theme manager on the phone: every step above ran on the emulator only. Choose a
  day and a night theme, check that the keyboard previews each and goes back to its own theme on
  leaving, and that TalkBack reads each theme as a day or night theme and says which is chosen.

## What remains

Parts 2 and 3: the theme editor. The Compose editor is 14 files, 3,549 non-blank lines:
`EditPropertyDialog` (1,200), `ThemeEditorScreen` (473), `ThemeEditorComponents` (344),
`SnyggValueIcon` (320), `EditCodeValueDialog` (312), `EditRuleDialog` (293), `ThemeTranslations`
(177), `TextKeyDataPreviewBox` (114), `EnumLikeAttributeBox` (113), `ThemeEditorLoading` (70),
`FineTuneDialog` (54), `SnyggLevel` (29), `DisplayKbdAfterDialog` (26) and `Color` (24).

### How the editor is reached today

The Compose editor lives inside `ExtensionEditScreen` (step 10): `ext/edit/<id>` opens an
in-memory workspace (`CacheManager.ExtEditorWorkspace`) holding the unpacked extension, and its
*Manage component* action shows `ThemeEditorScreen(workspace, editor)`. Saving the theme writes
the stylesheet back into the workspace; saving the extension zips it and installs it. So the
editor cannot simply be a new route: it shares unsaved state with a Compose screen that stays
until step 10.

The plan that fits that:

1. A Rust route `settings/theme/edit/<extension>/<component>` that the extension editor's
   *Manage component* opens instead of `ThemeEditorScreen`, with the workspace kept alive by the
   existing `CacheManager` while it is open.
2. Kotlin glue, like `SettingsThemes`, that reads the component's stylesheet text and the
   workspace's font and image file names and sends them with a new op (one of 28–39), and on
   *Save* takes the stylesheet text Rust built and writes it into the workspace as
   `ThemeEditorScreen`'s `handleSave` does. The keyboard preview uses
   `ThemeManager.previewThemeInfo`, as Compose does.
3. When step 10 ports the extension editor, the route needs no change.

### Part 2: the editor shell

* **Load.** `snygg::json::Stylesheet::from_json` already parses a stylesheet. The loading state
  (`ThemeEditorLoading`) is a spinner until the glue's report arrives, and an error page if the
  stylesheet does not parse.
* **Save.** Nothing in Rust writes a stylesheet yet. The `snygg` crate needs a serializer whose
  output matches Compose's `PrettyPrintConfig` (`ThemeEditorLoading.kt`): the same key order,
  indentation and number formats, held by a JVM match test against `SnyggStylesheet.toJson`, so
  a theme saved by Rust opens unchanged in an older build.
* **The rules list** (`ThemeEditorScreen`, `SnyggRuleRow`): rules in `snygg::editor::compare_order`
  order, each titled with its element's translated name (`ThemeTranslations`) and its selectors and
  attributes, with its properties under it; *Add rule*; tapping a rule opens the rule dialog;
  delete with confirmation. `snygg::editor::Editor` already has `rename`, `remove`,
  `copy_rule`, `set_property` and `build`, and `rule_edit` gives the dialog's errors.
* **The rule dialog** (`EditRuleDialog`, `EnumLikeAttributeBox`): element picker, the pressed,
  focus, disabled and selected selectors, and the attribute lists (key codes, modes, shift
  states), with the duplicate-rule error.
* **The component meta dialog** (`ComponentMetaEditorDialog`): label, authors, night flag.
* **Fine-tune** (`FineTuneDialog`, `SnyggLevel`, `DisplayKbdAfterDialog`): the editor level
  (basic, advanced, developer) and whether the keyboard shows after a dialog. These are two Rust
  catalogue rows on existing widgets.
* **Save and discard:** *Back* with changes asks whether to discard them.

### Part 3: the property editor

`EditPropertyDialog` and its helpers, built on `snygg::value`, `snygg::value_codec` and
`snygg::value_spec`, reusing the colour picker from step 4 and the existing list, field and
dialog widgets:

* the property picker, filtered by the editor level;
* each value kind: defined variables (with the variable picker), static colours (the step 4
  colour picker), dynamic light and dark colours (the Material You colour slots), font families
  (generic and custom, from the workspace's fonts), font style and weight, padding (four sides),
  shapes (rectangle, circle, cut and rounded corners in dp and in percent, each corner), dp, sp
  and percentage sizes, content scale, text align, decoration, max lines and overflow, and URIs
  (the workspace's images);
* switching a value between the kinds its property supports (the encoder picker);
* the code value editor (`EditCodeValueDialog`): recording a key's code, or typing one, with the
  key data preview (`TextKeyDataPreviewBox`);
* each value's small preview icon (`SnyggValueIcon`) in the rules list.

### Then the deletion

Delete the 14 files. Before that, move the four things other code still imports from the
package, which are mechanical changes to frozen code:

* `PrettyPrintConfig` and `DialogProperty`, used by `ExtensionEditScreen` (step 10);
* `ColorPreferenceSerializer` (`Color.kt`), used by `AppPrefs` for both accent colours;
* `DisplayKbdAfterDialogs` and `SnyggLevel`, used by `AppPrefs` and `EnumDisplayEntries`.

Then remove their rows from `docs/jvm-ledger.tsv`, mark them deleted in
`docs/stage5-inventory.md`, drop their lines from `app/src/main/baseline-prof.txt`, and record the
step as done in [compose-removal.md](compose-removal.md).

### Improvements proposed for parts 2 and 3

Each needs its own test when it is built.

* **A live preview of the rule being edited**, as good theme editors show (the keyboard redraws
  as a colour or size changes, before the dialog is confirmed).
* **Undo** for a deleted rule or property, as the user dictionary's undo does.
* **Contrast warning** when a key's text colour is hard to read on its background.
* **Search** in the property picker, which lists dozens of properties at the developer level.
