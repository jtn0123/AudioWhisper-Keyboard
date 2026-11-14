# Stage 4: Rust settings renderer

## Decision and checkpoint

User decision, **2026-11-14**: Rust will draw the settings Activity using the keyboard's
**Vello CPU / Parley / Skrifa** stack. This supersedes the old “accessible Compose controls
stay” stage-4 boundary. Every settings feature, add-on/extension operation and the theme
editor stays. Compose settings and keyboard remain the default/fallback until physical
phone acceptance. This checkpoint authorizes design and a bounded prototype, not the full port.

Step 0 isolates the test-only native seams; see the final section of
[package 20](stage2-consolidation.md). The implementation baseline is `04186d1d`.
No legacy callback is to be resurrected in the production APK for settings.

## Runtime boundary

Kotlin keeps a thin ComponentActivity, Android lifecycle/ActivityResultRegistry, SurfaceView,
window insets/edge-to-edge and the platform adapters. Reuse the keyboard's generation-owned
ANativeWindow worker, dirty-region comparison, cached Vello pixmap, Choreographer frame
submission, Parley system-font registration and Skrifa glyph path. A settings scene must
not enter keyboard key dispatch, learning, dictation or keyboard panel state. Reuse drawing
and lifetime infrastructure, with independent settings navigation/widget state.

Rust owns the navigation stack and route arguments; screen descriptions derived from the
existing Rust preference catalogue; widget state/layout, list/radio/slider dialogs, colour
picker and theme-editor controls; scrolling/fling, animations, hit testing, hardware focus
order and semantic nodes. It owns editable UTF-16 text, selection and composing ranges.
Do not replace the already-ported defaults, codecs, migrations or validation with UI copies.
Kotlin supplies preference snapshots and executes native write plans through the original
Jetpref typed serializers/setters. The datastore, ordering and file codec remain unchanged.

MotionEvents include pointer identities, history, coordinates, timestamps and cancellation.
Rust distinguishes tap from drag, owns velocity/decay and stops fling on new input. Hardware
key events include Unicode/modifiers/repeat; Tab and d-pad move native focus and activation
uses the same commands as touch and accessibility. Back has begin/progress/cancel/commit
phases. Predictive-back progress is visual only; cancellation must leave stack/dialog/drafts
unchanged. Only commit dismisses a dialog, returns a route, or requests Activity finish.

Kotlin executes typed platform plans: intent/URL, SAF open/create/share, permission request,
IME enable/picker, toast, clipboard and Room/file operations. Plans carry stable request IDs;
results return to Rust and are consumed once. Rust does not perform file IO. Android reads
bytes, Rust parses/serializes, Android writes atomically and reports success/failure. Pending
SAF/permission requests survive Activity recreation through ActivityResultRegistry and saved
request IDs; URI grants and cancellation are platform facts, never guessed native success.

The minimal editable View is an InputConnection endpoint, not the editing model. Its methods
forward commit/composition/deletion/selection/hardware events to Rust, return surrounding
UTF-16 and notify InputMethodManager of native selection/composition. Native layout supplies
its rectangle/cursor; Android supplies the keyboard and IME insets. InputConnection close,
rotation and back must finish/cancel composition explicitly. No text enters diagnostics.

Reuse the keyboard AccessibilityNodeProvider transport with settings roles (button, switch,
radio, seekbar, heading, editable and scroll), checked/selected/enabled state, range metadata,
labels, supported actions, bounds and traversal order supplied by Rust. Unlike a keyboard
key, exploring a settings button must not activate on hover exit. Focus/click, range changes,
scroll, text/selection edits and modal dismissal feed the same native event path. Only the
active modal is exposed while a dialog is open. Hide clipped nodes and preserve stable ids.
TalkBack tree parity and actual action delivery are separate proof obligations.

## Resources, appearance and lifecycle

Android string resources remain the source of truth. Kotlin resolves resource keys, named
placeholder formatting and plurals into a batched string map; Rust chooses resource keys
and lays out the resolved strings. Never fork English labels into the Rust implementation.
`EnumDisplayEntries.kt` is the current enum-label/description specification. This fork ships
English only (`localeFilters += "en"`); app strings are in values/strings.xml,
strings_glide.xml and strings_dont_translate.xml. Keep translation-ready resource IDs and
platform-localized locale names. A full port must test long strings and RTL even though this
APK currently filters translations; bundled dependency resources are not a new Rust dictionary.

