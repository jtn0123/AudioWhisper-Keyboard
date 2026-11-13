# Stage 3B packages 3 and 4: historical input/accessibility results

## Stage 3B package 4: accessibility implementation (2026-11-13)

The default-off renderer now supplies a Rust semantic scene and an Android
`AccessibilityNodeProvider`. **Runtime accessibility acceptance remains open.** The device
command failed twice before installing this package, and device validation stopped under
the requested failure rule. No Compose/Rust tree parity or TalkBack interaction result is
claimed. Compose remains the default.

### Semantic scene and host

`render::accessibility` describes physical key slots, visible smartbar action slots,
clipped candidates and open popup choices. Each node has a stable virtual ID, scene
bounds, label/content description, role, enabled/pressed/selected/checked/locked state and
supported actions. Key IDs survive casing and mode changes; bar IDs follow configured
slots; candidate IDs use opaque object identity tokens, without hashing their text; popup
IDs include the source slot and choice index. Candidate tokens are no longer recycled on
surface recreation. Popup ordering/geometry comes from the existing popup grid and its
UI-to-flat choice mapping. No-op layout spacers are not interactive nodes.

Rust selects all names and action labels, including smartbar lit mic/speaker names and
read-from-top, complete InsertText action text, expand/collapse names, shift/caps-lock
states, spoken punctuation, keyboard modes and Enter's IME action. Android transports a
static resource table and raw editor options; it contains no node labelling rules. Existing
English resources remain the source for Compose smartbar names. The app is English-only;
new fallback key strings are Android resources too. Existing autocorrect's placeholder
handler remains a placeholder; its toggle does not invent an enabled state.

