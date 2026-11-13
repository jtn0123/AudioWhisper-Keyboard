# Stage 2 package 6: native Latin session

`suggest::session::Session` owns the read-only dictionary, QWERTY proximity model,
traversal cache and retained decoder outputs. `Request` implements the suggester's dictionary
trait by calling `latin` in-process. The application no longer calls back through Kotlin or
Java to decode, look up probabilities, or arrange typing/spelling results.

Rust constructs previous-word/sentence-start slots, applies limits and tap geometry, reads
and stably orders decoder output, filters the word walk, and decides cursor-mid-word state.
Personal and learned stores remain native and are read directly during a suggestion request.
Kept-as-typed words still use the existing native store at the provider's editor eligibility
boundary. Android editor/input-field eligibility remains in the provider (stage 2 package 16).
Kotlin reads dictionary bytes from a file/APK range and converts JNI data into UI models.
The native session uses the wrapper's monitor for every operation and close; its Cleaner
releases the allocation once. No session lock spans a Kotlin callback.

The four LatinIME Java classes, old `LatinDictionaryEngine`, `QwertyGeometry` and callback
adapter survive only in test sources. Existing JNI malformed-array and closed-handle tests
still exercise those references on the host. The eleven legacy JNI exports are host-only;
production Android contains neither them nor the Java classes. `lib/latinime/src/main/jni`
is unchanged and remains the C++ oracle source, outside the Gradle application dependency graph.
The oracle keeps its own frozen pre-A1 geometry/rounding model, so it does not depend on the
new production suggester. Its development dependency tests the production native session.

## Parity evidence

Before changing production code at `6acde09c`, `A1Replay` saved:

- 4,000 seeded requests drawn from the bundled word list and public keyboard fixtures,
  including prefixes, typos, predictions, capitalization, previous words, sentence starts,
  offsets, invalid UTF-16 and word-length boundaries. Each records candidates in order,
  scores, kinds, both probabilities, and every retained output slot, confidence and weight bits.
- 2,000 seeded full-pipeline states, each producing a strip and spell verdict (4,000 pipeline
  calls). The strip records its autocorrect/typed-word flags; personal shortcuts and learned
  words/followers feed the same path as production. No device user dictionaries are read.
- Both renderer regressions: the 300-press digest `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`
  and eight-hold digest `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`.
- Both renderer suggestion fixtures: automatic correction on space and selecting a decoder
  candidate, checked using committed text digests without saving typed-state trees.

`A1SessionMatchTest` compares the frozen callback/Java path with the native session on the JVM.
Its complete files also match the saved pre-change JVM baseline. The independent C++ oracle
matches all 4,000 questions from seed `20261113`, including every live result field and exact
weight bits, and the entire ordered dictionary word list. The JVM replay checks untouched
buffer slots between successive calls, which the C++ helper's fresh output arrays do not retain.

Deliberately sorting candidates in ascending score order fails `A1SessionMatchTest`; restoring
the stable descending sort passes. Clearing sentence-start flags did not change this
dictionary's answers and is not counted as a successful mutation check.

`LatinSessionBoundaryTest` covers invalid dictionary bytes, idempotent close, use after close,
malformed tap transfers, NaN tap rejection and every UTF-16 unit in the cursor-mid-word rule.
An additional reference test caught the weight reset occurring after geometry conversion:
a rejected first NaN tap retained zero instead of -1. Resetting before conversion preserves
every retained output slot and the original exception message on this path as well.
The existing store, suggester, swipe and legacy-JNI tests remain in the full Gradle gate.

## Reproduction

Run the JVM match and oracle suites through the commands in `rust.md`. For the device-only
replay, build `:app:assembleDebug :app:assembleDebugAndroidTest -PsuggestFixture=true`, install
both APKs using `scripts/install-dev.sh` (the test APK uses `--probe`), and run
`com.jtn0123.audiowhisper.keyboard.test/com.jtn0123.audiowhisper.suggest.A1Instrumentation`.
It returns only SHA-256 digests; complete public-fixture outputs live in private `files/a1-replay`
until cleanup. This optional build preserves APIs called from the separate test APK and adds
the test-only buffer snapshot export. Neither is enabled in the normal application build.

With the owned runtime serial in `ANDROID_SERIAL`, the renderer checks are
`scripts/render-benchmark/regression.sh OUTPUT` and
`python3 -I scripts/render-benchmark/suggestions.py OUTPUT`. Keep agent-device idle during
these input harnesses. Device artifacts stay in ignored `build/stage2-a1`.

## Device result (2026-11-13)

On owned `aw_keyboard` / runtime `emulator-5556`, both complete replay files match the
pre-change files byte for byte. Decoder SHA-256 is
`daea23ac4c96fcf41d7c8d165faf3fde16e9fbf9ed5c6f5693be19e397e54019`;
pipeline SHA-256 is `658c8914602e62d97572366074775389d3f6c49c4941b41d0b90949b253555ba`.
The Compose and Rust renderer regressions retain both digests above. On each renderer,
autocorrect-on-space digest is `6e5ce6afa65bc328ed7ef2585ac6077dca716587a929ef60778a364b5680051c`
and candidate selection digest is `106a5842fc5fce6f663176285ed1516dbb1e3d15c05abab12fdca46d60b539b7`.
No replay data or screenshots are tracked. Final cleanup and the combined Gradle gate are
recorded with the package 9 delivery.
