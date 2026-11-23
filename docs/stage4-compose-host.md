# Stage 4: the Compose settings host

This page covers step 12 of [the Compose removal plan](compose-removal.md). Since step 10 Rust
has drawn every settings screen, and the Compose fallback activity behind it drew none: each
route left in `Routes.kt` was an empty entry that handed back to Rust. This step deletes that
host. `FlorisAppActivity` is now a thin Rust settings host, and a link to a page Rust does not
draw opens a Rust page that says so, where it used to open a blank Compose activity.

| Route | Drawn by | Notes |
| --- | --- | --- |
| `settings/not-found` | Rust | New here. The page a link to an unknown route opens. |

## The not-found page

Rust's `navigate` used to answer a route it does not draw with a `compose` effect, and
`FlorisAppActivity` opened `ComposeSettingsFallbackActivity` on it. Since step 10 that activity
was blank. Now Rust pushes its own page instead
([`catalogue/not_found.rs`](../lib/rust/crates/render/src/settings_ui/catalogue/not_found.rs)):

* the title *Page not found*, a line saying this version of the app has no such settings page,
  and a button to the settings home, which opens home with nothing under it;
* Back returns to the screen under it;
* it never shows the link's route. Another app chose that text;
* a second unknown link while it is open keeps one page;
* it is a native route like any other, so rotation and process death reopen it, and
  `SettingsUiOp.ROUTES` lists it;
