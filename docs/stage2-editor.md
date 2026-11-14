# Stage 2 packages 14 and 15: editor state and operation plans

The Java-free `editor` crate owns expected UTF-16 text, cursor and composing regions, cached
windows, the expected-content queue, reconciliation, last-commit positions and mass selection.
It interprets raw input types and IME flags, including keyboard/variation/composing/incognito
setup and enter-action selection. Android supplies the raw platform values.

Rust also owns character/text/completion/gesture/dictation commits, autocorrect and dictation
replacement/revert plans, backward/forward character and word deletion, selection extents,
phantom/automatic spacing, automatic capitalisation, arrow modifiers and the two-operation
double-space period sequence. It calls the existing `spacing` and `dictation` Rust rules
directly, including the `composing` rule table supplied when the loaded composer changes.
No character-by-character JNI callback implements an editor decision.

Kotlin retains the IME lifecycle and observable DTOs, preference/resource/provider access,
Android ICU segmentation, clipboard and rich-content APIs, key-event timestamps and the
InputConnection interpreter. The NLP provider still supplies a whole composing-word range;
provider orchestration belongs to package 16. ICU is retained under the roadmap's explicit
boundary rather than replaced with a different grapheme/word definition.

## Protocol and ownership

`EditorSession` owns one `Mutex<Session>` allocation through `awnative`. Its synchronized
allocation serializes calls with Cleaner/explicit close and frees once. Closed calls throw.
The logic crate forbids unsafe code. JNI only copies UTF-16 fields and wraps errors; every
unsafe block has a safety comment. No native lock spans platform or provider work.

One editor event produces an ordered InputConnection plan. When platform analysis is needed,
Rust first returns a whole-window read, ICU measurement or provider request. Kotlin
supplies the requested data and resumes the same native event. The final interpreter executes
all effects in order and reports every Boolean result. Editor-action/context-menu return
values are preserved; other effects retain the legacy return conventions. Rejected effects
are counted internally without text logging. Android's ordinary `endBatchEdit(false)` is
not classified as an editor failure.

The native state exposes the newest predicted content separately from the published content.
A selection notification drains predictions until both selection and composing match and the
128-unit safe margin is present. Otherwise it reads the real editor's 256/128-unit window.
Full extracted-text reports invalidate predictions, apply their start offset, and rebuild a
bounded window. Partial reports request a fresh InputConnection window, avoiding guessed
patch coordinates. Invalid reports clear published state. The old editor had no extracted-text
callback; the new callback is checked against the reference's canonical fresh-read recovery.