Preserve FlorisAppTheme's current neutralDynamicColorScheme, accent preference, system
Material You seed, AUTO/AUTO_AMOLED/LIGHT/DARK/AMOLED_DARK and status/navigation icon mode.
Kotlin resolves colour roles and density/fontScale and passes them as values. Rust uses
semantic Material roles, not Snygg keyboard colours for settings. Theme editor previews
continue using the real keyboard Snygg compiler and draft stylesheet snapshot path.

Use edge-to-edge surface dimensions minus Android system/cutout/IME insets; do not double
subtract navigation insets when Compose is the fallback. Rust reflows labels and grows rows
at fontScale 2.0 rather than clipping to a fixed 56dp row. Parley measures text at the actual
available width. Dialogs and scroll ranges must remain usable in landscape and with the IME.

Save a versioned native state packet through savedInstanceState: stack/route args, per-route
scroll, open modal and draft values, focus, UTF-16 selection/composition and pending platform
request IDs. Preference snapshots reload from Jetpref and take precedence over stale saved
committed values. Configuration recreation rebuilds surface/fonts/geometry without resetting
navigation. Process-death proof must kill a background process, relaunch from saved task state,
and check drafts/stack/focus; force-stop followed by a fresh launch is not process-death restore.
Packets must be bounded and validated; invalid/unknown versions fall back safely to Home.
Never persist a native pointer or retain an Activity in the worker. Surface destruction joins
the old generation before release. No render callbacks run while detached or idle.

## Route inventory

All `@Deeplink` paths below retain `ui://florisboard/` as their prefix, including the
BROWSABLE category used by scripts/render-benchmark. Typed arguments and serial names stay
compatible. Unported destinations route to the existing Compose NavHost with native return
state preserved. Setup remains Compose until independently ported; an incomplete setup must
not be bypassed by the prototype. Incoming ACTION_SEND / non-BROWSABLE ACTION_VIEW keeps the
existing cache/import path, not a settings deep-link reinterpretation.

| Route | Deep-link path | Arguments / special state |
| --- | --- | --- |
| Setup.Screen | no public deep link | IME enable/select and notification wizard |
| Settings.Home | `settings/home` | `` |
| Settings.Localization | `settings/localization` | `` |
| Settings.SelectLocale | `settings/localization/select-locale` | `` |
| Settings.LanguagePackManager | `settings/localization/language-pack-manage` | `(val action: LanguagePackManagerScreenAction)` |
| Settings.SubtypeAdd | `settings/localization/subtype/add` | `` |
| Settings.SubtypeEdit | `settings/localization/subtype/edit` | `(val id: Long)` |
| Settings.Theme | `settings/theme` | `` |
| Settings.ThemeManager | `settings/theme/manage` | `(val action: ThemeManagerScreenAction)` |
| Settings.Keyboard | `settings/keyboard` | `` |
| Settings.InputFeedback | `settings/keyboard/input-feedback` | `` |
| Settings.Smartbar | `settings/smartbar` | `` |
| Settings.Dictation | `settings/dictation` | `` |
| Settings.Typing | `settings/typing` | `` |
| Settings.Dictionary | `settings/dictionary` | `` |
| Settings.UserDictionary | `settings/dictionary/user-dictionary` | `(val type: UserDictionaryType)` |
| Settings.Gestures | `settings/gestures` | `` |
| Settings.Clipboard | `settings/clipboard` | `` |
| Settings.Media | `settings/media` | `` |
| Settings.Other | `settings/other` | `` |
| Settings.PhysicalKeyboard | `settings/other/physical-keyboard` | `` |
| Settings.Backup | `settings/other/backup` | `` |
| Settings.Restore | `settings/other/restore` | `` |
| Settings.About | `settings/about` | `` |
| Settings.ProjectLicense | `settings/about/project-license` | `` |
| Settings.ThirdPartyLicenses | `settings/about/third-party-licenses` | `` |
| Ext.Home | `ext` | `` |
| Ext.List | `ext/list` | `(val type: ExtensionListScreenType, val showUpdate: Boolean? = null)` |
| Ext.Edit | `ext/edit` | `(val id: String, @SerialName("create") val serialType: String? = null)` |
| Ext.Export | `ext/export` | `(val id: String)` |
| Ext.Import | `ext/import` | `(val type: ExtensionImportScreenType, val uuid: String? = null)` |
| Ext.View | `ext/view` | `(val id: String)` |
| Ext.CheckUpdates | `ext/check-updates` | `` |

