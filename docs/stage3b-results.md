# Stage 3B renderer results

## Accessibility harness follow-up (2026-11-13)

**Tree comparison and ordinary regression pass; TalkBack explore/lift and spoken-state
acceptance remain open.** All accepted runs use the same gated debug APK, SHA-256
`e4066717db0e238ac55f651049dbf2f0f2108d331eaf9eb09013565629909aa5`, installed through
`scripts/install-dev.sh`; its installed APK hash was checked against the build output.
The AudioWhisper Mac checkout was untouched. Only local `aw_keyboard` / `emulator-5556`
was operated; emulator-5554 was never used or closed.

### Fresh tree comparison

The disposable probe now owns a test-only AccessibilityService. It retrieves the IME
window, walks real and virtual children, and atomically writes every requested field,
including states, action IDs/labels and screen bounds. Requests have unique IDs, monotonic
timestamps and content-event generations. Android's accessibility cache is cleared before
each request. Nonempty-editor dumps are refused; tree sessions disable suggestions in the
probe to exclude candidate text. Normal typing and regression keep suggestions enabled.
The original two accessibility settings are saved and restored in `finally`, deleting an
original `null`. Settings switching uses the existing prototype preference UI.

| State | Compose nodes | Rust setting nodes | Labels/descriptions, roles, states, actions, bounds | Verdict |
|---|---:|---:|---|---|
| Letters | 133 | 63 | Same named targets; explicit virtual-key semantics | Acceptable |
| Shifted | 133 | 63 | Uppercase fresh; Rust Shift checked and “Shift enabled” | Acceptable |
| Caps lock | 151 | 67 | Uppercase survives commit/erase; Rust checked and “Caps lock enabled” | Acceptable |
| Symbols | 127 | 127 | Exact tree match; existing Compose fallback on both settings | Matches |
| Expanded quick actions | 151 | 67 | Same action names and named-control bounds | Acceptable |
| e popup | 151 | 69 | Six choice labels/grid targets; explicit selection and dismissal | Acceptable |

Counts include wrappers, hint/glyph leaves and system navigation. Rust supplies actionable
`Keyboard$Key` nodes, button/switch roles, spoken fallback names, click/long-click/dismiss
actions and toggle/selection states where Compose exposes text/icon leaves. These are
intentional differences, not an assertion of identical raw trees. Named control rectangles
and ordinary target centers agree within 3 px. Edge hit rectangles intentionally extend
beyond glyph centers to the screen edge; key targets must remain within the host viewport.
All raw nodes and every field difference have matches/acceptable/bug verdicts in
`accepted-trees/comparison.json`; the six states contain **zero bug verdicts**.

The harness polls stable geometry and expected casing/state. Shift needs a 1,000 ms hold
because its ordinary long-press threshold is 2.5 times the configured delay; a digest-only
uppercase commit/erase verifies caps-lock persistence. A queued 450 ms dump captures a
background 700 ms e-hold before the hold ends. The comparator's nine adversarial tests
reject wrong labels, missing keys/clicks, disabled states, wrong roles/Shift state, displaced
bounds and targets overlapping system navigation. No failed fixture is relabeled a match.

### TalkBack replay and limits

The installed component was found from `dumpsys package ... | grep -i TalkBackService`:
`com.google.android.marvin.talkback/.TalkBackService`. Only that service was enabled for
TalkBack steps, using secure settings. `dumpsys accessibility` confirms TalkBack bound and
`touchExplorationEnabled=true`. The AVD used `-no-audio`, DND off, zero on streams 1/2/3/5,
and the allowed minimum 1 on streams 4/10. A temporary notification-permission grant avoided
its first-enable dialog and was revoked afterwards. No settings UI enabled TalkBack.

**The requested 400 ms input fixture fails (exit 1).** Numeric event tracing showed shell
`input swipe` delivering raw touch, not exploration-hover. Both renderers commit configured
long-press hints instead of the requested word; the accent checkpoints also differ. This
run is retained as a measured failure. A separate 35 ms raw-touch control (exit 0) spells
`cat`, toggles Shift and enters the capital letter, selects an accent and activates Undo,
with identical digests/cursor offsets on both renderers. That control does not establish
physical TalkBack explore/lift behavior.

In the table, each cell is `UTF-16 units / SHA-256 prefix`. At **every** checkpoint the cursor
is collapsed at that unit count on both renderers. Full hashes follow below and in local
JSON. The stationary double-tap/hold then drag targets `(165,1440)`, the center of the dumped
first accent cell; a tap used to focus e also commits input in this raw-injection control.

| Step | 400 ms Compose | 400 ms Rust | 35 ms, both renderers |
|---|---|---|---|
| initial | 0 / `e3b0c44298fc` | 0 / `e3b0c44298fc` | 0 / `e3b0c44298fc` |
| key-1 | 1 / `265fda17a346` | 1 / `265fda17a346` | 1 / `2e7d2c03a950` |
| key-2 | 2 / `859a1b63428f` | 2 / `859a1b63428f` | 2 / `6959097001d1` |
| word | 3 / `5cf293066fe8` | 3 / `5cf293066fe8` | 3 / `77af778b51ab` |
| shift | 3 / `5cf293066fe8` | 3 / `5cf293066fe8` | 3 / `77af778b51ab` |
| capital | 4 / `247e04f20f74` | 4 / `247e04f20f74` | 4 / `979d49ae0b00` |
| accent-focus | 5 / `b0350f18d3e0` | 5 / `b0350f18d3e0` | 5 / `f52ceab1244e` |
| accent-attempt | 7 / `81a465e63133` | 7 / `46bc16194b90` | 7 / `866197b2d274` |
| expand | 7 / `81a465e63133` | 7 / `46bc16194b90` | 7 / `866197b2d274` |
| undo | 0 / `e3b0c44298fc` | 0 / `e3b0c44298fc` | 0 / `e3b0c44298fc` |
| caps-lock-attempt | 0 / `e3b0c44298fc` | 0 / `e3b0c44298fc` | 0 / `e3b0c44298fc` |

