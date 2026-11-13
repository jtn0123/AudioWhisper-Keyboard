/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.ime.keyboard

import android.content.Context
import android.view.KeyEvent
import android.widget.Toast
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.setValue
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.gif.GifSearchKeys
import com.jtn0123.audiowhisper.suggest.AutoCorrectionUndo
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.editorInstance
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.editor.EditorContent
import dev.patrickgold.florisboard.ime.input.InputEventDispatcher
import dev.patrickgold.florisboard.ime.input.InputKeyEventReceiver
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.nlp.ClipboardSuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.WordSuggestionCandidate
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.gestures.keyData
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyboardCache
import dev.patrickgold.florisboard.lib.titlecase
import dev.patrickgold.florisboard.lib.uppercase
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import java.lang.ref.WeakReference
import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import org.florisboard.lib.kotlin.collectIn
import org.florisboard.lib.kotlin.collectLatestIn

class KeyboardManager(context: Context) : InputKeyEventReceiver {
    internal val prefs by FlorisPreferenceStore
    internal val appContext by context.appContext()
    internal val clipboardManager by context.clipboardManager()
    internal val editorInstance by context.editorInstance()
    private val extensionManager by context.extensionManager()
    internal val nlpManager by context.nlpManager()
    internal val subtypeManager by context.subtypeManager()

    internal val scope = CoroutineScope(Dispatchers.Default + SupervisorJob())
    val layoutManager = LayoutManager(context)
    private val keyboardCache = TextKeyboardCache()

    val resources = KeyboardManagerResources(context, scope)
    val activeState = ObservableKeyboardState.new()
    var smartbarVisibleDynamicActionsCount by mutableIntStateOf(0)
    internal var lastToastReference = WeakReference<Toast>(null)

    /** AudioWhisper: the auto-correction a backspace typed straight after it would undo. */
    internal var autoCorrectionUndo: AutoCorrectionUndo? = null

    private val activeEvaluatorGuard = Mutex(locked = false)
    private var activeEvaluatorVersion = AtomicInteger(0)
    private val _activeEvaluator = MutableStateFlow<ComputingEvaluator>(DefaultComputingEvaluator)
    val activeEvaluator get() = _activeEvaluator.asStateFlow()
    private val _activeSmartbarEvaluator = MutableStateFlow<ComputingEvaluator>(DefaultComputingEvaluator)
    val activeSmartbarEvaluator get() = _activeSmartbarEvaluator.asStateFlow()
    private val _lastCharactersEvaluator = MutableStateFlow<ComputingEvaluator>(DefaultComputingEvaluator)
    val lastCharactersEvaluator get() = _lastCharactersEvaluator.asStateFlow()

    val inputEventDispatcher = InputEventDispatcher.new(
        repeatableKeyCodes = intArrayOf(
            KeyCode.ARROW_DOWN,
            KeyCode.ARROW_LEFT,
            KeyCode.ARROW_RIGHT,
            KeyCode.ARROW_UP,
            KeyCode.DELETE,
            KeyCode.FORWARD_DELETE,
            KeyCode.UNDO,
            KeyCode.REDO,
        )
    ).also { it.keyEventReceiver = this }

    init {
        scope.launch(Dispatchers.Main.immediate) {
            resources.anyChanged.observeForever {
                updateActiveEvaluators {
                    keyboardCache.clear()
                }
            }
            prefs.keyboard.numberRow.asFlow().collectLatestIn(scope) {
                updateActiveEvaluators {
                    keyboardCache.clear(KeyboardMode.CHARACTERS)
                }
            }
            prefs.keyboard.hintedNumberRowEnabled.asFlow().collectLatestIn(scope) {
                updateActiveEvaluators()
            }
            prefs.keyboard.hintedSymbolsEnabled.asFlow().collectLatestIn(scope) {
                updateActiveEvaluators()
            }
            prefs.keyboard.utilityKeyEnabled.asFlow().collectLatestIn(scope) {
                updateActiveEvaluators()
            }
            prefs.keyboard.utilityKeyAction.asFlow().collectLatestIn(scope) {
                updateActiveEvaluators()
            }
            activeState.collectLatestIn(scope) {
                updateActiveEvaluators()
            }
            subtypeManager.subtypesFlow.collectLatestIn(scope) {
                updateActiveEvaluators()
            }
            subtypeManager.activeSubtypeFlow.collectLatestIn(scope) {
                reevaluateInputShiftState()
                updateActiveEvaluators()
                editorInstance.refreshComposing()
                resetSuggestions(editorInstance.activeContent)
            }
            clipboardManager.primaryClipFlow.collectLatestIn(scope) {
                updateActiveEvaluators()
            }
            editorInstance.activeContentFlow.collectIn(scope) { content ->
                resetSuggestions(content)
            }

        }
    }