Extension editing also has internal metadata/dependencies/component-create/file-editor states
inside ExtensionEditScreen. ThemeEditorScreen is entered for a theme component, including
metadata, fine tuning, rule/code/property edits and the live IME preview; it is not an omitted
extra public Routes entry. ExtensionNotFoundScreen and import/restore error states are retained.

## Widget and dialog inventory

| Widget family | Current uses and native requirement |
| --- | --- |
| Preference / groups | Home navigation, links, explanatory summaries, disabled/conditional rows and section headings |
| SwitchPreference | Boolean values, dependent enable/visible expressions, switch-plus-list combinations |
| ListPreference | Enum radios, descriptions, selection, optional enable switch and Default/Cancel/OK behavior |
| DialogSliderPreference / Slider | Integer/float ranges/steps, two orientation values, reset/default and confirm/cancel |
| ColorPickerPreference | Palette chips, Material You sentinel, RGB/HSV/hex input, alpha where supported, advanced mode |
| Checkbox / TriStateCheckbox | Backup group selection and restore choices; mixed checked state |
| Time picker | Theme day/night time and current local-time codec |
| Buttons / menus / chips | App bar/back, overflow, FAB/add/delete/import/export, dropdown lists and maintainers/keywords |
| Text / validation | Search, inline and multiline editors; errors, keyboard types, selection, composing, copy/paste |
| Lists / grids / tabs / drag | Locales/subtypes, extensions/components/files, property/rule rows and reorderable controls |
| Dialog / sheet / progress | Confirmation/error/unsaved state, extension editor sheets, import/export/restore progress |
| Keyboard / key preview | Existing native Snygg scene, draft updates, selected code/type, font/colour edits |

Every current explicit dialog call site follows (library-provided preference dialogs are
covered by the widget families above). Paths without an app prefix are relative to app/.

| File | Dialog calls / definitions (line at baseline) |
| --- | --- |
| `ext/ExtensionEditFilesScreen.kt` | 184: `JetPrefAlertDialog(`<br>254: `JetPrefAlertDialog(` |
| `ext/ExtensionEditScreen.kt` | 388: `FlorisUnsavedChangesDialog(`<br>403: `JetPrefAlertDialog(` |
| `ext/ExtensionMaintainerChip.kt` | 64: `JetPrefAlertDialog(` |
| `ext/ExtensionViewScreen.kt` | 201: `FlorisConfirmDeleteDialog(` |
| `settings/dictionary/UserDictionaryScreen.kt` | 316: `JetPrefAlertDialog(` |
| `settings/localization/LanguagePackManagerScreen.kt` | 200: `FlorisConfirmDeleteDialog(` |
| `settings/localization/LocalizationScreen.kt` | 159: `DeleteSubtypeConfirmationDialog(`<br>173: `fun DeleteSubtypeConfirmationDialog(`<br>179: `JetPrefAlertDialog(` |
| `settings/localization/SubtypeEditorScreen.kt` | 386: `JetPrefAlertDialog(`<br>424: `JetPrefAlertDialog(` |
| `settings/media/MediaScreen.kt` | 173: `DeleteEmojiHistoryConfirmDialog(`<br>194: `fun DeleteEmojiHistoryConfirmDialog(`<br>200: `JetPrefAlertDialog(` |
| `settings/theme/EditCodeValueDialog.kt` | 86: `internal fun EditCodeValueDialog(`<br>180: `JetPrefAlertDialog(` |
| `settings/theme/EditPropertyDialog.kt` | 185: `internal fun EditPropertyDialog(`<br>245: `JetPrefAlertDialog(`<br>670: `JetPrefAlertDialog(`<br>896: `JetPrefAlertDialog(`<br>1049: `JetPrefAlertDialog(`<br>1195: `JetPrefAlertDialog(` |
| `settings/theme/EditRuleDialog.kt` | 67: `internal fun EditRuleDialog(`<br>117: `JetPrefAlertDialog(`<br>242: `EditCodeValueDialog(` |
| `settings/theme/EnumLikeAttributeBox.kt` | 92: `JetPrefAlertDialog(` |
| `settings/theme/FineTuneDialog.kt` | 34: `fun FineTuneDialog(onDismiss: () -> Unit) {`<br>35: `JetPrefAlertDialog(` |
| `settings/theme/ThemeEditorComponents.kt` | 79: `internal fun ComponentMetaEditorDialog(`<br>100: `JetPrefAlertDialog(` |
| `settings/theme/ThemeEditorScreen.kt` | 183: `JetPrefAlertDialog(`<br>430: `ComponentMetaEditorDialog(`<br>439: `FineTuneDialog(onDismiss = { showFineTuneDialog = false })`<br>444: `EditRuleDialog(`<br>471: `EditPropertyDialog(` |
| `settings/typing/TypingScreen.kt` | 106: `ForgetLearnedWordsDialog(onDismiss = { confirmForgetLearned = false })`<br>191: `private fun ForgetLearnedWordsDialog(onDismiss: () -> Unit) {`<br>195: `JetPrefAlertDialog(` |

