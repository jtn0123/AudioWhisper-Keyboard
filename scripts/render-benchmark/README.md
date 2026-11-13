# Stage 3A local renderer comparison

This disposable editor saves UTF-16 length and SHA-256 of UTF-8 text, never editor contents.
Do not run it against personal text. It is not packaged with the keyboard.

The editor never changes audio or Do Not Disturb. `run.sh` requests volume zero on
streams 1, 2, 3 and 5 from the host with `cmd media_session volume`, requests the allowed
alarm-stream minimum of 1 on this AVD, and saves responses in `host-muting.log`. Never enable DND as part of a trial.

The CPU-tail continuation records two alternating Compose/CPU trials before and after
separating strip/key damage, plus a Hybrid trial. See `docs/stage3a-results.md` for the
per-trial numbers, cause breakdown and final runtime evidence. Incomplete samples are
isolated under `tails-incomplete-*` and excluded.

Open the chosen emulator with the T3 device tools and use the returned agent-device command
for all UI actions. After boot, use the returned runtime device ID (for example,
`emulator-5556`) for screenshot/close; the AVD name may no longer resolve. Set `ANDROID_SERIAL` to that emulator's serial for every adb command,
installation and benchmark invocation. Never borrow another session's emulator.

1. Run the full repository gate and `ANDROID_SERIAL=<serial> scripts/install-dev.sh`.
2. Run `scripts/render-benchmark/build.sh`, then install it with
   `scripts/install-dev.sh --probe build/render-benchmark/probe.apk`. Enable/select the keyboard's IME and mute the emulator's audio streams.
3. With targeted adb, set `log.tag.aw-compose=DEBUG` for numeric Compose frame-commit timing
   and `log.tag.aw-emoji=DEBUG` for a synthetic colour-emoji strip when candidates are empty.
   `log.tag.aw-hybrid=DEBUG` selects the Vulkan experiment when a prototype surface is created;
   otherwise the prototype uses Vello CPU. `log.tag.aw-profile=DEBUG` at surface creation
   enables numeric CPU-tail attribution. Set these properties back to `INFO` afterwards.
4. In keyboard settings, turn **Rust keyboard renderer (prototype)** off for Compose and on
   for Rust. Open `com.jtn0123.renderprobe` through agent-device with `--no-test-ime` so the
   helper does not replace the selected keyboard. Focus the editor, warm it with a few presses,
   clear it, and wait for the candidate strip to settle before each run.
5. On the tested 1080×2400 portrait AVD, run:

   ```bash
   ANDROID_SERIAL=<serial> scripts/render-benchmark/run.sh '<returned-agent-device-command>' compose
   ANDROID_SERIAL=<serial> scripts/render-benchmark/run.sh '<returned-agent-device-command>' rust
   ANDROID_SERIAL=<serial> scripts/render-benchmark/run.sh '<returned-agent-device-command>' hybrid
   python3 -I scripts/render-benchmark/summarize.py \
     --native build/stage3a-evidence/rust-timing.log \
     --compose build/stage3a-evidence/compose-gfx-*.txt \
     --compose-touch build/stage3a-evidence/compose-timing.log \
     --profile build/stage3a-evidence/rust-timing.log
   ```

   Pass an optional third argument (output directory) to `run.sh` to preserve each trial,
   e.g. `build/stage3a-evidence/before-compose-1`. Alternate Compose/Rust at least twice
   each before and after any optimization; do not overwrite a prior sample.
   Clear the editor between runs. Coordinates are specific to this AVD and layout. Each run
   presses 300 keys (35 ms hold, 85 ms gap), then holds `e` eight times for 500 ms. Compare both
   digest files between renderers. The default long-press selection is the number hint; inspect
   and select the accent choices separately. This script does not prove general input parity.

6. Capture light/dark screenshots through agent-device with an empty editor. Appearance
   changes recreate the probe activity and the IME surface: wait for that transition,
   refocus the editor, then verify native frames and visible keys before capturing. An AX
   settle alone can finish before a SurfaceView has posted its first complete keyboard.
   Then use
   `swift scripts/render-benchmark/pair.swift COMPOSE.png RUST.png PAIR.png` to crop/join the
   keyboard regions without redrawing their contents. The crop is specific to this AVD.
