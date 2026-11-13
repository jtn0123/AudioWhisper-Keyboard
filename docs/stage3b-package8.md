# Stage 3B package 8: final IME Compose surfaces

## Inventory at 40c5f117 (before implementation)

Source reachability starts at `FlorisImeService.onCreateInputView` and
`onCreateExtractTextView`, then `FlorisImeViews` and `FlorisImeUi`. "YES" below
means the file executes a composable on the prototype path, even if it only supplies
state or Android interop. An inactive declaration is not counted as composing.
All paths in the first table are relative to
`app/src/main/kotlin/dev/patrickgold/florisboard/ime/`.

| File | Prototype on: evidence | Rust/platform replacement | Deletion blocker |
| --- | --- | --- | --- |
| `clipboard/ClipboardInputLayout.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `clipboard/FlorisCopyToClipboardActivity.kt` | No: separate share-target Activity, outside input window | Settings/share Activity remains Compose | Out of deletion scope |
| `clipboard/provider/ClipboardDatabase.kt` | No: Compose `rememberIcon` belongs to fallback cards | RendererClipboard resource adapter | Keep Room entities/storage, split UI helper |
| `core/SelectSubtypePanel.kt` | YES: `FlorisImeViews.Content` subtype branch ignores prototype | Package 8 native subtype sheet | Port sheet; keep state extension outside deleted UI |
| `keyboard/FlorisImeSizing.kt` | YES: sizing provider / collectAsState | Host measurement / state adapter | Contains non-UI model/platform API; retain or split |
| `keyboard/KeyboardState.kt` | YES: sizing provider / collectAsState | Host measurement / state adapter | Contains non-UI model/platform API; retain or split |
| `media/MediaInputLayout.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `media/emoji/EmojiPaletteView.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `onehanded/OneHandedPanel.kt` | YES: `FlorisImeUi.ImeUi` Row wraps both renderers | Package 8 native side controls/geometry | Port controls and geometry |
| `popup/PopupUi.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `popup/PopupUiController.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Keep shared PopupExtendedResult API; isolate Compose controller |
| `sheet/BottomSheetHostUi.kt` | YES: `FlorisBottomSheetHostUiView.Content` | Native modal scrim/layout | Move `isBottomSheetShowing` to state helper |
| `smartbar/CandidatesRow.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `smartbar/InlineSuggestionsUi.kt` | YES: `RustKeyboardLayout` inline reservation and style cache | Android InlineContentViews must remain | Replace Compose Row/scroll host with Android host; style cache is used by service |
| `smartbar/Smartbar.kt` | No: bypassed by `FlorisImeUi.ImeUi` prototype branch | Native scene/panels in `render`, dispatched by `RustKeyboardView` | Fallback retained until S24 acceptance |
| `smartbar/quickaction/QuickAction.kt` | No: `ActionButton` only used by fallback | RendererSmartbar/RendererPanels | Keep sealed action model and platform dispatch; split composable |
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
| `dictation/DictationPreview.kt` | YES: RustKeyboardLayout overlays preview | Package 8 native text | Port start ellipsis and semantics |
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
| `SnyggButton.kt` | YES: extract action | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggChip.kt` | No: fallback/settings-only primitive | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggColumn.kt` | YES: subtype/one-handed | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggIcon.kt` | YES: subtype/one-handed | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggIconButton.kt` | YES: one-handed | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggListItem.kt` | YES: subtype | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggPalette.kt` | YES: native palette transport | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggRow.kt` | YES: subtype/extract/dictation | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggSpacer.kt` | No: fallback/settings-only primitive | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggSurfaceView.kt` | YES: root background image surface | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggText.kt` | YES: subtype/extract/dictation | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |
| `SnyggUi.kt` | YES: theme locals/queries | Native theme/scene; Android images/insets | Shared settings or fallback callers must be split |

## Modal, transient and platform surfaces

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

## Implementation and verification

Pending. Prototype default remains false; no phone acceptance or Compose deletion is
claimed by this inventory. Later sections record actual checks, including missing evidence.