The confirmations cover subtype/extension/language-pack deletion, user-dictionary add/edit/
delete, emoji-history deletion, forgetting learned words, unsaved extension/theme changes,
file conflicts and rule/property removal. Theme value dialogs cover annotations/variables,
booleans/enums, dimensions/percentages, custom fonts, colours/palette, padding and corner shapes.

## Editable-field inventory

This includes reusable wrappers as well as concrete call sites. The preview editor is outside
app/ and dictation has its own package. Numeric slider and colour inputs owned by Jetpref are
also part of their corresponding widget port, not permission to omit editable controls.

| File | Field call sites (baseline lines) | Values |
| --- | --- | --- |
| `app/src/main/kotlin/com/jtn0123/audiowhisper/dictation/DictationScreen.kt` | 50 | Dictation vocabulary (multiline) |
| `ext/ExtensionEditFilesScreen.kt` | 222, 285 | Text file contents and filename |
| `ext/ExtensionEditScreen.kt` | 501, 510, 518, 526, 531, 537, 542, 547, 556, 809, 821, 834, 846, 876 | Metadata ID/version/title/description/keywords/homepage/issues/maintainers/license; new component ID/label/authors; shared editor wrapper |
| `settings/dictionary/UserDictionaryScreen.kt` | 371, 381, 388, 395 | Word, frequency, shortcut, locale |
| `settings/localization/SelectLocaleScreen.kt` | 96 | Locale search query |
| `settings/theme/EditCodeValueDialog.kt` | 270 | Key-code search/value |
| `settings/theme/EditPropertyDialog.kt` | 367, 527, 551, 574, 608, 697, 919, 1072, 1219 | Variable name, dp/sp/percent and numeric values, font name, padding/corner dimensions |
| `settings/theme/EditRuleDialog.kt` | 172 | Rule/element selector |
| `settings/theme/ThemeEditorComponents.kt` | 131, 140, 148, 163 | Component id, label, authors and draft properties |
| `app/src/main/kotlin/dev/patrickgold/florisboard/lib/compose/PreviewKeyboardField.kt` | 98 | Preview scratch text |
| `lib/compose/src/main/kotlin/org/florisboard/lib/compose/FlorisDropdownLikeButton.kt` | 56 | Dictation vocabulary (multiline) |

ThemeEditorScreen also holds the text-field drafts in its retained editing model. Fields can
be hidden in nested sheets/dialogs: searching only direct material3.TextField calls misses
JetPrefTextField and EditorSheetTextField. Full-port validation must include each row above.

## Platform effects inventory

