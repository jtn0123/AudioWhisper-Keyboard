# Stage 3B renderer results

## Stage 3B package 2: Rust smartbar and visual parity

The prototype remains **default-off**. Smartbar policy, measurement, painting and pointer
ownership now live in Rust. Android supplies the configured action identities, existing
handlers, system fonts, vector paths and platform panels. This does not close the stage 3
phone, accessibility or performance acceptance gates.

### Implementation

The Rust smartbar implements suggestions-only, actions-only, shared and extended layouts;
flipped controls; optional speaker/sticky actions; shared/extended expansion; all three
extended-row placements; configured action order and overflow capacity. It uses Compose's
physical-pixel row/edge rounding and SpaceEvenly arrangement. Classic candidates take at
most three weighted columns, including the weighted full-width singleton; dynamic columns
use remaining constraints and the 160 dp cap; scrollable candidates keep intrinsic widths,
clip to the center, and scroll without committing the dragged candidate. Dividers, candidate
icons/secondary captions, grapheme-safe end ellipsis and clipped hit regions are in Rust.

The host uses the existing `QuickAction.onPointerDown/Up/Cancel` methods. InsertKey actions
therefore retain KeyboardManager/InputEventDispatcher handling, repeats, undo/redo, dictation
press/release and read-aloud hold behavior. Candidates use the existing commit/removal methods
and long-press preference. Candidate removal clears the highlight without losing ownership
of the eventual release. Disabled actions retain tooltip gestures but cannot dispatch input.
Android anchors the action tooltip to Rust's measured rectangle; the speaker keeps its
special hold with no tooltip. Overflow, media/emoji, autofill/inline panels and dictation's
preview stay in Compose. The preview occupies Rust's measured center while the native mic
remains mounted. A SurfaceView pre-draw fix ensures returning from a panel recreates its
surface; three consecutive emoji-return/type/clear cycles passed.

`SystemFonts.getAvailableFonts()` provides the real Roboto family files through Android's
public API. Parley selects the requested weight, including real variable-font weight-axis
instances, and font caches include weight. No synthetic emboldening or bundled fonts are
used. The emulator-font probe covers 100 through 900: 400 has default coordinates and 700
has normalized coordinates `[11102, 0, 0]`; measured widths at 36 px are 111.05859 and
112.833984. This also covers the 400/700 used by bundled themes.

Painted key bounds now apply Snygg margins after Compose-equivalent coordinate/size rounding.
Borderless's 6 dp vertical space/enter margins are honored without shrinking their touch
bounds. Shadows use ambient and offset spot blurred rounded rectangles. Their scale/alpha
were tuned against actual screenshots, with damage bounds covering the blur tails. They
remain an approximation of Android's elevation shadow, not a pixel-equivalence claim.

### Match tests and intentional break

`ReferenceSmartbar` is a test-only transcription of Smartbar/MainRow, QuickActionsRow and
CandidatesRow. The JVM test compares Rust packets over **9,216 cases**: 1,152 combinations
of layout, flip, shared/extended expansion, extended placement, speaker, sticky action and
candidate display mode, each with eight seeded inputs. These vary density, fractional row
heights, width, 0/1/2/3/6/19/30/50 configured actions and empty/single/many candidate lists
including short/long, ZWJ family, flag, skin tone, keycap and accented strings. It checks
geometry, action identities/capacity, candidate bounds, clipping viewport and ellipsis
prefix selection. Advances are independently supplied; this JVM test does not compare
Android text shaping. Rust tests additionally exercise action cancellation, clipped scrolling,
no sliding from the toolbar into character keys, physical-pixel capacity, held candidate
removal/release and disabled tooltips.

The deliberate mutation negated the sticky-action term in overflow capacity. The match test
failed at case 5185 (expected packet length 32, actual 38), with Gradle exit 1. The mutation
was restored and the full gate passes. Evidence: `build/stage3b2/mutation.log`,
`mutation.exit` and `mutation-result.xml`.

