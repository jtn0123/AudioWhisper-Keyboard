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

The verdict column is the plan at `0500aa74f`. [What happened](#what-happened) says, family by
family, which were retired and which were left.

* **Unused (7 files).** No test reads them: nothing compares against them. They are deleted
  without goldens, since there is nothing to record.
* **Render (8 files): kept, in flight, phase 3 item 4.** Another branch is replacing the
  Compose root with a plain View, and owns `app/src/test/.../render`.
* **The render references build on 5 more:** `ReferenceLayoutAssembly`, `ReferencePopupSet`,
  `ReferencePopupGrid`, `ReferenceSwipeGesture` and `ReferenceGlideTypingGesture`. They stay
  until the render references go.
* **Device comparisons (19 files): kept.** The 11 under `app/src/androidTest` (the unused
  `ReferenceSpeaker` aside) and the 8 in `app/src/test` directories that `app/build.gradle.kts`
  compiles into the instrumented tests too (`suggest/a1`, `gif/shared`, `lib/ext/replay`). They
  compare on a device (`DictationInstrumentation`, `TtsInstrumentation`, `A1Instrumentation`,
  `RendererFixtureTest`, `ExtensionDeviceReplay`). Golden files for them would need a device run,
  and this work had no device.
* **The rest (127 files): retire.**

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
| editor/input | `fb/ime/editor/ReferenceEditorRange.kt` | 26 | EditorRange (c07983c8d) | `c07983c8d` | EditorBoundaryTest, EditorPlanMatchTest, FakeInputConnection | Retire |
| editor/input | `fb/ime/editor/ReferenceEditorState.kt` | 280 | EditorInstance's state handling (c07983c8d) | `c07983c8d` | none | Retire |
| editor/input | `fb/ime/editor/ReferenceImeOptions.kt` | 41 | ImeOptions (c07983c8d) | `c07983c8d` | EditorPlanMatchTest | Retire |
| editor/input | `fb/ime/editor/ReferenceInputAttributes.kt` | 120 | InputAttributes (c07983c8d) | `c07983c8d` | EditorPlanMatchTest | Retire |
| editor/input | `fb/ime/editor/ReferencePlatform.kt` | 54 | the platform facts both editor sides read | `c07983c8d` | EditorPlanMatchTest, KeyGraphTrace, NativeEditorDriver, NlpMatchTest, SettingsCodecMatchTest | Retire |
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
| keyboard | `fb/ime/core/ReferenceKeyboardSubtypes.kt` | 95 | SubtypeManager (61f1513fb) | `0a9fbaaf4` | SubtypeMatchTest, SubtypeTrace, SubtypeTraceSides | Retire |
| keyboard | `fb/ime/core/ReferenceSubtypeJson.kt` | 3 | the persisted subtype JSON codec | `586890ba1` | SettingsCodecMatchTest, SubtypeSessionTest | Retire |
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
| nlp | `fb/ime/smartbar/quickaction/ReferenceQuickActionJson.kt` | 19 | QuickActionJsonConfig | `586890ba1` | none | Retire |
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
| snygg | `lib/snygg: ReferenceSnyggPropertySetEditor.kt` | 333 | Snygg's SnyggPropertySetEditor | `6b09d0ca9` | SnyggEditorMatchTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggRule.kt` | 137 | Snygg's SnyggRule | `938765749` | SnyggEditorMatchTest, SnyggGrammarMatchTest, SnyggMatchSupport | Retire |
| snygg | `lib/snygg: ReferenceSnyggSpec.kt` | 149 | Snygg's SnyggSpec | `6b09d0ca9` | SnyggEditorMatchTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggStylesheet.kt` | 75 | Snygg's SnyggStylesheet | `938765749` | SnyggCompilationMatchTest, SnyggEditorMatchTest, SnyggGrammarMatchTest, SnyggMatchSupport | Retire |
| snygg | `lib/snygg: ReferenceSnyggStylesheetEditor.kt` | 50 | Snygg's SnyggStylesheetEditor | `6b09d0ca9` | SnyggEditorMatchTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggTheme.kt` | 137 | Snygg's SnyggTheme | `6b09d0ca9` | InlineChipMatchTest, RenderThemeMatchTest, SnyggCompilationMatchTest, SnyggThemeTest | Retire |
| snygg | `lib/snygg: ReferenceSnyggValueBridge.kt` | 64 | Snygg's SnyggValueBridge | `9f917ec17` | SnyggCompilationMatchTest, SnyggGrammarMatchTest, SnyggMatchSupport, SnyggValueCorpusTest, SnyggValueEdgesMatchTest, SnyggValueMatchTest | Retire |
| snygg | `lib/snygg: reference/ReferenceSnyggAppearanceValue.kt` | 160 | Snygg's SnyggAppearanceValue | `9f917ec17` | SnyggValueEdgesMatchTest | Retire |
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
| render | `aw/render/ReferenceAccessibilityLabels.kt` | 49 | Compose QuickAction display names and LatinIME key descriptions | `7922905bb` | RendererAccessibilityMatchTest | Keep: in flight, phase 3 item 4 |
| render | `aw/render/ReferenceKeyGraph.kt` | 287 | main's key evaluator and key graph (phase 3 item 2) | `9ac3d23ea` | KeyGraphTrace, KeyGraphTraceInputs, NativeEditorDriver, SmartbarActionsMatchTest, SubtypeTraceGraphs | Keep: in flight, phase 3 item 4 |
| render | `aw/render/ReferenceRendererModels.kt` | 63 | TextKeyboardLayoutController's models (7c0b8183) | `2dddd25d7` | none | Keep: in flight, phase 3 item 4 |
| render | `aw/render/ReferenceRendererSwipe.kt` | 225 | TextKeyboardLayoutController's swipe handling (7c0b8183) | `2dddd25d7` | none | Keep: in flight, phase 3 item 4 |
| render | `aw/render/ReferenceRendererTouch.kt` | 142 | TextKeyboardLayoutController (7c0b8183) | `2dddd25d7` | RendererTouchMatchTest | Keep: in flight, phase 3 item 4 |
| render | `aw/render/ReferenceRendererVelocity.kt` | 50 | Android 16's VelocityTracker LSQ2 | `2dddd25d7` | none | Keep: in flight, phase 3 item 4 |
| render | `aw/render/ReferenceSmartbarActions.kt` | 64 | main's smartbar action choices (phase 3 item 1) | `fa8d43c7c` | SmartbarActionsMatchTest | Keep: in flight, phase 3 item 4 |
| render | `aw/render/ReferenceSubtypeBuildRow.kt` | 31 | RustKeyGraphs.build (61f1513fb) | `0a9fbaaf4` | SubtypeMatchTest, SubtypeTraceSides | Keep: in flight, phase 3 item 4 |


## What happened

Filled in family by family as each is retired or left.
