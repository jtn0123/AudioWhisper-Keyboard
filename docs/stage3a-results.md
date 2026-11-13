# Stage 3A: pure Rust drawing prototype

**CPU tails are attributed and the current-build AVD runtime check is complete. The
performance gate remains unpassed.** Separate strip/key damage reduced frames over 2 ms
from **554/1,291 (42.9%)** to **129/1,275 (10.1%)** in two alternating trials each. CPU
scene+raster p95 is still **4.715–5.533 ms**, against the 2 ms target. Compose remains the
default; the Rust prototype setting was switched off after testing.

Measured on 2026-11-12 using only `aw_keyboard`, runtime ID `emulator-5556`, on host `local`:
Android 16, 1080×2400, density 420, 60 Hz. `emulator-5554` belongs to another session and was
not used or closed. Earlier retained samples below are explicitly historical. This is
milestone A evidence, not the full stage 3 or S24 GPU gate.

## Implementation

Settings > Keyboard contains **Rust keyboard renderer (prototype)**, default **off**.
Compose remains the default and fallback. Character keyboards use a FrameLayout/SurfaceView
when enabled; other modes and panels retain Compose. Rust consumes the existing evaluated
TextKeyboard list and geometry supplied by the `keys` crate. It does not assemble another
layout. Resolved bundled Snygg colours, type sizes, spacing, radius and elevation cross JNI
as plain values. QWERTY labels, hints, icons, pressed/shift states, candidates and the evaluated
`e` accent choices are drawn by Rust. Actions go through the existing InputEventDispatcher
and KeyboardManager. Android forwards action/index, pointer identities, every coordinate,
history, down/current timestamps and cancel.

`render` has no Java dependency and forbids unsafe code. Only `awnative` acquires the
ANativeWindow and constructs the Vulkan surface. A surface generation has one worker;
destruction stops and joins that worker, drops the renderer, then releases the window.
CPU drawing maintains a backing pixmap, paints separate dirty rectangles and honours the NDK lock's
expanded buffer-copy rectangle. Hybrid redraws the scene on changes. Neither schedules
continuous frames while idle. The existing Snygg background SurfaceView remains below this
surface; Compose/autofill stay above it.

## System emoji fonts

The AVD's `/system/etc/fonts.xml` names both fonts below. Files were pulled to the fresh
external directory `/tmp/aw-stage3a-fonts-20261112`, never added to the source tree.
The Rust `emoji_probe` example inspects tables with read-fonts, shapes with Parley and
rasterizes with Vello CPU/Fontations. The native keyboard discovers font files through
Android's public `SystemFonts.getAvailableFonts()` API.

| Font | Bytes | Colour tables | Absent |
|---|---:|---|---|
| NotoColorEmoji.ttf | 2,813,064 | COLR **v1** | CBDT/CBLC, COLR v0, sbix, SVG |
| NotoColorEmojiFlags.ttf | 913,564 | **CBDT + CBLC** | COLR, sbix, SVG |

Roboto-Regular, NotoSansSymbols-Regular-Subsetted2 and DroidSansMono were also supplied.
Symbols must be script fallback, not the explicit sans family: otherwise flags/VS16 can
select monochrome symbols before the emoji family.

| Sequence | Shaped glyph ids | Covered / coloured pixels in CPU probe |
|---|---|---:|
| ZWJ family 👨‍👩‍👧‍👦 | 857 | 5,089 / 0 |
| Flag 🇺🇸 | 261 | 3,839 / 2,946 |
| Skin tone 👍🏽 | 571 | 2,506 / 2,503 |
| Keycap 1️⃣ | 20 | 5,148 / 4,875 |
| VS16 ❤️ | 3547 | 3,249 / 3,247 |
| VS15 ❤︎ | 63 | 1,641 / 0 |

Each sequence is one nonzero glyph. The family is the system font's grayscale family
artwork, also visible that way in Compose; it is not missing-glyph output. VS15 remains
monochrome while VS16 selects colour. This proves these fonts/tables/sequences, not every
COLR paint graph or arbitrary emoji font. The S24 Ultra's Samsung emoji font must be pulled,
inspected and rendered with the same probe on the phone later; it was not tested here.

