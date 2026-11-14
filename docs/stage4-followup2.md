# Stage 4 follow-up 2: prototype checkpoint

This package keeps the Rust settings renderer **default off**. It does not begin the full
settings port or remove Compose. The rulings are recorded in [decisions](decisions.md#settings-prototype-parity-and-bootstrap-2026-11-14-follow-up-2).

**Acceptance is incomplete.** The two action sequences and byte comparisons passed, and
Rust's accessibility audit has zero violations. The retained Compose path has 23 unlabeled
controls. The 51 picture pairs do not meet the strict visual threshold. Final drawing
refinements and their fresh screenshots/performance rerun are on a device-tool hold after
`aw_keyboard_ssd` went offline. This is an implementation checkpoint, not permission to
start the full port or delete a fallback.

## Implemented changes

Rust uses the actual Compose Material 3 theme roles, outlined off switches with 16 dp thumbs,
onPrimary on thumbs, linked-row dividers, matching row/text/control dimensions and Android's
resolved nonlinear sp values. Three-line trailing switches use M3 top alignment rather
than centring in the whole row. The picker has the same 48 dp target and icon inset. Headers, text indentation, dialog widths, radios, sliders and
button flow use the retained Compose implementation as the specification. At font scale 1,
main-row titles/summaries use Compose's one/two-line ellipsis limits. Opening a dialog
retains the background screen's own scroll and text geometry; its 60% dim also applies to
system-bar backgrounds and icon appearance. Font-2 title and
summary wrapping is intentional. Selection handles are primary-colour Rust teardrops; their
hit testing and drag edits live in Rust. Android provides the floating selection ActionMode.
The retained Floris 4 dp scrollbar uses the same 1850 ms initial hold, 950 ms fade and
Compose cubic easing; scrolled app bars use surfaceContainer. A test-only frozen Kotlin
reference compares Rust against the real Compose easing, including navigation after idle.
In-app Back progress/cancel is forwarded through AndroidX's OnBackAnimationCallback adapter;
the root disables its callback to leave the root animation to Android.

The platform crash adapter runs before preference initialization. Its diagnostic model has a
compiled class/method allowlist, never Throwable messages, causes, filenames or app state.
A JVM test injects secret-bearing exception data and forged frames. The device test forces
native bootstrap failure and verifies a Button/TextView screen without ComposeView. It clicks Copy details, checks
the allowlisted payload, and restores the prior ClipData entirely in memory.

The direct InputConnection tests found a recreation issue: native editor focus had been
mistaken for actual Android endpoint focus, causing an unsolicited IME restart to finish
composition. Those focus states are now saved independently. The accessibility runner also
found a Compose preview-footer lifecycle race after navigating Home; preview visibility now
updates on the owning screen's resume rather than a departing screen's SideEffect.

## Latency boundary and cause

Receipts are under ignored `build/stage4-followup2/`. `performance-queued-final/` and
`performance-unbuffered/` are four-trial Compose/Rust/Rust/Compose captures with the same
fixture, gestures, display, font scale and trace configuration, with builds idle. The earlier
`performance-queued/` is an interrupted capture and is not evidence.

| Rust phase, p50 / p95 ms | Batched input, first Rust trial | Unbuffered, first Rust trial |
| --- | ---: | ---: |
| MotionEvent timestamp → delivered to View | 19.919 / 36.807 | 1.951 / 4.383 |
| View delivery → native reply | 1.410 / 2.434 | 1.500 / 2.470 |
| Reply → Choreographer submit, joined first posts | 0.096 / 0.578 | 7.167 / 14.416 |
| Submit → worker start, joined first posts | 0.115 / 0.503 | 0.125 / 0.469 |
| Worker start → post, joined first posts | 6.883 / 8.147 | 8.635 / 9.908 |
| Scene/raster draw | 2.225 / 2.853 | 1.939 / 2.767 |
| ANativeWindow lock/dequeue | 3.966 / 5.276 | 5.293 / 6.958 |
| Buffer copy | 0.321 / 0.390 | 0.313 / 0.387 |
| Unlock/post | 0.405 / 0.501 | 0.399 / 0.525 |
| Unique delivered input → first buffer post | 29.070 / 45.692 (233 events) | 20.155 / 26.894 (206 events) |

The second Rust trials are 32.052 / 49.686 ms batched and 19.565 / 26.780 ms unbuffered.
Perfetto itself confirms the delivery interval: queued Rust 19.832/36.761 and
22.777/41.199 ms; unbuffered Rust 1.909/4.362 and 2.050/5.194 ms, each 240 events.
The queried slices are app-thread `deliverInputEvent src=… eventTimeNano=…` timestamps;
the same trial traces contain `aw.settings.input`, `aw.settings.frame`,
`aw.settings.scene_reply` and `aw.worker` slices (`trace-phase-summary.csv`).
The main lost interval was Android input batching before the View callback. The View now
requests unbuffered dispatch on ACTION_DOWN. Choreographer still coalesces scene updates;
worker scheduling, drawing and buffer dequeue explain the remaining post interval.
Batched delivery arrives near its submission opportunity; after unbuffering, the explicit
Choreographer wait is now visible (7.167 ms median), while the worker queue itself is only
0.125 ms median. The worker takes 8.635 ms through post, including the 5.293 ms buffer lock.
These are correlated first-event/frame joins; percentiles from different phases must not
be added as if they described the same event. Coalesced
input events without a distinct post are not assigned invented latencies. The logs contain
only timestamps/counters, and the traces contain aw-settings/aw-render phase slices.

**No presented-speed or app-jank claim.** Both paths have SurfaceFlinger DisplayFrame tracks,
but the software ANativeWindow path has no app SurfaceFrame timeline. The common measured
boundary is SF display-frame start → present, not input → present or app submission → present.
Queued trials give 4.952/6.932, 6.740/8.659, 6.731/8.519, 4.808/6.483 ms in capture order.
All display frames are marked Early Present / Prediction Error on this emulator. Compose's
app SurfaceFrames cannot be compared to missing native SurfaceFrames. A buffer-to-present
bridge or equivalent observable app boundary remains owed; phone acceptance must retain this
qualification. `scripts/settings-performance-summary.py` exports track counts and flags.

## Memory cause and accounting

The old path allocated a 4096×4096 RGBA glyph atlas: **64 MiB**, including for ordinary
outline text. It also copied the **8,723,912-byte (8.32 MiB)** font payload twice. Font data
now shares immutable Blob backing across registries, workers and rasters. Settings CPU outlines do
not allocate the bitmap atlas; settings colour glyphs render directly too, so entering emoji
cannot silently restore the 64 MiB allocation. The host colour probe paints emoji, flags,
ZWJ and skin-tone forms through cached and direct paths: 7,194 painted pixels; after the
same 3/255 threshold and one-pixel edge band, **zero flat-interior differences**.
`color-probe.log` and `benchmark/settings-color-probe.rs` preserve that check.

Matched warm PSS KiB (before the final colour-glyph refinement):

| Trial | Compose | Rust | Rust minus paired Compose |
| --- | ---: | ---: | ---: |
| Queued 0/1 | 103,637 | 106,436 | +2,799 (2.73 MiB) |
| Queued 3/2 | 103,938 | 105,502 | +1,564 (1.53 MiB) |
| Unbuffered 0/1 | 103,904 | 104,865 | +961 (0.94 MiB) |
| Unbuffered 3/2 | 103,683 | 104,396 | +713 (0.70 MiB) |

First queued matched category breakdown, KiB:

| dumpsys category | Compose | Rust | Delta |
| --- | ---: | ---: | ---: |
| Native Heap | 17,229 | 38,650 | +21,421 |
| Dalvik Heap | 28,423 | 7,867 | −20,556 |
| Dalvik Other | 11,967 | 11,427 | −540 |
| .apk mmap | 26,889 | 29,521 | +2,632 |
| .so mmap | 1,879 | 1,885 | +6 |
| Stack | 1,700 | 1,820 | +120 |
| .jar mmap | 2,174 | 2,092 | −82 |
| .ttf mmap | 117 | 51 | −66 |
| .art mmap | 9,185 | 9,205 | +20 |
| Other mmap | 3,330 | 3,174 | −156 |
| Unknown | 679 | 679 | 0 |
| Other dev / Ashmem / .dex / .oat | 16 / 11 / 13 / 25 | 16 / 11 / 13 / 25 | 0 |
| Graphics / GL | 0 reported | 0 reported | 0 reported |

The emulator does not attribute a useful Graphics/GL PSS category; this is process PSS,
not a total device graphics-memory claim. The Rust Activity does not call setContent or construct a ComposeView; theme token/value
classes and a few Activity snapshot states remain linked during this prototype. The lower
Dalvik heap is consistent with avoiding the Compose settings composition, not deleting all
Compose dependencies. Allocation counters and PSS are different measurements. The native
heap stages separately measure
8,741,952 bytes for font registration, 321,504 for scene/registry preparation,
10,025,648 for raster construction (9,823,680 = 9.37 MiB is the one full-screen pixmap),
and 1,547,808 bytes for the first draw. There is no retained second full-screen Rust pixmap.
Remaining total overhead is below +10 MiB in all four matched pairs. The historical cold
81–84 MiB excess is not compared directly to a warm run; the removed allocations explain it.

## Reproducing device tests

Use only T3-opened `aw_keyboard_ssd`, with its returned command in AGENT_DEVICE. Current
runtime was `emulator-5556`; the reserved `emulator-5554` was never addressed. Capture an
immutable private baseline once, then run:

```sh
export ANDROID_SERIAL=emulator-5556
export AGENT_DEVICE=/path/returned/by/T3/agent-device
scripts/settings-device-tests.sh "$ANDROID_SERIAL" .device-backup/stage4-followup2 all
```

The wrapper runs these commands (no connected* or installDebug task):

```sh
./gradlew -PsettingsFixture=true assembleDebug assembleDebugAndroidTest
scripts/install-dev.sh emulator-5556
adb -s emulator-5556 install -r -t app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
adb -s emulator-5556 shell am instrument -w -e mode ui com.jtn0123.audiowhisper.keyboard.test/com.jtn0123.audiowhisper.settingsui.SettingsInstrumentation
```

Modes `text`, `death`, `crash` can run independently; the wrapper's `text` mode runs both
Compose and Rust InputConnections and compares the full files separately. The host Python
controller restores the baseline before each renderer, returns the prototype selector to
false on both paths, and compares real bytes, without sorting/normalizing preference lines.
No IME is involved in the direct editor tests. The death controller arms the fixture through
instrumentation, finishes instrumentation, backgrounds the actual Activity, then uses am kill
(or same-UID SIGKILL if Android retains the IME process) and reopens the saved task. The fixture
compares route, UTF-16 digest, selection, composing range and distinct PID. Synthetic editor
contents are never written into receipts or screenshots.

UiAutomation temporarily owns the accessibility connection; the task's agent session must
close before it starts. It walks every prototype tree, focuses/clicks items, exercises sliders
and scrolls, and compares screen facts. The suite is opt-in and outside the JVM gate. A real
TalkBack interaction and speech run remains owed for the user's phone session.

## Device results and exact commands

All receipt paths below are relative to `build/stage4-followup2/`. The latest complete UI
receipt is `instrument-ui-a11y.log`: every edit operation passes; the controller exits 1
because the Compose accessibility findings are **not waived**. Both pickers were observed.

| Check | Result | Receipt |
| --- | --- | --- |
| Home navigation | All 12 routes exercised on both paths | `instrumentation/{compose,rust}-ui.json` |
| Keyboard and 14 dialogs | 22 tree checkpoints per renderer; linked toggles, last choice in each list, both slider axes, set-progress, forward/back scroll, Default/Cancel/OK | same JSON files |
| Accessibility assertions | Rust **0**; Compose **23 missing labels** (linked switches and slider nodes) | `instrumentation/accessibility-result.json` |
| Native hover exit | Accessibility focus clears; does not activate the row | `instrumentation/rust-ui.json` |
| UI preference bytes | **1,698 identical bytes**; SHA-256 `8ed8163e0b4ef3fc1e42aac8bbcd047a094b59925dc67e97dee704694dbb89a8` | `instrumentation/preference-parity.json` |
| Direct editor | Rust 18 InputConnection operations, two handles and real handle drag; Compose 11 matching edit operations; no IME change | `instrumentation/rust-text.json`, `compose-compose-text.json` |
| Recreation during composition | Route, UTF-16 content, selection and composition restored; zero changed packet slots | `instrumentation/rust-text.json` |
| Ordinary process death | **am kill**, PID 18861 → 18983; saved bundle restored, all compared draft fields match | `instrumentation/process-death.json` |
| Text-only preference bytes | **686 identical bytes**; SHA-256 `1db282c121d5e48eb68c039f5d2d4ff663346b3324cf084d8070359836e1e92d` | `instrumentation/text-preference-parity.json` |
| Forced native-load failure | Platform Views, no ComposeView; copied diagnostic is sanitized and original clipboard restored | `instrumentation/rust-crash.json`, `instrument-crash-details.log` |

The accessibility report exports screen-by-screen roles, state, actions and label differences
in `instrumentation/accessibility-comparison.json`. Compose sometimes exposes its role as a
separate non-clickable virtual child; the raw facts are retained rather than declaring exact
class/tree equality. A focused Android node correctly offers CLEAR_ACCESSIBILITY_FOCUS in
place of ACCESSIBILITY_FOCUS, and disabled controls correctly omit click actions. The audit
checks both focus/clear operations and only requires click on enabled clickable items. It
found a real dead whole-screen Rust host button; that container no longer exposes a click.
The 23 remaining missing labels in the pinned Compose/Jetpref path are real findings.
After the device stop the runner was strengthened to preserve duplicate labels in its
comparison, include focusable disabled items and editor length/selection metadata, and
resolve Compose's separate role children instead of accepting a generic View as a role.
Those additional checks are compiled but not device-executed; 23 is the last executed
label-finding count, not a conformance claim for that stronger audit. Rust also now omits
all activation actions for disabled settings and the Android adapter refuses them; the
64-mask JVM match test asserts that boundary.

The executed independent mode commands were:

```sh
export ANDROID_SERIAL=emulator-5556
export AGENT_DEVICE=/Users/justin/.t3/userdata/device/targets/t3-510c3db1163370c9ac20c66f/agent-device
# Close this task's agent session before UiAutomation; never another task's session.
"$AGENT_DEVICE" close
export SETTINGS_AGENT_CLOSED=1
python3 -I scripts/settings-device-tests.py .device-backup/stage4-followup2 ui
python3 -I scripts/settings-device-tests.py .device-backup/stage4-followup2 text
python3 -I scripts/settings-device-tests.py .device-backup/stage4-followup2 death
python3 -I scripts/settings-device-tests.py .device-backup/stage4-followup2 crash
```

UI proof precedes the final scrollbar, switch-alignment and handle-orientation refinements;
those refinements need the final installed-suite rerun. The runner now iterates every
radio choice; the recorded run selected the last choice in each list. That extra iteration
coverage has been compiled but still needs device execution. The crash-copy test did run after
the final Material APK was installed. Host regressions cover normalized reversed selection requests, crossing a collapsed range
and off-centre grabs; their additional real-device run remains owed.

## Pictures and strict comparison

Before: `before/{compose,rust}-keyboard.png` and `before/{compose,rust}-dialog.png`.
The user's earlier full-theme references also remain at `../stage4-followup/pictures/`.
After the first fixes: `pictures/{light,dark,font2}/{compose,rust}-{home,keyboard,keyboard-lower}.png`
and `*-dialog-00.png` through `*-dialog-13.png` (**102 pictures, 51 pairs**). Dialog indices
follow the 14-title list in `scripts/settings-pictures.py`. They include the installed radio
painting correction; they are **not final-source acceptance pictures** after the last refinements.

Use RGB channel threshold **3/255**, excluding a pixel only if any neighbour within exactly
one physical pixel differs by >3 in either image; mask the top/bottom 72 px system bars.
There is no alignment, blur, second dilation or broad text mask. The emoji probe uses that
same neighbour rule. Geometry is checked separately against Compose's tokens and pictures.
Font-2 wrapping is permitted; the conservative whole-picture counts retain its changed
geometry, so they cannot be converted into a visual acceptance percentage.

| Gallery (pre-final refinements) | Home differing pixels | Keyboard | Hinted-number dialog | Key-spacing dialog |
| --- | ---: | ---: | ---: | ---: |
| Light | 3,050 | 24,612 | 33,822 | 193,717 |
| Dark | 3,088 | 24,612 | 34,316 | 206,270 |
| Font 2.0 | 88,512 | 183,044 | 205,376 | 261,804 |

**0/51 strict matches.** `pictures/comparison.json`, `diff-*.png` and
`picture-diff-final.log` preserve every count. Differences identified from the actual flat
pixel colours include the missing scrollbar, scrolled app-bar surface colour, 8 dp
three-line switch offset, picker inset, dialog title/switch alignment and selected-radio
alignment. These are corrected in source; the next gallery must verify them rather than
assuming the old counts disappear. The script now waits three seconds for both paths'
scrollbar/app-bar animations to settle and returns to the top before each dialog.

A static-label diagnostic also finds an unresolved font-advance difference despite identical
14 sp = **36.75 px** conversion: Android Paint gives **715 px** with Material letter spacing;
Parley gives **726.684 px** for the same fixed UI-label prefix and supplied Roboto file.
Without spacing the widths are **688** and **698.465 px**. This is not waived as anti-aliasing.
`instrumentation/compose-metrics.json`, `font-prefix-probe.log` and `font-metrics-probe.log`
are the evidence. The next device diagnostic should compare TextRunShaper's actual font
file, TTC index and variation axes with the Parley run; no font instance is guessed here.

```sh
python3 -I scripts/settings-pictures.py .device-backup/stage4-followup2
python3 -I -m venv build/settings-image-venv
build/settings-image-venv/bin/python3 -I -m pip install Pillow numpy
build/settings-image-venv/bin/python3 -I scripts/settings-picture-diff.py build/stage4-followup2/pictures
python3 -I scripts/settings-performance.py build/stage4-followup2/performance-final .device-backup/stage4-followup2
python3 -I scripts/settings-performance-summary.py build/stage4-followup2/performance-final build/stage4-followup2/trace_processor
```

The fresh `performance-final` capture was **not run**: device work stopped first. Earlier
valid paired traces remain the measurements above. These commands require a newly opened,
healthy owned AVD and verified baseline; do not run them against the reserved emulator.

## Device stop and restoration boundary

Only `aw_keyboard_ssd`, host `local`, runtime `emulator-5556`, was used. No command targeted
`emulator-5554`. Before every app/test run streams 1/2/3/5 were set to 0, 4/10 to 1, each
verified through get-stream-volume. No audio, microphone or speech command was used.

The last completed test cleanup (`instrumentation/cleanup.json`) verifies the original
preference and both word-store SHA-256 values, original IME and enabled-IME list, accessibility
services disabled as originally, and font scale 1.0. The gallery cleanup restored light mode.
Rotation and animation settings were not changed. A complete namespace/property baseline
verification passed earlier in the run. No fresh verification could be obtained after the
emulator went offline; that is an explicit restoration-verification limit.

ADB then returned `device offline`. T3 device_open returned:

> The emulator process for "aw_keyboard_ssd" exited with code 1 before coming online.

Its manual-start advice was followed once, with `-no-audio`; it failed:

> Running multiple emulators with the same AVD is an experimental feature.

No read-only/second instance was started. Device testing stopped. T3 shutdown returned but
the offline owned qemu remained, so its exact `-avd aw_keyboard_ssd -port 5556` command was
verified before SIGTERM. The process is confirmed absent (`device-stop.json`,
`device-start-retry.log`). No further device mutation or testing was attempted.

## Still owed before prototype acceptance

1. Fresh final-source light/dark/font-2 pictures and the strict control/glyph comparison;
   resolve the measured font-instance/advance difference. Current gallery is not accepted.
2. Final-source installed UI/text/death suite rerun after the last drawing/handle refinements;
   fix or explicitly resolve the retained Compose path's 23 unlabeled accessibility controls.
3. Native app SurfaceFrame/input-to-present observability, then a like-for-like presented
   latency/jank comparison. SF display-frame statistics are insufficient.
4. Final-source warm PSS/Perfetto refresh and post-reopen complete baseline verification.
   Current valid paired PSS is below +10 MiB; no newer numbers are invented.
5. Real TalkBack speech and interaction on the user's phone, actual focused-editor floating
   selection toolbar and predictive-back/system-animation confirmation.

The full settings port and Compose deletion remain outside this package.

## Host verification and commits

The required final gate is:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

Final-head receipt: `gate-final-head.log`. **828 JVM tests**: app 498, Rust wrapper 173,
Snygg 128, Kotlin 29; zero failures/errors/skips. Rust tests, fmt, Clippy and the oracle pass;
compiler warnings are zero. Lint has five warnings, all within the allowed three IDs:
AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport.
`test-counts.json` and `lint-audit.json` retain the XML-derived counts and IDs.
`fixture-final-compiled.log` records the separate app/test APK build without warnings.

The 13 native settings regressions include Material switches/dividers/slider tokens,
nonlinear sp, modal background preservation, selection handles, normalized reversed input
selection and off-centre dragging through a collapsed range, plus shared font ownership.
`native-handles-final.log` records their pass. Four deliberate mutations (off-thumb size,
heading weight, nonlinear-sp bypass, font payload copying) each fail; exact restoration
passes (`mutations/result.json`). A fifth mutation reintroduces handle loss at collapse:
it fails with exit 101, restores byte-for-byte, then passes
(`mutations/handle-collapse-result.json`). No test was skipped, ignored or weakened.

The frozen scrollbar reference uses the actual Compose CubicBezierEasing rather than a
second copy of Rust's numerical implementation. The 64-mask settings match test also
requires disabled controls to offer no activation actions. Device harness ownership tests
are **6/6**, including rejecting the reserved device before any adb call and admitting a
fresh owned runtime ID only after checking its AVD name (`python-settings-final.log`).
All modified code files remain below 500 nonblank/noncomment lines
(`source-line-audit.json`). The normal APK excludes the fixture Activity/runner
(`normal-apk-check.json`). Rust share was not remeasured.

Logical code commits:

- `50eb7b3b` — safe platform native-load crash screen and private diagnostic boundary.
- `50c8bc23` — Material metrics/theme roles, editor handles, scrollbar, back and lifecycle.
- `1b2967d7` — shared font data, settings-only atlas opt-out, unbuffered input and traces.
- `c8dbafd8` — instrumentation, byte comparisons, disabled actions and restoration tooling.

The documentation commit follows these four. Every commit carries the requested co-author
trailer. Publication is to main; this checkpoint explicitly retains the acceptance debts above.
