# Stage 3B package 8: final IME Compose surfaces

## Inventory (baseline 40c5f117 and current prototype path)

Source reachability starts at `FlorisImeService.onCreateInputView` and
`onCreateExtractTextView`, then `FlorisImeViews` and `FlorisImeUi`. "YES" below
means the file executes a composable on the prototype path, even if it only supplies
state or Android interop. An inactive declaration is not counted as composing. Resolved gaps retain their baseline
call-path evidence below; all unqualified YES/No entries describe the current path.
All paths in the first table are relative to
`app/src/main/kotlin/dev/patrickgold/florisboard/ime/`.

| File | Prototype on: evidence | Rust/platform replacement | Deletion blocker |
| --- | --- | --- | --- |
| `clipboard/ClipboardInputLayout.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `clipboard/FlorisCopyToClipboardActivity.kt` | No: separate share-target Activity, outside input window | Settings/share Activity remains Compose | Out of deletion scope |
| `clipboard/provider/ClipboardDatabase.kt` | No: Compose `rememberIcon` belongs to fallback cards | RendererClipboard resource adapter | Keep Room entities/storage, split UI helper |
| `core/SelectSubtypePanel.kt` | No after ports; baseline YES: `FlorisImeViews.Content` subtype branch ignored prototype | Package 8 native subtype sheet | Phone acceptance; state extension separated |
| `keyboard/FlorisImeSizing.kt` | YES: sizing provider / collectAsState | Host measurement / state adapter | Contains non-UI model/platform API; retain or split |
| `keyboard/KeyboardState.kt` | YES: sizing provider / collectAsState | Host measurement / state adapter | Contains non-UI model/platform API; retain or split |
| `media/MediaInputLayout.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `media/emoji/EmojiPaletteView.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `onehanded/OneHandedPanel.kt` | No after ports; baseline YES: `FlorisImeUi.ImeUi` wrapped both renderers | Package 8 native side controls/geometry | Phone acceptance; controls and geometry ported |
| `popup/PopupUi.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `popup/PopupUiController.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Phone acceptance; shared popup state/models remain outside controller |
| `sheet/BottomSheetHostUi.kt` | No after ports; baseline YES: `FlorisBottomSheetHostUiView.Content` | Native modal scrim/layout | Phone acceptance; state helper separated |
| `smartbar/CandidatesRow.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `smartbar/InlineSuggestionsUi.kt` | YES: `RustKeyboardLayout` inline reservation and style cache | Android InlineContentViews must remain | Replace Compose Row/scroll host with Android host; style cache is used by service |
| `smartbar/Smartbar.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `smartbar/quickaction/QuickAction.kt` | YES: RendererSmartbar/RendererPanels call composable localized display-name/tooltip helpers | Native smartbar/panels | Retain sealed model, dispatch and resource helpers |
| `smartbar/quickaction/QuickActionButton.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `smartbar/quickaction/QuickActionsEditorPanel.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `smartbar/quickaction/QuickActionsOverflowPanel.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `smartbar/quickaction/QuickActionsRow.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `text/TextInputLayout.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `text/keyboard/TextKeyboardLayout.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `theme/FlorisImeTheme.kt` | YES: input, sheet and extract wrappers | ThemeManager + native theme resolution | Composition locals/palette still shared with settings |

| Other reachable file | Prototype on: evidence | Replacement | Deletion blocker |
| --- | --- | --- | --- |
| `FlorisImeUi.kt` | YES: service → ComposeInputView → ImeUiWrapper | Native surface with Android host | Root layout, insets, background image and observable adapters |
| `FlorisImeViews.kt` | YES: input/sheet/extract roots | Android host + native chrome | ExtractEditText must remain a View; lifecycle/insets |
| `lib/compose/SystemUi.kt` | YES: ImeUiWrapper → SystemUiIme | WindowInsetsController platform adapter | Shared Activity helper remains |
| `render/RustKeyboardLayout.kt` | YES: main prototype branch | RustKeyboardView draws; Compose observes/transports | Live bindings, insets, inline view interop |
| `render/RendererSmartbar.kt` | YES: RustKeyboardLayout → rendererSmartbar | Native smartbar | Resource/state transport only |
| `render/RendererPanels.kt` | YES: RustKeyboardLayout → rendererPanels | Native panel | Resource/state transport only |
| `render/RendererEmoji.kt` | YES: rendererPanels → rendererEmoji | Native emoji | Resource/state transport only |
| `render/RendererClipboard.kt` | YES: rendererPanels → rendererClipboard | Native clipboard | Room/resource transport only |
| `render/RendererGif.kt` | YES: rendererPanels → rendererGif | Native GIF | Network/resource transport only |
| `render/RendererSearch.kt` | YES: RustKeyboardLayout → rendererSearch | Native search strip | Resource/state transport only |
| `dictation/DictationPreview.kt` | No after ports; baseline YES: RustKeyboardLayout overlaid preview | Package 8 native text | Phone acceptance; start ellipsis/semantics ported |
| `dictation/LitMic.kt` | No: fallback QuickActionButton | Native smartbar microphone | Fallback retained |
| `gif/GifPanel.kt` | No: fallback MediaInputLayout | Native GIF/search | Fallback retained |
| `app/settings/theme/ThemeEditorScreen.kt` | Activity only; live preview is the actual IME via previewThemeInfo → activeThemeInfo | Same native scene observes activeThemeInfo | Keep settings Activity |
| `app/settings/theme/EditRuleDialog.kt` | Activity only; TextKeyDataPreviewBox uses Material Compose (not Snygg in this baseline) | Offscreen native key pixels in Compose Image | Split oversized file; fallback until phone acceptance |

`render/`, `dictation/`, and `gif/` abbreviate
`app/src/main/kotlin/com/jtn0123/audiowhisper/`; other paths above are under
`app/src/main/kotlin/dev/patrickgold/florisboard/`.

All `lib/snygg/.../ui` files are individually inventoried below. These are shared
settings dependencies as well as fallback keyboard primitives: deleting the directory
requires removing settings references or retaining an explicitly separated settings API.

| File in `lib/snygg/src/main/kotlin/org/florisboard/lib/snygg/ui` | Prototype on: evidence | Replacement | Deletion blocker |
| --- | --- | --- | --- |
| `SnyggBox.kt` | YES: ImeUi root/extract | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggButton.kt` | No after ports; baseline YES: extract action | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggChip.kt` | No: fallback/settings-only primitive | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggColumn.kt` | No after ports; baseline YES: subtype/one-handed | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggIcon.kt` | No after ports; baseline YES: subtype/one-handed | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggIconButton.kt` | No after ports; baseline YES: one-handed | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggListItem.kt` | No after ports; baseline YES: subtype | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggPalette.kt` | YES: native palette transport | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggRow.kt` | No after ports; baseline YES: subtype/extract/dictation | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggSpacer.kt` | No: fallback/settings-only primitive | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggSurfaceView.kt` | YES: root background image surface | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggText.kt` | No after ports; baseline YES: subtype/extract/dictation | Native theme/scene; Android images/insets | Phone acceptance; deletion dry run compiles |
| `SnyggUi.kt` | YES: theme locals/queries | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |

