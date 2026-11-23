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

package dev.patrickgold.florisboard

import android.content.Context
import android.content.Intent
import android.content.res.Configuration
import android.inputmethodservice.ExtractEditText
import android.os.Bundle
import android.view.Gravity
import android.view.KeyEvent
import android.util.Size
import android.view.View
import android.view.ViewGroup
import android.view.WindowManager
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InlineSuggestionsResponse
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import android.widget.Toast
import androidx.compose.foundation.layout.padding
import androidx.core.view.WindowCompat
import androidx.lifecycle.lifecycleScope
import com.jtn0123.audiowhisper.dictation.Dictation
import com.jtn0123.audiowhisper.dictation.DictationFields
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.render.ImeExtractView
import com.jtn0123.audiowhisper.render.ImeRootView
import com.jtn0123.audiowhisper.render.ImeSubtypeSheet
import com.jtn0123.audiowhisper.rust.DictationRules
import dev.patrickgold.florisboard.app.FlorisAppActivity
import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.core.isSubtypeSelectionShowing
import dev.patrickgold.florisboard.ime.editor.EditorRange
import dev.patrickgold.florisboard.ime.editor.FlorisEditorInfo
import dev.patrickgold.florisboard.ime.input.InputFeedbackController
import dev.patrickgold.florisboard.ime.keyboard.FlorisImeSizing
import dev.patrickgold.florisboard.ime.landscapeinput.LandscapeInputUiMode
import dev.patrickgold.florisboard.ime.lifecycle.LifecycleInputMethodService
import dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofill
import dev.patrickgold.florisboard.ime.sheet.isBottomSheetShowing
import dev.patrickgold.florisboard.ime.smartbar.ExtendedActionsPlacement
import dev.patrickgold.florisboard.ime.smartbar.SmartbarLayout
import dev.patrickgold.florisboard.ime.theme.FlorisImeTheme
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogError
import dev.patrickgold.florisboard.lib.devtools.flogInfo
import dev.patrickgold.florisboard.lib.devtools.flogWarning
import dev.patrickgold.florisboard.lib.util.ViewUtils
import dev.patrickgold.florisboard.lib.util.launchActivity
import java.lang.ref.WeakReference
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.update
import org.florisboard.lib.android.AndroidInternalR
import org.florisboard.lib.android.systemServiceOrNull
import org.florisboard.lib.kotlin.collectIn

/**
 * Global weak reference for the [FlorisImeService] class. This is needed as certain actions (request hide, switch to
 * another input method, getting the editor instance / input connection, etc.) can only be performed by an IME
 * service class and no context-bound managers. This reference is exclusively used by the companion helper methods
 * of [FlorisImeService], which provide a safe and memory-leak-free way of performing certain actions on the Floris
 * input method service instance.
 */
private var FlorisImeServiceReference = WeakReference<FlorisImeService?>(null)

/**
 * Core class responsible for linking together all managers and UI compose-ables to provide an IME service. Sets
 * up the window and context to be lifecycle-aware, so LiveData and Jetpack Compose can be used without issues.
 */