## Measurement method

The disposable editor and scripts are in `scripts/render-benchmark/`; generated artifacts
are under ignored `build/stage3a-evidence/`. Each typing run sends 300 presses with 35 ms
holds and 85 ms gaps, followed by eight 500 ms holds on `e`. The editor persists only length
and a SHA-256 digest, not typed contents. It is cleared between trials. Synthetic emoji
candidates are opt-in only; actual candidate state takes precedence.

Native CPU draw time includes scene encoding and CPU rasterization. Hybrid draw time includes
scene and GPU command encoding, excluding surface acquisition; its GPU completion wall time
includes submission/driver waiting and is not a timestamp-query measurement. Native input
latency ends after NDK unlock/post or Vulkan present submission. Compose's opt-in callback
ends at HWUI frame commit; gfxinfo gives app-wide HWUI drawing/completion intervals.
These boundaries differ: do not turn them into a renderer speedup ratio or claim physical
touch-to-present. Repeated NLP updates with the same touch timestamp are excluded from
first-response input latency. Android 16's gfxinfo did not expose usable OldestInputEvent,
so Compose input latency comes from the explicit callback.

The emulator GPU is emulated and host contention is uncontrolled. Results are indicative;
acceptance still requires alternating phone trials at 120 Hz and Perfetto FrameTimeline.

Retained pre-continuation CPU trial: 627 posted frames and 616 distinct touch events. Compose: 946 collected
HWUI frames and 616 distinct touch events. Milliseconds, p50/p95/p99:

| Measurement | p50 | p95 | p99 |
|---|---:|---:|---:|
| Vello CPU scene + raster | 0.689 | 7.765 | 12.367 |
| Vello CPU touch to buffer post | 35.806 | 52.853 | 65.187 |
| Compose HWUI draw recording | 1.377 | 2.295 | 5.024 |
| Compose HWUI frame completion | 17.448 | 19.947 | 34.070 |
| Compose touch to frame commit | 33.822 | 48.947 | 52.187 |

Earlier CPU trials were 1.115/7.806/10.886 ms before strip culling and
0.668/9.994/14.899 ms after skipping unchanged emoji preparation but before candidate clipping.
Tail variability is material; the final trial is not evidence of a steady improvement rate.

The Vulkan adapter reported `llvmpipe (LLVM 21.1.4, 128 bits)`, device type CPU,
Mesa 26.0.0-devel, Vulkan backend. Warm Hybrid measurements (630 posted frames, 616 distinct
touch events), in milliseconds:

| Hybrid measurement | p50 | p95 | p99 |
|---|---:|---:|---:|
| CPU scene + command encoding | 1.571 | 5.234 | 7.117 |
| GPU submission-to-completion wall time | 5.709 | 8.070 | 9.641 |
| Touch to present submission | 35.965 | 52.890 | 61.423 |

The CPU and GPU targets were not demonstrated by this Vulkan trial. The GPU wall interval
cannot substitute for a device GPU timestamp, especially on this CPU-backed Vulkan adapter.

The initial idle check posted no extra native frames in 2.5 seconds (628 before and after).
Those retained measurements predate the candidate edge-cancel correction. The fresh trials
and runtime checks below include that correction.

## CPU tail attribution and fixes

Fresh baselines used `ce21874c`; optimized trials used the Rust source in `7e00ba09`.
Each sequence was **Compose, CPU, Compose, CPU**, followed by one warm Hybrid trial after
the fix. All used the same 300 presses plus eight `e` holds. CPU profiling, Compose timing
and synthetic-emoji tags were DEBUG; Hybrid was selected only for its trial. The lifecycle
trace added after timing trials does no work on normal key/scene updates, and the final
APK uses the identical Rust source and unchanged native artifact. Each valid trial matched both text digests.

