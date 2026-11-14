# Stage 2 package 16: NLP orchestration

The Java-free `nlp` crate owns provider/field/settings eligibility, request generations and the
8 ms coalescing window, stale-result rejection, candidate ordering and deduplication, current
input autocorrect eligibility, learning plans, inline-autofill sequences and pinned ordering,
and shared smartbar expansion. It uses the existing native suggestion, dictionary and personal
stores. The decoder, typed-word entry, scoring, case, threshold and learned-word store/learner
remain in `suggest`; they were already Rust after A1.

Kotlin retains provider lifecycle/execution, coroutine jobs/delays, Android clipboard and regex
APIs, dictionary/file I/O, ICU iterators, Android inline inflation/views and observable UI DTOs.
Both renderers subscribe to `NlpState.candidates`; neither constructs an alternative candidate
list. The smartbar consumes one native decision computed from candidates, inline ids and selection.
Glide preview refresh timing and the preview/commit candidate slice are native plans too;
Kotlin passes monotonic timestamps and executes the returned index range and commit index.
While a word request is pending, stale words are hidden and the expansion binding is held until
completion; an empty completed result expands actions. This avoids a toolbar flash and repeated
preference writes on each keystroke.
The existing `render::smartbar` policy still owns inline/dictation/shared-row presentation
precedence and the native candidate hit/semantic reservation; it is not duplicated in `nlp`.
Its existing smartbar reference/layout suites remain in the full gate.
The Compose click closure passes the displayed object to the native eligibility check instead
of indexing a potentially newer list. Stale candidate taps and removals are rejected as well as
stale result publication. Glide previews and commits use a native external-request generation.

The remaining provider composing rule (including the leading colon), name spelling decision
and swipe user-word union moved too. Name and unknown-word lookup operate directly on native
stores/dictionaries, with no per-word managed probability callback. The synthetic-dictionary
swipe factory survives only in test sources. Kotlin retains immutable DTO/cache identity checks.

## Protocol and lifetime

One synchronized `NlpSession` owns a `Mutex<Session>` through `awnative`. Explicit close and
Cleaner release the allocation once; closed calls throw. The logic crate forbids unsafe code.
All unsafe blocks are confined to JNI and carry SAFETY comments. No native lock spans platform
work, inflation, coroutine suspension or a callback into Kotlin.

A request transfers a complete UTF-16 editor snapshot, raw input flags/options, offsets,
subtype/provider identity, settings and privacy. The session uses complete equality, not a
text hash or a wall-clock timestamp. A new request invalidates older generations before launch.
Kotlin cancels the old executor job and executes the returned delay/provider flags. Completion
transfers the current editor identity together with result ids/metadata in one message: native
validation precedes publication, even if a cancelled provider finishes anyway. Equal-millisecond
requests and colliding Java text hashes have dedicated tests. Field restarts invalidate external
requests and inline inflations even if all visible editor fields are identical.

Candidate ids name managed objects; Rust retains their text/kind/autocommit metadata and owns
visible order. Stable first occurrence dedup preserves decoder order and the kept-as-typed entry.
Emoji precede words; an enabled prefixed emoji result replaces word results. Eligible clipboard
results take precedence. Android regex spans are supplied together; native code selects the
unique non-overlapping matches and applies expiry/sensitivity policy. Inline results contain
only opaque ids and pinned metadata. They are stably pinned-first and apply only to their live
sequence; empty, superseded and timed-out sequences cannot publish late views. Platform autofill
remains independent of word suggestions, including a legitimate password autofill response.

Learning produces one finished/swiped operation with the selected context/position. Kotlin
executes that operation against the existing native learner/store and writes its serialized
bytes after the native save-coalescing plan. Privacy, raw editors, language, dictionary readiness,
settings, password/no-suggestions and URL/email restrictions are decided in Rust. Undo and
kept-as-typed persistence retain the existing native store behavior.

## Reference and match proof