## Modal, transient and platform surfaces (baseline gap audit)

- Subtype sheet: Compose BottomSheetHostUi + SelectSubtypePanel (confirmed gap).
- Quick-action order sheet: native Panel kind 3, including scrim and dismissal.
- Clipboard clear confirmation and pin/delete menus: native clipboard panel; fallback
  ClipboardInputLayout contains the Compose versions.
- Emoji history/tone popups: native emoji panel; text popups: native popup_scene.
- One-handed controls: Compose OneHandedPanel (confirmed gap). No separate IME
  height/one-handed adjustment dialog exists; preferences are edited in settings screens.
- Incognito behind-keyboard watermark: fallback TextInputLayout only; Rust does not
  currently draw it (confirmed gap). Smartbar incognito toggle icon is already native.
- Landscape extraction: separate Compose extract chrome + Android ExtractEditText;
  keyboard remains Rust via the ordinary input view.
- Dictation microphone/status: native smartbar; preview: Compose DictationPreview.
  DictationScreen/ModelStatus are settings Activity UI, not IME overlays. Microphone
  permission and voice-input switching are Android system UI.
- Candidate tooltips: Android PopupWindow/TextView (`RendererActionTooltip`), not Compose.
- Toasts: Android Toast through service/platform helpers for dictation failures, clipboard,
  emoji removal and GIF send outcomes. They are system windows and remain platform-owned.
