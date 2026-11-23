# Stage 2 package 12: Pocket TTS and reader policy

The held-run account below is historical. Fresh SSD results are recorded in the dated
acceptance section at the end; earlier evidence is not relabelled.

**Implementation is present; device acceptance is incomplete.** The owned emulator stalled
while recording the unchanged-build canonical-store baseline, before any new APK was installed.
T3 shutdown failed twice after the advised refresh. The user explicitly permits commit/push
under this device-tool failure condition if the full local Gradle gate passes. This report
must not be read as byte-identical PCM, leak, timing, screenshot or final-cleanup acceptance.

## Recovery and baseline boundary

See [the retained recovery history](stage2-dictation.md#package-12-canonical-test-store-recovery-2026-11-13)
for the full held/live hashes and exact preference/accessibility/permission restoration.
The committed fixture has one synthetic learned word and an empty kept-as-typed file.
`store-session.py` force-stops, installs, hashes and records before every wrapper and repeats
that in `finally`. Every wrapper restoration is checked before the next wrapper begins.
Its failed-child control restored both hashes. Unavailable guests fail and leave a pending
host record; no `finally` implementation can restore files on an unresponsive guest.

The fresh unchanged dc7256e5 production-source build (895 byte-identical source files) is
66,265,488 bytes, APK SHA-256
`c2949714060dba3b70832eef3c3d3895e0a1eca3c00910a67708574dd4f77b7a`.
Both renderers pass the canonical primary and suggestion baselines, retaining all four digests:

| Fixture | UTF-8 SHA-256 |
| --- | --- |
| 300 keys | `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` |
| 308 units / eight popup holds | `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555` |
| Autocorrect | `6e5ce6afa65bc328ed7ef2585ac6077dca716587a929ef60778a364b5680051c` |
| Candidate pick | `106a5842fc5fce6f663176285ed1516dbb1e3d15c05abab12fdca46d60b539b7` |

No re-baselined digest is needed for these four. The unchanged extended editor replay failed
its email assertion. The fixed coordinates fall within the captured corresponding key bounds;
no cause or production fix is claimed. Its outer canonical restoration passed. The next run
stalled during initial store verification before any replay. No post-change device digest
comparison exists. The prior editor/input/NLP/glide/mode/extra and dictation proof remains
historical evidence, not a new JNI-free acceptance run.

## Deterministic long delete

[The debug hook and exact oracle](stage2-input.md#timestamp-driven-long-delete-oracle-package-12)
use dispatched key down/up times, each repeat attempt, its native-admitted/skipped outcome and coroutine wait deadlines.
The 300 ms hold and production repeat deadline must agree exactly with the Android schedule.
The delivered count, exact fixed-source prefix hash and selection must agree with the eligible
admitted actual ticks, including a tick admitted before UP whose log/delivery races release. A delayed coroutine wake does not create missed synthetic grid repeats. The JVM
fake-clock tests are unchanged. Five host oracle controls cover jitter, one missing/extra
delete, release/admission races, missing/up-before-attempt boundaries and a 50→51 ms interval mutation.
The actual production +1 ms mutation is tested against the unchanged JVM lifecycle reference.
The hooked APK never reached the device, so neither renderer has accepted this new device check.

## Native synthesis and lifetime

`reader` remains Java-free and forbids unsafe. `sherpa-ffi` owns the exact v2.0.1 C API layouts,
including all trailing fields of unused model families, generated from the checksum-pinned
header. Independent C compiler sizes/alignments/offsets are checked by Rust tests. The
additional exports are resolved from the same retained `libsherpa-onnx-c-api.so`.

The Pocket config retains lm_flow.int8, lm_main.int8, encoder, decoder.int8,
text_conditioner, vocab.json and token_scores.json, Loona's original reference PCM/rate,
four CPU threads, 50 cached voice embeddings, max_num_sentences=1, silence_scale=0.2,
speed=1.0, speaker id=0 and five flow steps. No extra generation parameters are introduced.
Kotlin performs model file checks/reads. Rust parses Loona's PCM16 WAV and owns its samples.
UTF-16 conversion preserves supplementary characters and replaces lone surrogates, as the
[pinned JNI path](https://github.com/k2-fsa/sherpa-onnx/blob/v2.0.1/sherpa-onnx/jni/offline-tts.cc)
and its `JStringToUtf8` helper do. The C API rejects embedded NUL in an utterance because its
text argument has no length; this input has no acceptance evidence and is not silently truncated.

Rust's `Generator<Owned>` lends the model to the synchronous synthesis call. Its lease
outlives every PCM callback, the blocking AudioTrack write and destruction of C generated audio.
Cancellation invalidates the generation atomically. Close requests stop, removes admission,
and frees an idle model immediately or defers physical free until the active lease ends.
The JNI token registry clones an Arc and unlocks before generation; no Java/native lock is held
across a callback. Close from inside a callback therefore cannot deadlock or free its buffer.
Cleaner closes tokens idempotently; calls after close fail instead of dereferencing a pointer.

The [pinned Pocket implementation](https://github.com/k2-fsa/sherpa-onnx/blob/v2.0.1/sherpa-onnx/csrc/offline-tts-pocket-impl.h)
invokes chunk callbacks serially on the generating thread, and frees each output tensor after
its callback returns. `awnative` creates a borrowed direct ByteBuffer; Kotlin supplies a read-only
view and completes/copies it before return. No callback or PCM pointer is retained. AudioTrack's
byte-count result is converted back to sample frames for the unchanged playback clock.

## Reader decisions and managed boundary

Rust now owns the remaining frame-based play queue, first-heard/finished events, clear/stop
counts, system-TTS progress-id parsing and stale/duplicate callback admission. Kotlin maps
opaque ids to managed callbacks and runs those after native calls return. Existing sentence
chunking stays in Rust with Android BreakIterator supplying the same sentence lengths.

The full depth-first walk, traversal limits, password/editable pruning, sideways hidden-node
pruning, parent fallback when children contribute no text, WebView classification, scroller
choice and in-list membership now run in `reader::walk`. One native walk requests plain node
facts through an Android adapter. Kotlin only fetches AccessibilityNodeInfo data/children,
retains node/callback ownership and executes reveal/scroll actions. No text enters logs.
The existing follow-along geometric decisions were already Rust; turn timing/count, 40 ms playback-head polling and two-block look-ahead now come
from Rust too. Playback-frame events retain exact highlight/reveal onset semantics. Existing
speaker-key stop/start/top policy remains in Rust; the C generator now owns cancellation.
No new skip UI or word-highlighting feature is introduced.

Kotlin retains AudioTrack, audio focus, Android/system-TTS API calls, accessibility service,
coroutine/executor scheduling, screen/window/IME integration, model file I/O and managed callbacks.
The obsolete production ReleasedOnce owner moves to test-only ReferenceReleasedOnce, with
its tests retained. Frozen ReferencePlayQueue, ReferenceScreenWalk, ReferenceSpeaker and
ReferenceVoiceEngine preserve replaced code/decisions in test sources.

## Differential checks and deliberate mutations

- 50,000 playback/frame events over 200 seeds compare pending starts, finished counts,
  sizes and stop/late-frame histories against frozen Kotlin. All nine existing play-queue
  tests remain.
- 50,000 system-TTS events over 200 seeds compare malformed/last/duplicate/stale progress ids,
  starts, ends, sizes and stop effects against frozen Speaker decisions.
- 10,000 synthetic node trees over 200 seeds compare the complete selection/order/scroller
  packet; 300-deep and 20,100-node fixtures preserve both original traversal limits.
- Existing ReaderRulesMatchTest and ScreenRulesMatchTest retain seeded chunking, page,
  speaker-key, geometry and timing boundaries. TtsChunkMatchTest adds the committed device
  sentences and long/exact-boundary sizes to the Kotlin/Rust comparison.
- Native fake-owner tests check that close cancels before free while an active callback still
  holds its buffer, plus 200 cancel/immediate-respeak ownership cycles. These establish the
  ownership contract; they are not a real-model device stress or a general native leak detector.

`scripts/tts-mutations.py` removes stale reports, requires actual assertion failures rather than
compiler failures, restores exact source bytes in `finally`, and reruns the restored test.
It mutates the production repeat interval, chunk boundary, stop-before-free request and password
screen rule. Results and assertion counts are recorded below after the final local run.

## File-only device proof runner (not executed)

```sh
./gradlew :app:assembleDebug :app:assembleDebugAndroidTest -PttsFixture=true
ANDROID_SERIAL=<owned-runtime> scripts/install-dev.sh
ANDROID_SERIAL=<owned-runtime> scripts/install-dev.sh --probe \
  app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk
ANDROID_SERIAL=<owned-runtime> python3 -I scripts/tts-proof.py OUTPUT
```

The test-only ReferenceVoiceEngine uses the old JNI path. `TtsInstrumentation` synthesizes five
committed synthetic sentences: short, long, punctuation, numbers and emoji/symbol. It writes
private PCM files only, comparing Float32 little-endian SHA-256, sample rate/count and every
callback chunk length/hash. Immediate re-speak checks its first chunk against the accepted
full-utterance chunk hash. It never constructs AudioTrack, system TextToSpeech or AudioRecord.
Ten alternating runs per path record start/first-audio/end monotonic timestamps, with model load
and file open/close excluded. First-audio is timestamped at callback entry before copies/hash/file I/O.
The elapsed synthesis interval includes callback hashing/file writes. RTF derives from that
interval / generated duration; no timed run overlaps a build.

Since phase 6 (2026-11-23) the reference voice is retired: the five sentences' audio is checked
against the golden tape `tts-pocket-loona`, which it wrote on an emulator, and the ten timed runs
are the Rust voice's alone ([phase6-kotlin-references](phase6-kotlin-references.md#what-happened)).

It also covers cancellation mid-sentence, 200 real-model cancel/immediate-respeak cycles after
20 warmups, and close from inside a callback. Native heap samples are retained every ten cycles;
the maximum of the final five samples must not exceed that of the initial five warmed samples.
A crash-free run alone is not accepted as leak proof. The runner verifies the plateau flag and
complete cycle counts. Actual PCM parity counts/hashes, time-to-first-audio/RTF and stress/leak
observations are **unavailable**, because device work stopped before installing this build.
Read-aloud stop/highlight screenshot pairs and JNI-free dictation rechecks are also **unavailable**.

## Final device state

Initial recovery logcat had zero literal FATAL/AndroidRuntime matches. Final logcat is unverified.
The last installed APK is the unchanged controlled baseline, with the disposable renderer probe;
no new TTS/hook build was installed. No model was installed or synthesized audio created in this
run. Final prototype/theme/preferences/accessibility/permissions/store restoration and shutdown
are unverified after the stall. The pending canonical record is retained for recovery. Do not
resume with the other session's emulator-5554. Full failure details are in the dictation report.

All four deliberate mutations were caught and restored byte-for-byte:

| Mutation | Fresh assertion failures | Mutant exit | Restored exit |
| --- | ---: | ---: | ---: |
| Production repeat interval +1 ms | 2 JVM | 1 | 0 |
| Chunk join `> max` → `>= max` | 1 JVM | 1 | 0 |
| Omit generator stop request before free/admission shutdown | 1 Rust | 101 | 0 |
| Omit password-node pruning | 1 JVM | 1 | 0 |

The stop mutation fails in the fake-owner ordering test, with no native model loaded and no
device crash. The timestamp-oracle 51 ms wait control also fails against its 50 ms reference.
The focused TTS comparison APK and test APK build exits 0. These compiled test paths have not
been installed or run after the device hold.

## JNI removal, APK size and source accounting

Normal builds do not put the sherpa AAR on the production implementation classpath. A typed
Gradle generated-source task extracts only arm64 C API and onnxruntime after `verifySherpa`
authenticates the entire pinned AAR and C library. Explicit comparison properties retain the
AAR/JNI oracle; the file-only comparison app/test APK build exits 0. Normal R8 retention rules
for sherpa Kotlin/JNI and the old callback are removed.

`scripts/tts-apk-check.py` verifies the finished normal APK: no sherpa JNI library, zero sherpa
Kotlin descriptors in either DEX, and both native payloads byte-identical to the pinned AAR.
The packaged C API SHA-256 remains
`0eadae93c4d595a26584d7eebde7b397b9a36d01eee3ebc7262e0d34df2fce10`;
onnxruntime is `33847ad43bffe204699fd4a27f7f3603452a8cdaf2f9a44983a0bc31ffcf2da1`.
Keeping these authenticated payloads skips Gradle's otherwise eight-byte ORT string-table/alignment rewrite.
A fresh package is used for size accounting: the incremental ZIP had an unreferenced 22 MB
old-entry gap after swapping payloads. Repackaging removes that stale space without source changes.

| Normal debug APK | Bytes |
| --- | ---: |
| Unchanged dc7256e5 production-source baseline | 66,265,488 |
| Final C API-only build | 61,650,231 |
| Net reduction (port plus JNI removal) | 4,615,257 (6.965%) |

Final APK SHA-256: `56372d29bd4787829453a7a370cad78ca1b41354395b29a9052640d6f64d47ce`.
The owned Rust library grows from 9,763,616 to 9,798,744 bytes; no new native dependency is added.

The documented nonblank, tracked, non-test source count is:

| Language | dc7256e5 | Final | Net |
| --- | ---: | ---: | ---: |
| Kotlin | 53,212 | 53,186 | -26 |
| Rust | 45,483 | 46,516 | +1,033 |
| Java | 321 | 321 | 0 |

Rust is **46.51%** of this counted source. The Kotlin reduction is net of new platform/JNI
adapters and the debug timing hook; it is not a claim that only 26 lines of policy moved.
Test-only references, generated character tables, benchmarks and the entirely test-only
latin-oracle crate are excluded exactly as in `rust.md`. Every changed source file is at most
500 lines. No suppressions, lint baselines or skipped/deleted tests were introduced.

## Final local gate

The final required invocation exits **0**, captured directly from Gradle, with **779 JVM tests**,
zero failures/errors/skips, Rust tests/format/clippy with `-D warnings`, and the full C++ oracle:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

App lint has exactly five warnings: one AndroidGradlePluginVersion, three NewerVersionAvailable,
and one ChromeOsAbiSupport. No other app warnings remain. The Rust Android library lint is clean.
The separate Python timestamp-oracle controls pass all five tests, including the release race.
The actual Gradle exit, test counts, lint ids, APK/hash inspection and source recount are under
`build/stage2-tts/final-gate5/` and `build/stage2-tts/source-count.json`. Private build artifacts,
models, downloads, audio, screenshots, dumps, settings and caches are excluded from all commits.


## Fresh SSD acceptance (2026-11-14)

The new `aw_keyboard_ssd` guest is the owned runtime `emulator-5556`. The old AVD and
`emulator-5554` are untouched. The initial whole gate passes 802 JVM tests; editor fix
`bfb3a4c2` adds a test and passes the whole gate with 803. Evidence is under ignored
`build/device-acceptance-2026-11-14/`. The previous pending record in
`build/stage2-tts/before/typing-retry/editor-stores/` belongs to the old guest and was
not replayed. This device has its own baseline: preferences SHA-256
`0e622df78757593b44492677be6295051f14dcd44314fe42b8e9231e1b2d17a4`, canonical
learned-store `7848a5161495f8822d228eb8e2d298b38359d3e229a79a5db04ea5babf7276f0`
and empty kept store `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`.
Each wrapper records and verifies exact preference/store hashes and accessibility restoration.

### Part A

Fresh post-editor-fix primary regression and suggestions pass on both renderers, with
all four original digests above: 300 keys, 308 units/eight holds, autocorrect and candidate
pick. Full input, timestamp-driven long delete, glide, all eight mode/password fixtures,
NLP, learning/unlearning and all 17 extra records pass. The long-delete trace admits 33
deletes per renderer, with exact 1,055-unit prefix hash and independent 50 ms deadlines;
see [the input record](stage2-input.md#fresh-ssd-acceptance-2026-11-14).

The unchanged email block passes on checksum-verified `dc7256e5` first and then main,
both renderers, with five-unit SHA-256
`2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824`.
It is classified as an old-guest environment/input artifact; the old trigger is unproved.
The combined editor wrapper initially fails caps preparation on old/main; a later complete
JNI-free main wrapper passes both renderers unchanged. Failed controls remain explicit in [the editor record](stage2-editor.md#fresh-ssd-acceptance-2026-11-14).
The fresh extra replay exposes and verifies the editor transaction fix documented there.
No expected typing digest changes.

### File-only PCM, timing and stress proof

The initial unseeded runner fails fixture 0: JNI returns 26,880 samples and the C API
32,640 at 24 kHz. A same-path control proves stochastic input: two JNI generations give
30,720/28,800 samples with different hashes; two C-API generations also differ in hash.
The pinned Pocket implementation defaults to seed -1. Fix `3b28d049` pins seed 42 on
both comparison paths, with a Cargo-feature-only JNI entry using the same production
lease/callback bridge. Normal builds omit that entry and retain the random default.
This is a harness-input correction, not a re-baselined PCM expectation. Reference and
native identities must still match exactly on every run.

The seed transport's Rust test passes. A +1 seed mutation produces one fresh assertion;
restoration is byte-identical, SHA-256
`2aa9be1bcc2f2ab68ed83b6e300425f62941c48a60c3d9ad434ddee3bbf11769`.
All three sherpa-ffi host tests and feature-enabled awnative clippy pass. The fixture now
persists heap observations before enforcing the unchanged plateau assertion.

`tts-seeded/comparison.json` passes all five Float32 little-endian PCM comparisons,
sample rate/count and every callback length/hash (32 chunks total):

| Fixture | Samples at 24 kHz | Chunks | PCM SHA-256 |
| --- | ---: | ---: | --- |
| 0 | 34,560 | 2 | `9b7a6cb7fcca1af92d05d9a7b9f23bed6a89dce2318173a511f03c2a2e74420e` |
| 1 | 345,600 | 13 | `0579b97b13c784e0037253fe20bce07f5b757b21851ed83f187fec013df8069b` |
| 2 | 205,440 | 8 | `947d897c2879eb33730f3fe6c7496285a9448e79c44b1b86dd8e074170a71f44` |
| 3 | 144,000 | 6 | `7653850753d8981b2a26c9389c96f884f481ace299b342cedb13c1cdb53efbc6` |
| 4 | 72,960 | 3 | `455429bf4e9739e7810637ced696b136a4c6e98e603af21f81a390c39e1a429a` |

Ten alternating runs per path execute with builds idle. JNI median first-audio is
261.38 ms (232.57–328.41); C API is 258.63 ms (238.02–335.51). Median synthesis RTF is
0.18854 / 0.18767 respectively. These overlapping small samples do not establish a speedup.
The measurement excludes model load/file open/close and includes callback hashing/file writes.

After 20 warmups, all 200 real-model cancel/immediate-re-speak cycles pass. Each cancellation
is mid-utterance and each immediate first chunk matches the seeded accepted full utterance.
All 21 native-heap samples are retained. Initial-five maximum is 609,582,400 bytes;
final-five maximum is 609,560,560, satisfying the exact plateau rule. Closing from inside
a callback and repeated close pass. This is the specified warmed-model plateau proof,
not a claim about every workload. No microphone or playback component runs in this proof.

### UI and dictation continuation

The checksum-verified old JNI build and main C-API renderer fixture each produce reading /
stopped screenshot pairs on both renderers (`reader-baseline-pill/`, `reader-main-pill/`).
Actual neural playback advances before the reading shot; the floating **Stop reading**
pill then yields IDLE, no neural track, and an empty masked editor. Old playheads are
4,896 frames each; main is 1,088 / 4,352 frames. Screenshots mask all source glyphs;
media stays at zero; Android enforces minimum one for accessibility/alarm. Earlier keyboard-stop-key controls remain failed evidence on
the old build (still READING after ten seconds), so this does not claim that key passed.
The accepted screenshot pairs use the floating stop control on both builds.

The normal debug build is reinstalled after the proof. Fresh file-only JNI/reference and
JNI-free production dictation both pass all seven fixtures, including production-listener
preview/final delivery, VAD, boundaries, flush and session states. The JNI-free APK contains
zero legacy JNI libraries/classes; APK-offset-aware process-map checks find no legacy JNI
in seven snapshots. All four fake-platform UI states pass without room-microphone access;
see [the dictation record](stage2-dictation.md#fresh-ssd-acceptance-2026-11-14).

### Still owed / limits

No core package-12 PCM, cancellation, timing, screenshot-pair or dictation replay remains
unrun. The earlier caps preparation trigger and old-build keyboard stop-key control remain
unresolved observations; neither is relabeled as a pass. Final gate and the cleanup hold are recorded below.


### Final gate and device hold

The exact whole-codebase gate exits **0 in 4m 48s** (`final-gate-restored.log`):
**803 JVM tests**, zero failures/errors/skips; Rust checks and full C++ oracle pass.
Lint has only AndroidGradlePluginVersion (1), NewerVersionAvailable (3) and
ChromeOsAbiSupport (1). The initial final attempt failed clippy because a public type
followed the seed test module; moving the declaration before the tests fixes it without
suppression. Four transport, four APK-map and five timestamp-oracle host controls pass.
Normal APK SHA is `cf9edb5fafd927f54dee12c3b78cbebe99e2cbba5c06821eede15fc61e547c4c`
(61,781,303 bytes). It has zero legacy JNI libraries/descriptors, no fixture receiver,
and no seeded comparison export. `install-dev.sh` installs it successfully.

Final restoration receipts verify preference SHA
`0e622df78757593b44492677be6295051f14dcd44314fe42b8e9231e1b2d17a4`, both canonical
store hashes and accessibility null/0. Prototype off, ALWAYS_DAY/Floris Day, owned IME
and microphone permission pass before the final volume assertion. Disposable probe/test
APKs, generated private PCM/WAVs and the two owned SAF archives are removed; models remain.

The agent-device launcher handoff fails with:

> Error (COMMAND_FAILED): Failed to launch com.google.android.apps.nexuslauncher
> Hint: Retry with --debug and inspect diagnostics log for details.

The single advised debug retry returns the same error, diagnostic `mwgg2phc-9fee4e5e`:
the normal MAIN/LAUNCHER intent does not resolve this Home-only activity. This was a
handoff-command mistake, not evidence of an app crash. No subsequent device calls run
once the retry failure is known. The cleanup runner had already started after the first
failure and stops at its own volume assertion: it incorrectly expects stream 10 to allow
zero, whereas the local muting receipt shows Android requires 1–15 and the existing helper
sets one. Media uses zero; neural reading uses USAGE_MEDIA. No room microphone is opened.

**Owed at handoff (completed in the next section):** corrected final volume/DND readback, the final literal unfiltered
`FATAL|AndroidRuntime` scan, and `device_close(shutdown=true)`. Shutdown is not attempted
because it would be another device operation after the requested stop. The guest is not
claimed shut down or finally logcat-clean. Earlier editor crash and failed test-probe
controls remain documented; no cleared idle window is presented as proof of the whole run.
Evidence: `final-hold.json`, `final-host.json`, `final-restoration/`.

### Final handoff completion (2026-11-14)

The orchestrator finished the three owed steps on `aw_keyboard_ssd` (emulator-5556) with adb
diagnostics and the T3 device tools.

**Volume and DND.** The first readback (`cmd media_session volume --get`) showed streams
0/1/2/3/4/5/10 at 3/5/5/5/6/5/6, so the device was **not muted** despite the earlier muting
steps. A direct `cmd media_session volume --stream 3 --set 0` printed success and left media at 5.
`cmd audio set-volume` set streams 1/2/3/5 to 0 and 0/4/10 to their minimum of 1. Both
`cmd audio get-stream-volume` and `media_session --get` read the new levels back. `zen_mode` was 0
(DND off) throughout. The emulator ran with `-no-audio`, so nothing reached the host speakers.
`ebf24eb7` moves the harness muting to `cmd audio set-volume` with a readback check.

**Logcat.** `logcat -d -b all` covers the whole boot, 02:21:17 to 05:36:15 (31,777 lines).
The literal unfiltered `FATAL|AndroidRuntime` scan matches 53 lines. They belong to three
`FATAL EXCEPTION`s in the keyboard process, each raised by the test-only `AcceptanceProbe`
through `FixtureInstrumentation` before `acceptance-fixture.pro` kept the members it calls:

| Time | Error | Probe frame |
| --- | --- | --- |
| 04:08:27 | `NoClassDefFoundError` `kotlin.io.FilesKt` | `AcceptanceProbe.reader` |
| 04:34:55 | `NoSuchMethodError` `LearnedWords.add$default` | `AcceptanceProbe.learnedFixture` |
| 04:38:17 | `NoSuchMethodError` `LearnedWords.add` | `AcceptanceProbe.learnedFixture` |

No production frame, native fatal signal or ANR appears. The scan is not literally clean
for that reason; the three crashes are harness defects that were fixed during the run.

**Shutdown.** `device_close(shutdown=true)` returned "Device shutdown failed: Could not
communicate with device support. Try refreshing devices." The advised refresh lists
`aw_keyboard_ssd` as not booted, adb no longer lists emulator-5556, and no emulator process
remains. The guest is shut down. The AVD and models on `/Volumes/2TB` are kept.

**Gate after the muting fix.** The first run failed in `:lib:rust:cargoOracleTest`: the
`latin-oracle` lib unit test binary (six tests) died with SIGSEGV. Standalone reruns of the same
binary passed 674 times: 14 sequential/default, 120 default, 480 with 12 concurrent runners, and
60 with macOS malloc scribble/guard checks. The rerun gate exits 0 in 2m 27s, with 803 JVM
tests, zero failures/errors/skips, and only the permitted lint warnings (AndroidGradlePluginVersion 1,
NewerVersionAvailable 3, ChromeOsAbiSupport 1).

**Resolved: intermittent `latin-oracle` SIGSEGV (2026-11-14).** The cause was a data race in
the oracle's fake JNI environment (test-only), not in shipped code. Every `latinime::Dictionary`
constructor logs through `LogUtils::logToJava`, which calls `FindClass("android/util/Log")`.
Dictionaries open under the oracle's shared lock, so the lib tests' parallel
`Dictionary::bundled()` calls could reach `findClass` together the first time that name was
looked up, when its unguarded class vector was growing. One thread could then read or
reallocate the vector while another wrote it. `locking.rs` had wrongly said only JNI_OnLoad
writes the fake environment's tables.

A scratch integration test that opens the bundled dictionary on 24 threads released by one
barrier reproduced the race at once. Uninstrumented, 8 of 1,300 processes died, by SIGSEGV
(exit 139) or by SIGABRT (exit 134) from heap corruption. With AddressSanitizer on the C++
(`CXXFLAGS="-fsanitize=address -fno-omit-frame-pointer -g"`, linking
`libclang_rt.asan_osx_dynamic.dylib`), 270 of 1,000 processes reported errors: 232
heap-buffer-overflows, 35 double-frees and 3 SEGVs. All 270 had `findClass` in their stacks,
under `logToJava` and `Dictionary::Dictionary`. The fix gives the class table its own mutex.
After it, the same test passed 1,000 of 1,000 ASan processes and 5,000 of 5,000 uninstrumented
ones.

The whole `latin` and `latin-oracle` suites under ASan (102 tests) then reported no memory
errors. One comparison differs only in the ASan build:
`layout::tests::sweet_spot_factor_matches_the_cpp` for a signalling NaN. The normal -O2 C++
compiles `std::min(C, x)` to `fminnm`, which the Rust port reproduces on purpose (see
`touch_position_correction_utils.rs`). The instrumented build compiles it to `fcmp`/`fcsel`
instead, verified by disassembling both objects. No tap distance is NaN, and the normal build,
which the gate runs, matches. The scratch race test is not committed. It catches a regression in
only about 0.6% of uninstrumented processes, so the docs record its recipe instead.
