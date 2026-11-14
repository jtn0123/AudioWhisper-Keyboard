# Stage 2 packages 10 and 11: dictation

Implementation and acceptance evidence for the session-policy and sherpa C API ports.

**Implementation and dictation-specific proof are complete; final emulator acceptance is
incomplete.** The strict legacy wall-clock long-delete fixture is not waived. Final cleanup,
settings/permission restoration, literal logcat scan and shutdown could not be verified after
the guest stalled and T3 close failed twice. See the final hold below. The required local
Gradle gate exits zero, and the user explicitly authorized commit/push under this device-tool
failure condition.

The Java-free `dictation` crate owns the lifecycle, generation rejection, model-readiness
classification, mic tap/hold, idle-model deadlines, UI projection, final/partial acceptance,
preview admission, VAD lead-in/history and preview schedule. It batches spoken-command,
spacing/capitalisation and undo plans using the existing dictation/editor rules. The managed
host supplies editor context and dictionary name facts, then executes effects outside the lock.
Each semantic insertion/undo resumes from the accepted editor context: InputConnection filters
may truncate or reject text, so subsequent spoken commands never use predicted editor contents.

Kotlin retains AudioRecord, permission/file I/O, thread executors, Android clocks, Flow/UI
bindings and InputConnection effects. No production Kotlin sherpa ASR or VAD imports remain.
The single JNI crate is still `awnative`. `sherpa-ffi` is the dedicated native C boundary;
logic crates still forbid unsafe code, and all native unsafe blocks have SAFETY comments.
Reader/TTS production code is untouched; its sherpa JNI and Kotlin API remain packaged.

## Library and settings

The pre-existing `app/libs/sherpa-onnx-2.0.1.aar` already contains the C library. Packaging now
excludes only the unused C++ API; no new binary or model is committed. The AAR SHA-256 matches
both the published release API digest and the release's `checksum.txt`. That checksum file's
own SHA-256 matches its published release API digest.

- AAR: `a8d09d1d32f3eb8520482c6d3d43fae674681c5b050385967c70a3c2c9bf0ee7`.
- arm64-v8a C API: `0eadae93c4d595a26584d7eebde7b397b9a36d01eee3ebc7262e0d34df2fce10`.