- No IME Snackbar host is present. Share-target clipboard bottom sheet is a separate Activity.

## Prototype path after the ports

The subtype sheet, one-handed controls, incognito watermark, extraction action/chrome,
and dictation preview draw through Rust. The settings key preview receives an offscreen
Vello CPU pixel buffer. Compose remains the fallback with the setting off.

`FlorisImeViews` selects `RustSubtypeSheet` instead of the old sheet. Its window-wide
`RustChromeView` hosts native panel kind 7 independently of compact keyboard placement. `FlorisImeUi` calls `RustOneHandedPanel` on the prototype branch.
`RustKeyboardLayout` no longer calls DictationPreview, and `NativeExtractContent` wraps
RustExtractView while retaining the real ExtractEditText. Shared portrait-only compact
font and row/smartbar scaling also call native rules; Android resources/insets stay in the host. The settings preview fallback
is now isolated in TextKeyDataPreviewBox.kt; EditRuleDialog.kt is below 500 lines.

Live editor snapshots needed a fix: ThemeInfo's constructor-cached stylesheetJson survived
`copy(stylesheet = draft)`. JSON is now lazily derived once per new ThemeInfo instance.
ThemePreviewSnapshotTest checks repeated draft copies through actual JNI pixels and
preservation of the original snapshot. A deliberate stale-base mutation fails that test.

Root Compose remains: ImeUiWrapper/FlorisImeTheme provide resources, sizing, palette and
observable bindings; the window SnyggBox/SnyggSurfaceView still handle the outer background,
including extension images and insets. InlineSuggestionsUi hosts Android InlineContentViews.
The bottom-sheet Compose root hosts `RustSubtypeSheet` as an AndroidView; the old
`BottomSheetHostUi` composable is bypassed. The extract ComposeView is an interop/state host on the prototype branch.
These shared host dependencies are explicitly retained in the deletion dry run below.

### Reachable shared Compose helpers

| File | Composes with prototype on? / evidence | Replacement | Deletion blocker |
| --- | --- | --- | --- |
| `reader/LitSpeaker.kt` | No: QuickActionButton fallback only | Native smartbar speaker | Fallback retained |
| `app/AppPrefs.kt` | Yes: fontSizeMultiplier from native host | Preference/platform transport | Shared settings model |
| `lib/StateAdapters.kt` | Yes: observed bottom offset/sizing | Flow transport | Shared settings/host binding |
| `lib/util/InputMethodUtils.kt` | No IME drawing: settings preview field helpers | Android IME APIs | Shared Activity helper |
| `lib/color/.../ColorMappings.kt` | Yes: dynamic palette in FlorisImeTheme | Platform color transport | Shared settings palette |
| `lib/compose/.../Resources.kt` | Yes: localized strings and ProvideLocalizedResources | Android resource transport | Shared application localization |
| `lib/compose/.../Modifier.kt` | Yes: inline scroll wrapper; fallback sheet/grid modifiers | Android scroll host | Inline interop and fallback callers |
| `lib/compose/.../DisposableLifecycleEffect.kt` | No: TextKeyboardLayout fallback frame hook | Native surface lifecycle | Fallback retained |
| `lib/compose/.../LazyGrid.kt` | No: emoji fallback staggered grid | Native panel layout | Fallback retained |

App paths above are relative to their existing `dev/patrickgold/florisboard` or
`com/jtn0123/audiowhisper` package, as applicable. `lib/compose/...` means
`lib/compose/src/main/kotlin/org/florisboard/lib/compose/`. The clipboard share Activity
also reaches FlorisAppActivity.kt, app/apptheme/Theme.kt, ExtensionImportScreen.kt,
lib/compose/FlorisScreen.kt, PreviewKeyboardField.kt and the generic library
FlorisAppBar.kt, FlorisBulletSpacer.kt, FlorisButtonBar.kt, FlorisButtons.kt and
FlorisCards.kt. Those files do not compose inside the input window and remain settings/
share Activity UI; they have no keyboard deletion replacement or blocker.