    private fun updateActiveEvaluators(action: () -> Unit = { }) = scope.launch {
        activeEvaluatorGuard.withLock {
            action()
            val editorInfo = editorInstance.activeInfo
            val state = activeState.snapshot()
            val subtype = subtypeManager.activeSubtype
            val mode = state.keyboardMode
            // We need to reset the snapshot input shift state for non-character layouts, because the shift mechanic
            // only makes sense for the character layouts.
            if (mode != KeyboardMode.CHARACTERS) {
                state.inputShiftState = InputShiftState.UNSHIFTED
            }
            val computedKeyboard = keyboardCache.getOrElseAsync(mode, subtype) {
                layoutManager.computeKeyboardAsync(
                    keyboardMode = mode,
                    subtype = subtype,
                ).await()
            }
            val computingEvaluator = ComputingEvaluatorImpl(
                manager = this@KeyboardManager,
                version = activeEvaluatorVersion.getAndAdd(1),
                keyboard = computedKeyboard,
                editorInfo = editorInfo,
                state = state,
                subtype = subtype,
            )
            for (key in computedKeyboard.keys()) {
                key.compute(computingEvaluator)
                key.computeLabelsAndDrawables(computingEvaluator)
            }
            _activeEvaluator.value = computingEvaluator
            _activeSmartbarEvaluator.value = computingEvaluator.asSmartbarQuickActionsEvaluator()
            if (computedKeyboard.mode == KeyboardMode.CHARACTERS) {
                _lastCharactersEvaluator.value = computingEvaluator
            }
        }
    }

    fun reevaluateInputShiftState() {
        // AudioWhisper: the decision is in Rust (ShiftLogic); setting an unchanged state does nothing.
        activeState.inputShiftState = ShiftLogic.reevaluate(
            activeState.inputShiftState,
            shiftPressed = inputEventDispatcher.isPressed(KeyCode.SHIFT),
            autoCapitalization = prefs.correction.autoCapitalization.get(),
            localeCapitalizes = subtypeManager.activeSubtype.primaryLocale.supportsCapitalization,
            capsMode = editorInstance.activeCursorCapsMode,
        )
    }

    fun resetSuggestions(content: EditorContent) {
        if (!(activeState.isComposingEnabled || nlpManager.isSuggestionOn())) {
            nlpManager.clearSuggestions()
            return
        }
        nlpManager.suggest(subtypeManager.activeSubtype, content)
    }

    /**
     * @return If the language switch should be shown.
     */
    fun shouldShowLanguageSwitch(): Boolean {
        return subtypeManager.subtypes.size > 1
    }

    suspend fun toggleOneHandedMode() {
        prefs.keyboard.oneHandedModeEnabled.set(!prefs.keyboard.oneHandedModeEnabled.get())
    }

    fun executeSwipeAction(swipeAction: SwipeAction) {
        swipeAction.keyData(activeState.keyboardMode)?.let { inputEventDispatcher.sendDownUp(it) }
    }

    fun commitCandidate(candidate: SuggestionCandidate) {
        scope.launch {
            candidate.sourceProvider?.notifySuggestionAccepted(subtypeManager.activeSubtype, candidate)
        }
        when (candidate) {
            is ClipboardSuggestionCandidate -> editorInstance.commitClipboardItem(candidate.clipboardItem)
            else -> {
                // AudioWhisper: a tapped (or auto-corrected) word is a finished word.
                if (candidate is WordSuggestionCandidate) {
                    nlpManager.learnCandidate(candidate.text.toString(), editorInstance.activeContent)
                }
                editorInstance.commitCompletion(candidate)
            }
        }
    }