### Emulator comparisons

Only `aw_keyboard` / runtime **emulator-5556** was used. Do Not Disturb was off; streams
1/2/3/5 were muted and alarm stream 4 stayed at its minimum 1. The keyboard was installed
only through `scripts/install-dev.sh`. Existing dictation models were used; none were
downloaded. Read-aloud's service was disabled, so its tap and hold both reached the same
accessibility-setup panel on both renderers; active audible playback was not exercised.

| Action | Compose and Rust result |
|---|---|
| Expand/collapse | Same shared-row preference and toolbar/candidate transitions. |
| Undo | Same 3-unit editor digest to empty; 650 ms hold repeats to empty on both. |
| Redo | Same original 3-unit digest restored. |
| Settings | Same keyboard settings activity; a 650 ms hold suppresses navigation on both. |
| Overflow (`...`) | Same Compose panel; 0 differing RGB channels in the keyboard crop. |
| Emoji | Same Compose media panel; 0 differing RGB channels in the keyboard crop; return remains functional. |
| Mic tap, then stop | Same lit/idle states and final 20-unit digest; existing model path. |
| Mic 350 ms hold | Same stop-on-release idle state and final digest. |
| Read-aloud tap / 650 ms hold | Same accessibility setup panel through the existing handler. |

`actions.json` stores only counts/digests and action flags. Its accessibility-label probe for
`Customize` is false for both renderers because the panel is absent from that snapshot;
the actual overflow screenshots and pixel comparison establish handover. Likewise the
appops `(running)` probe was not exposed by this emulator and is not recording-state proof.
Mic pictures retain only the mic region, without dictation-preview content. Evidence:
`actions-progress.log`, `actions.json`, `mic-states.json`, `panel-match.json`, `repro-progress.log`.

All screenshot paths below are relative to `build/stage3b2/screenshots/`. Each pair has
Compose on the left and Rust on the right, from real emulator captures. The pressed states
are frames from agent-device recordings, so video compression limits pixel measurements.

| Theme | Collapsed, candidates | Collapsed, none | Expanded actions | Pressed key |
|---|---|---|---|---|
| Floris Day | `pair-day-candidates.png` | `pair-day-none.png` | `pair-day-expanded.png` | `pair-day-pressed.png` |
| Floris Night | `pair-night-candidates.png` | `pair-night-none.png` | `pair-night-expanded.png` | `pair-night-pressed.png` |
| Floris Day Borderless | `pair-borderless-candidates.png` | `pair-borderless-none.png` | `pair-borderless-expanded.png` | `pair-borderless-pressed.png` |

Raw captures use `{compose,rust}-{day,night,borderless}-{candidates,none,expanded,pressed}.png`.
Toolbar/candidate placement and Borderless space/enter inset backgrounds now agree. Remaining
visible differences are small glyph/hint rasterization/placement differences and shadow blur,
especially lower-row tails. A flat Day q-key vertical shadow profile differs by at most 3 raw
RGB levels below the key and matches the sampled top edge exactly; this local sample does
not establish a whole-image tolerance. `pair-ellipsis.png` additionally shows the same constrained columns and end ellipsis with a one-space prefix-boundary difference in the long second/third candidates; font measurement is not yet pixel-identical. `pair-settings-tooltip.png` records the hold probe (both suppress navigation), with the platform tooltip still visible only on Rust at the capture time. The 12 requested pairs remain the primary visual review evidence.

### Timing, replay and final gate

Accepted order: Compose 1, Rust 1, Compose 2, Rust 2, using the final gated APK installed
with `scripts/install-dev.sh`. Builds/tests were paused. Every trial replayed the same
300 presses plus eight e-holds with suggestions enabled, Floris Day and Always day.
Earlier trials on the pre-caption-fix APK are excluded under `pre-final-apk/`.