class FlorisImeService : LifecycleInputMethodService() {
    companion object {
        /** Characters dictation reads before the cursor to space and capitalise a phrase. */
        private const val DICTATION_LOOK_BACK = 48

        fun currentInputConnection(): InputConnection? {
            return FlorisImeServiceReference.get()?.currentInputConnection
        }

        fun inputFeedbackController(): InputFeedbackController? {
            return FlorisImeServiceReference.get()?.inputFeedbackController
        }

        /**
         * Hides the IME and launches [FlorisAppActivity].
         */
        fun launchSettings() {
            val ims = FlorisImeServiceReference.get() ?: return
            ims.requestHideSelf(0)
            ims.launchActivity(FlorisAppActivity::class) {
                it.flags = Intent.FLAG_ACTIVITY_NEW_TASK or
                    Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED or
                    Intent.FLAG_ACTIVITY_CLEAR_TOP
            }
        }

        fun showUi() {
            val ims = FlorisImeServiceReference.get() ?: return
            ims.requestShowSelf(0)
        }

        fun hideUi() {
            val ims = FlorisImeServiceReference.get() ?: return
            ims.requestHideSelf(0)
            FlorisImeServiceReference.get()?.requestHideSelf(0)
        }

        fun switchToPrevInputMethod(): Boolean {
            val ims = FlorisImeServiceReference.get() ?: return false
            val imm = ims.systemServiceOrNull(InputMethodManager::class)
            try {
                return ims.switchToPreviousInputMethod()
            } catch (e: Exception) {
                flogError { "Unable to switch to the previous IME" }
                imm?.showInputMethodPicker()
            }
            return false
        }

        fun switchToNextInputMethod(): Boolean {
            val ims = FlorisImeServiceReference.get() ?: return false
            val imm = ims.systemServiceOrNull(InputMethodManager::class)
            try {
                return ims.switchToNextInputMethod(false)
            } catch (e: Exception) {
                flogError { "Unable to switch to the next IME" }
                imm?.showInputMethodPicker()
            }
            return false
        }

        /** The mic key: start or finish dictating, without leaving the keyboard. */
        fun pressMic() {
            FlorisImeServiceReference.get()?.dictation?.pressMic()
        }

        fun releaseMic() {
            FlorisImeServiceReference.get()?.dictation?.releaseMic()
        }

        /** Stops listening (what was said is still typed), as reading aloud starts. */
        fun stopDictating() {
            FlorisImeServiceReference.get()?.dictation?.stopListening()
        }
    }

    // Lazy, like FlorisAppActivity's: building the model reads native defaults, which a
    // service constructed without its engine must not do (the system would crash-loop it).
    internal val prefs: FlorisPreferenceModel by lazy { val model by FlorisPreferenceStore; model }
    internal val editorInstance by editorInstance()
    internal val keyboardManager by keyboardManager()
    private val nlpManager by nlpManager()
    private val subtypeManager by subtypeManager()
    internal val themeManager by themeManager()

    private val activeState get() = keyboardManager.activeState
    private var inputWindowView: View? = null
    /** What covers the whole window for the subtype sheet, Compose's or the plain-View root's. */
    private var sheetView: View? = null
    internal var inputViewSize = Size(0, 0)
    internal val inputFeedbackController by lazy { InputFeedbackController.new(this) }
    private var isWindowShown: Boolean = false
    // The Compose root collects these three; the plain-View root reads them when refreshed.
    internal val fullscreenUiMode = MutableStateFlow(false)
    internal val extractUiShown = MutableStateFlow(false)
    internal val localizedContext = MutableStateFlow<Context>(this)
    internal val resourcesContext: Context get() = localizedContext.value

    private var dictation: Dictation? = null

    /**
     * False when the native engine failed to load. FlorisApplication then skipped its setup,
     * and every manager here reaches the engine, so the service touches none of them: it
     * shows [PlatformCrashScreen.inputView] and leaves the rest to InputMethodService.
     */
    private val engine = com.jtn0123.audiowhisper.rust.NativeRuntime.load()