| New adapter file | Composes with prototype on? / evidence | Drawing owner | Deletion blocker |
| --- | --- | --- | --- |
| `render/RendererTheme.kt` | Yes: side and extract hosts | Native theme resolution | Shared palette/state binding |
| `render/RendererSubtypes.kt` | Yes: RustSubtypeSheet | Native subtype panel | Android locale/resource data binding |
| `render/RendererDecorations.kt` | Yes: RustKeyboardLayout | Native watermark/preview | Android resource/preference binding |
| `render/RustSubtypeSheet.kt` | Yes: FlorisBottomSheetHostUiView | RustChromeView/Vello | AndroidView host and subtype dispatch |
| `render/RustOneHandedPanel.kt` | Yes: FlorisImeUi | RustChromeView/Vello | AndroidView host and preference writes |
| `NativeExtractContent.kt` | Yes: prototype extract branch | RustExtractView/Vello + Android ExtractEditText | AndroidView host and editor action API |
| `app/settings/theme/EditCodeValueDialog.kt` | Settings only: extracted from EditRuleDialog | Same settings Material dialog, native key pixels when enabled | Retained settings UI |
| `app/settings/theme/EnumLikeAttributeBox.kt` | Settings only: extracted helper | Same Material settings controls | Retained settings UI |
| `app/settings/theme/TextKeyDataPreviewBox.kt` | Settings only: prototype branches to RustKeyPreview | Offscreen pixels; Compose key fallback until phone acceptance | Delete fallback branch, retain settings wrapper |
| `render/RustKeyPreview.kt` | Settings only: prototype key-preview branch | Offscreen Rust/Vello pixels | Keep Compose Image inside settings |

The clipboard fallback is now split across `ClipboardInputLayout.kt`,
`ClipboardHeader.kt`, `ClipboardItemView.kt` and `ClipboardComponents.kt`. All four
are bypassed with the prototype on and included in the deletion manifest. Shared
`ClipboardGridConstants.kt`, `PopupVisibilityState.kt`, `BottomSheetState.kt` and
`SubtypeSelectionState.kt` contain model/state only and remain. Inline scrollbar height
lives with its retained inline host. No compatibility stubs replace deleted functionality.

## Deletion dry run

On throwaway `package8-final-deletion-check`, removed 35 fallback files and mechanically
made Rust the sole input/sheet/extract branch. `:app:compileDebugKotlin` exits **0**.
The branch was discarded without a deletion commit, merge or push. Earlier trials exposed
an unused SnyggBox preview dependency and a clipboard constant defined inside fallback UI;
both real dependencies were separated on main before repeating the clean dry run.

| Nonblank tracked production lines | Main before deletion | Dry run |
| --- | ---: | ---: |
| Kotlin | 55,065 | 48,513 |
| Java | 627 | 627 |
| Rust | 41,959 | 41,959 |
| Rust share | 42.9683% | 46.0587% |

Net Kotlin removed: **6,552** (35 fallback files plus bypassed root/extract/settings-preview branches). Counting excludes `/test/`, `/tests/`, `/androidTest/`,
`*Test.kt`, `_test(s).rs`, `tests.rs`, `benchmark/` and generated Latin character tables.
Blank lines are excluded; comments count as nonblank lines. The dry run retains the four
root/theme Snygg files (`SnyggBox`, `SnyggSurfaceView`, `SnyggUi`, `SnyggPalette`), the
Compose state/resource/inline adapters and settings UI. It proves the inventoried keyboard
fallback deletion compiles, not that all Compose dependencies can be removed.

## Verification evidence

Final emulator and local-gate results follow below. Prototype default remains false;
phone acceptance and committing the Compose deletion remain outside this package.


### Rule parity and deliberate mutations