Baseline CPU trials posted 666/625 frames (283/271 over 2 ms); final trials posted 626/649
frames (59/70 over 2 ms). Each trial has 616 distinct touch responses. Additional posted
frames reflect asynchronous scene/candidate updates; they are not additional touches.
Incomplete harness/configuration attempts remain under `tails-incomplete-*`, excluded
from every number here. No failing acceptance test was skipped or deleted.

The profiler records full/dirty, bounding damage pixels, exact union paint pixels, region
count, press/release, ASCII case-change proxy, popup state/change, candidates and surface
creation. Complete ordered logs distinguish popup open/move/close; the script has no popup
move or manual Shift step. Scene encoding, raster, shaping/layout, glyph calls, cold-layout
and colour-font calls, explicit allocation sites, and NDK lock/copy/unlock-post time are
numeric only. The post timestamp precedes diagnostic formatting.

Vello's private atlas miss counts are not exposed by this pinned release. Cold-layout and
colour-font timings are **proxies, not actual atlas-miss counters**; deferred atlas work
is inside raster time. Explicit allocation-site counts exclude library internals, JNI,
scene construction and allocator-wide activity. Nothing typed or suggested is logged.

| Frames over 2 ms: overlapping group | Before count | Before p50 / p95 ms | After count | After p50 / p95 ms |
|---|---:|---|---:|---|
| Key press | 13 | 3.983 / 8.917 | 23 | 4.268 / 7.417 |
| Key release | 525 | 4.626 / 9.347 | 98 | 5.269 / 11.306 |
| Shift proxy | 0 | — | 0 | — |
| Popup open | 2 | 3.898 / 3.898 | 2 | 3.942 / 3.942 |
| Popup move | 0 | — | 0 | — |
| Popup close | 10 | 3.485 / 6.008 | 5 | 7.631 / 7.669 |
| Candidate strip change | 530 | 4.630 / 9.432 | 103 | 5.273 / 10.590 |
| Surface create/recreate | 1 | 55.628 / 55.628 | 0 | — |
| Other structural change | 1 | 28.691 / 28.691 | 0 | — |
| Full redraw | 2 | 28.691 / 28.691 | 0 | — |
| Dirty redraw | 552 | 4.622 / 9.641 | 129 | 5.121 / 10.198 |
| Shaping-cache miss | 534 | 4.630 / 9.503 | 115 | 5.239 / 10.590 |

Quantiles retain the summary script's floor((n−1)q) sample index; small groups are not
stable tail estimates. Groups overlap: a release and candidate change often share one
frame. Full/dirty and cache-miss rows are classifications rather than independent triggers. A zero means no
slow sample in that group; it does not prove a cold-start or Shift optimization. The one
baseline surface frame took 55.628 ms. Separate final recreation logs retain cold-frame
cost; warm percentile improvements do not eliminate it.

| Summed time in frames over 2 ms | Before ms (554 frames) | After ms (129 frames) |
|---|---:|---:|
| Scene encoding | 255.539 | 54.843 |
| CPU raster (includes Vello deferred atlas work) | 2704.669 | 669.357 |
| Layout-cache lookup + shaping (inside encoding) | 66.223 | 9.211 |
| Glyph preparation calls (inside encoding) | 58.000 | 14.150 |
| First-layout glyph calls (cold-work proxy) | 26.485 | 3.511 |
| Colour-font glyph calls | 1.723 | 0.000 |
| Explicit allocation-site time | 8.045 | 2.985 |

Raster was **91.4%** of summed baseline slow-frame encoding+raster time. The 530 candidate
change and 525 release samples point to bounding-rectangle amplification: a changed strip
and a distant released key repainted all intervening rows. Median candidate+release paint
area fell from **454,680 to 145,086 pixels**. After the fix, slow frames paint at most
149,722 pixels in these trials, versus 768,960 in the baseline. Explicit slow-frame
allocation sites were 69,197 before and 9,533 after; these are site executions, not measured
malloc counts. Layout misses were 773/219 in those same slow-frame subsets.

The shipped fixes are:

- Preserve separate strip, pressed-key and popup rectangles in `Scene::damage_regions`.
  Rasterize each into the retained pixmap and post once, honoring the NDK's expanded copy
  rectangle. The copy can still span the larger bounds even when painted pixels do not.
