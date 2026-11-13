# Rust in the keyboard

AudioWhisper Keyboard is moving its own logic from Kotlin to Rust, one piece at a time. Each
piece lands only when it gives the same results as the Kotlin it replaces, proven by a test that
runs both, and has been checked on the emulator.

## The 90% target (decided 2026-11-12)

The target is at least 90% Rust in our main source, because stopping at logic leaves most of
this keyboard in Compose and settings. Count non-blank lines of tracked `.kt`, `.java` and
`.rs` files; exclude paths containing `/test/`, `/tests/`, `/androidTest/` or `benchmark/`,
`*Test.kt`, `*_test.rs`, `*_tests.rs`, `tests.rs`, and generated tables
(`latin/src/chars/tables.rs`, `chars/src/tables/*`). Dependencies' native source is not ours
and does not count, including sherpa-onnx, onnxruntime and a renderer's dependencies.
The planning snapshot was about 25.8k Rust, 55.8k Kotlin and 0.6k Java (31% Rust).
After stage 1, the tracked-source recount on 2026-11-12 is 25,306 Rust, 53,808 Kotlin and
627 Java: 31.7% Rust. Recount after ports; never add Rust or remove tests just to improve a ratio.

### Stage 1: remove features we do not use (done)

Developer tools and non-Latin input were removed, including the Han provider and pack,
non-Latin composers, layouts, popup mappings and presets. This makes the next ports smaller,
but moves the percentage only to about 32%. Latin-script layouts, accented letters,
Vietnamese Telex, numeric/symbol/phone layouts, emoji, add-ons and the theme editor stay.

### Stage 2: remaining logic (roughly 50% Rust)

Port these packages in order, retaining a test-only Kotlin reference for each. Package sizes
are planning estimates, not delivery dates. Android ICU segmentation and locale casing stay
until a replacement matches Android. Keep Android work at the edge and avoid redundant JNI
wrappers: the point is to remove Kotlin decisions, not translate each line through a call.

1. Key/domain models: selectors, codes/types, currency slots, keyboard state and text-key evaluation (done; stage 2.1 below).
2. Layout assembly/cache: arrangements, merging, hints and cache keys; compare every retained layout's evaluated keys.
3. Snygg rules and stylesheet grammar: parsing, ordering and schema/version errors.
4. Snygg values: dimensions, colours, fonts, padding, shapes and variables.
5. Snygg compilation/editor model: cascade, inheritance, matching, editor operations and the live schema generator.
6. A1: let the Rust suggester call the Rust decoder through a native session; remove production LatinIME Java glue, keeping the C++ oracle.
7. Emoji history and quick-action arrangement.
8. Clipboard ordering, duplicate/expiry/limit/pinning and MIME policy; keep Room and the Android clipboard in Kotlin.
9. Extension metadata/import policy: identifiers, maintainers, validation and archive-entry decisions.
10. Dictation session state machine, compared with the existing schedules.
11. ASR and VAD through sherpa-onnx 2.0.1's exact C API; restore its excluded C library and preserve audio/decoder/VAD settings and flushing.
12. Pocket TTS through that C API: Loona, four threads, cancellation and stop-before-free; remove sherpa JNI only when unused.
13. Input dispatch and KeyboardManager routing: press/release/repeat and actions.
14. Editor state/reconciliation: expected text, selection/composing and UTF-16.
15. Editor operation plans: deletion, replacement and commits over a fake InputConnection.
16. NLP orchestration: provider eligibility, candidate state and stale-result rejection.
17. Subtype/settings defaults, validation, migration and active subtype.
18. Backup/restore manifests, conflicts and error classes.
19. Remaining utility rules: MIME matching, formatting and validation.
20. Consolidate engine boundaries, removing Kotlin–Rust callback chains and redundant wrappers.

### Stage 3: Rust draws the keyboard (roughly 68–76% Rust)

Kotlin hosts a SurfaceView in a FrameLayout under the IME service, maintains insets and
touchable regions, and forwards complete packed MotionEvents: action, pointer ids, coordinates,
history, timestamps and cancel. It keeps inline autofill views, GIF fetching/decoding and an
AccessibilityNodeProvider. Rust supplies the TalkBack semantic snapshot (stable ids, bounds,
labels, roles and actions), following AOSP LatinIME's accessibility delegate.