| Rule / test-only Kotlin reference | Match coverage | Mutation caught, then restored |
| --- | --- | --- |
| Subtype configured order, localized labels and Long selected id / `SubtypeMatchTest` | 4,000 seeded cohorts | Inverted selected-id comparison |
| Bottom-aligned sheet and keyboard geometry / `SubtypeMatchTest` | 1,701 layouts including navigation insets | Sheet top shifted by one pixel; navigation inset ignored |
| One-handed Row weights and side targets / `OneHandedMatchTest` | 408 weight combinations, 60 target layouts | Scale divisor 99; close target width +1 |
| Shared compact font and row/smartbar heights / `ChromeSizingMatchTest` | 24,684 font combinations, 324 height combinations | Font divisor 99; smartbar multiplier .752 instead of .753 |
| Extraction chrome / `ExtractChromeMatchTest` | 135 layouts | 15dp margin instead of 16dp |
| Intrinsic single-key preview / `KeyPreviewMatchTest` | 25 dimensions plus actual JNI pixels and icon precedence | 35dp minimum instead of 36dp |
| Start ellipsis / `DictationPreviewMatchTest` | 1,212 fixed-advance strings plus native grapheme tests | Appended ellipsis instead of prepending |
| Minimum header/action semantic and touch targets / `ChromeMinimumTargetMatchTest` | 32 density/height/header combinations | 47dp instead of 48dp |
| Incognito watermark visibility / `IncognitoWatermarkMatchTest` | All four boolean combinations | OR instead of AND |
| Editor snapshots / `ThemePreviewSnapshotTest` | Two successive stylesheet copies through actual JNI pixels | Stale base JSON reused |

All mutations produced test failures and were restored before the gate. Native tests also
cover translucent scrim repainting, selected-row semantics, chrome metadata/theme inheritance,
bitmap resizing and side-action targets. `ChromeThemeTransportTest` checks mode/shift transport.
The Python harness has 26 passing tests, including adversarial wrong label, state, action,
geometry and relative viewport-edge cases. No test was skipped or ignored.

### Local gate and source accounting

`./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck assembleDebug`
completed with **Gradle exit 0**, 1m36s after `:app:clean` (`build/stage3b8/local-gate14.log` and `.exit`).
Clippy runs with `-D warnings`. Lint reports only AndroidGradlePluginVersion (1),
NewerVersionAvailable (2), ChromeOsAbiSupport (1). App JVM reports: 408 tests;
Rust bridge JVM reports: 150 tests; zero failures, errors or skips in either cohort.
Every changed/new Kotlin, Java and Rust source file is at most 500 lines.

Relative to 40c5f117, nonblank production Kotlin: 1,440 added / 825 removed (net +615);
Rust: 1,232 added / 40 removed (net +1,192); Java unchanged. This package retains the
fallback, so it does not yet deliver the dry-run Kotlin reduction. Counts use the same
tracked-file exclusions as the deletion table.


### Theme editor and settings preview

An unsaved extension copied from each Floris Day/Night template was edited on both paths.
Changing `--primary` to the same fixed blue changed the real IME return key immediately;
`previewThemeInfo → activeThemeInfo → stylesheetJson → RustKeyboardLayout` stays live.
Before/after and renderer pairs are under `build/stage3b8/editor-final/`, named
`{day,night}-{before,after}-pair.png`. Each temporary component/extension draft was
removed/discarded through the editor; no theme was saved.

The code-97 single-key preview pairs are `{day,night}-key-preview-pair.png`. The final
accessibility check uses `compose-day/key-preview-final.json` and
`rust-day/key-preview-final.json`: both expose `a`, the Unicode display name and key type;
Rust uses an Image node for `a` where Compose used Text. Zero missing-label bug verdicts
(`editor-final/accessibility.json`). Icons retain the original decorative semantics.
Offscreen Rust uses the same theme resolver/Vello CPU path, Material foreground, 12dp
corner radius, 36dp minimum and 16sp text/24sp icon sizing as the baseline preview.
The settings Activity remains Compose, with the old key drawing only in the off fallback.


A startup landscape capture exposed a stale rich-editor-dependent "Select all" action.
KeyboardManager now observes `activeInfoFlow` as well as keyboard-state changes: a field
restart need not change any keyboard-state bit. The fixture waits for the known rich-editor
action state instead of accepting the unspecified startup editor. Earlier failing captures
are retained separately; final comparisons use fresh files after this fix.


