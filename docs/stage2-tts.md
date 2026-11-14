# Stage 2 package 12: Pocket TTS and reader policy

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