| Trial | Samples | Primary CPU p50 / p95 / p99 (ms) | Touch p50 / p95 / p99 (ms) |
|---|---:|---:|---:|
| compose-1 | 946 | 1.121 / 1.755 / 3.013 | 34.118 / 49.922 / 53.538 |
| rust-1 | 627 | 1.107 / 2.024 / 4.703 | 36.413 / 52.777 / 55.819 |
| compose-2 | 942 | 0.913 / 1.496 / 2.876 | 33.423 / 45.012 / 51.762 |
| rust-2 | 624 | 1.138 / 2.417 / 5.903 | 35.985 / 52.076 / 54.349 |

Rust's primary column is scene+raster CPU; its touch column ends at buffer post. Compose's
primary column is HWUI draw recording; its touch column ends at the existing commit callback.
These boundaries differ, so this is context, not a direct speedup claim. Compose frame
completion p50/p95/p99 is 17.442/19.698/33.607 ms and 17.486/19.515/33.883 ms respectively.
The samples column counts primary frames; each touch distribution contains 616 events.

Compared with package 1's single-thread p50 0.687–0.701 ms and p95 4.376–4.553 ms, this
package increases median cost to 1.107–1.138 ms while reducing p95 to 2.024–2.417 ms.
This measures the combined smartbar/shadow/margin changes; it does not isolate their individual
costs. The **<=2 ms p95 target still fails**. p99 is 4.703–5.903 ms. Single-thread Vello
remains selected. Full numeric evidence: `build/stage3b2/trial-summary.json` and
`trials/{compose-1,rust-1,compose-2,rust-2}/`.

All four trials matched both exact reference digests:

- 300 units: `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`
- 308 units/eight holds: `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`

Every accepted trial's timing and pre-runtime logs have no FATAL/AndroidRuntime matches.
The final `adb logcat -d` scan also has **zero matches** (`final-runtime-errors.txt`).
A recorder stop timed out once; its advised `--debug` retry succeeded. A speaker coordinate
press intentionally navigated to Android settings and triggered agent-device's app-boundary
guard; the advised fresh screenshot/snapshot established the expected screen, and the
stationary-gesture retry succeeded. No required device check was abandoned.

The exact local gate
`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug`
exited **0** (`BUILD SUCCESSFUL in 4m 8s`, 343 tasks, 48 executed/295 up-to-date).
Gradle output was redirected directly with its own status saved in `build/stage3b2/gate.exit`,
without a pipe. JVM/native differential suites, 28 renderer tests, oracle checks, rustfmt,
clippy with `-D warnings`, app lint and assembly pass. App lint contains exactly
AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport; Rust-module lint
is empty. No suppression, baseline, skipped or removed test was added.

Against `8bf52e57`, non-blank non-test source removes **23 Kotlin lines** and adds 345
(net **+322**); Rust adds **1,172 lines** and removes 46 (net **+1,126**). Totals are
53,392 Kotlin, 33,216 Rust and 627 Java. The count excludes tests and generated tables;
the font diagnostic example is included. Kotlin grows because platform adapters are added
while Compose remains the fallback. Changed source files are at most **388 lines**.
Screenshots, fonts, device backups, local settings and build outputs are not committed.

Cleanup verified persisted prototype **false**, theme mode **ALWAYS_DAY**, and day theme
**org.florisboard.themes:floris_day**. Suggestions and emoji suggestions are restored,
profiling tags are INFO, DND is off, the device is muted, and the probe app is uninstalled.
The runtime **emulator-5556** was closed with `shutdown: true`; emulator-5554 was never used.
Evidence: `restored-preferences.json`, `cleanup.exit`, and `cleanup-progress.log`.

Hosted CI status is reported with the pushed run; billing-blocked hosted checks are not
substituted for the successful local gate.

---

## Stage 3B package 1: renderer-owned Snygg styles

The prototype remains default-off. This package moves style decisions into Rust and
measures Vello CPU worker threads; it does not claim the full stage 3 performance,
phone, accessibility or visual-equivalence gate.

### Theme boundary

