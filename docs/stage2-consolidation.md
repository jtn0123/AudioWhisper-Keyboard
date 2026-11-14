# Stage 2 package 20: engine boundary consolidation

## Inventory baseline

Inventory captured at `4685918a` before production edits: **358 external fun declarations in 43 production Kotlin files** (357 in lib/rust, one UtilityNative in lib/kotlin). Largest files: KeyboardRenderer 41, WordStoresNative 32, SuggestNative 29, GifNative 22, KeyModels 18, ReaderNative 15. The tracked-source counter reports Kotlin 52,839, Java 321, Rust 51,310: **49.11% Rust**. The supplied 49.09% was an earlier revision. No ratio-driven changes are authorized.

The inventory below preserves the pre-implementation view from commit `598264f5`. Final row dispositions and proof follow the inventory. Every declared JNI method appears below. Paths in the entry column are relative to lib/rust/src/main/kotlin/com/jtn0123/audiowhisper/rust unless stated otherwise. Rust source paths are relative to lib/rust/crates/awnative/src. The JNI symbol is `Java_<package_with_underscores>_<owner>_<method>`; package is com.jtn0123.audiowhisper.rust except UtilityNative (org.florisboard.lib.kotlin). The Rust column gives the implementing Rust function location and engine calls. Callers are direct managed call sites, including lib/rust transport wrappers. Counts in the hot-path table are structural call counts, not timings.

## Hot paths and platform ownership

| ID / action | Current exchange count | Classification and proposed boundary |
| --- | --- | --- |
| C1 finished/swiped word | One learner JNI plus 0–4 probability callbacks, each re-enters LatinSession JNI; pending swipe can learn two words | remove callback chain: native learner uses live native session directly; preserve null-dictionary and failure behavior |
| C2 typing/spell/legacy dictation | Production decoder is one native session call, zero decode callbacks. Legacy Suggester and DictationRules.join callback adapters remain packaged for tests | remove shipped legacy callback exports, retain complete host/fixture reference seams |
| C3 read-aloud walk | One JNI invocation; 2 callbacks per visited node (facts/class), one per attempted child, and one per fallback text | keep Android API pending identical replacement: Android tree is mutable; eager snapshots change read/exception order, pruning and 20,000-node/250-depth limits. Measure and retain demand-driven behavior |
| G1 glide rerank of N guesses | One slot count; N learned-pair lookups, N ngram lookups, up to N probability lookups, then one rank call (up to 3N+2 JNI) | merge native store/session context and rank into one native exchange; retain synthetic lookup test seam |
| R1 read-aloud blocks/selection/resume | One JNI method each, but String[] costs per-element JNI reads/copies; resume transfers two arrays | merge string transfer to one UTF-16 string plus lengths; preserve lone surrogates, exact limits and errors |
| H1 key press / candidate publication | One packed renderer touch per MotionEvent, dispatch lifecycle and route plans separated by Android effects; editor resumes ICU/provider requests; NLP config/call emits whole candidate plan. State/evaluator getters depend on observed recomposition count | keep event/test seams: Android clocks, MotionEvent, InputConnection, ICU and observable bindings. No fixed per-key count exists under async callbacks; do not claim all 41 renderer methods run per key |
| H2 key/layout/domain evaluation | KeyModels scalar selectors/read/write are one call per property/selector; layout assembly already bulk; enabled/visible depend on locked clipboard/platform facts | keep test seam/platform snapshots; do not cache mutable Android facts or alter observable-state notification ordering |
| H3 GIF/clipboard/emoji | One call per distinct storage or panel event; native upload one call per decoded frame | keep Android API: Room, clipboard, network/frame decode and platform callbacks have separate lifetimes |

### Counted phases of a key/candidate action

For an accepted ordinary DOWN/UP pair, InputEventDispatcher makes five lifecycle
exchanges: DOWN, DELAYS, ACK_DOWN, UP, ACK_UP. Each receiver phase makes one
FACTS(code), one FACTS(SHIFT) through isPressed, and one InputRules.route: six more
for the pair. sendDownUp instead has three lifecycle exchanges plus the same six
receiver calls. A Rust-renderer MotionEvent adds one packed touch exchange; the
number of MOVE events is input-dependent. KeyModels property reads/writes, feedback,
locale casing/text conversion and the requested editor operation add separate calls.
Rejected downs, cancellation, hold/repeat and receiver absence follow their own plans.
This is a structural breakdown of the named layer, not an asserted end-to-end total.

One completed asynchronous candidate request has beginFor, ready and completeFor
(three NLP exchanges); its clipboard assembly adds clipboardBeginFor and clipboardFor
(two more). The Latin provider adds one native suggest call, with separate typed-word,
previous-word and tap packet projections as listed in the complete inventory. Emoji
and Android clipboard-provider work depend on enabled policy. Context observers,
selection callbacks, invalidation and UI recomposition may add calls independently.
No per-candidate loop crosses JNI to publish the strip; the completion returns the
whole visibility plan. These counts and phases are unchanged by package 20. Glide
and learner context loops are the measured per-item boundaries consolidated below.

## Rust to Kotlin callbacks (all call sites)

| ID | Rust source and callback | Production reachability | Classification |
| --- | --- | --- | --- |
| C1a | suggest/suggester.rs ask → WordProbabilities.probability (one call_int site) | WordLearner → Probabilities → engine.session.probability; also legacy dictionary adapter | remove production chain; keep host-only reference lookup |
| C2a | suggest/suggester.rs decode → SuggestDictionary.decode (one call_object site) | No production decoder caller after A1; legacy tests only | remove Android export; keep host/latin-fixture oracle |
| C2b | dictation.rs is_name → DictationNames.isName (one call_boolean site) | Legacy text-rule test seam; production session uses supplied name facts | remove Android export; keep host/fixture oracle |
| C3a | reader_runtime.rs node → ReaderTree.facts and className (two call_object sites) | AndroidTree reads AccessibilityNodeInfo | keep Android API; preserve order and exceptions |
| C3b | reader_runtime.rs text → ReaderTree.text (one call_object site) | Lazy fallback; password/editable/hidden pruned text never fetched | keep Android API; preserve lazy read |
| C3c | reader_runtime.rs child → ReaderTree.child (one call_int site) | Child fetch is observable even after visit cap is reached | keep Android API; preserve null child and fetch sequence |
| C4 | tts.rs generate → TtsAudio.audio (one call_boolean site) | Synchronous borrowed PCM ByteBuffer; callback completes before C frees samples | keep Android API: AudioTrack/cancellation/backpressure; no redundant Rust re-entry |

There are **8 callback invocation sites** outside the Env helper definitions: four reader, two suggestion, one dictation, one TTS. The generic ask site serves two dictionary methods (probability and ngramProbability); count method contracts separately: nine. No native lock may span a production callback that re-enters Rust.

## Managed forwarding owners

Only production methods are considered; DTOs, Android effects, lifecycle ownership and reference-only helpers are distinguished.

| ID / owner | Production methods and boundary | Classification |
| --- | --- | --- |
| W1 AutoCorrection | Pure forwarding; only threshold used in production; other methods exercise frozen rules | remove production wrapper; retain test-only facade unchanged |
| W2 WordCharacters | Pure continuesWord forwarder; no executable production caller after NLP port | remove production wrapper; retain test-only facade unchanged |
| W3 WordCase | Enum conversion plus forwarding; no executable production caller after A1 | remove production wrapper; retain test-only facade unchanged |
| W4 ScreenRules / Box | Pure conversion/forwarding over ScreenWalkRules; obsolete traversal helpers used only by tests | remove production wrapper; call ScreenWalkRules directly; retain test-only facade for original tests |
| W5 FollowAlong.fits | Sole forwarding helper in otherwise Android-owned FollowAlong | remove unused production helper; retain reference-test seam |
| W6 PageText / TextBlock / Start | JNI packets converted back to reveal callbacks; Android sentence BreakIterator and Locale.US | keep Android API and managed callback identity; not pure forwarding |
| W7 FollowAlong | refresh, extras/bounds, performAction, coroutine delay | keep Android API; not pure forwarding |
| W8 SpeakerKey | Android SystemClock injection and enum mapping over native state | keep Android clock/test seam; clock queried exactly once per down/up |
| W9 PreviousWords / TypingState / RawSuggestion / AutoCorrectionUndo | Value models plus CharSequence/DTO conversion and native constants | keep model/test seam; remove only duplicated engines |
| W10 TapTrail / TapOffset | Managed nullable tap model, native ownership and packed output projection | keep test seam; press and typed events separated by editor acceptance |
| W11 LearnedWords / KeptWords / PersonalWords / SwipeWords | File/Room IO, save locks, row identity, learned/native store ownership | keep Android IO/test seam; G1 removes per-guess queries |
| W12 WordLearner / Learner | Native lifetime and lookup adaptation; C1 removes Probabilities wrapper | merge C1; keep main-thread ownership/test seam |
| W13 LatinSuggester / LatinDictionaryEngine | File range IO and native/UI model conversion | keep IO/test seam; decoder is already native |
| W14 GlideRanker | Per-guess context forwarding/assembly and native ranking | merge G1 production engine path; retain synthetic lookup seam only in tests |
| W15 Dictation Spacing/Spoken/Phrases/MicKey/DictationUndo | Text/enum/clock/effect adapters to native rules; old Phrases path is test-only after session plans | keep test seams/platform effects; C2 restricts old callback exports |
| W16 GifSearchKeys / SearchText / GifResultCache / GifRecents / Giphy / GifSender / GifImages / GifFrames | UI model identity, network/file/decoder IO and coroutine scheduling | remove pure SearchText production facade; keep Android/test seams for the other owners and preserve existing match fixtures |
| W18 Transcriber / VoiceEngine / LoadedVoice / PlayQueue / ModelHolder | Native calls plus Android model files, coroutine/focus/playback and managed callback lifetimes | keep Android API/lifetime seams; not pure forwarding |
| W17 lib/rust synchronized owners and JNI DTO adapters | Cleaner, monitors, reachability fences, packet conversion, enum projections | keep lifetime/test seams. Deleting these would expose raw handles or JNI arrays to Android owners |

## Complete JNI method inventory