- Compare stable scene fields directly instead of cloning both scenes and their strings
  just to clear transient fields for comparison.
- Retain existing layout/font/glyph resources and pixmap reuse. Shaping and explicit
  allocation sites were small fractions of these tails; no speculative replacement cache
  or colour-emoji cache was added. No slow frame in the final typing trials contained
  colour-font glyph calls.
- Extend numeric attribution and reject benchmark runs unless the editor is empty and the
  IME is shown. Verify both expected digests before declaring a trial successful.

All **15 renderer tests** pass, including disjoint and overlapping damage with long clipped
text, fractional edges, shadows, light/dark colours and popup transitions, compared to full
redraws at every step. A combined clip-path experiment failed pixel parity and was replaced
by the existing rectangular clip semantics; the regression test remains.

A separate host debug probe compared atlas and direct outline drawing on 100 growing
three-cell candidate scenes. Total scene+raster p50/p95/p99 was
25.297/25.990/353.162 ms with the atlas and 33.948/38.642/38.797 ms without it. Direct drawing
reduced raster time but increased encoding, worsening typical total time, so it was
rejected. These are host debug diagnostics, not Android performance results. The passed
probe, numeric output and experimental patch are retained under ignored
`build/stage3a-evidence/tails-outline-experiment*`; no experimental backend switch ships.

## Fresh alternating measurements

All entries are milliseconds, **p50 / p95 / p99**. These are instrumented AVD
trials, not phone acceptance measurements. The table shows each trial instead of hiding
host variability in one pooled percentile.

| Boundary (ms, p50 / p95 / p99) | Before 1 | Before 2 | After 1 | After 2 |
|---|---|---|---|---|
| CPU scene + raster | 0.806 / 8.479 / 17.681 | 0.728 / 7.809 / 9.974 | 0.604 / 4.715 / 6.130 | 0.824 / 5.533 / 8.953 |
| CPU touch to buffer post | 37.356 / 74.755 / 105.917 | 36.770 / 54.619 / 64.973 | 36.021 / 55.187 / 65.613 | 39.830 / 70.876 / 86.595 |
| Compose HWUI draw recording | 1.054 / 3.170 / 5.149 | 1.492 / 2.771 / 4.810 | 1.208 / 2.036 / 2.981 | 1.311 / 3.740 / 5.682 |
| Compose HWUI frame completion | 17.855 / 35.317 / 53.055 | 17.727 / 34.025 / 50.450 | 17.389 / 19.730 / 33.919 | 23.642 / 67.394 / 84.156 |
| Compose touch to frame commit | 35.831 / 57.495 / 72.203 | 36.527 / 54.896 / 65.400 | 33.776 / 48.306 / 51.325 | 43.924 / 78.733 / 89.924 |

CPU scene+raster p95 spread was **7.809–8.479 ms before**, **4.715–5.533 ms after**; touch-to-post
p95 was **54.619–74.755 before**, **55.187–70.876 after**. There is no demonstrated input-latency
tail improvement. Unchanged Compose frame-completion p95 ranged from **19.730 to 67.394 ms**
in the two final rounds, showing substantial host variability. Native draw, native post,
Compose recording and HWUI completion remain different boundaries; no cross-renderer
speedup ratio or physical touch-to-presentation claim follows from them. The first baseline
Compose trial overlapped the attribution build, and targeted host Rust checks overlapped
parts of the baseline capture period. Other host contention was uncontrolled. Both versions
use the same core CPU timing/site counters; the optimized version additionally counts exact
paint-union pixels after posting. That extra bookkeeping is outside the reported draw/post
timestamps but can delay the worker's next frame; its overhead was not separately isolated.

| Hybrid boundary | Historical single trial | Fresh single trial |
|---|---|---|
| Scene + command encoding | 1.571 / 5.234 / 7.117 | 2.041 / 6.266 / 9.881 |
| Submission/completion wall proxy | 5.709 / 8.070 / 9.641 | 6.621 / 9.445 / 11.704 |
| Touch to present submission | 35.965 / 52.890 / 61.423 | 39.697 / 57.714 / 63.433 |

