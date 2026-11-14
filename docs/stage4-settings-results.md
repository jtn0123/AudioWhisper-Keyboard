# Stage 4 settings prototype checkpoint

## Status and boundary

Implementation checkpoint on 2026-11-14, based on `04186d1d`. The test-only JNI cleanup
and design were committed and pushed separately as `a47dbc89` and `08b49c4f`.
[Design and complete inventory](stage4-settings.md).

The default-off prototype is implemented. **Step 3 device acceptance is incomplete.**
The device command rejected text entry twice with `KEYBOARD_OVERLAY_BLOCKING`; the required
single diagnostic retry also failed, so device interaction stopped. No acceptance or
Compose deletion is claimed. Subsequent local corrections have not been installed or
visually checked. Compose settings and the Compose keyboard remain available and default.

## Implemented scope

Enable **Other > Rust settings renderer prototype** to select the new Activity surface.
Setup still uses Compose. Rust draws Home and the complete Keyboard preference screen,
including dependent visibility/enabled states, switches, linked switch/list dialogs,
single/paired sliders, Default/Cancel/OK and the scratch preview editor. The footer requests
the real Android IME picker. Input feedback links and all other routes, including add-ons,
extensions, dictionaries, backup/restore, dictation, theme management and the theme editor,
open the retained Compose Activity. Returning preserves native navigation state.

Native screen/widget/layout/editor/scroll state lives in `render::settings_ui`. It uses the
existing Vello CPU scene, Parley/Skrifa text, render worker, SurfaceView lifetime and damage
comparison. Keyboard panel preparation explicitly leaves settings scenes alone. Native
deep links coalesce Home/Keyboard to a bounded stack. Rust consumes pointer history and
identity, owns fling, hardware focus/reveal, predictive-back progress/cancel/commit and
modal drafts. Its versioned saved packet contains route, scroll, modal, focus and UTF-16
editor state; preferences reload independently. Android recreation/process-death behavior
still requires device proof.

Kotlin supplies resolved Android resources/icons, the existing Material You/app-theme
palette, density/font scale, insets and IME status. A minimal editable View forwards its
InputConnection to Rust. The shared AccessibilityNodeProvider transports native roles,
checked/selected/enabled state, range and editable actions; settings hover exploration
does not commit buttons. Kotlin applies write/reset effects to the original typed Jetpref
entries through one ordered queue. Defaults, admission and ranges reuse the settings crate.
File IO remains Android-owned; unported platform contracts continue on their Compose path.

## Local proof

| Obligation | Result / evidence under ignored `build/stage4-settings/` |
| --- | --- |
| Frozen Kotlin UI decisions | `SettingsUiMatchTest`: Home routes/platform plans; all 64 visibility/enabled combinations; 1,000 seeded switch/list/slider transactions; Default/reset; linked switches; focus reveal at font scale 2; bounded deep links; saved drafts and predictive cancellation |
| Independent editable model | `SettingsEditorMatchTest`: 10,000 seeded commands against the actual retained Compose `EditProcessor`, comparing every UTF-16 unit, selection and composition; invalid InputConnection requests reject without modifying state |
| Real Jetpref codec | `SettingsUiStoredFileMatchTest`: 20 traces exported through real Jetpref produce identical complete bytes for reference and native effects, including resets and cancellations. **Host proof, not the required run-as UI proof** |
| Native regressions | Three render tests cover touch/fling/cancel/pointer loss, nonfinite/out-of-range slider geometry without changing imported values, and modal-only semantics/cancelled drafts |
| UI/editor mutations | Four independent mutations (switch writes old value, Default stays open, surrogate deletion consumes one unit, predictive cancel commits back) each fail fresh JVM assertions; sources restored and the suite passes. `ui-mutations-final/result.json` |
| Native declaration guard | Step 0 mutation adds an unused unexported production declaration: gate task fails, exact restoration passes. `native-mutation/result.json`; final guard checks **357** declarations, missing **0**; normal APK fixture exports **0** (`final-gate.log`, `final-normal-apk.json`) |
| Required full gate | Exit **0**, 2m13s: all **826 JVM tests** (496 app / 173 wrapper / 128 Snygg / 29 Kotlin), no failures/errors/skips; Rust tests, formatting, Clippy and oracle pass. Only AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport lint IDs; zero compiler warnings. `final-gate.log`, `final-test-counts.json`, `final-lint-audit.json` |
| Harness regression tests | `test_muting.py`: 2/2; isolated `test_clipboard_gif` unittest suite: 5/5; all pass |

No existing test or assertion was disabled or removed. The new selector adds one catalogue
default and its independent reference entry; the defaults test now checks 144 entries.
The fixture-only callbacks remain absent from the normal APK. Android test Kotlin compilation
also passes; this does not claim instrumentation execution. All changed code files are below
500 total lines, including blanks/comments (`source-line-audit.json`).

## Observed device evidence and hold

Only AVD `aw_keyboard_ssd`, runtime `emulator-5556`, was opened through T3. The reserved
`emulator-5554` was never used. Installation used `scripts/install-dev.sh`. Audio was muted
immediately and verified with `cmd audio get-stream-volume`: streams 1/2/3/5 were 0;
4/10 were 1. The emulator process had `-no-audio`; no microphone or playback was used.

An intermediate build visibly rendered native Home and Keyboard. Other opened the Compose
fallback; the opt-in setting changed the Activity path. Empty-field accessibility trees
showed native button/switch/edit roles and visible checked/enabled states. These observations
do not establish TalkBack operation or parity for every control.

