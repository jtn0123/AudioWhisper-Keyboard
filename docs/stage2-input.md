# Stage 2 package 13: input dispatch and routing

## Boundary

The Java-free `input` crate owns key-down/up/cancel membership and blocked releases,
long-press and repeat delays/data, consecutive-event clocks, ordered key-code routes,
state-field transitions, space/double-space recipes, backward/forward delete continuations,
active-subtype fallback/cycling, evaluator label/icon choices and feedback eligibility,
effect selection and factors. It forbids unsafe code. Only `awnative` transfers JNI data.

`InputDispatch` uses the existing Cleaner/mutex handle convention. Calls copy their inputs
and return before invoking any receiver, Android API or provider. Timed callbacks carry a
press generation, so a cancelled callback cannot block or repeat a newer same-code press.
State effects identify a field rather than replacing a stale register after an editor call.
The canonical Rust protocol generates the Kotlin-visible Java constants during Gradle builds.

Both Compose and Rust rendering use `InputEventDispatcher` and the same native key routing.
The shared pointer rules also decide rolling-character release, slide bounds and long-press
space/shift/picker/popup behavior. The native renderer retains its existing Rust touch engine;
Compose retains MotionEvent/pointer-to-View bindings and executes the shared decisions.
GIF-search key routing now returns explicit search edits; the managed host no longer routes
those keys a second time. Existing Rust keys, layout assembly, shift, gesture, popup, spacing,
composing and editor engines are reused rather than replaced.

Kotlin retains Android clocks/coroutine wakeups, managed key and callback lifetimes,
observable state/Flow bindings, resources/ICU locale facts, asset loading and Android effects:
InputConnection, InputMethodService, clipboard/Room, preferences, vibrator and AudioManager.
This port left provider/candidate orchestration at the managed NLP boundary. [Package 16](stage2-nlp.md)
now owns its decisions and generations in Rust, with Android objects/execution retained in Kotlin;
decoder, correction/reversion rules and editor operations remain in their already-ported Rust engines.
Dictation/read-aloud effects call the existing
platform hosts. The autocorrect-toggle key retains its legacy placeholder toast.

## JVM proof

The removed receiver is retained as `reference/ReferenceKeyboardKeyEvents.kt`; platform calls
become recorded effects. `ReferenceRoutingManager` supplies independent state/shift logic and
platform facts. `ReferenceInputDispatcher`, `ReferenceInputFeedback`, `ReferenceInputPointer`,
`ReferenceInputVisual`, `ReferenceInputControl` and `ReferenceInputDelete` retain the replaced
policies without calling the new native decisions.

`InputDispatchMatchTest` compares native lifecycle replies, exact ordered routing effects and
final keyboard registers. Routed commits, deletion, selection and Enter run through the
existing reference/native editor pair, checking ordered InputConnection operations, final
UTF-16 text, selection/composing, expected/published windows and spacing. Seeded sessions mix
simultaneously held codes, clock jitter, duplicate downs, cancellation, holds/repeats and
mode changes. Directed tests cover every canonical code, key type and mode, GIF ownership,
clock boundaries, stale timers, malformed packets and closed handles.

`InputFixtureMatchTest` adds the public 300-press/eight-hold shape, shift/caps, rolling keys,
all mode switches and back, gesture/completion provider replies, repeat delete and IME actions.
Existing `RendererTouchMatchTest` retains recorded Android timestamps, complete MotionEvents,
glides, popups, pointer-index reordering and seeded multi-pointer sessions against frozen
Compose touch code. Existing editor suites retain the public correction/suggestion, Unicode,
external-edit and extended editor traces. The proof composes these boundaries; it does not
simulate Android's actual visual cursor movement or provider thread scheduling on the JVM.

Feedback tests compare every key code over feature masks and seeded volumes. Pointer tests
compare 50,000 slide positions plus rolling/hold predicates. Controls compare 20,000 subtype
cohorts (including duplicates, missing active values and empty lists), raw state transitions,
space settings and the complete delete short-circuit outcome matrix.

The match suite exposed and fixed signed widening of negative float volume bits. Inspection
also preserved predefined delete-repeat key metadata and distinguished the dynamic shifted
icon from the otherwise icon-less explicit caps-lock key. No input behavior divergence is
intended; the separate batch-balance correction is documented in [the editor report](stage2-editor.md).