    init {
        setTheme(R.style.FlorisImeTheme)
    }
    override fun onCreate() {
        super.onCreate()
        if (!engine) return
        FlorisImeServiceReference = WeakReference(this)
        WindowCompat.setDecorFitsSystemWindows(window.window!!, false)
        dictation = Dictation(this, dictationEditor)
        GifSearch.attach(filesDir)
        subtypeManager.activeSubtypeFlow.collectIn(lifecycleScope) { subtype ->
            val config = Configuration(resources.configuration)
            if (prefs.localization.displayKeyboardLabelsInSubtypeLanguage.get()) {
                config.setLocale(subtype.primaryLocale.base)
            }
            localizedContext.value = createConfigurationContext(config)
            (inputWindowView as? ImeRootView)?.refresh()
        }
        prefs.localization.displayKeyboardLabelsInSubtypeLanguage.asFlow().collectIn(lifecycleScope) { shouldSync ->
            val config = Configuration(resources.configuration)
            if (shouldSync) {
                config.setLocale(subtypeManager.activeSubtype.primaryLocale.base)
            }
            localizedContext.value = createConfigurationContext(config)
            (inputWindowView as? ImeRootView)?.refresh()
        }
        // AudioWhisper: the renderer setting picks the root; switching it swaps the root in place.
        prefs.keyboard.rustRendererPrototype.asFlow().collectIn(lifecycleScope) { rust ->
            if (inputWindowView == null || (inputWindowView is ImeRootView) == rust) return@collectIn
            setInputView(onCreateInputView())
            onCreateExtractTextView()?.let(::setExtractView)
        }
        prefs.physicalKeyboard.showOnScreenKeyboard.asFlow().collectIn(lifecycleScope) {
            updateInputViewShown()
        }
    }
    override fun onCreateInputView(): View {
        if (!engine) return com.jtn0123.audiowhisper.settingsui.PlatformCrashScreen.inputView(this)
        val timingStart = com.jtn0123.audiowhisper.render.RendererHostTiming.start()
        super.installViewTreeOwners()
        // AudioWhisper: with the Rust renderer on, plain Views and no composition.
        val rust = prefs.keyboard.rustRendererPrototype.get()
        val content = window.window!!.findViewById<ViewGroup>(android.R.id.content)
        sheetView?.let(content::removeView)
        sheetView = (if (rust) ImeSubtypeSheet(this) else FlorisBottomSheetHostUiView(this)).also(content::addView)
        val view = if (rust) ImeRootView(this) else ComposeInputView(this)
        inputWindowView = view
        return com.jtn0123.audiowhisper.render.RendererHostTiming.created(view, timingStart, rust)
    }
    override fun onCreateCandidatesView(): View? {
        // Disable the default candidates view
        return null
    }
    override fun onCreateExtractTextView(): View? {
        if (!engine) return super.onCreateExtractTextView()
        super.installViewTreeOwners()
        val defaultExtractView = super.onCreateExtractTextView()
        val rust = prefs.keyboard.rustRendererPrototype.get()
        if (defaultExtractView == null || defaultExtractView !is ViewGroup) {
            return if (rust) ImeExtractView(this) else ComposeExtractedLandscapeInputView(this, null)
        }
        val extractEditText = defaultExtractView.findViewById<ExtractEditText>(android.R.id.inputExtractEditText)
        (extractEditText?.parent as? ViewGroup)?.removeView(extractEditText)
        defaultExtractView.apply {
            removeAllViews()
            addView(if (rust) ImeExtractView(this@FlorisImeService, extractEditText ?: ExtractEditText(context))
                else ComposeExtractedLandscapeInputView(this@FlorisImeService, extractEditText))
        }
        return defaultExtractView
    }
    override fun onConfigurationChanged(newConfig: Configuration) {
        super.onConfigurationChanged(newConfig)
        if (!engine) return
        themeManager.configurationChangeCounter.update { it + 1 }
    }
    override fun onDestroy() {
        dictation?.release()
        dictation = null
        super.onDestroy()
        FlorisImeServiceReference = WeakReference(null)
        inputWindowView = null
    }
    override fun onStartInput(info: EditorInfo?, restarting: Boolean) {
        super.onStartInput(info, restarting)
        if (!engine) return
        // AudioWhisper: input started again, perhaps in another field. Chrome reports a move
        // between a page's fields as a restart of the same one, with nothing to tell them
        // apart, so any start ends a dictation rather than let it type into the wrong field (a
        // password, even). Typing and dictating don't restart input.
        dictation?.abandon()
        if (info == null) return
        val editorInfo = FlorisEditorInfo.wrap(info)
        editorInstance.handleStartInput(editorInfo)
    }
    override fun onStartInputView(info: EditorInfo?, restarting: Boolean) {
        super.onStartInputView(info, restarting)
        if (info == null || !engine) return
        val editorInfo = FlorisEditorInfo.wrap(info)
        // AudioWhisper: a new field starts on its own text, not a half-typed GIF search.
        if (!restarting) GifSearch.stopTyping()
        activeState.batchEdit {
            activeState.rawValue = com.jtn0123.audiowhisper.rust.InputRules.control(longArrayOf(
                com.jtn0123.audiowhisper.rust.InputProtocol.CONTROL_START.toLong(), activeState.rawValue.toLong(),
                if(prefs.clipboard.historyHideOnNextTextField.get()) 1 else 0,
                editorInfo.initialSelection.start.toLong(), editorInfo.initialSelection.end.toLong()))[0].toULong()
            editorInstance.handleStartInputView(editorInfo, isRestart = restarting)
        }
    }
    override fun onEvaluateInputViewShown(): Boolean {
        val config = resources.configuration
        return super.onEvaluateInputViewShown()
            || config.keyboard == Configuration.KEYBOARD_NOKEYS
            || engine && prefs.physicalKeyboard.showOnScreenKeyboard.get()
    }
    override fun onUpdateExtractedText(token: Int, text: android.view.inputmethod.ExtractedText) {
        super.onUpdateExtractedText(token, text)
        if (!engine) return
        editorInstance.handleExtractedTextUpdate(text)
    }
    override fun onUpdateSelection(
        oldSelStart: Int,
        oldSelEnd: Int,
        newSelStart: Int,
        newSelEnd: Int,
        candidatesStart: Int,
        candidatesEnd: Int,
    ) {
        flogInfo { "old={start=$oldSelStart,end=$oldSelEnd} new={start=$newSelStart,end=$newSelEnd} composing={start=$candidatesStart,end=$candidatesEnd}" }
        super.onUpdateSelection(oldSelStart, oldSelEnd, newSelStart, newSelEnd, candidatesStart, candidatesEnd)
        if (!engine) return
        activeState.batchEdit {
            activeState.isSelectionMode = (newSelEnd - newSelStart) != 0
            editorInstance.handleSelectionUpdate(
                oldSelection = EditorRange.normalized(oldSelStart, oldSelEnd),
                newSelection = EditorRange.normalized(newSelStart, newSelEnd),
                composing = EditorRange.normalized(candidatesStart, candidatesEnd),
            )
        }
    }
    override fun onFinishInputView(finishingInput: Boolean) {
        flogInfo { "finishing=$finishingInput" }
        super.onFinishInputView(finishingInput)
        if (!engine) return
        editorInstance.handleFinishInputView()
    }
    override fun onFinishInput() {
        flogInfo { "(no args)" }
        super.onFinishInput()
        if (!engine) return
        editorInstance.handleFinishInput()
        NlpInlineAutofill.clearInlineSuggestions()
        // AudioWhisper: the field is gone (another app, the screen off), so a phrase still
        // being transcribed has nowhere to go.
        dictation?.abandon()
    }
    override fun onWindowShown() {
        super.onWindowShown()
        if (!engine) return
        // AudioWhisper: hiding told the input view its window was gone; showing again before
        // the window really hid tells it nothing, and the Rust keyboard drew no keys.
        (inputWindowView as? ComposeInputView)?.windowShown()
        (inputWindowView as? ImeRootView)?.windowShown()
        if (isWindowShown) {
            flogWarning(LogTopic.IMS_EVENTS) { "Ignoring (is already shown)" }
            return
        } else {
            flogInfo(LogTopic.IMS_EVENTS)
        }
        isWindowShown = true
        inputFeedbackController.updateSystemPrefsState()
        // AudioWhisper: a word added to the personal dictionary counts from the next keystroke.
        nlpManager.refreshPersonalWords()
        dictation?.keyboardVisible(true)
    }
    override fun onWindowHidden() {
        super.onWindowHidden()
        if (!engine) return
        if (!isWindowShown) {
            flogWarning(LogTopic.IMS_EVENTS) { "Ignoring (is already hidden)" }
            return
        } else {
            flogInfo(LogTopic.IMS_EVENTS)
        }
        isWindowShown = false
        // Minimised (back, the hide key): stop listening, but still type what was said; the
        // field is still connected. Leaving the field drops the rest (onFinishInput).
        dictation?.stopListening()
        dictation?.keyboardVisible(false)
        GifSearch.reset()
        activeState.batchEdit {
            activeState.rawValue = com.jtn0123.audiowhisper.rust.InputRules.control(longArrayOf(
                com.jtn0123.audiowhisper.rust.InputProtocol.CONTROL_FINISH.toLong(), activeState.rawValue.toLong()))[0].toULong()
        }
    }

