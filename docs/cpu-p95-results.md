# The keyboard's CPU drawing gate: results (2026-11-17)

Rust draws the keyboard with Vello CPU. One of the four checks that still keep the Compose
keyboard as the fallback ([removal plan](compose-removal.md#k-the-compose-keyboard), K item 4)
is the CPU gate: **once warm, drawing a frame (building the scene and rasterizing it) takes at
most 2 ms at the 95th percentile.** The last measurement, on 2026-11-12, was 4.7–5.5 ms
([stage 3A](stage3a-results.md)). This page is how that came down, measured the same way.

## Verdict

**The 2 ms gate passes on the emulator. The phone still has to pass it.** In two alternating
trials on this branch, Rust's warm frame p95 is **1.257 and 1.107 ms**, down from 6.829 and
4.572 ms on main, and both typed exactly the same text as before. The median frame went from
about 1.1 ms to 0.29 ms, p99 from 7.9–15.5 ms to 2.9–3.8 ms, and the share of frames over 2 ms
from 15–20% to 3%. For scale, Compose's draw recording on the same emulator has a p95 of
5.2–6.2 ms.

On 2026-11-22, on a busier host, the same main code measured 2.46–2.97 ms, so the gate was not
met that day by main or by the key-press bubble; see
[the key-press bubble's kept image](#the-key-press-bubbles-kept-image-2026-11-22).

The gate was always meant for the S24 Ultra at 120 Hz. The number stays 2 ms there; what
measurement settles it is in
[decisions](decisions.md#the-keyboards-cpu-gate-stays-2-ms-settled-on-the-phone-at-120-hz-2026-11-17).

## How it was measured

* **The device.** The `aw_keyboard` emulator: Android 16, 1080×2400 portrait, density 2.625,
  60 Hz, the default `floris_day` theme, suggestions on, every audio stream muted. Nothing else
  ran on it.
* **One trial.** `scripts/render-benchmark/cpu-trials.py` picks the renderer, opens the
  disposable editor (`com.jtn0123.renderprobe`), warms the keyboard with three presses, clears
  the editor and waits for the suggestion row to settle. Then `run.sh` presses 300 keys (twelve
  letters, 25 each, 35 ms down and 85 ms apart) and holds `e` eight times for 500 ms to open its
  accent popup. Every UI action goes through the device tools' `agent-device` command.
* **Alternating.** Compose, Rust, Compose, Rust, so a slow stretch of the host hits both. Each
  trial gets its own new folder under `build/cpu-p95/` and none is ever reused: `before/`
  (main at 8d00f9b08) and `after/` (this branch).
* **What is timed.** For Rust, `draw`: how long the render thread takes, by the clock, to build
  the scene and rasterize it, one number per frame the keyboard posts. That is the gate's number.
  With `log.tag.aw-profile=DEBUG` each frame also logs where the time went (scene encoding,
  raster, text shaping, glyph drawing), how much of the surface it repainted, what kind of change
  caused it, and how long locking, copying into and posting the window buffer took; since this
  work, also the render thread's own CPU time for the draw (`cpu_ns`). Every logged value is a
  number; nothing typed or suggested is logged.
* **The Compose reference.** Android's own `gfxinfo` "draw recording" time for the Compose
  keyboard on the same emulator. It starts and stops at different points from Rust's `draw`, so it
  is a reference for scale, not a race.
* **The text typed.** Each trial saves the length and SHA-256 of the editor's text after the 300
  presses and after the popups, never the text. A Rust trial whose text is not the text the
  script expects stops the run; Compose's digests are recorded beside it. For the pixels, see
  [same output](#same-output).
* **Host bench.** `benchmark/keyboard-frame-timing.rs` replays the trial's frames on the Mac
  (the same QWERTY, suggestion row, styles and damage regions) so a change could be tried in
  seconds. Its numbers guided the work; the emulator numbers below are the evidence.

## Before and after

Rust's `draw`, in ms, per trial. "Over 2 ms" counts the posted frames that took longer.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms |
|---|---:|---:|---:|---:|---:|
| before, Rust 1 | 866 | 1.108 | 6.829 | 15.485 | 171 (19.7%) |
| before, Rust 2 | 862 | 1.060 | 4.572 | 7.910 | 126 (14.6%) |
| **after, Rust 1** | 813 | **0.287** | **1.257** | 3.794 | 26 (3.2%) |
| **after, Rust 2** | 822 | **0.292** | **1.107** | 2.860 | 24 (2.9%) |

Compose on the same emulator, `gfxinfo` draw recording, in ms (the reference; Compose did not
change):

| Trial | Frames | p50 | p95 | p99 |
|---|---:|---:|---:|---:|
| before, Compose 1 | 1,166 | 1.082 | 5.995 | 14.079 |
| before, Compose 2 | 1,156 | 0.920 | 5.421 | 11.206 |
| after, Compose 1 | 1,083 | 1.573 | 6.214 | 10.085 |
| after, Compose 2 | 1,078 | 1.074 | 5.208 | 8.127 |

All eight trials typed the same text: the 300-press digest `809d588b…` and the after-popups
digest `c14d19d2…`, Compose's included.

From a touch to the frame being posted, Rust, p50/p95/p99 in ms: before 43.7/106.5/162.9 and
42.1/78.5/148.1; after 40.6/56.0/65.7 and 39.6/56.2/65.0. Compose's touch to frame commit
(a different end point): before 42.8/90.3/192.4 and 41.4/66.2/80.4; after 39.8/62.0/70.1 and
39.7/57.2/64.5. Most of the time from touch to post is not drawing; see
[what is left](#what-is-left).

By kind of frame, Rust p50 / p95 in ms (trial 1, trial 2):

| Frame | Before | After |
|---|---|---|
| Key press | 0.905 / 3.786, 0.894 / 2.231 | 0.177 / 0.309, 0.181 / 0.325 |
| New suggestions only | 0.809 / 4.732, 0.776 / 4.753 | 0.365 / 2.860, 0.354 / 2.230 |
| Key release with new suggestions | 1.551 / 8.492, 1.465 / 5.908 | 0.376 / 1.447, 0.359 / 1.570 |
| Popup opening, moving, closing | 1.412 / 3.013, 1.153 / 4.910 | 0.262 / 0.551, 0.267 / 1.780 |
| Whole keyboard (the first popup) | 11.350, 10.954 | none: no longer happens |

## What each change bought

The six drawing commits, in order. The host bench numbers are medians on the Mac, in ms; the press
and release columns are the typing frames, the full column a redraw of the whole keyboard.

| Commit | What it does | Press | Release with new words | Full redraw |
|---|---|---:|---:|---:|
| (main) | | 0.75 | 1.04 | 9.6 |
| `rasterize only the damaged rectangle` | Vello was given the whole 1080×712 surface for every frame and unpacked and repacked every pixel of it. It now gets only the damage. | 0.68 | 0.90 | 9.6 |
| `leave out the shadow an opaque box hides` | The two blurred shadow layers under every key were most of the raster time. The part a key's own opaque box covers is no longer computed. | 0.43 | 0.62 | 5.1 |
| `popup styles repaint only the popup` | The theme sets the popup's style as it opens, which counted as a change to everything: the first popup redrew the whole keyboard (10.95 ms in the before trial). | 0.43 | 0.62 | 5.1 |
| `a press that only recolours a key repaints its box` | A press repainted the key and six shadow-widths around it, about 30,000 pixels. In `floris_day` a press changes only the key's background, so now only the box repaints. | 0.09 | 0.28 | 5.1 |
| `new words repaint only the words' area` | New suggestions repainted the whole suggestion row, the round toggle's shadow included. Now only the words' part. | 0.09 | 0.21 | 5.1 |
| `skip shadows and glyph runs the damage cannot show` | Anything near the damage was rebuilt whole. Now each shadow and each line of text is checked against the damage by how far it really reaches. | 0.08 | 0.17 | 5.1 |

The popup commit does not move the bench, which only draws popups after the theme has set them;
on the emulator it removes the one 11 ms frame each trial had.

On the emulator, between the first two commits and the rest:

| Emulator trial (Rust only) | Built from | p50 | p95 | p99 | Over 2 ms |
|---|---|---:|---:|---:|---:|
| before (two trials) | main | 1.06–1.11 | 4.57–6.83 | 7.9–15.5 | 15–20% |
| iteration 1 | the first commit and an early version of the second | 0.691 | 3.224 | 6.183 | 82 of 840 (9.8%) |
| iteration 2 | the first five | 0.332 | 1.154 | 2.884 | 18 of 832 (2.2%) |
| after (two trials) | all six | 0.287–0.292 | 1.107–1.257 | 2.86–3.79 | 2.9–3.2% |

The last commit took most of the text out of suggestion-row frames (glyph runs drawn per frame
fell from 21 to about 1), but on the emulator that is lost in the noise of the tail: its p95 and
p99 are within the spread between trials.

## Same output

* **Text.** All eight before and after trials typed the same text (digests above), and so did
  the two CPU-time trials.
* **Pixels, in the tests.** Every change only repaints less of the surface or skips drawing
  that cannot show. The render tests compare each narrowed frame with a full redraw of the
  same scene and require them to be identical, pixel for pixel. The shadow change is also
  identical to the bit against the old full shadow, over five shapes, four elevations, four
  sizes and four sub-pixel positions.
* **Pixels, on the emulator.** `scripts/render-benchmark/dirty-vs-full.py` types 36 keys, holds
  `e` for its popup and clears the editor, which draws 65 narrowed frames, then takes a
  screenshot. It then hides and reshows the keyboard, which makes a new surface
  drawn whole (the log shows `recreate=1 full=1`), and takes a second one. The bottom 1,000 rows
  (the whole keyboard) are identical in both (`build/cpu-p95/visual-check/`).

## What is left

About 3% of frames still take more than 2 ms. They are not slow drawing.

* **Nearly all are suggestion-row updates** (49 of the 50 in the two after trials). The same kind
  of frame usually takes 0.35 ms. In a slow one, a single phase (encoding or raster) takes many
  times its usual time for the same work: the same area and the same number of text runs, and
  only 2 of the 50 had a text-shaping miss.
* **The render thread was waiting, not working.** To check, the profile now logs the thread's
  own CPU time beside the wall time of each draw (`cpu_ns`). In two more Rust trials with it
  (`build/cpu-p95/cpu-time/`), wall time had p95 0.792 and 1.003 ms and p99 2.95 and 2.99 ms,
  while CPU time had **p95 0.531 and 0.571 ms and p99 0.788 and 1.175 ms**. Of the 37 frames
  over 2 ms of wall time, only 5 used more than 2 ms of CPU. In the median slow frame the
  thread ran for under a quarter of the time. The worst took 10.7 ms of wall time and 0.64 ms
  of CPU.
* **Why it waits.** A suggestion-row update arrives just as the suggestion engine finishes, when
  the keyboard's other threads are busy too. The emulator has 4 virtual CPUs, and the Mac
  running it was also running another session's emulator and other work (a load average of
  7 to 12 on its 6 performance cores during the trials). Even the 5 frames that used over 2 ms
  of CPU may be inflated: when the host pauses a virtual CPU, the guest can still count that
  time as the thread running.

**Posting the frame** is not part of `draw`, but it is most of the time from a touch to a posted
frame. The render thread waits about 10.4 ms (median) for a free window buffer, which is the
display's 60 Hz pace, not CPU; its p95 fell from 26.4 ms to 15.5–15.8 ms. Copying the changed
pixels takes 0.017 ms. This waiting happens on the render thread, so it does not hold up typing.
Whether it costs frames at 120 Hz is for the phone's FrameTimeline trace to show.

**On the host**, the same frames take 0.08 ms (press) and 0.17 ms (release with new words) at
the median. That is a Mac running natively, so it is no prediction for the phone. It does show
that the drawing work itself is now well under the gate.

## Tried and rejected

* **Clearing the target instead of drawing over it** (`TargetInit::Clear`). No measurable gain;
  reverted.
* **A cheaper copy into the window buffer.** Not worth it: the copy takes under 0.02 ms at the
  median. The ~10 ms spent locking the buffer is waiting for a free buffer at 60 Hz, not CPU
  work, and is not part of `draw`.
* **A fixed grid of cells**, each rasterized separately. Anything crossing many cells (the
  suggestion row, a full redraw) would have been encoded once per cell. Vello's own target offset
  gives a damage-sized target with no grid.
* **Culling text by its layout box plus a guessed margin.** Stacked accents and wide glyphs can
  reach past any fixed margin. The cull uses each font's own glyph bounds (`head`) instead.
* **Not tried again, as agreed:** the multi-worker CPU raster, and the Vulkan (Hybrid) path, which
  stays a build option that is off by default.

## A known limit

A key is skipped whole when its cell, widened by six shadow-widths, misses the damage. That was so
before this work and is unchanged. A label drawn further than that outside its own key (with the
2dp shadow about 31 px; with no shadow, any overflow at all) is not repainted when only a
neighbour changes. Labels are centred on their key and not squeezed to fit it, so a very long
label could do this. The test for text reaching into a neighbour uses a shadow big enough to keep
within that reach, and the commit that added the finer culling records the limit.

## The key-press bubble's kept image (2026-11-22)

The key-press preview bubble ([rust.md](rust.md#the-key-press-preview-bubble-added-2026-11-22))
is drawn once into an image and laid over the keys on every press. These trials compare it with
main as the trials above were run: `cpu-trials.py --rust-only`, one trial per build, main
(d2d88b6c3), the bubble branch, main, the branch, on `aw_keyboard`, with the bubble on (its
default). The folders are in `build/cpu-p95/press-bubble-kept-image/`.

**The 2 ms wall-clock gate was not met, by main or by the branch.** This is not a pass.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|
| main 1 | 1,036 | 0.952 | 2.966 | 6.383 | 118 (11.4%) | 1.393 |
| branch 1 | 1,018 | 0.908 | 3.032 | 7.228 | 107 (10.5%) | 1.487 |
| main 2 | 1,066 | 0.766 | 2.457 | 5.057 | 80 (7.5%) | 1.256 |
| branch 2 | 1,024 | 1.091 | 3.399 | 6.566 | 127 (12.4%) | 1.476 |

All four typed the expected text (`809d588b…`, and `c14d19d2…` after the popups).

**Why main missed it.** The main code that measured 1.107–1.257 ms above measured 2.457–2.966 ms
here. A press frame took 0.78–0.89 ms at the median, against 0.177 ms above, and a touch took
123–130 ms to reach a posted frame, against about 40. The host bench draws the same frames on
the Mac and is unchanged: a press 0.073 ms and a release with new words 0.159 ms, against 0.08
and 0.17. So the drawing work has not grown; the emulator ran about four times slower. Two other
emulators and other agents' Gradle builds were running on the Mac at the time.

**What the bubble adds.** Profile medians per trial, in ms. "Rest" is a frame's `draw_ns` less
its encoding and raster, which for the branch is the bubble's own work: on a press, keeping the
46,306 pixels under its damage and laying 39,910 pixels of image over them; on a release,
putting the 46,306 back. Each column is its own median, so they need not add up.

| Trial | Press | encode | raster | rest | Release, no new words | rest | New words only |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 | 0.887 | 0.269 | 0.540 | 0.005 | 0.796 | 0.004 | 0.971 |
| branch 1 | 0.941 | 0.261 | 0.523 | 0.118 | 0.819 | 0.059 | 0.809 |
| main 2 | 0.780 | 0.242 | 0.485 | 0.004 | 0.742 | 0.003 | 0.712 |
| branch 2 | 1.109 | 0.268 | 0.535 | 0.122 | 0.919 | 0.061 | 1.027 |

* A press's encoding and raster are main's: Vello still repaints only the pressed key's box.
  The bubble adds about 0.12 ms to a press and 0.06 ms to a release.
* Each trial drew 11 bubble images, each at the first press of the next key in the run; the
  other 280-odd presses drew none.
* The frames the bubble never touches (new words only) moved by more between trials, 0.71 to
  1.03 ms, than the bubble costs.
* CPU time p95 stayed under 2 ms in all four: main 1.26–1.39 ms, the branch 1.48–1.49 ms. As
  in [what is left](#what-is-left), most of the time over 2 ms is the render thread waiting.

**Trials thrown out.** Of the ten trials that reached the typed-text check, six failed it, on main
and the branch alike: 300 units with the wrong digest, cause unknown (owed: E29 in
[owed checks](owed-checks.md#emulator-checks-the-docs-owe)). One stopped on an agent-device
error (E30). Their folders are kept beside the four above. An earlier attempt
(`build/cpu-p95/press-bubble-cache/`) never typed: agent-device's snapshot helper, left
running by earlier recordings, kept the probe's accessibility service from binding until the
device session was closed and opened again.

**Still owed.** The same trials on a quiet host, where the bubble build's p95 must be at most
2 ms (E28), and on the phone (P18).

## A label too big for its key (2026-11-22)

The key-label fit ([rust.md](rust.md#a-label-too-big-for-its-key-fixed-2026-11-22)) checks every
label against its box and fits the ones that do not fit. These trials compare it with main
(da6f3649f) as the trials above were run: `cpu-trials.py --rust-only --trials 1`, main, the
branch, main, the branch, on `aw_keyboard` with Floris Day, the Rust renderer and the press
bubble on. No build ran; two other agents' emulators were up. The folders are in
`build/cpu-p95/key-label-fit/`.

**The 2 ms wall-clock gate was not met, by main or by the branch.** This is not a pass.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|
| main 1 | 1,054 | 0.993 | 3.155 | 6.328 | 143 (13.6%) | 1.608 |
| branch 1 | 1,021 | 1.008 | 2.819 | 5.615 | 126 (12.3%) | 1.504 |
| main 2 | 1,085 | 0.980 | 3.166 | 5.535 | 130 (12.0%) | 1.547 |
| branch 2 | 1,087 | 1.084 | 3.123 | 7.305 | 139 (12.8%) | 1.660 |

All four typed the expected text (`809d588b…`, and `c14d19d2…` after the popups).

Medians per trial, in ms, of a press frame, its encoding and raster, and the rest of it (the
bubble's copy and blend, which this change does not touch), a release that brought no new words
and its rest, and a frame that brought only new words:

| Trial | Press | encode | raster | rest | Release | rest | New words |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 | 1.071 | 0.266 | 0.539 | 0.114 | 0.906 | 0.066 | 0.908 |
| branch 1 | 1.016 | 0.268 | 0.530 | 0.121 | 0.841 | 0.057 | 0.946 |
| main 2 | 1.076 | 0.276 | 0.534 | 0.147 | 0.895 | 0.067 | 0.910 |
| branch 2 | 1.194 | 0.315 | 0.559 | 0.182 | 0.947 | 0.091 | 1.050 |

* The branch is not slower beyond noise. Its p95 is 2.819 and 3.123 ms against main's 3.155 and
  3.166. Branch 1's press, encoding and raster are at or under both of main's. Branch 2 is
  slower in every column, the bubble's untouched copy and blend among them, so that trial's
  emulator ran slower, not its code.
* On the Mac, `keyboard_frame_timing` run ten times each, alternating the two binaries, gave the
  same: p95 medians for a press of 0.084 ms on main and 0.085 on the branch (0.081–0.086 and
  0.083–0.085), a release 0.178 and 0.177, a popup 0.179 and 0.177. The mean raster of a press
  moved from 0.062 to 0.063 ms. No frame of the bench fits a label.
* Main missed 2 ms as it did earlier the same day (2.457–2.966 ms), on the same slow emulator.
  CPU time p95 stayed under 2 ms in all four (1.50–1.66 ms).

**Trials thrown out.** Three on main never started: "Accessibility service did not bind", while
agent-device's snapshot helper held a UiAutomation connection left from the device checks.
Closing and opening the device session again freed it (E30). Of the branch's first three tries,
one stopped on agent-device's "Android automation helper output could not be parsed" (E30), one
typed the wrong digest (E29), and the third is branch 1 above. The load average rose to 59 for a
minute at the start of the second, from macOS's XProtect scan. All the folders are kept.

**Still owed.** The CPU gate on a quiet host, main against the bubble and this fit (E28), and on
the phone (P18).

## The typing-event fan-out (2026-11-22)

The typing-event fan-out ([phase3-typing-fan-out.md](phase3-typing-fan-out.md)) sends each
typing event to Rust as one call that returns one plan, and has Rust choose the smartbar's
icons, arrow and labels. These trials compare it with the main it was built on (f1fdf3f34; main
has since moved to 0d7196c8d, the theme property editor, which touches neither path). They were
run as the trials above: `cpu-trials.py --rust-only --trials 1`, main, the branch, main, the
branch, on `aw_keyboard` with Floris Day, the Rust renderer and the press bubble on. The device
session was closed and opened again first. The folders are in `build/cpu-p95/typing-fan-out/`.

**The 2 ms wall-clock gate was not met, by main or by the branch.** This is not a pass.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|
| main 1 | 1,070 | 1.144 | 3.465 | 6.791 | 163 (15.2%) | 1.655 |
| branch 1 | 1,081 | 1.097 | 3.557 | 6.919 | 147 (13.6%) | 1.602 |
| main 2 | 1,070 | 1.106 | 3.366 | 6.455 | 156 (14.6%) | 1.697 |
| branch 2 | 1,085 | 1.098 | 3.084 | 6.442 | 153 (14.1%) | 1.529 |

All four typed the expected text (`809d588b…`, and `c14d19d2…` after the popups).

The change moves work done on a key's release, so the trials also give touch to post, and the
p50 draw time, in ms, of a press frame, a release frame and a frame that brought only new words:

| Trial | Touch to post p50 | p95 | Press | Release | New words |
|---|---:|---:|---:|---:|---:|
| main 1 | 107.2 | 261.3 | 2.785 | 3.117 | 2.805 |
| branch 1 | 110.5 | 272.4 | 2.604 | 3.493 | 3.132 |
| main 2 | 116.1 | 263.7 | 2.683 | 3.104 | 2.655 |
| branch 2 | 107.9 | 288.3 | 2.572 | 2.802 | 2.802 |

* The branch is not slower beyond noise. Its p95 is 3.557 and 3.084 ms against main's 3.465 and
  3.366, one above both of main's and one below both. Its CPU time p95, 1.602 and 1.529 ms, is
  under both of main's (1.655 and 1.697), and so is its share of frames over 2 ms.
* The branch's touch-to-post p95 is higher in both trials (272.4 and 288.3 ms against 261.3 and
  263.7). The same measure on unchanged code ran from 259.1 to 301.8 ms in the other trials this
  day (the press bubble's and the label fit's, main and branch alike), and its p50s here are
  level, so this is read as noise, not a cost. Touch to post is dominated by the emulator's input
  injection, at about 110 ms even at its median.
* The press, release and new-words medians move both ways between trials, by as much between
  main's two as between the builds.

**Trials thrown out.** Five tries at main's first trial failed before main 1 above. None was on
the branch's build, and all the folders are kept:

* `main-1` and `main-1-e` typed the wrong digest (E29), with 300 units each.
* `main-1-b` stopped on agent-device's `Error (COMMAND_FAILED): Android automation helper output
  could not be parsed` (E30).
* `main-1-c` stopped on agent-device's `Error (COMMAND_FAILED): Android system dialog is blocking
  longpress: Application Not Responding: com.android.systemui`, while the load average rose to
  30. Its hint was to close the dialog. `agent-device alert dismiss` chose "Close app", so
  System UI restarted (E35).
* `main-1-d` stopped on "Accessibility service did not bind". The alert calls had left
  agent-device's UiAutomation connection held. Closing and opening the device session again freed
  it (E30).

The host was not quiet. Other jobs held up to 13 of the 18 cores before the trials. During the
four kept trials, another job's simulator held about three cores, and the load average stayed
at 7.9 to 8.2.

**Still owed.** The CPU gate on a quiet host (E28), and on the phone (P4).

## The key and layout object graph (2026-11-22)

The key graph ([phase3-key-graph.md](phase3-key-graph.md)) builds and computes the keys the Rust
renderer draws in Rust, and answers the smartbar's questions from them. These trials compare it,
at 28fce464e, with the main it was built on (b10bee9b1). Main has since moved to 20b07e922
(step 10's extension screens), which touches neither path. They were run as the trials above:
`cpu-trials.py --rust-only --trials 1`, main, the branch, main, the branch, on `aw_keyboard`
with the Rust renderer, and with the settings the device had before this work restored. The
device session was closed and opened again first. The folders are in
`build/cpu-p95/key-graph/`.

**The 2 ms wall-clock gate was not met, by main or by the branch.** This is not a pass.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|
| main 1 | 1,156 | 0.993 | 3.238 | 6.165 | 164 (14.2%) | 1.490 |
| branch 1 | 1,128 | 1.054 | 3.279 | 6.862 | 164 (14.5%) | 1.752 |
| main 2 | 1,172 | 1.024 | 3.433 | 5.604 | 166 (14.2%) | 1.659 |
| branch 2 | 1,161 | 0.991 | 3.322 | 6.954 | 151 (13.0%) | 1.595 |

All four typed the expected text (`809d588b…`, 300 units, and `c14d19d2…`, 308 units, after the
popups).

Touch to post, and the p50 draw time, in ms, of a press frame, a release frame and a frame that
brought only new words:

| Trial | Touch to post p50 | p95 | Press | Release | New words |
|---|---:|---:|---:|---:|---:|
| main 1 | 141.1 | 320.1 | 2.692 | 3.238 | 2.920 |
| branch 1 | 125.1 | 278.2 | 2.760 | 2.350 | 2.772 |
| main 2 | 133.2 | 282.5 | 2.721 | 2.802 | 3.123 |
| branch 2 | 141.0 | 310.7 | 2.974 | 2.813 | 2.758 |

* The branch is not slower beyond noise. Its p95, 3.279 and 3.322 ms, lies between main's 3.238
  and 3.433. Its share of frames over 2 ms is 14.5% in one trial, against main's 14.2% in both,
  and 13.0% in the other.
* Its CPU time p95 is 1.752 and 1.595 ms against main's 1.490 and 1.659: one above both of
  main's, one between them. That is 0.093 ms over main's higher trial, while main's own two
  trials differ by 0.169, so it is read as noise.
* Touch to post moves both ways, and the release and new-words medians are level or lower on
  the branch.
* The press frame's median is higher on the branch in both trials, 2.760 and 2.974 ms against
  2.692 and 2.721, on 43 to 58 frames each. The release median moves by 0.46 ms between the
  branch's own two trials, so this is not read as a cost. It points the same way twice, though,
  so the phone's p95 (P4) should look at press frames.

The host was quieter than for the trials above, but not quiet. The load average was 6.2 when
main 1 started and fell to 2.9 by branch 2's start, ending at 4.9.

**Still owed.** The CPU gate on a quiet host (E28), and on the phone (P4).

## Theme and subtype (2026-11-22)

Phase 3 item 3 ([phase3-theme-subtype.md](phase3-theme-subtype.md)) moves the keyboard's theme
choice, the theme session, the base style and the inline chips' colours to Rust. These trials
compare the branch before its last rebase (0ca6c1ced, built on f4aa9fc30) with that main
(f4aa9fc30, built in a worktree of its own). The rebase onto 48b2e0018 brought only settings
changes (compose-removal steps 10 and 12) and touches neither path. They were run as the trials
above: `cpu-trials.py --rust-only --trials 1`, main, the branch, main, the branch, on
`aw_keyboard` with the Rust renderer, and with the settings the device had before this work
restored. The folders are in `build/cpu-p95/theme-subtype/`.

**The 2 ms wall-clock gate was not met, by main or by the branch.** This is not a pass.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p50 | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 (`main-1-b`) | 1,061 | 1.054 | 3.010 | 5.411 | 131 (12.3%) | 0.898 | 1.476 |
| branch 1 (`branch-1-d`) | 1,075 | 1.031 | 2.741 | 5.049 | 119 (11.1%) | 0.885 | 1.513 |
| main 2 (`main-2-c`) | 1,067 | 0.996 | 3.031 | 6.684 | 120 (11.2%) | 0.876 | 1.576 |
| branch 2 (`branch-2`) | 1,080 | 0.892 | 2.789 | 5.645 | 112 (10.4%) | 0.765 | 1.493 |

All four typed the expected text (`809d588b…`, 300 units, and `c14d19d2…`, 308 units, after the
popups). Touch to post, p50 and p95 in ms: main 116.6 and 246.9, 125.4 and 269.9; the branch
127.5 and 264.9, 128.2 and 258.7.

* The branch is not slower beyond noise. Its p95, 2.741 and 2.789 ms, is below both of main's,
  3.010 and 3.031, and so is its share of frames over 2 ms.
* Its CPU time p95, 1.513 and 1.493 ms, lies between main's 1.476 and 1.576.
* The branch drew 8 to 19 more frames per trial. Each trial has 300 press and 300 release
  frames and no recreate or full frame; the difference is all frames that brought only new
  words to the strip (440 and 448 on the branch, 426 and 431 on main). Trials of unchanged
  builds in earlier items ranged from 409 to 534 such frames, so this is read as the strip's
  timing, not the theme.
* Touch to post is 2 to 11 ms higher at p50 on the branch, and its p95 lies within main's.

**Thrown out, folders kept.** Six trials did not finish with the expected text and were run
again: `main-1`, `branch-1-b`, `branch-1-c`, `main-2` and `main-2-b` typed 300 units with
another digest (E29), and `branch-1` was stopped by agent-device's parse error at a load
average of 43 (E30). `explain-digest.py` explains four of the five from the digest alone: each
typed one key as its long-press hint. `main-2-b` is not explained.

The host was loaded by other jobs. Load averages at the trials' starts ran from 7.9 to 31.7,
and reached 44 by the end of two thrown-out trials; the four kept ones ended at 8.3 to 11.1.

**Still owed.** The CPU gate on a quiet host (E28), and on the phone (P4).

## Subtype (2026-11-22)

The subtype half of phase 3 item 3 ([phase3-theme-subtype.md](phase3-theme-subtype.md#the-subtype-half))
moves the subtype list and the active subtype to Rust and pairs glide's keys with the subtype
they were built for. It changes no drawing code. These trials compare the final build
(4b32b51bb) with main (b1cf45f18), each built in a worktree of its own: `cpu-trials.py
--rust-only --trials 1`, alternating main and the branch three times each, on `aw_keyboard`
with the Rust renderer, and with the settings the device had before this work restored. The
folders are in `build/cpu-p95/subtype/`.

**The 2 ms wall-clock gate was not met, by main or by the branch.** This is not a pass.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p50 | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 (`main-1`) | 1,043 | 0.996 | 3.030 | 5.259 | 124 (11.9%) | 0.852 | 1.423 |
| branch 1 (`branch-1-b`) | 1,108 | 0.986 | 3.395 | 6.343 | 162 (14.6%) | 0.849 | 1.610 |
| main 2 (`main-2`) | 1,092 | 1.000 | 2.860 | 4.687 | 134 (12.3%) | 0.848 | 1.438 |
| branch 2 (`branch-2`) | 1,046 | 0.986 | 3.135 | 5.331 | 113 (10.8%) | 0.837 | 1.388 |
| main 3 (`main-3-b`) | 1,063 | 1.014 | 3.459 | 6.467 | 141 (13.3%) | 0.851 | 1.585 |
| branch 3 (`branch-3`) | 1,113 | 0.910 | 3.293 | 7.553 | 125 (11.2%) | 0.799 | 1.457 |

All six typed the expected text (`809d588b…`, 300 units, and `c14d19d2…`, 308 units, after the
popups). Touch to post, p50 and p95 in ms: main 124.6 and 277.1, 121.4 and 255.6, 119.7 and
264.5; the branch 124.2 and 275.4, 123.8 and 273.0, 129.7 and 284.8.

* The branch is not slower beyond noise. Its p95, 3.135 to 3.395 ms, lies within main's 2.860
  to 3.459, and so does its CPU time p95, 1.388 to 1.610 against 1.423 to 1.585.
* Its p50 and its CPU time p50 are at or below main's in every trial.
* Its share of frames over 2 ms, 10.8 to 14.6%, lies around main's 11.9 to 13.3%. Most of those
  frames on both builds are frames that changed the strip's words (72 to 109 a trial).

**Thrown out, folders kept.** Two trials typed 300 units with another digest (E29) and were run
again: the branch's first (`branch-1`) and main's third (`main-3`). `explain-digest.py`
explains main's from its digest alone, two `r`s typed as their hint `4`; it does not explain
the branch's. The reruns came after the branch's second trial, so the order was main 1, the
branch's thrown-out 1, main 2, branch 2, branch 1 again, main's thrown-out 3, branch 3, main 3
again.

The host's load averages at the trials' starts and ends ran from 2.5 to 8.0.

**Still owed.** The CPU gate on a quiet host (E28), and on the phone (P4).

## The plain-View root (2026-11-23)

Phase 3 item 4 ([phase3-view-root.md](phase3-view-root.md)) hosts the Rust renderer in a plain
`FrameLayout` instead of the Compose root. A hosting change, so these trials compare the final
build (fe8dad380) with main (0500aa74f), each built in a worktree of its own:
`cpu-trials.py --rust-only --trials 1`, alternating main and the branch three times each, on
`aw_keyboard` (emulator-5558) with the Rust renderer, and with the settings the device had
before this work restored. The folders are in `build/cpu-p95/view-root/`.

**The 2 ms wall-clock gate was not met by main, and the branch met it in one trial of three (and in
neither of the two after the insets fix, below).**
This is not a pass: the branch's other two trials are 2.374 and 2.157 ms, on a loaded host.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p50 | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 (`main-1-b`) | 1,062 | 0.980 | 2.959 | 5.430 | 135 (12.7%) | 0.854 | 1.504 |
| branch 1 (`branch-1`) | 943 | 0.977 | 2.374 | 3.808 | 73 (7.7%) | 0.878 | 1.378 |
| main 2 (`main-2`) | 1,018 | 0.907 | 3.522 | 5.833 | 125 (12.3%) | 0.780 | 1.467 |
| branch 2 (`branch-2-b`) | 944 | 1.009 | 2.157 | 3.738 | 66 (7.0%) | 0.883 | 1.333 |
| main 3 (`main-3-b`) | 1,071 | 1.053 | 3.231 | 5.609 | 148 (13.8%) | 0.904 | 1.541 |
| branch 3 (`branch-3-b`) | 945 | 0.992 | 1.992 | 3.440 | 48 (5.1%) | 0.887 | 1.316 |

All six typed the expected text (`809d588b…`, 300 units, and `c14d19d2…`, 308 units, after the
popups). Touch to post, p50 and p95 in ms: main 121.6 and 265.4, 125.6 and 265.2, 127.3 and
266.4; the branch 101.6 and 241.9, 108.5 and 253.8, 97.6 and 227.6.

* **The branch is faster beyond noise.** Its p95, 1.992 to 2.374 ms, is below all of main's,
  2.959 to 3.522; its p99, 3.440 to 3.808, below all of main's, 5.430 to 5.833; its share of
  frames over 2 ms, 5.1 to 7.7%, about half of main's 12.3 to 13.8%; and its CPU time p95, 1.316
  to 1.378, below all of main's, 1.467 to 1.541. Its p50s lie within main's.
* **It draws fewer frames for the same typing.** Counted from the frames' own flags in
  `rust-timing.log`: the branch drew 308 press and 308 release frames in each trial and 319 to
  321 that only changed the strip's words; main drew 294 to 307 of each and 422 to 449 strip
  frames. Strip frames are full-width, so they are most of main's frames over 2 ms (80 to 101 a
  trial, the branch 10 to 25). The root rebuilds at most once a frame from every flow it
  watches (`RootWatch`, on the Choreographer); where main's extra strip frames come from was not
  traced further.
* **It is not only fewer frames.** The press and release frames alone have a p95 of 2.145 to
  2.492 ms on the branch and 2.807 to 3.231 on main; the strip frames alone, 1.814 to 2.397
  against 3.333 to 4.239.
* Touch to post is 13 to 30 ms lower at p50 on the branch, and its p95 lies below main's.

**Thrown out, folders kept.** Four trials typed 300 units with another digest (E29), each a key
typed as its long-press hint, and were run again at once: `main-1` (one key), `branch-2` (two),
`main-3` (one) and `branch-3` (one).
`explain-digest.py` explains each typing digest; their popup digests follow from the same wrong
text. The order was main 1 (thrown out, then again), branch 1, main 2, branch 2 (thrown out,
then again), main 3 (thrown out, then again), branch 3 (thrown out, then again).

The host was loaded by other jobs. The kept trials' load averages ran from 4.3 to 7.2 at their
starts and 4.1 to 11.7 at their ends; the thrown-out `branch-2` started at 12.7.

**After the insets fix.** A later fix (a root swapped in at runtime now asks for the window's
insets; see [phase3-view-root](phase3-view-root.md#insets-and-the-full-screen-editor)) adds one
insets dispatch per attach and nothing per frame. One trial of its build and one of the final
build, after main's and the branch's above:

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p50 | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|---:|
| insets fix (`fix-1-b`, `af2e1e34`) | 942 | 0.996 | 2.330 | 3.629 | 71 (7.5%) | 0.870 | 1.343 |
| final (`final-1-c`, `abc67621`) | 942 | 0.941 | 2.124 | 3.494 | 58 (6.2%) | 0.855 | 1.389 |

Both typed the expected text (`809d588b…`, 300 units; `c14d19d2…`, 308), and both lie within the
branch's spread above. Touch to post, p50 and p95: 110.4 and 233.5 ms, 103.6 and 245.9. Three
attempts were thrown out first, each one key typed as its long-press hint (E29): `fix-1`,
`final-1` and `final-1-b`. Across this section the first attempt hit E29 in 2 of main's 5 runs
and 5 of the branch's 9; the root does nothing per touch beyond its own dispatch (`rise` returns
at once outside full-screen), and E29's known cause is agent-device delivering the up late.

**Still owed.** The CPU gate on a quiet host (E28), and on the phone (P4).

## The Compose-free render path (2026-11-23)

[The Compose-free render path](phase3-compose-free-path.md) changes what the renderer is sent:
icon tokens instead of flattened paths, a palette made in Rust, StateFlows instead of Compose
state, and a theme's fonts. These trials compare its build (51a7be022; e75a095eb after it adds
only an instrumented test) with main (4d3b5d3ae), each built in a worktree of its own:
`cpu-trials.py --rust-only --trials 1`, alternating main and the branch three times each, on
`aw_keyboard` (emulator-5558) with the Rust renderer, and with the settings the device had
before this work restored. The folders are in `build/cpu-p95/compose-free/`.

**Neither build met the 2 ms gate, and the branch is neither faster nor slower beyond noise.**
This is not a pass: the host was loaded throughout.

| Trial | Frames | p50 | p95 | p99 | Over 2 ms | CPU time p50 | CPU time p95 |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 (`main-1-b`) | 945 | 0.982 | 6.043 | 11.369 | 166 (17.6%) | 0.885 | 4.441 |
| branch 1 (`branch-1`) | 981 | 0.986 | 6.147 | 11.650 | 177 (18.0%) | 0.880 | 4.974 |
| main 2 (`main-2`) | 943 | 0.947 | 3.157 | 6.372 | 109 (11.6%) | 0.823 | 2.069 |
| branch 2 (`branch-2-c`) | 981 | 0.949 | 2.665 | 5.468 | 77 (7.8%) | 0.847 | 1.582 |
| main 3 (`main-3`) | 944 | 0.916 | 2.116 | 3.931 | 59 (6.2%) | 0.833 | 1.355 |
| branch 3 (`branch-3`) | 983 | 0.927 | 2.269 | 3.731 | 67 (6.8%) | 0.831 | 1.326 |

All six typed the expected text (`809d588b…`, 300 units, and `c14d19d2…`, 308 units, after the
popups). Touch to post, p50 and p95 in ms: main 107.3 and 263.3, 102.8 and 247.6, 108.9 and
260.8; the branch 100.3 and 269.7, 108.7 and 261.7, 103.0 and 248.1.

* **The pairs track the host, not the build.** Each pair's p95s are close (6.043 and 6.147,
  3.157 and 2.665, 2.116 and 2.269) and fall together as the load fell. The p50s are within
  0.011 ms of each other in each pair. Touch to post is the same within its spread.
* **By kind of frame,** counted from the frames' own flags in `rust-timing.log`, neither build
  leads. The press and release frames' p95: main 5.754, 3.167 and 2.116 ms; the branch 7.000,
  2.577 and 2.269. The strip-only frames' p95: main 6.121, 3.155 and 2.107; the branch 4.558,
  2.670 and 2.287.
* **The branch draws 36 to 38 more frames, all of them strip-only.** Its presses, releases,
  popups and layout-cache misses match main's exactly: 308 press frames, 300 release frames,
  16 popup frames, and 42 frames with 85 misses in every trial (main 2: 307 and 299). Its
  strip-only frames number 357, 357 and 359, against main's 321, 321 and 320.
  * The extra frames are intermediate lists. After most presses the strip's words change once.
    After 49, 49 and 51 of the branch's presses they change twice, against 13, 14 and 12 of
    main's: first a list drawn with fewer labels (a median of 20), then the final list (21 or
    22), 28 to 44 ms apart at the median in both builds. Two branch trials shared only 11 of their 49 such presses, so it is timing,
    not particular words.
  * The branch does not touch the suggestion pipeline, so the lists are the same; the branch
    shows the first one in a frame of its own more often. A likely cause, not traced: main's
    snapshot flattened every Smartbar icon through Compose's `PathParser` on each rebuild, and
    the branch sends tokens, so its main thread is freer to reach a frame between the two
    lists. The first change comes 56 to 75 ms after the press on the branch at the median,
    76.5 to 127 on main, which fits but overlaps too little to prove it (E56).
  * Each strip frame paints the full-width strip, 99,666 pixels. The extra frames did not move
    the branch's p95 beyond its pair's.
* **Layout and shaping did not change:** the same 85 layout-cache misses in the same 42 frames.
  The over-2 ms profile counted 12 shape-cache-miss frames on `branch-1` against 3 on
  `main-1-b`, but that is only which of those same 42 frames crossed 2 ms.

**Thrown out, folders kept.** Three trials typed 300 units with another digest (E29), each one or
two keys typed as their long-press hints, and were run again at once: `main-1` (two keys),
`branch-2` (one) and `branch-2-b` (two). `explain-digest.py` explains each typing digest. The
order was main 1 (thrown out, then again), branch 1, main 2, branch 2 (thrown out twice, then
again), main 3, branch 3. Their strip changes fit the rest: main 1 changed twice after 17
presses, branch 2 after 51 and branch 2-b after 75.

The host was loaded by other jobs. The kept trials' load averages ran from 8.3 to 12.5 at their
starts and 7.8 to 12.5 at their ends.

**Still owed.** The CPU gate on a quiet host (E28), and on the phone (P4); why the branch shows
the intermediate strip list more often (E56).

## Reproducing

```bash
# Host bench, from lib/rust (the numbers above used this font)
cargo run --release -p render --example keyboard_frame_timing -- \
  ../../fastlane/staging/fonts/Roboto-Light.ttf

# Emulator trials: open the AVD with the device tools first, then
export ANDROID_SERIAL=<serial> AGENT_DEVICE='<returned agent-device command>'
python3 -I scripts/render-benchmark/cpu-trials.py build/cpu-p95/<new folder>
```

`--trials N` sets the number of alternations, `--rust-only` skips Compose while iterating, and
`--no-profile` leaves attribution off. The script sets the log tags back to `INFO` and restores
accessibility when it ends.