The fresh Hybrid trial posted 638 frames with 616 distinct touch responses. It still reports
`llvmpipe (LLVM 21.1.4, 128 bits)`, a CPU-backed Vulkan adapter. Hybrid always encodes a full
scene, so the separate CPU repaint rectangles do not optimize that path. One trial gives
no between-trial spread; the historical value is context, not a paired experiment. Neither
its CPU encoding nor its completion wall proxy meets the 2/3 ms targets.

Raw logs, per-trial summaries, pooled cause reports and digests are under ignored
`build/stage3a-evidence/tails-before-*` and `tails-after-*`. Profiling itself has overhead;
an uninstrumented S24 trial with Perfetto is still required before acceptance.

## Text, visuals and final runtime check

Every valid fresh before/after Compose and CPU trial, plus Hybrid, matched both checkpoints:

| Checkpoint | UTF-16 units | SHA-256 of UTF-8 |
|---|---:|---|
| 300 presses | 300 | `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` |
| Plus eight popup holds | 308 | `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555` |

The default hold selects the number hint. The fresh held `e` capture also shows its accent
row. This is bounded input parity, not general action-trace equivalence. Rust tests cover
candidate cancellation at the left/right boundaries, pointer history/reordering, rolling
fingers, retained Shift, popup selection/cancel and full/dirty pixel comparisons.

Fresh, visually inspected artifacts (relative to the repository root):

- `build/stage3a-evidence/tails-final/side-by-side-light.png`
- `build/stage3a-evidence/tails-final/side-by-side-dark.png`
- `build/stage3a-evidence/tails-final/rust-popup-held.png`
- Raw `compose-light.png`, `compose-dark.png`, `rust-light.png`, `rust-dark.png` in that directory.

The pairs are crops of actual device screenshots made with `pair.swift`, without repainting
keyboard contents. Bundled key geometry, colours and labels remain close. Rust still has
approximate shadows and a full-width native candidate strip rather than Compose's toolbar,
widths and ellipsis. These remain milestone B visual differences.

On the final APK, dragging from a populated candidate cell out of the strip preserved the
editor digest; an in-strip candidate tap changed it as a positive control. Portrait →
landscape → portrait was exercised with Rust visible before each rotation; the disposable
editor was refocused after its activity recreated. Screenshots confirm 2400×1080 landscape
and 1080×2400 portrait, and fresh native frames rendered in both orientations.
`surface-lifecycle.log` contains **5 acquisitions and 5 releases**, maximum owned count
**1**, final count **0**. `runtime-summary.json` records the numeric assertions.

`adb logcat -d | grep -E 'FATAL|AndroidRuntime'` had **no matches** after the final runtime
checks and cleanup. All valid trial logs are also clean. Final log:
`build/stage3a-evidence/tails-final/final-logcat.txt`. The early blank captures were taken
during asynchronous configuration/surface transitions and are isolated in
`tails-lifecycle-blank/`; accepted pairs were recaptured after visible keys and native
frames were confirmed. AX settle alone does not establish a posted native surface.

The prototype was turned **off** and its persisted preference verified false, the probe app
was uninstalled, and `aw-compose`, `aw-emoji`, `aw-profile` and `aw-hybrid` were reset to INFO.
Streams 1, 2, 3 and 5 stayed at zero, alarm stream 4 at its allowed minimum of 1. No DND
change was made. The `shutdown: true` close request for `emulator-5556` returned
`Device shutdown failed: Could not communicate with device support. Try refreshing devices.`
Refreshing confirmed `aw_keyboard` was already **booted: false**; its stale panel entry
was then removed without another shutdown request. The other emulator remained booted.

## Pinned stack

Rust **1.98.0**, target `aarch64-linux-android`; one shipped `libawnative.so`.