`render` depends directly on the Java-free `snygg` crate. A native session compiles the
active stylesheet with `CompiledTheme<Value>::from_json()` and resolves through
`resolve_with_palette()`. ThemeManager retains the original source loaded by the Kotlin
facade, including extension archives. Its existing theme selection chooses the source for
system mode; Snygg dynamic-light/dynamic-dark tokens select the supplied named palettes.

A separate JNI theme call carries that JSON, both named Material You palettes, density,
Android font scale and the keyboard's font multiplier. The view sends it on a theme/input
change, not on frames. Palette materialization stays at the Android/Compose boundary;
Kotlin no longer queries or packs rendered colours, font sizes, elevations or radii.

Scene updates carry existing evaluated geometry, labels/icons, key codes/types/enabled
flags, keyboard mode/shift state and candidate kind/auto-commit flags. Popup opening adds
choice labels/codes. Font-file discovery, Surface ownership, MotionEvent packets and action
return values retain the milestone A boundary. The frame call carries only its timestamp.

Rust caches resolved property sets and physical drawing styles by element and state, with
bounded retention and a fresh cache for every theme/scale/palette change. Keys, pressed and
disabled states, shift selectors, hints with their key parent, window background, smartbar
and candidate row, word/clipboard candidates and their text, popup boxes and focus choices
all use the compiled theme. Frames only select already-resolved states. Shapes retain all
four corners, dp/percentage units, round/cut/circle/rectangle forms and physical scale.
Candidate down/move/cancel/up now selects its cached pressed appearance too.

### Differential proof

The test-only Kotlin `ReferenceSnyggTheme` supplies the expected styles. The JVM test
compares **76,032** renderer queries across all **12** discovered bundled themes, three
modes, four shift states, six key codes/types, four selectors, candidate auto-commit flags
and two font multipliers. It crosses JNI into the renderer's actual resolver, including
the window/root and element-parent chains and named light/dark palettes.

A frozen reference fixture also supplies **2,376** native cases. Rust checks property
values and the materialized scene key/hint/candidate/popup styles, cache reuse, extension
source replacement, type and caps-lock selectors, physical sizes/shapes and explicit font
fallback. Existing touch and full-versus-dirty pixel tests remain in place.

The deliberate mutation replaced the requested selector with NONE in the renderer's
resolver. Gradle exited **1** on the first Material You pressed key: expected background
`rgba(232,226,208,1)`, actual `rgba(223,218,204,1)`. The selector was restored and the
JVM/native tests passed. The fixture cannot be rewritten by Rust's implementation.

A directed extension regression first failed with a 14sp popup choice instead of its
originating key box's 30sp size. The cache now retains the source box code separately
from the choice code. Native tests and six additional JVM comparisons cover alternating
source keys sharing the same choice, including focus state, so a cached inherited style
cannot leak between them.

### Fonts and visual boundaries

The currently supplied system regular sans face and system monospace face are supported.
Custom extension families, unsupported generic families (for example serif/cursive), and
non-regular theme weights explicitly fall back to **system sans regular**. Bundled themes
request `bold` (700) for auto-commit candidate text; that request currently falls back.
The bundled `monospace` hints load the system monospace face. Arbitrary extension font
assets and weight/style variants are not loaded by this package.

All captures used `aw_keyboard`, `local`, runtime **emulator-5556**, Android 16,
1080×2400, density 420, **Always day** theme mode and **system light** appearance.
Night sources were deliberately selected as the day theme to keep the mode/system state
identical. The other session's emulator was not used.

Before/after screenshots are under ignored `build/stage3b/screenshots/`:
`{before,after}-{rust,compose}-{day,night}.png`. In the keyboard region
`(0,1563)–(1080,2288)` (783,000 pixels), Rust Day and Night each differ in **589** pixels.
Compose differs in **616** and **617** pixels respectively. Every changed pixel lies in
the enter-action glyph, within `(966,2183)–(1013,2216)` for Rust; the editor action changed
between captures. All pixels outside the enter key are identical. This is a capture-state
difference shared with Compose, not evidence of a theme-colour improvement.
`pixel-results.json` records bounds and channel deltas.