The original manager, Latin provider, inline adapter, clipboard provider, glide request path,
swipe-word wrapper and candidate click path are retained as test-only `Reference*.kt` sources.
`ReferenceNlp` extracts their pure rules into an executor-independent contract. Four deliberate
safety changes are explicit in that contract rather than claimed as legacy equivalence:

- A request is accepted only for the latest live generation and exact editor identity; the old
  manager compared its timestamp only with the last *completed* request. An older request could
  therefore publish while a newer one was still pending, and equal timestamps could lose a result.
- Password/no-suggestions and disabled word-suggestion settings gate all word/emoji candidates.
  The old manager could still execute the word provider when only emoji suggestions were enabled,
  and clipboard/emoji assembly did not share the Latin provider's field restrictions.
- Repeated provider entries are stably deduplicated without reranking decoder output.
- Inline changes use the complete candidate/inline state for smartbar expansion. The old UI
  callback passed a null candidate list, allowing actions to expand over live candidates.

`NlpMatchTest` runs **50,000 events across 200 seeds**: rapid/equal-time typing, out-of-order
completions, empty/failing providers, cursor/selection and field changes, privacy/settings,
direct/glide-style lists, clipboard precedence, inline arrivals, picks and learning/save plans.
Every operation compares its return value and full native snapshot with the Kotlin contract.
`NlpLearningMatchTest` adds **50,000 events across 200 seeds**, executing finished/swiped/undo/reset
and kept-as-typed operations against the pre-port Kotlin learner and the native learner. It
compares serialized learned and blocked stores after every event, including rejected learning.

`NlpHarnessMatchTest` checks **54 public content traces and 2,056 prefix/cursor checkpoints**
using results from the real bundled dictionary. These cover the regression, suggestion, editor,
input, mode and glide-content fixtures, including emoji backspaces, external edits, field changes,
the long delete prefix and the new fast-pick/cursor cases. The additional panel, clipboard,
subtype/one-handed, GIF-editor and invalid-batch traces retain their no-suggestions editor context. They are content-level replays;
actual Android event timing and glide decoding are checked separately by the device harness.
Existing A1 decoder/pipeline/oracle, touch, editor and swipe tests remain in the full gate.
`NlpPureMatchTest` compares 50,000 composing-range decisions using ICU boundary metadata and
20,000 name decisions with/without a dictionary, plus the native unknown-word union.

The directed stale test injects an old result before the current result, again after it, then
attempts to pick its id: each is rejected. It also invalidates inline inflation across a field
change and an external glide request across an editor change. The production bulk-message test
uses `Aa` and `BB` (equal Java hashes) and equal timestamps, proving complete identity checks.
It also sends obsolete completion metadata after a new request: stale packets cannot roll the
context back or cancel the newer pending request. A shared native acceptance predicate guards
both the transport and result application.
Clipboard completion and synchronous autocorrect registration also require the request,
completion and native session to agree; they never configure context from a result packet.
A directed test injects obsolete clipboard/autocorrect metadata while newer work is pending,
checks the entire snapshot is unchanged, then successfully applies the newer word and clipboard.
Clipboard requests have their own native generation: a late response is rejected even when
two primary-clipboard changes happen within an otherwise identical editor. The seeded sessions
also inject these out-of-order clipboard completions.

The learning comparison caught a missing separating space on a swiped-word context while a
composing span existed; the native plan was corrected. Initial reference setup incorrectly
started with a configured editor, unlike a new native session; the comparison caught that test
initialization error too. The first post-change device replay also exposed an adapter mismatch: the editor flow publishes
acknowledged state, while `activeContent` reads the latest native prediction. Starting from the
flow payload and validating against the prediction correctly rejected those requests, but left
the strip empty. The notification now triggers a read of the current prediction; the real-editor
`NlpEditorHandoffMatchTest` preserves that delayed-acknowledgement case. The replay also caught
fresh synchronous autocorrect objects lacking native ids. They now receive a one-shot native
identity checked against the exact request/current editor, without weakening stale UI-pick checks.
The device comparison also preserved the old password composing spans by separating the
editor's global composing switch from native candidate eligibility. A password still has no word
candidates or learning; that does not require changing its InputConnection composing metadata.
The adapter's `NlpBindingMatchTest` also passes providerless glide candidates through the actual
shared binding: they remain word candidates, so enabling emoji alone cannot enable glide words.
It checks native deduplication and password blocking through that same managed-object projection.
All reference comparisons pass; device timing/setup exceptions are recorded separately below.

