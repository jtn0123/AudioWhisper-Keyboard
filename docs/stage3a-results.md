# Stage 3A: pure Rust drawing prototype

**Stage 3A is committed on main; runtime continuation remains incomplete.** The renderer is
behind a default-off preference. The follow-up removed all helper AudioManager/DND use,
replaced the composable early return with if/else, documented the measurement hooks and
added opt-in numeric CPU attribution plus a pixel-parity/cold-versus-warm Rust test.
No performance optimization or new device-performance claim is made by this continuation.

The attribution build, including the candidate edge-cancel fix, was installed through
`scripts/install-dev.sh`. The fixed helper launched without the prior DND exception.
The first Compose trial was terminated before completing because device tools failed twice:

1. `device_screenshot` with the AVD name returned
   `Device aw_keyboard is not open in this thread. Call device_open first.`
2. Following that advice with `device_open(local, aw_keyboard)` returned
   `No device aw_keyboard on 'This machine' (hostId local). Call device_list for current ids.`

The initial successful open had returned runtime device ID `emulator-5556`. Reusing the
AVD name instead was an operator error. The task's retry-once/stop rule was followed after
the second failure. No complete new typing trial, alternating pair, tail breakdown, fresh
visual pair, popup capture, rotation or surface-recreation check was obtained. Earlier
evidence below is retained and is not relabeled as current-build acceptance.

Cleanup succeeded with the returned runtime ID: the prototype was switched **off** in the
UI and its persisted `keyboard__rust_renderer_prototype=false` was verified; the probe was
uninstalled, all four measurement tags reset to INFO, and only aw_keyboard was shut down.
`build/stage3a-evidence/final-logcat.txt` contained **no matches** for `FATAL|AndroidRuntime`.
That clean partial-session log does not replace the missing full Rust runtime check.

Measured on 2026-11-12 using the `aw_keyboard` AVD on host `local`, Android 16,
1080×2400, density 420, 60 Hz. Only `emulator-5556` was used. The other session's
`emulator-5554` was not used or closed. This is milestone A evidence, not the full stage 3 gate.

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
CPU drawing maintains a backing pixmap, clips dirty rectangles and honours the NDK lock's
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

Idle check: no additional native frames were posted during a 2.5-second unchanged interval
(628 before and 628 after). A later candidate edge-cancel correction has Rust coverage and was installed during the
continuation, but still lacks a complete runtime check; the retained measurements predate it.

## Tail attribution and continuation limits

Re-parsing retained numeric logs reproduces all reported percentiles. **265/627 CPU frames
(42.3%) exceeded 2 ms**, as did 265/630 Hybrid encoding frames (42.1%). Those logs have no
damage/cache/allocation fields, so their causes cannot be reconstructed. In particular,
there is no measured full/dirty, shift, popup, candidate, cold-glyph or allocation breakdown.
The incomplete Compose continuation is isolated in `build/stage3a-evidence/before-compose-1/`
and is excluded from all reported percentiles and digest comparisons.

`log.tag.aw-profile=DEBUG` at surface creation now enables timings-only CPU attribution:

- Full versus dirty redraw and clamped, outward-rounded damage pixels.
- ASCII case-change proxy for shift, popup open/move/close and candidate-strip changes.
- Scene encoding and CPU rasterization; layout-cache lookup/shaping and misses; glyph calls.
- Glyph time for first use of a shaped layout (a cold-work proxy) and colour-font glyph calls.
- Explicit allocation-site time/count for label-key strings, coordinate vectors and box paths.
- Backend/font initialization separately, before the first posted frame.

These are overlapping intervals, not additive causes. Colour fonts are detected by
COLR/CBDT/sbix tables; cold-layout counts do not expose Vello's private glyph/emoji-cache
misses. Allocation-site timing excludes library internals, JNI and scene construction;
no allocator interception or unsafe code was added to the render crate. The opt-in Rust
test proves equal pixels, cold/warm distinctions and zero counters when disabled. The
profiler's own overhead must be controlled with equally instrumented alternating trials.
`summarize.py --profile LOG` groups the frames over 2 ms and reports their areas and timings.

| Boundary (ms, p50 / p95 / p99) | Before (retained) | After continuation |
|---|---|---|
| CPU scene + raster | 0.689 / 7.765 / 12.367 | Not measured |
| Hybrid scene + command encoding | 1.571 / 5.234 / 7.117 | Not measured |
| Hybrid submission/completion wall proxy | 5.709 / 8.070 / 9.641 | Not measured |
| Compose HWUI draw recording | 1.377 / 2.295 / 5.024 | Not measured |
| Compose HWUI frame completion | 17.448 / 19.947 / 34.070 | Not measured |

No clear measured cause was available after the device stop, so no speculative cache or
damage optimization was shipped. The requested alternating trials (at least two of each),
cause breakdown and before/after remeasurement remain required before performance acceptance.