Sources: [2.0.1 release](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.1),
[published checksums](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/checksum.txt),
[tagged C header](https://github.com/k2-fsa/sherpa-onnx/blob/v2.0.1/sherpa-onnx/c-api/c-api.h).
`verifySherpa` checks both pins before builds; `scripts/dictation-provenance.py` checks them
independently. `scripts/sherpa-bindings.py` checks the header SHA before regenerating the exact
configuration ABI, including unused trailing fields. Generated numeric Rust tests independently
check C-compiler sizes, alignment and every field offset on 64-bit targets.

Parakeet remains offline TDT v2 int8, 16 kHz/80 features, four CPU threads,
`nemo_transducer` and `greedy_search`; model filenames are unchanged. Silero remains
0.5 threshold, 0.6 s silence, 0.25 s speech, 512-sample windows, 20 s max speech,
one CPU thread and a 60 s internal buffer. Mic chunks remain 1,600 samples;
lead-in is 4,800, detection delay 4,512, history 400,000, preview period 16,000
(first preview after 8,000), and preview maximum 160,000 samples.

The original offline recognizer does **not** call input-finished or add explicit tail padding:
it accepts the complete waveform, decodes and retrieves a result. Rust preserves this exact
sequence and the internal feature padding. Final VAD `flush()` emits buffered tail speech before
free. No new synthetic tail silence was introduced. Native RAII frees result snapshots before
streams and streams before recognizers; synchronous decoding is serialized, close waits for
active calls, and atomic cancellation invalidates in-flight results. A later dictation can reuse
the cached recognizer with a new cancellation generation.

## Reproduction

```sh
python3 -I scripts/dictation-provenance.py
./gradlew :app:testDebugUnitTest --tests '*Dictation*Test' --tests '*ReleaseAtTest' \
  :lib:rust:testDebugUnitTest --tests '*DictationSegmentationMatchTest'
./gradlew :app:assembleDebug :app:assembleDebugAndroidTest -PdictationFixture=true
```

Use T3's owned `aw_keyboard` runtime serial, install only through `scripts/install-dev.sh`,
and install the disposable test APK through its `--probe` option. The opt-in test runner is
`com.jtn0123.audiowhisper.dictation.DictationInstrumentation`. Its default mode creates Pocket
TTS speech directly as private WAVs, adds silence/noise, a long mid-utterance pause, a very short
clip and a clipped ending, and feeds identical PCM to both paths. It never opens AudioRecord
or AudioTrack. `-e reuse true` reuses those exact WAVs under private `files/dictation-fixture-audio`.
The separately opt-in debug source set contains a shell-only receiver protected by Android
DUMP permission. `scripts/dictation-ui.py OUTPUT reference|native AGENT_DEVICE_COMMAND`
captures `listening`, `processing`, `no-model`, and `no-permission` without restarting the IME
through instrumentation. It temporarily hides one real model file and revokes the real mic
permission, restores both in `finally`, and uses no audio/model executor. The receiver is
absent from normal APKs. Baseline screenshots use an archived c071a487 build with only this
test source set added; production baseline dictation logic is unchanged.

Only hashes, lengths, sample positions, UI-state metadata and monotonic timing timestamps enter
evidence. Decoder text exists only in memory while both original/native session owners process
the same real result; session snapshots contain preview/editor hashes, never text. Generated WAVs,
models, APKs, screenshots, dumps, downloads and private settings stay outside git.


## State and segmentation proof

The frozen production owners and arithmetic live only in `Reference*.kt` test sources.
`DictationSessionMatchTest` compares **50,000 events across 200 seeds** with identical fake
executor schedules: mic gestures, chunk-derived levels and VAD flags, stop/cancel/restart,
partials/finals, missing/damaged models, load/decode/microphone failures, old-session results,
visibility and timeout edges. Snapshots compare public state, editor/preview UTF-16 hashes,
error ids, model state, unload deadlines, queued work and microphone stop/flush effects.
Directed Rust checks stop/cancel at all ten phases; managed checks reject callbacks after
release without overwriting a newer owner's UI. A real editor-filter regression caught and
fixed speculative batching across spoken commands; each continuation now reads accepted text.

Segmentation adds **50,000 PCM chunks across 200 seeds**, comparing bounded-history slices,
lead-in, segment audio and preview deadlines with frozen Kotlin arithmetic. The recorded model
trace adds **315 chunks across seven fixtures**, 14 previews and five phrases, replayed with
seeded PCM; the device additionally compares the actual audio and actual decoder results.
Actual JNI/C API transcripts also drive both session owners in memory, with **656 state
snapshots** compared without retaining text.

Four deliberate production mutations were detected and restored byte-for-byte:

| Mutation | Required failure |
| --- | --- |
| Model idle timeout 300,000 → 300,001 ms | JVM assertion failure |
| Accept final generation <= current, including cancelled sessions | JVM assertion failures |
| Silero threshold 0.5 → 0.99 | Device parity failure on fixture 0 |
| Remove final VAD tail flush | Device parity failure on clipped fixture 5 |

The mutation runner rejects compilation errors as evidence, requires actual assertion/parity
failures, removes stale reports and reruns restored code. The flush mutation removes the exact
legacy offline tail-flush operation; there was no caller-added recognizer padding to mutate.
The test also runs the real managed `PhraseListener`/`MicRecorder` executor with a private WAV
replay source: one phrase, four previews, zero microphone/model errors, identical PCM hashes,
and no leftover native-VAD model temporary files. This never opens the room microphone.


## Source accounting and local gate

Using the existing nonblank production-source measure, c071a487 contains 53,469 Kotlin,
44,101 Rust and 321 Java lines. The final source contains **53,212 Kotlin, 45,483 Rust and
321 Java**: **257 net production Kotlin lines removed and 1,382 Rust lines added**, or
**45.94% Rust**. Test/reference sources, benchmark code, generated character tables and the
entirely test-only `latin-oracle` harness are excluded. The counting rule now explicitly names
that test-only harness; including its 2,674 Rust lines would inflate both baseline and final
counts. Every changed/new source file is at most 500 lines. Dependencies are excluded.

The required gate exits **0**, captured directly from Gradle before any reporting command:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

There are **747 JVM tests**, zero failures/errors/skips. Rust formatting, tests, clippy with
`-D warnings` and the full C++ differential oracle pass. The five app lint warnings are one
AndroidGradlePluginVersion, three NewerVersionAvailable and one ChromeOsAbiSupport; Rust's
Android library lint has no findings. No warning suppressions or baselines were added.
Two existing Kotlin data-class constructor-visibility warnings were fixed by making the raw
integer constructors public, consistent with their existing public copy/wrap behavior.

The normal APK's C API entry has the pinned SHA above; JNI remains, OfflineTts remains,
OfflineRecognizer and Vad Kotlin classes are absent, and the UI receiver is absent. Reader
production source is unchanged. The opt-in comparison APK retains the old JNI path solely
as an oracle; it is replaced with the normal gated APK after testing.


## Audio parity and timing

The final owned `emulator-5556` run completed with `dictation_complete=true`. Across seven
private Pocket-generated fixtures, all **26 transcript comparisons** (seven whole, five segment
and 14 partial results) match byte-for-byte by UTF-8 SHA-256 and length. All **315 chunk-end
VAD observations**, raw segment boundaries, five segment PCM hashes, 14 preview PCM hashes,
and the one buffered final-flush result match. VAD timestamps are sample positions at the
original 1,600-sample observation granularity; exact raw segment boundaries are also compared.

| Fixture | Whole-result UTF-8 bytes | SHA-256 |
| --- | ---: | --- |
| 0 | 75 | `5ba81156b538277e476600860c4a46129d64d86da64a866be53bd84b427afffb` |
| 1 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 2 | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| 3 | 75 | `5ba81156b538277e476600860c4a46129d64d86da64a866be53bd84b427afffb` |
| 4 | 5 | `85fec70372b092bb42d021a6d1b373081dde8bcf90feb49e0725e285043fb0ad` |
| 5 | 57 | `238ae8fc5479290d1ce6be962263b7e305817bd683b0b4bd3dd481d477f47501` |
| 6 | 77 | `6bf1a55762808f5795592081437f0e9bb7978fab6c4751a51c3bed56569c79fd` |

Final sanitized report SHA-256:
`82147b55cf38e6ca7e5bc53c5f55ac72d82d3ccdcbbec0de7533627b75ae6a87`.

Ten alternating runs per path use the same 97,239-sample (6.0774375 s) fixture, with no builds
running. Reports retain monotonic model-load/start/first-partial/end timestamps only. Derived
median real-time factor is **0.168083 JNI / 0.162905 C API**; median first-partial replay compute
latency is **105.132188 ms JNI / 99.613480 ms C API**. Model load, the separate whole-utterance validation decode and teardown are excluded.
VAD creation and partial/segment/final processing are included. Replay is unpaced: this is
compute latency,
not elapsed live-microphone latency or a claim of a statistically established speedup.
The ten individual timestamp records per path and chart stay under ignored evidence.


## Harness controls

Incomplete setup attempts are kept separately from accepted evidence. The learning runner
compares exact store bytes: a missing scratch file differs from the serialized empty store
left by unlearning. Initializing scratch with the canonical empty header fixes that setup;
the unchanged learner/unlearner assertions then pass on both renderers. Original learned,
kept-as-typed and clipboard Room files are held privately and moved back byte-for-byte.

The pre-existing panel runner's reused coordinates failed at Select all on the new build
and later Paste on archived c071a487. Its scrolling grid now supplies fresh, stable bounds
for each editing action. A dedicated observer mode returns only bounds for five allowlisted
public action buttons, never an IME tree, candidate label or editor node. Full tree capture
still requires an empty disposable editor. Native tiles can retain bounds outside a clipped
grid, so only their visible body intersection is returned. Viewport/root/malformed bounds
controls pass;
all original selection, copy/paste digest, scrolling and arrangement assertions remain.
The archived baseline passes the corrected complete Compose panel replay.


## Emulator UI and additional typing fixtures

Four before/after screenshot pairs show listening, processing, missing-model and denied-mic
errors on Always day + Floris Day with the Rust renderer. All four state/status/problem
records match the baseline exactly. The empty editor and empty preview keep speech out of
the images. Baseline production code is archived c071a487, built fresh; preinstalled APKs
with uncertain provenance were excluded after their unrelated fixture mismatches.

Both renderers pass learning/unlearning with identical restored-store hashes, unchanged
incognito storage and zero password candidates/inline suggestions. All **17** extra panel,
emoji/recents/editing/arrangement, clipboard, subtype/one-handed, invalid-batch and no-key GIF
records match the package-16 accepted manifest exactly. The batch depth remains zero and
the GIF query has three units while its editor remains empty.

The normal gated APK passes the primary regressions on both renderers: 300 keys at
`809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`, 308 units after popup
holds at `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`, autocorrect at
`6e5ce6afa65bc328ed7ef2585ac6077dca716587a929ef60778a364b5680051c` and candidate picking at
`106a5842fc5fce6f663176285ed1516dbb1e3d15c05abab12fdca46d60b539b7`.

The ordinary editor replay initially timed out waiting for sentence-caps labels on Rust,
as in the earlier NLP proof. That incomplete preparation run is retained; the complete
unchanged replay subsequently passes all old digests and UTF-16 selection/composing offsets
on both renderers. No caps-specific production fix or timeout-cause claim is made.

For input, the first two new-APK runs matched both renderers but differed from the old
preinstalled snapshot's wall-clock long-hold result (1,056 versus 1,055 units). A fresh normal
c071a487 control was rebuilt with an isolated target directory; **all 789 original production
source files** match that commit byte-for-byte. Its full input replay also yields 1,056 units
on both renderers, and every input record exactly matches both new-APK runs. The genuine
controlled baseline replaces the unverified preinstalled snapshot for this comparison. No
hold duration, repeat schedule, predicate or expected transcript hash was changed. The
historical 1,055-unit record and both failed historical-snapshot comparisons remain retained;
wall-clock delivered-repeat counts are an explicit limit of comparisons across test sessions.


## Final device hold and recovery state

Six ordinary suites pass on the normal gated APK: primary regression, suggestions, extended
editor, glide, all eight keyboard modes, and NLP. The 17 extra fixture records and the
learning/privacy tests pass separately. The strict long-delete check is **not accepted**:
two complete new-APK records exactly equal the freshly rebuilt c071a487 control at 1,056
units, but subsequent live replays differ between renderers by one delivered repeat
(1,055/1,056). Short-tap controls delete exactly one unit with and without a 500 ms settling
period. Neither production timers nor the 2,000 ms hold or comparison predicates were changed.
A trial starting both renderers from a fresh IME process was interrupted by device failures;
that experimental harness change was reverted. No failed test was suppressed or deleted.

The owned guest disappeared once. T3 reopened `aw_keyboard` as `emulator-5556`; the probe APK
and seven generated WAVs were then observed absent. The probe and verified baseline were
reinstalled for controls, followed by the normal gated APK. The guest subsequently stalled:
even `adb shell true` and logcat timed out. The stuck local test runner/adb clients were
cancelled. No other device was used or closed.

The first `device_close(hostId=local, deviceId=emulator-5556, shutdown=true)` returned:

> Device close failed: Could not communicate with device support. Try refreshing devices.

The advised device refresh was performed once; it reported that the hub did not answer
within 40 seconds. The one permitted retry returned:

> Device shutdown failed: Could not communicate with device support. Try refreshing devices.

The user's two-failure stop rule applies. No further device work was attempted. Therefore:

- Final prototype-off / Always day / Floris Day, accessibility and permission restoration
  are **not verified**. Earlier checks showed exact microphone/notification grant flags.
- Final literal `FATAL|AndroidRuntime` logcat and shutdown are **not verified**. The primary
  regression runners' AndroidRuntime-error checks passed before the device failures.
- No models were downloaded or installed. Existing Parakeet/Pocket models were used.
  Seven private WAVs and a private Silero copy were created for testing; WAVs were observed
  absent after reopening. Final removal of the Silero copy, test APKs and other fixture files
  is **not verified**. The probe was reinstalled and may remain installed.
- `files/dictation-typing-held` may still contain the pre-replay `learned-words.txt` and
  `kept-as-typed.txt` originals. The wrapper normally copies these back and verifies hashes,
  but its final cleanup could not run against the stalled guest. Recover these originals
  before deleting held test state. Earlier completed replay wrappers restored their hashes.
- Initial settings and permission evidence remain privately under `before/preferences.json`,
  `before/accessibility.json` and `before/permissions.txt`. They provide the recovery baseline.

The unfinished checks are a device acceptance hold, not a claim of a complete green runtime
pass. Source implementation, native/session reference comparisons, all four restored mutations,
seven exact speech/VAD fixtures, 20 timing trials and the full local Gradle gate remain proven.

## Evidence locations (ignored, not committed)

All paths below are relative to `build/stage2-dictation/`:

| Evidence | Location |
| --- | --- |
| Final ASR/VAD/session/production-listener parity and 20 raw timestamp trials | `final-audio.json`, `final-audio.log` |
| State and device mutation failures plus restored passes | `mutations-state-accepted/`, `mutations-device-release/` |
| Four genuine screenshot pairs | `after/ui-receiver/*-pair.png` |
| Source-verified c071a487 input control | `baseline-source-verification.json`, `before/input-controlled/` |
| Six accepted ordinary typing suites and 17 extra records | `after/`, `after/extras/comparison.json` |
| Unwaived repeat variance / incomplete replays | `after/input-repeat-*`, `typing-*.log` |
| Actual Gradle exit zero, warnings and normal packaging | `final-gate-delivery/` |

The generated audio, models, APKs, screenshots, dumps, downloaded header/checksums, local
settings and build products are excluded from the commits. Reader production source is unchanged.
