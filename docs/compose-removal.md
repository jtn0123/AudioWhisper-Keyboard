# Removing the Compose UI

2026-11-16. Rust now draws the keyboard and settings by default ([decision](decisions.md#rust-draws-by-default-2026-11-16)).
This page is the order in which the Compose code that Rust replaced goes away, and the rule
for touching it until then. The [stage 5 inventory](stage5-inventory.md) is the source for
which files are covered: its ledger "Files already covered by settings/Compose-keyboard work"
lists every one with its size and scope (`stage4` = settings, `keyboard` = the Compose
keyboard). This page does not repeat that list. It orders it and names what blocks each step.

This is one part of [moving everything to Rust](all-rust.md): its phase 4 is steps 5–12 here,
and its phase 1 is step K.

Sizes are non-blank lines of production Kotlin, counted as [rust.md](rust.md) describes,
on `d5f220b00` unless a step says otherwise.

## The freeze

Compose is frozen as of 2026-11-16.

* **New features go into Rust only.** A new setting, screen, control or keyboard behaviour
  is built in the Rust catalogue, screens and renderer. Neither Compose path gets it.
* **Compose gets only two kinds of change:** a fix for a crash, or a fix for lost or damaged
  user data. Nothing else: no new rows, no polish, no refactors beyond what such a fix needs.
* **A feature for a screen that is still Compose** (see step 4 onwards) waits for that
  screen's Rust port, or ports the screen first. It is not added to the Compose screen.
* **Keyboard fallback.** The Compose keyboard stays until its phone checks pass (step K). It
  falls behind as Rust gains features; that is expected and is not a bug to fix in Compose.
* **Deletion is not frozen.** Removing Compose code that Rust has replaced is always allowed,
  following the order below.

A review that sees a Compose screen file or a file from the inventory's `keyboard` scope
changed should ask which of the two allowed reasons it is.

## Order

Settings go first. The keyboard step (K) does not depend on them and can happen as soon as
its phone checks pass.

### 1. Rust by default (done)

* **Deletes:** nothing.
* **What changed:** both switches default to on; an explicit off is kept.
* **Blocked by:** nothing.

### 2. Settings screens that Rust draws (done)

* **Deletes:** the Compose versions of every screen Rust draws and that was checked on the
  device: Home, Keyboard, Input feedback, Smartbar, Gestures, Clipboard, Media, Typing (with
  `SpellCheckerServiceSelector`), Dictionary, Physical keyboard, About, Dictation (with
  `ModelStatus`) and Third-party licences. That is 15 files, about 1,915 lines, under
  `FB/app/settings/{.,keyboard,smartbar,gestures,clipboard,media,typing,dictionary,advanced,about}`
  and `AW/dictation`, plus the strings and helpers only they used.
* **Also deletes:** the settings switch (`other__rust_settings_prototype`). Its "off" would
  have nothing left to choose. The stored key is dropped by the settings migration.
* **How routing works afterwards.** The settings activity always uses the Rust host once
  the keyboard is set up. Setup stays Compose; finishing it, or a `ui://florisboard/…` link
  arriving during it, hands over to the Rust host. Rust then opens the routes it does not
  draw in `ComposeSettingsFallbackActivity`, as before. In that activity the deleted
  screens' destinations stay as empty entries, so that reaching one still hands it back to
  Rust (Physical keyboard from Other, Home on Back). Since step 11 the activity always uses the
  Rust host, which draws setup too and picks the first screen.
* **Blocked by:** nothing. Every screen here passed the stage 4 device checks
  ([checkpoint](stage4-settings-results.md), [follow-up 3](stage4-followup3.md),
  [follow-up 4](stage4-followup4.md), [licences](stage4-licences.md)).
* **The fallback loses:** the choice of drawing these screens in Compose.
* **Progress** (non-blank lines deleted):
  * the settings switch, with every entry point routed to Rust;
  * Home, Keyboard and Input feedback: 3 files, 541 lines;
  * Smartbar, Gestures, Clipboard and Media: 4 files, 613 lines;
  * Typing (with `SpellCheckerServiceSelector`), Dictionary, Physical keyboard, About and
    Dictation (with `ModelStatus`): 7 files, 691 lines, plus `FlorisErrorCard` (24), which
    only the spell checker card used;
  * Third-party licences: 1 file, 70 lines. Step 2 is done: 15 files, 1,915 lines.

### 3. Dead code left behind by step 2 (done)

* **Deletes:** helpers, enum display entries, resources and facades that no build path
  reaches once step 2 is in. Each needs a grep or lint result as proof.
* **Blocked by:** step 2.
* **Done:**
  * the enum display entries: `KeyboardEnumDisplayEntries` is gone and `EnumDisplayEntries`
    kept the 8 enums the remaining Compose screens still asked for (App theme, colour
    representation, keyboard after dialogs, language names, shift state, keyboard mode,
    Snygg level, theme mode); step 4 took App theme and theme mode, and step 6 took language
    names, leaving 5. The other 23
    entries' strings all stay: Rust reads them through `SettingsStringIds`;
  * declarations nothing names, found by a scan of every top-level declaration in main
    Kotlin against every tracked Kotlin, Java, XML and Gradle file: `apptheme/Color.kt`,
    `ColorScheme.amoled`, `ExtensionList`, `FREQUENCY_DEFAULT`, `NativeStr` with
    `toJavaString`/`toNativeStr`, `NativeInstanceWrapper`, `stripUnicodeCtrlChars` with its
    matcher, `FlorisRef.hasAsset`, `ContentResolver.readAllText`/`writeAllText`,
    `removeAndReturn`, `RegexSerializer` and `FlorisButton`;
  * three baseline profile lines that named deleted code.
* **Left for later:** the app no longer calls the AboutLibraries runtime libraries (core and
  compose-m3); only `ReferenceLicenceScreens`, a JVM test, does. Moving them to
  `testImplementation` also drops them from the shipped licence list, which the device
  baseline compares, so it waits for a baseline refresh.

### 4. Other and Theme, with the colour and time pickers (done)

* **Deletes:** `OtherScreen` and `ThemeScreen`, and Jetpref's colour and time picker use.
* **What changed:** Rust draws both screens, with a new colour picker (presets, a
  saturation and brightness square, a hue bar and a hex field), a new time picker (a dial
  and two number fields), theme names from the extension index, and the add-on reference
  box. The theme manager and editor stay in Compose (step 9), and Back from them returns to
  Rust. See [stage 4: Other and Theme](stage4-other-theme.md).
* **Blocked by:** nothing.
* **The fallback loses:** both screens. Their routes stay as empty entries that hand back to
  Rust, as in step 2.
* **Progress** (non-blank lines deleted): `OtherScreen` (89), `ThemeScreen` (118),
  `AddonManagementReferenceBox` (39), the App theme and theme mode display entries (45) and
  `ColorMappings` (24), which only these screens used, plus 6 lines in `Routes.kt` and
  `SettingsFacts` rewritten as 4. Step 4 is done: 321 lines deleted, 4 added.

### 5. Backup and restore (done)

* **Deletes:** `BackupScreen` (362) and `RestoreScreen` (265). `BackupIO` (81) is file
  input and output and stays as an adapter.
* **What changed:** Rust draws both screens, with progress and errors in plain language, a
  confirmation before a restore, and the backup's details shown first. A new file picker
  boundary lets Rust ask for the system picker and decide what follows; Kotlin opens it, keeps
  documents under handles and runs the file work. Steps 7 and 10 reuse it. Restore now offers
  only the groups a backup holds ([decision](decisions.md#restore-offers-only-the-groups-a-backup-holds-2026-11-21)).
  See [stage 4: Back up and Restore](stage4-backup-restore.md).
* **Blocked by:** nothing.
* **The fallback loses:** both screens. Their routes stay as empty entries that hand back to
  Rust, as in step 2.
* **Progress** (non-blank lines deleted): `BackupScreen` (362), `RestoreScreen` (265),
  `CacheManager`'s Compose-only fields (4), 4 lines in `Routes.kt` rewritten as 2, and an
  import in `ExtensionEditScreen` (1), plus Compose's two failure strings. `RadioListItem`
  (42) moved unchanged into its own file for `ExtensionEditScreen` (step 10). Step 5 is done:
  636 lines deleted, 2 added.

### 6. Languages and layouts (done)

* **Deletes:** `FB/app/settings/localization`: 5 files, 1,078 lines (the list, locale
  picker, language pack manager and subtype editor with its state).
* **What changed:** Rust draws all four screens. A new editable list adds, opens, moves and
  deletes items, with *Move up* and *Move down* buttons, the same moves as TalkBack actions,
  and undo after a delete. A per-item editor goes with it. Step 7 reuses both. Rust writes
  the subtype list itself, byte for byte as SubtypeManager did; Kotlin reports locale names,
  layouts, presets and packs, and deletes a pack. Select language puts the device's languages
  first, and a picked language fills the form, both taken from Gboard. Language pack import
  and a pack's page stay in Compose (step 10). See
  [stage 4: Languages and layouts](stage4-languages.md).
* **Blocked by:** nothing.
* **The fallback loses:** all four screens. Their routes stay as empty entries that hand back
  to Rust, as in step 2.
* **Progress** (non-blank lines deleted): `LanguagePackManagerScreen` (208),
  `LocalizationScreen` (179), `SelectLocaleScreen` (142), `SubtypeEditorScreen` (449) and
  `SubtypeEditorState` (100), plus code only they used: `FlorisDropdownLikeButton` (68),
  `FlorisWarningCard` (25), the language names display entry (17), `SettingsSelection` (6)
  and five `SubtypeManager` functions (24), 18 lines in `Routes.kt` rewritten as 7, and
  Compose's two delete-confirmation strings. The native screens add `SettingsLanguages` (99),
  its string ids (53) and 18 lines across the accessibility, icon, string id and view
  adapters. Step 6 is done: 1,239 lines deleted, 177 added.

### 7. User dictionaries (done)

* **Deletes:** `UserDictionaryScreen` (385).
* **What changed:** Rust draws both user dictionaries, the keyboard's own and Android's: the
  languages words are stored under, a language's words, and a word editor. They reuse step 6's
  editable list, with a flag that leaves out *Move up* and *Move down*, and its item editor.
  The words stay where they were, in Room and in Android's provider; Kotlin makes each change
  as the old dialog made it, and JVM tests prove the same database calls against a frozen copy
  of the old code. The dialog's checks, the import's reading and the export's bytes move to
  Rust, through step 5's file picker; old export files import, and a new export is
  byte-compatible. Undo after a delete, a search, and refusing a duplicate word are new; a
  language's list titled by its language and a new word starting in it are taken from Gboard.
  See [stage 4: User dictionaries](stage4-user-dictionaries.md).
* **Blocked by:** nothing.
* **The fallback loses:** both dictionaries. The route stays as an empty entry that hands back
  to Rust, as in step 2.
* **Progress** (non-blank lines deleted): `UserDictionaryScreen` (385), plus code only it used:
  `UserDictionaryValidation` (62), `UserDictionary`'s import and export (87), 5 lines in
  `Routes.kt` rewritten as 1, and Compose's locale error string. `UserDictionaryType` (20
  with its licence header) moved out of the screen into its own file. The native screens add
  `SettingsWords` (140), its string ids (36) and 13 lines across the view, file, icon and
  string id adapters. Step 7 is done: 542 lines deleted, 210 added.

### 8. Project licence (done)

* **Deletes:** `ProjectLicenseScreen` (65).
* **What changed:** Rust draws the licence: the asset read off the main thread and sent back
  with a new settings op, the unwrapped 10 sp monospace text scrolled both ways, long-press
  word selection, the editor's handles dragged across lines and past the edges, a floating Copy
  and Select all toolbar, and TalkBack's selection, copy and text traversal. Kotlin keeps only
  the asset read, the clipboard, the haptic tick, the toolbar's `ActionMode` (the editor's, now
  serving both) and the accessibility event. A drag held at an edge keeps scrolling, the
  horizontal scrollbar stays on screen, and TalkBack can extend a selection and move by page,
  where Compose could not. See
  [stage 4: the project licence screen](stage4-licences.md#the-project-licence-screen).
  Commits 6f3f00768, 49bd129e3, 10025efd2, 4697ebb34, 72f420db2 and 0be2d6291.
* **Blocked by:** nothing.
* **The fallback loses:** the project licence. The route stays as an empty entry that hands
  back to Rust, as in step 2.
* **Progress** (non-blank lines deleted): `ProjectLicenseScreen` (65), and 2 lines in
  `Routes.kt` rewritten as 1. The Kotlin glue adds `SettingsReadOnlyText` (51), a read-only mode
  in `SettingsSelectionMenu` (43 added, 8 removed), the text node in `RendererAccessibility`
  (21 added, 4 removed), 6 lines in `RustSettingsView` (2 removed) and one string id. Step 8 is
  done: 81 lines deleted, 123 added.

### 9. Theme manager and theme editor (done)

* **Deletes:** the rest of `FB/app/settings/theme`: 15 files, 3,684 lines after step 4,
  including `EditPropertyDialog` (1,200) and `ThemeEditorScreen` (473).
* **Ported and improved, not cut** ([decision](decisions.md#everything-moves-to-rust-2026-11-21)):
  the port names each improvement it makes. See
  [stage 4: the theme manager and theme editor](stage4-theme-editor.md).
* **Part 1, the theme manager (done):** Rust draws the day and night managers: each installed
  theme extension in an outlined box, its themes sorted by label with radio buttons, the
  keyboard previewing the chosen theme while a manager is on top. Kotlin only reports the
  extension index and sets the keyboard's preview. Five improvements: *Create* and *Import* at
  the top (Gboard's *My themes* **+**), an extension's name, id and *View details* opening its
  page, *Edit* for themes made here, and TalkBack hearing each theme's day or night kind and
  which one is chosen. Commits cfdc0b943, 46f866859, 10247ba30, 7881f7ec1 and 709b3ebab.
* **Part 2, the theme editor's shell (done):** the extension editor's *Manage component* opens
  the Rust editor at `settings/theme/edit/<workspace>/<component>`, in its own
  `ThemeEditorActivity` so the Compose workspace stays open under it. Rust draws loading, the
  stylesheet error page, the component box, fine-tune, *Add rule*, the rules with their
  properties, the rule editor, the meta editor (id, label, authors, night, stylesheet path and
  Material You's flags), delete with its question, and *Back* asking to discard. It parses,
  edits and checks the stylesheet, and writes it with a new `snygg` serializer that a JVM test
  holds to Compose's `PrettyPrintConfig` on every shipped stylesheet. Kotlin reports the
  workspace's stylesheet, meta and file names over op 28 and writes Save back. Eight
  improvements: undo in place for a deleted rule or property, *Duplicate rule*, no *Delete* on a
  font's `src`, a font name that cannot be read back refused, edits kept as a draft until *Save*,
  attributes in file order, a repeated component id refused under its field, and the extension
  editor showing the new meta at once. Properties are listed read-only. Commits f2b7ce6c3 to
  3cc7ba633; see [part 2](stage4-theme-editor.md#part-2-the-theme-editor-shell-done).
* **Part 3, the property editor (done):** Rust draws the property editor for every Snygg value
  kind, opened by tapping a property or a rule's *Add property*: the property picker with
  search, the kinds a property takes, static colours through step 4's colour picker with an
  alpha slider, Material You colours with swatches from the keyboard's own palette, fonts, sizes,
  padding, shapes with each corner, the enum-like values, images and variables. It adds each
  value's preview icon in the rules list, a font's source sets with their Errors box, and the
  code editor for a key rule's codes: typed or recorded from this app's keyboard, with the key
  data preview. Kotlin adds only the key recorder, which takes one key-up from the running
  keyboard, and the palette and icon glue. Improvements include a live preview of the value
  before *Apply*, a contrast warning below 4.5:1, an existing property name refused, a value that
  must read back as written, undo for a deleted source set, several codes typed at once, and the
  editor surviving recreation; see
  [part 3](stage4-theme-editor.md#part-3-the-property-editor-done) for all of them with their
  tests. Commits a9701c8ca to b459379b7, the deletion (a186ed5cd), and a fix to part 2's
  *Duplicate* found on the emulator (f3121345c). A crash fix to the Compose fallback (85334f578)
  comes first: *Save* in the extension editor the manager opened left that screen drawn, and a
  second *Save* crashed on its closed workspace.
* **Blocked by:** nothing.
* **The fallback loses:** the theme manager. Its route stays as an empty entry that hands back
  to Rust, as in step 2. The Compose theme editor had no route of its own: since part 2 the
  extension editor opens the Rust one.
* **Progress** (non-blank lines deleted): part 1 deletes `ThemeManagerScreen` (135), and 7
  lines in `Routes.kt` rewritten as 2. The Kotlin glue adds `SettingsThemes` (44), its string ids
  (11), 5 lines in `RustSettingsView` (1 removed) and one rewritten line in `SettingsStringIds`.
  Part 1: 144 lines deleted, 63 added. Part 2 deletes nothing: the 14 Compose files are part 3's
  reference and other code imports from their package. It adds `SettingsThemeEditor` (113),
  its string ids (207), `ThemeEditorActivity` (22), 4 lines in `RustSettingsView`, 2 in
  `SettingsIcons` (1 removed), 2 in `SettingsStringIds` (1 removed) and 9 in
  `ExtensionEditScreen` (2 removed). Part 2: 4 lines deleted, 359 added. Part 3 deletes the 14
  files (3,549 lines), 2 lines in `ExtensionEditScreen` rewritten as 1, 5 imports in `AppPrefs`
  and `EnumDisplayEntries`, and 20 lines of `SnyggUi`'s palette call (2 added). It adds
  `SettingsKeyRecorder` (108), 85 lines of string ids (2 removed), 23 lines in
  `SettingsThemeEditor` (6 removed), 15 in `SettingsIcons`, 4 in `RustSettingsView`, 18 in
  `SnyggPalette` (4 removed; the palette call `SnyggUi` and the editor now share), and 32 in
  `FlorisAppActivity` (14 removed) for the crash fix. The items other
  code still imported move out unchanged: `ThemeEditorPrefs.kt` (55, with the licence) takes
  `PrettyPrintConfig`, `ColorPreferenceSerializer`, `DisplayKbdAfterDialogs` and `SnyggLevel`,
  and `ExtensionDialogProperty.kt` (48) takes `DialogProperty` until step 10 deletes it with
  `ExtensionEditScreen`. Part 3: 3,602 lines deleted, 391 added, 103 of them moved. Step 9 is
  done: 3,750 lines deleted, 813 added.

### 10. Extensions (done)

* **Deletes:** `FB/app/ext`: 15 files (import, export, edit, view, lists, `RadioListItem`, moved
  here from step 5, and `ExtensionDialogProperty`, moved here from step 9).
* **Ported and improved, not cut** ([decision](decisions.md#everything-moves-to-rust-2026-11-21)):
  the port names each improvement it makes. See [stage 4: extensions](stage4-extensions.md).
* **Part 1, the home, lists and page (done):** Rust draws the extension home, the theme,
  keyboard and language pack lists, an extension's page (or that none has its id) and *Search
  for updates*, with their boxes, chips and the delete question. It decides what each screen
  lists, the update link, which rows the page leaves out and each component's text. Kotlin only
  reports the extension index and deletes an extension. Twelve improvements, among them chips
  that wrap instead of scrolling off the screen, keywords in full ink, a maintainer chip and a
  link that tell TalkBack what they offer and where they go, the delete question surviving a
  rotation, no greyed-out *Edit* on built-ins, a page that waits for the index, and fixes to the
  update link's commas and to every outlined box, which TalkBack now reads title first. Commits
  8b87a32d6 and 66728a3ff (the mutation script split), 7c08876a0, 9ddcb169c, 14eb479f6,
  6a6db86cc, 16d0c391a and 8379b5d21; see
  [part 1](stage4-extensions.md#part-1-the-extension-home-lists-and-page-done).
* **Part 2, import and export (done):** Rust draws the four importers (any extension,
  keyboards, themes, language packs) and asks for Export's document itself. It decides which
  files can be imported and why the others cannot, what *Import* imports and every message.
  Kotlin reads the picked or shared documents into an import workspace, describes it, imports
  the files Rust chose and writes an export, all off the main thread. A file opened in or
  shared with the app starts the Rust importer on it, so the Compose fallback takes only later
  browsable links. Nine improvements, among them a file of the wrong type shown skipped instead
  of silently left out after *Imported*, a selection that survives rotation and is deleted
  however the importer is left, failures in plain words instead of stack traces, and a failed
  export that deletes its half-written file. Commits 1654613e5, db7f9ca57,
  bbb752daa, 358a737f6, b2c05973e, 75d7e697c, 10dac09e7 and ce675dea9; see
  [part 2](stage4-extensions.md#part-2-import-and-export-done).
* **Part 3, the extension editor (done):** Rust draws the extension editor, its sheets (meta
  data, dependencies, archive files, a file, naming a picked file, *Create component*) and each
  component's box, at *Edit*, the theme list's *Create theme extension* and the theme manager's.
  Rust holds the draft, decides every screen, check and message, and writes `extension.json`
  byte for byte as kotlinx did. Kotlin keeps the editor's workspace, now in `noBackupFilesDir`,
  opens the theme editor over it, and runs *Save* and the file work off the main thread. The
  hand-off to Compose (`fallbackHandles`, `fallbackDone`, owed P20) is deleted. Twenty
  improvements, among them deletions and file changes that count as unsaved changes, a second
  *Save* that can no longer crash, a component that cannot be built named instead of crashing,
  a draft that survives rotation and process death, an archive written beside its place and
  moved there, and a settings link mid-edit that lands over the editor; see
  [part 3](stage4-extensions.md#part-3-the-extension-editor-done).
* **Blocked by:** nothing.
* **The fallback loses:** the screens above, and with part 3 every screen it drew. Part 1's
  routes and `Ext.Edit` stay as empty entries that hand back to Rust, as in step 2; part 2's
  importer and export routes went, and so did the fallback's own reading of an opened or shared
  file.
* **Progress** (counted lines, the ledger's measure): part 1 deletes seven files (693):
  `ExtensionHomeScreen`, `ExtensionListScreen`, `ExtensionViewScreen`, `CheckUpdatesScreen`,
  `AddonBox`, `ExtensionKeywordChip` and `ExtensionMaintainerChip`; and 42 more in `Routes.kt`
  (14), `Extension.kt`'s `generateUpdateUrl` (24) and `ExtensionMeta`'s `getUpdateJsonPair`
  (4). It adds `SettingsExtensions` (74), its string ids (25), 3 lines in `RustSettingsView` and
  4 in `SettingsIcons`. Part 1: 735 lines deleted, 106 added. Part 2 deletes
  `ExtensionImportScreen` (294) and `ExtensionExportScreen` (56), and 46 more in
  `FlorisAppActivity` (21), `Routes.kt` (19), `CacheManager` (3, before fix 9 added 4) and
  `ExtensionPolicy` (3). It adds `ExtensionFileWork`
  (84), 20 string ids, 10 lines in `SettingsEntry`, 4 in `SettingsExtensions`, 4 in
  `CacheManager` and 1 each in `SettingsFiles` and `RustSettingsView`. Part 2: 396 lines
  deleted, 124 added; the ledger is 421 files and 34,201 counted lines (part 1 left 422 and
  34,473). Part 3 deletes `ExtensionEditScreen` (812), `ExtensionEditFilesScreen` (260),
  `ExtensionComponentView` (160), `ExtensionDialogProperty` (31), `RadioListItem` (25) and
  `ExtensionNotFoundScreen` (21), and 35 more in `FlorisAppActivity` (20), `Routes.kt` (9) and
  `ThemeEditorActivity` (6). It adds `SettingsExtEditor` (151) and `ExtensionEditorWork` (121),
  30 string ids, and 7 lines in `SettingsFiles`, 5 in `RustSettingsView`, 4 in `CacheManager`, 2
  in `SettingsIcons` and 1 in `SettingsThemeEditor`. Part 3: 1,344 lines deleted, 321 added;
  before the rebase the ledger was 417 files and 33,178 counted lines. Step 10 in all: 2,475
  deleted, 551 added.

### 11. Setup (done)

* **Deletes:** `FB/app/setup`: 2 files, 266 lines.
* **What changed:** Rust draws setup: FlorisStepLayout's four steps with their headers, the
  open step's text and button, and the footer. Rust also picks the first screen a fresh
  activity opens, with a new settings op, by the same rules as before: setup until it is
  finished, a settings link even during setup, and setup again while the notification question
  is unanswered. It works out each step from the input method and permission facts Kotlin
  reports, decides when to ask for the notification permission and what the answer means
  (answers come back through a second new op), and finishes setup. Kotlin keeps only the
  permission request, the permission's state, the notification settings, bringing the app back,
  and noticing input method changes. Notifications already allowed skip the question, a denial
  for good opens the notification settings, TalkBack reads each step's number and state, and
  coming back with a reopened step done opens the next one. See
  [stage 4: Setup](stage4-setup.md). Commits d7cbfcb08, 5525e982d and 13b421c97, and the docs after them.
* **Blocked by:** nothing.
* **The fallback loses:** setup. The route stays as an empty entry that hands back to Rust, as
  in step 2; it gains the deep link `setup` so the fallback can hand it back while it is the
  start destination.
* **Progress** (non-blank lines deleted): `SetupScreen` (240), `NotificationPermissionState` (26,
  its one-line enum moved into `AppPrefs.kt`), `FlorisStepLayout` (283, only setup used it),
  `SettingsEntry.rust` and its comment (8 removed, 3 added), 26 lines of `FlorisAppActivity`'s
  setup routing (15 added), and 4 lines in `Routes.kt` rewritten as 4. The Kotlin glue adds
  `SettingsSetup` (71), its string ids (30), 7 lines in `RustSettingsView` (1 removed) and one
  in `SettingsStringIds` (1 removed). Step 11 is done: 590 lines deleted, 133 added.

### 12. The Compose settings host

* **Deletes:** `Routes.kt` (270 after step 6), the Compose branch of `FlorisAppActivity`,
  `ComposeSettingsFallbackActivity`, `FB/app/apptheme` (3 files, 142 after step 3), the
  settings-only parts of `FB/lib/compose` (6 files, 416 after step 6), `EnumDisplayEntries` (158
  after step 6), and the Jetpref UI dependency.
* **Blocked by:** nothing in settings: steps 4–11 are done, and since step 10 the fallback
  draws no screen. Every route left in `Routes.kt` is an empty entry that hands back to Rust,
  and Rust's `compose` effect (`settings_ui/mod.rs`, for a route Rust does not draw) opens a
  blank fallback; it goes with the fallback, and an unknown route should then be said in Rust.
  `ThemeEditorActivity` stays: it is the theme editor's own activity over the extension
  editor, not a Compose host. Shared primitives in `lib/compose` (20 files, 1,832 after step 3)
  are also used by the Compose keyboard (17 keyboard files import them) and wait for step K.

### K. The Compose keyboard

* **Deletes:** the inventory's `keyboard` scope. The [package 8 dry run](stage3b-package8.md#deletion-dry-run)
  removed 35 fallback files, 6,552 lines then, and compiled. Repeat the dry run first.
  Then the keyboard switch (`keyboard__rust_renderer_prototype`) goes the same way as the
  settings switch in step 2.
* **Blocked by:** four phone checks, all still owed:
  1. a real TalkBack session on the phone. Reading order (grade item C13) is fixed and
     checked without TalkBack: Rust lists the strip before the keys and sends a hint for every
     node, and the emulator probe shows the order and hints arriving
     ([reading order](../scripts/render-benchmark/README.md#reading-order)). Still owed on
     the phone: that TalkBack's swipes follow it, that touch exploration finds a held key's
     choices over the keys, the one-handed side panel's place in the order, and right-to-left
     layouts, which still read left to right;
  2. the Samsung font check;
  3. font scale 2.0;
  4. the CPU gate: warm scene p95 at most 2 ms, on the S24 Ultra at 120 Hz. The emulator
     has passed it since 2026-11-17 (p95 1.257 and 1.107 ms, [results](cpu-p95-results.md));
     the phone measurement that settles it is in [decisions](decisions.md#the-keyboards-cpu-gate-stays-2-ms-settled-on-the-phone-at-120-hz-2026-11-17).
* **The fallback loses, meanwhile:** every keyboard feature added after 2026-11-16, under
  the freeze above.

After 12 and K, Compose itself can leave the build, except the Snygg host files the
inventory keeps (`SnyggBox`, `SnyggSurfaceView`, `SnyggUi`, `SnyggPalette`), which stage 5
package 2 replaces.