The helper never mutates audio. The host requests stream 1–5 volume zero using
`adb shell cmd media_session volume --stream N --set 0`; this AVD refuses alarm stream 4
at zero (allowed range 1–7). Other requests succeeded. The limitation is preserved in
`host-muting.log`; DND was never enabled as part of this continuation.

## Text and visuals

The retained pre-continuation Compose, CPU and Hybrid digest files were re-read and match:

| Checkpoint | UTF-16 units | SHA-256 of UTF-8 |
|---|---:|---|
| 300 presses | 300 | `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` |
| Plus eight popup holds | 308 | `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555` |

The default hold chooses the number hint; separate inspection showed `e`'s accents.
This is a bounded input comparison, not general action-trace parity. Rust tests also cover
rolling fingers, retained Shift, history/reordered pointers, leaving/re-entering keys, popup
selection and cancellation, dirty/full repaint parity including fractional edges/shadows,
and strip-only candidate damage. JVM tests exercise packing, selection and closed handles
through JNI without Android classes.

Retained pre-continuation screenshot paths (relative to the repository root); none were
recaptured during the continuation:

- `build/stage3a-evidence/side-by-side-light.png`
- `build/stage3a-evidence/side-by-side-dark.png`
- Raw captures: `compose-light.png`, `rust-light.png`, `compose-dark.png`, `rust-dark.png`
  in the same directory.
- `build/stage3a-evidence/rust-popup-held.png` shows the accent row.
- `build/stage3a-evidence/hybrid-light.png` shows the corrected Vulkan path.

The retained pairs crop and join the original device screenshots without repainting them. Key geometry,
bundled light/night colours and labels are close. Remaining visible differences include
approximate shadows and the full-width native strip rather than Compose's toolbar buttons,
candidate widths and ellipsis. Native candidates are clipped to their own cells; exact
ellipsis and toolbar layout need milestone B. Surface recreation and portrait/landscape
rotation were exercised before the continuation; acquired/released generation logs returned the owned window count
to zero, with no overlap above one.

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
pre-continuation CPU+Hybrid build **8,794,360 bytes**. The final continuation library is
**8,798,352 bytes** before Android stripping (increase **6,265,128 bytes**, about
5.98 MiB over baseline); the APK packages **8,798,344 bytes**.
The measured installed CPU+Hybrid build was 8,794,056 bytes; the final candidate edge-cancel
fix accounted for that build difference. The continuation installed the attribution build
with this fix, but did not complete its runtime check.

The full required command `./gradlew --stacktrace test :app:lintDebug :lib:rust:check
:lib:rust:oracleCheck assembleDebug` completed for the final source state with **Gradle exit
status 0**, `BUILD SUCCESSFUL in 1m 58s` (343 tasks; 29 executed, 314 up-to-date).
The direct Gradle process status was checked, without a logging pipe. Log:
`/tmp/aw-stage3a-final-ci-continuation.log`. Rust tests (including the 12 renderer tests),
JNI/JVM tests, clippy with `-D warnings`, rustfmt, C++ oracle comparisons, lint and assembly
passed. App lint contains exactly the three permitted warning IDs:
AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport; Rust-module lint
has no findings. Shell syntax and the tail-summary grouping check also pass.

The implementation, native boundary, setting/UI, harness and results are separated into
five reviewable commits with the requested co-author trailer. No generated evidence, APKs,
fonts, local settings or agent configuration are included. This build success does not
waive the incomplete current-build runtime checks. Push/CI status is reported separately
for the final main head.

Hybrid initially lost letter labels with a shared outline/colour glyph atlas. An experiment
disabling the atlas entirely exposed Hybrid's unsupported uncached bitmap-image path and
aborted the IME. The tested correction draws ordinary outlines directly, retaining the atlas
for COLR/CBDT/sbix fonts. The final screenshots and trials use this correction. The failed
experiments are preserved separately; they are not included in successful timing samples.

## Verdict

Continue the pure Rust experiment, keeping the setting off by default. Complete the
missing current-build runtime checks and tail attribution before accepting milestone A
or using these changes as a performance improvement claim. It can render the
AVD's COLRv1 and bitmap emoji and a recognizably matching bundled keyboard without Skia.
The simplest CPU path remains the prototype default; Vulkan is an explicit measurement
experiment via `log.tag.aw-hybrid=DEBUG`, with CPU fallback if initialization returns an error.
CPU has the lower median encoding/raster cost; Vulkan adds driver/submission work here and
does not establish a better input-latency tail. The emulator does not justify choosing it as
the default prototype backend.

**The performance acceptance gate is not passed:** CPU p95 exceeds 2 ms, Hybrid CPU encoding
p95 exceeds 2 ms, and its completion wall proxy exceeds 3 ms. Visual equivalence is also
incomplete. Do not remove Compose or enable Rust by default on this evidence. Profile the
tails and measure the real S24 Vulkan GPU before switching stacks: this AVD's llvmpipe is a
CPU renderer and cannot tell us whether a phone GPU meets the target. If the phone still
misses after profiling, compare skia-safe with the same traces, fonts and measured library
cost. No skia-safe implementation or performance claim is made here.

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
