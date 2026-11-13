# Stage 3B package 6: native emoji and action panels

The default-off Rust renderer now retains its surface for emoji, editing commands,
quick-action overflow and the action-order editor. Clipboard history and GIF/search remain
Compose. The Compose implementation remains available when the prototype is off.

## Source specification and ownership

The source specifications are:

- `ime/media/emoji/EmojiPaletteView.kt`, `EmojiSet.kt`, `EmojiHistory.kt` and
  `ime/media/MediaInputLayout.kt`.
- `ime/smartbar/quickaction/QuickActionsOverflowPanel.kt`, `QuickActionsEditorPanel.kt`,
  `QuickActionButton.kt` and `QuickActionArrangement.kt`.
- `ime/keyboard/KeyboardEditing.kt`, `KeyboardKeyEvents.kt` and
  `ime/input/InputEventDispatcher.kt` for editing actions and hold clocks.

This checkout has no standalone editing panel in `ime/editor` or `ime/text/editing`.
Compose exposes its editing controls as quick-action tiles. Rust exposes those same tiles,
including native repeat clocks for arrows, delete/forward delete and undo/redo. The shared
manual-selection toggle/collapse and arrow-endpoint policy also runs in Rust; the default
arrangement is preserved and does not acquire an invented selection-toggle tile.

Compose has no emoji search. Its GIF search field belongs to the separate GIF panel;
that panel and its text-entry route remain Compose for the next package.

The `render` crate owns adaptive cells, clipping, scroll offsets, category selection,
page gestures, hold/release/cancellation, popup choices, drag markers, placement and saved
arrangement, semantic targets, and painting. Scroll decay uses the Android/Compose spline
and the existing native LSQ2 velocity tracker. Android supplies supported emoji data,
preferences, localized resources, evaluator flags, system font paths and icons. Kotlin
executes the existing emoji, editor and `QuickAction` handlers and stores the unchanged
serialized preference types. No panel geometry or drag-order policy lives in Kotlin.

History ordering, automatic/manual prepend/append, pin/unpin, limits, movement and clearing
run in `emoji::history`. Arrangement distinctness, sticky/dynamic/hidden drag transitions,
placeholder cleanup and saved order run in `render::arrangement`. The same Rust policy is
used by the Compose fallback's storage helpers. A frozen recents page retains stable ids
until category entry or a history-menu action refreshes it. Category grids retain separate
scroll offsets. The skin-tone setting selects the base and excludes matching variants from
the long-press popup, as Compose does. Colour glyphs use the existing system-font
COLRv1/CBDT raster path.

The action-order editor is a naturally sized bottom sheet drawn inside the expanded,
transparent native surface. Its header, section counts, sticky/dynamic/hidden groups,
drag ghost, empty placeholders, back and scrim dismissal use the same native state.
Both pointer and accessibility actions reach that state. Labels match the Compose text;
Rust additionally names icon-only category, scroll, dismissal and history controls.

## Differential tests and deliberate mutations

The test-only Kotlin references remain under `lib/rust/src/test/.../rust/`:

- `ReferencePanelRules` / `PanelRulesMatchTest`: 76,800 seeded history steps across all
  16 strategy pairs and four limits per list, including disabled storage, pin/unpin,
  remove/clear and extreme movement offsets. Arrangement checks cover 2,000 distinctness
  cases with 30 drag schedules each, three groups, placeholders and 64 reorder endpoints.
- `PanelPresentationMatchTest`: 300,030 key classifications across hold/repeat/delete
  configurations, 108 adaptive grids, 12,000 preferred-tone/variant cases, and 38,016
  overflow-count/history-menu combinations including zero visible actions and the original
  history-menu boolean precedence.
- `PanelSelectionMatchTest`: 78,408 select/arrow plans, including invalid and reversed
  ranges and every manual-selection endpoint flag combination.
- `PanelScrollMatchTest`: 5,562 spline duration/distance/position comparisons across six
  densities, signed/zero velocities and times outside/inside the animation interval.
  Constants and float/double boundaries were checked against the installed Compose AAR.

Each policy family was deliberately broken and restored: history prepend direction;
drag-marker insertion position; editing hold multiplier; adaptive-grid remainder direction;
preferred-tone predicate; manual-selection branch; spline friction; overflow maximum count; and history-menu
boolean precedence. The corresponding
JVM match test exited nonzero for each mutation. Additional native tests exercise repeat
release, cancellation, scroll/fling, margin reservation, disabled/pressed style inheritance,
semantic actions, popup dismissal, frozen/locked history and category scroll restoration.

## Emulator evidence

Evidence is kept locally under ignored `build/stage3b6/`; no images, dumps, APKs or build
outputs are committed. All device work uses the dedicated `aw_keyboard` runtime
`emulator-5556`. `emulator-5554` was not used or closed. Installation uses only
`scripts/install-dev.sh`.

The panel fixture records public target geometry with an empty editor, then disables the
accessibility observer during typed editing input. Querying the IME while the disposable
editor's synchronous InputConnection is serving a selection read can perturb that read;
that observer effect and a disabled-target ancestor error were diagnosed separately from
the product. Full tree capture still rejects a nonempty editor. The probe's tree queries
run on a worker, and the typed fixture stores only digests, lengths and cursor ranges.
Agent-device remains idle during adb input and tree capture; it takes the requested PNGs.

