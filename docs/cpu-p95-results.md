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