Capitalisation is a Rust port of Android 16's
[TextUtils.getCapsMode](https://android.googlesource.com/platform/frameworks/base/+/refs/heads/android16-release/core/java/android/text/TextUtils.java).
It includes opening/closing punctuation, paragraph starts, spaces, sentence punctuation,
abbreviations and phantom-space lookahead. The small UTF-16 punctuation category table is
regenerated with `lib/rust/tools/generate-editor-punctuation.sh`; letter rules use `chars`.

## JVM proof

The removed Kotlin operations and state are compiled as `ReferenceAbstractEditorInstance`,
`ReferenceEditorState` and `ReferenceEditorFlow`. Input attributes, IME flags and content/range
models have independent `Reference*.kt` implementations too. Only Android/manager I/O is
adapted. The capitalisation oracle is the unchanged AOSP Java method in `ReferenceCaps.java`.
Existing spacing, composing, shift and correction-rule reference tests remain in the full gate.
The now-unused production spacing JNI adapter also moved to test-only `ReferenceSpacingAdapter.kt`.

`FakeInputConnection` models UTF-16 replacement, selection, composing spans, surrounding text,
code-point deletion and batch nesting. It records ordered effects, including modifiers and
repeat counts. ICU4J supplies JVM grapheme/word boundaries to both implementations. Android
fixtures below exercise the actual EditText and Android ICU. The fake records hardware intents;
it does not claim to reproduce Android's visual bidi cursor layout.

`EditorPlanMatchTest` runs the exact 300-press/eight-hold public pattern, autocorrect/completion
and combining-composer traces, Hebrew/Arabic, surrogate pairs, flags, skin tones and ZWJ emoji.
It additionally runs **25,000 operations over 100 seeds**, starting at `20261113`: text/character
commits, both delete directions and units, selection changes, external edits, composing
finalization/refresh, replacements, IME actions and delayed/coalesced selection notifications.

`EditorFlowMatchTest` adds **25,000 events over another 100 seeds** (`20261213` onward): suggestion
picks, gesture commits, autocorrect reversion, phantom/automatic spacing, dictation/reversion,
deletes, enter and external edits. Fixed cases include selection extents and compound
batch sequencing for double-space periods. Every checkpoint compares final text, UTF-16
selection/composing, the entire expected window, published state, queue presence,
last-commit position, and the exact ordered InputConnection effects.

Input-attribute and IME-option tests compare **50,000 integers each**. Capitalisation compares
all 65,536 UTF-16 units under all eight requested-mode combinations: **524,288 cases**.
Boundary tests cover raw editors, hardware modifiers, mass selection, arrow plans, full/partial
extraction, malformed messages, closed handles and idempotent close.

During implementation the comparisons exposed an unnecessary prediction added by a no-op
composing refresh and missing phantom-space lookahead when accepting a queued update.
Both were corrected. Inspection also found that out-of-window Kotlin `safeSubstring` returns
empty, rather than a clipped substring; the Rust model now follows that exact behavior, with
a regression for a cursor jumping outside the cached window.

Forward deletion still advances the predicted cursor until the real selection notification
reconciles it. Invalid low-level composing finalization deliberately diverges from legacy
FlorisBoard: it emits both `beginBatchEdit` and `endBatchEdit` before returning false. The
legacy unmatched begin could suppress editor selection/extracted-text updates for the rest
of the connection. The test-only reference includes this documented correction. Every plan
in both seeded suites is independently checked for nonnegative nesting and zero final batch
depth, including raw-editor and invalid-composing finalization. Dropping the new end is an
additional deliberate mutation, restored after the invariant catches it. The follow-up
[package 13 Android fixture](stage2-input.md) also measures the actual InputConnection:
invalid finalization remains rejected, while batch depth changes from 1 in the baseline
to 0 in the fixed build on both renderers, with unchanged text and selection.

The seeded suites compared **201,611 InputConnection effects at 93,887 state checkpoints**:
94,087 effects / 43,787 checkpoints in the low-level suite and 107,524 / 50,100 in editor flows.
Their final restored runs have zero differences.

Five deliberate mutations each failed actual JVM differential assertions, with Gradle exit 1:

| Mutation | Assertions failing | Restored |
|---|---:|---|
| Drop the end from invalid composing finalization | 1 | Yes |
| Delete one UTF-16 unit instead of two at a surrogate-pair boundary | 3 | Yes |
| Invert the pending phantom-space insertion rule | 2 | Yes |
| Remove both begin/end effects from the commit plan | 5 | Yes |
| Accept a matching selection with the wrong composing region | 3 | Yes |

All editor tests passed after restoration. `scripts/editor-mutations.py` reproduces the five
edits, verifies failures and restores sources in `finally`; evidence stays in ignored build output.

## Reproduction

```sh
./gradlew :app:testDebugUnitTest --tests '*Editor*Test'
python3 -I scripts/editor-mutations.py build/stage2-editor/mutations-fresh
```

With `ANDROID_SERIAL` set to the runtime ID returned by the T3 device tools, install only with
`scripts/install-dev.sh`. Build/install the disposable probe through
`scripts/render-benchmark/build.sh` and `scripts/install-dev.sh --probe build/render-benchmark/probe.apk`.
Keep agent-device idle during these serial-scoped input fixtures:

```sh
scripts/render-benchmark/regression.sh build/stage2-editor/regression-fresh
python3 -I scripts/render-benchmark/suggestions.py build/stage2-editor/suggestions-fresh
python3 -I scripts/render-benchmark/editor.py build/stage2-editor/editor-fresh BASELINE/editor.json
```

The editor runner pins one word-delete long press below its repeat delay, then backspaces
through a combining cluster, surrogate emoji and an entire family ZWJ. It also reverts an
autocorrect, types after a period in a sentence-capitalisation field, checks URL/email/password
fields, and has the probe replace text and move selection while a composing span is live.
Each renderer starts a fresh keyboard process in incognito mode: the revert fixture's
session-local keep-as-typed word cannot contaminate the following renderer. Preferences and
accessibility are restored in `finally`. Only hashes/counts/offsets are retained.

## Emulator result (2026-11-13)

Pre-change `09194151` and the new normal APK were checked on owned `aw_keyboard`, runtime
`emulator-5556`. **Compose and Rust both match**:

| Fixture | SHA-256 |
|---|---|
| 300 key presses | `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` |
| Eight e holds (the configured number hint) | `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555` |
| Autocorrect on space | `6e5ce6afa65bc328ed7ef2585ac6077dca716587a929ef60778a364b5680051c` |
| Suggestion pick | `106a5842fc5fce6f663176285ed1516dbb1e3d15c05abab12fdca46d60b539b7` |
| Autocorrect revert | `57a499486a01b8c5a2f876e89e000a0b83143e42b637e7675ec8fa2cc211dcfc` |
| Capital after a period | `6724d72c3ce41e988d31cecf7fa62f1193217e5c7011632e180582f99c8305c8` |
| URL / email / password | `2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824` |
| External edit while composing, then another key | `0ee338f146040b7750bb7cdde2f74e5e5bce965ac9533888f55d3dc2356b8692` |

The word-delete/backspace trace has UTF-16 lengths/cursors **24, 23, 21, 20, 18, 17, 6, 5**;
it deletes the combining cluster, surrogate emoji and full 11-unit family ZWJ. Every hash and
selection/composing offset at each of these checkpoints matches the old build on each renderer.
The final five-unit digest is `8ed3f6ad685b959ead7022518e1af76cd816f8e8ec7ccdda1ed4018e8f2223f8`.
The external edit begins with a live composing span, changes both the text and selection,
and retains identical final offsets. Password candidate eligibility remains disabled by the
native field setup; the inherited editor can still retain a composing span independently of
whether suggestions are displayed.

Final `FATAL|AndroidRuntime` logcat matches: **0**. Cleanup verified prototype off, Always day /
Floris Day, original accessibility and fixture preferences restored, project IME selected,
DND off, streams 1/2/3/5 at zero and streams 4/10 at their allowed minimum of one. The probe was
removed and `device_close(shutdown: true)` successfully shut down only the owned runtime.

## Source accounting

Nonblank production `.kt`/`.rs`/`.java`, excluding test/reference sources, benchmark sources,
generated `chars`/Latin character tables and the test-only C++ oracle crate:

- **930 Kotlin lines removed net**, **1,805 Rust lines added net**, Java unchanged.
- Totals: 54,027 Kotlin, 42,432 Rust and 321 Java.
- `AbstractEditorInstance.kt` and `EditorInstance.kt` are now below 500 lines, as is every changed
  source file. Input attributes and IME options cache one decoded native result per field rather
  than crossing JNI for every property read.

Full evidence is under ignored `build/stage2-editor/`; no screenshots, dumps, replay outputs,
local settings or build artifacts are tracked.

## Final local gate

The exact required command completed with **Gradle process exit 0**:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

The app has **428 JVM tests, zero failures/errors/skips**. Formatting, clippy with `-D warnings`,
the full C++ oracle and debug assembly pass. App lint has only `AndroidGradlePluginVersion`,
three `NewerVersionAvailable` notices and `ChromeOsAbiSupport`. No warning suppression, lint
baseline or skipped test was added. An earlier check overlapped my dependency edit and rustdoc
used its old dependency graph; that result was discarded. This final gate used frozen sources.

## Fresh SSD acceptance (2026-11-14)

The new `aw_keyboard_ssd` guest opened as `emulator-5556`; the old AVD and
`emulator-5554` were untouched. Evidence is under ignored
`build/device-acceptance-2026-11-14/`. The initial whole-codebase gate exits zero
with 802 JVM tests, no failures/errors/skips and the five permitted lint warnings.

The checksum-verified `dc7256e5` APK retains SHA-256
`c2949714060dba3b70832eef3c3d3895e0a1eca3c00910a67708574dd4f77b7a`.
Its unchanged URL/email/password block and `95c76b21`'s same block pass on both renderers:
five units, digest `2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`.
The old email failure is classified as an old-run environment/input artifact; its
exact trigger remains unproved. No digest or field policy was changed.

Before the concurrency fix below, the complete `95c76b21` editor wrapper passes on both renderers in
`editor-after-transport/`, including every cluster/backspace checkpoint, autocorrect
revert, sentence capitalization and external edits during composition. Earlier old/main
runs stopped at Rust caps preparation and remain separate failed evidence. The T3 UI
transport now preserves the original BROWSABLE settings-intent category; its control
finds the requested settings row only with that category. No assertion was relaxed.
Canonical stores, the fresh preference-file SHA-256 and accessibility values restore
exactly around each of these later wrappers.


### Regression found by the extra-panel replay

The new run exposed an actual editor protocol race: two editor plans could interleave
while the first plan suspended for word analysis. The second plan consumed/replaced the
pending analysis; the first resumed into `IllegalArgumentException: no pending editor
analysis`. The sanitized crash record has 64 literal FATAL/AndroidRuntime lines and
identifies `EditorNative.call`, `EditorPlanExecutor.execute` and start-input-view.

Fix `bfb3a4c2` serializes each complete query/reply exchange and its platform effects
with a coroutine mutex on the session. JNI calls still copy packets and release the
native mutex before Android callbacks. The real executor/native-session JVM test pauses
the first word reply and starts a second plan; it fails before the fix and passes after.
Removing the transaction lock deliberately produces one fresh assertion failure.
The source is then restored byte-for-byte, SHA-256
`1f84cb5ecdfa3c64dc770b772994416e4dc38a5b277ad935c15270993968339f`;
the restored editor suites pass. No editing rules or expected digests change.

The full post-fix local gate exits zero with 803 JVM tests and no failures/errors/skips;
lint retains only the five permitted warnings. An earlier build reused stale Android
Cargo workspace artifacts from the historical control build and failed compilation.
Isolating that cache and rebuilding the workspace gives the successful gate; no source
or check is weakened. Evidence: `postfix-gate-isolated.log`,
`editor-concurrency-before.*` and `editor-transaction-mutation/`.

All 17 extra replay records match the prior accepted records after this fix, including
both panels, subtype/one-handed workflows, invalid-finalize, GIF search and clipboard.
Each wrapper restores the exact fresh preferences and canonical stores; the clipboard
controller also restores its on-device Room database/WAL/SHM hashes exactly. A separate
learning/unlearning run passes on both renderers: learned-store hash changes from
`7848a5161495f8822d228eb8e2d298b38359d3e229a79a5db04ea5babf7276f0`
to `391e5fa71fb3adf58416baf9fd16c46ec33162a73f0e35f315ad57f5171fa5d5`
and returns exactly; password candidate and inline counts are zero, and incognito does
not alter the stores. These are fresh post-fix runs in `postfix-extras-gutter/` and
`postfix-learning/`.

The agent-device pan transport leaves Android UiAutomation attached, suppressing the
probe accessibility observer. The first post-fix extra run stopped at this observer
preflight (zero runtime failures); it did not exercise the panel fixture. Closing and
reopening only this owned T3 device cleared that attachment. With explicit user approval,
`panels.py` retains its original fixed gutter adb swipes; other navigation, taps, holds,
text and screenshots remain on the returned agent-device command. The subsequent 17-record
pass uses this transport. No arbitrary adb UI driving is added.

The post-fix normal JNI-free build passes all Compose editor checkpoints in
`postfix-typing-strip/editor/compose/` and the entire Rust replay separately in
`editor-rust-diagnostic/editor.json`. Both retain the same URL/email/password digest.
The combined wrapper again times out preparing Rust's empty caps field, before its text
assertion. That preparation timeout was also observed on the pre-change build and is
retained as unresolved evidence; the individual renderer passes do not erase it. Numeric
controls in `caps-state/` and `normal-caps-state/` show the correct caps input type and
uppercase state after a text-to-caps transition on both fixture and normal APKs; a direct
cold caps control in `caps-diagnostic/` stays lowercase. No digest is re-baselined and no
production change is inferred from these differing preparation paths.


### Complete wrapper follow-up

`editor-meta-after-automation-reset/editor.json` subsequently passes the **unchanged
complete wrapper on both renderers**, using the current JNI-free main sources and a
test-only numeric metadata receiver. Every fixed digest and UTF-16 selection/composing
offset matches. Both caps preparations report input type 16385, cursor caps 8192,
shift state 2, auto-capitalization enabled and zero editor units. No microphone is opened.
The observer first failed to bind after manual agent snapshots; closing/reopening only
the owned device cleared UiAutomation before the successful run. Earlier lowercase
caps preparation failures remain failed evidence, including a later Compose control;
the precise trigger of those earlier input-state differences is not established.
The successful full wrapper closes the replay obligation without changing a digest,
adding retries to an assertion, or attributing an unproved production cause.


### Final fresh-device gate and hold

The final exact whole-codebase gate exits **0** (4m 48s): **803 JVM tests**, no
failures/errors/skips; Rust checks/oracle pass and only the five allowed lint warnings
remain. Normal debug is installed; final receipts verify exact baseline preference/store
hashes and accessibility null/0. Prototype off, Always day/Floris Day, owned IME and mic
permission pass before a final volume-verifier assertion. Disposable APKs/private audio
are removed; models remain. A normal launcher intent fails, and its one advised debug
retry also fails. Device work stops under the requested rule. **Still owed:** corrected
volume/DND readback, literal `FATAL|AndroidRuntime` final scan and shutdown. The device
is not claimed closed. See [the complete hold and diagnostic](stage2-tts.md#final-gate-and-device-hold);
this does not alter the package-specific acceptance results above.