The final APK is recorded in `final-apk.json`; its SHA-256 is
`0bd77483c1896649d01659f60fed1545ad7809c11eb5883529b3b56f9dfba840`.
`final-compose-fresh/` and `final-rust/` match all four cursor checkpoints and all fixture
results, including the exact serialized arrangement:

| Fixture | SHA-256 |
| --- | --- |
| Five emoji, 12 UTF-16 units | `82f7506f5da3d363b65d4aa433066711e57445a62ded9daec27f57a745605a76` |
| Editing, 8 UTF-16 units | `3bc49b73e2fb201924d9dcce5fb6d6fd7cfbf58c49be8cc46439c05dc634b151` |
| Serialized swapped action order | `2e2de6c88b7bdea973948efde7e65cda5293544fca307037acc02758bdc5d2fc` |

The recents check passes with all five entries in the same order on both renderers.
Arrow-left gives cursor 3, select-all gives 0..4, Copy collapses to 4, arrow-right stays at 4,
and Paste produces the same 8-unit digest. The harness starts a fresh disposable probe
process, scrolls through the grid gutter to avoid arming an editing key, and records visible
text-label tap points before typed input. Seven harness/comparator tests reject wrong
labels, disabled states, actions, geometry, root-container targeting and clipped-leaf mistakes.

## Local gate and source accounting

`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug`
exits **0** (`gate-9.exit`), **BUILD SUCCESSFUL in 3m 6s**, 345 tasks (61 executed,
284 up-to-date). The process status is captured directly after Gradle, without a pipe.
All 681 JVM tests pass with zero skipped/failures/errors; native tests, formatting,
clippy with `-D warnings`, and the C++ oracle pass. App lint contains only
AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport. Rust-module lint
has no findings. No suppression, lint baseline or ignored/deleted test was added.

Compared with `823d3105`, non-test Kotlin has **225 removed / 297 added** lines (net +72,
including Android data/action transport). Rust has **3,142 added / 17 removed** lines;
2,627 added lines are outside Rust test files. Counts include blank/comment lines as git
does. Every changed/new source file stays at or below 500 lines. The Compose fallback is
preserved until the broader renderer/phone acceptance gates are met.

## Visuals, accessibility and regression

`final-visuals/` contains the 12 actual screenshot pairs (Compose left, Rust right):

| State | Floris Day | Floris Night |
| --- | --- | --- |
| Smileys category | `day-smileys-pair.png` | `night-smileys-pair.png` |
| People category | `day-people-pair.png` | `night-people-pair.png` |
| Skin-tone popup | `day-tone-popup-pair.png` | `night-tone-popup-pair.png` |
| Overflow | `day-overflow-pair.png` | `night-overflow-pair.png` |
| Editing controls | `day-editing-pair.png` | `night-editing-pair.png` |
| Arrangement editor | `day-edit-pair.png` | `night-edit-pair.png` |

All 12 matched accessibility comparisons have **zero bug verdicts** (`comparison.json`).
The comparator checks named targets, effective enabled states, click support and centers
within 3 px. It accounts for Compose's text/wrapper split, partially clipped tiles,
minimum touch rectangles and the modal sheet occluding the underlying IME tree. Explicit
native category/control labels and framework wrapper differences are recorded as acceptable.
This establishes the requested dump comparison; it does not claim a new spoken TalkBack trial.

The pairs were visually reviewed. They are **not pixel-identical**: several editor labels
wrap or ellipsize differently (notably One-handed, Switch language and Hide keyboard),
back-icon strokes, popup shadow and tab indicators differ, and the native implementation
does not reproduce Compose's ripple, overscroll/stretch or transient scroll-thumb animation.
Category changes snap rather than animate like the Compose pager. Grid targets, section
positions, theme colours, enabled states and committed results match the checked fixtures.
These remaining visual/transition differences are separate from clipboard/GIF's Compose
ownership and from the broader phone/performance acceptance gates.

With all builds/tests stopped and agent-device idle, `final-regression-clean-service/`
passes both fresh 300-press/eight-e-hold replays with suggestions enabled:

- 300 units: `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`
- 308 units: `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`

The final full logcat has **zero** `FATAL|AndroidRuntime` matches
(`final-cleanup/logcat-verdict.json`). Rejected development captures and the initial,
subsequently fixed JNI array-descriptor crash are retained separately from accepted runs.
The final APK and accepted fixtures have no runtime crash.

## Cleanup and publication boundary

`final-cleanup/state.json` verifies prototype **false**, **ALWAYS_DAY** and bundled
**Floris Day**; the session's exact original history/arrangement values (both originally
absent); original suggestion switches; original accessibility **null / 0**; our IME selected;
DND off; requested mute/minimum levels; and the probe uninstalled. Profiling tags are INFO.
`device_close(shutdown=true)` succeeded for local `emulator-5556`; `device-shutdown.json`
confirms `aw_keyboard` is no longer booted and this thread has no open device. Emulator-5554
was untouched. No screenshots, dumps, builds, local properties or session caches are committed.

Hosted CI is excluded under the account billing constraint. The successful local gate is
the publication gate. Compose remains the default and is retained for fallback and later
clipboard/GIF migration; this package does not remove the Compose keyboard.