The first preview-field typing request failed with:

> `KEYBOARD_OVERLAY_BLOCKING: Android text input is blocked because the focused input belongs to the active keyboard/IME.`

The suggested `--debug` retry failed with the same error. Diagnostic IDs:
`mwgsyife-82df8cc9` and `mwgsyz5v-3695fa19`; evidence `field-smoke/type-debug.log`.
Whether this is package ownership, targeting or another cause remains unresolved. The
device-tool stop rule required ending runtime interaction after that retry.

| Requested runtime proof | Actual result |
| --- | --- |
| Byte-identical run-as preference files after identical screen actions | **Not completed**. A same-actions runner is supplied as `scripts/render-benchmark/settings-ui.py`, but its full UI sequence is unvalidated |
| Light/dark/font-scale-2 screenshot pairs | **Only preliminary light Home pair** reviewed; final-source, dark and font-scale-2 pairs missing |
| Accessibility trees | Compose Home/Keyboard/dialog and intermediate native Home/Keyboard captured; no complete parity audit |
| TalkBack reachability and operation | **Not completed**; TalkBack service was not enabled and speech/actions were not measured |
| Scrolling p50/p95, Activity first frame, PSS/memory | **Not measured**; no performance conclusion |
| Rotation, process-death restore, predictive-back gestures, hardware focus | Native state/logic tests pass; **Android runtime acceptance missing** |
| Final APK | Built and checked locally; **not installed** after the device hold |

Screenshot paths relative to repository root:

- `build/stage4-settings/pictures/baseline-compose-home-light.png`
- `build/stage4-settings/pictures/rust-home-second.png`
- `build/stage4-settings/pictures/preliminary-home-light-pair.png`
- `build/stage4-settings/pictures/baseline-compose-hint-dialog.png`

The reviewed Home pair showed a less contrasting native footer/background and differing row
spacing. Subsequent palette/background and wrapped-row measurement corrections are local-only
and need new screenshots.
`rust-home-first.png` is a Compose Other transition capture and is excluded as native proof.
The intermediate APK receipt is `observed-apk.json` (SHA-256
`b886dd3502adbd6e409ac75cd944b71498a3f6a91ff9813aa880ad388a929bed`).

Tree files: `compose-home-baseline-tree.txt`, `compose-keyboard-baseline-tree.txt`,
`compose-hint-dialog-tree.txt`, `rust-home-second-tree.txt`, `rust-keyboard-first-tree.txt`.
These are empty-field observations; editor content is excluded from diagnostics.

Cleanup restored the exact initial run-as preference file, learned-word and kept-as-typed
stores (hashes in `device-cleanup/cleanup.json`), font scale 1.0, accessibility services
`null`/enabled `0`, enabled IMEs and original project IME selection. RECORD_AUDIO was
already granted at baseline; no permission request was made. Silent stream values were
rechecked. `device_close(...shutdown:true)` succeeded and `device-cleanup/adb-devices.txt`
confirms runtime 5556 absent. No native crash, SIGSEGV or FATAL EXCEPTION occurred; runtime
log matches were routine AndroidRuntime startup/exit. No crash artifact was necessary.

## Size and source budget

| Artifact | Pre-task captured APK | Final normal debug APK | Delta |
| --- | ---: | ---: | ---: |
| APK bytes | 75,836,683 | 71,966,547 | -3,870,136 |
| Packaged arm64 libawnative.so bytes | 9,908,424 | 10,032,056 | +123,632 |

This compares the pre-task artifact receipt with the final assembled APK; it includes the
fixture cleanup and packaging changes and is not a controlled rebuild benchmark of the
prototype alone. Baseline build-input equivalence was not separately established. No frame
or memory improvement is inferred. Receipts: `before-size.json`, `after-size.json`.
Final APK SHA-256: `3d858096f8ed1a4a0bb8091b739e2ed06474e00e02241d7835640af507dbeab9`.
Final libawnative.so SHA-256: `7bceb63726663af9ca35fdd1dbfd32fcd3ad28e215a363e3793eaff09853409a`.
Android test Kotlin compilation also exits **0** (`final-android-tests-build.log`).

The supplied counter reports **53,105 Kotlin / 321 Java / 53,641 Rust = 50.10% Rust**
with this prototype staged as tracked source. Before work: 52,745 / 321 / 51,500 = **49.25%**.
After the fixture cleanup alone: **49.33%**. The checkpoint gains **0.85 percentage points**;
it retains all Compose fallbacks and does not establish a 90% port. Receipts: before-share.json,
after-share.json. The exact script includes latin-oracle; its documented denominator caveat
in the design still applies.

The complete-settings sizing range remains **59–63% Rust after accepted settings deletion**,
and **63–67% after also deleting the Compose keyboard**. Those are estimates, not delivered
results. This prototype retains both fallbacks. The 90% endpoint needs a separate inventory
and authorized scope beyond settings; adding Rust or removing tests just for the ratio is
not acceptable.

## Next checkpoint

Resolve the device input ownership/targeting failure with an authorized working device
session, then install the final APK and complete every missing runtime row above. Review
font scale 1/2, dark/light, landscape, clipping, selection and Compose fallback/deep-link
returns before broadening the port. Do not delete either Compose implementation before
phone acceptance.

Before the full port, agree on the additional scope needed for 90%, visual tolerance for
Material widgets/selection handles/predictive animation, and the policy for oversized or
unsupported restored drafts. No full-port implementation is included in this checkpoint.