- `vello_cpu = 0.4.0`, `vello_hybrid = 0.2.0`, `wgpu = 29.0.3`, Vulkan only.
- `parley = 0.13.0`, `read-fonts = 0.47.0` for inspection.
- CPU glifo **0.5.0**, Skrifa **0.49.0**, read-fonts **0.46.0**.
- Hybrid glifo **0.3.0** (PNG feature explicitly enabled), Skrifa **0.44.0**,
  read-fonts **0.41.0**. These are the versions required by the two Vello releases.
- `pollster = 1.0.1`, boundary `raw-window-handle = 0.6.2`.

Direct dependencies are exact pins; all transitive versions are locked in Cargo.lock.
No system fonts, test APKs, build outputs or local settings are committed.

Native library sizes: baseline **2,533,224 bytes**, CPU-only intermediate **4,947,248 bytes**,
prior continuation **8,798,352 bytes**, final **8,805,280 bytes** before Android stripping.
The APK packages **8,805,272 bytes**. The final unstripped increase over baseline is
**6,272,056 bytes**, about 5.98 MiB. No fonts, screenshots, APKs, traces, local settings or
agent configuration are committed.

The final required command `./gradlew --stacktrace test :app:lintDebug :lib:rust:check
:lib:rust:oracleCheck assembleDebug` exited **0**, `BUILD SUCCESSFUL in 1m 30s`
(343 tasks: 13 executed, 330 up-to-date). Its direct Gradle process status was checked,
without a logging pipe. Log: `build/stage3a-evidence/tails-final-ci-passed.log`.
All Rust tests (including 15 renderer tests), JNI/JVM tests, oracle comparisons,
clippy with `-D warnings`, rustfmt, lint and assembly passed. App lint contains exactly
AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport warnings;
Rust-module lint has no findings. Shell syntax and numeric popup summary grouping pass.
The lifecycle logger uses one shared tag constant for its guard and output; no lint
finding is suppressed. Every changed source file remains under 500 lines.

Hybrid's earlier shared outline/colour atlas experiment lost labels; disabling its atlas
entirely hit the unsupported uncached bitmap path. The retained correction draws ordinary
Hybrid outlines directly and keeps its atlas for colour fonts. The current CPU path keeps
its own outline atlas; the fresh host probe above did not justify disabling it.

## Verdict

Continue the pure Rust experiment with Compose as the default and the prototype off.
The measured repaint fix substantially reduces slow-frame frequency, and the bounded AVD
runtime check now includes the candidate edge-cancel correction. **Performance acceptance
still fails:** CPU p95 is 4.715–5.533 ms against 2 ms, Hybrid encoding p95 is 6.266 ms, and its
completion wall proxy p95 is 9.445 ms against 3 ms. Input-post tails and cold initialization
remain unresolved; visual equivalence is incomplete.

This evidence supports another real-device measurement, not a renderer-stack switch.
Measure the S24 Ultra's actual fonts and Vulkan GPU at 120 Hz with Perfetto FrameTimeline
and GPU timestamps. This emulator's llvmpipe cannot settle that decision. If phone tails
still miss after profiling, compare skia-safe using the same scenes, fonts, traces, memory
and shipped library cost. No skia-safe implementation, size or performance was measured here.

## Milestone B

- Inspect the Samsung phone's actual emoji tables and rerun all sequence probes. Broaden
  coverage beyond these fonts, including colour-font combinations and variation selectors.
- Alternate Compose/CPU/Hybrid on the S24 Ultra at 120 Hz with Perfetto FrameTimeline,
  GPU timestamp queries, cold/warm traces, memory and idle power. Measure actual presentation
  and missed frames; investigate tail costs and touch scheduling before default-on.
- Match candidate/toolbar geometry, clipping/ellipsis, font metrics/weights, shadows and
  popup placement/selection. Review pressed and shifted visual pairs at phone density.
- Replay broader identical action traces: rolling multi-touch/Shift, cancel, repeat,
  accent drags, suggestions, mode changes, surface loss and rotation during active input.
- Cover supported public font fallback lists rather than this AVD's small selected cohort,
  retain bounded caches and test fallback on initialization/device loss.
- The later full stage 3 gate still includes glide trails, panels, live theme-editor preview,
  Snygg in Rust and TalkBack semantics. None is claimed by milestone A.
