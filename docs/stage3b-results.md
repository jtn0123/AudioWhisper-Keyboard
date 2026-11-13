# Stage 3B renderer results

## Stage 3B package 5: every text mode and inline autofill (2026-11-13)

**All text modes, mode/field switches, the number row, screenshots, accessibility trees and
ordinary regression pass on the emulator. Inline autofill hosting is implemented but live
service-produced inline views remain unverified.** The prototype stays default-off.
Earlier sections below describe pre-package-5 behavior.

### Implementation and differential tests

`TextInputLayout` no longer filters CHARACTERS or falls back when inline suggestions arrive.
One `RustKeyboardView` consumes the evaluated keys/geometry from the existing `keys` crate
for CHARACTERS, SYMBOLS, SYMBOLS2, NUMERIC, NUMERIC_ADVANCED, PHONE and PHONE2, including
password variation and number-row layouts. No second layout or mode-specific surface is
constructed. The deprecated smartbar-number/cursor row enums have no active text producer
in this tree; native state/geometry tests include those auxiliary values too. Emoji,
clipboard, media/GIF, action overflow and other unreplaced panels remain Compose; GIF search
still uses the existing Compose text path.

Rust reserves the inline center region, suppressing native candidates/drawing/hits/semantic
nodes there while retaining pinned controls and extended actions. Android hosts the existing
`InlineContentView` row over that measured reservation. Expanded shared actions and dictation
preview keep Compose's precedence. Inline arrival/removal preserves the surface and insets.
Native popups participate in the platform popup count so inline views use the same popup
z-order rule as Compose. No suggestion contents enter diagnostics. The AVD has a configured
Google autofill service, but no service-generated inline suggestions were exercised in this
probe; layout/hit/semantic tests do **not** establish live autofill interaction acceptance.

PHONE numeric digits and their letter hints now use Compose's side-by-side bias alignment;
PHONE2 and other key content keep their existing alignment. The rule is entirely Rust.
The JVM tests use frozen test-only Kotlin references: 9,216 new inline/preview/settings/width
cases (plus 9,216 existing smartbar cases), 4,200 phone placement cases and 49 key-specific
hold-clock cases. Native tests cover inline hit/semantic exclusion, stable regions/insets,
preview precedence, phone placement and 48 mode/row-count pointer/popup transitions.

Deliberate mutations inverted inline visibility and restored the old Shift hold multiplier:
both JVM suites failed, Gradle exit 1 (`mutation.*`). Centering phone digits deliberately
also failed its JVM match suite, exit 1 (`phone-mutation.*`). All mutations were restored;
the full final gate passes. No suppressed warnings, lint baseline or skipped tests were added.

### TalkBack divergence resolved

The 400 ms divergence was **case state, not accent-grid selection**. The accessibility bridge
scheduled Shift at the base 300 ms delay, while Compose's dispatcher uses 2.5× (750 ms).
A 400 ms Shift touch therefore locked caps only in Rust; the later base key and accent were
uppercase. Native semantic nodes now supply the same key-specific hold clocks as Compose
(2.5× Space/CJK Space/Shift, 2× language switch, 1× other keys).

The final 400 ms run matches every digest and cursor checkpoint, including the accent result
`81a465e631338e30ca3adaba26b96a8c3fe3c673dd45fe7eef23355994d8d736` on **both** renderers.
It still exits 1 for its intentionally unsuitable expected-word check: 400 ms shell-injected
raw touches choose long-press hints. The 35 ms control exits 0 with full expected-word and
cursor parity; its accent digest is
`866197b2d274fd014783ed5ada87b48415b2bea2a89640a63893d24bc7436195` on both.
These controls do not establish physical TalkBack explore/lift or spoken-output acceptance.

### Fresh emulator evidence

Only local `aw_keyboard`, runtime **emulator-5556**, was used. Emulator-5554 and the Mac
AudioWhisper checkout were untouched. The final APK was installed through `scripts/install-dev.sh`
and its installed SHA-256 equals the build output:

`553945602d10dbc083cae61868fe63d4c83ae28bc45f711d0bd2d0b4e054cffe`