A later runtime start caught stale incremental Compose output: callers referenced absent
`$stable` fields on the Keyboard and Clipboard preference classes. This was confirmed in
pre-R8 bytecode, independently of JNI. `ComposeStabilityLinkageTest` scans actual compiled
production field references; it failed on both missing fields before a clean app build.
The final gate includes this test and the rebuilt APK is used for the final runtime pass.
The original crash log and negative test are retained as
`build/stage3b8/failed-start-runtime.log` and `stale-stability-negative.log`.


### Emulator surface and workflow evidence

The dedicated `aw_keyboard` runtime was `emulator-5556`; the other session's emulator was
not used. Installation used `scripts/install-dev.sh`. Final chrome captures and full IME
accessibility dumps are in `build/stage3b8/visuals-complete/`:

| Surface | Floris Day pair | Floris Night pair |
| --- | --- | --- |
| Subtype sheet | `day-subtypes-pair.png` | `night-subtypes-pair.png` |
| Sheet over compact keyboard | `day-subtypes-compact-pair.png` | `night-subtypes-compact-pair.png` |
| One-handed left | `day-onehand-left-pair.png` | `night-onehand-left-pair.png` |
| One-handed right | `day-onehand-right-pair.png` | `night-onehand-right-pair.png` |
| Incognito watermark | `day-incognito-pair.png` | `night-incognito-pair.png` |
| Landscape extraction | `day-extract-pair.png` | `night-extract-pair.png` |

All **12** comparisons have **zero bug verdicts** (`a11y-complete.log`,
`visuals-complete/comparison.json`). The subtype header's disabled state, row labels,
selection/click/dismiss targets, both side controls and extract action are represented.
The real Android editor remains in the extraction tree. Rust adds explicit selected-row
and dismissal semantics where Compose used wrapper/glyph nodes.

The screenshots retain minor renderer differences in glyph antialiasing, icon edges and
subpixel rounding (including about one pixel of extract-field width). Empty-editor cursor
blink and draft extension ids/time differ between captures. No missing control, misplaced
sheet, wrong panel color or incorrect one-handed placement remains in these pairs. This
is visual/semantic emulator evidence, not pixel equality or phone acceptance.

`workflow-complete/comparison.json` matches both renderers: the real space-hold sheet
selects UK id 8008, with the exact configured subtype serialization preserved; the compact
END sheet also selects 8008. START typing stores enabled=true, side=START, scale=80 and
produces SHA-256 `77af778b51abd4a3c51c5ddd97204a9c3ae614ebccb75a606c3b6865aed6744e`,
3 UTF-16 units, cursor 3..3. One earlier cold-start sheet hold timed out without inserting
text; a fresh full workflow passed. That transient invocation was not isolated to a root
cause, and the failed run remains in `workflow-accepted.log`; no retry was hidden inside
the fixture or waived by its comparator.


### Typing regression and logcat

Both renderers pass the 300-press and eight `e`-hold fixture, with identical stored digest
and selection results (`build/stage3b8/regression-accepted/`):

| Phase | UTF-16 units / cursor | SHA-256, both renderers |
| --- | --- | --- |
| 300 presses | 300 / 300..300 | `809d588bf48a1e900159a2692fc202c6a7d10d95dfdcc48767851aaa041ef7d3` |
| Eight holds added | 308 / 308..308 | `c14d19d26f290a2478c424ae4fb4a4f28f538a5677f12491d526554126cbc555` |

The final accepted runtime window has no `FATAL|AndroidRuntime` matches: both
`pre-host-logcat.log` and `final-logcat.log` are empty. The earlier incremental-build crash
is preserved and described above, rather than counted as a successful trial.


### Root-host decision and creation timestamps

**Keep the current Compose root.** The condition that only empty root/sheet hosts remain
is not met: live theme/resource/preference adapters, Snygg window/background images,
insets/sizing and the Android inline-content wrapper still depend on composition. Merely
constructing RustKeyboardView from a plain View would still load Compose through them.
A complete lifecycle/Flow/inline/image host rewrite is not a clean factory swap. No plain-
View alternative was implemented or timed, and no plain-View performance benefit is claimed.
This decision is also recorded in `docs/decisions.md`.