Fresh side-by-side images, Compose left and Rust right:

| Theme | Screenshot | Visible differences |
|---|---|---|
| Floris Day | `pair-day.png` | Theme colours match; Rust has weaker shadows, slightly different hint placement and its full-width candidate strip instead of Compose's toolbar. |
| Floris Night | `pair-night.png` | Dark backgrounds/key colours match; shadow/font/hint and toolbar differences remain. |
| Floris Day Borderless | `pair-borderless.png` | Borderless colours match; Rust space/enter backgrounds fill more of the existing visible key bounds than Compose's inset backgrounds. |
| Floris Day Material You | `pair-material.png` | Dynamic palette/key/action colours match; shadows/hints and toolbar geometry remain different. |
| Floris Pure Night | `pair-pure.png` | Black window and darker keys match; text/hints and toolbar geometry remain different. |
| Floris Night Material You | `pair-night-material.png` | Dark dynamic colours match; shadows/hints and toolbar geometry remain different. |

All paths in the table are relative to `build/stage3b/screenshots/`; raw images use
`after-{rust,compose}-{theme}.png` with the same suffix. These are actual screenshots,
not redrawn keyboards. Existing candidate width/ellipsis, toolbar, shadows and popup
placement approximations remain milestone B follow-up work.

`live-before-day.png` and `live-after-night.png` show changing the selected theme while
Rust's keyboard stays visible; `mInputShown=true` was checked between selection and the
second capture. Keys and background repaint from Day to Night without reopening the editor.

### CPU worker-thread experiment


Vello CPU **0.4.0** offers the `multithreading` feature and
`RenderContext::new_with(..., RenderSettings { num_threads, .. })`. Zero workers selects
the direct single-thread dispatcher. The experiment used 0, 2 and 4 workers with the
same scene, dirty regions, glyph atlas and `TargetInit::SrcOver`; `flush()` is required
before multithreaded rasterization (a no-op in single-thread mode).

**Decision: keep single-thread rendering.** Both worker counts worsened p50 and p95.
The feature, worker switches and temporary probe are removed from committed code. The
reproduction patch and host pixel probe remain only in ignored `build/stage3b/`.

Fresh alternating order: Compose 1, single 1, two 1, single 2, four 1, Compose 2,
single 3, two 2, single 4, four 2. Every trial ran the existing 300-press/eight-hold
harness. Builds/tests were paused during trials. Profiling was enabled equally for
native trials; the keyboard used Floris Day, Always day, system light.

| Trial | Frames | Scene+raster p50 / p95 (ms) | Release p50 / p95 (ms) | Candidate-change p50 / p95 (ms) | Idle CPU |
|---|---:|---:|---:|---:|---:|
| single-1 | 625 | 0.701 / 4.451 | 0.862 / 5.016 | 0.862 / 5.016 | 0.0% |
| two-1 | 628 | 1.425 / 6.887 | 3.177 / 9.118 | 3.177 / 8.936 | 0.0% |
| single-2 | 628 | 0.692 / 4.553 | 0.844 / 5.267 | 0.845 / 5.267 | 0.2% |
| four-1 | 625 | 2.200 / 10.617 | 4.216 / 13.247 | 4.216 / 13.247 | 0.0% |
| single-3 | 624 | 0.691 / 4.422 | 0.860 / 5.176 | 0.860 / 5.176 | 0.0% |
| two-2 | 625 | 1.598 / 6.436 | 3.042 / 7.008 | 3.042 / 7.008 | 0.0% |
| single-4 | 627 | 0.687 / 4.376 | 0.841 / 5.017 | 0.840 / 5.017 | 0.0% |
| four-2 | 624 | 2.163 / 8.446 | 4.098 / 11.770 | 4.098 / 11.770 | 0.1% |

