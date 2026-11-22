# Stage 4 follow-up 3: prototype debts closed, then group 1

The Rust settings renderer stays **default off** and Compose stays in place. Part A closes the
debts [follow-up 2](stage4-followup2.md#still-owed-before-prototype-acceptance) left; Part B
ports the first group of further screens (see [Group plan](#group-plan)). Raw receipts are
under ignored `build/stage4-followup3/` and `/Volumes/2TB/aw-scratch/`; neither holds typed
text, learned words or clipboard contents.

## A1. Font advances

Parley laid labels out about 1.5% wider than Android's Paint. There was no single cause:
Compose's static text goes through Minikin, which lays out at `trunc(size)`, uses the
TrueType interpreter's hinted advances for hinted fonts (rounded linear advances otherwise),
splits letter spacing half before and half after each cluster and drops the outer halves,
shapes each line on its own and never breaks after a hyphen. `android_text.rs` replays those
rules over Parley's shaping. A device fixture records StaticLayout and TextRunShaper output
for every size settings uses (144 cases, 7,524 glyphs and carets); the host test matches all
of them within 0.0098 px, and each rule's mutation fails it. Commit `37bdda43`.

## A2. Fresh pictures and the strict diff

The last complete capture is `build/stage4-followup3/pictures-final`: 17 pictures × light,
dark and font 2.0, 51 pairs. It was compared with `scripts/settings-picture-diff.py`
(3/255 per channel, one-pixel neighbour band, 72 px system bars excluded, no alignment).
It was taken on the build with every fix below **except** the font-2 row rules and the
slow-scroll helper, which came out of this capture. Pixels above the threshold (max channel
error):

| Picture | Light | Dark | Font 2 |
| --- | ---: | ---: | ---: |
| home | 0 (0) | 0 (0) | 87,367 (225) |
| keyboard | 0 (0) | 0 (0) | 155,765 (225) |
| keyboard-lower | 0 (1) | 0 (1) | 128,572 (225) |
| dialog-00 | 0 (1) | 0 (1) | 174,003 (223) |
| dialog-01 | 0 (1) | 0 (1) | 179,337 (223) |
| dialog-02 | 0 (1) | 0 (1) | 322,245 (207) |
| dialog-03 | 0 (1) | 0 (1) | 276,896 (223) |
| dialog-04 | 0 (1) | 0 (1) | 151,704 (223) |
| dialog-05 | 0 (1) | 0 (1) | 227,690 (223) |
| dialog-06 | 0 (1) | 0 (1) | 161,452 (223) |
| dialog-07 | 0 (1) | 0 (1) | 164,754 (207) |
| dialog-08 | 0 (1) | 0 (1) | 315,275 (223) |
| dialog-09 | 0 (1) | 0 (1) | 215,247 (223) |
| dialog-10 | 0 (1) | 0 (1) | 230,244 (223) |
| dialog-11 | 0 (1) | 0 (1) | 111,941 (207) |
| dialog-12 | 0 (1) | 0 (1) | 112,701 (207) |
| dialog-13 | 12,881 (90) | 13,268 (94) | 204,612 (223) |

32 of 51 pairs match strictly. All 33 light and dark pairs except dialog 13 have zero
differing pixels, with a maximum channel error of 1. Dialog 13's background list stopped a
few pixels apart because of a still-running fling (fixed in the helper, below). The font-2
differences were measured before the three font-2 fixes below. **A fresh final-source capture
is owed:** `aw_keyboard_ssd` dropped off adb before it could be taken (see
[Still owed](#still-owed)).

The differences the strict diff found, and what each was:

- **Line boxes.** StaticLayout gives a line whole pixels from the font's rounded
  `FontMetricsInt` ascent and descent; an explicit line height makes each line
  `ceil(lineHeight)` tall, the descent taking `ceil(diff / 2)` (M3's `LineHeightStyle(Center,
  Trim.None)`). Rust used Parley's fractional block, so text sat up to a pixel off.
  `android_text::line_metrics` now does it Android's way.
- **Dialog buttons** measure their label box in whole pixels, as Compose's layout does.
- **Icon roles.** A top bar's navigation icon is onSurface; the preview field's trailing
  icon is onSurfaceVariant, focused or not. Rust drew both onSurface.
- **Scroll position.** ScrollState and LazyListState move layout by whole pixels and carry the
  remainder, so content shows `round(position)`; Rust drew the fractional position. A pushed
  destination is a new back-stack entry with a fresh ScrollState, so it opens at the top; Rust
  restored the earlier visit's position.
- **Fling ends.** Compose's drag handles each event in `DragGestureNode`: slop is judged at the
  event's final position, crossing it scrolls by the over-slop only, an unmoved pointer
  neither scrolls nor reaches the tracker, and the tracker is Compose's own `Lsq2`
  `VelocityTracker1D` (20-sample ring, 100 ms horizon, 40 ms stop, unweighted quadratic),
  not the framework's. `compose_velocity.rs` ports it step for step; a test-only
  `ReferenceSettingsDrag.kt` drives the real Compose tracker and `SettingsDragMatchTest`
  matches Rust against it.
- **Input dispatch.** With that in place, unbuffered input still ended flings a few pixels
  away from Compose, because Compose itself receives batched input: the resampled samples
  differ. The same scrolls, measured on the device as content offset in Compose scroll units
  (`/Volumes/2TB/aw-scratch/scroll-probe5`):

  | Gesture | Compose | Rust, unbuffered | Rust, batched |
  | --- | ---: | ---: | ---: |
  | agent-device default scroll | 1413 | 1419 | 1413 |
  | 1900 px in 400 ms | 1888 | 1879 | 1888 |
  | 1800 px in 400 ms | 1787 | 1779 | 1787 |
  | 1200 px in 400 ms | 1183 | 1179 | 1183 |
  | 600 px in 400 ms | 580 | 579 | 580 |
  | 600 px in 1200 ms | 579 | 579 | 579 |

  Batched input is the default again; `log.tag.aw-unbuffered DEBUG` restores unbuffered
  dispatch for measurement. The trade-off is latency (A5): unbuffered input presents about
  11 ms sooner at the median and 20 ms sooner at p95. Parity was chosen; that call is the
  user's to revisit.
- **Back-to-back flings are not deterministic** on either renderer (each fling's end depends
  on when the next touch lands), so the picture helper now scrolls slowly
  (`scroll down 480 --duration-ms 1200`) and dialog 13's background no longer differs.
- **Font scale 2.** Three real differences, all fixed: Compose keeps a line height's
  *ratio* to the font size under nonlinear font scaling (`resolveLineHeightInPx`, ui-text
  1.9.4), so 20/14 of the scaled 14 sp, not a separately scaled 20 sp; Material 3 ListItem
  makes an item three-line when its supporting text is taller than `30.sp.roundToPx()`
  (`isSupportingMultilineHeuristic`, M3 1.4.0), with 12 dp padding and an 88 dp minimum;
  and home rows are one-line ListItems, 8 dp vertical padding and 56 dp minimum, where Rust
  used 16 dp. `rows_follow_material_list_item_padding_and_the_multiline_heuristic` checks
  each with the real Roboto-Light; each mutation (old line height, old heuristic, 16 dp
  padding) fails it. What remains at font 2 is the allowed difference: Rust wraps titles,
  summaries, the app bar title and dialog titles where Compose ellipsizes them.

## A3. The whole instrumented suite

**Not completed on the final source; owed.** The suite has passed only its `metrics` mode
(the A1 typography fixture, `build/stage4-followup3/instrumentation-fixture2`) and its
`crash` mode, light and dark, on the A7 build (`a7-after`). The ui, text and death modes have
not completed on a device in this follow-up. Nor has the stronger audit (known-defect
classification, A4).

**First attempt.** `aw_keyboard_ssd`'s adb transport went `offline` and stayed so after one
`adb reconnect offline`. The emulator console still said "virtual device is running", with
host swap at 17.4 of 18.4 GB. T3 then reported "Device emulator-5556 was not found on host
local". The device part stopped there. That cleanup
(`/Volumes/2TB/aw-scratch/performance-a5/cleanup.json`) had restored:

* prefs `0e622df7…`, learned words `7848a516…`, kept-as-typed `e3b0c442…`;
* our IME, accessibility off and font scale 1.0.

**Second attempt** (22:57–23:05, source `8f890579`, receipts
`build/stage4-followup3/suite-final`). Setup was clean:

* `settings-device-baseline.py --verify` passed before anything changed: files, IME,
  accessibility, font scale, rotation, animations, dark mode. `selected_input_method_subtype`
  and the subtype history matched too.
* All six streams were muted and verified.
* The fixture build had no warnings, and `install-dev.sh` plus the test APK installed cleanly.

Then four ui runs:

1. Stopped at the runner's first step, which closes the agent-device session: none was open
   (`SESSION_NOT_FOUND: No active session`). Rerun with `SETTINGS_AGENT_CLOSED=1`.
2. The Compose run failed in Sounds & Vibration. The walk took the preview field's
   "Switch Keyboard" button for a row and opened the system picker.
3. With that fixed, the Compose run passed home (12 routes), the keyboard screen and its 14
   dialogs, Sounds & Vibration (both passes, including the vibrator-dependent rows), Smartbar
   and Gestures & Glide typing (both passes). It failed in Clipboard: reading a master after
   the first pass needed the top of the list, and `reveal` only scrolls down. Both were walk
   bugs, fixed in `8f890579`.
4. About two minutes into the Compose run, `am instrument` exited 255 and adb showed
   `emulator-5556 offline`. The one allowed `adb -s emulator-5556 reconnect` gave "device
   'emulator-5556' not found"; the transport then came back `offline` again. Swap was still
   17.4 of 18.4 GB. The device part stopped there, and the emulator was shut down through T3.

The run's cleanup could not reach the device, so **the AVD is not restored**. Its app prefs
are in that run's fixture state. Learned words and kept-as-typed were restored and verified
when the fixture was written, and the settings walk does not touch them; our IME was set at
the same point. Booting `aw_keyboard_ssd`, restoring with `SettingsDevice.cleanup` and
`settings-device-baseline.py --verify` is owed before anything else uses it.

To run it: open `aw_keyboard_ssd` through T3, then
`AGENT_DEVICE=… SETTINGS_AGENT_CLOSED=1 SETTINGS_RECEIPTS=build/stage4-followup3/suite-final scripts/settings-device-tests.sh emulator-5556 .device-backup/stage4-followup3 all`.
The transport has now dropped in three device sessions in a row on this host, each time with
swap nearly full. It needs a host with memory to spare, or the phone.

## A4. The 23 unlabeled controls

The 23 are audit nodes, counted at every checkpoint that saw them, not 23 distinct controls.
All sit inside Jetpref, which this repository vendors as a binary snapshot. 13 are the bare
switch that `ListPreference` draws for a linked row (`android.view.View`, checkable): its row
is labelled, the switch node is not, and nothing outside Jetpref can name it. 10 are the
Material `Slider` inside `DialogSliderPreference`'s dialog (`SeekBar`, two per dual-slider
dialog). Our code builds neither node and no thin wrapper reaches them; fixing them means
forking Jetpref. They stay **owed**.

The suite now reports them apart from violations, by name, instead of failing on them or
passing them silently. `SettingsHarness.knownDefect` recognises exactly those two node shapes,
and only on the Compose path. Every one goes into `accessibility-result.json` as
`compose_known_defects_owed` (name, the control it belongs to, occurrences, screens); the run
prints a `KNOWN DEFECT, OWED` line for each, and `acceptance_complete` stays false while any
remains. The Rust path is asserted to have none.

## A5. Comparable frame timing

**Getting native frames into FrameTimeline.** `ANativeWindow_setFrameTimelineInfo` is not
exported to apps, so a buffer posted through ANativeWindow carries no vsync id. The CPU path
now presents through an `ASurfaceControl` child of the SurfaceView's window
(`render_surface.rs`): three `AHardwareBuffer`s, the damage copied into a released one, and
each transaction tagged with `ASurfaceTransaction_setFrameTimeline(vsyncId)`. The vsync id is
`Choreographer.FrameData.preferredFrameTimeline.vsyncId` from `postVsyncCallback`, the Java
face of `AChoreographer_postVsyncCallback`. The symbols are resolved at run time because the
native libraries link against the API 26 stubs. SurfaceFlinger now records the Rust frames as
app SurfaceFrames on the `aw-settings` layer, as it does HWUI's.

**Like for like.** Receipts: `/Volumes/2TB/aw-scratch/performance-a5` (eight trials, same
fixture, same scroll script of three 1100 px pans each way, builds idle, `aw_keyboard_ssd`
at 1080×2400, 60 Hz) and its `summary.json`. For each input event, the summary takes the
`Choreographer#doFrame <vsyncId>` it was delivered in (or the next one), then the app
SurfaceFrames carrying that vsync id, and reports their present time minus the event time.
That is the same boundary for HWUI and for the Rust layer.

| Trial (capture order) | Input → present, p50 / p95 ms (240 events) | App deadline missed / SurfaceFrames | PSS KiB |
| --- | ---: | ---: | ---: |
| 0 Compose | 28.14 / 48.74 | 0 / 356 | 100,790 |
| 1 Rust, CPU raster | 31.35 / 46.24 | 0 / 383 | 101,884 |
| 2 Rust, CPU raster | 30.91 / 46.64 | 2 / 381 | 102,311 |
| 3 Compose | 30.65 / 51.52 | 1 / 352 | 99,706 |
| 6 Rust, unbuffered input | 19.28 / 26.61 | 0 / 375 | 104,910 |
| 7 Rust, unbuffered input | 19.78 / 27.31 | 0 / 373 | 103,766 |

On batched input, Rust and Compose present about equally fast. Paired with the adjacent
Compose trial (0 with 1, 3 with 2), Rust's medians are 3.21 and 0.26 ms higher and its p95s
2.50 and 4.88 ms lower. App jank is equally rare: 2 missed app deadlines for Rust, 1 for
Compose. Compose trial 3 also had 59 buffer-stuffed frames (queueing, not a missed
deadline). Nearly every frame of both renderers carries SurfaceFlinger's own Prediction
Error on this emulator, so it is excluded from app jank. The input delivery interval explains
most of the latency: batched events reach the app 18–24 ms after their timestamp, unbuffered
ones in 2 ms. That is the parity-versus-latency trade-off described in A2.

**CPU raster or hybrid GPU.** The hybrid path presents through wgpu's swapchain, which carries
no vsync id, so it has no SurfaceFrames (the six in its trials are the activity window's). It
is compared on the renderer's own per-frame logs from the same scroll script, on frames that
repaint the scrolling list:

| Per frame, p50 / p95 ms | CPU raster (trials 1, 2) | Hybrid GPU (trials 4, 5) |
| --- | ---: | ---: |
| Scene draw | 2.04 / 2.37, 2.01 / 2.38 | 1.97 / 2.67, 1.92 / 2.46 |
| GPU wait | 0 | 5.59 / 6.58, 5.55 / 6.34 |
| Worker start → frame presented | 5.28 / 6.33, 5.30 / 6.61 | 8.18 / 10.94, 8.12 / 10.62 |
| Touch → first post | 30.96 / 46.32, 30.55 / 46.76 | 33.43 / 49.75, 35.68 / 53.20 |
| PSS KiB | 101,884, 102,311 | 107,933, 105,932 |

**Recommendation: the CPU raster.** Its whole per-frame cost, including presentation, is
6.3–6.6 ms at p95, inside the 8.3 ms budget of a 120 Hz frame. The hybrid path's is 10.6–10.9
ms, over it. The hybrid path's scene draw is no faster, it waits 5.5 ms for the GPU, and it costs
4–6 MiB more PSS. Two cautions apply. This host's CPU is faster than a phone's. The emulator's
GPU is the host's, translated, so the GPU wait is not a phone number. The CPU path's headroom
(1.7–2.0 ms at p95) has to be confirmed on the phone. The CPU worker's 3.3 ms beyond the draw
is now broken down by the presenter's profile log (buffer wait, lock, copy, apply), but the
device stopped before a run with that build (see [Still owed](#still-owed)).

## A6. Keyboard memory

`bitmap_atlas: true` was the default for every CPU raster: the keyboard chrome, key previews
(`Raster::shared`) and the settings CPU fallback. vello_cpu 0.4.0's atlas page is a fixed
4096×4096 RGBA `Pixmap`, and `Pixmap::new` fills it, so the first cached glyph makes the whole
page resident: one `[anon:scudo:secondary]` mapping of 65,540 KiB with RSS = PSS = 65,540 KiB.

`scripts/keyboard-memory.py` measures the IME process by dumpsys category at three points in
one process: settings open with the keyboard hidden, the Rust keyboard shown in the "Try out
your setup" field, and after 60 letter presses (no space, so nothing is committed or learned).

| PSS KiB | settings | shown | typed | native heap, shown |
| --- | ---: | ---: | ---: | ---: |
| Atlas for every run (before) | 85,987 | 204,744 | 204,887 | 122,780 |
| Atlas for colour fonts only (after) | 86,267 | 140,559 | 139,946 | 57,224 |

The page was the whole difference: −64,185 KiB shown, −64,941 KiB typed, and no
atlas-sized mapping after. Drawing outline glyphs directly does not slow typing:

| Keyboard frame draw, ms | frames | p50 | p95 | max |
| --- | ---: | ---: | ---: | ---: |
| Atlas | 114 | 0.558 | 4.158 | 4.577 |
| Direct | 115 | 0.564 | 0.760 | 1.56 |

The first two frames after showing fell from 20.5 and 40.0 ms to 14.4 and 27.8 ms, as the
page is no longer allocated and filled. Outline glyphs now always draw directly; colour fonts
(COLR, CBDT or sbix) keep the atlas, because without it their bitmaps are decoded every frame.
**Remaining:** drawing a colour emoji (the emoji panel, an emoji candidate) still creates the
64 MiB page. Fixing that needs a smaller atlas than vello_cpu 0.4.0's fixed constant, or a
colour-glyph cache of our own. `RasterProfile::atlas_runs` counts atlas runs, and
`profiling_preserves_pixels…` asserts outline text never uses it; making every run use the
atlas fails that test. Receipts: `/Volumes/2TB/aw-scratch/keyboard-memory-atlas` and
`keyboard-memory-direct`.

## A7. Native-load failure outside settings

`FlorisApplication.onCreate` returns early when `NativeRuntime.load()` fails, but the system
still binds FlorisImeService into that process. Forcing the failure on the device showed it
crash-looping: the service's `prefs` property built the preference model in the constructor,
and the model's defaults reach native code (`UnsatisfiedLinkError` →
`PreferenceModelNotFoundException`, `build/stage4-followup3/a7-before-ime/crash-stack.txt`).

The service now checks the engine once. Without it, it touches no manager: `prefs` is lazy,
every lifecycle override returns after calling super, and the input view is
`PlatformCrashScreen.inputView`, a notice with one button that opens the system keyboard
picker. The spell checker in the same process offers no suggestions instead of reaching the
engine. `SettingsImeFallbackDevice` forces the failure, binds our keyboard into the crash-mode
process, shows and hides it three times, checks the notice and its single button, checks the
process survived and the spell checker stayed inert, and restores the IME and its subtype keys.

The crash screen's summary is now a string resource (`crash__native_load_summary`;
`CrashReport.summary` is a `@StringRes`), and the screen draws with
`Theme.DeviceDefault.DayNight`: window background from `colorBackground`, text from
`textColorPrimary`, and light or dark system-bar icons to match. `SettingsCrashThemeDevice`
checks those against the platform's own attributes; the runner repeats the crash mode in dark
mode. Both passed, light and dark (`build/stage4-followup3/a7-after/rust-crash*.json`).

## Part B. Group 1

### Group plan

Every remaining Compose settings screen was sorted by the widget families it needs. A screen
joins a group only when everything it draws is already a Rust family, or one small addition
covers it.

| Group | Screens | Needs |
| --- | --- | --- |
| **1 (this follow-up)** | Sounds & Vibration (`settings/keyboard/input-feedback`), Smartbar, Gestures & Glide typing, Clipboard, Emojis (`settings/media`) | Toggles, list and slider dialogs and linked list-switch rows, which the keyboard screen already had. Added: a Yes/No confirmation (emoji history resets), a fixed disabled row (Smartbar auto-expand), vibrator-dependent rows and a preview vibration, and plural value labels |
| 2 | Typing (+ Dictionary and the forget-learned-words confirmation), Other (+ Physical keyboard), Dictation, About, Theme | An info card with a hyperlink (Typing), a confirmation with its own message that deletes learned words, the colour picker (`ColorPickerPreference`, in Other and Theme), the dictation model field (an `OutlinedTextField`, built on the editor family), an app-icon header, copy-to-clipboard with a toast, and launching external URLs (About) |
| Later | Project licence, third-party licences, Languages & Layouts, user dictionaries, theme manager and Snygg editor, extensions (import/export), backup & restore | Data-driven lists, per-item editors over user data, file pickers and progress, and the Snygg editor. Each is its own port |

All five group 1 candidates were confirmed against their Compose sources at `33f903a2`. Each
row is a JetPref `SwitchPreference`, `ListPreference` (some with `switchPrefKey`),
`DialogSliderPreference` or `Preference`, plus the two `JetPrefAlertDialog` resets in Emojis.
No other composables are drawn. Five texts in them were Kotlin literals; they are now string
resources Compose reads too (`01e8cefa`).

### What moved to Rust

Screens are data (`settings_ui/catalogue/*.rs`). A screen is a list of rows (heading, toggle,
list, slider, route, confirm, fixed), and each row has an enable condition and a visibility
condition over preferences and the vibrator facts. The keyboard screen moved onto the same
catalogue, so all six screens share one layout, widget, dialog and event path. Rust owns
navigation, and each back-stack entry has its own scroll; saved state v2 keeps the stack, the
scrolls and any open dialog draft. An invalid restore falls back to home. Rust also owns which
rows the vibrator facts enable, the preview vibration a haptic slider asks for when it is
released, the Yes/No reset confirmation, value labels (zero labels such as "Automatic",
resolved plurals, bare numbers), and the emoji skin-tone entries.

Kotlin only carries out what Rust asks for. It supplies strings for the keys Rust lists
(op 18) and plurals for the ranges Rust requests (op 16), through the explicit
`SettingsStringIds.kt` map (`getIdentifier` is a lint error here). It reports
`hasVibrator`/`hasAmplitudeControl` in the config, vibrates on `["vibrate", ms, strength]`,
and deletes emoji history through `EmojiHistoryHelper` on `["deleteEmojiHistory", which]`.
The Compose fallback receives Rust's native-route list (op 19), also from the ext and send
intents, and hands back any route Rust draws. Preference writes go through the same Jetpref
serializers as before.

### Proof

* **Rust:** `render/tests/settings_ui_screens.rs` (10 tests). Every key Rust shows is an
  Android resource and every resource key the code names is listed. It also covers native and
  fallback routes, the scroll per stack entry, the vibrator-dependent sliders and their
  preview, the Yes/No resets, the Smartbar fixed row, value labels and skin-tone entries.
* **JVM:** `ReferenceGroupOneScreens.kt` is test-only. It transcribes each screen's rows,
  conditions, choices, slider bounds and confirmations from the Compose sources at
  `33f903a2`. `SettingsGroupOneMatchTest` checks the visibility, enable state and role of
  every row against it, for every combination of the conditioning preferences under three
  vibrator states. It also runs 400 random actions per screen against Compose's own
  transactions, and every actionable row must be exercised. Further checks: plural ranges,
  native routes, and deep stacks with scrolls and dialog drafts surviving recreation.
  `SettingsResourcesTest` pins the string and plural maps to exactly the keys Rust lists.
* **Mutations:** `scripts/settings-ui-mutations.py` (now `scripts/settings_ui_mutation_cases/general.py`) carries seven group 1 mutations
  beside the four earlier ones. Each must fail a fresh assertion:
  * an enable condition (clipboard auto-clean minutes on history alone);
  * the order of two sync entries;
  * the duration slider's preview turned off;
  * the pinned reset deleting recent;
  * the shared-only fixed row shown for the extended layout too;
  * a dropped `action__yes` string id;
  * the Emojis route dropped from the native list.

  Both runs are recorded. The full run is `/Volumes/2TB/aw-scratch/ui-mutations-final`. The
  foreground rerun was split in two (`ui-mutations-rerun-a`, `-rerun-b`) so that neither could
  pass the tool time limit and leave a source mutated. In both runs all 11 mutations were caught
  with fresh assertion failures (route-dropped broke 7 assertions, the other group 1 cases 1
  each, the earlier four 1 to 4), and each
  restored run passed with the sources byte-identical.
* **Device:** the instrumented `ui` walk now opens each group 1 screen on both renderers.
  It exercises every enabled row from the top down: toggles twice, linked switches twice,
  every dialog's radios and sliders, and the resets with No, then Yes. It then switches the
  screen's masters and makes a second pass for the rows they enable:
  * vibration mode set to "Use vibrator directly";
  * the Smartbar layout set to shared;
  * glide typing flipped;
  * the clipboard and emoji masters turned on.
  Each screen, its end, its expanded state and each dialog are audited. The host script
  requires identical preference bytes, the same rows exercised in the same order, and the
  same never-enabled rows on both paths (`group-one.json`). The gallery adds every group 1
  screen (top and lower pages) and all 39 dialogs to the strict diff.

### Device results

**Partial; owed.** Only the Compose path has run the group 1 walk (A3, second attempt, run 3).
It passed Sounds & Vibration, Smartbar and Gestures & Glide typing, both passes each,
including the check that vibration duration and strength are enabled exactly as the
emulator's vibrator facts allow. It stopped in Clipboard on a walk bug, since fixed. Neither
path has a saved report: the Rust run, the preference-byte and walk comparison, the audits,
the gallery (top and lower pages of every group 1 screen, all 39 dialogs, light, dark and
font 2) and its strict diff have not run. The transport dropped first.

## Still owed

Each item names what would close it.

* **Device restore (first).** The AVD's app prefs are in a fixture state (A3). Boot
  `aw_keyboard_ssd`, restore with `SettingsDevice.cleanup` and run
  `settings-device-baseline.py --verify`.
* **A2. Final-source pictures.** A fresh capture of the 51 keyboard pairs and the group 1
  gallery, with the strict diff. The last full capture predates the font-2 row rules and the
  slow-scroll helper.
* **A3. The whole instrumented suite** on the final source: ui on both renderers with
  preference-byte parity, text, death and crash.
* **A4. The 23 Jetpref nodes.** They are named and owed by design, but the classification
  has not run on a device yet. Fixing them needs a Jetpref fork.
* **A5. Phone numbers.** The presenter breakdown (buffer wait, lock, copy, apply) on a device
  run, and the CPU raster's 1.7–2.0 ms headroom at 120 Hz confirmed on the phone.
* **A6. Colour emoji.** Drawing one still makes vello_cpu 0.4.0's 64 MiB atlas page resident.
  It needs a smaller atlas or our own colour-glyph cache.
* **TalkBack.** A real TalkBack session on the phone. The suite drives accessibility actions
  through UiAutomation, which is not TalkBack.
* **Part B on a device.** The group 1 walk on both renderers with the walk and preference
  comparison, the group 1 gallery and its strict diff.
* **The fixed row's switch.** The Smartbar auto-expand row draws a checked, disabled switch
  with no toggle semantics, as Compose does. Its painted geometry has been checked only by
  the JVM tests, not against a Compose picture.