`host.py` records monotonic timestamps at input-view factory entry, return and first
nonzero global layout, without text payloads. Cold means a new keyboard process; warm
means configuration recreation in the same process. Each renderer has 10 cold samples
with 10 distinct PIDs and 10 warm samples with one PID. Frame profilers and the Hybrid
experiment are off. Accepted raw timestamps/summaries are in `host-complete/runs.json`
and `host-complete/summary.json`; `host-complete/timings.html` is an interactive chart.
An earlier trial with frame profiling enabled is excluded (`host/`).

| Current path | Start condition / n | Factory-return median ms | First-layout median ms | First-layout min..max ms |
| --- | --- | ---: | ---: | ---: |
| Compose | cold / 10 | 138.116 | 481.757 | 414.511..528.014 |
| Compose | warm / 10 | 26.428 | 124.089 | 108.537..129.898 |
| Rust | cold / 10 | 4.345 | 506.209 | 469.973..639.574 |
| Rust | warm / 10 | 21.016 | 103.632 | 94.257..128.384 |

Factory-return cold samples are bimodal (roughly 1–198ms) and both paths instantiate the
same Compose hosts; the factory median difference is not evidence of a renderer or host
speedup. First-layout distributions overlap. These are emulator factory/layout boundaries,
not first native frame, touch-to-display latency, whole-process startup or S24 performance.
They do not change the earlier renderer performance gate or authorize default-on.

### Restoration and completion boundary

`build/stage3b8/cleanup.json` verifies exact original preference bytes, original
accessibility values, selected project IME, probe removal, zero saved temporary-theme
files, DND off, profiling tags off and no fatal logcat entries after timing. Thus subtype
list/id, one-handed side/enabled/scale, height, incognito and theme settings are restored;
prototype is **off**, **Always day + Floris Day**. The dedicated runtime was closed with
`device_close(shutdown=true)` successfully. No other session's emulator was used or closed.

No Compose deletion was committed, and the default remains false. Root/inline Compose
removal, live service-generated inline suggestions, live GIPHY results, voice-engine
acceptance and the physical S24 acceptance/performance pass are not established by this
package's emulator/static/native evidence. The existing package 7 limitations still apply.

### Reviewable commits

The final evidence-document commit is reported separately in the handoff. Implementation
and preparatory commits, oldest first (all include the requested co-author trailer):

| SHA | Subject |
| --- | --- |
| `dc7bbc73` | docs: inventory final Compose surfaces for renderer package 8 |
| `a729f9f4` | render: draw subtype selection sheet and semantics in Rust |
| `7e7b5981` | render: port one-handed geometry and side controls to Rust |
| `6584f128` | render: draw fullscreen extract chrome around Android editor |
| `529d659b` | render: add themed incognito watermark behind native text keys |
| `da0d7e74` | render: draw dictation preview with native start ellipsis |
| `5b3eaf7e` | render: replace settings key preview with offscreen Vello pixels |
| `ea28069b` | render: retain dimmed keyboard beneath native subtype sheet |
| `76fa65e0` | render: match side-control icon size and chrome theme inheritance |
| `feb00c08` | render: preserve extract text styling and action touch target |
| `e598c188` | render: give preview icons precedence over key labels |
| `1c0b5184` | render: refresh native theme JSON when editor drafts are copied |
| `3bf365f0` | test: add chrome parity fixtures and timestamp-only host measurements |
| `d65ac852` | refactor: separate shared IME state from deletable Compose surfaces |
| `d508d781` | refactor: remove the remaining shared dependencies on Compose panels |
| `de5507e2` | render: keep subtype modal window-wide over compact keyboards |
| `6a58f8b9` | refactor: keep clipboard fallback files within the source size limit |
| `616133f2` | render: match modal scrims and minimum chrome accessibility targets |
| `e5a7b145` | render: preserve the text key preview accessibility label |
| `b9ec2f2c` | fix: refresh keyboard evaluators when the editor changes without a state change |
| `3707cdc7` | render: own compact font and row height scaling in Rust |
| `b83f4f63` | test: reject missing Compose stability fields in compiled output |
| `bbe7ef4c` | test: isolate creation timestamps from frame profiling |