## Deliberate mutations

`scripts/nlp-mutations.py` changes actual production Rust, runs JNI/JVM comparisons, requires
fresh `AssertionFailedError` reports, restores each source byte-for-byte in `finally`, and reruns
the restored suites. Compilation failures do not count as detection.

| Mutation | Failing assertions | Restored |
|---|---:|---|
| Accept an old request generation (final shared guard recheck) | 4 | Yes |
| Invert candidate text dedup | 2 | Yes |
| Allow password suggestions | 5 | Yes |
| Expand when either candidate/inline list is empty | 8 | Yes |

The original full four-mutation run caught 3/2/4/4 assertions. A final full run after adding
pending smartbar-state coverage caught 4/2/5/7 assertions and passed the restored comparisons.
The delivered implementation's final full run, including clipboard generations and glide plans,
caught **4/2/5/8** assertions; all four source edits were restored.
After strengthening the bulk
transport to reject obsolete metadata before context changes, its shared generation guard was
mutated again: four assertions failed, including production bulk-message rejection. It was
restored and the reference suites passed. Dedup/password/expansion still use the same tested
functions. The runner accepts an optional mutation selector for this focused recheck.

## Reproduction

```sh
./gradlew :lib:rust:testDebugUnitTest --tests '*Nlp*MatchTest' \
  :app:testDebugUnitTest --tests '*Nlp*MatchTest'
python3 -I scripts/nlp-mutations.py build/stage2-nlp/mutations-fresh
```

Open owned `aw_keyboard` through T3, use its returned runtime serial for `ANDROID_SERIAL`, and
install only through `scripts/install-dev.sh`. Build/install the disposable probe as described
in [the input proof](stage2-input.md). Keep agent-device idle during scripted adb input.

```sh
scripts/render-benchmark/regression.sh build/stage2-nlp/regression-fresh
python3 -I scripts/render-benchmark/suggestions.py build/stage2-nlp/suggestions-fresh
python3 -I scripts/render-benchmark/editor.py build/stage2-nlp/editor-fresh BASELINE/editor.json
python3 -I scripts/render-benchmark/input.py build/stage2-nlp/input-fresh BASELINE/input.json
python3 -I scripts/render-benchmark/modes.py build/stage2-nlp/modes-fresh
python3 -I scripts/render-benchmark/input-glide.py build/stage2-nlp/glide-fresh BASELINE/glide.json
python3 -I scripts/render-benchmark/nlp.py build/stage2-nlp/nlp-fresh \
  --baseline BASELINE/nlp.json --agent '<device_open agent-device command>'
python3 -I scripts/render-benchmark/compare-nlp.py build/stage2-nlp/nlp-fresh
```

The learning runner additionally needs `:app:assembleDebugAndroidTest -PrendererFixture=true`,
with its test APK installed using `scripts/install-dev.sh --probe ...`. Run
`python3 -I scripts/render-benchmark/nlp-learning.py OUTPUT`. Its test-only instrumentation
returns candidate/inline counts and invokes the existing unlearn operation for the fixed
synthetic word. No production receiver or logging hook is added. Count-file publication is
atomic and startup waits for a receiver handshake; incomplete earlier harness attempts are
excluded. A warm settings deep link sometimes had an empty navigation host (main thread idle,
no crash/ANR); renderer preparation records its fallback to the same persisted prototype flag on
a fresh process. Actual native virtual keys and all digest assertions are still required. This
fixture recovery does not claim acceptance of warm settings navigation. The script verifies the controller's completion marker, not merely `am instrument`'s
shell exit status.