* `@enable` stays an effect (the keyboard's enable screen), not a page.

Three strings are new (`settings__not_found__title`, `__message`, `__home`). Their ids are in
`SettingsSetupStringIds`, because `SettingsStringIds` is at 498 of its 500 counted lines.

## Every way into settings

Each row is tested by `SettingsEntryTest`, which drives the renderer from the intent the way
`FlorisAppActivity` does: `SettingsEntry.route` reads the intent, `SettingsEntry.request` picks
`START` for a fresh activity or `NAVIGATE` for a later one, and the test reads the stack Rust
saves. *Later* means the intent reaches an open activity (`singleTask`, so `onNewIntent`).

| Entry point | Intent | Fresh | Later, over open settings | Test |
| --- | --- | --- | --- | --- |
| Launcher icon | `MAIN` to `SettingsLauncherAlias` | setup until it is finished, then home | no move | `everyWayIntoSettingsLandsOnItsRustScreen`, `theLauncherAndTheKeyboardsSettingsButtonOpenSetupUntilItIsFinished` |
| Android's keyboard settings and spell-checker settings | `MAIN` to the alias, which `method.xml` and `spellchecker.xml` name | as the launcher | as the launcher | the same, and `theManifestSendsEverySettingsIntentToTheRustHost` |
| The keyboard's settings key | no action, to `FlorisAppActivity` (`FlorisImeService.launchSettings`) | as the launcher | no move | `everyWayIntoSettingsLandsOnItsRustScreen` |
| Setup coming back after the keyboard is enabled | no action, to `FlorisAppActivity` (`SettingsSetup`'s `RETURN`) | — | setup stays where it was | `everyWayIntoSettingsLandsOnItsRustScreen` |
| A setup link | `VIEW ui://florisboard/setup` | setup, set up or not | setup | `everyWayIntoSettingsLandsOnItsRustScreen` |
| A settings link (dictation's model button) | `VIEW ui://florisboard/settings/dictation` | the screen over home, even during setup | the screen on top | `everyWayIntoSettingsLandsOnItsRustScreen`, `aSettingsLinkOpensItsScreenEvenDuringSetup` |
| The theme editor | `VIEW ui://florisboard/settings/theme/edit/…` to `ThemeEditorActivity` | the editor, as a root | — | `everyWayIntoSettingsLandsOnItsRustScreen` |
| A `.flex` opened | `VIEW content://…` with the extension type | the importer over home, set up or not | the importer on top | `everyWayIntoSettingsLandsOnItsRustScreen`, `anOpenedOrSharedFileStartsTheImporterOnIt` |
| A `.flex` shared | `SEND` with the file in the clip | as opened | as opened | the same |
| An unknown link | `VIEW ui://florisboard/anything` | the not-found page over home | the not-found page on top, once | `everyWayIntoSettingsLandsOnItsRustScreen`, `settings_ui_not_found.rs` |
| Rotation and process death | the saved stack | each of the above reopens | — | `recreationReopensTheScreensEachEntryLeft` |

`theManifestSendsEverySettingsIntentToTheRustHost` reads the manifest and the two input method
files: `FlorisAppActivity` holds exactly the three intent filters above and is `singleTask`, the
launcher alias targets it, both XML files name the alias, `ThemeEditorActivity` extends
`FlorisAppActivity` and is not exported, and no other activity takes a `ui://` link or the
extension type.

## What was deleted

Counts are non-blank lines, then counted lines (the ledger's, without comments).

| File | Non-blank | Counted | Why it could go |
| --- | ---: | ---: | --- |
| `FB/app/Routes.kt` | 219 | 199 | the empty Compose navigation graph |
| `FB/app/ComposeSettingsFallbackActivity.kt`, and its manifest entry | 5 | 4 | blank since step 10 |
| `FB/app/settings/dictionary/UserDictionaryType.kt` | 20 | 5 | only `Routes` used it |
| `FB/app/EnumDisplayEntries.kt` | 156 | 141 | Compose list entries; Rust's catalogue replaced them |
| `FB/app/apptheme/Shape.kt` | 24 | 9 | nothing used it |
| `FB/lib/compose/FlorisScreen.kt` | 141 | 124 | only the deleted host and graph drew it |
| `FB/lib/compose/PreviewKeyboardField.kt` | 138 | 123 | the same |
| `FB/lib/compose/FlorisDialogs.kt` | 61 | 46 | nothing used it |
| `FB/lib/compose/FlorisHyperlinkText.kt` | 45 | 30 | nothing used it |
| `FB/lib/compose/Validation.kt` | 42 | 27 | nothing used it |
| `COMPOSE/FlorisAppBar.kt` | 46 | 31 | only `FlorisScreen` used it |
| `COMPOSE/FlorisButtons.kt` | 124 | 109 | only `FlorisScreen` used it |
| `COMPOSE/FlorisButtonBar.kt` | 119 | 104 | nothing used it since an earlier step |
| `COMPOSE/FlorisCards.kt` | 244 | 225 | the same |
| `COMPOSE/FlorisChip.kt` | 81 | 66 | the same |
| `COMPOSE/FlorisCanvasIcon.kt` | 62 | 46 | the same |
| `COMPOSE/FlorisBulletSpacer.kt` | 41 | 26 | the same |
| `COMPOSE/DpSpSizeFuns.kt` | 35 | 20 | the same |
| **18 files** | **1,603** | **1,335** | |

`COMPOSE/` is `lib/compose/src/main/kotlin/org/florisboard/lib/compose/`; `FB/` is
`app/src/main/kotlin/dev/patrickgold/florisboard/`.

Also gone:

* `FlorisAppActivity`'s Compose branch: `setContent`, `AppContent`, the `LocalNavController`
  local, the fallback launcher and its result, the `allowRustSettings` switch (the last
  Compose settings switch) and the theme flow only Compose read. 213 counted lines to 104.
* `RustSettingsView`'s fallback, its `compose` effect, the unused `nativeRoutes` and the two
  wrappers `SettingsEntry.request` replaced. 281 to 275.
* Rust's `compose` effect, and the assertions in nine Rust test files and five JVM test files
  that a route was not handed to Compose. Where a test only checked that, it now checks the
  route is native; an unknown route now has to open the not-found page.
* `androidx.navigation:navigation-compose`, which only the graph used, and `lib/compose`'s
  `jetpref-material-ui` dependency, which none of its files imports.
* 495 baseline-profile lines naming the deleted classes, navigation's, and Jetpref's
  preference UI.

Lint's UnusedResources gate found nothing to delete. Every string and resource the deleted code
named is still read by Rust's catalogue, the string id maps, or the keyboard.

### What stays, and why

* **`ThemeEditorActivity`**: the theme editor's own activity over the extension editor that
  opened it, because `FlorisAppActivity` is `singleTask`. Rust draws it too.
* **`jetpref-datastore-ui`**: its preference screens are unused and R8 drops them, but the same
  artifact holds `observeAsState`, which 24 keyboard files read preferences with, the Rust
  renderer's Kotlin glue among them. The datastore itself (`jetpref-datastore-model`) is
  untouched. It goes with step K.
* **`FB/app/apptheme/Theme.kt` and `Type.kt`**: `FlorisCopyToClipboardActivity`, the image share
  target, draws with `FlorisAppTheme`, and `SettingsTypographyTokensTest` checks Rust's
  typography against `Type.kt`.
* **The Compose keyboard's primitives**: `COMPOSE/ClickableModifiers`, `FlorisAnimation`,
  `LazyGrid`, `RtlModifiers`, `ScrollableModifiers`, the forward-delete icon, and
  `FB/lib/compose/ImageVector` (and `SystemUi`). The ledger moves them from S (deleted at steps
  5–12) to K (deleted at step K). The nine Snygg UI files still marked S are the keyboard's
  too; another branch is moving Snygg into Rust, so their rows are left to it.
* **`SettingsUiOp.ROUTES`**: no production Kotlin reads it now that the fallback is gone; the
  JVM tests use it to check the native route list, as they do `STRING_KEYS`.

## Kotlin

| File | Counted lines | Change |
| --- | --- | --- |
| `FB/app/FlorisAppActivity.kt` | 213 → 104 | the Compose branch gone |
| `AW/settingsui/RustSettingsView.kt` | 281 → 275 | the fallback, `nativeRoutes`, `start` and `navigate` gone |
| `FB/app/SettingsEntry.kt` | 17 → 20 | `request`, the START or NAVIGATE choice, moved here from the activity so JVM tests and mutations reach it |
| `AW/settingsui/SettingsSetupStringIds.kt` | 28 → 31 | the not-found page's three string ids |
| 18 files deleted | 1,335 → 0 | above |

The ledger goes from 420 files and 33,483 counted lines (main at f2db3557d) to 402 files and
32,039. The two raises are hand edits with their reasons in the ledger's note column.

## Mutations

`scripts/settings_ui_mutation_cases/entry.py` adds 12 cases.

| Case | Mutation | Caught by |
| --- | --- | --- |
| `entry-unknown-opens-home` | an unknown link opens home | `settings_ui_not_found` |
| `entry-unknown-ignored` | an unknown link does nothing | `settings_ui_not_found` |
| `entry-enable-a-page` | `@enable` opens the not-found page | `settings_ui_not_found` |
| `entry-page-home-elsewhere` | the page's button opens About | `settings_ui_not_found` |
| `entry-page-message-dropped` | the message becomes a second button | `settings_ui_not_found` |
| `entry-fresh-link-dropped` | a fresh activity ignores its link | `SettingsEntryTest` |
| `entry-later-link-dropped` | a later link is ignored | `SettingsEntryTest` |
| `entry-link-prefix-kept` | the route keeps `ui://florisboard/` | `SettingsEntryTest` |
| `entry-link-any-action` | a shared `ui://` text reads as a link | `SettingsEntryTest` |
| `entry-send-reads-view` | SEND imports the intent's data, not its stream | `SettingsEntryTest` |
| `entry-method-settings-elsewhere` | Android's keyboard settings name the deleted fallback activity | `SettingsEntryTest` |
| `entry-link-filter-host` | the settings link filter takes another host | `SettingsEntryTest` |

All 12 were caught (`scripts/settings-ui-mutations.py <out> entry`). Each mutated run failed
on an assertion or a panic, not a build error: the five Rust cases in `settings_ui_not_found`
(exit 101), the seven JVM cases in `SettingsEntryTest` (exit 1). With the sources restored, the
JVM suites and `settings_ui_not_found` passed again. Every case in the package still matches its
source exactly once (552 cases).

## Measurements

Both builds are from the same tree: main at f2db3557d and this branch on top of it.

**APK size.** `personal` is the shrunk, optimised build that ships; `debug` is what the
emulator runs.

| Build | main | branch | Change |
| --- | ---: | ---: | ---: |
| `personal` | 56,220,818 B | 55,662,803 B | −558,015 B (−1.0%) |
| `debug` | 68,104,527 B | 66,104,628 B | −1,999,899 B (−2.9%) |

Almost all of it is Kotlin. In `personal`, `classes.dex` goes from 5,789,084 to 5,236,228 bytes
(−552,856, −9.5%); `libawnative.so` grows 688 bytes (the not-found page); resources shrink 98
bytes stored (navigation's licence entry left `aboutlibraries.json`); and the compiled baseline
profile grows 404 bytes, although 495 lines left the text profile. In `debug`, the dex files
shrink 1,998,512 bytes.

**Settings cold start**, on `emulator-5556` (Android 16): `am start -W` on the launcher alias
after a force-stop, so the process and activity both start cold (`LaunchState=COLD`, the process
down before each run). Gboard was the default keyboard while timing, because with ours the
system rebinds the keyboard service after a force-stop and warms the process first: one such run
took 451 ms where the rest took 900 to 1,000. The builds alternated, main, branch, main, branch,
each round one discarded run and three timed ones, at a host load of 11.7 to 13.8.

| Build | Runs (ms) | Median | Mean |
| --- | --- | ---: | ---: |
| main | 973, 982, 995, 987, 892, 950 | 977.5 | 963.2 |
| branch | 918, 966, 904, 979, 953, 899 | 935.5 | 936.5 |

The branch is 42 ms faster at the median and 27 ms at the mean, but the two ranges overlap
(892–995 and 899–979), so this is a small gain at most, not a proven one. That fits: on main the
activity already installed Rust's host at start (`allowRustSettings` was true), and its
`setContent` branch ran only in the fallback subclass, so the branch has no Compose work to skip
at launch.

## Improvements

1. **A link to a page Rust does not draw now says so.** It used to open a blank Compose
   activity with no way back but Back.
2. **The Compose settings host is gone.** That is 18 files and 1,335 counted lines of Kotlin,
   one activity, and the navigation library. The ledger goes from 420 files to 402.
3. **The shipped APK is 558 KB smaller**, with 9.5% less dex.
4. **Every way into settings is tested in one place.** `SettingsEntry.request` makes the
   choice the activity used to make inline. `SettingsEntryTest` drives the real renderer through
   it for each intent, and checks the manifest and the two input method files against it.
5. **Cold start**, 42 ms at the median: small, and within the runs' spread.

## Device check

On `emulator-5556` (`aw_keyboard_ssd`, Android 16), with the debug build installed by
`scripts/install-dev.sh` and driven by agent-device. Screenshots are in
`build/compose-host-check/` and are not committed.

| Check | Result | Screenshot |
| --- | --- | --- |
| Launcher | home | `01-launcher-home.png` |
| Dictation link, settings open on home | Dictation over home; Back to home | `02-dictation-link-later.png` |
| Dictation link, settings closed | Dictation; Back to home | `06-dictation-link-fresh.png` |
| Unknown link, settings open | the not-found page over home | `03-not-found-later.png` |
| A second unknown link while it is open | still one page: one Back reached home | — |
| Unknown link, settings closed | the not-found page; Back to home | — |
| Rotation on the not-found page, there and back | the page stays | `04-not-found-landscape.png` |
| Process ended on the not-found page, then relaunched | a new process reopened the page | `05-not-found-after-process-death.png` |
| *Go to the settings home* | home, with nothing under it: Back left the app | — |
| The keyboard's settings action, settings open on About | About, unmoved | `08-settings-key-later.png` |
| The same, settings closed | home | `10-settings-key-fresh.png` |
| Theme manager, *Create extension*, *Add*, *Edit* | `ThemeEditorActivity` with Rust's editor; Back to the extension editor; *Cancel* and *Discard* | `11-theme-editor.png` |
| A `.flex` opened from Files, settings open on Theme extensions | Android offered *Import Extension*; *Just once* opened the importer on top; Back to Theme extensions | `12c-after-tap.png`, `13-flex-view-importer.png` |
| The same file shared from Files, settings closed | the importer over home; Back to home | `14-share-sheet.png`, `15-flex-send-importer.png` |
| Setup from a fresh state | setup, step 1 | `18-setup-fresh.png` |
| Turning the keyboard on in Android's list | setup's return intent (no action) came back by itself, with step 2 open | `19-setup-returned.png` |
| Dictation link during setup | Dictation on top of setup; Back to setup | — |
| Draw the keyboard in Rust, off | the Compose keyboard drew in the probe editor | `16-compose-keyboard.png` |
| The same, on again | the Rust keyboard drew | `17-rust-keyboard-again.png` |

The renderer behind each keyboard was told apart by its surfaces. With the switch off, the input
method window had one `SurfaceView` (the Smartbar); with it on, two (the Smartbar and the
keyboard). The two keyboards look alike, but their hint digits sit a few pixels apart.

The fresh setup state did not need the app's data wiped. With the app stopped, the two setup
preferences (`internal__is_ime_set_up` and `internal__notification_permission_state`) were taken
out of the preference file, after a copy was made beside it, and the keyboard was disabled.
Setup went as far as step 2; Android's input method picker is a system window that agent-device
cannot read, and [stage4-setup](stage4-setup.md#proof) ran the whole first run.

Restored afterwards: the preference file byte for byte (the same SHA-256 as before), and the copy
deleted; the enabled input methods, compared as the identical string; our keyboard as the
default; the Rust keyboard switch on again; the fixture deleted from Downloads (empty before and
after); and the rotation (accelerometer off, portrait) and media volume (0, muted) as they were.
Against the copy `install-dev.sh` took of this emulator before the check, the preference file
differs only in `internal__version_last_use`, which the app writes when it starts; that copy was
taken while main's build was installed. No crash or ANR was logged for the app.

Driving notes:

* agent-device's plain `press` on a file in Android's Files app did nothing: no intent was sent
  and Files logged nothing. A press held for 80 ms (`--hold-ms 80`) opened it.
* `--relaunch` force-stops the app, and Android then switches the default keyboard to Gboard.
  The keyboard was set back each time.
* While these checks ran, another `install-dev.sh` run on another emulator rewrote this
  checkout's `.device-backup/EMULATOR36X5X11X0/settings.tar`. Every emulator reports that serial,
  so they share one backup (below).

## Owed

* **P6** in [owed-checks](owed-checks.md#phone-checks-the-docs-owe), a real TalkBack session
  on the phone for each settings screen, was meant to run before this step. It did not, and P6
  stays owed: the Compose host it would have compared against is gone, so whatever TalkBack
  finds wanting is fixed in Rust. The not-found page joins its list.
* **P27.** This step on the phone: every check above ran on the emulator only. One UI's keyboard
  and spell-checker settings name the launcher alias, and nothing here opened them.
* **E43.** No instrumented test sends the settings intents to `FlorisAppActivity` on a device.
  `SettingsEntryTest` drives the renderer through the same two calls on the JVM, and checks the
  manifest, but `onNewIntent` and `onCreate` are checked only by the hand-run check above.
* ~~`scripts/install-dev.sh` names each device's backup after `ro.serialno`, so emulators share
  one backup.~~ Fixed in 48b2e0018: an emulator's backup is named by its AVD
  (`.device-backup/avd-<name>`), since the adb serial changes with the port; phones keep their
  serial.
* Retiring `SettingsUiOp.ROUTES`, which only JVM tests read now. Its tests would read the route
  list some other way first.

## What step K needs from settings

* `jetpref-datastore-ui` goes with the Compose keyboard: `observeAsState` is the only part of it
  still used, by the keyboard's Kotlin.
* The `lib/compose` primitives the ledger now marks K, and `FB/lib/compose/ImageVector` and
  `SystemUi`.
* `FB/app/apptheme/Theme.kt` and `Type.kt` stay until `FlorisCopyToClipboardActivity` is drawn
  without Compose; `SettingsTypographyTokensTest` reads `Type.kt` until then.
* The nine Snygg UI files still marked S, which the theme branch is moving.
* Nothing else in settings uses Compose.