| Digest prefix | Full SHA-256 |
|---|---|
| `e3b0c44298fc` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `265fda17a346` | `265fda17a34611b1533d8a281ff680dc5791b0ce0a11c25b35e11c8e75685509` |
| `859a1b63428f` | `859a1b63428fd7ee5663fc29faa1d0f1351b854b9ac35a4be3e2884d04892590` |
| `5cf293066fe8` | `5cf293066fe81f7020f5798c58d2b655717d2b0845c31ea851a5f0939b4af5a0` |
| `247e04f20f74` | `247e04f20f74953cd170646c188fb03eeeb775788107688429c6fa241379b183` |
| `b0350f18d3e0` | `b0350f18d3e0e1eb1dc485147349391ddcf7ead543d296e10468d1fd18c7e169` |
| `81a465e63133` | `81a465e631338e30ca3adaba26b96a8c3fe3c673dd45fe7eef23355994d8d736` |
| `46bc16194b90` | `46bc16194b905edee9da30f509e41134b3024930ad7335296d7fb8b0319831b5` |
| `2e7d2c03a950` | `2e7d2c03a9507ae265ecf5b5356885a53393a2029d241394997265a1a25aefc6` |
| `6959097001d1` | `6959097001d10501ac7d54c0bdb8db61420f658f2922cc26e46d536119a31126` |
| `77af778b51ab` | `77af778b51abd4a3c51c5ddd97204a9c3ae614ebccb75a606c3b6865aed6744e` |
| `979d49ae0b00` | `979d49ae0b00ee3cf9d0556509e22e29a466c2bafc36132779bce719b4f25e91` |
| `f52ceab1244e` | `f52ceab1244e07d7522f69110f4371427e7dc951cbd26aa62844b661b2bd6909` |
| `866197b2d274` | `866197b2d274fd014783ed5ada87b48415b2bea2a89640a63893d24bc7436195` |

The final 35 ms accent result matches the same first popup accent on both renderers.
The earlier endpoint `(165,1560)` was ambiguous under Compose's drag-selection heuristic
and is excluded from the matching control. Shift/caps-lock fixed words were searched in
TalkBack logcat, and accessibility dumps were inspected. **No speech announcement was
confirmed on either renderer.** Bound services and checked semantic nodes are not spoken
output evidence. No general speech/event text or editor contents were retained.

Early runs with a permission dialog, a stale/dead IME connection after APK replacement,
an old APK, or an editor recreation leaving the native surface unavailable are excluded.
The harness now checks service binding and foreground activity, can rebind its own IME,
and resets the existing probe editor without recreating it for the regression. The final
installed APK hash and digest-positive controls establish the accepted runtime boundary.

### Fixes, gate, regression and cleanup

- `e1f3415e`: disposable accessibility/TalkBack harness, fresh capture, comparator and adb regression adapter.
- `e57eaaac`: clip extended physical-key semantic bounds to the renderer viewport. The new native test failed before the fix and passes afterwards.
- `8c84b546`: complete held-popup source release/cancellation and permit captured popup drags above the host rectangle. Seven JVM tests cover these paths, and the final accent/Undo control passes on-device.

The exact gate `./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug` exited **0**, `BUILD SUCCESSFUL in 1m 33s`, 345 tasks
(23 executed/322 up-to-date). Gradle's own status is saved directly in `gate-2.exit`, without
a pipe. The stored JVM results total 670 tests, zero skipped/failures/errors. Rust tests,
format/clippy and C++ oracle checks pass. App lint contains only AndroidGradlePluginVersion,
NewerVersionAvailable and ChromeOsAbiSupport; Rust-module lint has no findings. No
suppression, baseline, ignored test or unsafe Rust was added. Changed files stay below
500 lines; older packages 1/2 are preserved in `stage3b-renderer-history.md`.

Both renderers' fresh TalkBack-off 300-press/eight-e-hold replays passed with suggestions enabled:

- 300 units: `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`
- 308 units: `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`

The final full `logcat -d` filter for `FATAL|AndroidRuntime` has **zero matches**. The old
standalone-uiautomator `UiAutomationService already registered` crash was present in the
pre-run baseline and is preserved separately as tool evidence, not a keyboard crash or an
accepted-run result. No new uiautomator or agent-device accessibility injection was used.

Cleanup confirmed prototype **false**, **ALWAYS_DAY**, **org.florisboard.themes:floris_day**,
original accessibility values **null / 0**, no bound accessibility service, our IME selected,
DND off, and the probe uninstalled. `device_close(shutdown=true)` shut down local
`emulator-5556`; the aw_keyboard emulator process is gone. The first shutdown call while
the device was absent from the panel was a no-op; reopening the same runtime and closing
with shutdown completed it. Emulator-5554 was untouched.

All generated evidence remains ignored under `build/stage3b-accessibility/`: `accepted-trees/`,
`final-talkback-{400,35}-center/`, `accepted-regression/`, `accepted-apk.json`, `gate-2.*`,
`final-cleanup/` and `device-shutdown.json`. No dumps, screenshots or build outputs are
committed. Hosted CI remains outside acceptance because of the account billing limit;
the successful local gate is the gate for this work.

---

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


## Earlier renderer measurements

[Packages 1 and 2: renderer styles, smartbar, visual and timing measurements](stage3b-renderer-history.md).
