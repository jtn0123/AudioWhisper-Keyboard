# Phase 6: the Kotlin references become golden files

2026-11-23. This is the second half of
[phase 6](all-rust.md#6-retire-the-references-m-c-half-done). The owner decided that the frozen
`Reference*.kt` test copies become fixed test data, and then are deleted
([decision](decisions.md#everything-moves-to-rust-2026-11-21)). The C++ half was done first
([record](phase6-oracle-goldens.md)); this page follows its method.

Each `Reference*.kt` is a frozen copy of Kotlin that has since moved to Rust (or of a Compose screen
Rust now draws), and a differential test holds the Rust's answers to the copy's. For each copy the
choice is:

1. **Retire it.** Record the copy's answers to the test's seeded questions as a golden file, have
   the test check the Rust's answers against that file every run, prove with the mutation scripts
   that the file catches what the copy caught, then delete the copy.
2. **Keep it, with a reason**, recorded in the table below.

## The inventory

Taken at `0500aa74f`, before anything was deleted: 166 files, 15,850 counted lines (non-blank,
non-comment, as `scripts/jvm-ledger.py` counts them; 21,648 lines in all). "Frozen at" is the
commit that made the file a test copy; the commit in "Copy of", where there is one, is the source
it was copied from. "Tests that use it" leaves out other `Reference*.kt` files; `AT:` is an
instrumented (device) test. Every copy mirrors code that is now Rust: where a production Kotlin
class of the same name is left, it is a thin adapter over the Rust.

The verdict column is the plan at `0500aa74f`, changed twice on 2026-11-23. First, another branch
took the render path off Compose, Snygg's palette with it, so its references are kept "in flight,
the Compose-free render path". Then the second branch kept three more that the render path or
the test drivers stand on, the lead kept three Snygg copies with the palette's two, and two small
codecs moved to the app family that uses them. [What happened](#what-happened) says, family by
family, which were retired and which were left.

* **Unused (7 files).** No test reads them: nothing compares against them. They are deleted
  without goldens, since there is nothing to record.
* **Render (8 files): kept, in flight, the Compose-free render path.** Another branch is taking
  the render path off Compose (icons, Compose types, the Material You palette maths, theme fonts),
  and owns `app/src/test/.../render`.
* **The render references build on 5 more:** `ReferenceLayoutAssembly`, `ReferencePopupSet`,
  `ReferencePopupGrid`, `ReferenceSwipeGesture` and `ReferenceGlideTypingGesture`. They stay
  until the render references go.
* **Device comparisons (19 files): kept.** The 11 under `app/src/androidTest` (the unused
  `ReferenceSpeaker` aside) and the 8 in `app/src/test` directories that `app/build.gradle.kts`
  compiles into the instrumented tests too (`suggest/a1`, `gif/shared`, `lib/ext/replay`). They
  compare on a device (`DictationInstrumentation`, `TtsInstrumentation`, `A1Instrumentation`,
  `RendererFixtureTest`, `ExtensionDeviceReplay`). Golden files for them would need a device run,
  and this work had no device.
* **Snygg's palette (2 files): kept, in flight, the Compose-free render path.**
  `ReferenceSnyggTheme` resolves dynamic colours through Material 3's colour schemes, and
  `ReferenceSnyggAppearanceValue` holds the colour values named by `ColorPalette`. The palette
  code they guard is that branch's.
* **The rest (125 files): retire.**

| Family | File | Lines | Copy of | Frozen at | Tests that use it | Verdict |
|---|---|---:|---|---|---|---|
| dictation | `AT:aw/dictation/ReferenceAudioHistory.kt` | 18 | AudioHistory (the listener's recent audio) | `9a269ccb2` | AudioHistoryTest | Keep: device comparison (androidTest) |
| dictation | `AT:aw/dictation/ReferenceDictationSession.kt` | 270 | DictationSession before its state moved to Rust | `9a269ccb2` | AT:LiveSessionComparison, DictationSessionMatchTest | Keep: device comparison (androidTest) |
| dictation | `AT:aw/dictation/ReferenceMicKey.kt` | 29 | the mic key's tap and hold (Kotlin adapter over Rust's) | `9a269ccb2` | MicKeyComparison, MicKeyMatchTest, MicKeyTest | Keep: device comparison (androidTest) |
| dictation | `AT:aw/dictation/ReferenceModelHolder.kt` | 24 | ModelHolder (load once, remember a failed load) | `9a269ccb2` | none | Keep: device comparison (androidTest) |
| dictation | `AT:aw/dictation/ReferencePhraseListener.kt` | 77 | PhraseListener (VAD phrase cutting) | `9a269ccb2` | AT:DictationInstrumentation | Keep: device comparison (androidTest) |
| dictation | `AT:aw/dictation/ReferencePhrases.kt` | 11 | Phrases (adapter over Rust's phrase joining) | `9a269ccb2` | DictationRulesMatchTest, PhrasesTest, SpokenTest | Keep: device comparison (androidTest) |
| dictation | `AT:aw/dictation/ReferenceSpoken.kt` | 19 | Spoken (adapter over Rust's command parsing) | `9a269ccb2` | DictationRulesMatchTest, SpokenTest | Keep: device comparison (androidTest) |
| dictation | `AT:aw/dictation/ReferenceTranscriber.kt` | 39 | Transcriber (Parakeet through sherpa-onnx) | `9a269ccb2` | AT:DictationInstrumentation | Keep: device comparison (androidTest) |
| dictation | `aw/dictation/ReferenceAudioHistory.kt` | 18 | AudioHistory (the listener's recent audio) | `9a269ccb2` | AudioHistoryTest | Retire |
| dictation | `aw/dictation/ReferenceDictation.kt` | 98 | Spoken.kt, Phrases.kt, DictationUndo.kt (2b55078b) | `30eef4b80` | DictationRulesMatchTest | Retire |
| dictation | `aw/dictation/ReferenceDictationSession.kt` | 270 | DictationSession before its state moved to Rust | `9a269ccb2` | AT:LiveSessionComparison, DictationSessionMatchTest | Retire |
| dictation | `aw/dictation/ReferenceMic.kt` | 74 | MicRecorder, MicAudio.Replay.pcm16, Mic.bufferBytes (b1cf45f18) | `8d7ee540c` | MicTraceTest | Retire |
| dictation | `aw/dictation/ReferenceMicKey.kt` | 24 | the Kotlin MicKey (04c48f72c) | `04c48f72c` | MicKeyComparison, MicKeyMatchTest | Retire |
| dictation | `aw/dictation/ReferenceMicKeyAdapter.kt` | 29 | the mic key's tap and hold (Kotlin adapter over Rust's) | `9a269ccb2` | MicKeyComparison, MicKeyMatchTest, MicKeyTest | Retire |
| dictation | `aw/dictation/ReferenceModelCheck.kt` | 39 | ModelCheck (b1cf45f18) | `8d7ee540c` | ModelCheckTraceTest | Retire |
| dictation | `aw/dictation/ReferenceModelHolder.kt` | 24 | ModelHolder (load once, remember a failed load) | `9a269ccb2` | none | Retire |
| dictation | `aw/dictation/ReferencePhrases.kt` | 11 | Phrases (adapter over Rust's phrase joining) | `9a269ccb2` | DictationRulesMatchTest, PhrasesTest, SpokenTest | Retire |
| dictation | `aw/dictation/ReferenceSpoken.kt` | 19 | Spoken (adapter over Rust's command parsing) | `9a269ccb2` | DictationRulesMatchTest, SpokenTest | Retire |
| reader | `AT:aw/reader/ReferenceReader.kt` | 149 | PageText, TextBlock.isShown, FollowAlong.fits, SpeakerKey | `8817f82df` | AT:BoundaryReplay, ReaderComparison, ReaderRulesMatchTest, TtsChunkMatchTest | Keep: device comparison (androidTest) |
| reader | `AT:aw/reader/ReferenceReleasedOnce.kt` | 17 | ReleasedOnce (use/close guard) | `a4e630fd6` | ReleasedOnceTest | Keep: device comparison (androidTest) |
| reader | `AT:aw/reader/ReferenceSpeaker.kt` | 69 | Speaker (the phone's own TTS) | `a4e630fd6` | none | Retire: unused, no test reads it |
| reader | `AT:aw/reader/ReferenceVoiceEngine.kt` | 88 | VoiceEngine (Pocket TTS through sherpa-onnx) | `a4e630fd6` | AT:TtsInstrumentation | Keep: device comparison (androidTest) |
| reader | `aw/reader/ReferenceFollowAlongFacade.kt` | 4 | FollowAlong.fits (adapter over Rust's) | `65fc5b346` | FollowAlongTest, KeyboardToastTextTest, ReaderComparison, ReaderRulesMatchTest, ReaderRulesTest, TextKeyboardComparison, TextKeyboardMatchTest | Retire |
| reader | `aw/reader/ReferenceLoadedVoice.kt` | 57 | LoadedVoice (b1cf45f18) | `8633cc255` | LoadedVoiceTraceTest | Retire |
| reader | `aw/reader/ReferencePlayQueue.kt` | 44 | PlayQueue (the neural voice's timeline) | `a4e630fd6` | PlaybackQueueMatchTest | Retire |
| reader | `aw/reader/ReferenceReader.kt` | 149 | PageText, TextBlock.isShown, FollowAlong.fits, SpeakerKey (66706ff4) | `8a68277d9` | AT:BoundaryReplay, ReaderComparison, ReaderRulesMatchTest, TtsChunkMatchTest | Retire |
| reader | `aw/reader/ReferenceReading.kt` | 142 | ReadAloudService's reading (b1cf45f18) | `2e5737672` | ReadingTraceTest | Retire |
| reader | `aw/reader/ReferenceReleasedOnce.kt` | 17 | ReleasedOnce (use/close guard) | `a4e630fd6` | ReleasedOnceTest | Retire |
| reader | `aw/reader/ReferenceScreen.kt` | 50 | ScreenRules.kt (6c6821ce) | `4ea75f05c` | ScreenComparison, ScreenRulesMatchTest | Retire |
| reader | `aw/reader/ReferenceScreenRulesFacade.kt` | 26 | ScreenRules' Box (adapter over Rust's) | `65fc5b346` | ScreenComparison, ScreenRulesTest | Retire |
| reader | `aw/reader/ReferenceScreenWalk.kt` | 43 | ScreenWalk.visit/walk (dc7256e5) | `a4e630fd6` | ScreenTraversalMatchTest | Retire |
| gif | `aw/gif/ReferenceGif.kt` | 169 | GifSearch.kt's rules, GifSearchKeys, GifResultCache, GifRecents (68e56337) | `3815d2c1b` | GifComparison, GifRulesMatchTest | Retire |
| gif | `aw/gif/ReferenceGifSearch.kt` | 217 | GifSearch, Giphy's fetch, GifImages, GifSender sequencing (b1cf45f18) | `0a715ae9c` | GifSearchTraceTest | Retire |
| gif | `aw/gif/ReferenceGiphy.kt` | 54 | reading GIPHY's answers and building its URLs (3cbbc5a89) | `3cbbc5a89` | GifComparison, GiphyComparison, GiphyRulesMatchTest | Retire |
| gif | `aw/gif/ReferenceLruCache.kt` | 25 | android.util.LruCache, as GifImages used it | `0a715ae9c` | none | Retire |
| gif | `aw/gif/shared/ReferenceSearchTextFacade.kt` | 11 | SearchText (adapter over Rust's search-field editing) | `65fc5b346` | AT:RendererFixtureTest, GifComparison, SearchTextTest | Keep: compiled into androidTest for a device comparison |
| suggest | `aw/suggest/ReferenceBoundaryGlideRanker.kt` | 22 | GlideRanker at the JNI boundary | `8817f82df` | BoundaryGlideMatchTest | Retire |
| suggest | `aw/suggest/ReferenceBoundaryLearner.kt` | 17 | WordLearner at the JNI boundary | `8817f82df` | BoundaryLearnerMatchTest | Retire |
| suggest | `aw/suggest/ReferenceLatin.kt` | 217 | LatinSuggester.kt and WordLearner.kt (f89496da) | `66706ff4c` | BoundaryLearnerMatchTest, LatinComparison, LatinRulesMatchTest, NlpLearningMatchTest | Retire |
| suggest | `aw/suggest/ReferenceNlpSwipeWords.kt` | 26 | NlpSwipeWords (b00f8c561) | `b00f8c561` | none | Retire: unused, no test reads it |
| suggest | `aw/suggest/ReferenceStores.kt` | 246 | LearnedWords, KeptWords, PersonalWords, TypingHistory (dd0543de) | `d09f766cf` | StoreComparison, WordStoresMatchTest | Retire |
| suggest | `aw/suggest/ReferenceSuggest.kt` | 259 | WordCharacters, WordCase, TapTrail, QwertyGeometry, PreviousWords, AutoCorrection(Undo) (d36ff93f) | `dd0543de7` | SuggestComparison, SuggestRulesMatchTest | Retire |
| suggest | `aw/suggest/ReferenceSwipe.kt` | 74 | GlideRanker.kt and SwipeWords.kt (d09f766c) | `f89496dae` | SwipeComparison, SwipeRulesMatchTest | Retire |
| suggest | `aw/suggest/ReferenceSwipeWordsFactory.kt` | 6 | a SwipeWords lookup adapter for the reference tests | `b00f8c561` | SwipeComparison, SwipeWordsTest | Retire |
| suggest | `aw/suggest/a1/ReferenceAutoCorrectionFacade.kt` | 10 | AutoCorrection (adapter over Rust's) | `65fc5b346` | AutoCorrectionTest, LatinComparison, SuggestComparison | Keep: compiled into androidTest for a device comparison |
| suggest | `aw/suggest/a1/ReferenceLatinDictionaryEngine.kt` | 114 | LatinDictionaryEngine before the native session (7c06586fa) | `7c06586fa` | A1Replay, A1SessionMatchTest, SplitWordsTest | Keep: compiled into androidTest for a device comparison |
| suggest | `aw/suggest/a1/ReferenceLatinSuggester.kt` | 43 | LatinSuggester before the native session (7c06586fa) | `7c06586fa` | A1Replay, LatinComparison, LatinSuggesterTest | Keep: compiled into androidTest for a device comparison |
| suggest | `aw/suggest/a1/ReferenceQwertyGeometry.kt` | 20 | QwertyGeometry (adapter over Rust's) | `7c06586fa` | none | Keep: compiled into androidTest for a device comparison |
| suggest | `aw/suggest/a1/ReferenceWordCaseFacade.kt` | 17 | WordCase (adapter over Rust's) | `65fc5b346` | SuggestComparison, WordCaseTest | Keep: compiled into androidTest for a device comparison |
| suggest | `aw/suggest/a1/ReferenceWordCharactersFacade.kt` | 5 | WordCharacters (adapter over Rust's) | `65fc5b346` | SuggestComparison, WordCharactersTest | Keep: compiled into androidTest for a device comparison |
| settingsui | `aw/settingsui/ReferenceExtensionEditor.kt` | 36 | the Compose extension editor's decisions (step 10 part 3) | `3a883efc6` | SettingsExtensionEditorMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceExtensionScreens.kt` | 45 | the Compose extension screens' decisions (step 10) | `14eb479f6` | SettingsExtensionEditorMatchTest, SettingsExtensionsMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceGroupOneScreens.kt` | 136 | InputFeedback, Smartbar, Gestures, Clipboard and Media screens (33f903a2) | `ab9084acd` | SettingsGroupOneMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceGroupTwoScreens.kt` | 131 | Typing, Dictionary, PhysicalKeyboard, About and Dictation screens (d752cb96) | `f5f3421cc` | SettingsGroupOneMatchTest, SettingsGroupTwoMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceLicenceScreens.kt` | 65 | ThirdPartyLicensesScreen over AboutLibraries 15.3.0 | `df3f05c95` | SettingsLicencesMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceOtherThemeScreens.kt` | 131 | OtherScreen and ThemeScreen (b0d4a9a4f) | `089e8113d` | SettingsGroupOneMatchTest, SettingsGroupTwoMatchTest, SettingsOtherThemeMatchTest, SettingsThemeEditorColourMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceSettingsDrag.kt` | 78 | Compose 1.12.1 verticalScroll drag and fling | `c2a8f824f` | SettingsDragMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceSettingsScreens.kt` | 41 | KeyboardScreen and HomeScreen (04186d1d) | `d4ac289f0` | SettingsUiMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceSettingsScrollbar.kt` | 9 | lib/compose ScrollableModifiers' scrollbar timing | `50c8bc232` | SettingsScrollbarMatchTest | Retire |
| settingsui | `aw/settingsui/ReferenceThemeEditor.kt` | 52 | ThemeEditorComponents' CustomRuleComparator, ThemeTranslations' PropertyNameMap | `a186ed5cd` | SettingsThemeEditorMatchTest | Retire |
| app | `fb/app/ReferenceColorSerializer.kt` | 9 | the Jetpref colour serializer (586890ba1) | `586890ba1` | SettingsDerivationMatchTest, SettingsOtherThemeMatchTest | Retire |
| app | `fb/app/ReferencePreferenceMigration.kt` | 143 | migrateAppPreference (586890ba1) | `586890ba1` | SettingsMigrationMatchTest | Retire |
| app | `fb/app/ReferencePreferenceModel.kt` | 282 | FlorisPreferenceModel's declarations (586890ba1) | `586890ba1` | none | Retire |
| app | `fb/app/ReferenceQuickActionDefaults.kt` | 32 | the default quick-action arrangement (586890ba1) | `586890ba1` | SettingsCodecMatchTest | Retire |
| app | `fb/app/ReferenceSettingsBounds.kt` | 30 | the Compose sliders' bounds (33b8db50) | `586890ba1` | SettingsDefaultsMatchTest, SettingsGroupOneMatchTest | Retire |
| app | `fb/app/ReferenceSettingsCodecs.kt` | 13 | the Jetpref JSON codecs (586890ba1) | `586890ba1` | SettingsCodecMatchTest, SettingsLanguagesMatchTest, SettingsMigrationMatchTest, SettingsNumericMatchTest, SettingsStoredFileMatchTest, SettingsSubtypeMatchTest | Retire |
| app | `fb/app/ReferenceSettingsModel.kt` | 153 | the preference model's declared entries (586890ba1) | `586890ba1` | SettingsDefaultsMatchTest, SettingsGroupOneMatchTest, SettingsGroupTwoMatchTest, SettingsOtherThemeMatchTest, SettingsStoredFileMatchTest, SettingsUiMatchTest, SettingsUiStoredFileMatchTest, SettingsUiWriteCancellationMatchTest | Retire |
| app | `fb/app/ReferenceSubtype.kt` | 186 | Subtype, SubtypePreset and their maps (586890ba1) | `586890ba1` | SettingsCodecMatchTest | Retire |
| app | `fb/app/ReferenceSubtypeEditorState.kt` | 95 | SubtypeEditorState (586890ba1) | `586890ba1` | SettingsLanguagesMatchTest | Retire |
| app | `fb/app/ReferenceSubtypeFallback.kt` | 73 | the removed-layout fallbacks (586890ba1) | `586890ba1` | SettingsSubtypeMatchTest | Retire |
| app | `fb/app/ReferenceSubtypeManager.kt` | 133 | SubtypeManager's codec and repair (586890ba1) | `586890ba1` | SettingsCodecMatchTest, SubtypeSessionTest | Retire |
| app | `fb/app/settings/advanced/ReferenceBackupPolicy.kt` | 46 | BackupScreen/RestoreScreen and ClipboardManager branches (1d2cbc1f6) | `1d2cbc1f6` | BackupArchiveMatchTest, BackupFilesTest, BackupPlanMatchTest, SettingsBackupFlowTest | Retire |
| editor/input | `fb/ime/editor/ReferenceAbstractEditorInstance.kt` | 290 | AbstractEditorInstance (c07983c8d) | `c07983c8d` | EditorPlanMatchTest | Retire |
| editor/input | `fb/ime/editor/ReferenceEditorContent.kt` | 34 | EditorContent (c07983c8d) | `c07983c8d` | none | Retire |
| editor/input | `fb/ime/editor/ReferenceEditorFlow.kt` | 118 | EditorInstance's replaced branches (c07983c8d) | `c07983c8d` | EditorFlowMatchTest | Retire |
| editor/input | `fb/ime/editor/ReferenceEditorRange.kt` | 26 | EditorRange (c07983c8d) | `c07983c8d` | EditorBoundaryTest, EditorPlanMatchTest, FakeInputConnection | Keep: the fake field answers Rust with it |
| editor/input | `fb/ime/editor/ReferenceEditorState.kt` | 280 | EditorInstance's state handling (c07983c8d) | `c07983c8d` | none | Retire |
| editor/input | `fb/ime/editor/ReferenceImeOptions.kt` | 41 | ImeOptions (c07983c8d) | `c07983c8d` | EditorPlanMatchTest | Retire |
| editor/input | `fb/ime/editor/ReferenceInputAttributes.kt` | 120 | InputAttributes (c07983c8d) | `c07983c8d` | EditorPlanMatchTest | Retire |
| editor/input | `fb/ime/editor/ReferencePlatform.kt` | 54 | the platform facts both editor sides read | `c07983c8d` | EditorPlanMatchTest, KeyGraphTrace, NativeEditorDriver, NlpMatchTest, SettingsCodecMatchTest | Keep: the driver answers Rust with it; render builds on it |
| editor/input | `fb/ime/editor/ReferenceSpacing.kt` | 153 | EditorInstance's auto-space and phantom-space rules (f44cef4d9) | `f44cef4d9` | SpacingComparison, SpacingMatchTest | Retire |
| editor/input | `fb/ime/editor/ReferenceSpacingAdapter.kt` | 65 | Spacing (adapter over Rust's) | `c07983c8d` | SpacingComparison, SpacingMatchTest | Retire |
| editor/input | `fb/ime/input/reference/ReferenceInputControl.kt` | 40 | KeyboardManager's control branches (c826ca10e) | `c826ca10e` | InputControlMatchTest | Retire |
| editor/input | `fb/ime/input/reference/ReferenceInputDelete.kt` | 20 | KeyboardEditing's delete branches (c826ca10e) | `c826ca10e` | InputDeleteMatchTest | Retire |
| editor/input | `fb/ime/input/reference/ReferenceInputDispatcher.kt` | 54 | InputEventDispatcher (c826ca10e) | `c826ca10e` | InputDispatchMatchTest, InputFixtureMatchTest | Retire |
| editor/input | `fb/ime/input/reference/ReferenceInputFeedback.kt` | 22 | InputFeedbackController's branches (c826ca10e) | `c826ca10e` | InputPolicyMatchTest | Retire |
| editor/input | `fb/ime/input/reference/ReferenceInputPointer.kt` | 17 | Compose popup, rolling and hold decisions (c826ca10e) | `c826ca10e` | InputPolicyMatchTest | Retire |
| editor/input | `fb/ime/input/reference/ReferenceInputVisual.kt` | 34 | ComputingEvaluator's labels and icons (c826ca10e) | `c826ca10e` | InputPolicyMatchTest | Retire |
| editor/input | `fb/ime/input/reference/ReferenceKeyboardKeyEvents.kt` | 177 | KeyboardManager's key routing (c826ca10e) | `c826ca10e` | none | Retire |
| editor/input | `fb/ime/input/reference/ReferenceRoutingManager.kt` | 103 | KeyboardManager's routing state (c826ca10e) | `c826ca10e` | InputDispatchMatchTest | Retire |
| keyboard | `fb/ime/core/ReferenceKeyboardSubtypes.kt` | 95 | SubtypeManager (61f1513fb) | `0a9fbaaf4` | SubtypeMatchTest, SubtypeTrace, SubtypeTraceSides | Keep: its trace drives the render reference `ReferenceSubtypeBuildRow` |
| keyboard | `fb/ime/core/ReferenceSubtypeJson.kt` | 3 | the persisted subtype JSON codec | `586890ba1` | SettingsCodecMatchTest, SubtypeSessionTest | Retire, with the app family |
| keyboard | `fb/ime/dictionary/ReferenceUserDictionary.kt` | 121 | UserDictionaryScreen and UserDictionary (f5a6fc180) | `52a7f1fe9` | SettingsWordsMatchTest, SettingsWordsTest | Retire |
| keyboard | `fb/ime/keyboard/ReferenceKeyCodes.kt` | 98 | KeyCode and its helpers (82543cee2) | `82543cee2` | KeyModelsMatchTest | Retire |
| keyboard | `fb/ime/keyboard/ReferenceKeyMetadata.kt` | 68 | key metadata rules (82543cee2) | `82543cee2` | KeyModelsMatchTest | Retire |
| keyboard | `fb/ime/keyboard/ReferenceKeyRules.kt` | 71 | key data rules (82543cee2) | `82543cee2` | KeyModelsMatchTest, KeySelectorsMatchTest, LayoutFixtures, LayoutSnapshot | Retire |
| keyboard | `fb/ime/keyboard/ReferenceKeySelectors.kt` | 82 | the case, width, direction, shift and variation selectors (82543cee2) | `82543cee2` | KeySelectorsMatchTest | Retire |
| keyboard | `fb/ime/keyboard/ReferenceKeyboardState.kt` | 106 | KeyboardState (82543cee2) | `82543cee2` | KeyModelsMatchTest | Retire |
| keyboard | `fb/ime/keyboard/ReferenceLayoutAssembly.kt` | 92 | LayoutManager's assembly (310e74a02) | `310e74a02` | KeyGraphTrace, LayoutAssemblyMatchTest | Keep: the render references build on it |
| keyboard | `fb/ime/keyboard/ReferenceShift.kt` | 69 | KeyboardManager's shift handlers (7dca4f587) | `7dca4f587` | ShiftComparison, ShiftMatchTest | Retire |
| keyboard | `fb/ime/keyboard/ReferenceTextKeyboardCache.kt` | 10 | TextKeyboardCache (310e74a02) | `310e74a02` | LayoutAssemblyMatchTest | Retire |
| keyboard | `fb/ime/media/emoji/ReferenceEmoji.kt` | 159 | emoji data and suggestions (b0cc7212f) | `b0cc7212f` | EmojiComparison, EmojiInputs, EmojiMatchTest | Retire |
| keyboard | `fb/ime/popup/ReferencePopupGrid.kt` | 212 | the extended popup's grid (72f16306f) | `72f16306f` | PopupComparison, PopupMatchTest, RendererPopupMatchTest | Keep: the render references build on it |
| keyboard | `fb/ime/popup/ReferencePopupSet.kt` | 112 | MutablePopupSet (72f16306f) | `72f16306f` | PopupComparison, PopupMatchTest | Keep: the render references build on it |
| keyboard | `fb/ime/text/composing/ReferenceWithRules.kt` | 20 | the rule-table composer (8dada20bb) | `8dada20bb` | ComposingComparison | Retire |
| keyboard | `fb/ime/text/gestures/ReferenceGlideClassifier.kt` | 389 | StatisticalGlideTypingClassifier (ed5a2fe7d) | `ed5a2fe7d` | SwipeIndexTest | Retire |
| keyboard | `fb/ime/text/gestures/ReferenceGlideTypingGesture.kt` | 121 | GlideTypingGesture.Detector (f1f7a3338) | `f1f7a3338` | GesturesMatchTest, GlideGestureComparison | Keep: the render references build on it |
| keyboard | `fb/ime/text/gestures/ReferenceGlideTypingManager.kt` | 82 | GlideTypingManager (b00f8c561) | `b00f8c561` | none | Retire: unused, no test reads it |
| keyboard | `fb/ime/text/gestures/ReferenceSwipeGesture.kt` | 151 | SwipeGesture.Detector (f1f7a3338) | `f1f7a3338` | GesturesMatchTest, SwipeGestureComparison | Keep: the render references build on it |
| keyboard | `fb/ime/text/keyboard/ReferenceTextKeyboard.kt` | 220 | TextKeyboard's geometry (301df36de) | `301df36de` | LayoutSnapshot, TextKeyboardComparison, TextKeyboardMatchTest | Retire |
| nlp | `fb/ime/nlp/ReferenceClipboardSuggestionProvider.kt` | 94 | ClipboardSuggestionProvider (b00f8c561) | `b00f8c561` | none | Retire: unused, no test reads it |
| nlp | `fb/ime/nlp/ReferenceNlpInlineAutofill.kt` | 74 | NlpInlineAutofill (b00f8c561) | `b00f8c561` | none | Retire: unused, no test reads it |
| nlp | `fb/ime/nlp/ReferenceNlpManager.kt` | 333 | NlpManager (b00f8c561) | `b00f8c561` | NlpMatchTest | Retire |
| nlp | `fb/ime/nlp/ReferenceTypingFanOut.kt` | 174 | main's typing fan-out (phase 3 item 1) | `ae452c49b` | TypingTraceMatchTest, TypingTraceSides | Retire |
| nlp | `fb/ime/nlp/latin/ReferenceLatinLanguageProvider.kt` | 336 | LatinLanguageProvider (b00f8c561) | `b00f8c561` | none | Retire |
| nlp | `fb/ime/smartbar/ReferenceCandidatesRow.kt` | 213 | CandidatesRow (b00f8c561) | `b00f8c561` | none | Retire: unused, no test reads it |
| nlp | `fb/ime/smartbar/quickaction/ReferenceQuickActionJson.kt` | 19 | QuickActionJsonConfig | `586890ba1` | none | Retire, with the app family |
| theme | `fb/ime/theme/ReferenceFlorisImeThemeBaseStyle.kt` | 333 | FlorisImeThemeBaseStyle (89019851f) | `89019851f` | ThemeMatchTest | Retire |
| theme | `fb/ime/theme/ReferenceThemeManager.kt` | 103 | ThemeManager (phase 3 item 3) | `4b48cd624` | ThemeMatchTest, ThemeTrace | Retire |
| lib | `fb/lib/crashutility/ReferenceCrash.kt` | 71 | CrashUtility's handler and report (b1cf45f18) | `7e54619a4` | CrashTraceTest | Retire |
| lib | `fb/lib/ext/ReferenceExtensionPolicy.kt` | 71 | extension import and archive policy (091941512) | `091941512` | ExtensionArchiveMatchTest, ExtensionMetadataMatchTest, SettingsExtensionImportMatchTest | Retire |
| lib | `fb/lib/ext/ReferenceExtensionValidation.kt` | 166 | ExtensionValidation (091941512) | `091941512` | ExtensionMetadataMatchTest | Retire |
| lib | `fb/lib/ext/replay/ReferenceExtensionMaintainer.kt` | 58 | ExtensionMaintainer (091941512) | `091941512` | ExtensionDeviceReplay, ExtensionMaintainerMatchTest | Keep: compiled into androidTest for a device comparison |
| lib | `fb/lib/io/ReferenceZipUtils.kt` | 136 | ZipUtils (1d2cbc1f6) | `091941512` | BackupArchiveMatchTest, BackupFilesTest, ExtensionArchiveMatchTest, SettingsBackupFlowTest | Retire |
| lib | `fb/lib/reference/ReferenceFlorisLocale.kt` | 140 | FlorisLocale (1d2cbc1f6) | `1d2cbc1f6` | A1Replay, AT:SettingsOtherThemeDevice, ComposingComparison, EmojiInputs, GifComparison, GiphyComparison, GlideRankerTest, JavaTextComparison, JavaTextTest, KeySelectorsMatchTest, LatinComparison, LayoutFixtures, LayoutSnapshot, NlpPureMatchTest, RendererAccessibilityMatchTest, SettingsOtherThemeMatchTest, SettingsThemeEditorColourMatchTest, StoreComparison, SuggestComparison, SyntheticLayouts, TypingTrace, UtilityRulesMatchTest | Retire |
| lib | `fb/lib/reference/ReferenceFlorisRef.kt` | 155 | FlorisRef (1d2cbc1f6) | `1d2cbc1f6` | none | Retire: unused, no test reads it |
| lib | `fb/lib/reference/ReferenceUtilityRules.kt` | 27 | utility rules (1d2cbc1f6) | `1d2cbc1f6` | UtilityRulesMatchTest | Retire |
| lib | `fb/lib/reference/ReferenceVersionName.kt` | 69 | VersionName (1d2cbc1f6) | `1d2cbc1f6` | UtilityRulesMatchTest | Retire |
| lib | `lib/kotlin: ReferenceMimeTypeFilter.kt` | 77 | MimeTypeFilter (07c8bef83) | `1d2cbc1f6` | MimePolicyMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferenceClipboard.kt` | 38 | ? | `07c8bef83` | ClipboardMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferenceGlideTrail.kt` | 29 | ? | `d7b21e291` | GlideTrailMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferenceMedia.kt` | 65 | ? | `ea735d32a` | MediaPresentationMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferenceNlp.kt` | 192 | ? | `9c2b99943` | NlpHarnessMatchTest, NlpMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferenceNlpComposing.kt` | 7 | ? | `9c2b99943` | NlpPureMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferencePanelRules.kt` | 90 | ? | `b089e7711` | PanelRulesMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferenceRendererSmartbarTouch.kt` | 44 | ? | `743bdb248` | RendererSmartbarTouchMatchTest | Retire |
| rust boundary | `lib/rust/src/test/kotlin/aw/rust/ReferenceSmartbar.kt` | 138 | ? | `2acffcd62` | SmartbarMatchTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggAttributes.kt` | 180 | Snygg's SnyggAttributes | `938765749` | SnyggEditorMatchTest, SnyggGrammarMatchTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggPropertyCodec.kt` | 41 | Snygg's SnyggPropertyCodec | `9f917ec17` | SnyggValueCorpusTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggPropertySetEditor.kt` | 333 | Snygg's SnyggPropertySetEditor | `6b09d0ca9` | SnyggEditorMatchTest | Keep: in flight, the Compose-free render path |
| snygg | `lib/snygg: ReferenceSnyggRule.kt` | 137 | Snygg's SnyggRule | `938765749` | SnyggEditorMatchTest, SnyggGrammarMatchTest, SnyggMatchSupport | Retire |
| snygg | `lib/snygg: ReferenceSnyggSpec.kt` | 149 | Snygg's SnyggSpec | `6b09d0ca9` | SnyggEditorMatchTest | Keep: in flight, the Compose-free render path |
| snygg | `lib/snygg: ReferenceSnyggStylesheet.kt` | 75 | Snygg's SnyggStylesheet | `938765749` | SnyggCompilationMatchTest, SnyggEditorMatchTest, SnyggGrammarMatchTest, SnyggMatchSupport | Retire |
| snygg | `lib/snygg: ReferenceSnyggStylesheetEditor.kt` | 50 | Snygg's SnyggStylesheetEditor | `6b09d0ca9` | SnyggEditorMatchTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggTheme.kt` | 137 | Snygg's SnyggTheme | `6b09d0ca9` | InlineChipMatchTest, RenderThemeMatchTest, SnyggCompilationMatchTest, SnyggThemeTest | Keep: in flight, the Compose-free render path (palette) |
| snygg | `lib/snygg: ReferenceSnyggValueBridge.kt` | 64 | Snygg's SnyggValueBridge | `9f917ec17` | SnyggCompilationMatchTest, SnyggGrammarMatchTest, SnyggMatchSupport, SnyggValueCorpusTest, SnyggValueEdgesMatchTest, SnyggValueMatchTest | Keep: in flight, the Compose-free render path |
| snygg | `lib/snygg: reference/ReferenceSnyggAppearanceValue.kt` | 160 | Snygg's SnyggAppearanceValue | `9f917ec17` | SnyggValueEdgesMatchTest | Keep: in flight, the Compose-free render path (palette) |
| snygg | `lib/snygg: reference/ReferenceSnyggFontValue.kt` | 90 | Snygg's SnyggFontValue | `9f917ec17` | SnyggValueEdgesMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggIdToValueMap.kt` | 26 | Snygg's SnyggIdToValueMap | `9f917ec17` | SnyggValueMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggPaddingValue.kt` | 78 | Snygg's SnyggPaddingValue | `9f917ec17` | SnyggValueEdgesMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggShapeValue.kt` | 236 | Snygg's SnyggShapeValue | `9f917ec17` | SnyggShapeValueTest, SnyggValueMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggSizeValue.kt` | 69 | Snygg's SnyggSizeValue | `9f917ec17` | SnyggValueMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggTextValue.kt` | 77 | Snygg's SnyggTextValue | `9f917ec17` | SnyggValueMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggUriValue.kt` | 55 | Snygg's SnyggUriValue | `9f917ec17` | SnyggValueEdgesMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggValue.kt` | 75 | Snygg's SnyggValue | `9f917ec17` | none | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggValueEncoder.kt` | 37 | Snygg's SnyggValueEncoder | `9f917ec17` | none | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggValueSpec.kt` | 243 | Snygg's SnyggValueSpec | `9f917ec17` | SnyggValueMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggVarValue.kt` | 27 | Snygg's SnyggVarValue | `9f917ec17` | SnyggValueEdgesMatchTest | Retire |
| render | `aw/render/ReferenceAccessibilityLabels.kt` | 49 | Compose QuickAction display names and LatinIME key descriptions | `7922905bb` | RendererAccessibilityMatchTest | Keep: in flight, the Compose-free render path |
| render | `aw/render/ReferenceKeyGraph.kt` | 287 | main's key evaluator and key graph (phase 3 item 2) | `9ac3d23ea` | KeyGraphTrace, KeyGraphTraceInputs, NativeEditorDriver, SmartbarActionsMatchTest, SubtypeTraceGraphs | Keep: in flight, the Compose-free render path |
| render | `aw/render/ReferenceRendererModels.kt` | 63 | TextKeyboardLayoutController's models (7c0b8183) | `2dddd25d7` | none | Keep: in flight, the Compose-free render path |
| render | `aw/render/ReferenceRendererSwipe.kt` | 225 | TextKeyboardLayoutController's swipe handling (7c0b8183) | `2dddd25d7` | none | Keep: in flight, the Compose-free render path |
| render | `aw/render/ReferenceRendererTouch.kt` | 142 | TextKeyboardLayoutController (7c0b8183) | `2dddd25d7` | RendererTouchMatchTest | Keep: in flight, the Compose-free render path |
| render | `aw/render/ReferenceRendererVelocity.kt` | 50 | Android 16's VelocityTracker LSQ2 | `2dddd25d7` | none | Keep: in flight, the Compose-free render path |
| render | `aw/render/ReferenceSmartbarActions.kt` | 64 | main's smartbar action choices (phase 3 item 1) | `fa8d43c7c` | SmartbarActionsMatchTest | Keep: in flight, the Compose-free render path |
| render | `aw/render/ReferenceSubtypeBuildRow.kt` | 31 | RustKeyGraphs.build (61f1513fb) | `0a9fbaaf4` | SubtypeMatchTest, SubtypeTraceSides | Keep: in flight, the Compose-free render path |


## What happened

Three branches so far, each stopped at a green point between families. **100 references are
retired.**

* **`all-rust/phase6-kotlin`: dictation, reader and gif.** It retired 30: the 23 `app/src/test`
  copies in dictation (10), reader (9) and gif (4), and the **7 that no test read**
  (`368849831`), one of them under `androidTest`. The 31st file deleted is `ReleasedOnceTest`,
  which tested only its copy.
* **`all-rust/phase6-kotlin-2`: suggest, nlp, editor/input and keyboard.** It retired 38, in nine
  deletion commits: suggest 7, nlp 3, editor/input 16 and keyboard 12. The keyboard went in four
  parts (its pure models, the text keyboard, the glide classifier and the user dictionary),
  each with its own generator, cases and deletion.
* **`all-rust/phase6-kotlin-3`: app, lib, rust boundary and three settings copies.** It retired
  32, in five deletion commits: app 13 (with `SettingsSubtypeMatchTest`'s inline session copy), lib
  8, rust boundary 8, settingsui's drag and scrollbar copies, and its licence screen copy: the three
  of that family with nothing to do with colour, palette, Snygg values or the theme editor. Its 32
  are the inventory's 58 "Retire" verdicts in those families less the 26 below.
* **Kept:** the 19 device comparisons, the 8 render references and the 5 they build on, Snygg's
  5, and 3 more the second branch kept: `ReferenceKeyboardSubtypes`, `ReferencePlatform` and
  `ReferenceEditorRange` (with `ReferenceCaps.java`, which the inventory did not count).
  [Kept, and why](#kept-and-why) lists them.
* **Held for the render path (26):** settingsui's other 7, theme's 2, snygg's 16 and app's
  `ReferenceColorSerializer`. The render-path branch moves the palette maths and touches Snygg and
  the theme editor, so they wait for it to land. Their verdicts above are still the plan;
  [Kept, and why](#kept-and-why) says why each waits.

### The method, as built

* **`GoldenTape`** (`lib/rust/src/testFixtures`, `074e00b53`) records the answers a differential
  test compared, as the text it compared. It chains them into an FNV-1a 64 digest, with
  checkpoints at each power of two, and checks them against `golden/<name>.txt` on the test
  classpath. A golden file holds the count, the final digest and the checkpoints, never an
  answer, so no fixture text reaches a file or a log. A failure names the first range of answers
  that differs. It is kotlin.test's `AssertionFailedError`, which the mutation scripts count as
  caught. `GoldenTapeTest` checks the digest against one worked out apart, in Python.
* **A generator commit per family.** Each test still asserts Rust == reference, and also records
  the reference's answers on `GoldenTape.ofReference`. The goldens are written there with
  `REFERENCE_GOLDEN=write` and the run checked twice without it. A tape of the code under test
  refuses to write, so a regression cannot be recorded as the reference.
* **A deletion commit per family.** The tests record Rust's answers on `GoldenTape.check`, the
  same text in the same order, and the references go, with any helper they leave unused.
* **`GoldenTape.answer`** (`c39145c5f`) records a value beside the question that gave it, for the
  tests whose answers are not text already. The value's `toString()` must be its content, so a
  data class or a list of them records, and `PopupSet`, which prints no content, is recorded by its
  keys.
* **Proof.** `scripts/phase6-mutations.py` runs a family's faults (`scripts/phase6_mutation_cases/`)
  against the JVM tests that compared with the copies. A case counts as caught only by an
  `AssertionFailedError` in a fresh report, so a mutant that does not compile is never caught,
  and the restored source must pass. The scripts that
  already covered the same code ran beside it. Every script ran at `0500aa74f`, with the case
  modules copied in, and again at the family's deletion commit. The second and third branches ran
  each at the family's cases commit, where the references still are, and at its deletion commit. The
  goldens must catch every case the references caught. Each run is a batch that ends well inside
  the two-hour limit on a background command, so no worktree is left with a fault in it.

### Goldens

There are 178: 150 under `app/src/test/resources/golden`, 27 under `lib/rust`'s and 1 under
`lib/kotlin`'s. They hold 95,512 bytes for 48,336,545 answers, the largest file 798 bytes. The first
branch wrote 40 (22,298 bytes), the second 73 (42,464 bytes), the third 65 (30,694 bytes).
The strip-tap fix (`fe49b7f86`) rewrote two of the nlp family's from the Rust after review, 56
bytes longer in all, with the same counts.

| Family | Generator | Files | Bytes | Answers | Tests |
|---|---|---:|---:|---:|---|
| dictation | `c8dfa5719` | 9 | 5,191 | 2,007,199 | DictationRulesMatchTest, DictationSessionMatchTest, MicKeyMatchTest, MicTraceTest, ModelCheckTraceTest |
| reader | `0fa825bff` | 20 | 10,405 | 1,204,691 | ReaderRulesMatchTest, ScreenRulesMatchTest, TtsChunkMatchTest, PlaybackQueueMatchTest, ScreenTraversalMatchTest, LoadedVoiceTraceTest, ReadingTraceTest |
| gif | `527ff3242` | 11 | 6,702 | 3,197,785 | GifRulesMatchTest, GiphyRulesMatchTest, GifSearchTraceTest |
| suggest | `a142c4cab` | 22 | 13,427 | 21,944,586 | SuggestRulesMatchTest, LatinRulesMatchTest, WordStoresMatchTest, SwipeRulesMatchTest, BoundaryGlideMatchTest, BoundaryLearnerMatchTest, NlpLearningMatchTest |
| nlp | `e2dc488cf`, `fe49b7f86` | 3 | 2,093 | 1,200,000 | TypingTraceMatchTest |
| input policy | `8f7677bf2` | 6 | 3,352 | 272,587 | InputControlMatchTest, InputDeleteMatchTest, InputPolicyMatchTest |
| editor pair | `448e43677` | 16 | 8,298 | 3,495,571 | EditorPlanMatchTest, EditorFlowMatchTest, EditorBoundaryTest, NlpEditorHandoffMatchTest, SpacingMatchTest, InputDispatchMatchTest, InputFixtureMatchTest |
| keyboard models | `ccf820d33` | 17 | 11,022 | 9,807,325 | ShiftMatchTest, ComposingMatchTest, EmojiMatchTest, KeyModelsMatchTest, KeySelectorsMatchTest, LayoutAssemblyMatchTest |
| text keyboard | `a652f64cc` | 3 | 2,074 | 2,854,694 | TextKeyboardMatchTest, LayoutAssemblyMatchTest's two layout tests |
| glide classifier | `2c76c1088` | 3 | 1,148 | 522 | SwipeIndexTest's three ranking tests |
| user dictionary | `c050132b4` | 3 | 1,106 | 5,021 | SettingsWordsMatchTest, SettingsWordsTest |
| app | `153d569da` | 18 | 8,726 | 227,362 | SettingsMigrationMatchTest, SettingsCodecMatchTest, SettingsNumericMatchTest, SettingsDefaultsMatchTest, SettingsStoredFileMatchTest, SettingsUiStoredFileMatchTest, SettingsSubtypeMatchTest, SettingsLanguagesMatchTest, BackupPlanMatchTest, BackupArchiveMatchTest, BackupFilesTest, SettingsBackupFlowTest |
| lib | `e34b159e2` | 15 | 7,676 | 1,408,237 | UtilityRulesMatchTest, CrashTraceTest, ExtensionMetadataMatchTest, ExtensionArchiveMatchTest, SettingsExtensionImportMatchTest, BackupArchiveMatchTest, BackupFilesTest; lib/kotlin's MimePolicyMatchTest |
| rust boundary | `c9e59485d` | 27 | 12,371 | 709,341 | lib/rust's ClipboardMatchTest, GlideTrailMatchTest, MediaPresentationMatchTest, NlpPureMatchTest, PanelRulesMatchTest, RendererSmartbarTouchMatchTest, SmartbarMatchTest, NlpMatchTest, NlpHarnessMatchTest |
| settingsui: drag, scrollbar | `428ce96d1` | 2 | 811 | 643 | SettingsDragMatchTest, SettingsScrollbarMatchTest |
| settingsui: licences | `5876fe9f7` | 3 | 1,110 | 981 | SettingsLicencesMatchTest |

**To write a family's goldens again**, check out its generator commit, where the references
still are, and run its tests with the variable set. Then run them twice without it, and copy the
files to the branch:

```sh
git switch --detach c8dfa5719     # or 0fa825bff, 527ff3242
export ANDROID_HOME=$(sed -n 's/^sdk.dir=//p' local.properties)
REFERENCE_GOLDEN=write ./gradlew :app:testDebugUnitTest --rerun \
  --tests '*DictationRulesMatchTest' --tests '*DictationSessionMatchTest' \
  --tests '*MicKeyMatchTest' --tests '*MicTraceTest' --tests '*ModelCheckTraceTest'
```

Name only the family's tests: any other tape in the same run refuses the variable. The second
branch's commands, each at its generator commit:

```sh
# a142c4cab, suggest
--tests '*SuggestRulesMatchTest' --tests '*LatinRulesMatchTest' --tests '*WordStoresMatchTest' \
  --tests '*SwipeRulesMatchTest' --tests '*BoundaryGlideMatchTest' --tests '*BoundaryLearnerMatchTest' \
  --tests '*NlpLearningMatchTest'
# e2dc488cf, nlp: main's answers, two of which fe49b7f86 replaced (below)
--tests '*TypingTraceMatchTest'
# 8f7677bf2, input policy
--tests '*InputControlMatchTest' --tests '*InputDeleteMatchTest' --tests '*InputPolicyMatchTest'
# 448e43677, editor pair
--tests '*EditorPlanMatchTest' --tests '*EditorFlowMatchTest' --tests '*EditorBoundaryTest' \
  --tests '*NlpEditorHandoffMatchTest' --tests '*SpacingMatchTest' --tests '*InputDispatchMatchTest' \
  --tests '*InputFixtureMatchTest'
# ccf820d33, keyboard models
--tests '*ShiftMatchTest' --tests '*ComposingMatchTest' --tests '*EmojiMatchTest' --tests '*KeyModelsMatchTest' \
  --tests '*KeySelectorsMatchTest' --tests '*LayoutAssemblyMatchTest'
# a652f64cc, text keyboard (the cache test is checked by then, so only the two layout tests)
--tests '*TextKeyboardMatchTest' \
  --tests '*LayoutAssemblyMatchTest.everyBundledLayoutModeAndRetainedPresetMatches' \
  --tests '*LayoutAssemblyMatchTest.seededMissingReferencesEmptyRowsSelectorsAndHintSettingsMatch'
# 2c76c1088, glide classifier
--tests '*SwipeIndexTest.theDictionarysWordsRankAsBefore' --tests '*SwipeIndexTest.oddWordsRankAsBefore' \
  --tests '*SwipeIndexTest.strangeSwipesRankAsBefore'
# c050132b4, user dictionary
--tests '*SettingsWordsMatchTest' --tests '*SettingsWordsTest'
```

Each follows `REFERENCE_GOLDEN=write ./gradlew :app:testDebugUnitTest --rerun`.

The third branch's are whole commands, as two of them run other modules' tests. The lib family
names three of the backup tests alone: their other tests hold Rust's tapes, which refuse the
variable.

```sh
# 153d569da, app
REFERENCE_GOLDEN=write ./gradlew :app:testDebugUnitTest --rerun \
  --tests '*SettingsMigrationMatchTest' --tests '*SettingsCodecMatchTest' --tests '*SettingsNumericMatchTest' \
  --tests '*SettingsDefaultsMatchTest' --tests '*SettingsStoredFileMatchTest' --tests '*SettingsSubtypeMatchTest' \
  --tests '*BackupPlanMatchTest' --tests '*BackupArchiveMatchTest' --tests '*BackupFilesTest' \
  --tests '*SettingsBackupFlowTest' --tests '*SettingsLanguagesMatchTest' --tests '*SettingsUiStoredFileMatchTest'
# e34b159e2, lib
REFERENCE_GOLDEN=write ./gradlew :app:testDebugUnitTest --rerun \
  --tests '*UtilityRulesMatchTest' --tests '*CrashTraceTest' --tests '*ExtensionArchiveMatchTest' \
  --tests '*ExtensionMetadataMatchTest' --tests '*SettingsExtensionImportMatchTest' \
  --tests '*BackupArchiveMatchTest.everyArchiveExtractsAsTheLegacyExtractionDid' \
  --tests '*BackupFilesTest.aBackupIsTheArchiveComposeMadeUnderTheNameItGave' \
  --tests '*BackupFilesTest.everyComposeBackupUnpacksToTheTreeComposeUnpacked' \
  :lib:kotlin:test --rerun --tests '*MimePolicyMatchTest'
# c9e59485d, rust boundary
REFERENCE_GOLDEN=write ./gradlew :lib:rust:testDebugUnitTest --rerun \
  --tests '*ClipboardMatchTest' --tests '*GlideTrailMatchTest' --tests '*MediaPresentationMatchTest' \
  --tests '*NlpPureMatchTest' --tests '*PanelRulesMatchTest' --tests '*RendererSmartbarTouchMatchTest' \
  --tests '*SmartbarMatchTest' --tests '*NlpMatchTest' --tests '*NlpHarnessMatchTest'
# 428ce96d1, settingsui's drag and scrollbar
REFERENCE_GOLDEN=write ./gradlew :app:testDebugUnitTest --rerun \
  --tests '*SettingsScrollbarMatchTest' --tests '*SettingsDragMatchTest'
# 5876fe9f7, settingsui's licence screen
REFERENCE_GOLDEN=write ./gradlew :app:testDebugUnitTest --rerun --tests '*SettingsLicencesMatchTest'
```

The nlp family's goldens are no longer all main's answers. `fe49b7f86` fixed the stale strip tap
(below), which changes two of the three traces, and no reference is left to write them from.
They were written from the Rust at that commit after review: for one run with the variable set,
`TypingTraceMatchTest` made its tape with `GoldenTape.ofReference("typing-trace-$seed")` and
finished it after the trace, in place of `GoldenTape.check`, and was then put back. Write them
again the same way.

Three tests needed repeatable inputs before a digest could hold them:

* `ModelCheckTraceTest` sets each written file's time from a counter, not the clock. It records
  keys with its temporary folder written as `<dir>`.
* `ReadingTraceTest`'s fake tells a message by its name, not its resource id, which moves
  whenever a string is added.
* `GifSearchTraceTest` already set every file it writes to the trace's own clock.

`MicTraceTest` holds Rust to main's answer, except for the WAV files main looped on or crashed
over. For those it holds Rust to the intended refusal, and the golden records that refusal.

### Proof

Each script ran at `0500aa74f` (the references) and at the family's deletion commit (the goldens),
before the branch was rebased onto `29cb06dcd`. That rebase brought the plain-View root, which
changes no dictation, reader or gif code or test and no file a case edits.
"Caught" is the script's own verdict.

| Family | Script | Before | After |
|---|---|---|---|
| dictation | `phase6-mutations.py --family dictation` | 16 of 16 | 16 of 16 |
| dictation | `dictation-mutations.py state` | 2 of 2 | 2 of 2 |
| dictation | `edges-mutations.py --part dictation` | 21 of 21 | 21 of 21, by the same targets |
| dictation, reader | `wire-mutation.py renumbered-followed` | builds and passes | builds and passes |
| reader | `phase6-mutations.py --family reader` | 29 of 33 | the same 29 of 33 |
| reader | `tts-mutations.py tts` | 3 of 3 | 3 of 3 |
| reader | `edges-mutations.py --part reading` | 33 of 33 | 33 of 33, by the same targets |
| reader | `edges-mutations.py --part voice` | 16 of 16 | 16 of 16, by the same targets |
| gif | `phase6-mutations.py --family gif` | 50 of 50 | 50 of 50 |
| gif | `edges-mutations.py --part gif` | 34 of 34 | 34 of 34, by the same targets |

The wire case is a renumbering that both sides follow, so it must pass at both ends: that shows
the goldens hold answers, not wire codes.

Four reader cases survive at both ends, so the goldens lose none:

* `cut-at-a-leading-space` and `area-saturates` are equivalent mutants. `split_long` trims what
  is left of a sentence before it looks for a space, so the space is never at index 0. A visible
  area is the product of two `i32` differences, which always fits in an `i64`, so wrapping and
  saturating multiplication agree.
* `middle-dot-kept` and `start-moves` are gaps in the seeded inputs, which the reference had too
  ([owed](owed-checks.md#not-device-checks-but-still-owed)). No seeded page has a `·`, and
  `PlaybackQueueMatchTest` never ends a block ahead of the head.

`edges-mutations.py --part gif` ran into the two-hour limit on a background command at both
ends. Each stopped run's source was restored, and the cases it had not finished ran again by
name, at the same commit; the restored source passed after every part. The counts and targets
above join the parts at each end.

The second branch ran each family's cases at its cases commit, where the references still are,
and at its deletion commit. Main had not moved when the branch was gated, so these are the commits
that ship. Both ends of a row ran the same cases.

| Family | Script | Before | After |
|---|---|---|---|
| suggest | `phase6-mutations.py --family suggest` (`d20a65e7a`, `255f3324b`) | 38 of 46 | 39 of 46 |
| suggest | `consolidation-mutations.py learner glide` | 2 of 2 | 2 of 2 |
| nlp | `phase6-mutations.py --family nlp` (`bd7b38e71`, `7a92198bf`) | 19 of 24 | the same 19 of 24 |
| input policy | `phase6-mutations.py --family input` (`afdf35bfb`, `a210c3be7`) | 37 of 39 | the same 37 of 39 |
| editor pair | `phase6-mutations.py --family editor` (`c2788a34a`, `23853d0ab`) | 36 of 41 | the same 36 of 41 |
| editor pair | `editor-mutations.py` (`c2788a34a`, `23853d0ab`) | 5 of 5 | 5 of 5 |
| editor pair | `input-mutations.py` (`c2788a34a`, `23853d0ab`) | 4 of 4 | 4 of 4 |
| keyboard models | `phase6-mutations.py --family keyboard`, 43 cases (`9797094d9`, `790c40672`) | 41 of 43 | the same 41 of 43 |
| text keyboard | the same, 16 cases (`0051bd697`, `6917e9829`) | 16 of 16 | 16 of 16 |
| glide | the same, 11 cases (`f86bd326a`, `6917e9829`) | 11 of 11 | 11 of 11 |
| user dictionary | the same, 21 cases (`19f318c40`, `7578ffdbe`) | 21 of 21 | 21 of 21 |

`habit-weight-at-the-boundary` is the one case caught only after: the golden is stronger than
its reference ([what was found](#what-moved-and-what-was-found)). `editor-mutations.py` had
stopped at its first case since `5725d2fed`; `3f23e8d5a` points its five cases at where their
lines are now, and it ran in that form at both ends. `input-mutations.py` faults the dispatcher's
lifecycle, the routing and the pointer policy. It ran at the editor pair's two commits, which
retired the dispatcher's and routing's copies; the pointer policy's copy had gone with the input
policy, so `rolling-multi-pointer` met goldens at both ends. All four were caught at both.

The strip-tap fix added two nlp cases, run by name at `fe49b7f86` with the rewritten goldens:
`stale-tap-refused` is caught by three of `StripTapTest`'s tests and the trace, and
`stale-tap-by-value` by `StripTapTest`'s replaced-row test and the trace. `14cc68f54` moved
`aim-setting-ignored` and `undone-always-counted` with the code they name, in this family and in
`typing-plan-mutations.py`; run by name there, both are caught in both scripts.

No case the references caught got past the goldens. 21 cases survive at both ends, so the
goldens lose none; the references let them through too:

* **input policy (2):** `done-shows-an-arrow` and `no-enter-action-ignored`. The test draws each
  key's options with `r.nextInt()`, and ENTER gets only 110 draws (11 modes by 10 types). Option
  6 with the flags clear comes up about once in 1,024 draws, and the no-enter difference about
  once in 170, so neither is reached.
* **keyboard (2):** `tld-shows-its-code` is an equivalent mutant: `URI_COMPONENT_TLD` is -255,
  already below `SPACE`, so dropping its own test changes no answer. `digits-in-a-query` is a gap
  in the seeded inputs: a query is drawn from an emoji's letter runs (`[A-Za-z]+`), random
  letters or odd characters, none of them a digit, so a query with a digit is never asked.
* **Not yet classified:** suggest's `zero-score-scored`, `trail-trusted-past-a-deletion`,
  `acronym-ahead-of-itself`, `replaced-at-another-start-in-nlp`, `undone-at-another-start-in-nlp`,
  `crlf-read-as-two-lines` and `ngram-step-at-the-boundary`; nlp's `arm-only-when-accepted`,
  `undo-with-a-selection`, `second-save-lost`, `undo-not-published` and `second-timer-dropped`;
  the editor pair's `select-in-a-raw-editor`, `backspace-word-without-immediate`,
  `unchanged-refresh-refused`, `late-by-one-more` and `stale-hold-accepted`. Each is a gap in
  the seeded inputs or an equivalent mutant, not yet told apart; telling them apart is owed
  ([owed-checks](owed-checks.md#not-device-checks-but-still-owed)).

The third branch ran each family's cases the same way, at its cases commit and at its deletion
commit; main had not moved when the branch was gated. Every case caught at both ends was caught by
the same targets (the script's named groups of test classes).

| Family | Script | Before | After |
|---|---|---|---|
| app | `phase6-mutations.py --family app` (`0ec0bd2f6`, `69b187850`) | 34 of 37 | the same 34 of 37 |
| lib | `phase6-mutations.py --family lib` (`0326b73ee`, `e80392ff8`) | 44 of 47 | the same 44 of 47 |
| lib: MIME filters | `phase6-mutations.py --family lib_mime` (`0326b73ee`, `e80392ff8`) | 6 of 6 | 6 of 6 |
| rust boundary | `phase6-mutations.py --family boundary` (`02b20a875`, `dc2825ad3`) | 56 of 60 | the same 56 of 60 |
| settingsui: drag, scrollbar | `phase6-mutations.py --family settings` (`001ca7d89`, `73511210c`) | 17 of 21 | the same 17 of 21 |
| settingsui: licences | `phase6-mutations.py --family settings_licences` (`56ceaa541`, `088772046`) | 20 of 20 | 20 of 20 |

`int-range-unchecked` first came out uncaught before: the Gradle daemon went away during its run,
and its report held no assertion failure at all. Run again by name at `0ec0bd2f6`, it is caught by
`SettingsCodecMatchTest`, as it is after.

No case the references caught got past the goldens. 14 cases are not counted at either end:

* **Caught, but not by an assertion (1):** app's `time-dot-refused`. Refusing a stored time whose
  ninth character is a dot stops `FlorisPreferenceModelImpl` from being built, so
  `SettingsDefaultsMatchTest` fails in its constructor, with an `IllegalStateException`, at both
  ends. The script counts only assertion failures.
* **Gaps in the seeded inputs (13)**, which the references had too
  ([owed](owed-checks.md#not-device-checks-but-still-owed)):
  * **app (2).** `history-unknown-field-allowed`: the only seeded history with an unknown field is
    `{"pinned":[],"recent":[],"unknown":1}`, whose empty lists read as the same empty history the
    refusal gives. `vendor-prefix-shortened`: no package offered starts with `dev.patrickgold`
    without starting with `dev.patrickgold.florisboard` (the six are empty, a space, a no-break
    space, the two FlorisBoard ids and the app's own).
  * **lib (3).** `theme-label-untrimmed`: a seeded label is one part, so it is all whitespace
    (refused as blank) or has nothing to trim. `relative-path-keeps-slash`: of the 10,040 values
    the reference-facts loop draws its paths from, only `//a/b` starts with a slash, and its
    10,000 draws do not take it. `extra-value-ignored`: both sides of a seeded comparison share
    one extra value, so comparing the extras gives 0 whenever both have one, as the mutant does.
  * **rust boundary (4).** `mime-wildcard-without-slash`: no concrete type starts with a
    wildcard's top-level type and goes on without a slash, as `textual/plain` would for
    `text/*`. `nlp-uri-is-prose`, which despite its name moves the web e-mail variation (0xd0) to
    the phonetic one (0xc0): the NLP test's eleven input types (1, 0x81, 0x91, 0xe1, 0x12, 0x21,
    0x11, 0x80001, 0x8021, 0x8011 and 0x80b1) have neither. `media-min-delay-1ms` and
    `media-visible-not-touched`: an upload is accepted only at an entry's next frame index, which
    the random traces draw from 0 to 65, so a run of 200 operations keeps about one first frame
    and rarely a second, and the scripted tail uses one id and delays of 100 and 200 ms. Neither
    change moves an answer: the trace reports each step's frame, earliest deadline, bytes and
    entry count, not which entry an eviction took.
  * **settingsui (4).** `history-only-last-sample`, `ring-of-nineteen`, `up-at-40-ms-stops` and
    `maximum-halved`: the gestures that would show them start or end at the bottom edge.
    "Batched history" and "up exactly 40 ms" start there, and the long drag and "samples sharing
    a millisecond" end there. The halved maximum, 10,500 px/s, would cap only a fling from the
    end, or the reversal, which reaches the top either way.

### Lines deleted

| Commit | What | Files deleted | Counted lines |
|---|---|---:|---:|
| `368849831` | the seven no test read | 7 | −713 |
| `c2606ae7a` | dictation | 10 | −742, +14 |
| `1dfaa30f7` | reader | 10 | −672, +2 |
| `4c7db78a2` | gif | 4 | −616 |
| `255f3324b` | suggest | 7 | −1,131, +6 |
| `7a92198bf` | nlp: the typing fan-out | 1 | −254 |
| `b484b7dcc` | nlp: the manager and Latin provider no test read | 2 | −669 |
| `a210c3be7` | input policy | 5 | −161 |
| `23853d0ab` | editor pair | 11 | −1,510, +65 |
| `790c40672` | keyboard models | 7 | −669 |
| `0e12ccfa7` | text keyboard | 3 | −436, +2 |
| `6917e9829` | glide classifier | 1 | −403 |
| `7578ffdbe` | user dictionary | 1 | −166, +25 |
| `69b187850` | app | 13 | −1,341, +10 |
| `e80392ff8` | lib | 8 | −845, +17 |
| `dc2825ad3` | rust boundary | 8 | −663 |
| `73511210c` | settingsui: drag and scrollbar | 2 | −100 |
| `088772046` | settingsui: licences | 1 | −74 |

Across the first branch the Kotlin is −2,639 and +206 counted lines (`GoldenTape` and its test
are 176 of the added). The mutation scripts and cases add 325 lines of Python, and cargo gains
the four tests that moved. Across the second the Kotlin is −5,136 and +143 counted lines (net
−4,993), the cases add 589 lines of Python, and cargo is unchanged. A deletion's added lines are
helpers that outlive their reference: `SpacingAdapter` (renamed from `ReferenceSpacingAdapter`),
`SwipeWordsFixtures` and `RecordingUserDictionaryDao`.

Across the third branch the Kotlin is −2,769 and +138 counted lines (net −2,631); the deleted
files alone are 2,720 of them (app 1,208, lib 757, boundary 603, settingsui 152). The cases add 432
lines of Python, and `phase6-mutations.py` takes a family's `TASK`, for the plain JVM module
`lib/kotlin`, whose tests run as `test` (+9, −6). The build loses org.json and TagSoup: four lines
of `app/build.gradle.kts` and four of the version catalogue. Cargo is unchanged.

### What moved, and what was found

* **Into cargo.** `AudioHistoryTest`'s three tests of its copy alone became tests of the Rust
  history they stood for (`dictation::segmentation`). `ReleasedOnceTest` tested only its copy, since
  production's `ReleasedOnce` went in `a4e630fd6`. Its repeated-close case is now
  `reader::generator`'s `close_frees_once_however_often_it_is_called`; use after close and use in
  progress were already covered there. Several adapter tests (`PhrasesTest`, `SpokenTest`,
  `MicKeyTest`, `FollowAlongTest`, `ScreenRulesTest`) keep every case and call the bindings
  (`DictationRules`, `LegacyDictation`, `MicKeyState`, `ReaderRules`, `ScreenWalkRules`), where
  they called the copies of the Kotlin adapters over them.
* **What went with the references:** `FakeNet` and `FakeConnection` in `GifHarness.kt`, which
  only the reference's fetches used. `GiphyFormatException`, `ReferenceReadAloud` and
  `ReferenceWindow` were declared in the reference files, and the tests that called them for the
  reference's side now record only the Rust's answers. Also the JDK 22 `URLEncoder` workaround, which only the reference's URLs
  needed. The goldens hold the URLs the Rust
  builds, which are Android's and JDK 21's.
* **A stale mutation case.** `dictation-mutations.py`'s `late-result` had matched nothing since
  `9a269ccb2`, when the session started matching `events::FINAL` rather than the string
  `"final"`. State mode stopped at that case before running it, at both ends. `7c7dff12f` points
  it at the match arm, and the table's state counts are from that version, at both ends. It is
  the second such case: the owed `--check` mode in
  [owed-checks](owed-checks.md#not-device-checks-but-still-owed) would have caught it.
* **Not a reference file.** `lib/rust`'s `DictationSegmentationMatchTest.kt` holds a private
  `ReferenceSegmentation` class. It is not one of the 166 files and was left alone.

The second branch:

* **Two goldens are stronger than their references.** `ReferenceBoundaryGlideRanker` asked Rust's
  `SwipeRules` for its bonuses and order, so a fault in `glide_ranker.rs` moved both sides alike;
  the golden was written before any fault, and catches `habit-weight-at-the-boundary`, which the
  reference let through. `LayoutAssemblyMatchTest`'s snapshots built both sides' popups with
  production's `MutablePopupSet` (Rust's popup order) and the same `ReferenceKeyRules`, so only
  the flex and the geometry were compared; the golden pins the popup order as well.
* **A swipe frequency is recorded as the `Float` the glide classifier takes.** Off a phone the
  JVM's `exp` and Rust's differ in the last place of 35,333 of the doubles (18 in frequencies,
  35,315 in word lists), which the comparison already allowed as long as that `Float` agreed. A
  case moves the swipe cost per step by 1e-7 to show the `Float` still tells such a change.
* **A stale tap on the strip, in production since main, now fixed.** The typing trace takes a
  strip tap from the row the keyboard shows. `NlpSessionState`'s row is a `StateFlow`, which keeps
  the old objects when a list equal to them is published, and the session looked a tap up by
  identity among the objects it held, so a tap on a kept row carried no id and Rust refused it.
  Over the three seeds, 238 of 7,489 shows left the row holding old objects, and 11 of the 494 taps
  on the row hit one (8 picks and 3 removals); main's fan-out and the branch both refused all 11.
  * **The fix.** `fe49b7f86` takes a tap's id from its place in the row last published; the row is
    kept only when it equals that list, so the places line up. An object not in the row, as the
    auto-correction, is found by identity as before. Of the 11, 5 now go in (4 picks and 1
    removal). The other 6 stay refused, as is every tap whose own event empties the row: the trace
    configures the session on 7 in 10 editor changes, where production does on each, so a tap's
    event can carry a context the session has not had, and configuring it empties the row first.
    On main's path, 196 of 196 such taps with a found id were refused, and 287 of 287 with a found
    id that did not empty the row went in.
  * **The goldens.** Seed 7's only stale tap is one of the 6, so its golden is unchanged. Seed
    20261122's first tap now taken is event 42,653 and seed 424242's is event 7,931; every event's
    record from there to the end differs (7,347 and 42,069 of 50,000), because the two paths part
    there, and no earlier one does. Those two goldens were rewritten from the Rust after review.
  * **Why by place.** A lookup by value would find the same place on a kept row, since Rust never
    shows two candidates of one kind and text (`Session::filter`). It differs on a tap on an
    object of a row since replaced, when the new row shows the same word: main refused it, and a
    lookup by value would take it. Over the fix's three traces that is 1 of 9,096 taps and none of
    the 1,216 auto-corrections, so the place lookup changes the kept-row taps and nothing else.
    A new list on every publish would have to break `List`'s equality, and the row would be drawn
    again at every publish. Every tap and removal reaches the session through `event()`, so the
    frozen Compose strip and the Rust root, which only show the row, are unchanged.
  * **Proof.** `StripTapTest` fails 3 of its 5 tests on main. Two nlp mutation cases bring the
    refusal back (`stale-tap-refused`) and the lookup by value (`stale-tap-by-value`); both are
    caught (see [proof](#proof)).
  * **On the emulator.** On emulator-5556 the keyboard typed short made-up prefixes into the
    probe's field and tapped the strip, compared by hashes only. On the fix's build, 68 taps
    over six paths all went in: a fresh row; the last letter deleted and typed again, once and
    twice; the field tapped again; the cursor moved into the word; and Delete with the last
    letter as one two-finger tap. Each put the tapped word in and ended the composing region,
    except after the cursor move, where the word before the cursor was replaced. A build with
    main's lookup did the same on 50 of those taps, so none of these paths reached a kept row:
    in each, a different list came between two equal ones, or no equal list followed. A row is
    kept only when a second suggestion pass for the same word completes while the first list is
    still shown. The kept row is shown on the host, by `StripTapTest` and the trace; on a device
    it is owed ([owed-checks](owed-checks.md#emulator-checks-the-docs-owe), E55). Settings and
    learned words went back byte for byte; of the 13 files backed up, only `profileInstalled`
    differs, which Android's profile installer rewrites at each install.
* **Questions asked by name.** Key codes and predefined keys are asked in the order of their
  sorted names, not the order reflection lists fields in, which the JVM does not fix.
* **What stayed as helpers.** `ReferenceSwipeWordsFactory` was no copy of anything: it builds the
  production `SwipeWords` from a word list, and is now `SwipeWordsFixtures`.
  `ReferenceSpacingAdapter` is the spacing tests' way into Rust, now `SpacingAdapter`.
  `RecordingUserDictionaryDao`, which shared the user dictionary reference's file, has its own.
  The learner test's address check and the rules test's base letter stay as questions.
* **The user dictionary's intended differences.** Its frozen export is a literal,
  `frozenExport`, in Room's row order and the provider's. The drafts' outcome check counts refused,
  trimmed and written drafts from the native calls; the reference split the 4,045 refused into
  3,902 invalid and 143 its language had already. A file with a bad line wrote the lines before it
  under Compose, then threw; the native import writes nothing, which is still checked.
* **The per-step operations nothing called.** `14cc68f54` retired the NLP operations `AIM`,
  `TALLY` and `TALLY_CHANGED` from `crates/wire`, the generated boundary (three `NlpOp` entries and
  the `NlpAim`, `NlpTally` and `NlpTallyEvent` objects) and the session. They travel through
  `NlpNative.call`, so no native went: 466 stay, none hand-written. The typing plan's aim check now
  reads its own event row, and an undone correction's count is a session method the plan calls.
  Moving two of `typing-plan-mutations.py`'s cases with that code showed two others there, on the
  render path's `RendererSmartbar.kt`, have matched nothing since `c66e98a04`
  ([owed-checks](owed-checks.md#not-device-checks-but-still-owed)).
* **A third stale mutation script.** `editor-mutations.py` stopped at its first case,
  `invalid-finalize-batch`, at both ends: `5725d2fed` had named the editor's operations from
  `crates/wire`, and `32e62fb4a` had moved the reconciliation check into `Content::reported_by`.
  `3f23e8d5a` points each of its five cases at the same change on today's lines.

The third branch:

* **Exact where the comparisons allowed slack, with one rounded tape.** `SmartbarMatchTest`'s
  first test compared layouts within 0.002, and `SettingsDragMatchTest` flings within .01; Rust
  matched the references bit for bit on all 463,968 Smartbar fields and all 11 flings, so both
  tapes are exact. Floats are recorded with `Float.toString`, which round-trips. The scrollbar's
  alpha is the exception: Rust's cubic solve and Compose's `CubicBezierEasing` agree to 6.6e-7
  but differ in the last place on 48 of 522 frames. Its tape records alpha in ten-thousandths,
  where all 522 agree (at hundred-thousandths 2 would not), 30 times finer than the .003 the
  comparison allowed.
* **What took the reference's place in a check.** The drag test counted flings by the
  reference's velocity, which Rust does not report; it now counts the ups that move the content
  (six of eleven: the 60 ms rest, the 41 ms pause and the hold stop, and "batched history" and
  "up exactly 40 ms" fling forward from the end). The licence test still checks the Apache
  dialog's literal text, now of Rust's row.
* **Kept out of the goldens**, as what would change them without a change to the code under test
  (the lib family): the URLs in `src/main`'s sources, which `UtilityRulesMatchTest` walked for
  reference shapes (26 made-up shapes take their place); the app's MIME literals, which
  `MimePolicyMatchTest` read from `app/src/main` (the 13 it found are listed); the JDK's installed
  locales and their names (28 fixed real tags are recorded without names); Android string
  resource ids (messages record the name); and file listing order (manifests and trees are
  recorded in path order).
* **Tests that only borrowed a reference change mechanically** (the app family). The settings
  screens' defaults, and the held `ReferenceOtherThemeScreens`' theme and time reads, come from
  `FlorisPreferenceModelImpl`; `SettingsGroupOneMatchTest` reads slider bounds from
  `SettingsRules`; `SettingsOtherThemeMatchTest` writes a time as Jetpref's serializer did,
  `LocalTime.toString`; the write-cancellation test shows Jetpref's defect on a two-preference
  plain model; `SubtypeSessionTest` writes its saved JSON with `SettingsCodecs`. The backup tests'
  fixtures, which Compose's `ZipUtils.zip` wrote, are written by a few lines of `java.util.zip`.
* **The NLP pairing became a recording session.** `NlpMatchTest`'s `Pairing` put every call to the
  reference and to Rust side by side; it now records Rust's answer, snapshot and Smartbar words
  after each call, and the context row it was given moves to the test's companion.
  `PanelRulesMatchTest` feeds each history step the state Rust gave the step before.
* **What went with the references.** org.json and TagSoup were test dependencies only for the
  licence copy, so they leave the app's test classpath and the version catalogue.
* **Mutation runner.** `phase6-mutations.py` takes a family's `TASK`, for `lib/kotlin`'s plain JVM
  `test` task; every other family runs `testDebugUnitTest` as before.

### Kept, and why

* **Device comparisons (19):** the 11 under `androidTest` (dictation's 8, reader's
  `ReferenceReader`, `ReferenceReleasedOnce` and `ReferenceVoiceEngine`), and the 8 compiled into
  the instrumented tests from `app/src/test` (`gif/shared`, `suggest/a1`, `lib/ext/replay`). Their
  goldens need a device run ([owed-checks E54](owed-checks.md#emulator-checks-the-docs-owe)).
* **Render (8): kept, in flight, the Compose-free render path.** That branch owns
  `app/src/test/.../render`.
* **What the render references build on (5):** `ReferenceLayoutAssembly`, `ReferencePopupSet`,
  `ReferencePopupGrid`, `ReferenceSwipeGesture` and `ReferenceGlideTypingGesture`.
* **Snygg (5): kept, in flight, the Compose-free render path.** `ReferenceSnyggTheme` and
  `ReferenceSnyggAppearanceValue` guard the palette; the lead kept `ReferenceSnyggPropertySetEditor`,
  `ReferenceSnyggSpec` and `ReferenceSnyggValueBridge` with them, since the render path's branch
  may touch `lib/snygg`'s palette and colour code.
* **`ReferenceKeyboardSubtypes`: kept.** Its trace (`SubtypeTraceSides`' main side, which
  `SubtypeMatchTest` runs) is what drives the render reference `ReferenceSubtypeBuildRow`, kept in
  flight. Retire the two together.
* **The editor drivers' platform: kept.** `ReferencePlatform` holds the ICU breaks and the
  platform facts that `NativeEditorDriver` answers Rust's queries with, and the render path's
  `ReferenceKeyGraph` and `KeyGraphTrace` build on it. `ReferenceEditorRange` is the range type of
  the fake `InputConnection` and of `ReferencePlatform`. `ReferenceCaps.java`, Android's
  `TextUtils.getCapsMode`, answers Rust's caps queries; the inventory counted only `.kt` files.
  They stand in for the platform rather than copying code that moved, so there are no answers of
  theirs to record.
* **Held for the render path (26 files marked "Retire").** The render-path branch moves the palette
  maths and touches Snygg and the theme editor; the third branch took only copies with nothing to
  do with colour, palette, Snygg values or the theme editor, and these wait for it:
  * **settingsui (7).** `ReferenceThemeEditor` is the theme editor's. `ReferenceOtherThemeScreens`
    holds ThemeScreen's colour and theme-mode rows; the app family changed only where it reads its
    defaults. `ReferenceGroupOneScreens` and `ReferenceGroupTwoScreens` share their tests with it
    (the dial plurals and ThemeScreen's routes are asked in the same tests), so they go together.
    `ReferenceExtensionEditor` and `ReferenceExtensionScreens` decide what the theme extension
    editor saves and lists (`ThemeExtensionEditor`, the theme editor's model), and that branch
    changes both screens' Rust test fixtures.
    `ReferenceSettingsScreens` (KeyboardScreen and HomeScreen) has no colour in it, but its
    transaction test is driven by the copy's row model (each row's kind, choices and slider
    bounds); retiring it means a new driver that reads those from Rust's nodes, so it waits for the
    family's turn rather than being rushed in here.
  * **theme (2).** The render-path branch changes `ThemeManager.kt`, `ThemeMatchTest` and
    `ThemeTrace`, which `ReferenceThemeManager` and `ReferenceFlorisImeThemeBaseStyle` are compared
    in.
  * **snygg (16).** Snygg's values, held by rule; the render-path branch changes `SnyggPalette`
    and `SnyggUi`.
  * **app's `ReferenceColorSerializer`.** Jetpref's colour serializer, read by the colour tests
    (`SettingsDerivationMatchTest`, `SettingsOtherThemeMatchTest`).

  Each takes the same steps: a generator commit, its cases, the before and after runs, then the
  deletion commit. A settingsui or theme reference that guards the Material You palette
  (`SettingsThemeEditorPaletteMatchTest` and its kin) stays with the render path's branch.