Rust takes ANativeWindow through JNI, owns a render thread with surface-generation ids, and
renders only dirty or animating frames using Choreographer timestamps. It owns the scene,
hit testing, keys, smartbar, candidates, panels, glide trail and popups inside an enlarged
surface. Existing Snygg JSON themes remain the input; density, font scale and Material You
colours come from Android. The theme editor hosts the same renderer through AndroidView and
passes draft snapshots, so previews and the keyboard use the same rules.

Try Vello (hybrid over wgpu/Vulkan or CPU, chosen by measurements), Parley and Skrifa first.
Keep system fonts and emoji, loaded through public APIs. First inspect the emulator and S24
fonts' CBDT/CBLC or COLRv1 format; prove bitmap/colour emoji, ZWJ families, flags, skin tones,
keycaps and VS15/VS16 work. skia-safe is the fallback if pure Rust cannot match appearance or
speed; it brings Skia C++ and an estimated 15–35 MiB, so measure its actual cost before choosing.

The prototype stays behind a setting with Compose as fallback. It must show QWERTY normal,
pressed and shifted labels, a suggestion strip with colour emoji, long-press e accents, a
glide trail, one bundled light and dark theme and a live theme-editor change. Exercise surface
recreation, rotation, cancel and a minimal TalkBack key list. Accept only identical committed
text and action traces from recorded input, plus reviewed visual pairs within anti-aliasing
tolerances. Target warm CPU scene p95 <= 2 ms and GPU p95 <= 3 ms. Use Perfetto FrameTimeline
for touch-to-present, missed frames, memory and idle power; measure added native-library size.
Alternate Compose/Rust trials first on the emulator, then on the S24 Ultra at 120 Hz. Only
after phone acceptance remove the Compose keyboard, smartbar/candidates/popups, media/emoji/
clipboard panels and Snygg UI; this prevents the fallback becoming permanent duplicate code.

### Stage 4: smaller settings (to 90%+)

At roughly 55–64k Rust lines, Kotlin and Java together must be around 6k. Settings, extensions
and the theme editor alone were about 13k at planning time. Rebuild them as a few data-driven
Compose screens: Rust describes settings/properties and owns validation, editing models and
workflows; accessible Compose controls stay. Slim the service shell and JNI declarations too,
then recount. Stage 4 is part of the decision, because drawing the keyboard alone cannot reach 90%.

The user chose 90% including settings, removal of developer tools and non-Latin input,
retention of add-ons and the theme editor, and pure Rust rendering first with Skia as fallback.
The implementation defaults are system fonts/emoji, exact behaviour with reviewed pixel
variation, deleting Compose after phone acceptance, and a 120 Hz S24 test before default-on.
The differential tests, deliberate mutation check and emulator checks below apply to every port.

## What is in Rust

