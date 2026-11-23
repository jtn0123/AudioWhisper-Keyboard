# Stage 5: remaining Kotlin/Java ownership inventory

The live ledger is [docs/jvm-ledger.tsv](jvm-ledger.tsv), checked by the gate; this page is the 2026-11-14 snapshot.

2026-11-14. Research and documentation only; this inventory authorizes no port or deletion.
Kotlin production inputs are unchanged from `d4ac289f`; the follow-up only changes Rust
dialogs, the device harness and documentation. The [ownership decision](rust.md) is logic
and all application drawing in Rust. Percentages below describe source, never a target.

## Scope and method

Enumerated every tracked `.kt` and `.java` below a module `src/main`, excluding benchmark
source sets. There are **500 production Kotlin files / 53,105 nonblank lines and zero
production Java files**. Tests, generated outputs, vendored native code and tooling are
not production. Each of the 389 remaining files has one ruling below; 111 files / 18,984
nonblank lines already covered by the settings/Compose-keyboard port are listed separately.
The exclusion ledger prevents silent omissions and double counting. Shared hosts, settings
adapters, setup, share-target/crash UI and the four retained Snygg host files remain in scope.
In particular, keeping Compose today does not make its remaining drawing a permanent adapter.

Counts are nonblank source lines **including comments**, matching the supplied Rust-share
counter; they are not executable statements or estimates of portable statements. The 500-code-
line rule excludes comments and blanks and is a different audit. Existing large resource/schema
maps must be split before editing; this task creates no oversized code file. Each mixed-file
count includes its Android portion; do not report the whole file as removable logic.

Ownership review used source declarations, imports, effect paths and production call sites,
with the stage 4 design and package 8 deletion boundaries as cross-checks. Search all source,
manifests, R8/JNI retention and reflection before any later deletion. `Devtools` is still used
by `CrashDialogActivity`; it is not dead merely because developer tools were removed. The
only whole-file dead ruling here is the unused `removeAndReturn` extension.

Classes: **A** Android adapter that stays (API named); **M** adapter with remaining logic or
drawing to move while retaining that API; **P** portable domain/model/state/utility to port
or consolidate into its existing Rust owner; **D** dead code to delete after a reference check.
Native-method declarations, packet/result projections and Cleaner-backed handle ownership
are **ART JNI adapters** (`System.loadLibrary`, JVM native-method interface). They also run
on the host for tests; that does not imply independent Kotlin policy. A transport `when`
decoding a native effect is not itself a duplicated domain decision. Conversely, handwritten
visibility, hit testing, eviction, fallback, readiness, timing and layout decisions are M/P.
P includes portable models whose algorithms are already Rust-backed: consolidate ownership
and remove redundant object graphs, rather than re-porting the same algorithm or deleting
necessary boundary DTOs for the count. Android ICU/locale/font facts remain platform facts.

## Class totals

| Class | Files | Nonblank Kotlin | Production Java |
| --- | ---: | ---: | ---: |
| A: Android adapters | 143 | 6,913 | 0 |
| M: Mixed adapters | 120 | 17,509 | 0 |
| P: Portable | 125 | 9,673 | 0 |
| D: Dead | 1 | 26 | 0 |
| Inventoried remainder | 389 | 34,121 | 0 |
| Existing settings/keyboard scope | 111 | 18,984 | 0 |
| All production | 500 | 53,105 | 0 |

The supplied source counter additionally counts `lib/rust/tools/JavaChars.java` (245 lines)
and `tools/voice-bench/Bench.java` (76): **321 tooling Java lines**, not production adapters.
The benchmark probe Java files are excluded by its `benchmark/` rule. This distinction is
retained in share calculations instead of silently changing the denominator.

## Current share and projection

Current follow-up: **53,675 Rust / 53,105 Kotlin / 321 Java = 50.1162% Rust** using the
supplied rules. Excluding the test-only `latin-oracle` main-source 2,676 lines, as the written
policy requests, gives **50,999 Rust = 48.8379%**. Both are reported; neither is a quota.
The stage 4 settings/keyboard-only estimate of 63–67% remains historical sizing.

An illustrative endpoint after all portable ownership moves and accepted fallback deletion
is roughly **74–89% on the supplied denominator**. This is a planning range, not a measured
future result. An exact projection is impossible before splitting the mixed files and deciding
which native-backed DTOs still serve Android. Explicit assumptions make the range reviewable:

- Keep all 6,913 A lines in this conservative budget, including JNI projections.
- Retain 25–50% of the 17,509 M lines as platform adapters; port/consolidate the remainder.
- Retain 0–20% of the 9,673 P lines as unavoidable boundary DTOs; move their ownership.
- Delete the 26 D lines after validation. Of the 18,984 excluded UI lines, allow 0–3,000
  to survive as extracted platform adapters; none of its application drawing may remain Kotlin.
- Stage 4 adds 7–12k Rust as previously sized. Other retired portable code adds 0–1 Rust
  line per retired line: zero allows existing Rust-backed facades to collapse without
  duplicating behavior. No tests are removed and no code is added just for the ratio.

This leaves approximately 11.3–20.6k Kotlin, 60.7–88.5k Rust, and the separately counted
321 Java tooling lines. Applying the pessimistic/optimistic combinations gives about
74.4–88.4%. If fallbacks remain for phone acceptance, their retained lines still count;
no deletion or endpoint is claimed here. Recount actual accepted source after each package.

## Ordered stage 5 packages

1. **Boundary contracts and portable model ownership.** Freeze every M/P decision against
   the retained behavior. Consolidate key/layout/popup/subtype/editor/NLP/theme models and
   caches into existing native owners; keep typed Android packets. Include `RawSuggestion`,
   `DictationFields`, `LargeModels`, `FlorisRect`, pointer registries and native-facade helpers.
2. **IME root, appearance and accessibility.** Replace remaining Compose root placement,
   background/Snygg drawing, label/icon/visibility assembly and tooltip/stop-control drawing.
   Move accessibility focus recovery, hover/hit/hold policy and semantic traversal to Rust;
   retain SurfaceView, fonts/resources, Choreographer, AccessibilityNodeProvider and
   InlineContentView transport. Requires real TalkBack and phone/120 Hz acceptance.
3. **Editor/input/NLP integration.** Move remaining admission/publication/repeat/feedback
   branches and provider/cache lifetime choices; retain InputConnection, EditorInfo, Android
   ICU boundary facts, haptics and real spell-checker/inline APIs. Differential traces must
   preserve composition, surrogate handling, cancellation, generation and effect ordering.
4. **Clipboard, dictionary and media.** Native state owns clipboard/share/import/error UI,
   GIF request/loading/fallback/cache state, frame parsing and data-provider choices. Android
   retains Room schemas/migrations, ContentResolver, URI grants, ImageDecoder and network IO.
   Use local GIF fixtures while no GIPHY key is available; do not claim live API acceptance.
5. **Dictation and read-aloud edges.** Port RMS/PCM math, verification/eviction/readiness
   choices and remaining playback/reader orchestration. Keep AudioRecord/AudioTrack/TTS,
   audio focus, platform scheduler and AccessibilityService node/action transport. No audio
   or microphone use is authorized by this inventory itself.
6. **Extension/file/setup/crash and utilities.** Port archive/path/workspace rules, resource-
   independent schema/value models, locale/tag choice, setup navigation and crash/report
   presentation. Keep SAF/intents/permissions, file/network IO and Android locale facts.
   Preserve the crash-report path when native loading itself fails; resolve the bootstrap
   contract before replacing the existing emergency screen.
7. **Acceptance and removal.** Delete proven dead code, then accepted Compose fallbacks and
   unused facades/resources after their own phone gates. Verify no Kotlin drawing or domain
   decisions remain, exact stored data and effects, full gate, privacy, lifecycle, memory and
   performance. Keep adapter lines honestly; do not optimize for the projected percentage.

The full settings rewrite remains stage 4, separately authorized; this package order is
research only. No model, UI, feature or test was removed for the inventory.