The candidate observer refuses arbitrary nonempty editors. Only its fixed `helo` / `the `
fixtures permit IME-strip-only capture; labels are SHA-256 values, and no editor tree is read.
The comparator checks ordered labels, enabled click actions and target geometry; six comparator
controls distinguish the fixed expanded-action buttons from candidates and reject wrong labels,
missing clicks, displaced targets and password candidates.
Requested screenshot pairs preserve actual synthetic fixture pixels and remain under ignored
`build/stage2-nlp/` with private store/settings snapshots and all other evidence.

## Source accounting

The tracked nonblank production-source count excludes test/reference sources, benchmark code,
generated character tables and the test-only C++ oracle. The starting commit `4786041e` contains
53,518 Kotlin, 43,404 Rust and 321 Java lines. The final count is 53,469 Kotlin, 44,101 Rust and
321 Java: **49 non-test Kotlin lines removed, 697 Rust lines added**, and 45.05% Rust overall.
This package moves decisions rather than claiming that the 90% target has been reached.
Every changed source file remains at or below 500 lines.

## Emulator acceptance

The owned Android 16 `aw_keyboard` device returned runtime id **`emulator-5556`**. All installs
used `scripts/install-dev.sh`; `emulator-5554` was never used. The baseline APK is from
`4786041e`. The delivered source is compared on both Compose and Rust, using Floris Day.
The installed APK digest, full gate exit status, mutation reports and private state snapshots
remain under ignored `build/stage2-nlp/`.

Accepted primary evidence is under `build/stage2-nlp/acceptance/`: `nlp`, `regression`,
`suggestions`, `editor-atomic`, `input-idle`, `modes`, `glide` and `learning`.
`comparison.json` compares **12 complete primary manifests** with the pre-change results,
including every editor/input digest and selection/composing checkpoint, all eight mode digests,
all five glide checkpoints, and the new NLP/learning fixtures. Every comparison matches.

| Fixture | UTF-16 units | Identical before/after and on both renderers |
|---|---:|---|
| 300 keys | 300 | `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` |
| Eight popup holds | 308 | `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555` |
| Autocorrect | 4 | `6e5ce6afa65bc328ed7ef2585ac6077dca716587a929ef60778a364b5680051c` |
| Candidate pick | 4 | `106a5842fc5fce6f663176285ed1516dbb1e3d15c05abab12fdca46d60b539b7` |
| Fast typing with a mid-word pick | 7 | `533f70950b1b957e6874cf105faef3431321a32fbe4c1dd80f6859e9d44b8a2c` |
| Autocorrect then cursor move | 5 | `4eea2b0b883f8ae8b534b943b8bbaf052df1ad6b343d7d7b6691a539c1ba4fdb` |
| Password typing, zero word candidates | 4 | `f4e454f802b88d2f64168ff1742e8cf413fd677d38b87cbefb45821f8981b912` |
| Original two-second delete | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| New two-second delete: 1,088-unit start | **1,055** | `07b9f1e0ce48a5235a5c48458ecd255a131e004a2e5762aa9e59a701434d9055` |

The synthetic learning fixture changes the learned-store digest from `14128737…` to
`11149ca2…`; unlearning returns exactly to `14128737…` on both renderers. Incognito leaves
the store unchanged. Password counts are zero candidates and zero inline suggestions.
Full digests are in `learning/learning.json`; no store contents enter reports or logs.

Six actual before/after candidate-strip pairs are in `acceptance/pairs/`, named
`{compose,rust}-{typing,after-autocorrect,password}.png`. Before is on the left. All were
visually reviewed; labels, order and the password action controls match. The Rust candidate
strip dump/comparator reports **zero bug verdicts** in all three states, including enabled
click actions and target geometry. This is accessibility-tree evidence, not a TalkBack speech test.
The emulator offered **no inline suggestions** in these fixtures; inflation/live autofill is
not claimed. Inline eligibility, sequence rejection, timeout, pinned order and smartbar
interaction are covered by the JVM reference comparisons only.

### Timing and harness limits

