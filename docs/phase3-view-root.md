# Phase 3, item 4: a plain-View root

Phase 3 of [the all-Rust plan](all-rust.md#3-rust-owns-the-keyboard-l) hands the keyboard to Rust
one piece at a time. Item 4 replaces the Compose root with plain Views while the Rust renderer is
on.

**Before this work** the input method window was always a composition: `ComposeInputView` set
the theme and settings as composition locals, `ImeUiWrapper` and `TextInputLayout` sized the
keyboard from the window's insets and the height settings, `SnyggSurfaceView` drew the window's
background and image, `RustKeyboardLayout` hosted the Rust renderer's `SurfaceView`, and Compose
also hosted the one-handed panels, the subtype sheet, the full-screen extract view and the inline
autofill chips. Package 8 kept that root because every one of those reached Compose
([decisions](decisions.md#a-plain-view-root-for-the-rust-renderer-the-compose-root-only-as-the-fallback-2026-11-23)).

**Now**, with the Rust renderer on, `onCreateInputView` returns `ImeRootView`, a `FrameLayout`,
and the window holds no composition. **Rust decides** where everything goes and what the window
draws: `render::ime_root` plans the window (`ImeWindowRules.layout`), and `imetheme::window`
answers the window's background (`ImeThemeRule.WINDOW`). Kotlin measures, reads the settings
and the flows, decodes the image Rust names, and places the views where the plan says.

With the renderer off, the frozen Compose root runs exactly as before; it is the fallback until
[step K](compose-removal.md#k-the-compose-keyboard). Switching the setting while the keyboard
shows swaps the root in the same process.

Branch `all-rust/view-root`, on main 0500aa74f (main had not moved at the final gate). Its commits:

| Commit | What |
|---|---|
| e407edcaf | Rust plans the input view's window (`render::ime_root`) |
| 3ff0f0b68 | Rust answers the window's background (`imetheme::window`) |
| b3112332a | The plain-View root, its sheet, panels, extract view and inline chips |
| 59881cc75 | Seeded differential, composition check and runtime swap scripts |
| cc8c0d6dd | The ledger's rows for the root's files |
| 92f8aef0d | Mutations of the window plan and background |
| e4a6e73da | View constructors lint accepts, and KTX's `createBitmap` |
| 6845f7884 | Tests that catch every window-plan and background mutation |
| fe8dad380 | The differential waits for the bars to settle; the input view is timed |
| 4832dda00 | A root or sheet swapped in on an existing window asks for insets |
| 6469d68e5 | Device checks, and the swap check that a swapped-in root sits where a fresh one does |
| (last) | This record, the decision, and the owed checks |

## What moved to Rust

### The window plan (`lib/rust/crates/render/src/ime_root.rs`)

`layout(facts)` takes what Android measured and what the settings say (the wire's
`ImeWindowFacts`: the measure spec, the safe drawing insets, the bars, the orientation, the
height and bottom-offset settings, one-handed mode, the keyboard's mode and rows, which panel is
up, then the Smartbar's config) and answers `ImeWindowPlan`, in pixels:

* **The input view's height**, `FlorisImeSizing`'s arithmetic: the base height scaled between the
  resources' smallest and largest fractions of the display, the height setting, and rows, never
  fewer than four, with Compose's rounding (half up) and its weighted split of a remainder.
* **Which panel shows**: the actions overflow over the editor, then the media panels by UI mode,
  GIF results over emoji, and the search bar under GIF search, whose Smartbar config it rewrites.
* **The row**: the safe insets on each side, the bottom offset (with the safe bottom), and the
  one-handed panels, on the side away from the keyboard, in portrait only, as tall as the keys.
* **The column**: the view's height for its measure spec, the window box at its bottom, or at
  the top under full-screen's extract view, where no spacer goes above it.
* **The surface**: the keyboard's box and its surface, which reaches `popup_scene::headroom`
  above the keys for popups; the inline chips' row; the row and Smartbar heights
  `FlorisImeSizing` keeps for the rest of the app.
* **`RISE`**: under a full-screen extract view, the keyboard's surface rises above the window
  while a finger is down or a popup is open (below).

### The window's background (`lib/rust/crates/imetheme/src/window.rs`)

`ImeThemeRule.WINDOW` answers what `SnyggSurfaceView` drew under the keyboard: the `window`
rule's background colour (inside `root`, as Compose nested them), its `background-image` by the
asset name the theme gives (`flex:/…`, percent-decoded, query dropped, never above the theme's
directory), where the image goes for the surface's size and its own (`content-scale` and
alignment), and the navigation bar's light buttons and enforced contrast. The inline chips'
colours and the window share one walk of the stylesheet (`styled`). Kotlin decodes the file Rust
names, inside the unpacked theme's directory, and paints the rectangle Rust answers.

### Wire and boundary

`crates/wire/src/ime_window.rs` declares the facts and the plan; `ime_theme.rs` gains the window
row. `ImeWindowRules.layout(facts: FloatArray): FloatArray` is generated by `crates/boundary`;
no `external fun` is hand-written. The checker counts 466 production natives (main has 465),
none hand-written.

## What the root hosts

The root is Android-forced hosting: views, measure specs, insets, the window's flags, surfaces
and the settings' flows. It decides nothing the plan or the theme answers.

| File | Counted lines | What |
|---|---:|---|
| `AW/render/ImeRootView.kt` | 219 | The input view: the facts it measures, the plan's rectangles, the snapshot the keyboard draws, rebuilds once a frame, the surface's rise |
| `AW/render/ImeRootReads.kt` | 131 | What the adapters read where Compose's read its locals: managers, settings, strings in the keyboard's language, the theme handed to the renderer; `RootWatch`, the flows that trigger a rebuild |
| `AW/render/ImeRootBackdrop.kt` | 155 | `ImeBackdrop`, the window's background and image on a surface of its own, decoded with `ImageDecoder` |
| `AW/render/ImeRootSheet.kt` | 60 | `ImeSubtypeSheet`, the subtype sheet over the whole window, and `safeDrawing`, Compose's safe drawing insets |
| `AW/render/ImeRootInline.kt` | 49 | `ImeInlineChips`, Android's inline autofill chips in a scrolling row |
| `AW/render/ImeRootExtract.kt` | 49 | `ImeExtractView`, the full-screen editor: Android's `ExtractEditText` inside a Rust chrome view |

`FlorisImeService` picks the root, the sheet and the extract view by the setting, swaps them when
it changes, and tells the root when the window's mode, its extract view or its language changes.

### Lifecycle

* **Create**: `onCreateInputView` makes the sheet (on the window's content) and the root. The
  root makes a fresh `RustKeyboardView` each time it is attached, starts watching its flows and
  builds a snapshot before the first layout, so the first frame draws keys.
* **Show, hide, show again**: the root passes the window's visibility on as the Compose root did
  (`InputViewVisibility`), including a show that comes before the window really hid.
* **Configuration change and rotation**: Android recreates the input view; the root re-reads the
  status bar and the insets on the next measure.
* **Detach**: the root stops its flows and releases the keyboard's surface.
* **Process death**: the next show creates the service and the root afresh.
* **Runtime swap**: a collector on the setting calls `setInputView(onCreateInputView())` and
  replaces the extract view; the old sheet is removed from the window's content first.

### Insets and the full-screen editor

The root reads the window's own insets (`rootWindowInsets`), as Compose's safe drawing did:
in full-screen mode an ancestor consumes the navigation bar's before they reach the view. The
status bar's height comes from the window manager, since the IME's insets have none.

Android dispatches insets to a view at the window's first traversal or when they change. A root
swapped in on a window that already exists gets neither, so the root and the sheet read
`rootWindowInsets` when attached and call `requestApplyInsets()`. Without that, a swap from the
Compose root back to the plain root left the safe area's bottom at 0 and drew the keyboard a
navigation bar (126 px) too low, under the IME's own navigation buttons. Compose's root asked
for insets on attach itself, so main never showed it. The "teh" probe found it: it swaps the
renderer while the service lives and then taps fixed points, and one landed in the strip.

**The owed full-screen clipping is fixed.** In full-screen landscape the extract view sits
directly above the keyboard. The keyboard's surface sits behind the window (media overlay), and
the window's opaque extract view covered whatever the surface drew above the keys, so a top-row
key's popup and press bubble were cut at the keyboard's top. Under that layout Rust's plan sets
`RISE`, and `RustKeyboardView.rise` moves the surface above the window while a finger is down or
a popup is open, then back. Only the layer moves (`setZOrderOnTop` /
`setZOrderMediaOverlay`): the surface is not recreated, and no `surfaceChanged` fires. Below the
window again, TalkBack's focus box (drawn by the window) shows over the keys.

## The proof

### Gate

The full gate on 6469d68e5, rebased on main 0500aa74f (`cargo fmt`, then `./gradlew test
:app:lintDebug :lib:rust:check :app:checkNativeDeclarations :app:checkJvmLedger assembleDebug`
and the instrumented sources' compile) passed in 11 min 41 s:

| Check | Main | Branch |
|---|---:|---:|
| JVM tests (files) | 1,044 (258) | 1,044 (258), 0 failed, 0 skipped |
| Cargo results | 1,582 | 1,605, 0 failed, 0 ignored |
| Native declarations | 465 | 466, 0 hand-written, 0 missing |
| Ledger rows | 406 | 408, matching 408 production files |
| Lint | 10 allowed | 10 allowed: AndroidGradlePluginVersion 4, NewerVersionAvailable 4, ChromeOsAbiSupport 1, GradleDependency 1 |

The one new native is `ImeWindowRules.layout`; the window's background is a new rule
(`ImeThemeRule.WINDOW`) on an existing call. The ledger gains the root's six files and loses
the four it replaces (`RustKeyboardLayout`, `RustOneHandedPanel`, `RustSubtypeSheet`,
`NativeExtractContent`).

### Mutations

`scripts/view-root-mutations.py` puts 34 faults in the window plan, the window's background and
its row across JNI, each against the plan's tests, the theme's, or the search bar's JVM transport
test. **All 34 are caught** in a fresh report (`build/view-root/mutations-final2/`), and the
restored source passes all three targets.

The first full run caught 28 of 35. Five were test gaps, now covered: the base height winning
over the display's fractions, fewer than four rows, a one-handed panel taller than the keys (a
GIF search with extended actions), a `window` rule inheriting `root`'s background, and the light
bars and contrast flags across the row. One case did not compile (an untyped `0.0`), and one was
an equivalent mutant: `one_handed` already gives a side no weight unless the mode is on, in
portrait and on that side, so the plan now reads the weights alone and that case is gone.

### No composition in the Rust path

`scripts/render-benchmark/view-root-composition.py` dumps the visible windows' view hierarchies
(`cmd window dump-visible-window-views`) and reads only class names from the InputMethod
window's. `view-root-diff.py swap` switches the renderer while the service lives: the Rust root,
the Compose fallback, then the Rust root again, each checked and screenshotted. Each step must
also sit where a fresh process puts the keyboard: the Rust steps' window, surface and key bounds
equal the first step's, and the Compose step's window does.

That placement check is new. Before the insets fix above, on the build the CPU trials measured
(`build/view-root/swap4-before/`), it failed the third step: the keys' union was
`[0, 1801, 1080, 2400]` against a fresh process's `[0, 1675, 1080, 2274]`.

On the final build (`build/view-root/swap5/`, APK `abc67621`), in one keyboard process:

| Step | Classes in the IME window | Compose classes | `ImeRootView` | Placed as a fresh process | Expected |
|---|---:|---:|---|---|---|
| Rust renderer on | 21 | 0 | yes | yes (the reference) | pass |
| Switched off: the Compose fallback | 28 | 4 | no | yes (its window) | pass |
| Switched on again | 26 | 0 | yes | yes: keys `[0, 1675, 1080, 2274]` | pass |

The five classes the second Rust step has over the first are the full-screen editor
(`ImeExtractView`, `RustExtractView`, `RustChromeView`, Android's `ExtractEditLayout` and
`ExtractEditText`): the swap installs the extract view at once, where a fresh start makes it
when the window first asks; outside full-screen it is not shown. The device checks below run
the same check in every pose, and every one found the plain root and no Compose class.

### Seeded differential against main

`scripts/render-benchmark/view-root-diff.py capture` poses the device in 18 window
configurations (six fixed, twelve seeded: orientation, gesture or 3-button navigation, a display
cutout, height 80/100/125, bottom offset 0/12, one-handed off/start/end, font scale 1.0/1.3/2.0,
full-screen on or off) and records the IME window's touchable region and content insets, and the
keyboard's surface, keys and chrome nodes by bounds. `compare` diffs main's capture and the
branch's, both on `emulator-5558` with the same settings.

Main (0500aa74f) was captured first, then the branch (fe8dad380), both on emulator-5558 with
the same settings, each restored byte for byte after; after the insets fix the final build
(`abc67621`) was captured again (`branch8`) and compared with `main7`. Before comparing, the input view's
own container nodes are taken out and counted: main's Compose host has 67 across the 18
configurations, the plain root 9. The differential ran three times (`build/view-root/diff/main6`
against `branch6`, `main7` against `branch7`, and `main7` against the final build's `branch8`).
Until the second run the script looked for the window's frame under a name this Android does
not print, so only the second and third compare it.

* **Identical, 15, 14 and 14 of 18**: the window's touchable region and insets, the keyboard's
  surface, its 34 keys and every chrome node, to the pixel; from the second run the input
  method window's frame too.
* **2 intended** (the two full-screen 3-button landscape configurations), in every run: the
  branch exposes the keyboard's `SurfaceView` node where main's Compose host exposed none. The
  window, the touchable region and every key are identical.
* **Flaky, 1, 2 and 2, the same on both builds (E53).** The keyboard's top moves by a pixel or two
  between runs, on main and on the branch alike:
  * `fd44adddbc` (portrait, 3-button navigation, font scale 2.0, height 80, bottom offset 12),
    different in both runs: the top is at 1682 or 1684. Main gave 1684 (`main4`, `main7`) and
    1682 (`main6`); the branch 1682 (`branch-seq1`, `branch7`) and 1684 (`branch-seq2`,
    `branch6`).
  * The baseline (`9b82474e24`: portrait, gesture navigation, nothing else set), different in the
    second run only: the top is at 1561 or 1562. Main gave 1562 (`main6`, `main7`) and 1561 (the
    image-theme screenshots below); the branch 1562 (`branch6`, the image-theme screenshots) and
    1561 (`branch7`). The window's frame is the same in both. Identical in the third run.
  * `7a0cde4a3f` (portrait, a cutout, font scale 1.3, height 125, bottom offset 12, one-handed
    start), different in the third run only: the window's frame starts at 132 instead of 126
    (the status bar's height before the cutout's overlay settled), the content top is 1335
    instead of 1340 and the keys start at 1590 instead of 1589. Main gave exactly these values
    once (`main4`) and the others three times (`main5`, `main6`, `main7`); the branch the others
    twice (`branch6`, `branch7`) and these once (`branch8`).

  Main also differs from itself by 1 px in two other configurations (`main4` against `main5`).
  Both roots measure the screen and its bars while SystemUI is still resizing them after the
  previous configuration's change; `settle()` waits for the bars' frames to hold still, which
  removed some 1 px cases but not these.

### Digests and probes

All on emulator-5558 with the Rust renderer, settings restored after each.

* **300 presses and 8 popups: unchanged.** Every kept CPU trial typed the expected text, typing
  `809d588b…` (300 UTF-16 units) and, after the popups, `c14d19d2…` (308), on main and the
  branch alike: main's three, the branch's three (fe8dad380), one on the insets fix
  (`af2e1e34`) and one on the final build (`abc67621`). A thrown-out trial typed one key as its
  long-press hint (E29), which `explain-digest.py` names; its popup digest follows from the same
  wrong text. See [the CPU trials](#cpu-trials).
* **"teh" and a space types "the ".** `suggestions.py` replays it, and a candidate pick, on the
  Compose renderer and then the Rust one in the same process, with the learned-state files moved
  aside and put back (E39): both renderers matched both digests on the final build. On
  fe8dad380 it failed: Compose passed, then the Rust root, swapped in, sat a navigation bar too
  low, and the fixed taps typed 9 units where the fixture has fewer. That was the insets bug
  above; the fix's build passed, and so did the final one.
* **Reshow: 16 of 16** trials drew their keys after a quick hide and show, on fe8dad380, the
  insets fix and the final build.
* **Reading order passes.** `a11y-order.py`'s five label-free captures (letters 57 nodes,
  expanded 63, collapsed 57, words 59, a popup 69) read in screen order under
  `compare-a11y.py --order`, with no issue, on fe8dad380 and the final build.

### Measurements

`scripts/render-benchmark/view-root-timing.py` takes stage 3b package 8's samples: 10 cold
creations, each in a new keyboard process, and 10 warm ones, each a recreation by rotation in
the same process. Times are medians in ms from the factory's start (`onCreateInputView`) to its
return, to the first non-empty layout, and to the first frame the renderer posts; PSS is the
process's total with the keyboard shown. Main and the branch (fe8dad380) alternated on
emulator-5558, twice each; the insets fix (`af2e1e34`) and the final build (`abc67621`) were
timed once each after (`build/view-root/timing/`).

| Build | Cold: created | layout | first frame | PSS (MiB) | Warm: created | layout | first frame |
|---|---:|---:|---:|---:|---:|---:|---:|
| main 1 | 436.0 | 1,797 | 2,075 | 139.8 | 59.0 | 276 | 372 |
| branch 1 | 157.4 | 937 | 1,501 | 134.8 | 3.8 | 187 | 336 |
| main 2 | 501.3 | 1,808 | 2,093 | 139.2 | 61.2 | 297 | 382 |
| branch 2 | 178.5 | 1,026 | 1,633 | 136.0 | 2.2 | 132 | 252 |
| insets fix | 197.1 | 1,014 | 1,581 | 131.4 | 3.9 | 165 | 322 |
| final | 164.8 | 961 | 1,543 | 134.1 | 5.0 | 149 | 292 |

* **Creating the input view** takes 157 to 197 ms cold, against main's 436 to 501, and 2 to 5 ms
  warm, against 59 to 61: no composition is set up.
* **The first frame** comes 1.5 to 1.6 s after the factory starts cold, against 2.1 s, and 252 to
  336 ms warm, against 372 to 382.
* **Memory**: 131.4 to 136.0 MiB, against 139.2 to 139.8, about 4 MiB less.
* The insets fix asks for one more insets dispatch per attach; its build and the final one sit
  inside the branch's earlier spread.

### CPU trials

Three trials of each build, alternating, with `cpu-trials.py --rust-only --trials 1`, then one
each of the insets fix and the final build; the tables and the thrown-out trials are in
[cpu-p95-results](cpu-p95-results.md#the-plain-view-root-2026-11-23).

* **The branch is faster beyond noise.** Its wall-clock p95 is 1.992 to 2.374 ms, against main's
  2.959 to 3.522; frames over 2 ms are 5.1 to 7.7% of its frames, against 12.3 to 13.8%; its CPU
  time p95 is 1.316 to 1.378 ms, against 1.467 to 1.541.
* It draws about 120 fewer frames for the same typing (943 to 945, against 1,018 to 1,071), all
  of them frames that only changed the strip's words, and its press and release frames alone
  are faster at p95 too (2.145 to 2.492 ms, against 2.807 to 3.231).
* **After the insets fix**, one trial of its build and one of the final build: p95 2.330 and
  2.124 ms, 7.5% and 6.2% of frames over 2 ms, CPU time p95 1.343 and 1.389, both within the
  branch's spread, with the expected digests.
* **The 2 ms gate is still not met.** Main misses it in every trial; the branch meets it in one
  of five (1.992 ms) and misses it in four, on a host with load averages of 3 to 12. That is not
  a pass. The gate stays owed on a quiet host (E28) and the phone (P4).

### Device checks (emulator-5558)

`scripts/render-benchmark/view-root-device.py` poses the device on the installed build, checks
the IME window holds the plain root and no Compose class, and screenshots it into
`build/view-root/device/`. Every touch goes through agent-device. It keeps only class counts,
pixel tallies and lengths: the probe's field stays empty with suggestions off, and the dictation
screenshots are greyed above the keys before anything looks at them. Settings were restored byte
for byte, and rotation, navigation mode, cutout, font scale and accessibility put back, after
every run.

| Check | Classes | Compose | What else it showed |
|---|---:|---:|---|
| Portrait / landscape | 21 / 21 | 0 | |
| 3-button navigation, portrait / landscape | 13 / 14 | 0 | Gesture navigation adds the system's IME navigation views |
| One-handed, start / end | 22 / 22 | 0 | The panel's view on the side away from the keyboard |
| Font scale 1.3 / 2.0 | 21 / 21 | 0 | |
| Full-screen landscape | 26 | 0 | The extract view and the keyboard, one window |
| A top-row key's popup in full-screen (`e`, held) | 26 | 0 | The popup whole above the Smartbar, over the extract view |
| A top-row key held in full-screen (`p`) | 26 | 0 | Its popup (`0`) whole, likewise; the press bubble alone was seen whole before (`build/view-root/screens/fullscreen-bubble-after.png`) |
| Subtype sheet (Space held) | 22 | 0 | The sheet and its scrim change rows 52 to 2399 against portrait |
| Background-image theme, day / night | 21 / 21 | 0 | 131,037 / 131,123 pixels of that image's stripes, none of the other's; main draws the same counts |
| Rotation and back | 21 / 21 | 0 | Shown both times, the same process |
| Process death (`kill -9`) | 21 | 0 | A new process shows the plain root |
| Dictation preview (the fixture WAV) | 21 | 0 | 103 strip rows changed during the preview; 68 UTF-16 units typed |
| GIF panel (fixture build) | 21 | 0 | The media panel's GIF tab: the search bar, the chips and the fixture's GIFs, drawn by Rust on the plain root; built from fe8dad380 with `-PgifFixture=true` |
| Subtype sheet after a swap | 26 | 0 | Renderer off and on in one process, then Space held: 0 pixels differ from the fresh sheet's below the window's top. The build before the insets fix differed in 734,227 |
| Runtime swap, both ways | see above | | The same process throughout; each root placed as a fresh process places it (after the insets fix) |
| Quick hide and reshow | | | The reshow probe, 16 of 16 (below) |

* **The image theme** is a test extension (`build/view-root/imgtheme/`), two copies of the
  default stylesheets whose `window` rule has a striped `background-image`, put in the app's
  theme directory for the check and taken out after. The image draws under the navigation bar,
  where the keyboard's own background does not cover it. Main's screenshots differ from the
  branch's in three places only: the keys and Smartbar sit 1 px apart (E53, main's keyboard the
  taller this time), 378 pixels on the image squares' edges under the navigation bar (the crop
  scaled for a box 1 px apart), and the status bar's clock and icons. The extension's zip must
  carry a `stylesheets/` directory entry: `ZipUtils.unzip` creates no parent directories, so
  without one the theme fails to load and falls back to the base style without a log line, on
  main too (owed, in [owed-checks](owed-checks.md)).
* **The GIF panel**'s fixture source lives in the keyboard's process only, and posing the device
  restarts that process, so the check switches the fixture in after posing. Its first run, which
  switched it in before, found no GIF tab: `GifSearch.available` was false in the new process.
* **Dictation** replays `files/debug-mic.wav`; the check refuses to start without it, since the
  debug build would otherwise open the microphone, and the emulator was muted. While the level
  meter moves, the accessibility dump service answers with an error instead of nodes, so the
  check did not press stop and let the recording's silence end it.

Screenshots: `build/view-root/device/` (the branch), `build/view-root/device-main/` (main's
image-theme pair), `build/view-root/device-gif/`, `build/view-root/swap3/`, and for the insets
fix `build/view-root/swap4-before/`, `build/view-root/device-sheet-before/` (the build before
it), `build/view-root/swap5/` and `build/view-root/device-final/` (portrait, the sheet fresh
and swapped, and the image theme day and night, on the final build).

## Intended differences

* **Full-screen landscape: a top-row key's popup and press bubble are whole.** Main's Compose
  root cut them at the keyboard's top. This is the owed fix, not a regression.
* **The accessibility tree in full-screen landscape** shows the keyboard's `SurfaceView` node; main's
  Compose host showed none (2 of the differential's 18 configurations). Keys, their order and
  their bounds are the same.
* **Container nodes.** Compose's root contributes unlabelled `View` nodes to the tree (67 across the
  18 configurations); the plain root has 9. None has a label or text.

Nothing else differs in the differential, the digests or the probes.

## The size

Production Kotlin, counted as the JVM ledger counts (`scripts/jvm-ledger.py`), in the files
this item touched: **2,075 on main, 2,450 here, 375 more.** The plain root is new while the
Compose root stays as the fallback, so both are in the build until step K.

| File | Main | Here |
|---|---:|---:|
| `AW/render/ImeRootView.kt` (new) | — | 221 |
| `AW/render/ImeRootBackdrop.kt` (new) | — | 155 |
| `AW/render/ImeRootReads.kt` (new) | — | 131 |
| `AW/render/ImeRootSheet.kt` (new) | — | 61 |
| `AW/render/ImeRootExtract.kt` (new) | — | 49 |
| `AW/render/ImeRootInline.kt` (new) | — | 49 |
| `AW/render/RustKeyboardLayout.kt` (deleted) | 89 | — |
| `AW/render/RustOneHandedPanel.kt` (deleted) | 37 | — |
| `AW/render/RustSubtypeSheet.kt` (deleted) | 30 | — |
| `FB/NativeExtractContent.kt` (deleted) | 25 | — |
| `FB/FlorisImeService.kt` | 416 | 434 |
| `AW/render/RustKeyboardView.kt` | 404 | 411 |
| `FB/FlorisImeViews.kt` | 226 | 213 |
| `FB/FlorisImeUi.kt` | 141 | 134 |
| `AW/render/RendererSmartbar.kt` | 150 | 145 |
| `FB/ime/smartbar/quickaction/QuickAction.kt` | 117 | 119 |
| `AW/render/RendererTheme.kt` | 73 | 40 |
| `FB/ime/text/TextInputLayout.kt` | 77 | 67 |
| `AW/render/RendererEmoji.kt` | 67 | 65 |
| `AW/render/RendererPanels.kt` | 64 | 47 |
| `AW/render/RendererClipboard.kt` | 58 | 45 |
| `AW/render/RendererGif.kt` | 41 | 35 |
| `AW/render/RendererSearch.kt` | 21 | 12 |
| `AW/render/RendererSubtypes.kt` | 21 | 12 |
| `AW/render/RendererDecorations.kt` | 18 | 5 |

The adapters shrank because their `@Composable` readers went: they read `ImeRootReads` now.
`RendererSearchTransportTest` (test) grew from 29 to 42 for the search bar's plan across JNI.

The ledger has 408 rows (main 406): six new, four deleted. Three files were raised, each with its
reason in the ledger: `FlorisImeService` +18 (it picks and swaps the roots), `RustKeyboardView`
+7 (`rise`), `QuickAction` +2 (two string ids pulled out of composables). Two new files were
raised after their first rows, for the swap's insets fix: `ImeRootView` 219 to 221 and
`ImeRootSheet` 60 to 61.

**Against the end-state budget** ("IME service, plain-View root, surface, touch, insets", about
600 lines) this lands well over it: the root's six files are 666 lines, before `FlorisImeService`
(434) and `RustKeyboardView` (411). Step K takes the Compose root and the service's branches
for it; the rest has to come from moving decisions, not hosting: `ImeRootReads` is mostly
settings and strings that move with phase 4, and `ImeBackdrop`'s second surface would go if the
renderer drew the window's background and image itself.

## What remains

* **Icons are still Compose's.** The panels, the one-handed panel, the subtype sheet, the search
  bar and the Smartbar take Material icons as `ImageVector`s and flatten them with Compose's
  `PathParser` (`RendererIcons.iconPath`); the incognito watermark loads its drawable through
  `vectorResource`. No composition is made, but `androidx.compose.ui` and
  `material-icons` are still needed. The brief asked for icon tokens mapped to drawables; that was
  not done here. Step K needs it.
* **Other Compose types in the Rust path, none of them a composition:** `IntSize` and
  `LayoutDirection` (`ImeRootView`), `mutableStateOf` (`FlorisImeService`), `Color` and `toArgb`
  (`RendererTheme`, `ImeRootReads`), and the palette (`SnyggPalette`, made by materialkolor).
* **Theme fonts.** A theme's own `font-family`, unpacked from its extension, is not drawn by the
  Rust renderer, here or on main. Font scale and the size multiplier are.
* **Live inline autofill** is hosted (`ImeInlineChips` takes Android's views into the row Rust
  plans) but was not seen with a real autofill service: the emulator has none offering inline
  suggestions (E52 and P35).
* `RustKeyPreview.kt` has no caller, here or on main.
* What step K now needs is in [compose-removal step K](compose-removal.md#k-the-compose-keyboard).
