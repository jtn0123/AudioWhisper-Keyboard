# Rust in the keyboard

AudioWhisper Keyboard is moving its own logic from Kotlin to Rust, one piece at a time. Each
piece lands only when it gives the same results as the Kotlin it replaces, proven by a test that
runs both, and has been checked on the emulator.

## What is in Rust

| Piece | Rust | Replaced | Proof it matches |
|---|---|---|---|
| Glide typing's scorer: pruning a swipe's candidate words and ranking them | `crates/glide` | `StatisticalGlideTypingClassifier`'s `Pruner`, ideal gestures and scoring | `SwipeIndexTest` against `ReferenceGlideClassifier`; on the emulator, 512 of 512 scorings were identical |
| Dictation's text rules: spoken commands, joining phrases transcribed separately, and taking back the last one | `crates/dictation` | `Spoken.parse`, `Phrases.join` and `punctuate`, `DictationUndo.revertAt` | `DictationRulesMatchTest` against `ReferenceDictation`: every UTF-16 unit through each rule, and 100,000 random phrases. The same cases on the emulator: none of 1,579,648 differed. Dictated recordings typed the same text |
| The mic key's tap and hold: a tap starts dictation and the next stops it, a hold of 300 ms or more dictates until let go, a press while listening finishes on release, and a release with no press seen toggles. Kotlin still starts and stops dictation | `crates/dictation` (`mic_key`) | `MicKey`, now a wrapper | `MicKeyMatchTest` against `ReferenceMicKey`: 2,000 random runs of 40 presses and releases in every status, at gaps either side of a hold and with clocks where a Long wraps. It fails for each of 18 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 200,000 differed. A press and release takes 0.10 µs against Kotlin's 0.04, the cost of crossing into Rust, far below a finger's time. A tap and a hold both work on the emulator |
| Typing suggestions' smaller rules: a word's case, the words before it, which characters continue a word, whether a correction is sure enough to make by itself, undoing one, and where taps land on the decoder's QWERTY layout | `crates/suggest` | `WordCase`, `PreviousWords`, `WordCharacters`, `AutoCorrection`, `AutoCorrectionUndo`, `TapOffset`, `TapTrail` and `QwertyGeometry`, now wrappers | `SuggestRulesMatchTest` against `ReferenceSuggest`: every code point and every UTF-16 unit through each rule, 200,000 random words and corrections, 200,000 taps and 20,000 tap trails. The same on the emulator: none of 15,586,581 differed. Typing the same keys on both builds gave the same text |
| The words typing learns and keeps: words and word pairs the user has typed, words whose correction they undid, and their personal dictionary's words and shortcuts. Kotlin still reads and writes the files; Rust parses and writes their text | `crates/suggest` | `LearnedWords`, `KeptWords` and `PersonalWords`, now wrappers | `WordStoresMatchTest` against `ReferenceStores`: 2,000 random runs of learning, undoing, saving and loading damaged files on each kind of store, asking both the same questions after every step, and 100,000 random personal dictionaries. The same on the emulator: none of 5,647,957 differed. Typing the same keys on both builds gave the same text and saved the same files, before and after a restart |
| Swipe typing's word list and ranking: which words a swipe can spell, how strongly the glide classifier favours each, and how the words before a swipe re-rank its guesses. Kotlin still asks the dictionary and the learned words what they know of each word | `crates/suggest` | `GlideRanker` and `SwipeWords`' rules, now wrappers | `SwipeRulesMatchTest` against `ReferenceSwipe`: every bonus for pair counts up to a million and for every n-gram and probability from -260 to 520, 100,000 random rankings (some longer than 20 guesses, where a sort that isn't stable would show), every frequency, every UTF-16 unit and 100,000 random words, and 20,000 random dictionaries with the user's words. The same on the emulator: none of 5,798,258 differed, frequencies to the bit. Swiping the same words on both builds, with and without a typing habit and a learned word, gave the same text. Building the 160,000-word list takes 12-15 ms, against Kotlin's 14-17 |
| The suggester and what typing learns: which corrections, completions and the user's own words the strip shows and in what order, whether to auto-correct, the spell checker, next-word predictions, and which spelling of each finished word is learned and when it is taken back. Kotlin still runs the C++ decoder, which Rust asks through a callback | `crates/suggest` | `LatinSuggester` and `WordLearner`, now wrappers | `LatinRulesMatchTest` against `ReferenceLatin`: 40,000 random strips and spell checks over a made-up dictionary whose every answer is a pure function of the question, with random personal and learned words, thresholds and taps, and 1,500 random runs of finished, swiped and undone words on two learners, comparing what each stored. It fails for each of 60 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 246,648 differed; and over the bundled dictionary, none of 4,000 typing states once both saw the same answers from it (see below). Typing the same keys on both builds (corrections and their undo, "came fro", a made-up word learned, suggestions and predictions tapped, a swipe) gave the same text and saved the same files, before and after a restart. A suggestion takes 7.4 ms either way, nearly all of it the C++ decoder |
| Reading a page aloud: which block reading starts at, how the pieces a walk of the screen found join into paragraphs, where a selection starts, where reading carries on after a scroll, how text is cut into what the speech engine takes, whether the words being read still fit in view, and the speaker key's tap and hold. Kotlin still walks the screen and drives the speech engine | `crates/reader` | `PageText`, `FollowAlong.fits` and `SpeakerKey`'s rules, now wrappers | `ReaderRulesMatchTest` against `ReferenceReader`: 40,000 random pages, joins and scrolls, 20,000 texts to speak (some about as long as the speech engine takes), 100,000 views and 2,000 runs of presses on the speaker key. It fails for each of 63 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 400,000 differed. Reading a Settings screen aloud on the emulator found its 23 blocks, read and stopped. Joining and finding take about two thirds and half of Kotlin's time; carrying on after a scroll takes 23 µs against Kotlin's 0.3, the cost of handing Rust the page's text, once a scroll |
| The screen walk's decisions: whether a scrollable view goes sideways (a carousel or a pager), how much of a list is in sight, which of the screen's lists is the one that scrolls and which text is inside it, and how following along brings a web page's piece into view. Kotlin still walks Android's accessibility tree and scrolls | `crates/reader` | `ScreenRules`, now a wrapper, which `ScreenWalk` and `FollowAlong` call | `ScreenRulesMatchTest` against `ReferenceScreen`: 40,000 random scrollable views and class names, 100,000 pairs of boxes (some as wide as `Int` goes, so their width wraps as Kotlin's does), 40,000 random trees of scrollers holding text and 100,000 pieces to bring into view. It fails for each of 51 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 620,001 differed. Reading a Settings screen aloud on the emulator walked the same 41 nodes into the same 23 blocks, 22 in the list, read and stopped. Placing 600 pieces in the list takes 3 µs against Kotlin's 6; the rest take about the same, under 2 µs a call |
| The GIF panel's rules: editing the search box and offering the word just typed, the order of the category chips, which keys and swipes reach the search box, the problem shown when GIPHY fails, which searches the result cache keeps and for how long, and the order of recently sent GIFs, saved and loaded. Kotlin still calls GIPHY, holds the GIFs and draws the panel | `crates/gif` | `SearchText`, `GifSearch.chips` and `problem`, `GifSearchKeys`, `GifResultCache` and `GifRecents`' rules, now wrappers | `GifRulesMatchTest` against `ReferenceGif`: 100,000 random queries and texts (odd spaces, emoji and half emoji, apostrophes), 40,000 words for the chips, every key code from -10,000 to 13,000 with every key type, 40,000 swipes, every status near HTTP's, and 2,000 random runs each of a result cache (comparing what it holds as well as what it answers) and of recents. It fails for each of 63 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 1,086,149 differed. Every rule takes a few µs at most; sending a GIF again takes 4.5 µs against Kotlin's 1.2 |
| Reading GIPHY's answers and building its URLs: the JSON read exactly as kotlinx.serialization 1.12 read it into the panel's classes (members without commas, only four whitespace characters, unknown values skipped by brackets alone, raw control characters in strings), sizes read as Kotlin's `toFloatOrNull` and Java's parser read them, to the bit, which GIFs are kept, and each search's URL encoded as Android's `URLEncoder` encodes it. Kotlin still makes the call | `crates/gif` (`json`, `java_float`, `giphy`) | `Giphy.parse` and `Giphy`'s URLs, now a wrapper | `GiphyRulesMatchTest` against `ReferenceGiphy`: 60,000 random GIPHY-like answers, a third of them broken (commas, whitespace, escapes, nulls, repeats, cuts), 100,000 sizes (decimals and hexadecimals exactly halfway between two floats, far exponents, suffixes, padding) and 30,000 URLs for odd searches. It fails for each of 55 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 350,000 differed. Desktop Java 22 and later lose half an emoji at the end of one of `URLEncoder`'s 8-character chunks, where Android and Java 21 write `?` as the Rust does; the JVM test allows exactly that and the emulator holds the Rust to Android's own encoder. Reading a 30-GIF answer takes 0.41 ms against Kotlin's 2.4; a search's URL 1.0 µs against 5.4. Trending, a search and sending a GIF work on the emulator with a real key |

Next, roughly in order: upstream FlorisBoard's pure logic, then the LatinIME C++
(`lib/latinime`), and last sharing a core with the Mac app.

## Layout

Everything lives in `lib/rust`, one Gradle module that is also a cargo workspace:

```
lib/rust/
  Cargo.toml, Cargo.lock      the workspace; release builds use LTO and abort on panic
  rust-toolchain.toml         the pinned Rust version, with the Android target
  .cargo/config.toml          16 KB page alignment for Android; cargo's output in build/cargo
  crates/glide/               glide typing's scorer: plain Rust, no unsafe, no JNI
  crates/dictation/           dictation's text rules and the mic key
  crates/suggest/             typing suggestions, auto-correction, the spell checker, what typing learns
  crates/reader/              reading a page aloud: blocks, selections, scrolls, speech, the speaker key, the screen walk's decisions
  crates/gif/                 the GIF panel: its search field, chips, keys, result cache, recents and GIPHY's JSON and URLs
  crates/chars/               Kotlin's character rules, for crates that judge text as Kotlin did
  crates/awnative/            libawnative.so: the JNI functions Kotlin calls, and nothing else
  tools/                      generates crates/chars' tables (generate-char-tables.sh)
  src/main/kotlin/            the Kotlin side: loads the library and wraps each handle
  src/test/kotlin/            JVM tests that go through JNI
  build.gradle.kts            runs cargo
```

There is one native library, `libawnative.so`, however many crates there are. Each area of logic
is its own crate with no unsafe code (`#![forbid(unsafe_code)]`) and no knowledge of Android or
the JVM, so it is tested with plain `cargo test`. `awnative` is the only crate with `unsafe`: it
converts JNI arrays to slices, calls the logic crate, and turns bad input into a Java exception
instead of a crash.

Native objects (an index, a query, a word store) cross JNI as a `Long` handle. The Kotlin wrapper
frees each one with a `java.lang.ref.Cleaner` once it is unreachable; a query is also
`AutoCloseable`, and using one after `close()` throws rather than touching freed memory. An object
that changes and is used from more than one thread (a tap trail, the learned and kept words) sits
behind a `Mutex` (`handle::with_locked`).

Files stay in Kotlin. A store's wrapper reads its file and hands Rust the text, and writes the text
Rust gives back, so Rust never touches the disk and the file's format is in one place.

A long list of strings, such as a dictionary's words, crosses as one string and their lengths
(`jni::cut`). A `String[]` costs several JNI calls a string: 160,000 words that way took longer
than the Kotlin being replaced took to do the work.

## Building

`./gradlew assembleDebug` builds the Rust too; nothing else is needed but
[rustup](https://rustup.rs). The first build installs the pinned toolchain and the Android target
from `rust-toolchain.toml`. If your rustup doesn't install them automatically, run
`rustup toolchain install` in `lib/rust`.

Gradle runs cargo twice:

- `:lib:rust:cargoBuildAndroid` builds `libawnative.so` for arm64 Android, linked with the NDK's
  clang, and packages it as the module's native library.
- `:lib:rust:cargoBuildHost` builds it for this computer, which the JVM unit tests (in `lib/rust`
  and `app`) load. It is an input of those tests, so changing the Rust runs them again.

Gradle finds cargo in `$CARGO_HOME/bin` or `~/.cargo/bin`, because Android Studio doesn't see
the shell's `PATH`.

## Checks

```bash
./gradlew :lib:rust:check   # cargo test, clippy with warnings as errors, rustfmt --check
./gradlew test              # every JVM test, including those that go through JNI
```

Or, from `lib/rust`, plain `cargo test`, `cargo clippy --all-targets -- -D warnings` and
`cargo fmt`.

## Rules

- **Same results as the Kotlin.** A port keeps the Kotlin's arithmetic: `Float` stays `f32`,
  `Double` stays `f64`, operations happen in the same order, and Java's `Math.max`/`min`
  (NaN wins; -0 is less than +0) are written out by hand, since Rust's `max`/`min` differ, as is
  Java's `Float.compare` for sorting (-0 before +0, NaN last). On the phone both use bionic's
  `exp`, so costs match to the bit. On a computer the JVM's `Math.exp` can differ from the
  system's in the last bit, so the JVM tests let a swipe word's frequency be one unit in the last
  place off as long as the `Float` the classifier takes of it is the same; the emulator check
  allows none.
- **Judge characters as Kotlin does.** Text crosses JNI as UTF-16 (`&[u16]`), and anything that
  asks what a character is (`isLetter`, `isWhitespace`, `uppercaseChar`, a regex's `\s`) asks
  `crates/chars`, never Rust's `char` methods, which answer differently for some characters.
  Kotlin's answers come from `java.lang.Character`, which differs between a computer's JVM and
  Android (ICU): on Android `isLowerCase` leaves out ª and ʰ, and `\s` takes in the no-break
  spaces. So `chars` has a table for each, generated by asking Java on each; the library built for
  Android uses Android's, and the one the JVM tests load uses the JVM's. They are Unicode 16 (JDK
  24, Android 16). After moving to a JDK or an Android with a newer Unicode, run
  `ANDROID_SERIAL=<emulator> tools/generate-char-tables.sh` in `lib/rust`.
  The same goes for strings: `lowercase()`, `uppercase()`, `equals`/`startsWith(ignoreCase)`,
  `toIntOrNull()` and code points are `chars::case`, `chars::text` and `chars::code_point`, which
  copy each platform's way of doing them (Android compares `ignoreCase` char by char; the JVM, code
  point by code point). `JavaTextTest` holds them up against Java on every character, every code
  point and random strings of the hardest ones; on the emulator, none of 17,792,939 comparisons
  differed. One difference is on purpose: Rust decides whether Σ ends a word (ς) by ICU's rule,
  as Android does, which the JVM's `toLowerCase` doesn't, so the JVM tests count ς and σ as one.
- **Prove it with a differential test.** Before deleting Kotlin, copy it into the tests as a
  reference (as `ReferenceGlideClassifier`) and compare the two over real data and odd cases.
  Make sure the test fails when the Rust is changed slightly, so it isn't passing for nothing.
  The JVM tests see the JVM's character rules, so where those matter, also compare the two on the
  emulator: a temporary patch runs the test's cases there and logs only how many differ.
  Where the code asks something outside it, as the suggester asks the C++ decoder, give both the
  same answers: the test's made-up dictionary answers each question by hashing it, so it doesn't
  matter how often or in what order each side asks. The real decoder isn't so steady: asked for the
  same strip twice, it gave a different one 3 times in 4,000 on the emulator (odd words such as
  "vis-a-vis"), so a comparison over the bundled dictionary remembers each of its answers and hands
  both sides the same ones.
- **Write down every difference that's on purpose**, in the Rust and in the test. The glide
  scorer has two: equally near keys keep layout order (Kotlin used hash order, which changed
  from run to run), and an empty swipe gives nothing (Kotlin scored stale storage). A tap trail
  can't be made with a negative capacity (Kotlin's took one, then threw on the first character
  typed into it). Text can't be cut for speech into pieces of no length or less (Kotlin's cut
  forever for 0 and threw `StringIndexOutOfBoundsException` for less); the app only asks for 3,900.
- **One copy of each number.** A Kotlin wrapper reads the rule's numbers (the QWERTY layout's
  sizes, the correction threshold) from Rust rather than repeating them, so the two can't drift.