Builds must be idle during timed input comparisons. A trial overlapping Gradle work left
1,058 units after the long delete and did not record the Search action; it remains under
`acceptance/input` and is excluded from the controlled before/after comparison. The idle run
passes every input assertion and exactly matches the baseline's 1,055-unit remainder on both
renderers. The long fixture measures delivered repeats over wall-clock time, including scheduler
effects; its sensitivity is precisely what the original empty digest lacked.

Two editor-preparation runs timed out waiting for uppercase labels in an empty sentence-caps
field during overlapping build work. The metadata-only controller reported the expected input
flags, cursor caps and automatic shift in its controlled run. The complete ordinary
`editor-atomic` replay subsequently passed both renderers and all old checkpoints. No caps-specific
production fix or proof of the timeout's cause is claimed. A separate partial-JSON read exposed
non-atomic publication in the disposable probe; `digest.json` now uses a closed temporary file
and rename. This fixes the harness data race without changing a digest or weakening an assertion.
All incomplete runs remain separate from the accepted manifests.

The additional emoji/recents/editing/action-arrangement, clipboard paste/pin/unpin/delete/clear,
subtype/language-key/one-handed, invalid-finalize batch and no-key GIF fixtures also pass on both
renderers. **17 additional fixture records** in `acceptance/extras` match
`before/extras-accepted` exactly. Invalid finalize leaves batch depth zero; the no-key GIF query
has three units while its editor stays empty. These are included in the public JVM trace corpus.

The required local gate, with Gradle's own exit status captured before any reporting command,
exits **0** (`acceptance/final-gate/gate.exit`):

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

The JVM reports contain 741 tests, zero failures/errors/skips. Rust formatting, tests, clippy
with `-D warnings`, and the C++ oracle checks pass. The five lint warnings are exactly one
`AndroidGradlePluginVersion`, three `NewerVersionAvailable` and one `ChromeOsAbiSupport`.
No suppression, baseline or ignored test was added. Hosted CI is not used as this acceptance gate.

Cleanup restores the original **286-byte learned file** and **zero-byte blocked file** exactly,
including presence and SHA-256. Private clipboard/Room snapshots and accessibility settings are
restored. The final preference comparison reports **zero differing keys**, allowing only the
requested prototype-off, Always day and Floris Day overrides. Our IME is selected, DND is off,
streams 1/2/3/5 are zero and streams 4/10 are at their permitted minimum of one. Measurement tags
are reset, test-only app files are removed, and the disposable probe/test APKs are uninstalled.

The first literal log scan preserved six non-error `AndroidRuntime` launcher notices from
`am instrument` (normal startup and exit code zero), with no fatal exception. After instrumentation
finished, a fresh log window replays the complete new NLP fixtures on both renderers, matches the
baseline again, and reports zero accessibility bugs. Cleanup then repeats. The requested literal
`adb logcat -d | grep -E 'FATAL|AndroidRuntime'` check has **zero matching lines**; adb exits zero
and grep exits one for no matches. Both the earlier launcher notices and final empty result remain
in ignored evidence. Device close with `shutdown: true` succeeds; device inventory confirms
`aw_keyboard` is no longer open and is **not booted**.

## Canonical emulator store (package 12 recovery)

The dedicated emulator's accumulated synthetic learned words are now replaced by the committed
`scripts/render-benchmark/store-fixture/` before every wrapper. `nlp-learning.py` owns a
`fixture_session` directly; use `store-session.py OUTPUT/stores -- COMMAND...` around other
learning/extra wrappers, and `typing-suite.py OUTPUT [--baseline BEFORE]` for ordinary typing.
Each session force-stops the IME before installing, records and verifies fixture hashes, and
reinstalls/verifies them in `finally` after success or failure. A stalled restoration fails and
leaves a pending record. See [the recovery proof](stage2-dictation.md#package-12-canonical-test-store-recovery-2026-11-13)
for full hashes and exact configuration recovery. The previous historical store-size/hash
records above do not define the new canonical baseline.