7. Turn the prototype off, reset measurement properties, check targeted logcat for
   `FATAL|AndroidRuntime`, uninstall `com.jtn0123.renderprobe`, then close/shut down only this
   device through the device tools.

All generated files go under ignored `build/`. Pull system fonts into a fresh external
directory. From `lib/rust`, run `cargo run --release -p render --example emoji_probe --
OUTPUT_DIR FONT...`; it records tables and verifies shaped/rasterized sequence coverage.
The Samsung phone needs its own font inspection later.

Native `draw_ns` measures CPU scene encoding+rasterization (CPU) or scene+GPU command encoding
(Hybrid). `gpu_wait_ns` is submission-to-completion wall time including the driver wait, not
a GPU timestamp query. Native `post_ns` follows unlock/post or Vulkan present submission.
Compose's opt-in callback follows HWUI frame commit; gfxinfo reports app-wide HWUI recording
and completion intervals. These are different measurement boundaries, and none is physical
touch-to-display presentation. Deduplicate touch timestamps to take the first response to
each event rather than later NLP updates. Use phone Perfetto FrameTimeline before acceptance.

The `--profile` report groups frames over 2 ms by full/dirty region, case-change proxy,
press/release, surface creation, popup open/move/close, candidate change and shaping-cache
miss. Groups overlap. It includes
damage pixels and encoding/raster/shaping/glyph/allocation-site times. `cold_glyph_ns`
means glyph calls for a newly shaped layout, not a private Vello atlas miss; `color_glyph_ns`
covers calls using COLR/CBDT/sbix fonts. `profile_init_ns` covers backend/font initialization
before the first frame. Explicit allocation-site counters cover our label-key strings,
coordinate vectors and box paths, excluding library internals and JNI/scene allocation.
These nested intervals must not be added together. Profiling itself adds overhead; compare
only equally instrumented trials and retain separate uninstrumented acceptance trials.

`buffer_post_ns` isolates the NDK lock, buffer copy and unlock/post interval. The frame's
post timestamp is captured before diagnostic formatting. `area_px` remains the bounding
rectangle supplied to the NDK; `paint_px` counts the exact union of rounded/clamped CPU
damage pixels, and `regions` gives the number of rectangular passes. Overlap is counted
once in `paint_px` even when a popup transition needs overlapping passes. Surface recreation
means the first frame of a new native worker. The shift flag remains an ASCII case-change
proxy; the scripted letter/hold trial does not exercise manual Shift.

Kotlin surface diagnostics also use the opt-in `aw-profile` tag. Numeric `view event`
codes are 2 created, 3 changed, 4 destroyed, 5 released, 6 attached, 7 detached,
10 rejected touch and 11 rejected update. The view identity and live/released flags are
numeric; these events never include input contents. Use them with native acquired/released
window generations when checking configuration changes. They add no logging to normal
key/scene updates.

## Package 3 gesture replay

The probe also stores `selection_start`/`selection_end` UTF-16 offsets; it never stores text.
With an empty, focused editor and the returned agent-device command, run:

```bash
ANDROID_SERIAL=<runtime-id> python3 -I scripts/render-benchmark/gestures.py \
  '<agent-device-command>' build/stage3b3/gestures/compose
```

Repeat from an empty editor on Rust. `gestures.json` records the digest and selection after
three fixed synthetic glides, a candidate pick, cursor and delete swipes, hold-repeat,
double/held Shift, two-pointer rolling and a cancelled glide. The multi-segment paths and
CANCEL use targeted `adb shell input motionevent`; agent-device handles taps and straight
pans. No path is recorded from a person. Coordinates assume the same portrait AVD as above.
Global configured swipe actions additionally require turning glide off in Gestures, replaying
the four cardinal swipes, and turning glide back on. Never silently compare disabled actions.