    /** Dictation types through the editor, which keeps its view of the text in step. */
    private val dictationEditor = object : Dictation.Editor {
        override fun allowsDictation(): Boolean = DictationFields.allows(editorInstance.activeInfo.inputAttributes)

        override fun textBeforeCursor(): CharSequence? =
            currentInputConnection?.getTextBeforeCursor(DICTATION_LOOK_BACK, 0)

        override fun type(text: String, replacing: Int) {
            editorInstance.commitDictation(text, replacing)
        }

        override fun isName(word: String): Boolean = nlpManager.isName(word)

        override fun tidy(phrase: String): String =
            DictationRules.format(phrase, prefs.dictation.vocabulary.get(), prefs.dictation.removeFillers.get())

        override fun undoLastPhrase(): Boolean = editorInstance.undoDictation()

        override fun problem(message: Int) {
            Toast.makeText(this@FlorisImeService, message, Toast.LENGTH_SHORT).show()
        }

        override fun tick() {
            inputFeedbackController.keyPress()
        }
    }
    override fun onEvaluateFullscreenMode(): Boolean {
        val config = resources.configuration
        if (!engine || config.orientation != Configuration.ORIENTATION_LANDSCAPE) {
            return false
        }
        return when (prefs.keyboard.landscapeInputUiMode.get()) {
            LandscapeInputUiMode.DYNAMICALLY_SHOW -> super.onEvaluateFullscreenMode()
            LandscapeInputUiMode.NEVER_SHOW -> false
            LandscapeInputUiMode.ALWAYS_SHOW -> true
        }
    }
    override fun updateFullscreenMode() {
        super.updateFullscreenMode()
        if (!engine) return
        fullscreenUiMode.value = isFullscreenMode
        updateSoftInputWindowLayoutParameters()
        (inputWindowView as? ImeRootView)?.refresh()
    }
    override fun onUpdateExtractingVisibility(info: EditorInfo?) {
        if (!engine) return super.onUpdateExtractingVisibility(info)
        if (info != null) {
            editorInstance.handleStartInputView(FlorisEditorInfo.wrap(info), isRestart = true)
        }
        when (prefs.keyboard.landscapeInputUiMode.get()) {
            LandscapeInputUiMode.DYNAMICALLY_SHOW -> super.onUpdateExtractingVisibility(info)
            LandscapeInputUiMode.NEVER_SHOW -> isExtractViewShown = false
            LandscapeInputUiMode.ALWAYS_SHOW -> isExtractViewShown = true
        }
    }
    override fun setExtractViewShown(shown: Boolean) {
        super.setExtractViewShown(shown)
        extractUiShown.value = shown
        (inputWindowView as? ImeRootView)?.refresh()
    }
    override fun onCreateInlineSuggestionsRequest(uiExtras: Bundle) = if (engine) createInlineSuggestionsRequest() else null
    override fun onInlineSuggestionsResponse(response: InlineSuggestionsResponse) = engine && receiveInlineSuggestions(response)
    override fun onComputeInsets(outInsets: Insets?) {
        super.onComputeInsets(outInsets)
        if (outInsets == null || !engine) return

        val inputWindowView = inputWindowView ?: return
        // TODO: Check also if the keyboard is currently suppressed by a hardware keyboard
        if (!isInputViewShown) {
            outInsets.contentTopInsets = inputWindowView.height
            outInsets.visibleTopInsets = inputWindowView.height
            return
        }

        val visibleTopY = inputWindowView.height - inputViewSize.height
        val needAdditionalOverlay =
            prefs.smartbar.enabled.get() &&
                prefs.smartbar.layout.get() == SmartbarLayout.SUGGESTIONS_ACTIONS_EXTENDED &&
                prefs.smartbar.extendedActionsExpanded.get() &&
                prefs.smartbar.extendedActionsPlacement.get() == ExtendedActionsPlacement.OVERLAY_APP_UI &&
                keyboardManager.activeState.imeUiMode == ImeUiMode.TEXT

        outInsets.contentTopInsets = visibleTopY
        outInsets.visibleTopInsets = visibleTopY
        outInsets.touchableInsets = Insets.TOUCHABLE_INSETS_REGION
        val left = 0
        val top = if (keyboardManager.activeState.isBottomSheetShowing() || keyboardManager.activeState.isSubtypeSelectionShowing()) {
            0
        } else {
            visibleTopY - if (needAdditionalOverlay) FlorisImeSizing.Static.smartbarHeightPx else 0
        }
        val right = inputViewSize.width
        val bottom = inputWindowView.height
        outInsets.touchableRegion.set(left, top, right, bottom)
    }