| Kotlin caller(s) | JNI entry (owner.method / declaration file) | Rust function location; engine calls | Classification |
| --- | --- | --- | --- |
| UtilityRules.kt:6 | `UtilityNative.call` / UtilityRules.kt | `backup_utils.rs:40` → native scalar/handle transport | keep: already one plan/exchange; Android effects outside call |
| BackupPolicy.kt:5 | `BackupNative.call` / BackupPolicy.kt | `backup_utils.rs:30` → native scalar/handle transport | keep: already one plan/exchange; Android effects outside call |
| ClipboardManager.kt:72 | `ClipboardRules.mimeMatches` / ClipboardRules.kt | `clipboard.rs:82` → policy::mime_matches, u8::from | keep: typed JNI/test seam; lifetime or independent event |
| ClipboardManager.kt:349 | `ClipboardRules.eligible` / ClipboardRules.kt | `clipboard.rs:99` → policy::eligible, u8::from | keep: typed JNI/test seam; lifetime or independent event |
| ClipboardRules.kt:8 | `ClipboardNative.plan` / ClipboardRules.kt | `clipboard.rs:13` → Box::new, policy::duplicate, policy::expiry, policy::groups, policy::limit, policy::order | keep: already one plan/exchange; Android effects outside call |
| Composing.kt:30 | `ComposingNative.newRules` / ComposingNative.kt | `composing.rs:45` → Rules::new | keep: typed JNI/test seam; lifetime or independent event |
| Composing.kt:47 | `ComposingNative.freeRules` / ComposingNative.kt | `composing.rs:73` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Composing.kt:41 | `ComposingNative.rules` / ComposingNative.kt | `composing.rs:85` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:38 | `DictationNative.parse` / DictationNative.kt | `dictation.rs:50` → dictation::parse, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:54 | `DictationNative.join` / DictationNative.kt | `dictation.rs:68` → dictation::join, ptr::null_mut | remove shipped callback; keep reference-test seam (C2) |
| Dictation.kt:58 | `DictationNative.punctuate` / DictationNative.kt | `dictation.rs:107` → dictation::punctuate, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:65 | `DictationNative.revertsAt` / DictationNative.kt | `dictation.rs:129` → dictation::reverts_at | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:35 | `DictationNative.format` / DictationNative.kt | `dictation.rs:235` → ptr::null_mut, transcript::Vocabulary::parse, transcript::format | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:102 | `DictationNative.holdMs` / DictationNative.kt | `dictation.rs:150` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:82 | `DictationNative.newMicKey` / DictationNative.kt | `dictation.rs:159` → MicKey::default | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:97 | `DictationNative.freeMicKey` / DictationNative.kt | `dictation.rs:167` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:90 | `DictationNative.micKeyPress` / DictationNative.kt | `dictation.rs:200` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Dictation.kt:94 | `DictationNative.micKeyRelease` / DictationNative.kt | `dictation.rs:217` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| DictationSessionPlan.kt:21 | `DictationSessionNative.speech` / DictationSessionPlan.kt | `dictation_session.rs:128` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| DictationSessionPlan.kt:25 | `DictationSessionNative.model` / DictationSessionPlan.kt | `dictation_session.rs:111` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| DictationSessionPlan.kt:29 | `DictationSessionNative.preview` / DictationSessionPlan.kt | `dictation_session.rs:91` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| DictationSessionPlan.kt:5 | `DictationSessionNative.create` / DictationSessionPlan.kt | `dictation_session.rs:9` → dictation::session::Session::new | keep: typed JNI/test seam; lifetime or independent event |
| DictationSessionPlan.kt:31 | `DictationSessionNative.free` / DictationSessionPlan.kt | `dictation_session.rs:23` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| DictationSessionPlan.kt:17 | `DictationSessionNative.call` / DictationSessionPlan.kt | `dictation_session.rs:32` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| DictationSessionPlan.kt:34 | `DictationSessionNative.releaseAt` / DictationSessionPlan.kt | `dictation_session.rs:62` → dictation::session::release_at | keep: typed JNI/test seam; lifetime or independent event |
| DictationSessionPlan.kt:35 | `DictationSessionNative.ending` / DictationSessionPlan.kt | `dictation_session.rs:72` → dictation::session::ending, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| EditorSession.kt:14 | `EditorNative.create` / EditorSession.kt | `editor.rs:10` → editor::Session::new | keep: typed JNI/test seam; lifetime or independent event |
| EditorSession.kt:33 | `EditorNative.free` / EditorSession.kt | `editor.rs:22` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| EditorSession.kt:31 | `EditorNative.call` / EditorSession.kt | `editor.rs:31` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| EditorSession.kt:37<br>EditorSession.kt:46 | `EditorNative.attributes` / EditorSession.kt | `editor.rs:59` → editor::attributes::call, editor::integer, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Emoji.kt:81 | `EmojiNative.parse` / EmojiNative.kt | `emoji.rs:40` → EmojiData::parse | keep: typed JNI/test seam; lifetime or independent event |
| Emoji.kt:70 | `EmojiNative.free` / EmojiNative.kt | `emoji.rs:60` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Emoji.kt:35<br>Emoji.kt:74 | `EmojiNative.layout` / EmojiNative.kt | `emoji.rs:115` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Emoji.kt:97 | `EmojiNative.validateQuery` / EmojiNative.kt | `emoji.rs:131` → ptr::null_mut, suggest::validate_query | keep: typed JNI/test seam; lifetime or independent event |
| Emoji.kt:67 | `EmojiNative.suggest` / EmojiNative.kt | `emoji.rs:151` → ptr::null_mut, suggest::suggest | keep: typed JNI/test seam; lifetime or independent event |
| Emoji.kt:104 | `EmojiNative.tones` / EmojiNative.kt | `emoji.rs:175` → emoji::tones::tones | keep: typed JNI/test seam; lifetime or independent event |
| Emoji.kt:115 | `EmojiNative.resolveAsset` / EmojiNative.kt | `emoji.rs:192` → emoji::asset::resolve | keep: typed JNI/test seam; lifetime or independent event |
| ExtensionPolicy.kt:6<br>ExtensionPolicy.kt:7<br>ExtensionPolicy.kt:9<br>ExtensionPolicy.kt:10<br>ExtensionPolicy.kt:13<br>ExtensionPolicy.kt:16<br>ExtensionPolicy.kt:17<br>ExtensionPolicy.kt:21<br>ExtensionPolicy.kt:23<br>ExtensionPolicy.kt:24<br>ExtensionPolicy.kt:27<br>ExtensionPolicy.kt:28<br>ExtensionPolicy.kt:29<br>ExtensionPolicy.kt:31 | `ExtensionPolicyNative.call` / ExtensionPolicy.kt | `extensions.rs:7` → extensions::call, ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| Gestures.kt:22 | `GesturesNative.newSwipes` / GesturesNative.kt | `gestures.rs:53` → Swipes::default | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:81 | `GesturesNative.freeSwipes` / GesturesNative.kt | `gestures.rs:61` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:31 | `GesturesNative.swipesReset` / GesturesNative.kt | `gestures.rs:72` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:37 | `GesturesNative.swipesDown` / GesturesNative.kt | `gestures.rs:94` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:46 | `GesturesNative.swipesMove` / GesturesNative.kt | `gestures.rs:121` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:65 | `GesturesNative.swipesUp` / GesturesNative.kt | `gestures.rs:152` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:74 | `GesturesNative.swipesCancel` / GesturesNative.kt | `gestures.rs:190` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:125 | `GesturesNative.newGlideStart` / GesturesNative.kt | `gestures.rs:213` → GlideStart::new | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:154 | `GesturesNative.freeGlideStart` / GesturesNative.kt | `gestures.rs:222` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gestures.kt:146 | `GesturesNative.glideTouch` / GesturesNative.kt | `gestures.rs:262` → Touch::of | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:15 | `GifNative.maxLength` / GifNative.kt | `gif.rs:54` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:18 | `GifNative.categories` / GifNative.kt | `gif.rs:63` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:24 | `GifNative.recents` / GifNative.kt | `gif.rs:74` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:98 | `GifNative.cacheTtlMs` / GifNative.kt | `gif.rs:88` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:99 | `GifNative.cacheCapacity` / GifNative.kt | `gif.rs:97` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:27 | `GifNative.recentsCapacity` / GifNative.kt | `gif.rs:106` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:38 | `GifNative.type` / GifNative.kt | `gif.rs:115` → ptr::null_mut, search_text::type_text | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:40 | `GifNative.typeWord` / GifNative.kt | `gif.rs:133` → ptr::null_mut, search_text::type_word | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:42 | `GifNative.deleteLast` / GifNative.kt | `gif.rs:151` → search_text::delete_last | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:44 | `GifNative.deleteWords` / GifNative.kt | `gif.rs:167` → search_text::delete_words | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:46 | `GifNative.lastWord` / GifNative.kt | `gif.rs:183` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:48 | `GifNative.chips` / GifNative.kt | `gif.rs:199` → chips::chips, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:51 | `GifNative.route` / GifNative.kt | `gif.rs:216` → keys::route | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:54 | `GifNative.swipeDelete` / GifNative.kt | `gif.rs:232` → keys::swipe_delete | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:57 | `GifNative.problem` / GifNative.kt | `gif.rs:255` → Failure::Status, problem::problem, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:102 | `GifNative.cacheKey` / GifNative.kt | `gif.rs:277` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:80 | `GifNative.newCache` / GifNative.kt | `gif.rs:288` → ResultCache::new, self::capacity | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:94 | `GifNative.freeCache` / GifNative.kt | `gif.rs:304` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:87 | `GifNative.cacheGet` / GifNative.kt | `gif.rs:315` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:91 | `GifNative.cachePut` / GifNative.kt | `gif.rs:338` → From::Before, From::Sent, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:61 | `GifNative.recentsAdd` / GifNative.kt | `gif.rs:370` → ptr::null_mut, recents::add, self::capacity | keep: typed JNI/test seam; lifetime or independent event |
| Gif.kt:65 | `GifNative.recentsLoad` / GifNative.kt | `gif.rs:390` → ptr::null_mut, recents::load, self::capacity | keep: typed JNI/test seam; lifetime or independent event |
| Giphy.kt:18<br>Giphy.kt:42 | `GiphyNative.parse` / GiphyNative.kt | `giphy.rs:15` → giphy::parse, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Giphy.kt:34 | `GiphyNative.url` / GiphyNative.kt | `giphy.rs:45` → giphy::url, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Giphy.kt:37 | `GiphyNative.trendingUrl` / GiphyNative.kt | `giphy.rs:80` → giphy::trending_url, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Giphy.kt:40 | `GiphyNative.searchUrl` / GiphyNative.kt | `giphy.rs:95` → giphy::search_url, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Glide.kt:28 | `GlideNative.distinctChars` / GlideNative.kt | `glide.rs:54` → glide::distinct_units, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Glide.kt:89 | `GlideNative.build` / GlideNative.kt | `glide.rs:71` → GlideIndex::build | keep: typed JNI/test seam; lifetime or independent event |
| Glide.kt:74 | `GlideNative.free` / GlideNative.kt | `glide.rs:105` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Glide.kt:69 | `GlideNative.query` / GlideNative.kt | `glide.rs:116` → Gesture::from_points | keep: typed JNI/test seam; lifetime or independent event |
| Glide.kt:130 | `GlideNative.freeQuery` / GlideNative.kt | `glide.rs:144` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Glide.kt:107 | `GlideNative.candidates` / GlideNative.kt | `glide.rs:166` → Layout::new, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Glide.kt:117 | `GlideNative.score` / GlideNative.kt | `glide.rs:186` → Layout::new | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `GlideTrailRules.circles` / GlideTrailRules.kt | `render_trail.rs:7` → Scene::default, Trail::default, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| AppPrefs.kt:239 | `ImeChromeRules.fontMultiplier` / ImeChromeRules.kt | `render_chrome.rs:287` → render::ime_chrome::font_multiplier | keep: typed JNI/test seam; lifetime or independent event |
| FlorisImeSizing.kt:127 | `ImeChromeRules.rowHeights` / ImeChromeRules.kt | `render_chrome.rs:309` → ptr::null_mut, render::ime_chrome::row_heights | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `ImeChromeRules.minimumTarget` / ImeChromeRules.kt | `render_chrome.rs:240` → Default::default, ptr::null_mut, render::panels::Command::Subtype, render::panels::Item::new, render::panels::rect | keep: typed JNI/test seam; lifetime or independent event |
| FlorisImeUi.kt:135 | `ImeChromeRules.oneHanded` / ImeChromeRules.kt | `render_chrome.rs:89` → ptr::null_mut, render::ime_chrome::one_handed | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `ImeChromeRules.watermark` / ImeChromeRules.kt | `render_decorations.rs:58` → crate::jni::run, render::ime_decorations::show_watermark, u8::from | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `ImeChromeRules.previewSize` / ImeChromeRules.kt | `render_chrome.rs:193` → ptr::null_mut, render::key_preview::size | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `ImeChromeRules.startEllipsis` / ImeChromeRules.kt | `render_chrome.rs:210` → ptr::null_mut, render::ime_decorations::start_ellipsis | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `ImeChromeRules.extract` / ImeChromeRules.kt | `render_chrome.rs:153` → ptr::null_mut, render::extract_chrome::geometry | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `ImeChromeRules.sideButtons` / ImeChromeRules.kt | `render_chrome.rs:111` → ptr::null_mut, render::ime_chrome::side_buttons | keep: typed JNI/test seam; lifetime or independent event |
| FlorisImeService.kt:264<br>FlorisImeService.kt:343<br>KeyboardEditing.kt:90<br>KeyboardEditing.kt:179<br>KeyboardManager.kt:164 | `InputRules.control` / InputDispatch.kt | `input.rs:146` → input::control::plan, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| KeyboardKeyEvents.kt:46 | `InputRules.route` / InputDispatch.kt | `input.rs:54` → input::routing::route, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| InputFeedbackController.kt:77 | `InputRules.feedback` / InputDispatch.kt | `input.rs:70` → input::feedback::plan, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| ComputingEvaluator.kt:131<br>ComputingEvaluator.kt:152 | `InputRules.visual` / InputDispatch.kt | `input.rs:107` → input::visual::icon, input::visual::label | keep: typed JNI/test seam; lifetime or independent event |
| TextKeyboardLayoutController.kt:220 | `InputRules.hold` / InputDispatch.kt | `input.rs:135` → input::pointer::hold | keep: typed JNI/test seam; lifetime or independent event |
| TextKeyboardLayoutController.kt:122<br>TextKeyboardLayoutController.kt:269 | `InputRules.pointer` / InputDispatch.kt | `input.rs:86` → input::pointer::roll_release, input::pointer::slide | keep: typed JNI/test seam; lifetime or independent event |
| InputDispatch.kt:5 | `InputNative.create` / InputDispatch.kt | `input.rs:9` → input::lifecycle::Dispatcher::default | keep: typed JNI/test seam; lifetime or independent event |
| InputDispatch.kt:14 | `InputNative.free` / InputDispatch.kt | `input.rs:23` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| InputDispatch.kt:12 | `InputNative.call` / InputDispatch.kt | `input.rs:32` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| JavaText.kt:30 | `JavaTextNative.unitFacts` / JavaTextNative.kt | `text.rs:29` → chars::code_point::decomposed_first, chars::to_lower, chars::to_upper, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:35 | `JavaTextNative.codePointFacts` / JavaTextNative.kt | `text.rs:64` → jint::from, ptr::null_mut, u32::try_from | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:39 | `JavaTextNative.lowercase` / JavaTextNative.kt | `text.rs:122` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:41 | `JavaTextNative.uppercase` / JavaTextNative.kt | `text.rs:133` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:43 | `JavaTextNative.trim` / JavaTextNative.kt | `text.rs:144` → chars::trim | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:45 | `JavaTextNative.equalsIgnoreCase` / JavaTextNative.kt | `text.rs:155` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:48 | `JavaTextNative.startsWithIgnoreCase` / JavaTextNative.kt | `text.rs:167` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:50 | `JavaTextNative.isBlank` / JavaTextNative.kt | `text.rs:179` → chars::is_blank | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:52 | `JavaTextNative.codePointCount` / JavaTextNative.kt | `text.rs:194` → chars::text::code_point_count | keep: typed JNI/test seam; lifetime or independent event |
| JavaText.kt:55 | `JavaTextNative.toIntOrNull` / JavaTextNative.kt | `text.rs:211` → chars::text::to_int_or_null | keep: typed JNI/test seam; lifetime or independent event |
| KeyCode.kt:20 | `KeyModels.codes` / KeyModels.kt | `key_models.rs:213` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| KeyboardState.kt:84 | `KeyModels.read` / KeyModels.kt | `key_models.rs:9` → models::read | keep: typed JNI/test seam; lifetime or independent event |
| KeyboardEditing.kt:98<br>KeyboardState.kt:86<br>KeyboardKeyEvents.kt:58 | `KeyModels.write` / KeyModels.kt | `key_models.rs:24` → models::write | keep: typed JNI/test seam; lifetime or independent event |
| KeyData.kt:290<br>TextKeyData.kt:167 | `KeyModels.select` / KeyModels.kt | `key_models.rs:42` → models::select | keep: typed JNI/test seam; lifetime or independent event |
| CurrencySet.kt:39 | `KeyModels.isCurrency` / KeyModels.kt | `key_models.rs:53` → models::is_currency | keep: typed JNI/test seam; lifetime or independent event |
| CurrencySet.kt:44 | `KeyModels.currencySlot` / KeyModels.kt | `key_models.rs:62` → models::currency_slot | keep: typed JNI/test seam; lifetime or independent event |
| CurrencySet.kt:33 | `KeyModels.currencyDefaults` / KeyModels.kt | `key_models.rs:71` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| KeyData.kt:100 | `KeyModels.isSpace` / KeyModels.kt | `key_models.rs:84` → models::is_space | keep: typed JNI/test seam; lifetime or independent event |
| TextKeyData.kt:74 | `KeyModels.predefined` / KeyModels.kt | `key_models.rs:94` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| TextKeyData.kt:80 | `KeyModels.codeKind` / KeyModels.kt | `key_models.rs:111` → models::code_kind | keep: typed JNI/test seam; lifetime or independent event |
| CurrencySet.kt:34<br>TextKeyData.kt:232 | `KeyModels.text` / KeyModels.kt | `key_models.rs:120` → models::text, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| TextKeyData.kt:82<br>TextKeyData.kt:223 | `KeyModels.multiText` / KeyModels.kt | `key_models.rs:142` → models::multi_text, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| TextKey.kt:65 | `KeyModels.popupMode` / KeyModels.kt | `key_models.rs:162` → models::popup_mode | keep: typed JNI/test seam; lifetime or independent event |
| TextKey.kt:67 | `KeyModels.popupGroup` / KeyModels.kt | `key_models.rs:171` → models::popup_group, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| TextKey.kt:74 | `KeyModels.popupSource` / KeyModels.kt | `key_models.rs:188` → models::popup_source | keep: typed JNI/test seam; lifetime or independent event |
| KeyboardComputingEvaluator.kt:74<br>KeyboardManager.kt:212 | `KeyModels.enabled` / KeyModels.kt | `key_models.rs:222` → models::enabled | keep: typed JNI/test seam; lifetime or independent event |
| KeyboardComputingEvaluator.kt:77 | `KeyModels.visible` / KeyModels.kt | `key_models.rs:233` → models::visible | keep: typed JNI/test seam; lifetime or independent event |
| TextKey.kt:167 | `KeyModels.phoneHint` / KeyModels.kt | `key_models.rs:198` → models::phone_hint, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| KeyboardRenderer.kt:5 | `RenderNative.create` / KeyboardRenderer.kt | `render.rs:44` → Session::default, render::text::Text::new, std::fs::read | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:10 | `RenderNative.theme` / KeyboardRenderer.kt | `render.rs:68` → crate::render_theme::theme | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:14 | `RenderNative.update` / KeyboardRenderer.kt | `render.rs:92` → Default::default, render::smartbar_scene::Smartbar::prepare | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:17 | `RenderNative.decorations` / KeyboardRenderer.kt | `render_decorations.rs:9` → render::ime_decorations::prepare, render::ime_decorations::show_watermark | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:20 | `RenderNative.chrome` / KeyboardRenderer.kt | `render_chrome.rs:10` → Default::default | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:22 | `RenderNative.keyPreview` / KeyboardRenderer.kt | `render_key_preview.rs:15` → Default::default, crate::render_chrome::argb, ptr::null_mut, render::key_preview::size, render::raster::Raster::new | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:24 | `RenderNative.chromeInfo` / KeyboardRenderer.kt | `render_chrome.rs:134` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:26 | `RenderNative.pixels` / KeyboardRenderer.kt | `render_chrome.rs:60` → ptr::null_mut, render::raster::Raster::new | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:29 | `RenderNative.subtypes` / KeyboardRenderer.kt | `render_subtypes.rs:9` → Default::default | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:32 | `RenderNative.search` / KeyboardRenderer.kt | `render_search.rs:33` → Default::default | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:34 | `RenderNative.gifSent` / KeyboardRenderer.kt | `render_gif.rs:75` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:36 | `RenderNative.gifs` / KeyboardRenderer.kt | `render_gif.rs:10` → Default::default | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:38 | `RenderNative.mediaRepeat` / KeyboardRenderer.kt | `render_media.rs:71` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:40 | `RenderNative.mediaRequests` / KeyboardRenderer.kt | `render_media.rs:6` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:43 | `RenderNative.mediaUpload` / KeyboardRenderer.kt | `render_media.rs:24` → render::media_cache::Frame::argb, u8::from | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:45 | `RenderNative.clipboard` / KeyboardRenderer.kt | `render_clipboard.rs:10` → ClipboardPanel::default, Default::default | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:48 | `RenderNative.emoji` / KeyboardRenderer.kt | `render_emoji.rs:10` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:50 | `RenderNative.panels` / KeyboardRenderer.kt | `render_panels.rs:50` → Panel::default, render::arrangement::Editor::new | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:52 | `RenderNative.panelArrangement` / KeyboardRenderer.kt | `render_panels.rs:151` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:55 | `RenderNative.smartbar` / KeyboardRenderer.kt | `render_smartbar.rs:15` → Config::decode, Default::default, Smartbar::new, crate::render::strings | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:59 | `RenderNative.semanticConfig` / KeyboardRenderer.kt | `render_accessibility.rs:34` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:61 | `RenderNative.semanticSnapshot` / KeyboardRenderer.kt | `render_accessibility.rs:76` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:63 | `RenderNative.semanticAction` / KeyboardRenderer.kt | `render_accessibility.rs:98` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:65 | `RenderNative.semanticCancel` / KeyboardRenderer.kt | `render_accessibility.rs:13` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:69 | `RenderNative.touch` / KeyboardRenderer.kt | `render_input.rs:57` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:72 | `RenderNative.tooltipShown` / KeyboardRenderer.kt | `render_smartbar.rs:253` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:75 | `RenderNative.candidateBindings` / KeyboardRenderer.kt | `render_smartbar.rs:231` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:78 | `RenderNative.gestures` / KeyboardRenderer.kt | `render_input.rs:13` → GestureConfig::decode | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:80 | `RenderNative.longPress` / KeyboardRenderer.kt | `render_input.rs:34` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:83 | `RenderNative.tick` / KeyboardRenderer.kt | `render_smartbar.rs:136` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:85 | `RenderNative.nextDeadline` / KeyboardRenderer.kt | `render_smartbar.rs:175` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:87 | `RenderNative.barSettings` / KeyboardRenderer.kt | `render_smartbar.rs:200` → ptr::null_mut | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:90 | `RenderNative.candidateLongPress` / KeyboardRenderer.kt | `render_smartbar.rs:119` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:93 | `RenderNative.popupGrid` / KeyboardRenderer.kt | `render.rs:373` → render::popup_scene::configure | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:96 | `RenderNative.popup` / KeyboardRenderer.kt | `render.rs:229` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:99 | `RenderNative.attach` / KeyboardRenderer.kt | `render.rs:276` → crate::render_window::Worker::new | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:102 | `RenderNative.frame` / KeyboardRenderer.kt | `render.rs:308` → u8::from | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:105 | `RenderNative.detach` / KeyboardRenderer.kt | `render.rs:344` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| KeyboardRenderer.kt:114 | `RenderNative.close` / KeyboardRenderer.kt | `render.rs:361` → native scalar/handle transport | keep Android/render event: distinct surface/UI/test seam; see H1 |
| RendererSmartbar.kt:47 | `SmartbarRules.layout` / KeyboardRenderer.kt | `render_smartbar.rs:80` → Config::decode, ptr::null_mut, smartbar::layout | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `SmartbarRules.ellipsis` / KeyboardRenderer.kt | `render_smartbar.rs:104` → smartbar::ellipsis | keep: typed JNI/test seam; lifetime or independent event |
| Keys.kt:15 | `KeysNative.flex` / KeysNative.kt | `keys.rs:47` → keys::flex, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Keys.kt:32 | `KeysNative.desiredKey` / KeysNative.kt | `keys.rs:65` → keys::desired_key, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Keys.kt:48 | `KeysNative.layout` / KeysNative.kt | `keys.rs:97` → keys::layout, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Keys.kt:55 | `KeysNative.keyAt` / KeysNative.kt | `keys.rs:159` → keys::key_at | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:68 | `LatinSessionNative.open` / LatinSession.kt | `suggest/session.rs:54` → Session::open | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:65 | `LatinSessionNative.free` / LatinSession.kt | `suggest/session.rs:72` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:15 | `LatinSessionNative.probability` / LatinSession.kt | `suggest/session.rs:81` → native scalar/handle transport | merge production learner/glide path (C1/G1); retain lookup test seam |
| LatinSession.kt:17 | `LatinSessionNative.ngram` / LatinSession.kt | `suggest/session.rs:95` → native scalar/handle transport | merge production learner/glide path (C1/G1); retain lookup test seam |
| LatinSession.kt:21 | `LatinSessionNative.words` / LatinSession.kt | `suggest/session.rs:112` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:20 | `LatinSessionNative.wordLengths` / LatinSession.kt | `suggest/session.rs:126` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:27 | `LatinSessionNative.decode` / LatinSession.kt | `suggest/session.rs:153` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:45 | `LatinSessionNative.suggest` / LatinSession.kt | `suggest/session.rs:177` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:54 | `LatinSessionNative.spell` / LatinSession.kt | `suggest/session.rs:217` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LayoutAssembly.kt:10 | `LayoutRules.plan` / LayoutRules.kt | `layout_rules.rs:8` → assembly::plan, ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| LayoutAssembly.kt:14 | `LayoutRules.modifierType` / LayoutRules.kt | `layout_rules.rs:23` → assembly::modifier_type | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `LayoutRules.mergeRows` / LayoutRules.kt | `layout_rules.rs:32` → assembly::merge, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LayoutAssembly.kt:22 | `LayoutRules.hints` / LayoutRules.kt | `layout_rules.rs:56` → assembly::hints, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LayoutCache.kt:6<br>LayoutCache.kt:7 | `LayoutRules.cacheKey` / LayoutRules.kt | `layout_rules.rs:82` → assembly::cache_key | keep: typed JNI/test seam; lifetime or independent event |
| LayoutCache.kt:13 | `LayoutRules.invalidated` / LayoutRules.kt | `layout_rules.rs:92` → assembly::invalidated, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `MediaRules.scrollChips` / MediaRules.kt | `media_oracle.rs:93` → ptr::null_mut, render::gif_panel::scroll_chips | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `MediaRules.grid` / MediaRules.kt | `media_oracle.rs:69` → ptr::null_mut, render::gif_panel::grid | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `MediaRules.trace` / MediaRules.kt | `media_oracle.rs:7` → Default::default, Frame::argb, MediaCache::default, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:12<br>NlpSession.kt:29 | `NlpNative.isName` / NlpSession.kt | `nlp_stores.rs:11` → nlp::stores::is_name, u8::from | keep: typed JNI/test seam; lifetime or independent event |
| LatinSession.kt:14 | `NlpNative.unknownWords` / NlpSession.kt | `nlp_stores.rs:36` → nlp::stores::unknown_words, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| NlpSession.kt:5 | `NlpNative.create` / NlpSession.kt | `nlp.rs:10` → nlp::Session::default | keep: typed JNI/test seam; lifetime or independent event |
| NlpSession.kt:14 | `NlpNative.free` / NlpSession.kt | `nlp.rs:22` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| NlpSession.kt:12 | `NlpNative.call` / NlpSession.kt | `nlp.rs:31` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| native declaration/test seam | `PanelRules.clipboardDialog` / PanelRules.kt | `panel_oracle.rs:273` → ptr::null_mut, render::clipboard_dialog::geometry, render::panels::rect | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.clipboardTargets` / PanelRules.kt | `panel_oracle.rs:229` → Default::default, ptr::null_mut, render::panels::Command::Clipboard, render::panels::Item::new, render::panels::rect | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.clipboardMenu` / PanelRules.kt | `panel_oracle.rs:172` → Default::default, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.clipboardGrid` / PanelRules.kt | `panel_oracle.rs:146` → ptr::null_mut, render::clipboard_panel::columns, render::clipboard_panel::lane | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.overflowAndHistoryMoves` / PanelRules.kt | `panel_oracle.rs:121` → Default::default, ptr::null_mut, render::panel_geometry::history_moves | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.scroll` / PanelRules.kt | `panel_oracle.rs:98` → ptr::null_mut, render::panel_scroll::decay, render::panel_scroll::fraction | keep: typed JNI/test seam; lifetime or independent event |
| KeyboardEditing.kt:44<br>KeyboardEditing.kt:58 | `PanelRules.selection` / PanelRules.kt | `panel_oracle.rs:80` → ptr::null_mut, render::editing::selection | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.editing` / PanelRules.kt | `panel_oracle.rs:6` → ptr::null_mut, render::editing::closes, render::editing::hold_delay, render::editing::repeat_code, render::editing::repeat_delay, render::editing::repeatable | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.cells` / PanelRules.kt | `panel_oracle.rs:36` → ptr::null_mut, render::panel_geometry::cells | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `PanelRules.emojiChoices` / PanelRules.kt | `panel_oracle.rs:58` → ptr::null_mut, render::panel_geometry::base, render::panel_geometry::variants | keep: typed JNI/test seam; lifetime or independent event |
| PanelRules.kt:17 | `PanelPolicyNative.history` / PanelRules.kt | `panel_rules.rs:7` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| PanelRules.kt:19 | `PanelPolicyNative.arrange` / PanelRules.kt | `panel_rules.rs:45` → Editor::new, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Popup.kt:53 | `PopupNative.extend` / PopupNative.kt | `popup.rs:62` → grid::extend, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Popup.kt:71 | `PopupNative.pointer` / PopupNative.kt | `popup.rs:105` → grid::pointer | keep: typed JNI/test seam; lifetime or independent event |
| Popup.kt:81 | `PopupNative.elementAt` / PopupNative.kt | `popup.rs:136` → Key::NumberPopup, Key::Relevant, Key::SymbolPopup, grid::element_at, jlong::from | keep: typed JNI/test seam; lifetime or independent event |
| Popup.kt:123 | `PopupNative.order` / PopupNative.kt | `popup.rs:170` → Other::Keys, order::order, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Popup.kt:143 | `PopupNative.merge` / PopupNative.kt | `popup.rs:221` → Step::Add, order::merge, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:16 | `ReaderNative.maxUtterance` / ReaderNative.kt | `reader.rs:50` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:19 | `ReaderNative.recent` / ReaderNative.kt | `reader.rs:58` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:104 | `ReaderNative.holdMs` / ReaderNative.kt | `reader.rs:66` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:26 | `ReaderNative.firstOnScreen` / ReaderNative.kt | `reader.rs:76` → page_text::first_on_screen | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:30 | `ReaderNative.allShown` / ReaderNative.kt | `reader.rs:95` → page_text::all_shown | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:35 | `ReaderNative.join` / ReaderNative.kt | `reader.rs:116` → page_text::blocks, ptr::null_mut | merge per-string JNI transfer into packed UTF-16 (R1) |
| Reader.kt:47 | `ReaderNative.find` / ReaderNative.kt | `reader.rs:154` → page_text::find, ptr::null_mut | merge per-string JNI transfer into packed UTF-16 (R1) |
| Reader.kt:51 | `ReaderNative.resumeAfter` / ReaderNative.kt | `reader.rs:185` → page_text::resume_after | merge per-string JNI transfer into packed UTF-16 (R1) |
| Reader.kt:58 | `ReaderNative.utterances` / ReaderNative.kt | `reader.rs:206` → page_text::utterances, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:61 | `ReaderNative.fits` / ReaderNative.kt | `reader.rs:229` → follow_along::fits | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:77 | `ReaderNative.newSpeakerKey` / ReaderNative.kt | `reader.rs:242` → SpeakerKey::default | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:99 | `ReaderNative.freeSpeakerKey` / ReaderNative.kt | `reader.rs:250` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:85 | `ReaderNative.speakerKeyDown` / ReaderNative.kt | `reader.rs:271` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:88 | `ReaderNative.speakerKeyUp` / ReaderNative.kt | `reader.rs:288` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Reader.kt:92 | `ReaderNative.speakerKeyCancel` / ReaderNative.kt | `reader.rs:304` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:59 | `ReaderWalk.walk` / ReaderRuntime.kt | `reader_runtime.rs:66` → Walk::run, ptr::null_mut | keep Android API: demand-driven traversal; see C3 |
| ReaderRuntime.kt:17 | `ReaderWalk.followConfig` / ReaderRuntime.kt | `reader_runtime.rs:102` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| ReaderRuntime.kt:24 | `PlaybackNative.create` / ReaderRuntime.kt | `reader_runtime.rs:119` → PlayQueue::default | keep: typed JNI/test seam; lifetime or independent event |
| ReaderRuntime.kt:32 | `PlaybackNative.call` / ReaderRuntime.kt | `reader_runtime.rs:135` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| ReaderRuntime.kt:36 | `PlaybackNative.system` / ReaderRuntime.kt | `reader_runtime.rs:152` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| ReaderRuntime.kt:38 | `PlaybackNative.free` / ReaderRuntime.kt | `reader_runtime.rs:126` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `RenderContentRules.phoneLabel` / RenderContentRules.kt | `render_content.rs:7` → ptr::null_mut, render::key_content::phone_label, render::key_content::telpad | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:29 | `ScreenNative.isSideways` / ScreenNative.kt | `screen.rs:64` → screen::is_sideways | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:32 | `ScreenNative.visibleArea` / ScreenNative.kt | `screen.rs:85` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:35 | `ScreenNative.visiblePart` / ScreenNative.kt | `screen.rs:102` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:42 | `ScreenNative.mainScroller` / ScreenNative.kt | `screen.rs:120` → screen::main_scroller | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:49 | `ScreenNative.inList` / ScreenNative.kt | `screen.rs:140` → ptr::null_mut, screen::is_within | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:53 | `ScreenNative.moveIntoView` / ScreenNative.kt | `screen.rs:165` → screen::move_into_view | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:57 | `ScreenNative.afterTurn` / ScreenNative.kt | `screen.rs:184` → screen::after_turn | keep: typed JNI/test seam; lifetime or independent event |
| ScreenWalk.kt:61 | `ScreenNative.isInside` / ScreenNative.kt | `screen.rs:201` → screen::is_inside | keep: typed JNI/test seam; lifetime or independent event |
| SettingsRules.kt:5 | `SettingsNative.call` / SettingsRules.kt | `backup_utils.rs:50` → native scalar/handle transport | keep: already one plan/exchange; Android effects outside call |
| SherpaAsr.kt:58 | `SherpaNative.audioConfig` / SherpaAsr.kt | `sherpa.rs:139` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| SherpaAsr.kt:10 | `SherpaNative.createRecognizer` / SherpaAsr.kt | `sherpa.rs:13` → Recognizer::new | keep: typed JNI/test seam; lifetime or independent event |
| SherpaAsr.kt:18 | `SherpaNative.transcribe` / SherpaAsr.kt | `sherpa.rs:51` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| SherpaAsr.kt:20 | `SherpaNative.cancel` / SherpaAsr.kt | `sherpa.rs:37` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| SherpaAsr.kt:21 | `SherpaNative.freeRecognizer` / SherpaAsr.kt | `sherpa.rs:28` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| SherpaAsr.kt:28 | `SherpaNative.createVad` / SherpaAsr.kt | `sherpa.rs:69` → Vad::new | keep: typed JNI/test seam; lifetime or independent event |
| SherpaAsr.kt:36 | `SherpaNative.vad` / SherpaAsr.kt | `sherpa.rs:117` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| SherpaAsr.kt:42 | `SherpaNative.freeVad` / SherpaAsr.kt | `sherpa.rs:84` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| SherpaTts.kt:11 | `TtsNative.create` / SherpaTts.kt | `tts.rs:26` → Tts::new, reader::wave::pcm16 | keep: typed JNI/test seam; lifetime or independent event |
| SherpaTts.kt:13 | `TtsNative.rate` / SherpaTts.kt | `tts.rs:54` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| SherpaTts.kt:17 | `TtsNative.generate` / SherpaTts.kt | `tts.rs:105` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `TtsNative.cancel` / SherpaTts.kt | `tts.rs:63` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| SherpaTts.kt:28 | `TtsNative.close` / SherpaTts.kt | `tts.rs:84` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Shift.kt:42 | `ShiftNative.next` / ShiftNative.kt | `shift.rs:83` → shift::next | keep: typed JNI/test seam; lifetime or independent event |
| Shift.kt:47 | `ShiftNative.isConsecutive` / ShiftNative.kt | `shift.rs:104` → shift::is_consecutive | keep: typed JNI/test seam; lifetime or independent event |
| Shift.kt:50 | `ShiftNative.isDoubleSpacePeriod` / ShiftNative.kt | `shift.rs:120` → shift::is_double_space_period | keep: typed JNI/test seam; lifetime or independent event |
| SnyggCompilerAdapter.kt:14 | `SnyggCompiler.call` / SnyggCompiler.kt | `snygg_resolve.rs:194` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| SnyggGrammarAdapter.kt:11<br>SnyggGrammarAdapter.kt:13 | `SnyggGrammar.call` / SnyggGrammar.kt | `snygg.rs:244` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| SnyggPropertyCodec.kt:16<br>SnyggPropertyCodec.kt:35<br>NativeSnyggValues.kt:50 | `SnyggValues.call` / SnyggValues.kt | `snygg_value.rs:249` → ptr::null_mut | keep: already one plan/exchange; Android effects outside call |
| native declaration/test seam | `SpacingNative.commitChar` / SpacingNative.kt | `spacing.rs:147` → spaces::commit_char | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `SpacingNative.phantomSpaceBefore` / SpacingNative.kt | `spacing.rs:182` → spaces::phantom_space_before | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:97 | `SpacingNative.supportsAutoSpace` / SpacingNative.kt | `spacing.rs:217` → spaces::supports_auto_space | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:100 | `SpacingNative.fallbackRule` / SpacingNative.kt | `spacing.rs:233` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:103 | `SpacingNative.inactive` / SpacingNative.kt | `spacing.rs:255` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:106 | `SpacingNative.autoSpaceActive` / SpacingNative.kt | `spacing.rs:264` → state::auto_space_active | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:110 | `SpacingNative.phantomSpaceActive` / SpacingNative.kt | `spacing.rs:274` → state::phantom_space_active | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:113 | `SpacingNative.isActive` / SpacingNative.kt | `spacing.rs:288` → state::is_active | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:116 | `SpacingNative.showsComposingRegion` / SpacingNative.kt | `spacing.rs:298` → state::shows_composing_region | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:119 | `SpacingNative.afterUpdate` / SpacingNative.kt | `spacing.rs:308` → state::after_update | keep: typed JNI/test seam; lifetime or independent event |
| Spacing.kt:122 | `SpacingNative.keepsCandidate` / SpacingNative.kt | `spacing.rs:318` → state::keeps_candidate | keep: typed JNI/test seam; lifetime or independent event |
| native declaration/test seam | `SubtypeRules.geometry` / SubtypeRules.kt | `render_subtypes.rs:75` → crate::jni::run, render::modal_scene::offset, render::subtype_panel::content_height, render::subtype_panel::sheet_start, std::ptr::null_mut | keep: settings excluded by task; existing bulk plan |
| native declaration/test seam | `SubtypeRules.rows` / SubtypeRules.kt | `render_subtypes.rs:49` → crate::jni::run, render::subtype_panel::rows, std::ptr::null_mut | keep: settings excluded by task; existing bulk plan |
| Suggest.kt:38 | `SuggestNative.continuesWord` / SuggestNative.kt | `suggest.rs:63` → word_characters::continues_word | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:41 | `SuggestNative.wordCase` / SuggestNative.kt | `suggest.rs:73` → WordCase::of | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:44 | `SuggestNative.applyCase` / SuggestNative.kt | `suggest.rs:90` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:47 | `SuggestNative.isMostlyCaps` / SuggestNative.kt | `suggest.rs:108` → word_case::is_mostly_caps | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:50 | `SuggestNative.decoderInput` / SuggestNative.kt | `suggest.rs:125` → ptr::null_mut, word_case::decoder_input | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:25 | `SuggestNative.maxPreviousWords` / SuggestNative.kt | `suggest.rs:171` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:27 | `SuggestNative.previousSlotCount` / SuggestNative.kt | `suggest.rs:295` → previous_words::slot_count | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:29 | `SuggestNative.isCursorMidWord` / SuggestNative.kt | `suggest.rs:46` → word_characters::cursor_mid_word | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:54 | `SuggestNative.previousWords` / SuggestNative.kt | `suggest.rs:141` → previous_words::PreviousWords::from, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:22 | `SuggestNative.modestThreshold` / SuggestNative.kt | `suggest.rs:162` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:63 | `SuggestNative.exceedsThreshold` / SuggestNative.kt | `suggest.rs:181` → auto_correction::suggestion_exceeds_threshold | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:67 | `SuggestNative.normalizedScore` / SuggestNative.kt | `suggest.rs:211` → auto_correction::normalized_score | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:70 | `SuggestNative.editDistance` / SuggestNative.kt | `suggest.rs:233` → auto_correction::edit_distance, u32::try_from | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:77 | `SuggestNative.revertAt` / SuggestNative.kt | `suggest.rs:263` → ptr::null_mut, undo::revert_at | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:32 | `SuggestNative.tapLimit` / SuggestNative.kt | `suggest/tap.rs:17` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:35 | `SuggestNative.tapTrailCapacity` / SuggestNative.kt | `suggest/tap.rs:26` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:87 | `SuggestNative.tapOffset` / SuggestNative.kt | `suggest/tap.rs:35` → TapOffset::of, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:123 | `SuggestNative.newTapTrail` / SuggestNative.kt | `suggest/tap.rs:63` → TapTrail::new | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:161 | `SuggestNative.freeTapTrail` / SuggestNative.kt | `suggest/tap.rs:79` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:131 | `SuggestNative.tapPressed` / SuggestNative.kt | `suggest/tap.rs:105` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:137 | `SuggestNative.tapTyped` / SuggestNative.kt | `suggest/tap.rs:125` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:143 | `SuggestNative.tapDeleted` / SuggestNative.kt | `suggest/tap.rs:142` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:149 | `SuggestNative.tapCleared` / SuggestNative.kt | `suggest/tap.rs:158` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:158 | `SuggestNative.tapOffsetsFor` / SuggestNative.kt | `suggest/tap.rs:175` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:92 | `SuggestNative.qwertySizes` / SuggestNative.kt | `suggest/tap.rs:200` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:102 | `SuggestNative.qwertyKeys` / SuggestNative.kt | `suggest/tap.rs:219` → ptr::null_mut, qwerty::keys | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:105 | `SuggestNative.keyCenter` / SuggestNative.kt | `suggest/tap.rs:232` → ptr::null_mut, qwerty::key_center | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:112 | `SuggestNative.touchPoint` / SuggestNative.kt | `suggest/tap.rs:251` → ptr::null_mut, qwerty::touch_point | keep: typed JNI/test seam; lifetime or independent event |
| Suggest.kt:115 | `SuggestNative.proximityGrid` / SuggestNative.kt | `suggest/tap.rs:274` → ptr::null_mut, qwerty::proximity_grid | keep: typed JNI/test seam; lifetime or independent event |
| Suggester.kt:74 | `SuggesterNative.suggest` / SuggesterNative.kt | `suggest/suggester.rs:156` → JavaDictionary::new, ptr::null_mut | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:96 | `SuggesterNative.spell` / SuggesterNative.kt | `suggest/suggester.rs:218` → JavaDictionary::new, ptr::null_mut | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:112 | `SuggesterNative.newLearner` / SuggesterNative.kt | `suggest/word_learner.rs:19` → WordLearner::default | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:145 | `SuggesterNative.freeLearner` / SuggesterNative.kt | `suggest/word_learner.rs:27` → JavaLookup::new | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:120 | `SuggesterNative.learnerFinished` / SuggesterNative.kt | `suggest/word_learner.rs:68` → native scalar/handle transport | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:125 | `SuggesterNative.learnerSwiped` / SuggesterNative.kt | `suggest/word_learner.rs:93` → native scalar/handle transport | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:130 | `SuggesterNative.learnerDropSwiped` / SuggesterNative.kt | `suggest/word_learner.rs:116` → native scalar/handle transport | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:136 | `SuggesterNative.learnerUndo` / SuggesterNative.kt | `suggest/word_learner.rs:139` → native scalar/handle transport | remove production callback: native learner; keep host oracle seam |
| Suggester.kt:140 | `SuggesterNative.learnerReset` / SuggesterNative.kt | `suggest/word_learner.rs:156` → native scalar/handle transport | remove production callback: native learner; keep host oracle seam |
| Swipe.kt:14 | `SwipeNative.maxGlideBonus` / SwipeNative.kt | `suggest/swipe.rs:17` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:32 | `SwipeNative.glideBonus` / SwipeNative.kt | `suggest/swipe.rs:26` → glide_ranker::bonus | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:35 | `SwipeNative.rankGlide` / SwipeNative.kt | `suggest/swipe.rs:43` → glide_ranker::rank, ptr::null_mut | merge: gather native context and rank once (G1) |
| Swipe.kt:16 | `SwipeNative.maxProbability` / SwipeNative.kt | `suggest/swipe.rs:88` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:19 | `SwipeNative.common` / SwipeNative.kt | `suggest/swipe.rs:96` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:22 | `SwipeNative.costPerStep` / SwipeNative.kt | `suggest/swipe.rs:104` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:25 | `SwipeNative.learnedProbability` / SwipeNative.kt | `suggest/swipe.rs:112` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:42 | `SwipeNative.isSwipeable` / SwipeNative.kt | `suggest/swipe.rs:121` → swipe_words::is_swipeable | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:51 | `SwipeNative.swipeable` / SwipeNative.kt | `suggest/swipe.rs:138` → ptr::null_mut, swipe_words::is_swipeable | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:55 | `SwipeNative.frequencyOf` / SwipeNative.kt | `suggest/swipe.rs:162` → swipe_words::frequency_of | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:58 | `SwipeNative.frequency` / SwipeNative.kt | `suggest/swipe.rs:162` → swipe_words::frequency_of | keep: typed JNI/test seam; lifetime or independent event |
| Swipe.kt:66 | `SwipeNative.userWords` / SwipeNative.kt | `suggest/swipe.rs:185` → ptr::null_mut, swipe_words::user_words | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:12 | `WordStoresNative.sentenceStart` / WordStoresNative.kt | `suggest/learned_words.rs:17` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:15 | `WordStoresNative.knownAfter` / WordStoresNative.kt | `suggest/learned_words.rs:31` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:18 | `WordStoresNative.learnedWordCapacity` / WordStoresNative.kt | `suggest/learned_words.rs:40` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:21 | `WordStoresNative.learnedPairCapacity` / WordStoresNative.kt | `suggest/learned_words.rs:49` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:24 | `WordStoresNative.maxWordLength` / WordStoresNative.kt | `suggest/learned_words.rs:58` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:27 | `WordStoresNative.keptCapacity` / WordStoresNative.kt | `suggest/kept_words.rs:17` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:30 | `WordStoresNative.isStorable` / WordStoresNative.kt | `suggest/learned_words.rs:67` → learned_words::is_storable | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:37 | `WordStoresNative.previousKey` / WordStoresNative.kt | `suggest/learned_words.rs:85` → learned_words::previous_key, ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:40 | `WordStoresNative.appliesTo` / WordStoresNative.kt | `suggest/personal_words.rs:15` → personal_words::applies_to | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:53 | `WordStoresNative.newLearnedWords` / WordStoresNative.kt | `suggest/learned_words.rs:105` → LearnedWords::new | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:111 | `WordStoresNative.freeLearnedWords` / WordStoresNative.kt | `suggest/learned_words.rs:115` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:61 | `WordStoresNative.learnedAdd` / WordStoresNative.kt | `suggest/learned_words.rs:141` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:67 | `WordStoresNative.learnedRemove` / WordStoresNative.kt | `suggest/learned_words.rs:162` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:72 | `WordStoresNative.learnedClear` / WordStoresNative.kt | `suggest/learned_words.rs:181` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:78 | `WordStoresNative.learnedKnownSpellings` / WordStoresNative.kt | `suggest/learned_words.rs:197` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:82 | `WordStoresNative.learnedSpellingOf` / WordStoresNative.kt | `suggest/learned_words.rs:212` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:89 | `WordStoresNative.learnedCompletions` / WordStoresNative.kt | `suggest/learned_words.rs:229` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:93 | `WordStoresNative.learnedFollowers` / WordStoresNative.kt | `suggest/learned_words.rs:248` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:98 | `WordStoresNative.learnedPairCount` / WordStoresNative.kt | `suggest/learned_words.rs:274` → native scalar/handle transport | merge production learner/glide path (C1/G1); retain lookup test seam |
| WordStores.kt:103 | `WordStoresNative.learnedMerge` / WordStoresNative.kt | `suggest/learned_words.rs:294` → Saved::parse | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:108 | `WordStoresNative.learnedSavedText` / WordStoresNative.kt | `suggest/learned_words.rs:312` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:120 | `WordStoresNative.newKeptWords` / WordStoresNative.kt | `suggest/kept_words.rs:26` → KeptWords::new | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:144 | `WordStoresNative.freeKeptWords` / WordStoresNative.kt | `suggest/kept_words.rs:35` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:128 | `WordStoresNative.keptContains` / WordStoresNative.kt | `suggest/kept_words.rs:61` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:132 | `WordStoresNative.keptAdd` / WordStoresNative.kt | `suggest/kept_words.rs:77` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:136 | `WordStoresNative.keptMerge` / WordStoresNative.kt | `suggest/kept_words.rs:95` → KeptWords::parse | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:141 | `WordStoresNative.keptSavedText` / WordStoresNative.kt | `suggest/kept_words.rs:113` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:154 | `WordStoresNative.newPersonalWords` / WordStoresNative.kt | `suggest/personal_words.rs:37` → PersonalWords::new | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:172 | `WordStoresNative.freePersonalWords` / WordStoresNative.kt | `suggest/personal_words.rs:69` → native scalar/handle transport | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:161 | `WordStoresNative.personalOrder` / WordStoresNative.kt | `suggest/personal_words.rs:99` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:165 | `WordStoresNative.personalSpellingOf` / WordStoresNative.kt | `suggest/personal_words.rs:116` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |
| WordStores.kt:169 | `WordStoresNative.personalCompletions` / WordStoresNative.kt | `suggest/personal_words.rs:133` → ptr::null_mut | keep: typed JNI/test seam; lifetime or independent event |

## Proof and results

Each logical change preserves independent Kotlin references and existing assertions. Mutations back up and restore exact source bytes. Bulk/hot changes use paired APK measurements on aw_keyboard_ssd; both-renderer replays and relevant package fixtures are recorded below. The exact full gate must exit zero before each push. Latin-oracle is unchanged.

## Open items

C1/C2, G1, R1, W1–W5 and the pure SearchText part of W16 are implemented. C3 is deliberately retained on Android semantics and measured cost. H1–H3 and the other managed owners remain platform/lifetime/test seams. Live GIPHY results need a key; phone/120 Hz renderer acceptance remains outside this package. No merge/remove row is deferred. Final gate/device accounting follows below.

Baseline full gate: exit **0**, 2m 37s; no oracle crash. Evidence: ignored `build/stage2-consolidation/before/gate.log` and `gate.exit`.

## C1: native learning exchange

The production dictionary and null-dictionary paths now call `NativeLearnerNative.call`
once per finished/swiped event. `WordLearner` invokes the existing native learner and
`Session::probability` in process. The session monitor covers the exchange and close;
the learned-store lock is still held only for store effects. No managed callback runs
under a production learner lock. Synthetic `WordLookup` adapters remain reference seams.

Dictionary lookup remains lazy. A closed dictionary does not reject a lowercase/no-op
event that formerly needed no lookup; a spelling lookup fails at the same point with
the same exception, after the same prior store effects. Null and closed handles have
distinct transport representations. Stores and owners remain reachable through JNI.

`ReferenceBoundaryLearner` freezes the old callback adapter. `BoundaryLearnerMatchTest`
compares **30,000 events / 100 seeds** against both it and the independent pre-port
`ReferenceLatin.WordLearner`, including pending swipes, replacement, undo/reset/drop,
null dictionaries, capacities, capitalization and invalid UTF-16. Existing learner and
50,000-event NLP learning comparisons also pass. Returning no probability for every
native lookup causes one fresh assertion failure; restoring the backed-up bytes passes.
Restored source SHA-256: `68f38c6b4293c89077cd8f3a4c1d44bf40be6ddb1c2f358bd5a80849c3b8fcbc`.

The test-only boundary instrumentation measures the same public operations on both
APKs, with 100 warmups and 20 batches; samples include managed model conversion and
result fingerprinting. It uses only synthetic in-memory stores and never opens audio.
The reader reference contributes 6,000 independent Android comparisons per replay.
The initial probe failed because asynchronous application startup deletes `cacheDir`;
its dictionary now uses private `filesDir`, as A1 does. An unused default-constructor
stub had also been removed by R8; supplying the identical default callback explicitly
fixes the test-only linkage without adding retention or changing production code.
The one resulting test-instrumentation fatal is retained in `before/probe-files-runtime.log`.
Neither failed probe is counted as measurement or parity evidence.

Pre-change normal-build Compose/Rust regressions retain both `809d588b…`/`c14d19d2…`
digests; autocorrect, candidate-pick and glide/cancel replays pass on both renderers.
Canonical stores, exact preferences and accessibility restore around every wrapper.
Baseline timing is `before/timing.json`; its full result fingerprint is
`babe9cdc337ec6c3857c5c551ed5534bf428efb82d0b2ba40c1daeb11f358d73`.
All evidence paths here are relative to ignored `build/stage2-consolidation/`.

The verified C1 fixture retains the same full result fingerprint and both accepted A1
decoder/pipeline digests. Learner transaction p50/p95 is **7,441 / 8,664 ns**, versus
**9,340 / 20,457 ns** before. Unmodified controls also varied considerably between
APK runs; these small emulator samples do not establish an end-to-end speedup.
Evidence: `c1-timing-verified/`, `c1-a1-verified/`, `c1-fixture-apk.json`.

The first full C1 gate exits 1 at APK packaging after an overlapping fixture build
removed an intermediate DEX. The overlap also left a stale normal native payload
inside a fixture APK; A1 correctly rejects its missing snapshot export. The sequential
native outputs were correct, but APK packaging stayed up-to-date with old bytes.
Removing only generated packaging state and rebuilding gives an APK whose actual
ELF exports pass `consolidation-apk-check.py --fixture`; A1 then passes. The rejected
APK/timing attempt and its instrumentation fatal are not accepted fixture evidence.
The final full gate is still required before pushing the implementation commits.

## C3 refinement after the inventory

Keep the demand-driven walk and Android adapter, including every child fetch after
the node/depth cap and lazy text fallback. However, facts and class name are always
requested consecutively for a visited node. Those two reads can be batched into one
per-node UTF-16 packet without collecting the whole tree. Facts must be copied before
fetching class name, preserving mutation/exception order. `ReaderBoundaryOrderTest`
freezes the current bridge's password, fallback, malformed-facts and platform-failure
read traces before evaluating the candidate. Existing 10,000 seeded tree comparisons
and exact 20,000-node/250-depth checks remain mandatory. This candidate was subsequently
**rejected on measured cost** below; all reader callbacks remain **keep**.

## C2: reference-only callbacks stay out of normal Android

The old `SuggesterNative.suggest/spell`, callback `learnerFinished/learnerSwiped`
and `DictationNative.join` exports now compile only on the host or with
`legacy-fixture`. Existing Latin/TTS fixture features enable it; the dictation
comparison property enables its own feature too. The shared native-session store
and result transport remains independent of that adapter. Every legacy test and
frozen reference is retained. Consumer rules retain the callback names for opt-in R8
fixtures. Normal production suggestion/learning/dictation paths use their native sessions.

The normal APK contains **zero of those five legacy exports**, while the host/JVM
boundary/A1 checks and Rust checks pass. The actual Cargo default was deliberately
changed to enable `legacy-fixture`: that normal APK compiled, and the packaged-export
checker rejected it. Restoring the backed-up manifest bytes and rebuilding passes.
Manifest restoration SHA: `e619381a98fa2fea1e8a28c419c6c3cb966105c36571baf3f7cdb84378055c3e`.
Evidence: `c2-check.log`, `c2-normal-apk.json`, `c2-mutation/`.

This removes three callback invocation sites from normal Android (generic probability,
decode and name lookup), retaining them only as host/fixture seams. Kotlin external
declarations remain available for those tests. Production Android still has the four
demand-driven reader call sites and the synchronous PCM consumer at this point.
No policy moved in C2; the deliberate feature/export mutation proves the shipping boundary.
C1's post-change primary regression, suggestion and glide/cancel wrappers also pass on
both renderers, with exact canonical restoration (`c1-typing/`).

## C3: retain the measured Android traversal boundary

The per-node facts/class packet candidate preserved all results, mutable-array copy
order, exact platform exception identity, password/text pruning and post-cap child
fetch failures. Swapping its password field caused two match assertions to fail and
the backed-up prototype source was restored. It also passed the 6,000 Android reader
comparisons and actual muted neural playback/stop on both renderers.

It was nevertheless **reverted, not shipped**. Three alternating before/after APK
rounds (60 timing samples per side), using the same test APK and exact result hash,
showed the 513-node synthetic walk p50 **776,743 → 1,040,816.5 ns** and p95
**922,666 → 1,417,900 ns**. Fewer callbacks did not pay for the extra packet packing.
The original reader bridge remains byte-identical to C2; no eager tree snapshot or
new reader wire format is in production. C3 stays **keep: Android API and measured cost**.
The synthetic timing excludes actual AccessibilityNodeInfo/Binder latency; the real
reader UI checks establish platform execution, not a full-screen walk latency claim.
Evidence: `c3-control/`, `c3-paired/comparison.json`, `c3-before-reader/` and
`c3-after-reader-verified/`. Prototype mutation assertions are not counted as shipped
consolidation mutation coverage. The read-order/cap/UTF-16 regressions remain in tests.

The combined opt-in fixture now uses the existing renderer instrumentation for boundary
timing (`--runner renderer`) and live workflows, with `-PsuggestFixture=true
-PrendererFixture=true`. Separate A1-only/renderer-only properties keep their existing
runners. An attempted second manifest runner was rewritten by AGP; that unaccepted
preflight is retained separately, and no extra manifest is shipped. No assertion changed.
C1 live learning/unlearning, password candidate/inline counts and incognito storage
checks pass on both renderers with exact baseline restoration (`c1-learning/`).

## G1: gather and rank a glide cohort in process

`NativeGlide.rank` sends one UTF-16 batch (words followed by previous words), lengths,
costs and native session/store ownership. `suggest::glide_ranker::rank_from` preserves
the early returns and per-word ngram → conditional probability → learned-pair order,
then calls the unchanged stable float ranking. JNI only copies fields and resolves
the requested native lookups. No Android work, JNI callback or native lock across
managed work is introduced. The session monitor serializes close; stores remain reachable.
Synthetic lookup/history adapters remain reference seams.

For N≥2 with context, this replaces **N pair queries + N ngrams + up to N probabilities
+ slot count + rank (up to 3N+2 JNI calls)** with **one** exchange. Empty/single/no-context
requests never query the dictionary/store; closed dictionaries preserve lazy exceptions.

The frozen pre-change `ReferenceBoundaryGlideRanker` matches **10,000 real-dictionary
cohorts**, including missing dictionaries, learned pairs, duplicate/tied words, malformed
UTF-16, NaN, infinities and signed zero. Existing synthetic glide comparisons remain.
Directed closed-handle and malformed-packet tests pass. Removing learned-pair influence
causes one fresh assertion failure; byte restoration passes, SHA
`3e31ba49187ea43f554c1cff3bb8498e6c60fb44bdc88a72213538321cd8d69a`.
Evidence: `g1-tests.log`, `g1-boundaries.log`, `g1-mutation/`. Final device/timing evidence
is recorded with the completed stack below.

## R1: bulk reader text fields and results

Reader join/find/resume now send one UTF-16 string plus lengths; resume packs both
lists with an explicit split index. Join returns one string plus count/first-piece/
top/bottom/length metadata. This shares `Utf16Batch` with G1. The Java-free reader
decisions and Android node callbacks, sentence segmentation and reveal ownership
are unchanged. Empty fields and surrogate boundaries remain explicit, never delimiters.

Existing frozen reader comparisons pass: 40,000 page, 40,000 join and 40,000 scroll
cohorts, plus the retained speech/view/key tests. New boundary checks preserve every
UTF-16 unit across 65,536 separate fields and indexes beyond 65,535, and reject bad
length/count packets. All 65,536 round trips also pass on Android, alongside 6,000
independent reader comparisons per device replay. Shifting first-piece metadata by
one causes a fresh assertion failure; exact restoration passes (historical mutation
source SHA `d009e6217c628b31c25057b3a7edbcb3ad4c5cc332f9e1312cf59b0cabe26a92`).

Three alternating original-baseline/current APK rounds use one test APK, 100 warmups
and 20 batches per round: 60 samples per side. Every result fingerprint matches the
original `babe9cdc…` value, with exact canonical restoration. These are per-operation
synthetic emulator costs, including managed projection/fingerprinting, not phone or
touch-to-present acceptance. The unchanged walk is a useful noise control.

| Public operation | Before p50 / p95 (µs) | After p50 / p95 (µs) |
| --- | ---: | ---: |
| Finished/pending-swipe learner transaction | 7.00 / 7.67 | 6.19 / 6.69 |
| Glide rank, 32 guesses | 68.34 / 91.31 | 47.38 / 51.22 |
| Reader join, 512 pieces | 82.87 / 98.73 | 66.90 / 82.41 |
| Reader find, 512 blocks | 98.54 / 103.58 | 80.90 / 94.10 |
| Reader resume, 512 blocks | 54.38 / 69.26 | 38.48 / 44.33 |
| Unchanged 513-node callback walk | 729.88 / 787.68 | 749.67 / 787.95 |

Both renderer reader UI checks pass after R1: real muted neural playback advances,
stop returns IDLE and the masked fixture editor stays empty. Evidence: `r1-tests.log`,
`r1-boundaries.log`, `r1-mutation/`, `r1-paired/`, `r1-reader/` and `r1-evidence/`.
The APK checker now also compares actual packaged native bytes with the compiled
stripped artifact to reject stale package contents before device execution.

## W1–W5 and W16: remove redundant production facades

AutoCorrection, WordCharacters, WordCase and ScreenRules/Box now live only in test
sources, with their original forwarding implementations. FollowAlong.fits is a test-only
extension; FollowAlong itself retains Android refresh/scroll/delay ownership. Production
uses SuggestRules.modestThreshold and ScreenWalkRules directly. ScreenWalk builds the
same visible-part input and Rect, without an intermediate Box. SearchText likewise lives
in shared JVM/Android test sources; GifSearch calls the same GifRules methods directly,
including the former default delete count of one. No rule decision or independent
reference was removed, and the old match tests still exercise their original facades.
PageText and SpeakerKey remain because they own Android segmentation/reveal callbacks
and clock reads respectively. Lifecycle/IO/model adapters stay as classified above.

The complete JVM test task passes (`w-tests.log`). The combined renderer runner can
also invoke the unchanged A1 replay, avoiding different target APKs for live and native
checks. That replay matches both accepted decoder/pipeline digests (`w-a1/`), and the
final boundary replay matches the original fingerprint, 6,000 reader comparisons and
all 65,536 UTF-16 units (`w-boundaries/`). Muted neural playback advances and stops on
both renderers after the direct screen-rule calls (`w-reader/`). There is no new policy
mutation for these forwarding-only edits: the independent references and original
match assertions are retained. An Android test import exposed the need to share the
frozen SearchText facade; its initial failed compilation is retained in
`w-fixture-build.log`, and the corrected fixture build passes in `w-fixture-shared.log`.
The no-key GIF search replay also matches on both renderers: query digest and typing
state match, key_available is false, and no text reaches the editor (`w-gif/`). Live
GIPHY results remain unverified because no key is available.

## Final boundary accounting

| Inventory rows | Disposition | Reason / resulting exchange |
| --- | --- | --- |
| C1, W12 | remove callback chain; retain ownership | One native learner transaction uses the native dictionary directly; no Rust → Kotlin → Rust lookup |
| C2, C1a/C2a/C2b | remove normal Android callback exports; retain reference seams | Five legacy exports are fixture/host-only. The decode callback was already unused by production after A1; it was not an active decoder chain |
| G1, W14 | merge | Native context queries and stable rank share one exchange, up to 3N+2 → 1 JNI calls |
| R1 | merge | join/find/resume carry one UTF-16 string plus lengths instead of per-string JNI arrays |
| W1–W5; SearchText in W16 | remove production facade | Call the same rules directly; preserve original facade and independent comparisons in test sources |
| C3a–C3c | keep | Android reads are lazy/observable; identical per-node packing regressed measured p50/p95 and was reverted |
| C4 | keep | Synchronous Android PCM consumer owns AudioTrack/cancellation/backpressure; borrowed samples expire after callback |
| H1–H3, W6–W11, W13, W15, remainder of W16, W17–W18 | keep | Android effects, IO, mutable UI/model identity, native ownership or independent test seams as itemized above |

The complete per-method table is a baseline inventory, not a list of deleted symbols.
Lifecycle/reset/undo/drop entries (including SuggesterNative.new/freeLearner) remain
required, callback-free ownership operations. Typed legacy declarations remain for
references; production consumers no longer enter their five removed Android exports.

| Metric | Before | After |
| --- | ---: | ---: |
| Production-source `external fun` declarations | 358 / 43 files | 360 / 45 files |
| Rust → Kotlin invocation sites in all source | 8 | 8 (3 reference-only) |
| Invocation sites compiled into normal Android | 8 | 5 |
| Callback method contracts in normal Android | 9 | 5 |
| Active production invocation sites | 6 | 5 |
| Tracked main Kotlin / Java / Rust nonblank lines | 52,839 / 321 / 51,310 | 52,745 / 321 / 51,498 |
| Rust share, supplied `rust_share.py` | 49.11% | 49.25% |

The extra declarations are NativeLearnerNative.call and NativeGlideNative.rank. A lower
source declaration count would require deleting reference seams or folding unrelated
lifetime operations into an opcode API without a measured benefit; neither is this
package's objective. The provided 49.09% belonged to 95c76b21; the verified starting
checkout was 4685918a. No tests were removed or weakened to change this ratio.

Four shipped-boundary deliberate mutations were caught and restored: learner probability
(1 fresh assertion), glide learned-pair effect (1), reader first-piece metadata (1),
and accidentally enabling legacy-fixture in a normal APK (1 packaged-export rejection).
The rejected C3 prototype adds two supplemental assertion failures, excluded from the
four shipped-boundary mutation count. Every restored comparison/check exits zero.

## Final validation

The exact required command exits **0** in **3m 18s**:

```sh
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug
```

Evidence: `final-gate.log`, `final-gate.exit`. JVM reports contain 486 app, 173 lib/rust
and 128 lib/snygg tests: **787 tests, zero failures/errors/skips**. Rust tests, Clippy,
formatting and the unchanged C++ Latin oracle pass. No SIGSEGV occurred, so no crash
rerun was needed. Lint XML contains only AndroidGradlePluginVersion,
NewerVersionAvailable and ChromeOsAbiSupport. There are no compiler warnings. The
pre-existing unit-test manifest merge notice about removing an absent EmojiCompat
initializer appears in both baseline and final gates; it is not a lint/compiler finding.
The earlier overlapping-build packaging failure is retained above, not substituted
for this successful gate.

The normal APK has no legacy fixture exports or A1 snapshot entry. Its packaged
libawnative bytes equal the compiled stripped artifact, SHA-256
`4ece59a340436419269b44518a12d16e3ca8702b239e0a46db4bbd6df5373fea`
(`final-apk.json`). App/probe installation uses install-dev.sh exclusively. No source
in latin-oracle, settings/stage 4 or local.properties changed; Compose remains. All
changed/new owned code files are below 500 nonblank/noncomment lines. Models, APKs,
grade reports and .kotlin sessions are not committed.

The additional harness checks pass: test_muting.py **2 tests**, and test_clipboard_gif
**5 tests** (unittest discovery from scripts/render-benchmark, retaining Python -I).
Evidence: `final-muting-tests.log`, `final-clipboard-gif-tests.log`.

Final **normal APK** checks (`final-typing/typing-results.json`) exit zero with verified
canonical-store restoration around each wrapper:

- regression.sh invokes run.sh on Compose and Rust: both retain exactly 300 UTF-16
  units / `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3`,
  then 308 / `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555`;
- suggestions.py matches autocorrect and candidate-pick digests on both renderers;
- input-glide.py matches all five existing glide/candidate/cancel checkpoints on both.

Relevant fixture checks elsewhere in this package include both A1 digests, the 6,000
reader comparisons and 65,536 transport units, native cohort timing/fingerprints, live
learning/unlearning, incognito storage, password candidate/inline counts, real muted
neural read/stop and no-key GIF search. Inline delivery itself and live GIPHY responses
are not claimed; the password inline count remains zero. No microphone was opened.
Only aw_keyboard_ssd / returned emulator-5556 was driven. The reserved emulator-5554
was never targeted. All UI input used the returned agent-device transport through the
project runners; application/probe installation used install-dev.sh.

Cleanup retains its initial failed literal-runtime audit (`final-cleanup.log` and
`final-cleanup/runtime-errors.txt`). All **20** matching lines are DEBUG/INFO startup
and exit-0 messages from exactly four shell `am` processes. Every message was matched
to the complete five-message startup/exit sequence; there is no app exception/fatal
in that final runtime window (`runtime-audit.json`). Earlier failed test-instrumentation
fatals are retained above and are not erased from this report's history.

The final byte audit also caught a late write to the transient
smartbar__shared_actions_expand_with_animation flag during NLP initialization. Stopping
the selected IME makes Android choose its fallback, so final cleanup selects the project
IME first, then restores the original durable preference bytes after that last bind.
Three consecutive hash/IME checks pass before shutdown. No application assertion was
changed. These cleanup preflights are retained; they are not accepted zero-error logs.

Pre-shutdown cleanup (`final-cleanup/cleanup.json`) verifies the original preference
SHA `0e622df78757593b44492677be6295051f14dcd44314fe42b8e9231e1b2d17a4`,
prototype off, Always Day/Floris Day, original accessibility null/0, project IME,
DND off, streams 1/2/3/5 = 0 and 4/10 = 1, original canonical learned/kept hashes,
and no pending store records. Disposable probe/test APKs and private dictionary/replay
files were removed; models and the normal app remain.

`device_close(shutdown=true)` reported “Could not communicate with device support”.
The advised refresh confirms aw_keyboard_ssd booted=false; `adb devices` confirms
emulator-5556 is absent (`final-adb-devices.txt`). Only the untouched reserved runtime
remains. The Device panel inventory retains a stale open-session row, despite the AVD
being off. No further device action was attempted.