    fun commitGesture(word: String) {
        if (isTypingIntoGifSearch) return GifSearch.typeWord(fixCase(word))
        nlpManager.learnSwipedWord(fixCase(word), editorInstance.activeContent)
        editorInstance.commitGesture(fixCase(word))
    }

    /**
     * AudioWhisper: the GIF search box is open, so the app's field behind it must not change.
     * Keys go through [typeIntoGifSearch]; the touch code checks this before its swipes.
     */
    val isTypingIntoGifSearch: Boolean get() = GifSearch.typing.value

    /** AudioWhisper: [data] is a key the GIF search box takes or blocks, not one for the keyboard. */
    internal fun heldByGifSearch(data: KeyData): Boolean =
        isTypingIntoGifSearch && GifSearchKeys.route(data.code, data.type) != GifSearchKeys.Route.KEYBOARD

    /**
     * AudioWhisper: while the GIF search field is open, typing goes into it and keys that would
     * edit the app do nothing. Returns false for keys it leaves to the keyboard (shift, layouts).
     */
    internal fun typeIntoGifSearch(data: KeyData): Boolean {
        when (GifSearchKeys.route(data.code, data.type)) {
            GifSearchKeys.Route.SEARCH -> editGifSearch(data)
            GifSearchKeys.Route.IGNORE -> Unit
            GifSearchKeys.Route.KEYBOARD -> {
                if (data.code == KeyCode.IME_UI_MODE_MEDIA || data.code == KeyCode.IME_UI_MODE_CLIPBOARD) {
                    GifSearch.stopTyping()
                }
                return false
            }
        }
        return true
    }

    internal fun editGifSearch(data: KeyData) {
        when (data.code) {
            KeyCode.DELETE -> GifSearch.deleteLast()
            KeyCode.DELETE_WORD -> GifSearch.deleteWord()
            KeyCode.SPACE, KeyCode.CJK_SPACE -> GifSearch.type(" ")
            KeyCode.ENTER -> {
                GifSearch.submit()
                activeState.imeUiMode = ImeUiMode.MEDIA
            }
            else -> {
                GifSearch.type(data.asString(isForDisplay = false))
                activeState.inputShiftState = ShiftLogic.keyTyped(
                    activeState.inputShiftState, shiftPressed = inputEventDispatcher.isPressed(KeyCode.SHIFT),
                )
            }
        }
    }

    /**
     * Changes a word to the current case.
     * eg if [KeyboardState.isUppercase] is true, abc -> ABC
     *    if [caps]     is true, abc -> Abc
     *    otherwise            , abc -> abc
     */
    fun fixCase(word: String): String {
        return when(activeState.inputShiftState) {
            InputShiftState.CAPS_LOCK -> {
                word.uppercase(subtypeManager.activeSubtype.primaryLocale)
            }
            InputShiftState.SHIFTED_MANUAL, InputShiftState.SHIFTED_AUTOMATIC -> {
                word.titlecase(subtypeManager.activeSubtype.primaryLocale)
            }
            else -> word
        }
    }

    /**
     * Handles [KeyCode] arrow and move events, behaves differently depending on text selection.
     */
    override fun onInputKeyDown(data: KeyData) = handleInputKeyDown(data)

    override fun onInputKeyUp(data: KeyData) = handleInputKeyUp(data)

    override fun onInputKeyCancel(data: KeyData) = handleInputKeyCancel(data)

    override fun onInputKeyRepeat(data: KeyData) = handleInputKeyRepeat(data)

    fun onHardwareKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        when (keyCode) {
            KeyEvent.KEYCODE_SPACE -> {
                handleHardwareKeyboardSpace()
                return true
            }
            KeyEvent.KEYCODE_ENTER -> {
                handleEnter()
                return true
            }
            KeyEvent.KEYCODE_SHIFT_LEFT, KeyEvent.KEYCODE_SHIFT_RIGHT -> {
                inputEventDispatcher.sendDown(TextKeyData.SHIFT)
                return true
            }
            else -> return false
        }
    }

    fun onHardwareKeyUp(keyCode: Int, event: KeyEvent?): Boolean {
        when (keyCode) {
            KeyEvent.KEYCODE_SHIFT_LEFT, KeyEvent.KEYCODE_SHIFT_RIGHT -> {
                inputEventDispatcher.sendUp(TextKeyData.SHIFT)
                return true
            }
            else -> return false
        }
    }

}