| Compose context | HWUI draw recording p50 / p95 (ms) | Frame completion p50 / p95 (ms) | Touch-to-commit p50 / p95 (ms) | Idle CPU |
|---|---:|---:|---:|---:|
| compose-1 | 1.345 / 2.148 | 17.484 / 19.931 | 33.959 / 48.171 | 0.0% |
| compose-2 | 0.936 / 1.526 | 17.420 / 19.482 | 33.670 / 47.688 | 0.0% |

Compose timing uses different boundaries and is host-noise context, not a direct speedup
comparison. Single-thread p95 stayed in 4.376–4.553 ms, two workers in 6.436–6.887 ms,
and four in 8.446–10.617 ms. The stage 3 CPU <=2 ms target still fails.

Idle measurements sample whole-process utime+stime for 10 seconds at verified `CLK_TCK=100`.
All ten samples posted **zero additional native frames**. Idle CPU was 0.0–0.2% for
single-thread, 0.0% for two workers, 0.0–0.1% for four workers and 0.0% for Compose.
This coarse emulator CPU sample is not a phone power measurement.

The first pre-flush two-worker startup aborted at Vello's explicit
`attempted to rasterize before flushing` assertion. The pinned API documentation and
upstream tests identified the required flush; adding it fixed startup. The isolated
pre-flush diagnostics are in `trial-failure-logcat.txt` and `trials/`, and are excluded
from measurements. The corrected host probe has zero pixel differences for 2/4 workers
over sixteen dirty text/shadow transitions each. Every accepted trial has matching
digests and no runtime failure. No dependency fork, warning suppression or architecture
change was used to make multithreading pass.

All measurements are saved under `build/stage3b/trials-final/<trial>/`;
`trial-summary.json` records per-trial timing, idle and digest results. The retained
scratch UI logs contain action metadata only; initial settle diagnostics that echoed
synthetic probe text were removed.

### Final gate and cleanup

The exact required command
`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug`
exited **0**, with `BUILD SUCCESSFUL in 2m 44s` (343 tasks: 60 executed, 283 up-to-date).
The direct Gradle status is saved separately in `build/stage3b/local-gate.exit`; its output
was redirected without a pipe to `local-gate.log`. All JVM/native differential suites,
20 renderer tests, the C++ oracle checks, rustfmt, clippy with `-D warnings`, lint and
assembly passed. App lint contains exactly AndroidGradlePluginVersion,
NewerVersionAvailable and ChromeOsAbiSupport; Rust-module lint has no findings.

Compared with `25c17b0a`, non-blank non-test source removes **67 Kotlin lines** and adds
63 (net **-4**), and adds **635 Rust lines** while removing 84 (net **+551**).
Totals under the 90% plan's counting rules are 53,070 Kotlin, 32,090 Rust and 627 Java.
Tests, generated tables and benchmark scaffolding are excluded. All changed source files
are at most **486 lines**. No suppressions, lint baseline, ignored tests, fonts, screenshots,
local settings or build outputs are committed.


The final single-thread APK was installed only with `scripts/install-dev.sh`. All six
visual pairs and the live Day→Night switch were recaptured on that APK. Its fresh Rust
and Compose replays both matched the 300-press digest
`809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` and the
308-unit/eight-hold digest
`c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`.
Final evidence: `build/stage3b/final-acceptance/{rust,compose}/`.

Final `adb logcat -d` contains **no FATAL or AndroidRuntime matches**;
`final-acceptance/runtime-errors.txt` is empty. The earlier missing-flush experiment
abort remains explicitly recorded above and is not described as a clean run.

The persisted JetPref file was checked after UI restoration: prototype **false**,
`ALWAYS_DAY`, `org.florisboard.themes:floris_day`. The probe app was uninstalled,
measurement tags were reset to INFO and Do Not Disturb was verified off. Streams
1/2/3/5 were muted and alarm stream 4 used its minimum 1 throughout. Cleanup assertions
are saved in `final-acceptance/restored-preferences.json` and `acceptance.exit`.