| Owner | Contract / action / result handling |
| --- | --- |
| SetupScreen | RequestPermission POST_NOTIFICATIONS; persist permission state; IME enabler/picker; reopen Activity; privacy/repository URLs |
| HomeScreen / PreviewKeyboardField | Observe IME enabled/selected; ACTION_INPUT_METHOD_SETTINGS; InputMethodManager.showInputMethodPicker; failure toast |
| PhysicalKeyboardScreen | StartActivityForResult ACTION_HARD_KEYBOARD_SETTINGS; refresh on result |
| SpellCheckerServiceSelector | ACTION_INPUT_METHOD_SUBTYPE_SETTINGS with spell-checker extras; observe selected/enabled spellchecker |
| BackupScreen | CreateDocument ZIP; copy native-serialized backup via content resolver; share chooser/FileProvider; cancellation/errors/toasts |
| RestoreScreen | GetContent ZIP; read/extract/validate native restore plan; cancellation/errors/progress |
| UserDictionaryScreen | GetContent import; CreateDocument export combined list; system user-dictionary intent; Room/system provider access and errors |
| ExtensionImportScreen | GetMultipleContents; cache URI bytes, validate/import, preserve supplied uuid, cancellation/errors |
| ExtensionExportScreen | CreateDocument flex archive; copy/export result; cancellation/errors |
| ExtensionEditFilesScreen | GetContent for image/ordinary file; file write/conflict/new text file; cancellation/errors |
| AddonBox | Store URL and generated update URL |
| ExtensionMaintainerChip | mailto and homepage URLs |
| AboutScreen | Version-formatted changelog, repository and privacy-policy URLs |
| TypingScreen | Feedback-thread URL and forgetting learned-store effects/toast |
| Dictation settings / ModelStatus | Download/model file operations, external app settings link; no microphone needed for settings prototype |
| FlorisAppActivity | BROWSABLE deep links, ACTION_VIEW content import and ACTION_SEND clip URI import |

Activity-result registrations and concrete launcher invocations in app/ (not coroutine
`scope.launch` or imports) are listed explicitly below to make omissions reviewable.

| File | Source line / call |
| --- | --- |
| `ext/ExtensionEditFilesScreen.kt` | 115: `val importLauncher = rememberLauncherForActivityResult(`<br>238: `importLauncher.launch("*/*")`<br>247: `importLauncher.launch("*/*")` |
| `ext/ExtensionExportScreen.kt` | 57: `val exportLauncher = rememberLauncherForActivityResult(`<br>80: `exportLauncher.launch(ExtensionDefaults.createFlexName(ext.meta.id))` |
| `ext/ExtensionImportScreen.kt` | 132: `val importLauncher = rememberLauncherForActivityResult(`<br>190: `importLauncher.launch("*/*")` |
| `settings/advanced/BackupScreen.kt` | 129: `val backUpToFileSystemLauncher = rememberLauncherForActivityResult(`<br>179: `backUpToFileSystemLauncher.launch(backupWorkspace!!.zipFile.name)` |
| `settings/advanced/PhysicalKeyboardScreen.kt` | 38: `val activityForResult = rememberLauncherForActivityResult(`<br>48: `activityForResult.launch(Intent(Settings.ACTION_HARD_KEYBOARD_SETTINGS))` |
| `settings/advanced/RestoreScreen.kt` | 96: `val restoreDataFromFileSystemLauncher = rememberLauncherForActivityResult(`<br>187: `restoreDataFromFileSystemLauncher.launch("*/*")` |
| `settings/dictionary/UserDictionaryScreen.kt` | 136: `val importDictionary = rememberLauncherForActivityResult(`<br>161: `val exportDictionary = rememberLauncherForActivityResult(`<br>215: `importDictionary.launch("*/*")`<br>222: `exportDictionary.launch("my-personal-dictionary.clb")` |
| `setup/SetupScreen.kt` | 82: `rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { isGranted ->`<br>230: `requestNotification.launch(android.Manifest.permission.POST_NOTIFICATIONS)` |

The often-quoted “33 launcher usages” includes imports and callback return labels in a text
search; it is not 33 independent system contracts. Preserve each actual contract and every
invocation above. Android permissions additionally include user-dictionary/provider access,
model/network operations and the IME's existing microphone path; settings does not request
microphone merely to edit vocabulary. No platform UI or permission grant is simulated by Rust.

## Prototype scope and proof plan

Default-off `other__rust_settings_prototype` is separate from the keyboard renderer setting.
Port Home and the full Keyboard preference screen, including switches, list choices and
paired sliders, plus the existing scratch preview text field and IME picker action. The
preview field supplies the text-field proof without modifying user dictionaries. All other
routes remain functional through Compose. Preserve native stack on fallback return; deep-link
routing and back must work from either root. No Compose fallback is removed at this checkpoint.