Pass `glide` as run.sh's fourth argument to include glide frames in an alternating timing
trial. The exact 300-press/eight-hold digest files are captured before those extra gestures.
Both native timing and Compose gfxinfo then include the additional glide/fade frames.

## Accessibility without UiAutomation

Close the dedicated runtime device's agent-device session with `device_close` (without
shutdown) first. These test runners use serial-scoped adb; never run standalone
uiautomator alongside them. `emulator-5554` is explicitly rejected.

```bash
export ANDROID_SERIAL=<runtime-id>
scripts/render-benchmark/build.sh
scripts/install-dev.sh
scripts/install-dev.sh --probe build/render-benchmark/probe.apk
scripts/render-benchmark/a11y.sh build/a11y-unique-run
python3 -I scripts/render-benchmark/compare-a11y.py build/a11y-unique-run
python3 -I scripts/render-benchmark/test_compare_a11y.py
python3 -I scripts/render-benchmark/talkback.py build/talkback-unique-run
```

`a11y.sh` records both secure accessibility values and restores them in `finally`, including
`settings delete` for an original `null`. SIGINT follows the same cleanup. Use a fresh
output directory per run; a hard process kill cannot execute cleanup, so retain the saved
original values. The disposable service alone is enabled for dumps. It retrieves the
`TYPE_INPUT_METHOD` window, clears Android's accessibility cache on every request, walks
all real/virtual children, and publishes JSON atomically. Each response has a unique request
ID, monotonic timestamp and content-event generation. Nonempty-editor dumps are refused.
Tree runs disable suggestions in the disposable editor to exclude candidate text. Typing
and regression runs keep suggestions enabled. No editor tree is traversed.

The harness opens the existing Keyboard settings page with a BROWSABLE deep link, activates
the existing prototype switch by its label, and polls the persisted preference. It waits
for the requested casing/state and stable node geometry before saving each tree. Caps lock
is held for 1,000 ms because the ordinary Shift long-press threshold is 2.5 times the
configured delay; a digest-only uppercase commit/erase checks persistence. The 700 ms `e`
hold runs in the background while a delayed request captures its popup at 450 ms.

The comparator writes `comparison.json` and prints both a table and every node difference.
It reports framework/text-hint wrappers, explicit key roles/actions/toggle names and
extended edge hit rectangles as acceptable differences. Ordinary target centers and named
control rectangles must agree within 3 px. Unexpected labels, disabled-state changes,
missing clicks, displaced targets and incorrect Shift states are bugs. Symbols are the
existing Compose fallback on both renderer settings. Adversarial comparator tests reject
wrong labels, missing keys/actions, displaced bounds, wrong classes and wrong Shift states.

TalkBack is selected exclusively through secure settings; its exact installed component is
parsed from `dumpsys package`. The probe must remain the foreground activity at every
checkpoint. Its notification permission is temporarily granted with `pm` to avoid the
first-enable dialog, then restored. The AVD must be launched with `-no-audio`; muting requests
zero on streams 1/2/3/5/10 and falls back to the allowed minimum, including alarm stream 4.
DND is explicitly off. Only digest/length/cursor values and whitelisted fixed announcement
words are saved. An enabled service or checked Shift node is not evidence of spoken output.

**Input boundary:** on this AVD, shell `input swipe` delivers raw touch even with TalkBack
exploration enabled. A 400 ms letter hold therefore selects configured hints on Compose;
it does not spell the intended word. The requested run remains a measured failure of that
fixture. `--hold-ms 35` runs a separate short-touch compatibility control; neither control
claims physical TalkBack explore/lift acceptance. The double-tap/drag accent attempt is
reported by its digest, never assumed to work because input injection returned success.

For a fresh TalkBack-off regression, prepare an empty normal probe on the chosen renderer,
restore accessibility, then use `run.sh adb compose|rust OUTPUT`. The adb adapter preserves
the existing 300 × 35 ms presses and eight × 500 ms `e` holds. Shut down only the dedicated
runtime device after restoring prototype off, Always day/Floris Day and accessibility,
removing the probe and checking logcat.