The Compose source audit covered `TextKeyboardLayout`, `QuickActionButton`,
`QuickAction.computeDisplayName`, `CandidatesRow`, `Smartbar` and the Snygg primitives.
Key glyph/hint text can contribute text leaves, but keys have no merged key semantics or
semantic activation action and their icons explicitly have null content descriptions.
Smartbar buttons specify Role.Button and contentDescription, candidates supply primary
and secondary text, and expansion controls have no explicit description. Rust uses those
existing labels, with LatinIME-style fallbacks where descriptions are absent; it adds
explicit key actions and toggle state. The fallback conventions were checked against
[AOSP KeyCodeDescriptionMapper](https://android.googlesource.com/platform/packages/inputmethods/LatinIME/+/refs/heads/main/java/src/com/android/inputmethod/accessibility/KeyCodeDescriptionMapper.java).
This source audit is not a measured tree comparison.

The Android provider translates virtual nodes, accessibility focus/actions and hover/touch
events. Explore-by-touch focuses a hovered node; a lift within that node activates it,
while leaving the viewport cancels. Click and double-tap/hold route through Rust action
packets into the existing InputEventDispatcher, QuickAction and candidate handlers, without
starting glide recognition from hover. Popup choices support selection and dismissal;
focus enters the selected choice and returns to the originating key, or the nearest
surviving key/control when necessary. Only changed virtual nodes receive content events. Mode and
shift state events use fixed native labels; content-change events carry no text. Candidate
and key text is exposed for intentional focus/hover, never logged. Snapshot transport is
inactive while accessibility is disabled, except for an explicit tree query. Released
providers stop querying native handles.

### Match tests and mutation

`RendererAccessibilityMatchTest` compares the native labelling functions used by the scene
with a frozen, test-only Kotlin transcription of Compose's smartbar rules and the agreed
LatinIME key fallback. It visits all **132 retained keyboard JSON files**, including every
retained layout and popup mapping: **5,659 key/popup leaves × four shift states = 22,636
cases**. Auto-text leaves supply lower/upper display casing. This is leaf-label coverage,
not a replay of every assembled keyboard's geometry or every selector combination. Separate
cases compare all 25 Compose quick-action mappings, three lit states and complete InsertText
labels, plus the real snapshot/action JNI path, localization, stable IDs and popup mapping.

Four native tests cover shift on/locked, symbols, labels, roles, disabled/pressed/checked
state, smartbar expansion, candidate identity across reordering, localization, exact popup
grid bounds/choice ordering, popup dismissal, candidate clipping, mode labels and all Enter
IME actions with multiline/no-enter overrides. Existing renderer/input tests remain enabled.

The intentional mutation changed Rust's `Shift enabled` fallback to `Broken shift`.
The JVM match suite failed at fixture 1586 in shifted_manual, with expected/actual labels
and **Gradle exit 1**. It was restored. Evidence is under ignored `build/stage3b4/`:
`mutation.log`, `mutation.exit`, `mutation-result.xml`, and the restored match/gate reports.

### Device validation boundary

Only `aw_keyboard`, runtime **emulator-5556**, was opened; emulator-5554 was not used.
DND was turned off and streams 1/2/3/5 muted, with alarm stream 4 at minimum 1.
Accessibility stream 10 rejected index 0 because its allowed range is 1–15. TalkBack
**16.0.0.738667889** is installed, but it was never enabled and no speech was played.

Opening TalkBack settings through the returned agent-device command failed with
`Error (COMMAND_FAILED): adb exited with code 1`. Its suggested `--debug` retry failed
again. Diagnostics show the command expanded the supplied activity into the nonexistent
`com.google.android.marvin.talkback.com.android.talkback.TalkBackPreferencesActivity`
(`Error type 3`). Device work stopped at that point as instructed; no alternate UI route
or direct adb launch was attempted. The final APK has not been installed on this AVD.

| Requested state | Compose tree | Rust tree | Node count / labels / roles / bounds |
|---|---|---|---|
| Letters | Not captured | Not captured | Unmeasured: device command failure. |
| Shifted | Not captured | Not captured | Unmeasured: device command failure. |
| Caps lock | Not captured | Not captured | Unmeasured: device command failure. |
| Symbols | Not captured | Not captured | Unmeasured: device command failure. |
| Smartbar expanded | Not captured | Not captured | Unmeasured: device command failure. |
| Long-press popup | Not captured | Not captured | Unmeasured: device command failure. |

TalkBack explore/lift word entry, shift, accent selection and smartbar activation were
not exercised on either renderer. The fallback hover-injection checks and the fresh
TalkBack-off 300-press/eight-hold replay are also missing. The expected existing digests
remain `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` and
`c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`;
**neither was reverified for this package**.

The pre-install logcat scan contained 11 AndroidRuntime shell startup/exit informational
or debug lines and no FATAL or error-level AndroidRuntime entry. That is baseline evidence,
not a runtime check of this implementation. Cleanup confirmed prototype false,
`ALWAYS_DAY`, `org.florisboard.themes:floris_day`, accessibility services unchanged at
`null` (TalkBack off), DND off, and no installed probe app. The runtime device was closed
with `shutdown: true`. Local evidence: `device-cleanup.json`, `baseline-runtime-errors.txt`.

### Local gate and source accounting

The exact required command exited **0**, `BUILD SUCCESSFUL in 1m 27s`, with 345 tasks
(22 executed, 323 up-to-date):

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

The shell recorded Gradle's own status without a pipe in `accepted-gate.exit`; output is
`accepted-gate.log`. All JVM/native tests, rustfmt, clippy with `-D warnings`, C++ oracle
checks, Android lint and assembly passed. Lint contains exactly AndroidGradlePluginVersion,
NewerVersionAvailable and ChromeOsAbiSupport; the Rust module has no lint findings. Earlier
full gates also passed; the accepted run follows the final provider lifecycle/focus fixes.

Relative to `d7b21e29`, nonblank non-test production source removes **3 Kotlin lines**
and adds **389** (net **+386**); Rust adds **608** and removes **2** (net **+606**).
The platform accessibility bridge grows Kotlin; no Compose fallback is removed in this
package. Totals under the plan's exclusions are 53,895 Kotlin, 35,348 Rust and 627 Java.
All changed source files, including tests, are at most **377 lines**. No new suppressions,
allow attributes, baselines, ignored/deleted tests, screenshots, dumps, build outputs or
local settings are committed. Source accounting is in `source-count.json`.

Hosted CI is blocked by the account billing limit before job steps start. Its new run's
state is reported with the pushed implementation handoff; local Gradle is the acceptance
gate authorized for this package. Device accessibility acceptance remains open above.

## Stage 3B package 3: touch parity and the glide trail (2026-11-13)

The default-off prototype now recognizes keyboard and smartbar gestures in Rust. Kotlin
packs MotionEvents and executes native action packets through the existing
InputEventDispatcher, KeyboardManager, QuickAction and GlideTypingManager paths. The
emulator comparisons below pass. The stage 3 performance target remains open: native CPU
scene+raster p95 is 4.490–4.553 ms in these glide-inclusive trials, above the 2 ms target.
This package does not establish phone, accessibility, GPU or pixel-exact visual acceptance.

### Touch audit and implementation

The audit followed `TextKeyboardLayout.kt`, `TextKeyboardLayoutController.kt`,
`TextKeyboardSwipe.kt`, `KeyboardKeyEvents.kt`, `InputEventDispatcher.kt`,
`GlideTypingGesture.kt`, `GlideTypingManager.kt`, the gesture preferences and settings UI.
The native controller preserves four-slot PointerMap iteration, pointer IDs versus indices,
rolling character release while retaining modifiers, computed-data-on-down semantics,
current-sample key sliding and hysteresis, no key reactivation after leaving, and mass
selection begin/end. The swipe policy covers the nine preference slots, distance/velocity
thresholds, cursor first-unit dead zones, precise character/word selection, delete-on-release,
GIF/raw editor behavior, shift slide and all SwipeAction values. Velocity uses Android 16's
LSQ2 arithmetic, history horizon and stop rule. Glide recognition uses the existing gestures
crate; ordered point/complete/cancel callbacks feed the existing glide decoder and candidate
handlers. Rich-editor, password and keyboard-mode eligibility follow Compose.

Key long presses and repeat timing still use the same InputEventDispatcher as Compose:
longPressDelay, its space/shift/language multipliers, ViewConfiguration key-repeat delay,
word-repeat multiplier and configured delete repeat data. Rust returns the long-press
consumption, popup, caps-lock and input-method-picker decisions. Double-tap caps lock,
held shift and repeat actions therefore retain the existing KeyboardManager shift pipeline.
Popup ownership, two-row grid selection, prioritized-to-flat choice mapping and exit/re-hit
behavior use the existing Rust popup crate rather than the prototype's former flat strip.

Smartbar pointer ownership, toggle consumption, enabled-action invalidation, candidate hold
ownership and deadlines moved from Kotlin into Rust. Kotlin retains opaque candidate-object
bindings, platform action calls and tooltip presentation. Rust retains a held candidate's
identity across suggestion replacement; a tooltip that cannot be shown does not consume a
normal release. Read-aloud's special hold and tooltip linger behavior remain intact. One
scheduled native deadline replaces the Kotlin per-pointer timers; it uses longPressDelay.
`render/src/protocol.rs` is the canonical definition for action codes, toggle identity,
strides, offsets and flags. Gradle generates RenderProtocol.java from it.

The native trail resolves `glide-trail` foreground through Snygg with Compose's green
fallback. Its physical-pixel 20 px head radius, 3 px circle spacing, 0.99 taper, point age
cutoff and quadratic fade match the frozen Compose drawing reference. It honors showTrail
and trailDuration, damages old and new trail bounds, and uses Choreographer timestamps.
The native frame result requests another callback only while circles remain visible (or an
existing smartbar animation is active). No periodic renderer timer remains after fade.

### Match coverage and deliberate mutations

Production Compose remains the control. Test-only frozen Kotlin references preserve its
controller, swipe policy, detectors, popup grid, smartbar host and trail drawing rules.

| Match suite | Coverage |
|---|---|
| RendererTouchMatchTest | Five tests: scripted gestures; all 30 SwipeAction values in each of nine preference slots; 1,800 seeded schedules with 24 scheduled steps, four pointers, reordering/reuse, three densities, history, raw/GIF and hold decisions; two historical-sample regressions; 616 recorded Android touch timestamps from the public 300-press/eight-hold benchmark. Recorded target identities are normalized to the test geometry; no user paths or text are retained. |
| RendererSmartbarTouchMatchTest | 3,200 seeded schedules over four layouts and 100/300/750 ms delays, deadline boundaries, two pointers, cancellation, invalidation and toggles. |
| RendererPopupMatchTest | 5,300 schedules covering 1–14 choices, 0–3 prioritized choices, both orientations, selection and exit; calls the actual renderer JNI and frozen Compose grid. |
| GlideTrailMatchTest | 2,400 seeded point/time/duration/age/fade/strip cases, comparing exact float circle arrays through JNI. |
| Native renderer / JNI regressions | Actual glide callback-to-trail path, outside cancellation before/after recognition, full-versus-dirty pixels through overlapping active/fading trails, 120 idle frames, hidden trails, native frame continuation, candidate identity and unavailable tooltip release. |

Seven intentional mutations each made its match test fail with Gradle exit 1, and each was
restored: smartbar hold consumption; cursor first-unit dead zone; word-selection unit divisor;
space long-press consumption; trail taper 0.99→0.98; popup UI-to-flat choice mapping; and
resampled MOVE boundary cancellation using any outside sample instead of an entirely outside
batch. Logs, exit files and failing XML are under `build/stage3b3/*mutation*`.

Real emulator replay also exposed Android resampling extrapolation: the current MOVE can
fall outside the surface while genuine historical samples remain inside. Cancellation now
requires an entirely outside MOVE batch, or an outside release of the tracked finger.
Independent injected MOVE downTime may also be newer than its historical samples; timestamp
validation checks sample order rather than treating that valid packet as cancellation. Both
have JVM regressions. Earlier failed exploratory attempts remain evidence, but are excluded
from the accepted comparison and timing measurements.

### Emulator comparison

Only `aw_keyboard`, runtime **emulator-5556**, was used. The other session's emulator-5554
was never accessed. DND stayed off; streams 1/2/3/5 were muted and stream 4 was at minimum 1.
The keyboard was installed through `scripts/install-dev.sh`. The same scripted gestures
ran on both surfaces, with digest and selection captured after each step. Continuous public
synthetic glide fixtures are interpolated to avoid discontinuous injected jumps. Evidence
stores only lengths, SHA-256 and selection offsets, never editor text or recorded glide paths.

All 13 editor checkpoints match. The table shows UTF-16 length and collapsed cursor position;
full hashes and both selection endpoints are in `build/stage3b3/gesture-comparison.json`.

| Step | Compose | Rust | Result |
|---|---|---|---|
| Glide 1 | 4 / 4 | 4 / 4 | Same committed digest. |
| Pick candidate | 4 / 4 | 4 / 4 | Same replacement digest, changed from glide 1. |
| Glide 2 | 10 / 10 | 10 / 10 | Match. |
| Glide 3 | 15 / 15 | 15 / 15 | Match. |
| Space swipe left | 15 / 6 | 15 / 6 | Match. |
| Space swipe right | 15 / 11 | 15 / 11 | Match. |
| Delete swipe | 4 / 0 | 4 / 0 | Same deletion and selection. |
| Reposition before repeat | 4 / 4 | 4 / 4 | Nonempty prefix asserted. |
| Backspace hold, 2 s | 0 / 0 | 0 / 0 | Four characters removed; repeated deletion asserted. |
| Double shift, then two keys | 2 / 2 | 2 / 2 | Same digest and caps-lock effect. |
| Shift held with character | 3 / 3 | 3 / 3 | Same digest. |
| Rolling two-finger input | 5 / 5 | 5 / 5 | Same digest. |
| Cancelled glide | 5 / 5 | 5 / 5 | Editor digest and selection unchanged. |
| Configured swipe up: shift, then key | 1 / 1 | 1 / 1 | Same capitalized digest. |
| Configured swipe down: hide | Hidden | Hidden | Both report mInputShown=false. |
| Configured swipe left/right: next/previous subtype | Stable editor | Stable editor | Routing covered by JVM traces; only one installed subtype, so no visible subtype transition is claimed. |

The default global swipe checks temporarily disabled glide, as Compose suppresses those
character-key swipes during glide recognition. Glide was restored afterward. Other space
swipe/long-press choices, precise word deletion, language-key holds, shift-slide, raw/GIF
branches and all configurable action assignments are covered by JVM traces; they were not
individually exercised as device configurations. Repeat scheduling itself is shared code.
An initial repeat check at cursor zero was inadequate; the accepted replay adds repositioning
and requires at least two deletions. Accepted evidence: `gesture-acceptance5.exit` (0),
`gestures/{compose,rust}/gestures.json`, `gesture-comparison.json`, `global-swipes.json`.

Mid-gesture screenshots are extracted from real agent-device recordings; video compression
and different in-stroke progress limit pixel comparisons. Paths relative to `build/stage3b3/`:

- `screenshots/compose-day-trail.png`
- `screenshots/rust-day-trail.png`
- `screenshots/compose-night-trail.png`
- `screenshots/rust-night-trail.png`

After fade, the native posted-frame counter stayed **490→490 over 2.045 seconds** with the
surface visible and no input. Evidence: `idle.json`. Hidden-trail and frame-continuation
behavior also pass JNI/native tests. The four timing trials below each reran the 300-press
and eight `e`-hold scripts and matched both reference digests:

- 300 UTF-16 units: `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`
- 308 UTF-16 units: `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`

### Alternating glide-inclusive timing trials

Order: Compose 1, Rust 1, Compose 2, Rust 2. Each includes 300 taps, eight popup holds,
three glides, a candidate pick and a cancelled glide, including trail/fade frames. No build
or test was running during these measurements. CPU numbers have different boundaries:
Compose is HWUI draw recording, while Rust is native scene+raster CPU. No speedup ratio is
inferred. End-to-end callbacks also differ (Compose frame commit versus native buffer post).

| Trial | CPU frames | CPU p50 / p95 / p99 ms | Touch latency samples | Touch p50 / p95 / p99 ms |
|---|---:|---|---:|---|
| Compose 1 | 1,063 | 1.185 / 2.041 / 2.852 | 866 | 31.623 / 45.038 / 50.781 |
| Rust 1 | 922 | 1.178 / 4.490 / 6.225 | 859 | 28.434 / 51.308 / 53.490 |
| Compose 2 | 1,065 | 0.862 / 1.428 / 3.345 | 866 | 30.806 / 45.064 / 51.182 |
| Rust 2 | 925 | 1.145 / 4.553 / 6.666 | 859 | 28.468 / 51.370 / 54.486 |

Compose HWUI frame-completion p50/p95/p99 is 17.386/19.682/35.554 ms and
17.474/22.687/35.015 ms. These are emulator observations, not physical-phone results.
Raw numeric logs, framestats and digest files are under `build/stage3b3/trials/`;
`timings.json` and `comparison.html` summarize them. Final retained logcat scan found zero
FATAL/AndroidRuntime errors (`logcat-errors.txt` is empty). No GPU p95 or phone claim is made.

### Local gate, source accounting and cleanup

The exact required command completed with **Gradle exit 0**, `BUILD SUCCESSFUL in 2m 11s`,
345 tasks (16 executed, 329 up-to-date):

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

Evidence: `build/stage3b3/final-gate.log` and `final-gate.exit`. Clippy runs with `-D warnings`.
App lint has only AndroidGradlePluginVersion, NewerVersionAvailable and ChromeOsAbiSupport;
Rust module lint has no findings. No suppressions, allow attributes, baselines or skipped tests
were added. Changed source files stay below 500 lines (largest is a 482-line test file).

Relative to 7c0b8183, the tracked nonblank production-source measure removes **138 Kotlin
lines** and adds **255** (net **+117**); Rust adds **1,731** and removes **205** (net
**+1,526**). Kotlin platform-host code grew while pointer ownership moved native. Tests,
benchmark code and generated sources are excluded. `source-count.json` records the totals.

Cleanup restores prototype off, Always day + Floris Day, default gestures (including glide
on), INFO profiling tags, DND off and muted streams. The probe app is removed and only
emulator-5556 is shut down through device_close. Cleanup evidence: `cleanup.json`.
Hosted GitHub CI is blocked by account billing before steps start; it cannot replace the
successful local gate. The final pushed run state is reported with the implementation handoff.