- **No file over 500 lines**, generated tables aside. A crate's tests go in `<module>/tests.rs`
  (`#[cfg(test)] mod tests;`) rather than inline once a file grows past that.
- **No warnings, and none silenced.** Clippy runs with `-D warnings`; fix what it says rather than
  adding `#[allow]`.
- **Unsafe stays in `awnative`**, and each `unsafe` block says why it is sound (`// SAFETY:`).
- **Never hold a lock across a call back into Kotlin.** The suggester locks the learned words
  only for each read, not while it waits on the decoder, so a callback can't deadlock against
  another thread learning a word.
- **Kotlin that Rust calls back is kept by name.** R8 shrinks the debug build too, and deletes or
  renames what it doesn't see Kotlin call. Rust finds a callback (such as `DictationNames.isName`)
  by name, so `consumer-rules.pro` keeps it.
- **A panic is a bug.** Release builds abort on panic and the panic hook writes it to logcat
  (tag `awnative`) first. Bad input from Kotlin is an exception, not a panic.
- **Never log what the user types.** Timings and counts only, as everywhere else.

## Adding a piece

1. Add a crate under `crates/` with the logic and its tests; no JNI, no unsafe.
2. Add its JNI functions to `awnative` (a new module, named after the crate) and the `external`
   declarations and wrapper in `src/main/kotlin`.
3. Copy the Kotlin it replaces into the app's tests as a reference and write the differential
   test. Run it, and make it fail once on purpose.
4. Switch the app over, delete the Kotlin, and check it on the emulator.
5. Add it to the table above.