All fixtures match expected SHA-256, UTF-16 length and collapsed cursor on both renderers.
Rust virtual keys are required, so symbols cannot pass through the old Compose fallback.
No surface create/change/destroy/attach/detach or native-window acquire/release occurs during
these field/mode replays. Full lifecycle records are in `accepted-modes/*-lifecycle.json`.

| Mode / field | Units | SHA-256, both renderers |
|---|---:|---|
| characters | 3 | `77af778b51abd4a3c51c5ddd97204a9c3ae614ebccb75a606c3b6865aed6744e` |
| symbols | 7 | `6095cfc68dcf682e9307e52e8f383dea072ca302424a721e7b2df221f11ab61b` |
| symbols2 | 6 | `a8eb83b745f8cc9c78c086cf27c25f6fcd40cc10156c7553396ebda5c2bb2cdb` |
| numeric | 6 | `eb76e2254ff1cb43115299491e1906fbf39921b384b983cf529ebe674d9915c1` |
| numeric_advanced | 6 | `8d2c644296639a7e51044ad9ba8223343cb736ee9043875b4639fe3f95805702` |
| phone | 6 | `c94dbac6cb932d1623b3c4405371f0d9cc2f26118115ee64b1a68853f4cd9375` |
| phone2 | 6 | `8d2c421453ad197267c43b73563fddbc320cb38ed9fb7e4630724d9eb24a91aa` |
| password | 5 | `01410c699ca99d4d9c5b3a49d9b69fbbd64930ca7c2080e6203b02b0fe921a10` |

The five-row number-row character→symbols→symbols2→character replay also matches: 5 units, `657f8b5ecfe3daa49d0664bd5c60e8be03cde405f7fa8ee8b7b4ef3bad9a6159`. The preference is restored afterwards.

The fresh `a11y.sh` dumps have **zero bug verdicts**. The comparator maps spoken punctuation
and mode names to their exact Compose glyphs and checks geometry; a disabled icon leaf
inherits its merged button's disabled state. Twelve adversarial comparator tests include
wrong/missing punctuation, displaced targets, disabled-state changes and missing clicks.

| State | Compose nodes | Rust nodes | Bugs |
|---|---:|---:|---:|
| letters | 133 | 63 | 0 |
| shifted | 133 | 63 | 0 |
| caps-lock | 151 | 67 | 0 |
| symbols | 127 | 69 | 0 |
| numeric | 89 | 50 | 0 |
| expanded | 151 | 67 | 0 |
| popup | 151 | 69 | 0 |

Capture sessions temporarily disable both word and emoji suggestions through settings:
the text-only no-suggestions flag does not cover numeric/phone fields. Both original switches
are restored before normal typing/timing. A preliminary numeric capture exposed retained
predictions and an inner disabled-icon comparison; its aggregate failed counts are retained,
while candidate-bearing raw artifacts were removed. No failed result was relabeled a pass.

### Reviewed screenshot pairs

Compose is left, Rust is right. All ten accepted pairs show the expected evaluated keys,
mode controls, field-specific Enter action and phone digit/letter placement. Residual
variation is antialiasing, shadow softness and small glyph/hint offsets; no pixel-identity
claim is made. Paths are local ignored evidence relative to the repository:

| State | Floris Day | Floris Night |
|---|---|---|
| symbols | [Day](../build/stage3b5/screenshots/day-symbols-pair.png) | [Night](../build/stage3b5/screenshots/night-symbols-pair.png) |
| symbols2 | [Day](../build/stage3b5/screenshots/day-symbols2-pair.png) | [Night](../build/stage3b5/screenshots/night-symbols2-pair.png) |
| numeric | [Day](../build/stage3b5/screenshots/day-numeric-pair.png) | [Night](../build/stage3b5/screenshots/night-numeric-pair.png) |
| phone | [Day](../build/stage3b5/screenshots/day-phone-pair.png) | [Night](../build/stage3b5/screenshots/night-phone-pair.png) |
| password | [Day](../build/stage3b5/screenshots/day-password-pair.png) | [Night](../build/stage3b5/screenshots/night-password-pair.png) |

