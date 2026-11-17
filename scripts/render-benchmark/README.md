# Stage 3A local renderer comparison

This disposable editor saves UTF-16 length and SHA-256 of UTF-8 text, never editor contents.
Do not run it against personal text. It is not packaged with the keyboard.

The editor never changes audio or Do Not Disturb. `run.sh` sets volume zero on
streams 1, 2, 3 and 5 from the host with `cmd audio set-volume`, sets the allowed
alarm-stream minimum of 1 on this AVD, reads each stream back with `cmd audio get-stream-volume`,
stops on any mismatch, and saves responses in `host-muting.log`. `a11y_device.py`'s `mute()` does
the same, with accessibility (10) at its minimum of 1. Do not use `cmd media_session volume --set`:
on the android-36 AVD it prints success but leaves the volume unchanged (2026-11-14: media read
5/15 after the acceptance run's muting steps, and a direct `--set 0` left it at 5). Its `--get`
readback is accurate. Never enable DND as part of a trial.

The CPU-tail continuation records two alternating Compose/CPU trials before and after
separating strip/key damage, plus a Hybrid trial. See `docs/stage3a-results.md` for the
per-trial numbers, cause breakdown and final runtime evidence. Incomplete samples are
isolated under `tails-incomplete-*` and excluded.

`cpu-trials.py` runs those alternating Compose/Rust trials end to end with attribution on
(steps 3 to 5 below), each into a new folder; `dirty-vs-full.py` checks on the device that the
keyboard after many narrowed frames equals a fresh full redraw. Their use and the 2026-11-17
numbers are in `docs/cpu-p95-results.md`.

Open the chosen emulator with the T3 device tools and use the returned agent-device command
for all UI actions. After boot, use the returned runtime device ID (for example,
`emulator-5556`) for screenshot/close; the AVD name may no longer resolve. Set `ANDROID_SERIAL` to that emulator's serial for every adb command,
installation and benchmark invocation. Never borrow another session's emulator.

1. Run the full repository gate and `ANDROID_SERIAL=<serial> scripts/install-dev.sh`.
2. Run `scripts/render-benchmark/build.sh`, then install it with
   `scripts/install-dev.sh --probe build/render-benchmark/probe.apk`. Enable/select the keyboard's IME and mute the emulator's audio streams.
3. With targeted adb, set `log.tag.aw-compose=DEBUG` for numeric Compose frame-commit timing
   and `log.tag.aw-emoji=DEBUG` for a synthetic colour-emoji strip when candidates are empty.
   `log.tag.aw-hybrid=DEBUG` selects the Vulkan experiment when a prototype surface is created,
   in a build made with `-PhybridRenderer=true` (the `hybrid` Cargo feature); otherwise, and in
   every other build, where the switch only logs `hybrid_unavailable`, the prototype uses Vello CPU. `log.tag.aw-profile=DEBUG` at surface creation
   enables numeric CPU-tail attribution. Set these properties back to `INFO` afterwards.
4. In keyboard settings, turn **Draw the keyboard in Rust** off for Compose and on
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
fixture, and the script returns nonzero for expected-word/capital failures or digest/cursor
parity failures after completing both renderers (`result.json`). `--hold-ms 35` runs a
separate short-touch compatibility control; neither control
claims physical TalkBack explore/lift acceptance. The double-tap/drag accent attempt is
reported by its digest, never assumed to work because input injection returned success.

For a fresh TalkBack-off regression, prepare an empty normal probe on the chosen renderer,
restore accessibility, then use `run.sh adb compose|rust OUTPUT`. The adb adapter preserves
the existing 300 × 35 ms presses and eight × 500 ms `e` holds. Shut down only the dedicated
runtime device after restoring prototype off, Always day/Floris Day and accessibility,
removing the probe and checking logcat.

## Package 5 text modes and inline reservation

The prototype now draws every evaluated text keyboard, including symbols, numeric,
advanced numeric and phone. `modes.py` replays fixed public fixtures in all seven main
modes and a password field; each mode includes any applicable keyboard switches. It
requires real Rust virtual keys, checks expected digests/cursors, and rejects surface
create/change/destroy/attach events during the replay. `number-row.py` toggles the existing
Number row preference through settings, compares character/symbol switches, then restores
that preference. Number-field and phone-field resets exercise evaluator-driven switches.
The probe's field buttons support manual screenshots too. Both retain only digests/offsets.

```bash
ANDROID_SERIAL=<runtime-id> python3 -I scripts/render-benchmark/modes.py build/modes-unique
ANDROID_SERIAL=<runtime-id> python3 -I scripts/render-benchmark/number-row.py build/number-row-unique
ANDROID_SERIAL=<runtime-id> python3 -I scripts/render-benchmark/visuals.py \
  '<returned-agent-device-command>' build/visuals-unique
```

`visuals.py` selects Always day/night with the bundled Floris themes and captures empty
SYMBOLS, SYMBOLS2, NUMERIC, PHONE and password keyboards via agent-device. It restores Always
day and the original accessibility settings. Keep the agent-device session idle while the
adb harness prepares each state; no snapshot/injection helper should run alongside the
probe service. Actual cropped screenshot pairs put Compose on the left and Rust on the right.

`a11y.sh` now includes a real number field and compares actual Rust symbols, not the old
Compose fallback. The comparator maps explicit spoken punctuation/mode names to the exact
Compose glyphs and checks target geometry, roles, enabled states and clicks. Its negative
tests include swapped or displaced punctuation. An unexpected missing glyph remains a bug.

The 400 ms TalkBack fixture is still a raw-touch fixture and deliberately fails the expected
word check: ordinary long holds choose hints. Its former renderer divergence came from
Shift timing: the accessibility bridge used the base delay instead of Compose's 2.5× Shift
delay, locking caps in Rust. Native semantic snapshots now supply key-specific hold clocks.
The 35 ms control and the 300-press/eight-hold regression remain separate controls.

Inline autofill is hosted by the existing platform `InlineContentView` row in Rust's measured
center reservation. Rust suppresses its own drawing/hits/semantic candidates there and
preserves pinned controls, extended rows, shared-action precedence and dictation-preview
precedence. Rust popups participate in the platform popup count for inline-view z-order.
Native and JVM layout tests do not establish live autofill-service acceptance; report that
separately when no service-generated suggestion views were exercised.

Numeric and phone captures temporarily turn off both word and emoji suggestions through
settings: the text-only no-suggestions input flag does not apply to those field classes.
The harness restores both original switches before typing regressions and timing trials.

Phone digit/hint placement uses the same Rust phone-key content rule tested against a
frozen Kotlin `BiasAlignment` reference. Character/numeric/PHONE2 content stays centered;
only PHONE keys of type NUMERIC use the side-by-side digit/letter layout.

Open the probe's agent-device screenshot target **before** adb preparation. A redundant
`agent-device open` after `probe()` starts another activity and can destroy the prepared
IME surface while its semantic snapshot remains available. `visuals.py` now establishes
the target first, requires a fresh native posted frame for each Rust state, and rejects
native draw/post errors. Blank launch-fixture captures are excluded from visual acceptance.
Timing trials also reject draw/post errors and fewer than 100 measured CPU frames.

Package 6 adds `panels.py` for emoji/recents, editor-action cursor checkpoints and the exact
serialized action arrangement. It records public action rectangles with an empty
editor, then disables the observer during typed editing input to avoid synchronous
InputConnection/accessibility-query interference. Full trees require an empty editor. Each fixture saves
and restores history/arrangement preferences, including absent keys, and both suggestion
preferences. Restore the session's original accessibility values during final cleanup too.

```sh
ANDROID_SERIAL=<runtime> python3 -I scripts/render-benchmark/panels.py compose build/panels-compose
ANDROID_SERIAL=<runtime> python3 -I scripts/render-benchmark/panels.py rust build/panels-rust
ANDROID_SERIAL=<runtime> python3 -I scripts/render-benchmark/panel-visuals.py '<agent-device-command>' build/panel-visuals
python3 -I scripts/render-benchmark/compare-panels.py build/panel-visuals
```

The visual runner captures empty-editor Day/Night pairs and real accessibility targets for
two emoji categories, variants, overflow, editing buttons and the arrangement editor. It
keeps agent-device idle during adb-only input; only the explicit screenshot calls use it.
`panel-pair.swift` retains the full editor overlay. The comparator compares visible tile
wrappers, including partially clipped tiles whose Compose text leaf is offscreen, and
excludes the IME tree occluded by a modal sheet. It rejects missing labels, wrong enabled
states, missing clicks, target centers displaced by over 3 px and unexplained extra labels.
Native category, scroll, dismissal and history controls have explicit labels where Compose
had icon leaves. Run `python3 -I scripts/render-benchmark/test_panel_harness.py` for seven
adversarial targeting/comparator checks.

Each fixture starts a fresh disposable probe process. The harness unbinds its own observer
before replacing that process and rebinds afterwards, preserving other accessibility
services. Typed editing uses recorded visible-label points and scrolls through the grid
gutter, avoiding accidental editing-key holds during a scroll. Full tree capture never
runs while the disposable editor contains text.

## Package 8 IME chrome

With the dedicated device open and its runtime id exported as `ANDROID_SERIAL`, keep
agent-device navigation idle while these serial-scoped fixtures execute:

```bash
python3 -I scripts/render-benchmark/chrome.py workflow build/stage3b8/workflow-unique
python3 -I scripts/render-benchmark/chrome.py visuals build/stage3b8/visuals-unique \
  --agent '<device_open agent-device command>'
python3 -I scripts/render-benchmark/compare-chrome.py build/stage3b8/visuals-unique
python3 -I scripts/render-benchmark/host.py build/stage3b8/host-unique
```

The workflow uses the real space-hold subtype sheet, including a compact keyboard, and
compares persisted subtype/one-handed state and the fixed typing digest. Visuals capture
Day/Night subtype, compact subtype, both sides, incognito and landscape extraction; the
Swift pairing helper supports both orientations. Chrome comparisons check actionable
ancestors and the retained keyboard viewport. The harness restores private preferences,
rotation and accessibility settings in `finally`.

Close the agent-device UI automation session before starting the dump harness; reopen the
probe with `--no-test-ime` without taking a snapshot when screenshots are needed. A live
UiAutomation snapshot session prevents the disposable accessibility observer from binding.
Host measurements opt into numeric timestamps only: ten process-cold starts and ten
configuration recreations in one process per renderer, factory return and first layout.
These compare the current complete paths, not an unimplemented plain-View alternative or
physical presentation latency. See [package 8 results](../../docs/stage3b-package8.md).

## Stage 2 NLP orchestration

`nlp.py` replays fixed fast typing/candidate picks, autocorrect with a cursor move and a password
field on both renderers, and compares full digest/selection/composing results with the old build.
Pass `--agent` with the exact device_open command to capture typing/autocorrect/password screenshots.
It uses the disposable observer's narrowly allowlisted synthetic-field strip capture: labels are
hashed, and application/editor trees are never inspected. `compare-nlp.py` checks candidate order,
click actions and geometry, distinguishing the four fixed expanded-action controls from candidates.

```sh
python3 -I scripts/render-benchmark/nlp.py build/nlp-unique --baseline BASELINE/nlp.json \
  --agent '<device_open command>'
python3 -I scripts/render-benchmark/compare-nlp.py build/nlp-unique
python3 -I scripts/render-benchmark/test_nlp_harness.py
```

For learning, build `:app:assembleDebugAndroidTest -PrendererFixture=true` and install its test APK
through `scripts/install-dev.sh --probe`. `nlp-learning.py OUTPUT` starts/stops the test controller
for each renderer, learns a fixed synthetic word, takes it back through the existing manager
operation, checks the saved store digest returns exactly, then checks password counts and incognito.
It waits for an atomic receiver response and verifies instrumentation completion. Preserve and
restore the session's original learned/kept files separately; none of these scripts may erase them.

`input.py` retains the old empty repeat-delete fixture and adds `repeat-delete-2s-long`: the
probe starts with 1,088 UTF-16 units, holds Delete for two seconds, and saves remaining length,
digest and cursor. The longer prefix makes repeat-rate changes observable. Before/after comparison
requires the same remaining content; a digest of the empty string alone cannot establish the rate.
Run timed repeat fixtures with builds idle. The probe publishes `digest.json` by temporary-file
rename so host reads cannot observe a truncated JSON record. The test APK's `editor-state` operation
reports only numeric field/caps/shift metadata and refuses anything except an empty probe editor.
Before clipboard workflows, preserve the original Room files and the probe's `CLIP_STATE` snapshot;
restore the payload with `operation=restore`, then restore Room while its owner is stopped.

### Dedicated test-store sessions

For T3-owned acceptance, export `AGENT_DEVICE` as the exact command returned by
`device_open`, and `ANDROID_SERIAL` as its runtime serial. The shared adapter uses
agent-device for launches, settings clicks, taps, holds and text. Settings deep links
retain their original `android.intent.category.BROWSABLE` category. Double Shift uses
the dedicated `--double-tap` action: ordinary repeated host-dispatched taps can reach
Android's 300 ms cutoff despite a requested 80 ms pause.

The existing multi-segment/cancel/open-popup fixtures retain their documented input
runner. The existing `panels.py` 500 ms gutter swipes also retain their original
serial-scoped adb call. On 2026-11-14, agent-device's pan left `Ui Automation` attached
and suppressed the probe observer; subsequent service binding failed with zero new
AndroidRuntime matches. Closing/reopening only the owned T3 session releases it.
The user explicitly authorized these fixed scrolls; other navigation stays on agent-device.

`DEVICE_BASELINE` optionally names the fresh device's baseline directory, containing
`baseline.json` and `preferences.jetpref`. Store sessions validate its serial/hash,
restore exact preference bytes and accessibility values before/after each wrapper,
and record verification alongside the canonical store hashes. An old guest's pending
record does not apply merely because its numeric runtime serial is reused.

`acceptance-extras.py OUTPUT --baseline OLD_EXTRAS` checks all 17 existing records.
Its clipboard probe hashes rows in the app process; clipboard item labels are hashed
before dump publication. Empty original Room files remain on-device and their exact
restoration is verified. No clipboard payload database is exported. The disposable
`reader` field masks application glyphs while retaining accessibility text, allowing
real stop/highlight screenshots without saving the read-aloud source.

Run typing wrappers through `python3 -I scripts/render-benchmark/typing-suite.py OUTPUT` or
`python3 -I scripts/render-benchmark/store-session.py OUTPUT/stores -- COMMAND...`.
The latter also wraps dictation/extra wrappers. `nlp-learning.py` uses the same session itself.
Every wrapper starts and ends with the canonical synthetic learned/empty blocked fixtures,
installed with the IME stopped and checked by SHA-256. Each restoration is checked before the
next wrapper. Guest failure leaves `stores-pending.json` and fails the session; rerunning resets
the store first. No accumulated test words are backed up or logged. Keep ANDROID_SERIAL scoped
to the owned dedicated emulator and agent-device idle during scripts.

The restored SSD baseline has `smartbar__shared_actions_expanded=true`. The candidate
fixture must collapse that strip before typing: otherwise the fixed candidate coordinate
hits More actions instead. `suggestions.py` and `nlp.py` explicitly select the candidate
strip; hash-only strip geometry records distinguish this fixture-preparation failure from
an NLP regression. The enclosing state wrapper restores the original expanded flag and
exact preference bytes. Expected commit digests are unchanged.


### File-only speech and platform acceptance probes

`reader-ui.py OUTPUT` requires the masked `reader` probe field and rendererFixture test APK.
It verifies real muted neural playback, uses the floating Stop reading pill and records
reading/stopped screenshot pairs on both renderers. Its metadata bridge uses Java I/O
so historical unchanged APKs do not need unused Kotlin I/O facades retained.

For the seven-fixture dictation comparison, build main/test with `-PdictationFixture=true`,
install both with the installer, and seal their copies under ignored `build/`. Then run:

```sh
ANDROID_SERIAL=<owned-runtime> python3 -I scripts/dictation-proof.py OUTPUT \
  --apk SEALED_MAIN_APK --test-apk SEALED_TEST_APK
```

The runner retains a fresh hash-only oracle on-device. Rebuild with
`-PdictationFixture=true -PjniFreeFixture=true`, seal/install both APKs, then run with
`--native-only --oracle PREVIOUS_OUTPUT/comparison.json` and the new APK paths.
This uses the production `MicAudio.Replay` file producer and checks all seven fixtures
against the fresh oracle. `apk-maps.py` validates installed APK hashes, guards ZIP entry
paths/types/sizes before payload inspection, and resolves uncompressed ELF mappings by
APK offsets. A `.so` filename grep alone cannot establish absence of JNI libraries.

The rendererFixture controller also offers hash-only acceptance snapshots of all declared
preferences, subtype IDs/counts, MIME/locale/ref results, and fixed synthetic learned-store
mutation. The `dictationFixture` numeric `editor-meta` operation observes field/caps/shift
state inside the existing IME process; instrumentation startup would restart that process.
Neither hook is included in normal debug builds. All fixtures require exact canonical-state
restoration and never authorize opening the room microphone.