    internal fun updateSoftInputWindowLayoutParameters() {
        val w = window?.window ?: return
        // TODO: Verify that this doesn't give us a padding problem
        WindowCompat.setDecorFitsSystemWindows(w, false)
        ViewUtils.updateLayoutHeightOf(w, WindowManager.LayoutParams.MATCH_PARENT)
        val layoutHeight = if (fullscreenUiMode.value) {
            WindowManager.LayoutParams.WRAP_CONTENT
        } else {
            WindowManager.LayoutParams.MATCH_PARENT
        }
        val inputArea = w.findViewById<View>(android.R.id.inputArea) ?: return
        ViewUtils.updateLayoutHeightOf(inputArea, layoutHeight)
        ViewUtils.updateLayoutGravityOf(inputArea, Gravity.BOTTOM)
        val inputWindowView = inputWindowView ?: return
        ViewUtils.updateLayoutHeightOf(inputWindowView, layoutHeight)
    }
    override fun getTextForImeAction(imeOptions: Int): String? {
        return try {
            when (imeOptions and EditorInfo.IME_MASK_ACTION) {
                EditorInfo.IME_ACTION_NONE -> null
                EditorInfo.IME_ACTION_GO -> resourcesContext.getString(AndroidInternalR.string.ime_action_go)
                EditorInfo.IME_ACTION_SEARCH -> resourcesContext.getString(AndroidInternalR.string.ime_action_search)
                EditorInfo.IME_ACTION_SEND -> resourcesContext.getString(AndroidInternalR.string.ime_action_send)
                EditorInfo.IME_ACTION_NEXT -> resourcesContext.getString(AndroidInternalR.string.ime_action_next)
                EditorInfo.IME_ACTION_DONE -> resourcesContext.getString(AndroidInternalR.string.ime_action_done)
                EditorInfo.IME_ACTION_PREVIOUS -> resourcesContext.getString(AndroidInternalR.string.ime_action_previous)
                else -> resourcesContext.getString(AndroidInternalR.string.ime_action_default)
            }
        } catch (_: Throwable) {
            super.getTextForImeAction(imeOptions)?.toString()
        }
    }
    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        return engine && keyboardManager.onHardwareKeyDown(keyCode, event) || super.onKeyDown(keyCode, event)
    }
    override fun onKeyUp(keyCode: Int, event: KeyEvent?): Boolean {
        return engine && keyboardManager.onHardwareKeyUp(keyCode, event) || super.onKeyUp(keyCode, event)
    }

}