An initial Day set was blank because the screenshot fixture redundantly reopened the probe
**after** preparing its editor, destroying the prepared native surface. A before/after
replay reproduced that exact lifecycle (`day-repro/native.log`, before/after PNGs). The
screenshot target is now opened before preparation; every Rust capture requires a fresh
posted native frame and no draw/post errors. Accepted pairs pass those guards. The rejected
set is retained separately as `screenshots-rejected-launch/`; an interrupted timing attempt
is excluded under `trials-interrupted/`. This is distinct from the mode-switch lifecycle
checks, which pass without recreating the surface.

### Alternating CPU trials and regression

Order: Compose 1, Rust 1, Compose 2, Rust 2. Each runs the original 300 short presses plus
eight 500 ms e holds, with suggestions enabled. Both reference digests pass in **all four**:

- 300 units: `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`
- 308 units: `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`

No build or test runs during these measurements. CPU backend only; `aw-profile=INFO`,
no added glide segment. Compose measures HWUI draw recording; Rust measures scene+raster
CPU. Their touch endpoints are Compose commit versus native buffer post. These are different
boundaries, so no speedup ratio is inferred. This does not replace glide-inclusive/profiled,
GPU, physical-phone/120 Hz or Perfetto acceptance.

| Trial | CPU frames | CPU p50 / p95 / p99 ms | Touch samples | Touch p50 / p95 / p99 ms |
|---|---:|---|---:|---|
| compose-1 | 949 | 0.727 / 1.057 / 1.594 | 616 | 20.449 / 26.764 / 29.739 |
| rust-1 | 625 | 1.238 / 1.744 / 4.511 | 616 | 19.791 / 22.471 / 23.138 |
| compose-2 | 948 | 0.720 / 1.042 / 1.360 | 616 | 20.602 / 27.599 / 31.528 |
| rust-2 | 627 | 1.201 / 1.711 / 3.951 | 616 | 20.190 / 22.534 / 23.222 |

Raw timing, framestats and digest files are under `trials/`; `trials/timings.json` summarizes
all four. The final full `adb logcat -d | grep -E 'FATAL|AndroidRuntime'` scan has **zero
matches**, and all accepted visual/trial native logs contain no draw/post errors.

### Gate, source accounting and cleanup

The exact required command completed with **Gradle exit 0**, `BUILD SUCCESSFUL in 3m 6s`,
345 tasks (54 executed/291 up-to-date):

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

Evidence: `gate-4.log` and `gate-4.exit`. Format, Rust tests, clippy `-D warnings`, JVM tests
and C++ oracle pass. App lint contains only AndroidGradlePluginVersion, NewerVersionAvailable
and ChromeOsAbiSupport; Rust-module lint has no findings. The first gate caught an unused
new-test import; it was fixed, with no suppression. The final JVM reports for app/Rust/Snygg
contain 646 tests with zero failures/errors/skips. Changed code stays below 500 lines
(largest: 418-line `raster.rs`).

Against `72876637`, nonblank production Kotlin adds **42**, removes **10** (net **+32**);
Rust adds **135**, removes **9** (net **+126**). Tests, benchmark code and generated sources
are excluded. This package adds small Android hosting/oracle boundaries; it does not delete
the Compose renderer before phone acceptance. `source-count.json` records the measure.

Cleanup verifies prototype **off**, **Always day + Floris Day**, original accessibility
**null / 0**, both suggestion switches restored, original number-row setting restored,
our IME selected, DND off and the probe removed. Streams 1/2/3/5 are zero; Android rejects
zero for stream 10 (allowed minimum 1), so 10 and alarm stream 4 are at minimum 1.
Only emulator-5556 is shut down through `device_close(shutdown=true)`.

All generated evidence is ignored under `build/stage3b5/`. No screenshots, dumps, build output,
`.claude/`, `.kotlin/sessions` or `local.properties` is committed; local.properties is unchanged.
Hosted GitHub CI remains blocked by the stated account billing limit; the successful local
gate is the acceptance gate for this work.

---

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

## Package 6: emoji, editing and actions panels

[Implementation, parity fixtures, mutations and device evidence](stage3b-package6.md).
The default-off Rust surface now owns emoji, editing action tiles, overflow and its
arrangement editor. Clipboard history and GIF/search remain Compose.

---

## Earlier renderer measurements

[Packages 3 and 4: touch, glide and accessibility implementation history](stage3b-input-history.md).

[Packages 1 and 2: styles, smartbar, visual and timing measurements](stage3b-renderer-history.md).