| Piece | Rust | Replaced | Proof it matches |
|---|---|---|---|
| Key/domain models: selector choices, canonical key codes and predefined types/labels, currency slots, state bits, key strings, popup-source precedence, phone hints and enabled/visible policy | `crates/keys` (`codes`, `model_data`, `models`) | `KeyData` selectors, `CurrencySet`, `KeyboardState` bits, `TextKeyData` tables/string rules, and the pure parts of `TextKey`/`ComputingEvaluatorImpl` | `KeyModelsMatchTest` and `KeySelectorsMatchTest` against the test-only `ReferenceKey*` and `ReferenceKeyboardState`; every bundled layout in all selector states, seeded nested selectors, raw registers and transitions, currency sets, key/string and popup/availability cases. Reversing the unshifted case choice in Rust failed both selector tests. See stage 2.1 below for counts and emulator proof |
| Glide typing's scorer: pruning a swipe's candidate words and ranking them | `crates/glide` | `StatisticalGlideTypingClassifier`'s `Pruner`, ideal gestures and scoring | `SwipeIndexTest` against `ReferenceGlideClassifier`; on the emulator, 512 of 512 scorings were identical |
| Dictation's text rules: spoken commands, joining phrases transcribed separately, and taking back the last one | `crates/dictation` | `Spoken.parse`, `Phrases.join` and `punctuate`, `DictationUndo.revertAt` | `DictationRulesMatchTest` against `ReferenceDictation`: every UTF-16 unit through each rule, and 100,000 random phrases. The same cases on the emulator: none of 1,579,648 differed. Dictated recordings typed the same text |
| Dictation's filler words and custom vocabulary, new settings (Settings > Dictation): the Mac app's rules, not a port of the keyboard's own. The keyboard's spoken commands, joining and capitals stay as they were; a phrase is tidied before they see it | AudioWhisper-Core's `transcript`, a pinned git dependency of `crates/awnative` | Nothing replaced: `DictationRules.format`, called from `Dictation.type` | The Rust is the Mac app's, proven against its Swift there (AudioWhisper-Core's differential tests). Here `DictationRulesTest` checks the JNI: fillers, terms, CamelCase splits, `heard -> written`, comments, and text that isn't valid UTF-16 left alone; it fails for each of 2 breaks in the JNI. The page renders on the emulator |
| The mic key's tap and hold: a tap starts dictation and the next stops it, a hold of 300 ms or more dictates until let go, a press while listening finishes on release, and a release with no press seen toggles. Kotlin still starts and stops dictation | `crates/dictation` (`mic_key`) | `MicKey`, now a wrapper | `MicKeyMatchTest` against `ReferenceMicKey`: 2,000 random runs of 40 presses and releases in every status, at gaps either side of a hold and with clocks where a Long wraps. It fails for each of 18 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 200,000 differed. A press and release takes 0.10 µs against Kotlin's 0.04, the cost of crossing into Rust, far below a finger's time. A tap and a hold both work on the emulator |
| Typing suggestions' smaller rules: a word's case, the words before it, which characters continue a word, whether a correction is sure enough to make by itself, undoing one, and where taps land on the decoder's QWERTY layout | `crates/suggest` | `WordCase`, `PreviousWords`, `WordCharacters`, `AutoCorrection`, `AutoCorrectionUndo`, `TapOffset`, `TapTrail` and `QwertyGeometry`, now wrappers | `SuggestRulesMatchTest` against `ReferenceSuggest`: every code point and every UTF-16 unit through each rule, 200,000 random words and corrections, 200,000 taps and 20,000 tap trails. The same on the emulator: none of 15,586,581 differed. Typing the same keys on both builds gave the same text |
| The words typing learns and keeps: words and word pairs the user has typed, words whose correction they undid, and their personal dictionary's words and shortcuts. Kotlin still reads and writes the files; Rust parses and writes their text | `crates/suggest` | `LearnedWords`, `KeptWords` and `PersonalWords`, now wrappers | `WordStoresMatchTest` against `ReferenceStores`: 2,000 random runs of learning, undoing, saving and loading damaged files on each kind of store, asking both the same questions after every step, and 100,000 random personal dictionaries. The same on the emulator: none of 5,647,957 differed. Typing the same keys on both builds gave the same text and saved the same files, before and after a restart |
| Swipe typing's word list and ranking: which words a swipe can spell, how strongly the glide classifier favours each, and how the words before a swipe re-rank its guesses. Kotlin still asks the dictionary and the learned words what they know of each word | `crates/suggest` | `GlideRanker` and `SwipeWords`' rules, now wrappers | `SwipeRulesMatchTest` against `ReferenceSwipe`: every bonus for pair counts up to a million and for every n-gram and probability from -260 to 520, 100,000 random rankings (some longer than 20 guesses, where a sort that isn't stable would show), every frequency, every UTF-16 unit and 100,000 random words, and 20,000 random dictionaries with the user's words. The same on the emulator: none of 5,798,258 differed, frequencies to the bit. Swiping the same words on both builds, with and without a typing habit and a learned word, gave the same text. Building the 160,000-word list takes 12-15 ms, against Kotlin's 14-17 |
| The suggester and what typing learns: which corrections, completions and the user's own words the strip shows and in what order, whether to auto-correct, the spell checker, next-word predictions, and which spelling of each finished word is learned and when it is taken back. Kotlin still runs the C++ decoder, which Rust asks through a callback | `crates/suggest` | `LatinSuggester` and `WordLearner`, now wrappers | `LatinRulesMatchTest` against `ReferenceLatin`: 40,000 random strips and spell checks over a made-up dictionary whose every answer is a pure function of the question, with random personal and learned words, thresholds and taps, and 1,500 random runs of finished, swiped and undone words on two learners, comparing what each stored. It fails for each of 60 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 246,648 differed; and over the bundled dictionary, none of 4,000 typing states once both saw the same answers from it (see below). Typing the same keys on both builds (corrections and their undo, "came fro", a made-up word learned, suggestions and predictions tapped, a swipe) gave the same text and saved the same files, before and after a restart. A suggestion takes 7.4 ms either way, nearly all of it the C++ decoder |
| Reading a page aloud: which block reading starts at, how the pieces a walk of the screen found join into paragraphs, where a selection starts, where reading carries on after a scroll, how text is cut into what the speech engine takes, whether the words being read still fit in view, and the speaker key's tap and hold. Kotlin still walks the screen and drives the speech engine | `crates/reader` | `PageText`, `FollowAlong.fits` and `SpeakerKey`'s rules, now wrappers | `ReaderRulesMatchTest` against `ReferenceReader`: 40,000 random pages, joins and scrolls, 20,000 texts to speak (some about as long as the speech engine takes), 100,000 views and 2,000 runs of presses on the speaker key. It fails for each of 63 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 400,000 differed. Reading a Settings screen aloud on the emulator found its 23 blocks, read and stopped. Joining and finding take about two thirds and half of Kotlin's time; carrying on after a scroll takes 23 µs against Kotlin's 0.3, the cost of handing Rust the page's text, once a scroll |
| The screen walk's decisions: whether a scrollable view goes sideways (a carousel or a pager), how much of a list is in sight, which of the screen's lists is the one that scrolls and which text is inside it, and how following along brings a web page's piece into view. Kotlin still walks Android's accessibility tree and scrolls | `crates/reader` | `ScreenRules`, now a wrapper, which `ScreenWalk` and `FollowAlong` call | `ScreenRulesMatchTest` against `ReferenceScreen`: 40,000 random scrollable views and class names, 100,000 pairs of boxes (some as wide as `Int` goes, so their width wraps as Kotlin's does), 40,000 random trees of scrollers holding text and 100,000 pieces to bring into view. It fails for each of 51 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 620,001 differed. Reading a Settings screen aloud on the emulator walked the same 41 nodes into the same 23 blocks, 22 in the list, read and stopped. Placing 600 pieces in the list takes 3 µs against Kotlin's 6; the rest take about the same, under 2 µs a call |
| The GIF panel's rules: editing the search box and offering the word just typed, the order of the category chips, which keys and swipes reach the search box, the problem shown when GIPHY fails, which searches the result cache keeps and for how long, and the order of recently sent GIFs, saved and loaded. Kotlin still calls GIPHY, holds the GIFs and draws the panel | `crates/gif` | `SearchText`, `GifSearch.chips` and `problem`, `GifSearchKeys`, `GifResultCache` and `GifRecents`' rules, now wrappers | `GifRulesMatchTest` against `ReferenceGif`: 100,000 random queries and texts (odd spaces, emoji and half emoji, apostrophes), 40,000 words for the chips, every key code from -10,000 to 13,000 with every key type, 40,000 swipes, every status near HTTP's, and 2,000 random runs each of a result cache (comparing what it holds as well as what it answers) and of recents. It fails for each of 63 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 1,086,149 differed. Every rule takes a few µs at most; sending a GIF again takes 4.5 µs against Kotlin's 1.2 |
| Reading GIPHY's answers and building its URLs: the JSON read exactly as kotlinx.serialization 1.12 read it into the panel's classes (members without commas, only four whitespace characters, unknown values skipped by brackets alone, raw control characters in strings), sizes read as Kotlin's `toFloatOrNull` and Java's parser read them, to the bit, which GIFs are kept, and each search's URL encoded as Android's `URLEncoder` encodes it. Kotlin still makes the call | `crates/gif` (`json`, `java_float`, `giphy`) | `Giphy.parse` and `Giphy`'s URLs, now a wrapper | `GiphyRulesMatchTest` against `ReferenceGiphy`: 60,000 random GIPHY-like answers, a third of them broken (commas, whitespace, escapes, nulls, repeats, cuts), 100,000 sizes (decimals and hexadecimals exactly halfway between two floats, far exponents, suffixes, padding) and 30,000 URLs for odd searches. It fails for each of 55 small breaks in the Rust, the JNI and the wrappers. The same on the emulator: none of 350,000 differed. Desktop Java 22 and later lose half an emoji at the end of one of `URLEncoder`'s 8-character chunks, where Android and Java 21 write `?` as the Rust does; the JVM test allows exactly that and the emulator holds the Rust to Android's own encoder. Reading a 30-GIF answer takes 0.41 ms against Kotlin's 2.4; a search's URL 1.0 µs against 5.4. Trending, a search and sending a GIF work on the emulator with a real key |
| The text keyboard's geometry: each key's flex factors, how keys in a row grow and shrink to fill it, row margins, each key's touch and visible bounds (end keys' touches reach the edges), the desired key's size, and which key a touch lands on. Kotlin still turns dp into pixels and holds the keys | `crates/keys` | `TextKeyboard.layout` and `getKeyForPos`, `TextKey`'s flex tables, `TextKeyboardLayout`'s desired key, now wrappers | `TextKeyboardMatchTest` against `ReferenceTextKeyboard`: 40,000 flex lookups, 20,000 desired keys and 10,000 random keyboards laid out and touched on and one float either side of key edges. It fails for each of 40 small breaks. On the emulator none of 1,485,902 differed. Laying out a QWERTY keyboard takes 2.2 µs against Kotlin's 4.0; finding a touched key is the same, 0.2 µs; a key's flex 0.2 µs against 0.04, once per key per layout. Typing works on the emulator |
| Swipes and the start of a glide: whether a finger's move or lift is a swipe, its direction of eight and how many units it went (Java's `atan2` and `toDegrees` matched to the bit), and whether a press becomes glide typing, which points the glide gets, and when it completes or is cancelled. Kotlin still reads touch events and the velocity tracker and calls the listeners | `crates/gestures` | `SwipeGesture.Detector` and `GlideTypingGesture.Detector`, now wrappers | `GesturesMatchTest` against `ReferenceSwipeGesture` and `ReferenceGlideTypingGesture`: 3,000 runs of 60 swipe steps and 3,000 of 40 glide events, plus the one point found to sit on a direction's edge. It fails for each of 64 small breaks. On the emulator none of 759,568 differed. A swipe move takes 0.03 µs against Kotlin's 0.13; a glide move about the same, 0.3 µs. Gliding a word and swiping the space bar to move the cursor work on the emulator |
| The keyboard's own spaces: the phantom space after a picked word, the space put before or after punctuation (off by default), deleting the space before a mark, and the flags that remember both, kept or dropped as the cursor moves. Kotlin still cuts the text before the cursor by graphemes, and only when a rule asks for it | `crates/spacing` | `EditorInstance.commitChar`'s spacing, `PhantomSpaceState.determine`, the two flag classes, `PunctuationRule.Fallback`, `FlorisLocale.supportsAutoSpace`, now wrappers | `SpacingMatchTest` against `ReferenceSpacing`: every UTF-16 unit in each place the rules look, then 4,000 runs of 50 random keystrokes, commits and cursor moves, comparing answers, flags and which texts were looked up. It fails for each of 40 small breaks. On the emulator, where Java's character rules differ, none of 2,164,983 differed. A keystroke's spacing takes 0.4–2 µs against Kotlin's 0.03–0.2, the cost of crossing into Rust, far below a keystroke's time. A phantom space after a picked word works on the emulator |
| Long-press popups: how the extended popup's grid is laid out (rows, which side it grows to, where the prioritized keys sit, its bounds), which key the finger is on as it moves or that it left, and the order of a key's popup keys by hint mode, keeping the old code's habit of handing back the set's own list in a few cases. Kotlin still builds the keys and draws the popup | `crates/popup` | `PopupUiController.extend` and `propagateMotionEvent` (through a new `ExtendedPopup`), `PopupSet.initPopupList` and `mergeInternal`, now wrappers | `PopupMatchTest` against `ReferencePopupGrid` and `ReferencePopupSet`: 100,000 random layouts (NaN, infinite, negative and huge floats, Int extremes, keyboards narrower than the popup), 200,000 finger positions either side of each edge, 20,000 lookups and 6,000 runs on popup sets. It fails for each of 40 small breaks. On the emulator none of 618,254 differed. A finger move takes 0.03 µs either way; laying out a popup 0.7 µs against 0.1, and ordering one 1 µs against 0.3, once per long press. Long-pressing e showed its accents around the 3 and sliding picked é on the emulator |
| The shift key and capitals: shift's press, release, caps lock and cancel under both capitalisation behaviours, automatic capitals, unshifting after a typed key, the double-tap timing, and the double-space full stop. Kotlin still knows which keys are down, reads the clock and settings, and does `fixCase`, which needs locale casing (Turkish, Lithuanian) the character tables don't have | `crates/shift` | `KeyboardManager`'s shift handling and double-space check, `InputEventDispatcher.isConsecutiveDown`/`Up`, through a new `ShiftLogic` | `ShiftMatchTest` against `ReferenceShift`: every state, event, behaviour, caps mode and flag combination, 5,000 random runs of 40 events, 200,000 double taps around the timeout with wrapping clocks, and every UTF-16 unit around 26 partner characters for the double space. It fails for each of 50 small breaks. On the emulator, where Android's regex `\s` takes more spaces than the JVM's, none of 3,920,160 differed. Each call takes 0.04 µs against Kotlin's 0.01; the double-space check 0.5 µs against 1.2. Shift, caps lock and the double-space full stop work on the emulator |
| Generic rule-table composing, including Vietnamese Telex: what a keystroke deletes and types given the text before it | `crates/composing` | `Composer.WithRules.getActions`, through `ComposingRules` | `ComposingMatchTest` against `ReferenceWithRules`: the bundled Telex rules, random rule tables, case changes, lone surrogates and every UTF-16 unit. Hangul, kana and their feature-only tests were removed from the Latin-only keyboard |
| Emoji: reading the bundled emoji files (categories, skin-tone sets, the by-tone lists), an emoji's skin tone and hair style, which locale's file to load, and the `:` suggestions, scored, sorted and cut to size. Kotlin still reads the files and settings and builds the emoji it shows; the parsed file stays in Rust so each keystroke is one call | `crates/emoji` | `EmojiData`'s parsing, `Emoji`'s tone and hair style, `EmojiSuggestionProvider`'s query check and scoring, now wrappers | `EmojiMatchTest` against `ReferenceEmoji`: every bundled file, 2,000 damaged copies of them, and 20,000 random queries, tones and locales each. It fails for each of 40 small breaks. On the emulator none of 163,808 differed (17,463 file, 58,883 damaged-file and 87,462 query checks). Reading a file takes 3.5 ms against 11; a `:` keystroke's suggestions about 0.75 ms against 0.8, still off the main thread. Typing `:smi` offered ☺️ 😏 😼 on the emulator |

Many large pure-logic pieces are in Rust. The next stages move the remaining decisions, then
the keyboard drawing and settings models, so Kotlin becomes a small Android shell.

### Stage 2.1: key/domain boundary and proof (2026-11-12)

The existing `keys` crate now owns domain decisions as well as geometry. Codes have one
canonical Rust table; Kotlin exposes named values from it. Predefined keys carry their types
and labels in Rust. Kotlin retains the kotlinx.serialization classes and enums, Compose's
layout-direction value, observable state and batch notifications, Android fact gathering,
ICU's mark predicate and locale casing/cache, and assembly of key/popup objects. Rust chooses
children and popup sources by index, so JNI does not need to understand Kotlin objects or
call back into an evaluator. Clipboard queries still stay behind the locked-device guard.
Layout assembly/cache and Android labels/icons remain for their later work packages.

The JVM differential tests retain the replaced Kotlin as `ReferenceKeySelectors`,
`ReferenceKeyboardState`, `ReferenceKeyMetadata`, `ReferenceKeyCodes` and `ReferenceKeyRules`:

- 93 bundled layouts across 132 JSON files, 5,840 key nodes in 80 shift/variation/direction/width
  combinations: 467,200 cases. Both sides receive the same casing snapshots on the JVM,
  because Android ICU itself was not ported. Another 40,000 seeded nested selectors include
  missing children and children whose computation returns null; only missing children fall back.
- All 65,536 low state registers and 20,000 random full-width registers, then 2,000 seeded
  runs of 40 public mutations (80,000 transitions), including snapshots and wrapping/high bits.
- 123,003 key-code inputs through lookup, space/currency classification and UTF-16 string
  rules; all 60 predefined key records and every named code pinned against the old values.
  Invalid code points are ignored, while lone surrogate code points are retained, as before.
- 200,300 currency lookups over 100 seeded sets of different lengths, plus all 26 bundled
  currency sets (52,026 lookups) and 80 explicit space/type cases.
- All 5,120 popup-source masks/variations and 598,026 enabled/visible code/fact combinations,
  plus popup modes/groups and phone hints. Fallback, disabled utilities and password cases
  retain the old precedence.

The deliberate mutation made an unshifted case selector choose its upper child. Both the
bundled-layout and seeded-selector tests failed; the change was removed and the full checks
passed. Tests also caught and fixed a port mistake: code-info labels for positive control
characters use appendCodePoint, whereas display labels use the key's label. A separate audit
against the original Kotlin caught a temporary extraction regex missing the digit in
VIEW_SYMBOLS2: correcting the reference first made the lookup test fail, then fixing the
Rust restored parity. Existing quirks
stay: currency lookup takes abs(code), including positive codes and Int wrapping, and
SHOW_SUBTYPE_PICKER is predefined but absent from the code-info lookup.

Using the plan's non-blank tracked-main-source measure, stage 2.1 removes 487 Kotlin lines
and adds 991 Rust lines (including JNI); Java is unchanged. Totals are 53,321 Kotlin,
26,297 Rust and 627 Java, about 32.8% Rust. This is a migration count, not a claim that the
90% target has been reached. Test references do not count toward either total.

On the muted s24_like emulator (`local`, `emulator-5554`), the baseline and port builds were
installed with `scripts/install-dev.sh` and driven through the T3 device session. The same
key sequence matched in alphabet/symbol and numeric fields, including automatic capitals,
manual shift, caps lock and release back to lowercase. Long-press e showed the same accent
and number-hint grid; the dollar key and its five currency alternatives matched, including
the prioritized cent key. Password input stayed masked, without suggestions or an enabled
mic. Phone keys retained their secondary labels. The requested FATAL/AndroidRuntime logcat
search found only routine runtime startup/exit messages, with no fatal or error-level runtime
entries. The temporary field-check APK is a build artifact, not a new app feature.

### Sharing a core with the Mac app

The Mac app's text rules are Rust too, in [AudioWhisper-Core](https://github.com/jtn0123/AudioWhisper-Core)
(public), which the Mac app builds as a submodule:

- `transcript`: spoken punctuation, filler words, custom vocabulary, fitting a phrase to the text before
  the cursor. Merged into the Mac app in AudioWhisper PR #59.
- `apple-text`: the Swift and Foundation text semantics and ICU regular expressions those rules need.
- `correction`: the safety gate that rejects a model's "correction" that changed too much (PR #60).

Each was ported against the Mac's Swift with a differential test and a mutation check, as here. The
keyboard takes `transcript` as a git dependency pinned to a commit (`crates/awnative/Cargo.toml`) for its
filler words and vocabulary; bump the `rev` to take a newer core. The keyboard keeps its own spoken
commands, joining and capitals: they were written for phrases transcribed one at a time as the user
speaks, where the Mac's rules see a whole dictation at once, so neither set replaces the other.

That is where the Mac's port stops (2026-11-10). What is left of its Swift is Apple-framework glue
(AudioToolbox file reading, Accelerate's FFT, which beats plain Rust, SwiftUI and AppKit), the realtime
audio buffer, where a call into Rust on every audio callback would add risk for nothing, and prompt text
only the Mac uses. None of it is shared or faster in Rust.

### The LatinIME port

The C++ is ported against itself rather than a Kotlin copy. `crates/latin-oracle` builds
LatinIME's own sources for the computer running the tests (with the NDK's `jni.h` and a fake JNI
environment that runs the production `JNI_OnLoad`), so a test can put the same question to the
C++ and to the Rust: open the bundled dictionary, a word's probability, an n-gram's, its word
list, and a typing decode with taps and previous words. `cargo run --release -p latin-oracle
--example oracle -- gen 20000 20261109` writes seeded questions shaped like the keyboard's, and
`oracle run` answers them.

First, everything the keyboard never calls was deleted: the writable dictionary formats (v4 and
v402) and their writers, the dictionary-building and GC code, `BinaryDictionaryUtils`, and every
native method of `BinaryDictionary` but open, close, suggestions, the two probabilities and the
word list. The 20,000 questions got byte-identical answers before and after, and the comparison
caught a changed scoring constant (1,864 of the first 2,000 answers differed). What is left is
about 11,600 lines of code (16,800 with comments) in 138 files.

That remainder is now ported, in `crates/latin`, one module per area of the C++: `chars` and
`bytes` (character rules, integers and code points), `header` (the dictionary's header and its
attributes), `trie` (the version 2 trie: words, probabilities, word pairs, shortcuts and the word
list), `layout` (the keyboard and the taps on it) and `search` (the typing search, its node cache
and its scoring). The port keeps the C++'s types and order of operations, so its answers match to
the bit. Each module is tested against the C++ through the oracle, and each was broken on purpose,
one change at a time, to show the tests notice; the few changes they could not notice are ones no
input can tell apart.

The app runs on the port. `crates/awnative/src/latinime.rs` holds the eleven natives of
`BinaryDictionary`, `DicTraverseSession` and `ProximityInfo`, which Java finds by their JNI names;
`NativeLibrary` loads libawnative, and libjni_latinime.so is no longer built or shipped. A closed
handle gets the C++'s answers; input the C++ would read through a null pointer or past an array
throws instead. On the emulator, 4,000 seeded questions (typos, prefixes, predictions, previous
words, tap offsets) got byte-identical answers from the C++ build and the Rust build: every output
array, including the slots a call leaves alone, the weight, both probabilities and the whole word
list. Typing "helo" on the emulator offers help, helo and hell.

`lib/latinime/src/main/jni` stays only as the reference `latin-oracle` compiles for the tests.

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
  crates/keys/                the text keyboard's geometry: flex, layout, hit-testing
  crates/gestures/            swipes and the start of a glide
  crates/spacing/             phantom and automatic spaces
  crates/popup/               long-press popups: grid, finger tracking, key order
  crates/shift/               the shift key, automatic capitals, the double-space full stop
  crates/composing/           rule-table composers, including Vietnamese Telex
  crates/emoji/               emoji files, skin tones and `:` suggestions
  crates/gif/                 the GIF panel: its search field, chips, keys, result cache, recents and GIPHY's JSON and URLs
  crates/latin/               LatinIME's dictionary and typing search, ported from its C++
  crates/chars/               Kotlin's character rules, for crates that judge text as Kotlin did
  crates/awnative/            libawnative.so: the JNI functions Kotlin calls, and nothing else
  crates/latin-oracle/        test-only: LatinIME's C++ built for this computer, to compare the port with
  tools/                      generates crates/chars' tables (generate-char-tables.sh)
  src/main/kotlin/            the Kotlin side: loads the library and wraps each handle
  src/test/kotlin/            JVM tests that go through JNI
  build.gradle.kts            runs cargo
```

There is one native library, `libawnative.so`, however many crates there are. Each area of logic is
its own crate with no unsafe code (`#![forbid(unsafe_code)]`) and no knowledge of Android or the
JVM, so it is tested with plain `cargo test`. `awnative` is the only shipped crate with `unsafe`
(the test-only `latin-oracle` calls C++): it converts JNI arrays to slices, calls the logic crate,
and turns bad input into a Java exception instead of a crash.

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