## Reproduction

```sh
./gradlew :app:testDebugUnitTest --tests '*Input*MatchTest' --tests '*RendererTouchMatchTest' --tests '*Editor*Test'
python3 -I scripts/input-mutations.py build/stage2-input/mutations-fresh
```

With `ANDROID_SERIAL` set to the runtime ID of owned `aw_keyboard`, install only through
`scripts/install-dev.sh`; install the probe through its `--probe` option. Close the owned
agent-device app session after snapshots: its UiAutomation owner suppresses accessibility
services even when idle. Keep agent-device idle during these harness operations:

```sh
scripts/render-benchmark/regression.sh build/stage2-input/regression-fresh
python3 -I scripts/render-benchmark/suggestions.py build/stage2-input/suggestions-fresh
python3 -I scripts/render-benchmark/editor.py build/stage2-input/editor-fresh BASELINE/editor.json
python3 -I scripts/render-benchmark/modes.py build/stage2-input/modes-fresh
python3 -I scripts/render-benchmark/input.py build/stage2-input/input-fresh BASELINE/input.json
python3 -I scripts/render-benchmark/input-glide.py build/stage2-input/glide-fresh BASELINE/glide.json
```

The input runner uses only fixed synthetic fixtures. The two-finger case injects overlapping
fixed strokes through the disposable accessibility service. The original repeat fixture starts
with a short prefix and holds for two seconds until it is empty. Package 16 retains it and adds
`repeat-long`: 64 repetitions of `alpha beta gamma ` (1,088 UTF-16 units). The same two-second
hold leaves a nonempty prefix, making repeat-rate changes observable in both length and digest.
The package-16 before/after results are recorded below and in [the NLP proof](stage2-nlp.md).
Search Enter must deliver one SEARCH editor action without a newline;
multiline Enter must commit a newline without an editor action. After recording the digest,
the runner clears only the probe editor to inspect mode/shift labels on the same connection.
Only hashes, counts, offsets, action IDs and empty-editor keyboard trees are retained.

## Mutation proof

All four deliberate changes failed actual JVM assertions (Gradle process exit 1) and were
restored. `scripts/input-mutations.py` restores each file in `finally` and runs the restored
input, editor and touch suites afterward.

| Mutation | Assertion failures |
|---|---:|
| Extend the caps-lock consecutive-down window by 1 ms | 3 |
| Start delete repetition 1 ms early | 3 |
| Route symbols to symbols2 | 4 |
| Stop releasing a suitable character on a second pointer down | 4 |

The multi-pointer mutation additionally failed both the recorded/scripted and seeded complete
MotionEvent comparisons against frozen Compose touch code (two input assertions plus two
touch assertions). The batch fix has its separate missing-end mutation and per-plan nesting invariant in
[stage2-editor.md](stage2-editor.md).


## Local gate

The exact required command completed with **Gradle process exit 0**:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

The full gate has **750 JVM tests, zero failures/errors/skips**, including 442 app tests and
153 Rust-wrapper, 128 Snygg and 27 Kotlin-library tests. Rust formatting, tests, clippy with `-D warnings`, the C++ oracle and
normal debug assembly pass. App lint reports only AndroidGradlePluginVersion, three
NewerVersionAvailable notices and ChromeOsAbiSupport. No warning suppression, lint baseline
or skipped test was added. Hosted CI is not the acceptance gate because of the stated billing limit.

The input seeded suite compared **50,000 events across 200 seeds** with **12,906 ordered editor
operations**, zero differences. The separately retained editor suites compare another 50,000
events across 200 seeds, 201,611 operations and 93,887 checkpoints, with zero differences.


The disposable batch counter is enabled only for the `batch` field. An early harness
revision wrapped every editor connection and changed startup capitalisation callbacks on
both the unchanged baseline and the new APK. Restricting it to the batch field restored
the baseline editor suite. The caps fixture also explicitly waits for uppercase labels
before typing; no expected text or digest was relaxed. The first direct-wrapper Android
fixture hit R8 dead-code removal; the final fixture uses the kept editor JNI transport and
executes only its returned batch effects against the live Android InputConnection.


The Android batch fixture additionally uses the repository's test-only controller:

```sh
./gradlew -PrendererFixture=true :app:assembleDebugAndroidTest
scripts/install-dev.sh --probe app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
# Run this controller in another terminal while executing the fixture commands:
adb -s "$ANDROID_SERIAL" shell am instrument -w com.jtn0123.audiowhisper.keyboard.test/com.jtn0123.audiowhisper.render.FixtureInstrumentation
# In the fixture terminal (0 for the fixed APK; 1 for the original baseline):
python3 -I scripts/render-benchmark/input-batch.py build/stage2-input/batch-fresh 0
python3 -I scripts/render-benchmark/clipboard-gif.py search build/stage2-input/gif-fresh
adb -s "$ANDROID_SERIAL" shell am broadcast -a com.jtn0123.audiowhisper.RENDER_FIXTURE -p com.jtn0123.audiowhisper.keyboard --es op finish
```


## Source accounting

Count nonblank production Kotlin/Java/Rust against `65829830`, excluding tests, benchmark
fixtures, generated character tables and the C++ oracle as in `rust.md`. Kotlin falls from
54,027 to **53,518** (**509 removed**); Rust rises from 42,432 to **43,404** (**972 added**,
including the separate one-line batch fix); Java remains 321. These are net source changes,
not the size of the frozen references. This package advances the project target; it does not
make the whole keyboard 90% Rust. Changed source files remain at or below 500 lines.


## Emulator comparison (Android 16, owned `aw_keyboard`)

The unchanged `65829830` APK and the new normal debug APK were replayed on the runtime
serial returned by T3 (`emulator-5556`). Both Compose and the Rust prototype pass the
300-press and eight-`e`-hold regression (`809d588b…`, `c14d19d2…`), autocorrect
(`6e5ce6af…`) and candidate selection (`106a5842…`). Every extended editor digest and
UTF-16 selection/composing checkpoint in [stage2-editor.md](stage2-editor.md) matches.
The five glide/candidate/cancel checkpoints, emoji insertions, editing quick actions,
clipboard paste/pin/unpin/delete/clear, subtype picker, language-key wrap/advance and
one-handed controls also match. The full native mode fixture confirms virtual keys rather
than a Compose fallback and no surface recreation during replay.

New fixed synthetic input results are identical before/after and between renderers.
Every final selection is collapsed at the listed UTF-16 offset; the mode is characters.
Caps-lock is off at the end of the caps fixture.

| Fixture / final synthetic text | Selection | SHA-256 |
|---|---:|---|
| Caps on/off: `ABc` | 3 | `4f1a1434703d440eadd31ca4fc6bd39f85c5e7f7711faf3f6f4e11b374728c46` |
| Long-press popup: `é` | 1 | `4a99557e4033c3539de2eb65472017cad5f9557f7a0625a09f1c3f6e2ba69c4c` |
| Two-second delete: empty | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| Two-second delete, long prefix (package 16) | 1,055 | `07b9f1e0ce48a5235a5c48458ecd255a131e004a2e5762aa9e59a701434d9055` |
| Two-finger rolling: `as` | 2 | `f4bf9f7fcbedaba0392f108c59d8f4a38b3838efb64877380171b54475c2ade8` |
| Search Enter: `hi` | 2 | `8f434346648f6b96df89dda901c5176b10a6d83961dd3c1ac88b59b2dc327aa4` |
| Multiline Enter: `hi\n` | 3 | `98ea6e4f216f2fb4b69fff9b3a44842c38686ca685f3f55dc48c5d3fb1107be4` |

Search Enter delivers exactly one SEARCH action (ID 3), with no newline; multiline Enter
commits the newline and delivers no editor action. Mode-switch fixtures compare the entire
final digest/selection/composing state, including returning to characters from symbol modes
and changing the editor class for number/phone modes:

| Mode fixture | Final UTF-16 offset | SHA-256 |
|---|---:|---|
| characters | 3 | `77af778b51abd4a3c51c5ddd97204a9c3ae614ebccb75a606c3b6865aed6744e` |
| symbols | 7 | `6095cfc68dcf682e9307e52e8f383dea072ca302424a721e7b2df221f11ab61b` |
| symbols2 | 6 | `a8eb83b745f8cc9c78c086cf27c25f6fcd40cc10156c7553396ebda5c2bb2cdb` |
| numeric | 6 | `eb76e2254ff1cb43115299491e1906fbf39921b384b983cf529ebe674d9915c1` |
| numeric_advanced | 6 | `8d2c644296639a7e51044ad9ba8223343cb736ee9043875b4639fe3f95805702` |
| phone | 6 | `c94dbac6cb932d1623b3c4405371f0d9cc2f26118115ee64b1a68853f4cd9375` |
| phone2 | 6 | `8d2c421453ad197267c43b73563fddbc320cb38ed9fb7e4630724d9eb24a91aa` |
| password | 5 | `01410c699ca99d4d9c5b3a49d9b69fbbd64930ca7c2080e6203b02b0fe921a10` |

The batch-only Android InputConnection counter reports `accepted=false` in both builds,
with depth **1 before / 0 after** on both renderers; text and selection are unchanged.
Accessibility comparison has **zero bug verdicts** across letters, shifted, caps-lock,
symbols, numeric, expanded and popup states. This is semantic-tree proof; it does not claim
physical TalkBack speech testing. Dictation and read-aloud were never started.

GIF-search typing matches the original query digest (`77af778b…`) and leaves the target
editor empty on both renderers, with no API key and no network-result claim. The disposable
test controller finishes successfully and is uninstalled. Its root `am instrument` command
emits five normal `AndroidRuntime` VM startup/exit messages (exit code 0); that capture is
retained separately. The final logcat check uses a fresh normal-app regression window after
the instrumentation controller has exited.

The newly gated normal APK was installed once more through `scripts/install-dev.sh`.
Both renderer 300-press/eight-hold regressions passed again. The final full
`FATAL|AndroidRuntime` logcat expression matches **zero lines**. Cleanup verifies prototype
**off**, **Always day + Floris Day**, the project IME selected, original accessibility
(`services=null`, `enabled=0`), DND off, streams 1/2/3/5 at zero and 4/10 at their allowed
minimum of one. All fixture preferences and empty clipboard history are restored; only
ordinary app-version bookkeeping changes. The disposable probe and test controller APKs
are removed, and T3 closes the owned emulator with `shutdown: true`.

Ignored local evidence is under `build/stage2-input/`: `final-gate-summary.json`,
`final-local-gate.log`, `final-device-comparison.json`, `final-verified/`,
`final-normal-regression/`, `final-normal-cleanup/` and the mutation directories.
The harness's 26 Python self-tests also pass. Screenshots, dumps, APKs, private state copies
and build outputs are excluded from commits.

## Timestamp-driven long-delete oracle (package 12)

The debug-only opt-in `aw-delete` hook records down/up, each actual repeat attempt and its admitted/skipped tick,
and each scheduled wait/deadline in uptime milliseconds. It records timestamps only. The
NO_ACTION delete fixture checks the 300 ms hold deadline and every production repeat deadline
against the Android repeat delay passed to `input::lifecycle::repeat_delay`. Coroutine delays
start after the preceding callback/work; late wakeups do not create hypothetical missed repeats.
The exact expected deletion count is the admitted actual ticks, with attempts/deadlines and
release checked. UP can cancel an attempt before native admission, or race a previously
admitted tick before its log/editor delivery. Each attempt has an explicit timestamped
admitted/skipped outcome; an admitted attempt cannot start after completed UP. The editor's
exact count is checked independently of those timer admissions. The digest must equal the exact fixed source prefix of that length;
selection must equal that length too. No tolerance or historical 1,055/1,056 count is used.
All other input records and the long hold's mode/action/batch facts still compare directly.
The JVM fake-clock tests are unchanged.

`test_delete_timing.py` checks jitter, missing/extra deletes, missing release, a tick after
release and a 50→51 ms production-wait mutation with an independent 50 ms Android deadline.
The actual source interval mutation and restored JVM results are recorded in the TTS report.
**Both-renderer device acceptance remains missing:** the emulator stalled before installing
the timing-hook build, and T3 shutdown failed twice. No device mutation result is claimed.

The production +1 ms mutation produces two fresh assertion failures in
InputDispatchMatchTest (direct repeat-delay boundaries and the 50,000-event lifecycle match).
Restoring exact source bytes gives Gradle exit 0 and all six tests passing. The host timestamp
oracle separately rejects 51 ms waits against the unchanged 50 ms reference deadlines.