## File ledger

Path prefixes (each path below expands against the repository root):

- `AW/` = `app/src/main/kotlin/com/jtn0123/audiowhisper/`
- `FB/` = `app/src/main/kotlin/dev/patrickgold/florisboard/`
- `ANDROID/` = `lib/android/src/main/kotlin/org/florisboard/lib/android/`
- `COLOR/` = `lib/color/src/main/kotlin/org/florisboard/lib/color/`
- `COMPOSE/` = `lib/compose/src/main/kotlin/org/florisboard/lib/compose/`
- `KOTLIN/` = `lib/kotlin/src/main/kotlin/org/florisboard/lib/kotlin/`
- `JNI/` = `lib/rust/src/main/kotlin/com/jtn0123/audiowhisper/rust/`
- `SNYGG/` = `lib/snygg/src/main/kotlin/org/florisboard/lib/snygg/`

| File | Nonblank | Class | API retained / ownership action |
| --- | ---: | :---: | --- |
| `AW/dictation/Dictation.kt` | 167 | M | permission/model/platform-effect adapter; recheck field gates and model-selection facts |
| `AW/dictation/DictationFields.kt` | 17 | P | field admission and secret-field classification |
| `AW/dictation/DictationSession.kt` | 220 | M | Handler/worker adapter; released-event and callback forwarding guards around native plan |
| `AW/dictation/DictationUndo.kt` | 19 | A | ART JNI; native undo-result DTO |
| `AW/dictation/LargeModels.kt` | 23 | P | cross-model idle eviction registration and choice |
| `AW/dictation/MicAudio.kt` | 100 | M | AudioRecord; PCM16 conversion/replay admission and audio buffer policy |
| `AW/dictation/MicRecorder.kt` | 68 | M | AudioRecord via MicAudio; RMS/level math and pump error decisions |
| `AW/dictation/ModelCheck.kt` | 71 | M | Context files / SharedPreferences; file verification cache and readiness decisions |
| `AW/dictation/ModelFiles.kt` | 33 | A | Context.filesDir / java.io.File; model-path and native-config transport |
| `AW/dictation/ModelHolder.kt` | 25 | A | ART JNI; executes native model lifetime plans |
| `AW/dictation/PhraseListener.kt` | 48 | M | AudioRecord / ByteBuffer; VAD result delivery/flush boundaries |
| `AW/dictation/Transcriber.kt` | 20 | A | ART JNI / ByteBuffer; sherpa recognizer handle and cancellation |
| `AW/gif/GifDataSource.kt` | 7 | A | HttpURLConnection / ImageDecoder adapter interface; asynchronous network/preview effects only |
| `AW/gif/GifFrames.kt` | 59 | P | GIF container block/frame parsing |
| `AW/gif/GifImages.kt` | 58 | M | ImageDecoder / HttpURLConnection / LruCache; preview budget/cache ownership |
| `AW/gif/GifRecents.kt` | 58 | M | Context files / File; startup merge and persistence sequencing |
| `AW/gif/GifResultCache.kt` | 34 | M | native key cache plus Kotlin payload-map lifetime |
| `AW/gif/GifSearch.kt` | 194 | M | SystemClock / network IO; request jobs, generations, loading/publication state |
| `AW/gif/GifSearchKeys.kt` | 42 | P | key/search routing facade; consolidate native protocol, retain only boundary tags |
| `AW/gif/GifSender.kt` | 73 | M | InputConnection.commitContent / FileProvider; fallback-link and pruning choices |
| `AW/gif/Giphy.kt` | 75 | M | HttpURLConnection; response/error dispatch and domain result mapping |
| `AW/gif/LimitedRead.kt` | 22 | P | bounded stream-copy byte/overflow rules (leave Java stream IO adapter) |
| `AW/reader/FollowAlong.kt` | 62 | M | AccessibilityNodeInfo.performAction; reveal/scroll/selection sequencing |
| `AW/reader/NeuralVoice.kt` | 127 | M | AudioTrack; queue/idle/error handling around native synthesis |
| `AW/reader/PageText.kt` | 96 | M | Java BreakIterator sentence facts; consolidate portable block/reveal DTO orchestration |
| `AW/reader/PlayQueue.kt` | 21 | A | ART JNI; native playback queue transport |
| `AW/reader/ReadAloud.kt` | 109 | M | AccessibilityService / Intent / Toast; reading status and attachment orchestration |
| `AW/reader/ReadAloudService.kt` | 244 | M | AccessibilityService; page/session/settle/resume choices and callbacks |
| `AW/reader/ReadFromHereActivity.kt` | 17 | A | Activity / Intent ACTION_PROCESS_TEXT; forwards platform selection |
| `AW/reader/ScreenWalk.kt` | 82 | A | AccessibilityNodeInfo; node facts and native traversal callbacks |
| `AW/reader/Speaker.kt` | 79 | M | TextToSpeech; pending utterance and completion/error lifecycle |
| `AW/reader/SpeakerKey.kt` | 21 | A | SystemClock / ART JNI; native hold-state transport |
| `AW/reader/StopButton.kt` | 67 | M | WindowManager TYPE_ACCESSIBILITY_OVERLAY; Rust must draw/layout the stop control |
| `AW/reader/Voice.kt` | 45 | A | AudioManager.requestAudioFocus / abandonAudioFocusRequest |
| `AW/reader/VoiceEngine.kt` | 134 | M | Context files / SystemClock; model idle/failure/cross-model eviction state |
| `AW/render/AccessibilityRelease.kt` | 27 | P | hover/hold release target choice and hit testing |
| `AW/render/KeyGraphRows.kt` | 110 | M | LayoutManager files / java.util.Locale casing; the key graph's build row from the files a mode needs, its facts row, and Android's casing answers (added at phase 3 item 2) |
| `AW/render/RenderTiming.kt` | 14 | M | ViewTreeObserver / Log; fallback-only timing scaffolding, remove with its callers |
| `AW/render/RendererAccessibility.kt` | 285 | M | AccessibilityNodeProvider; focus recovery, hover/hit/hold/touch state still Kotlin |
| `AW/render/RendererAccessibilityResources.kt` | 99 | A | Context.getString; localized semantic-label map |
| `AW/render/RendererActionTooltip.kt` | 42 | M | PopupWindow / View; tooltip selection and lifetime around native text |
| `AW/render/RendererCandidateBindings.kt` | 23 | A | ART JNI; object identity handles for native candidates |
| `AW/render/RendererClipboard.kt` | 59 | M | Room / KeyguardManager; item projections and eligible preview/resource choices |
| `AW/render/RendererClipboardActions.kt` | 34 | M | ClipboardManager / Room; conditional unpin/delete and feedback dispatch |
| `AW/render/RendererDecorations.kt` | 18 | M | Resources; watermark visibility and icon selection |
| `AW/render/RendererEmoji.kt` | 70 | M | Paint / Typeface / KeyguardManager; label/availability projection and bindings |
| `AW/render/RendererGestureHost.kt` | 77 | M | MotionEvent / Resources; action-index mapping and gesture-plan execution |
| `AW/render/RendererGif.kt` | 41 | M | network/resource adapter; loaded/error/empty state selection |
| `AW/render/RendererGifActions.kt` | 42 | M | InputConnection / Toast; send-outcome transitions and panel return state |
| `AW/render/RendererGifDecoder.kt` | 42 | M | Bitmap / ImageDecoder; frame/animation eligibility and upload lifecycle |
| `AW/render/RendererHostTiming.kt` | 23 | A | ViewTreeObserver / SystemClock / Log; numeric creation/layout measurement only |
| `AW/render/RendererIcons.kt` | 30 | A | Resources / Android Path; resource vector serialization, no widget layout |
| `AW/render/RendererMediaBuffers.kt` | 62 | M | ContentResolver / Bitmap; decode-size and upload/cache ownership |
| `AW/render/RendererMotionPacket.kt` | 22 | A | MotionEvent; complete pointer/history/timestamp transport |
| `AW/render/RendererPanels.kt` | 71 | M | Resources / Jetpref; panel choice, action assembly and label fallbacks (phase 3 item 2: each action's enabled state, icon token and label come from the Rust key graph's `actions` op through `ActionKeys`; the panels no longer read the smartbar's evaluator) |
| `AW/render/RendererSearch.kt` | 21 | M | Resources; search-strip state/label projection |
| `AW/render/RendererSmartbar.kt` | 110 | M | Resources / Paint; item choice, measurement and action binding (each action's icon, arrow, flags, label and what the toggle flips moved to Rust's `smartbar_actions` at phase 3 item 1; this file keeps the evaluator's facts and the icon vectors; phase 3 item 2: each action's enabled state, icon and label are answered from the Rust key graph, so this file no longer writes them) |
| `AW/render/RendererSmartbarTouch.kt` | 47 | M | InputConnection / PopupWindow; enabled/removal/tooltip action gating |
| `AW/render/RendererSubtypes.kt` | 21 | A | Resources / Jetpref Flow; localized subtype snapshot transport |
| `AW/render/RendererTheme.kt` | 27 | A | Resources.Configuration / DisplayMetrics; theme and density transport |
| `AW/render/RustChromeView.kt` | 98 | M | View / Bitmap / Canvas; native pixels stay at edge, move hardcoded subtype bootstrap dimensions/timing constants |
| `AW/render/RustExtractView.kt` | 37 | A | ExtractEditText / ViewGroup; applies native extract bounds to platform editor |
| `AW/render/RustKeyGraphs.kt` | 96 | M | Context / Resources / LayoutManager; the Rust renderer's key graphs per mode and subtype, their label resources and the question-and-answer loop (added at phase 3 item 2) |
| `AW/render/RustKeyPreview.kt` | 34 | A | Bitmap / ART JNI; displays native preview pixels (replace Compose interop host) |
| `AW/render/RustKeyboard.kt` | 130 | M | Keyboard / TextKey adapter; reads the Rust key graph's keys back for the input path and a key's platform effects (added at phase 3 item 2) |
| `AW/render/RustKeyboardLayout.kt` | 95 | M | SurfaceView / WindowInsets / InlineContentView; Kotlin root sizing/state branches |
| `AW/render/RustKeyboardView.kt` | 405 | M | SurfaceView / Choreographer / SystemFonts; snapshot assembly and dispatch/fallback policy |
| `AW/render/RustOneHandedPanel.kt` | 37 | M | View/Jetpref; side-panel sizing/visibility and preference-effect binding |
| `AW/render/RustSubtypeSheet.kt` | 31 | A | WindowInsets / AndroidView; native sheet surface interop only |
| `AW/settings/SettingsCodecs.kt` | 67 | A | Jetpref typed serializers / ART JNI; Android DTO construction only |
| `AW/settings/SettingsComponentSerializer.kt` | 12 | A | Jetpref typed serializer / ART JNI; component codec transport |
| `AW/settings/SettingsDefaults.kt` | 37 | A | Jetpref PreferenceModel / ART JNI; reads native defaults |
| `AW/settings/SettingsKeys.kt` | 58 | A | Jetpref key serializer / ART JNI; platform key DTO tags only |
| `AW/settings/SettingsScalars.kt` | 13 | A | Jetpref typed serializer / ART JNI; scalar codec transport |
| `AW/settings/SettingsSelection.kt` | 6 | A | Jetpref serializer / ART JNI; native enum index |
| `AW/settings/SettingsSubtypes.kt` | 15 | A | Jetpref store / ART JNI; native subtype operation-plan DTO |
| `AW/settings/SettingsTimeSerializer.kt` | 12 | A | Jetpref typed serializer / java.time.LocalTime; native codec transport |
| `AW/settingsui/RustSettingsView.kt` | 174 | A | SurfaceView / Choreographer / BaseInputConnection; native effects, state and Android IME visibility transport only |
| `AW/settingsui/SettingsIcons.kt` | 19 | A | Resources / Android Path; resource icon map |
| `AW/settingsui/SettingsInputView.kt` | 53 | A | View / BaseInputConnection / InputMethodManager; native text endpoint |
| `AW/settingsui/SettingsPlatform.kt` | 56 | M | Jetpref / Resources / WindowCompat; palette/appearance selection and write-queue binding |
| `AW/settingsui/SettingsResources.kt` | 89 | A | Context.getString; batched string-resource map |
| `AW/suggest/AutoCorrectionUndo.kt` | 24 | A | ART JNI / InputConnection consumer; native undo-result projection (moved to test sources at phase 3 item 1: the NLP session arms and takes the undo; kept for the frozen reference fan-out) |
| `AW/suggest/GlideRanker.kt` | 53 | A | ART JNI; copies scores/context into native ranking |
| `AW/suggest/KeptWords.kt` | 47 | A | Context files / java.io.File; native word-store persistence |
| `AW/suggest/LatinDictionaryEngine.kt` | 35 | A | RandomAccessFile / ART JNI; dictionary bytes and native decoder handle |
| `AW/suggest/LatinSuggester.kt` | 63 | A | ART JNI; native suggest/spell result projection |
| `AW/suggest/LearnedWords.kt` | 78 | A | Context files / java.io.File; native store effects and serialized bytes |
| `AW/suggest/PersonalWords.kt` | 33 | A | Room snapshot / ART JNI; native personal-store transport and returned order |
| `AW/suggest/PreviousWords.kt` | 27 | A | ART JNI; native previous-word context DTO |
| `AW/suggest/RawSuggestion.kt` | 52 | P | decoder flag classification and portable result models |
| `AW/suggest/SwipeWords.kt` | 58 | P | portable SwipeWords domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `AW/suggest/TapTrail.kt` | 56 | P | portable TapTrail domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (phase 3 item 1: its typing events moved into the NLP session's plan; presses, orientation, aim and retarget remain) |
| `AW/suggest/TypingHistory.kt` | 26 | A | Room/file-store / ART JNI; combines Android store handles |
| `AW/suggest/WordLearner.kt` | 37 | A | ART JNI; native learning operation transport (moved to test sources at phase 3 item 1: the NLP session learns each typing event itself; kept for the frozen reference fan-out) |
| `FB/FlorisApplication.kt` | 142 | M | Application / BroadcastReceiver; manager startup and initialization sequencing |
| `FB/FlorisImeService.kt` | 448 | M | InputMethodService; input/extract/keyboard-switch/lifecycle branches |
| `FB/FlorisImeUi.kt` | 161 | M | WindowInsets / Compose host; root sizing/placement/background drawing |
| `FB/FlorisImeViews.kt` | 229 | M | ExtractEditText / ComposeView; input/sheet/extract branch/layout ownership |
| `FB/FlorisInlineSuggestions.kt` | 58 | A | InputMethodService.onCreateInlineSuggestionsRequest / InlineSuggestionsResponse |
| `FB/FlorisSpellCheckerService.kt` | 105 | M | SpellCheckerService; language/session/result assembly around native spell |
| `FB/NativeExtractContent.kt` | 25 | A | ExtractEditText / AndroidView; native chrome with platform text editor |
| `FB/app/AppPreferenceMigration.kt` | 27 | A | Jetpref migration callback / ART JNI; applies native migration result |
| `FB/app/AppPrefs.kt` | 301 | M | Jetpref PreferenceModel; Android stores stay, derived UI/sizing values must move |
| `FB/app/ComposeSettingsFallbackActivity.kt` | 5 | A | ComponentActivity; retained fallback Activity identity (deleted at compose-removal step 12) |
| `FB/app/EnumDisplayEntries.kt` | 737 | A | Resources.getString; enum resource IDs/descriptions (split existing oversized map) (deleted at compose-removal step 12) |
| `FB/app/FlorisAppActivity.kt` | 267 | M | ComponentActivity / ActivityResultRegistry; retained routing/import/default-selection branches (a thin Rust settings host since compose-removal step 12: splash, insets, Back, saved state and intents, 104 counted lines; the START or NAVIGATE choice is `SettingsEntry.request`) |
| `FB/app/Routes.kt` | 295 | P | portable Routes domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (deleted at compose-removal step 12) |
| `FB/app/apptheme/Color.kt` | 95 | P | portable Color domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/app/apptheme/Shape.kt` | 24 | P | portable Shape domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (deleted at compose-removal step 12) |
| `FB/app/apptheme/Theme.kt` | 81 | M | Resources / WindowInsetsController; preference-to-theme choice still in Kotlin (after compose-removal step 12 only `FlorisCopyToClipboardActivity`, the image share target, draws with it; it goes with that activity) |
| `FB/app/apptheme/Type.kt` | 41 | P | portable Type domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (`Theme.kt`'s typography, and the tokens `SettingsTypographyTokensTest` checks Rust against) |
| `FB/app/settings/advanced/BackupIO.kt` | 81 | M | ContentResolver / Room / File; backup traversal, selected records and restore effects |
| `FB/app/setup/NotificationPermissionState.kt` | 26 | P | setup permission-progress state |
| `FB/app/setup/SetupScreen.kt` | 240 | M | ActivityResultContracts / InputMethodManager; setup steps/navigation and Compose drawing |
| `FB/ime/ImeUiMode.kt` | 25 | P | portable ImeUiMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/clipboard/ClipboardGridConstants.kt` | 3 | P | portable ClipboardGridConstants domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/clipboard/ClipboardHistory.kt` | 27 | P | portable ClipboardHistory domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/clipboard/ClipboardManager.kt` | 324 | M | android.content.ClipboardManager / Room; lifecycle, collection and effect sequencing |
| `FB/ime/clipboard/ClipboardPolicy.kt` | 15 | A | ART JNI; native model/geometry/plan projection only, no independent domain choice |
| `FB/ime/clipboard/ClipboardSyncBehavior.kt` | 22 | P | portable ClipboardSyncBehavior domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/clipboard/FlorisCopyToClipboardActivity.kt` | 176 | M | Intent ACTION_SEND / ClipData; share-target drawing and error UI |
| `FB/ime/clipboard/provider/ClipboardDatabase.kt` | 338 | M | Room / ClipData / ContentResolver; entities remain, equality/display/serialization policy moves |
| `FB/ime/clipboard/provider/ClipboardFileStorage.kt` | 77 | A | ContentResolver / java.io.File; URI-byte storage and file deletion |
| `FB/ime/clipboard/provider/ClipboardMediaProvider.kt` | 178 | M | ContentProvider / ParcelFileDescriptor; MIME/query/update dispatch and metadata rules |
| `FB/ime/core/DisplayLanguageNamesIn.kt` | 25 | P | portable DisplayLanguageNamesIn domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/core/Subtype.kt` | 177 | P | portable Subtype domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/core/SubtypeFallback.kt` | 6 | A | ART JNI; native model/geometry/plan projection only, no independent domain choice |
| `FB/ime/core/SubtypeManager.kt` | 102 | M | Context / Jetpref; subtype lookup/resources and persistence orchestration |
| `FB/ime/core/SubtypeSelectionState.kt` | 4 | P | portable SubtypeSelectionState domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/dictionary/DictionaryManager.kt` | 156 | M | Room / UserDictionary provider; dictionary readiness/query composition |
| `FB/ime/dictionary/UserDictionary.kt` | 427 | M | Room / ContentResolver / Cursor; import/export/filter and locale rules |
| `FB/ime/dictionary/UserDictionaryValidation.kt` | 57 | P | portable UserDictionaryValidation domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/editor/AbstractEditorInstance.kt` | 151 | M | InputConnection; composing setup, reconciliation and content publication guards |
| `FB/ime/editor/EditorContent.kt` | 98 | P | portable EditorContent domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/editor/EditorInstance.kt` | 325 | M | InputConnection / ClipboardManager; rich-content and editor operation orchestration |
| `FB/ime/editor/EditorPlanExecutor.kt` | 90 | A | InputConnection / KeyEvent; ordered native editor effects |
| `FB/ime/editor/EditorRange.kt` | 51 | P | portable EditorRange domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/editor/FlorisEditorInfo.kt` | 86 | M | EditorInfo / EmojiCompat; field comparisons, compatibility and initial-text projection |
| `FB/ime/editor/ImeOptions.kt` | 53 | M | EditorInfo / InputType; native bit facts stay at edge, move Kotlin enum lookup/default fallback decisions |
| `FB/ime/editor/InputAttributes.kt` | 87 | M | EditorInfo / InputType; native bit facts stay at edge, move Kotlin enum lookup/default fallback decisions |
| `FB/ime/input/CapitalizationBehavior.kt` | 20 | P | portable CapitalizationBehavior domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/input/DeleteTiming.kt` | 18 | A | SystemClock / Log; numeric repeat-delete measurement instrumentation |
| `FB/ime/input/HapticVibrationMode.kt` | 20 | P | portable HapticVibrationMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/input/InputEventDispatcher.kt` | 118 | M | SystemClock / ViewConfiguration; event/repeat scheduling and admission wrapper |
| `FB/ime/input/InputFeedbackActivationMode.kt` | 20 | P | portable InputFeedbackActivationMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/input/InputFeedbackController.kt` | 90 | M | AudioManager / Vibrator / View.performHapticFeedback; fallback feedback execution policy |
| `FB/ime/input/InputShiftState.kt` | 44 | P | portable InputShiftState domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/ComputingEvaluator.kt` | 223 | M | Resources / ImageVector; key label/icon/default choices (phase 3 item 2: the label resources and icon vectors split out so the key graph's adapter and the Rust renderer look them up by token) |
| `FB/ime/keyboard/CurrencySet.kt` | 45 | P | portable CurrencySet domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/FlorisImeSizing.kt` | 173 | M | WindowManager / WindowInsets; host layout and height selection (phase 3 item 2: sizes from any keyboard's row count, a Rust one's too) |
| `FB/ime/keyboard/IncognitoMode.kt` | 21 | P | portable IncognitoMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/Key.kt` | 89 | P | portable Key domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/KeyData.kt` | 269 | P | portable KeyData domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (phase 3 item 2: the selectors and every `compute` serve only the Compose fallback; `KeyData` stays the input path's key data) |
| `FB/ime/keyboard/Keyboard.kt` | 112 | P | portable Keyboard domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (phase 3 item 2: `rowCount` on the base class; `PlaceholderLoadingKeyboard` is still a `TextKeyboard`) |
| `FB/ime/keyboard/KeyboardComputingEvaluator.kt` | 89 | M | Context / Resources; enabled/visible/fallback fact assembly |
| `FB/ime/keyboard/KeyboardEditing.kt` | 197 | M | KeyEvent / InputConnection; commit/delete/selection/autocorrect orchestration |
| `FB/ime/keyboard/KeyboardExtension.kt` | 85 | P | portable KeyboardExtension domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/KeyboardKeyEvents.kt` | 116 | M | InputConnection / IME APIs; remaining CLEAR_CLIP preference branch and route fact selection |
| `FB/ime/keyboard/KeyboardManager.kt` | 276 | M | Context / Toast; input/editor/NLP/state coordination (phase 3 item 2: with the Rust renderer on, its keyboards come from `RustKeyGraphs`; the Compose branch keeps `LayoutManager` and the keyboard cache) |
| `FB/ime/keyboard/KeyboardManagerResources.kt` | 98 | M | Context.assets / LiveData; extension lookup and resource publication |
| `FB/ime/keyboard/KeyboardMode.kt` | 38 | P | portable KeyboardMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/KeyboardState.kt` | 207 | M | Android host Flow/Compose binding; mutation batches and model ownership |
| `FB/ime/keyboard/LayoutArrangement.kt` | 31 | P | portable LayoutArrangement domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/LayoutAssembly.kt` | 23 | A | ART JNI; native model/geometry/plan projection only, no independent domain choice (phase 3 item 2: `merge` and `hints` serve only the Compose fallback; `plan` and `modifier` name the files the Rust graph is built from) |
| `FB/ime/keyboard/LayoutManager.kt` | 216 | M | Context.assets; cache/load/concurrency and layout object ownership (serves only the Compose fallback since phase 3 item 2: the Rust renderer's keys come from Rust's key graph (`keygraph`); deleted at compose-removal step K) |
| `FB/ime/keyboard/LayoutType.kt` | 60 | P | portable LayoutType domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/keyboard/ShiftLogic.kt` | 85 | A | ART JNI; native model/geometry/plan projection only, no independent domain choice |
| `FB/ime/keyboard/SpaceBarMode.kt` | 21 | P | portable SpaceBarMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/landscapeinput/LandscapeInputUiMode.kt` | 21 | P | portable LandscapeInputUiMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/lifecycle/LifecycleInputMethodService.kt` | 75 | A | InputMethodService / LifecycleRegistry / SavedStateRegistry |
| `FB/ime/media/emoji/Emoji.kt` | 92 | P | portable Emoji domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/media/emoji/EmojiCategory.kt` | 55 | P | portable EmojiCategory domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/media/emoji/EmojiData.kt` | 122 | M | Context.assets; asset selection and native table projection |
| `FB/ime/media/emoji/EmojiHistory.kt` | 78 | P | portable EmojiHistory domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/media/emoji/EmojiSet.kt` | 44 | P | portable EmojiSet domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/media/emoji/EmojiSuggestionProvider.kt` | 117 | M | Context.assets; provider lifecycle and suggestion publication |
| `FB/ime/media/emoji/EmojiSuggestionType.kt` | 20 | P | portable EmojiSuggestionType domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/media/emoji/FlorisEmojiCompat.kt` | 69 | M | EmojiCompat / font APIs; loading and readiness fallbacks |
| `FB/ime/media/emoticon/EmoticonKeyData.kt` | 28 | P | portable EmoticonKeyData domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/media/emoticon/EmoticonLayoutData.kt` | 33 | P | portable EmoticonLayoutData domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/nlp/BreakIteratorGroup.kt` | 110 | M | android.icu.text.BreakIterator; keep boundaries, move count/clamp/cache policy |
| `FB/ime/nlp/ClipboardSuggestionProvider.kt` | 86 | M | ClipboardManager; candidate gating and removal sequencing |
| `FB/ime/nlp/LanguagePackExtension.kt` | 49 | P | portable LanguagePackExtension domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/nlp/NlpInlineAutofill.kt` | 41 | A | InlineSuggestion.inflate / InlineContentView; actual Android autofill views |
| `FB/ime/nlp/NlpManager.kt` | 363 | M | SystemClock / provider IO; remaining worker/transport and candidate publication orchestration |
| `FB/ime/nlp/NlpProviders.kt` | 260 | P | portable NlpProviders domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/nlp/NlpState.kt` | 117 | M | Context / InputConnection facts; native protocol projection and publication |
| `FB/ime/nlp/TypingEvents.kt` | 95 | A | EditorContent / save timers; one NlpSession.event row per typing event, then the plan's Android effects (added at phase 3 item 1) |
| `FB/ime/nlp/PunctuationRule.kt` | 84 | P | portable PunctuationRule domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/nlp/SpellingLanguageMode.kt` | 23 | P | portable SpellingLanguageMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/nlp/SpellingResult.kt` | 106 | M | SuggestionsInfo; remaining result flag/result-list mapping |
| `FB/ime/nlp/SuggestionCandidate.kt` | 154 | M | Resources / ClipData / ImageVector; display/icon/removal decisions |
| `FB/ime/nlp/latin/LatinLanguageProvider.kt` | 384 | M | Room / File / SystemClock; dictionary/provider selection, caches and load/error state |
| `FB/ime/onehanded/OneHandedMode.kt` | 23 | P | portable OneHandedMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/popup/ExtendedPopup.kt` | 47 | A | ART JNI; native model/geometry/plan projection only, no independent domain choice |
| `FB/ime/popup/PopupMapping.kt` | 34 | P | portable PopupMapping domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/popup/PopupSet.kt` | 202 | P | portable PopupSet domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (phase 3 item 2: `MutablePopupSet` and its merging serve only the Compose fallback; `PopupSet` stays as the popup identity a Rust key carries, `PopupKeys` as what a press shows) |
| `FB/ime/popup/PopupVisibilityState.kt` | 4 | P | portable PopupVisibilityState domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/sheet/BottomSheetState.kt` | 4 | P | portable BottomSheetState domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/smartbar/CandidatesDisplayMode.kt` | 24 | P | portable CandidatesDisplayMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/smartbar/ExtendedActionsPlacement.kt` | 24 | P | portable ExtendedActionsPlacement domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/smartbar/IncognitoDisplayMode.kt` | 20 | P | portable IncognitoDisplayMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/smartbar/InlineSuggestionsUi.kt` | 91 | M | InlineContentView; replace Compose layout with Rust bounds and Android interop |
| `FB/ime/smartbar/SmartbarLayout.kt` | 22 | P | portable SmartbarLayout domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/smartbar/quickaction/QuickAction.kt` | 134 | M | InputConnection / Resources; pointer dispatch and dynamic display labels |
| `FB/ime/smartbar/quickaction/QuickActionArrangement.kt` | 48 | P | portable QuickActionArrangement domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/composing/Composer.kt` | 69 | P | portable Composer domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/gestures/GlideTypingClassifier.kt` | 50 | P | portable GlideTypingClassifier domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/gestures/GlideTypingGesture.kt` | 143 | M | MotionEvent / Resources; detector lifecycle and pointer-to-domain projection |
| `FB/ime/text/gestures/GlideTypingManager.kt` | 110 | M | SystemClock / resource IO; async generation and publication orchestration |
| `FB/ime/text/gestures/StatisticalGlideTypingClassifier.kt` | 255 | M | Context / Android collections; index/cache/build/query ownership |
| `FB/ime/text/gestures/SwipeAction.kt` | 97 | P | portable SwipeAction domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/gestures/SwipeGesture.kt` | 183 | M | MotionEvent / VelocityTracker; event qualification and detector transport |
| `FB/ime/text/gestures/SwipeIndex.kt` | 90 | M | Normalizer / Android collections; index construction and word/key projection |
| `FB/ime/text/key/KeyCode.kt` | 119 | P | portable KeyCode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/key/KeyHintConfiguration.kt` | 28 | P | portable KeyHintConfiguration domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/key/KeyHintMode.kt` | 25 | P | portable KeyHintMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/key/KeyType.kt` | 58 | P | portable KeyType domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/key/KeyVariation.kt` | 35 | P | portable KeyVariation domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/key/UtilityKeyAction.kt` | 26 | P | portable UtilityKeyAction domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/text/keyboard/TextKey.kt` | 170 | P | portable TextKey domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (serves only the Compose fallback since phase 3 item 2: the Rust renderer's keys come from Rust's key graph (`keygraph`); deleted at compose-removal step K) |
| `FB/ime/text/keyboard/TextKeyData.kt` | 213 | M | android.icu.lang.UCharacter; keep platform casing facts, move retained model/cache state (phase 3 item 2: `compute` and the auto-text casing serve only the Compose fallback; the data classes carry the Rust graph's keys to the input path) |
| `FB/ime/text/keyboard/TextKeyboard.kt` | 131 | P | portable TextKeyboard domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (serves only the Compose fallback since phase 3 item 2: the Rust renderer's keys come from Rust's key graph (`keygraph`); deleted at compose-removal step K) |
| `FB/ime/text/keyboard/TextKeyboardCache.kt` | 36 | P | portable TextKeyboardCache domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs (serves only the Compose fallback since phase 3 item 2: the Rust renderer's keys come from Rust's key graph (`keygraph`); deleted at compose-removal step K) |
| `FB/ime/theme/FlorisAssetResolver.kt` | 44 | A | Context.filesDir / assets; native-selected asset path resolution |
| `FB/ime/theme/FlorisImeTheme.kt` | 77 | M | Resources / composition locals; base-style/layout-default choice and theme hosting |
| `FB/ime/theme/FlorisImeThemeBaseStyle.kt` | 348 | P | portable FlorisImeThemeBaseStyle domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/theme/FlorisImeUi.kt` | 373 | P | portable Snygg element/attribute schema; move schema ownership to Rust |
| `FB/ime/theme/FlorisImeUiSpec.kt` | 303 | P | portable FlorisImeUiSpec domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/theme/ThemeExtension.kt` | 52 | P | portable ThemeExtension domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/theme/ThemeExtensionComponent.kt` | 89 | P | portable ThemeExtensionComponent domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/ime/theme/ThemeManager.kt` | 345 | M | Resources / InlineSuggestionUi; active-theme/day-night choice and inline styles |
| `FB/ime/theme/ThemeMode.kt` | 27 | P | portable ThemeMode domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/FlorisLocale.kt` | 290 | M | Android Locale/ICU locale names; tag/selection/cache behavior needs explicit native ownership |
| `FB/lib/FlorisRect.kt` | 200 | P | rectangle geometry/intersection/containment |
| `FB/lib/Native.kt` | 71 | A | ART JNI / ByteBuffer; native string and pointer wrapper ownership |
| `FB/lib/Pointer.kt` | 167 | P | pointer registry, reuse and reset rules |
| `FB/lib/StateAdapters.kt` | 60 | A | LiveData.observe / LifecycleOwner; observable platform bindings |
| `FB/lib/Unicode.kt` | 42 | A | android.icu.lang.UCharacter; Android character facts |
| `FB/lib/Validation.kt` | 124 | M | Resources; validation-result/model and message-choice ownership |
| `FB/lib/cache/CacheManager.kt` | 202 | M | Context.cacheDir / ContentResolver; workspace/import lifetime and selection |
| `FB/lib/compose/SystemUi.kt` | 54 | M | WindowInsetsController; Kotlin luminance/icon-mode selection |
| `FB/lib/crashutility/CrashDialogActivity.kt` | 139 | M | Activity / Intent / ClipboardManager; crash screen drawing and report composition |
| `FB/lib/crashutility/CrashUtility.kt` | 384 | M | Application.ActivityLifecycleCallbacks / NotificationManager; crash retention/routing policy |
| `FB/lib/devtools/Devtools.kt` | 141 | M | Build / Resources; still called by crash dialog; report composition is portable |
| `FB/lib/devtools/Flog.kt` | 304 | M | android.util.Log; topic gate, tag/message and redaction/formatting policy |
| `FB/lib/devtools/LogTopic.kt` | 45 | P | portable LogTopic domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/ext/Extension.kt` | 137 | M | Context / Uri; file handles and extension load/unload ownership |
| `FB/lib/ext/ExtensionCodec.kt` | 29 | P | portable ExtensionCodec domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/ext/ExtensionComponent.kt` | 77 | P | portable ExtensionComponent domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/ext/ExtensionDefaults.kt` | 27 | P | portable ExtensionDefaults domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/ext/ExtensionMaintainer.kt` | 51 | P | portable ExtensionMaintainer domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/ext/ExtensionManager.kt` | 237 | M | FileObserver / Context.assets; import/index/refresh lifetime and routing |
| `FB/lib/ext/ExtensionMeta.kt` | 97 | P | portable ExtensionMeta domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/ext/ExtensionValidation.kt` | 153 | P | portable ExtensionValidation domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/io/AssetManager.kt` | 164 | M | Context.assets / File; source selection, admission and operation composition |
| `FB/lib/io/FileRegistry.kt` | 71 | P | portable FileRegistry domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/io/FlorisRef.kt` | 322 | M | Context / Uri / File; scheme/subpath validation and model ownership |
| `FB/lib/io/ZipUtils.kt` | 159 | M | ContentResolver / ZipFile; archive traversal and path/admission policy |
| `FB/lib/util/AppVersionUtils.kt` | 66 | M | Context / Jetpref; version/changelog choice and comparison |
| `FB/lib/util/InputMethodUtils.kt` | 81 | A | InputMethodManager / Settings.Secure; IME status/picker/enabler |
| `FB/lib/util/LaunchUtils.kt` | 78 | A | Context.startActivity / Intent / Toast; executes requested effects |
| `FB/lib/util/NetworkUtils.kt` | 47 | P | portable NetworkUtils domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `FB/lib/util/TimeUtils.kt` | 29 | A | android.icu.text.SimpleDateFormat / Instant; platform clock formatting |
| `FB/lib/util/ViewUtils.kt` | 92 | A | View.layoutParams / Resources.DisplayMetrics; Android bounds application |
| `ANDROID/AndroidAssetsManager.kt` | 114 | A | android.content.res.AssetManager; stream access/closing |
| `ANDROID/AndroidClipboardManager.kt` | 26 | A | android.content.ClipboardManager; Android service accessor |
| `ANDROID/AndroidInternalR.kt` | 58 | A | Resources.getIdentifier; platform resource lookup |
| `ANDROID/AndroidKeyguardManager.kt` | 17 | A | android.app.KeyguardManager; Android service accessor |
| `ANDROID/AndroidSettings.kt` | 79 | A | Settings.System / Secure / Global; platform queries and observation |
| `ANDROID/Configuration.kt` | 23 | A | Resources.Configuration; Android orientation facts |
| `ANDROID/ContentResolver.kt` | 108 | A | android.content.ContentResolver; streams and asset-file descriptors |
| `ANDROID/Context.kt` | 85 | A | Context.getSystemService / Resources; Android lookup extensions |
| `ANDROID/FileObserver.kt` | 33 | A | android.os.FileObserver; platform file-change delivery |
| `ANDROID/SparseArray.kt` | 26 | D | removeAndReturn has no production/test call or import; extension has no reflective entry point |
| `ANDROID/SystemSettingObserver.kt` | 33 | A | ContentObserver / Handler; setting-change transport |
| `ANDROID/Toast.kt` | 124 | A | android.widget.Toast; requested Android notifications |
| `ANDROID/UriSerializer.kt` | 34 | A | android.net.Uri; platform URI string transport |
| `ANDROID/Vibrator.kt` | 44 | M | Vibrator / VibrationEffect; amplitude/duration/fallback choices |
| `COLOR/ColorMappings.kt` | 99 | M | Resources system accent; neutral/dynamic role construction and colour rules |
| `COLOR/ColorPalette.kt` | 100 | M | Android Material colour facts; role-name/default mappings |
| `COLOR/MaterialYouFlags.kt` | 38 | P | portable MaterialYouFlags domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `COMPOSE/DisposableLifecycleEffect.kt` | 43 | A | LifecycleEventObserver; lifecycle subscription (Compose wrapper later replaceable) |
| `COMPOSE/Modifier.kt` | 24 | M | View scrolling interop; Compose scroll/layout wrapper must become native bounds |
| `COMPOSE/Resources.kt` | 102 | A | Resources.getString / getQuantityString / Configuration; locale and resource facts |
| `COMPOSE/State.kt` | 75 | A | Settings content observers / LifecycleEventObserver; Android state binding |
| `KOTLIN/Class.kt` | 26 | P | portable Class domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/Collections.kt` | 26 | P | portable Collections domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/Flow.kt` | 28 | P | portable Flow domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/GuardedByLock.kt` | 40 | P | portable GuardedByLock domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/Library.kt` | 28 | P | portable Library domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/MimeTypeFilter.kt` | 65 | P | portable MimeTypeFilter domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/Numbers.kt` | 17 | P | portable Numbers domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/Regex.kt` | 31 | P | portable Regex domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/Result.kt` | 43 | P | portable Result domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/Strings.kt` | 81 | P | portable Strings domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `KOTLIN/UtilityRules.kt` | 10 | A | ART JNI; utility request/result packet transport |
| `KOTLIN/io/FsFile.kt` | 48 | P | portable FsFile domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `JNI/BackupPolicy.kt` | 26 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ClipboardRules.kt` | 13 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Composing.kt` | 43 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ComposingNative.kt` | 14 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Dictation.kt` | 72 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/DictationNative.kt` | 20 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/DictationSessionPlan.kt` | 47 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/EditorSession.kt` | 51 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Emoji.kt` | 101 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/EmojiNative.kt` | 17 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ExtensionPolicy.kt` | 35 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Gestures.kt` | 135 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/GesturesNative.kt` | 51 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Gif.kt` | 75 | M | ART JNI; residual signed-order decoding helpers and cache payload lifetime |
| `JNI/GifNative.kt` | 32 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Giphy.kt` | 39 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/GiphyNative.kt` | 14 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Glide.kt` | 116 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/GlideNative.kt` | 23 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/GlideTrailRules.kt` | 6 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ImeChromeRules.kt` | 13 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/InputDispatch.kt` | 30 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/JavaText.kt` | 42 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/JavaTextNative.kt` | 31 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/KeyModels.kt` | 27 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/KeyboardRenderer.kt` | 147 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Keys.kt` | 51 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/KeysNative.kt` | 29 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/LatinSession.kt` | 78 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/LayoutCache.kt` | 14 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/LayoutRules.kt` | 23 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/MediaRules.kt` | 7 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/NativeGlide.kt` | 17 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/NlpSession.kt` | 28 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/PanelRules.kt` | 23 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Popup.kt` | 123 | M | ART JNI; residual popup model/index/default projection |
| `JNI/PopupNative.kt` | 52 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Reader.kt` | 94 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ReaderNative.kt` | 32 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ReaderRuntime.kt` | 46 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/RenderContentRules.kt` | 6 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/RustLibrary.kt` | 16 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ScreenNative.kt` | 18 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ScreenWalk.kt` | 51 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SettingsRules.kt` | 54 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SherpaAsr.kt` | 57 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SherpaTts.kt` | 39 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Shift.kt` | 52 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/ShiftNative.kt` | 13 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SnyggCompiler.kt` | 6 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SnyggGrammar.kt` | 6 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SnyggValues.kt` | 6 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Spacing.kt` | 119 | M | ART JNI; residual editor callbacks/spacing orchestration facade |
| `JNI/SpacingNative.kt` | 21 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SubtypeRules.kt` | 6 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Suggest.kt` | 129 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SuggestNative.kt` | 40 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Suggester.kt` | 39 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SuggesterNative.kt` | 16 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Swipe.kt` | 62 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/SwipeNative.kt` | 22 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/Utf16Batch.kt` | 6 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/WordStores.kt` | 138 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `JNI/WordStoresNative.kt` | 49 | A | ART JNI (System.loadLibrary / external methods / Cleaner); native results and handle/packet transport |
| `SNYGG/Snygg.kt` | 76 | P | portable Snygg domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggCompilerAdapter.kt` | 125 | M | Resources colour/font materialization / ART JNI; retire Compose-domain object tokens after ports |
| `SNYGG/SnyggEditorProperties.kt` | 58 | P | editor map ownership and concurrent-modification semantics |
| `SNYGG/SnyggGrammarAdapter.kt` | 90 | P | portable SnyggGrammarAdapter domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggJsonSchemaGenerator.kt` | 202 | P | portable schema generation; source-set tooling, not Android API |
| `SNYGG/SnyggPropertyCodec.kt` | 51 | P | portable SnyggPropertyCodec domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggPropertySet.kt` | 254 | P | portable SnyggPropertySet domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggPropertySetEditor.kt` | 318 | P | portable SnyggPropertySetEditor domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggRule.kt` | 250 | P | portable SnyggRule domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggSpecDecl.kt` | 196 | P | schema declaration and builder state |
| `SNYGG/SnyggStylesheet.kt` | 106 | P | portable SnyggStylesheet domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggStylesheetEditor.kt` | 92 | P | portable SnyggStylesheetEditor domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/SnyggTheme.kt` | 87 | M | Android font files / ColorScheme; loading and default materialization around native compiler |
| `SNYGG/ui/SnyggBox.kt` | 105 | M | Android background surface; remaining Compose box/layout/drawing |
| `SNYGG/ui/SnyggPalette.kt` | 17 | A | Resources / ART JNI; native palette projection |
| `SNYGG/ui/SnyggSurfaceView.kt` | 209 | M | SurfaceView / Drawable / Canvas; background scale/alignment/drawing must move to Rust |
| `SNYGG/ui/SnyggUi.kt` | 361 | M | Resources fonts/density; modifier/layout/style/default conversion and Compose drawing |
| `SNYGG/value/NativeSnyggValues.kt` | 85 | P | portable NativeSnyggValues domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggAppearanceValue.kt` | 73 | P | portable SnyggAppearanceValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggFontValue.kt` | 93 | P | portable SnyggFontValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggIdToValueMap.kt` | 41 | P | portable SnyggIdToValueMap domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggPaddingValue.kt` | 29 | P | portable SnyggPaddingValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggShapeValue.kt` | 112 | P | portable SnyggShapeValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggSizeValue.kt` | 45 | P | portable SnyggSizeValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggTextValue.kt` | 76 | P | portable SnyggTextValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggUriValue.kt` | 69 | P | portable SnyggUriValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggValue.kt` | 111 | P | portable SnyggValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggValueEncoder.kt` | 86 | P | portable SnyggValueEncoder domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggValueSpec.kt` | 267 | P | portable SnyggValueSpec domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |
| `SNYGG/value/SnyggVarValue.kt` | 29 | P | portable SnyggVarValue domain model/codec/state; consolidate Rust ownership, retain only necessary boundary DTOs |

## Files already covered by settings/Compose-keyboard work

These are excluded from the class totals above, not newly ruled dead. Preserve every feature
and extract Android APIs before accepted deletion. Scope names identify the existing plan:
`stage4` is settings drawing/helpers; `keyboard` is the retained Compose keyboard; combined
primitives serve both. Actual deletion remains behind the original acceptance gates.

| File | Nonblank | Existing scope |
| --- | ---: | --- |
| `AW/dictation/DictationPreview.kt` | 30 | keyboard |
| `AW/dictation/DictationScreen.kt` | 62 | stage4 |
| `AW/dictation/LitMic.kt` | 104 | keyboard |
| `AW/dictation/ModelStatus.kt` | 65 | stage4 |
| `AW/gif/GifPanel.kt` | 319 | keyboard |
| `AW/reader/LitSpeaker.kt` | 29 | keyboard |
| `FB/app/ext/AddonBox.kt` | 141 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/CheckUpdatesScreen.kt` | 32 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionComponentView.kt` | 175 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionEditFilesScreen.kt` | 276 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionEditScreen.kt` | 829 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionExportScreen.kt` | 74 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionHomeScreen.kt` | 63 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionImportScreen.kt` | 312 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionKeywordChip.kt` | 33 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionList.kt` | 48 | stage4 (deleted in 9ebadce41, with code nothing reached) |
| `FB/app/ext/ExtensionListScreen.kt` | 176 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionMaintainerChip.kt` | 113 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionNotFoundScreen.kt` | 36 | stage4 (deleted at compose-removal step 10) |
| `FB/app/ext/ExtensionViewScreen.kt` | 282 | stage4 (deleted at compose-removal step 10) |
| `FB/app/settings/HomeScreen.kt` | 163 | stage4 |
| `FB/app/settings/about/AboutScreen.kt` | 123 | stage4 |
| `FB/app/settings/about/ProjectLicenseScreen.kt` | 65 | stage4 (deleted at compose-removal step 8) |
| `FB/app/settings/about/ThirdPartyLicensesScreen.kt` | 68 | stage4 |
| `FB/app/settings/advanced/BackupScreen.kt` | 362 | stage4 |
| `FB/app/settings/advanced/OtherScreen.kt` | 95 | stage4 |
| `FB/app/settings/advanced/PhysicalKeyboardScreen.kt` | 57 | stage4 |
| `FB/app/settings/advanced/RestoreScreen.kt` | 265 | stage4 |
| `FB/app/settings/clipboard/ClipboardScreen.kt` | 153 | stage4 |
| `FB/app/settings/dictionary/DictionaryScreen.kt` | 54 | stage4 |
| `FB/app/settings/dictionary/UserDictionaryScreen.kt` | 385 | stage4 |
| `FB/app/settings/gestures/GesturesScreen.kt` | 154 | stage4 |
| `FB/app/settings/keyboard/InputFeedbackScreen.kt` | 186 | stage4 |
| `FB/app/settings/keyboard/KeyboardScreen.kt` | 192 | stage4 |
| `FB/app/settings/localization/LanguagePackManagerScreen.kt` | 208 | stage4 |
| `FB/app/settings/localization/LocalizationScreen.kt` | 179 | stage4 |
| `FB/app/settings/localization/SelectLocaleScreen.kt` | 142 | stage4 |
| `FB/app/settings/localization/SubtypeEditorScreen.kt` | 449 | stage4 |
| `FB/app/settings/localization/SubtypeEditorState.kt` | 100 | stage4 |
| `FB/app/settings/media/MediaScreen.kt` | 204 | stage4 |
| `FB/app/settings/smartbar/SmartbarScreen.kt` | 102 | stage4 |
| `FB/app/settings/theme/Color.kt` | 24 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/DisplayKbdAfterDialog.kt` | 26 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/EditCodeValueDialog.kt` | 312 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/EditPropertyDialog.kt` | 1200 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/EditRuleDialog.kt` | 293 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/EnumLikeAttributeBox.kt` | 113 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/FineTuneDialog.kt` | 54 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/SnyggLevel.kt` | 29 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/SnyggValueIcon.kt` | 320 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/TextKeyDataPreviewBox.kt` | 114 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/ThemeEditorComponents.kt` | 344 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/ThemeEditorLoading.kt` | 70 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/ThemeEditorScreen.kt` | 473 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/ThemeManagerScreen.kt` | 135 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/theme/ThemeScreen.kt` | 118 | stage4 |
| `FB/app/settings/theme/ThemeTranslations.kt` | 177 | stage4 (deleted at compose-removal step 9) |
| `FB/app/settings/typing/SpellCheckerServiceSelector.kt` | 122 | stage4 |
| `FB/app/settings/typing/TypingScreen.kt` | 199 | stage4 |
| `FB/ime/clipboard/ClipboardComponents.kt` | 114 | keyboard |
| `FB/ime/clipboard/ClipboardHeader.kt` | 131 | keyboard |
| `FB/ime/clipboard/ClipboardInputLayout.kt` | 440 | keyboard |
| `FB/ime/clipboard/ClipboardItemView.kt` | 165 | keyboard |
| `FB/ime/core/SelectSubtypePanel.kt` | 90 | keyboard |
| `FB/ime/media/MediaInputLayout.kt` | 194 | keyboard |
| `FB/ime/media/emoji/EmojiPaletteView.kt` | 619 | keyboard |
| `FB/ime/onehanded/OneHandedPanel.kt` | 104 | keyboard |
| `FB/ime/popup/PopupUi.kt` | 130 | keyboard |
| `FB/ime/popup/PopupUiController.kt` | 290 | keyboard |
| `FB/ime/sheet/BottomSheetHostUi.kt` | 66 | keyboard |
| `FB/ime/smartbar/CandidatesRow.kt` | 229 | keyboard |
| `FB/ime/smartbar/Smartbar.kt` | 416 | keyboard |
| `FB/ime/smartbar/quickaction/QuickActionButton.kt` | 219 | keyboard |
| `FB/ime/smartbar/quickaction/QuickActionsEditorPanel.kt` | 339 | keyboard |
| `FB/ime/smartbar/quickaction/QuickActionsOverflowPanel.kt` | 89 | keyboard |
| `FB/ime/smartbar/quickaction/QuickActionsRow.kt` | 94 | keyboard |
| `FB/ime/text/TextInputLayout.kt` | 94 | keyboard |
| `FB/ime/text/keyboard/TextKeyboardLayout.kt` | 323 | keyboard |
| `FB/ime/text/keyboard/TextKeyboardLayoutController.kt` | 401 | keyboard |
| `FB/ime/text/keyboard/TextKeyboardSwipe.kt` | 253 | keyboard |
| `FB/lib/compose/FlorisDialogs.kt` | 61 | stage4 (deleted at compose-removal step 12) |
| `FB/lib/compose/FlorisHyperlinkText.kt` | 45 | stage4 (deleted at compose-removal step 12) |
| `FB/lib/compose/FlorisScreen.kt` | 135 | stage4 (deleted at compose-removal step 12) |
| `FB/lib/compose/ImageVector.kt` | 28 | keyboard (stage4 until compose-removal step 12; only the Compose keyboard uses it since) |
| `FB/lib/compose/PreviewKeyboardField.kt` | 138 | stage4 (deleted at compose-removal step 12) |
| `FB/lib/compose/Validation.kt` | 42 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/ClickableModifiers.kt` | 38 | keyboard (stage4 until compose-removal step 12; only the Compose keyboard uses it since) |
| `COMPOSE/DpSpSizeFuns.kt` | 35 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisAnimation.kt` | 50 | keyboard (stage4 until compose-removal step 12; only the Compose keyboard uses it since) |
| `COMPOSE/FlorisAppBar.kt` | 46 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisBulletSpacer.kt` | 41 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisButtonBar.kt` | 119 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisButtons.kt` | 156 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisCanvasIcon.kt` | 62 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisCards.kt` | 294 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisChip.kt` | 81 | stage4 (deleted at compose-removal step 12) |
| `COMPOSE/FlorisDropdownLikeButton.kt` | 68 | stage4 |
| `COMPOSE/FlorisStepLayout.kt` | 283 | stage4 |
| `COMPOSE/LazyGrid.kt` | 26 | keyboard (stage4 until compose-removal step 12; only the Compose keyboard uses it since) |
| `COMPOSE/RtlModifiers.kt` | 28 | keyboard (stage4 until compose-removal step 12; only the Compose keyboard uses it since) |
| `COMPOSE/ScrollableModifiers.kt` | 257 | keyboard (stage4 until compose-removal step 12; only the Compose keyboard uses it since) |
| `COMPOSE/icons/KeyboardForwardDelete.kt` | 61 | keyboard (stage4 until compose-removal step 12; only the Compose keyboard uses it since) |
| `SNYGG/ui/SnyggButton.kt` | 86 | keyboard/settings |
| `SNYGG/ui/SnyggChip.kt` | 139 | keyboard/settings |
| `SNYGG/ui/SnyggColumn.kt` | 91 | keyboard/settings |
| `SNYGG/ui/SnyggIcon.kt` | 161 | keyboard/settings |
| `SNYGG/ui/SnyggIconButton.kt` | 116 | keyboard/settings |
| `SNYGG/ui/SnyggListItem.kt` | 158 | keyboard/settings |
| `SNYGG/ui/SnyggRow.kt` | 98 | keyboard/settings |
| `SNYGG/ui/SnyggSpacer.kt` | 87 | keyboard/settings |
| `SNYGG/ui/SnyggText.kt` | 126 | keyboard/settings |

Reproduction: enumerate `git ls-files "*.kt" "*.java"`, retain module `src/main` paths
excluding `benchmark/`, count `bool(line.strip())`, then join to the two ledgers above.
The ledgers must cover every production path exactly once. Tests and tooling remain outside
the production inventory. Research receipts (including imports, declarations, branch
candidates and class totals) are under ignored `build/stage4-followup/inventory-*.json`.
No generated build products or user text are committed.
