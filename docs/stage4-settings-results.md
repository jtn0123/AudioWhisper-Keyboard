# Stage 4 settings prototype checkpoint

## Status and boundary

**Follow-up 2026-11-14:** see [the acceptance follow-up](#acceptance-follow-up-2026-11-14)
for current measurements, fixes and the renewed device-tool hold. Step 3 is still incomplete.
The original checkpoint below is retained as history. The 90% target has been replaced by
[logic and all application drawing in Rust](rust.md).

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

## Historical next checkpoint (superseded by the follow-up)

Resolve the device input ownership/targeting failure with an authorized working device
session, then install the final APK and complete every missing runtime row above. Review
font scale 1/2, dark/light, landscape, clipping, selection and Compose fallback/deep-link
returns before broadening the port. Do not delete either Compose implementation before
phone acceptance.

Before the full port, agree on the additional scope needed for 90%, visual tolerance for
Material widgets/selection handles/predictive animation, and the policy for oversized or
unsupported restored drafts. No full-port implementation is included in this checkpoint.

## Acceptance follow-up (2026-11-14)

**A and the stage 5 research are complete; step 3 acceptance is still held.** Only
`aw_keyboard_ssd` / `emulator-5556` was used. The stock-IME authorization got past the
original typing ownership refusal, but clearing the Compose preview field remains unreliable.
No full settings port, fallback deletion or default change was made. Current ownership and
percentage policy is in [rust.md](rust.md); [stage 5 inventory](stage5-inventory.md) contains
research only. Receipts below are relative to ignored `build/stage4-followup/`.

### Runner validation and fixes

The runner parsed successfully and its number-row/font-size initial assumptions were checked
against the captured run-as baseline before execution. A real Compose dialog then exposed
an agent-device targeting failure: `Ref @e27 has no parent-owned touch point outside its
interactive descendants`. Following its specific-child/selector advice succeeded. The
runner now presses a complete role/label selector; two regression cases failed before this
change and pass afterwards (`runner-before-fix.log`, `runner-final-tests.log`). Stock IME
selection additionally checks `ime list -a -s` and Android's system-package list, admits only
the Google/AOSP Latin keyboard, and restores/verifies our IME in `finally`. Two more tests
cover system-keyboard selection/refusal. No helper IME is selected.

Screenshot review found a short list dialog occupying nearly the full surface. Native
measurement now uses its actual painted row heights, centers the content-sized dialog and
bounds it to the viewport. Radio nodes also lacked checked state; they now expose checked
and selected consistently. Both tests fail on the old source (`dialog-both-before-fix.log`)
and pass after the correction. Reviewing the installed correction later caught a regression
introduced by that checked-state change: the generic widget painter drew switch tracks on
radio rows as well. `radio_checked_semantics_do_not_draw_switch_tracks` fails before its
one-line role guard and passes afterwards. **This last drawing correction was made after
the device stop and is host-tested only; it was not installed or recaptured.** All six native
settings scene tests pass (`dialog-final-local-tests.log`). Expected assertion/panic output
is also retained under `/Volumes/2TB/aw-scratch/stage4-*-before-fix.log` and the named
`stage4-dialog-*-expected-regression*.log` files.

Harness checks: four settings-runner tests, two mute tests and five clipboard/GIF tests pass.
`python3 -I -m unittest test_clipboard_gif` cannot import a current-directory module in
isolated mode; the same five tests pass with `python3 -I -m unittest discover -s
scripts/render-benchmark -p test_clipboard_gif.py`. No test was disabled or skipped.

### Device results and failures

| Obligation | Actual follow-up result |
| --- | --- |
| Complete run-as preference byte parity | **Incomplete.** Compose executes number-row/list/slider confirm/cancel, scroll/back and the real IME picker, then types the synthetic field fixture. Clearing fails before either renderer's complete-file receipt; no `parity.json` success exists. Host stored-file tests remain separate proof. `parity.log`, `parity-font1.log` |
| Home/Keyboard/dialog pictures in light/dark/font scale 2 | Eighteen empty-field pictures captured from the installed correction and reviewed. Light/dark use font scale **0.94** (agent-device's `medium` preset); the accessibility set was verified **2.0**. Original scale **1.0** was restored. Home colours/footer are close; keyboard row density, switch styling, nonlinear font scaling and radio placement differ. Dialog sizing is improved, but captures show the subsequently fixed extra-switch drawing regression. **Final-source dialog pictures and visual acceptance remain owed.** |
| Semantic roles/state/labels and modal isolation | Native pictures' trees show switch/button/edit/seekbar/radio transport; modal trees expose only modal controls. All three captured dialog trees have RadioButton roles with one checked/selected choice and no underlying Number-row/picker nodes (`modal-semantics.json`). This does not prove every control's action/reachability. |
| Actual TalkBack | Enabled through Android Settings, permission accepted while streams stayed muted. Closing our agent session let TalkBack bind with touch exploration enabled (`talkback-after-close.txt`). Reopening through the bound agent command left it enabled but unbound, touch exploration false (`talkback-after-open.txt`); the earlier dump also shows UiAutomation. This indicates automation interference. **No actual TalkBack control-reachability/operation or hover-exit acceptance is claimed.** Android hover injection is not exposed by the documented agent command. |
| Stock-IME text entry | Google system LatinIME selected and verified. Compose typed 15 UTF-16 units with the expected digest; one `--debug` clearing retry passed (`field-diagnostic/typed-digest.json`, `result.json`). A full trace then reproduced the focus refusal with the stock IME, so device work stopped. Rust typing, live composition, selection/delete, rotation mid-composition and background-process-death restore **not completed**. Native editor/reference tests do not substitute for these. |
| No field contents in logs | No filled tree/screenshot or typed contents were retained by our runner. The app-UID log buffer contained **zero** occurrences of the synthetic text fixture at cleanup; only digest/count metadata was saved (`device-cleanup/extra-state.json`). Scope is this tested fixture/buffer, not a blanket privacy proof. |
| Back | A short edge pan preserved the open modal; a completed edge gesture dismissed it without a preference write; plain back returned Keyboard to Home (`back/result.json`). This observes gesture outcomes, not every predictive-animation phase or composition/back interaction. |
| Rotation/process death | **Not performed**, following the recurrent input-tool stop. No force-stop/fresh-launch is labeled process-death restore. |

The stock keyboard was
`com.google.android.inputmethod.latin/com.android.inputmethod.latin.LatinIME`.
The first clear failure was:

> `Error (COMMAND_FAILED): Field focus was refused`

The tool's advice was `Retry with --debug and inspect diagnostics log for details`.
That one diagnostic retry passed at restored font scale 1.0. The unchanged full trace then
failed at the same clear step with the same error, diagnostic **`mwgvbetc-2e64f130`**
(first failure **`mwgv837f-4b735088`**). Further device tests stopped; no input fallback or
new emulator was used. The earlier `KEYBOARD_OVERLAY_BLOCKING` error did not recur for
stock-IME `type`, but this is still not a reliable full text-field proof.

The first cleanup correctly refused completion because `medium` had set font scale to 0.94;
`large` restored the actual original 1.0. Repeated final file/secure/system readbacks pass.
This mismatch is recorded in `cleanup-first-difference.json`, not treated as restoration.

Picture paths, all prefixed by `build/stage4-followup/pictures/`:

| Appearance | Compose | Rust |
| --- | --- | --- |
| Light Home | `light/compose-home.png` | `light/rust-home.png` |
| Light Keyboard | `light/compose-keyboard.png` | `light/rust-keyboard.png` |
| Light dialog | `light/compose-dialog.png` | `light/rust-dialog.png` |
| Dark Home | `dark/compose-home.png` | `dark/rust-home.png` |
| Dark Keyboard | `dark/compose-keyboard.png` | `dark/rust-keyboard.png` |
| Dark dialog | `dark/compose-dialog.png` | `dark/rust-dialog.png` |
| Font 2 Home | `font2/compose-home.png` | `font2/rust-home.png` |
| Font 2 Keyboard | `font2/compose-keyboard.png` | `font2/rust-keyboard.png` |
| Font 2 dialog | `font2/compose-dialog.png` | `font2/rust-dialog.png` |

The corresponding `.json` trees contain empty fields only. The older before-dialog-fix
captures live in `pictures-before-dialog-fix/`; neither set is final-source visual approval.

### Controlled baseline, timing and memory

The prototype is a runtime Jetpref switch, not a separate compile variant. Both paths ran
from the **same installed debug APK**, initialized from the same saved preference/store files
except that switch; stock keyboard switching was confined to text tests. A second
`assembleDebug` from the same unchanged inputs reproduced byte-identical APK and `.so`
(`fixed-size.json`, `baseline-rebuild-size.json`, `baseline-rebuild.log`). Thus the
prototype-off/on packaged size delta is **zero**. This measures runtime path cost, not the
code-size overhead relative to a build with the prototype implementation deleted.

The installed measurement APK is **61,929,139 bytes**, SHA-256
`09946b2d81b0a6a0a9f1d26de955a58a120733ca787c567d78f7295a0d82195d`;
its arm64 `libawnative.so` is **10,034,824 bytes**, SHA-256
`ca9f01858e5d292c40eb16f9761be8bf056fdaa489f4a5551ae23bb5c347f4d3`.
These receipts precede the last host-only radio drawing correction. The final gate builds
an **uninstalled** APK of **61,929,143 bytes** (SHA-256
`2cbf0c2982e21add10d0f3716f4f405a1044746e0baf71e85a78b9d0e77ae522`),
with `.so` **10,034,792 bytes** (SHA-256
`7116ad8147c76411c59d17cc65a57b4de370f23d78cd6504af421e5393888754`),
recorded separately in `final-built-size.json`. Do not attribute its tiny artifact difference
(including build-commit metadata) to runtime performance. The earlier
**-3,870,136-byte** delta compared unrelated pre-task/final artifacts with fixture/packaging
changes and no build-input equivalence; it is not a prototype size saving.

Build was idle during four ordered Compose/Rust/Rust/Compose scroll trials. Each trial used
three 600 ms up/down pan pairs on Keyboard; font scale 0.94, light mode, 1080×2400 AVD,
TalkBack disabled, the original stores and verified mute levels. Separate native and HWUI
metrics cannot establish a like-for-like end-to-end rendering speedup. No Perfetto presented-
frame/FrameTimeline comparison was completed, so that acceptance proof remains owed.

| Trial | Measured boundary | Frames | p50 ms | p95 ms | Total PSS KiB | Total RSS KiB |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Compose 1 | HWUI intended-vsync to completion | 119 | 38.885 | 41.093 | 100,085 | 237,616 |
| Rust 1 | native scene/raster CPU draw | 356 | 2.263 | 3.838 | 183,064 | 319,768 |
| Rust 2 | native scene/raster CPU draw | 355 | 2.214 | 3.286 | 185,950 | 322,680 |
| Compose 2 | HWUI intended-vsync to completion | 119 | 17.235 | 21.308 | 100,097 | 237,588 |

Rust touch-to-buffer-post (234 distinct timestamps per run) was p50/p95 **29.416/50.188 ms**
and **30.424/59.153 ms**. Compose HWUI draw-recording p50/p95 was **0.518/0.983 ms** and
**0.548/0.918 ms**. These CPU/post/completion boundaries are different; no GPU or presented-
frame claim follows. Rust PSS is about **81–84 MiB higher** here; no memory improvement is
claimed. The measurements do not by themselves identify a leak. Raw numeric logs and full
meminfo are under `performance/`, summarized in `performance/summary.json`.

Cold Activity first-Displayed times were Compose **426 and 804 ms**, Rust **564 and
585 ms**. The first Compose trial recreated again (+383 ms) after its first displayed
frame; it is not a stable-ready time. Rust first buffer post minus the logged Activity START
was **598.295 and 661.040 ms**. Activity Drawn and native buffer-post are separate events;
no universal first-visible/presented-frame claim is made. `first-frame/summary.json` and
per-trial monotonic logs retain both boundaries and the extra Compose recreation. Android
`am force-stop` was used only to create cold-start timing trials, never as restoration proof.

### Restoration, checks and remaining contract

`device-cleanup/cleanup.json` verifies the original preference/learned/kept file SHA-256s,
accessibility services `null` / enabled `0`, exact enabled IMEs and our selected component
`com.jtn0123.audiowhisper.keyboard/dev.patrickgold.florisboard.FlorisImeService`.
`extra-state.json` also verifies font scale 1.0, sensor rotation/user rotation 1/0, light mode,
all original global animation/DND settings, unchanged granted RECORD_AUDIO permission,
restored empty numeric profiling log properties, and silent streams 1/2/3/5 = 0, 4/10 = 1.
No microphone or audible playback was used. Final crash buffer is empty; an unrelated
Bluetooth Hardware Error 0x42/SIGABRT observed after boot is saved at
`/Volumes/2TB/aw-scratch/stage4-talkback-crash.log`. No app/native runtime crash was observed.

`device_close(hostId=local, deviceId=emulator-5556, shutdown=true)` succeeded.
`device-cleanup/adb-devices.txt` confirms 5556 absent. The reserved emulator was not operated.

The exact required full gate passed before the A push and after the initial dialog fixes:
**826 JVM tests**, all XML modules counted: app 496 (119 XML), lib/rust 173 (48), lib/snygg
128 (33), lib/kotlin 29 (3); lib/android, lib/color and lib/compose have zero test XML.
Only AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport lint IDs,
zero compiler warnings, Rust formatting/Clippy/tests/oracle and native declaration guard pass.
The post-stop drawing guard passes the final same gate: exit **0**, **3m45s**;
receipt `gate-final.log` / `gate-final-audit.json`. No warnings/lints/tests are suppressed.
All changed code files are below 500 total lines, including comments and blanks.

To close step 3, first obtain an agent-device path that preserves a bound TalkBack service
and reliably focuses/clears this same-package editor. Then complete full run-as byte parity,
actual TalkBack reachability/actions/hover exit, native text composition/selection/delete,
rotation/process-death restore, final corrected dialog pictures and comparable presented-frame
measurements. Visual tolerance for Material controls, font scaling/selection handles and
predictive animation still needs an explicit ruling before the full settings port. The
inventory also identifies the native-loader-failure crash UI bootstrap contract for stage 5.
