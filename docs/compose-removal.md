# Removing the Compose UI

2026-11-16. Rust now draws the keyboard and settings by default ([decision](decisions.md#rust-draws-by-default-2026-11-16)).
This page is the order in which the Compose code that Rust replaced goes away, and the rule
for touching it until then. The [stage 5 inventory](stage5-inventory.md) is the source for
which files are covered: its ledger "Files already covered by settings/Compose-keyboard work"
lists every one with its size and scope (`stage4` = settings, `keyboard` = the Compose
keyboard). This page does not repeat that list. It orders it and names what blocks each step.

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
  Rust (Physical keyboard from Other, Home on Back).
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
    keeps the 8 enums the remaining Compose screens still ask for (App theme, colour
    representation, keyboard after dialogs, language names, shift state, keyboard mode,
    Snygg level, theme mode). The other 23 entries' strings all stay: Rust reads them
    through `SettingsStringIds`;
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

### 4. Other and Theme, with the colour picker

* **Deletes:** `OtherScreen` (95) and `ThemeScreen` (118), and Jetpref's colour picker use.
* **Blocked by:** Rust widgets that do not exist yet (see
  [follow-up 4](stage4-followup4.md)): the colour picker's saturation/value plane, hue bar,
  gradient slider tracks and hex fields; a time picker dialog for the day and night theme
  times; theme labels from the extension index; the add-on reference box.
* **The fallback keeps:** both screens. Their Compose versions are the only ones until then.

### 5. Backup and restore

* **Deletes:** `BackupScreen` (362) and `RestoreScreen` (265). `BackupIO` (81) is file
  input and output and stays as an adapter.
* **Blocked by:** file picker (SAF) plans, progress and error screens in Rust.

### 6. Languages and layouts

* **Deletes:** `FB/app/settings/localization`: 5 files, 1,078 lines (the list, locale
  picker, language pack manager and subtype editor with its state).
* **Blocked by:** data-driven lists and a per-item editor over the subtype list in Rust.

### 7. User dictionaries

* **Deletes:** `UserDictionaryScreen` (385).
* **Blocked by:** an editable list over the dictionary database, and import/export.

### 8. Project licence

* **Deletes:** `ProjectLicenseScreen` (65).
* **Blocked by:** read-only text selection in the Rust settings text (about 2–3 days).

### 9. Theme manager and theme editor

* **Deletes:** the rest of `FB/app/settings/theme`: 16 files, 3,802 lines in all, including
  `EditPropertyDialog` (1,200) and `ThemeEditorScreen` (473).
* **Blocked by:** step 4's widgets, and a Snygg rule and property editor in Rust.

### 10. Extensions

* **Deletes:** `FB/app/ext`: 14 files, 2,590 lines (import, export, edit, view, lists).
* **Blocked by:** file pickers (step 5) and per-file editors in Rust.

### 11. Setup

* **Deletes:** `FB/app/setup`: 2 files, 266 lines.
* **Blocked by:** a step layout in Rust, and the notification-permission request plan.
  Setup is first-run only, so it goes last among the screens.

### 12. The Compose settings host

* **Deletes:** `Routes.kt` (295), the Compose branch of `FlorisAppActivity`,
  `ComposeSettingsFallbackActivity`, `FB/app/apptheme` (3 files, 142 after step 3), the
  settings-only parts of `FB/lib/compose` (7 files, 509 in all), `EnumDisplayEntries` (220
  after step 3), and the Jetpref UI dependency.
* **Blocked by:** steps 4–11. Shared primitives in `lib/compose` (20 files, 1,832 after step 3) are
  also used by the Compose keyboard (17 keyboard files import them) and wait for step K.

### K. The Compose keyboard

* **Deletes:** the inventory's `keyboard` scope. The [package 8 dry run](stage3b-package8.md#deletion-dry-run)
  removed 35 fallback files, 6,552 lines then, and compiled. Repeat the dry run first.
  Then the keyboard switch (`keyboard__rust_renderer_prototype`) goes the same way as the
  settings switch in step 2.
* **Blocked by:** four phone checks, all still owed:
  1. a real TalkBack session on the phone;
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