| Risk | Required proof |
| --- | --- |
| Stored behavior diverges | Identical script on Compose/Rust from the same run-as Jetpref file; normalize only the prototype selector before comparing full bytes; controls return identical stored preferences without a re-baseline |
| Navigation/widget policy moves incorrectly | Test-only frozen Kotlin reference for migrated decisions, JVM seeded match suites and deliberate production mutations; no removed assertions |
| Text input breaks | Rust-owned UTF-16 model versus Kotlin reference including composition, surrogate deletion, selection, hardware edits and saved drafts; live IME typing without retaining contents |
| Appearance/font clipping | Reviewed Compose/Rust Home and Keyboard screenshots in light/dark/fontScale 2.0; scrolled dialogs and landscape; account for native antialiasing |
| Accessibility cannot operate controls | Fresh trees for both paths, roles/states/labels/ranges/actions; execute every native control type, dialog/focus/back/scroll; TalkBack bound while muted |
| Surface/thread leaks | Rotation/recreation/foreground-background and process-death restore; balanced native-window counts, no stale callbacks or pointers |
| Performance regresses | Alternating scrolling p50/p95, Activity first posted frame, APK/.so bytes and process PSS; build idle during timing; retain measurement boundaries |
| Fallback strands a route | Replay each public deep link and return across native/Compose; extension import intents keep current behavior |
| JNI declaration drift | Existing full gate includes source-to-packaged-native guard; missing-symbol mutation must fail and restoration pass |

The exact required Gradle gate must exit zero before every push. Only the three named lint
IDs are permitted; no compiler warnings, suppressed lint, disabled tests or silent retries.
Run both render-benchmark harness test suites after harness edits. Save any crash output under
/Volumes/2TB/aw-scratch/. Device proof uses only aw_keyboard_ssd through its T3 agent-device
session, install-dev.sh, verified silent streams and exact preference/accessibility/IME/store
restoration. Shut down only that AVD through device_close and confirm its runtime absent.

## Source budget and full-port endpoint

At `04186d1d`, the exact supplied rust_share.py rules give **52,745 Kotlin / 321 Java /
51,500 Rust = 49.25% Rust**. These are nonblank tracked lines, including comments, not a
performance metric. The supplied script includes latin-oracle main source even though the
written target excludes that test-only crate; retain the requested script result and do not
quietly change its denominator. Fixtures use `/test/` paths so they stay excluded.

Current scope is 12,537 nonblank Kotlin in app/, 1,889 in lib/compose, 1,754 in Snygg/ui
and 503 in app's shared Compose helpers. Several files include retained Android/model code,
so **16,683 lines is an upper bound on removable scope**, not a promised deletion count.
Stage 3's measured keyboard deletion removes another 6,552 lines at its historical baseline;
refresh that dry run after the full port, since shared ownership has changed.

Planning range for the complete settings rewrite: remove 13–16k Kotlin and add 7–12k Rust
(including JNI). With the current fallback retained until phone acceptance, implementation
alone increases the denominator. After accepted settings deletion, the expected share is
roughly **59–63%**; after also deleting the Compose keyboard, roughly **63–67%**. These
are sizing assumptions, not measured delivered code or an ETA. This scope alone does **not**
reach 90%. At 65–75k Rust, 90% requires at most about 7.2–8.3k Kotlin+Java. A separate
inventory must remove/port roughly another 20k of remaining model/wrapper/host source;
never manufacture Rust or delete tests to close that gap.

Kotlin that must remain: IME/Activity/service lifecycle; Android InputConnection and minimal
editable View; SurfaceView/Choreographer/insets/system colours/fonts/resources; Android
AccessibilityNodeProvider and actions; ActivityResult/SAF/permissions/intents/toasts; Jetpref,
Room/clipboard/content-provider and file/network IO; Android ICU/Locale; microphone/playback/
audio-focus APIs and rich/inline platform views. Eliminate decisions from these adapters,
not platform capabilities. The full-port budget must count these honestly before 90% is claimed.

Open decisions before a full port: approve the detailed endpoint scope beyond settings needed
for 90%; agree on visual tolerance for Material controls, selection handles and predictive-back
animation; choose whether unsupported/oversized restored drafts should be discarded or kept
in an Android private-state file. No answer is needed to build this bounded default-off
prototype. Phone acceptance, 120Hz rendering measurements and deletion remain later checkpoints.
