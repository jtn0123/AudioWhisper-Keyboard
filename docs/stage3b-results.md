# Stage 3B package 1: renderer-owned Snygg styles

The prototype remains default-off. This package moves style decisions into Rust and
measures Vello CPU worker threads; it does not claim the full stage 3 performance,
phone, accessibility or visual-equivalence gate.

## Theme boundary

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

## Differential proof

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

## Fonts and visual boundaries

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

## CPU worker-thread experiment


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

## Final gate and cleanup

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
