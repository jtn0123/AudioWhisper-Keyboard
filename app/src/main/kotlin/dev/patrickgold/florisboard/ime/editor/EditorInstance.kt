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

package dev.patrickgold.florisboard.ime.editor

import android.content.ClipDescription
import android.content.ContentUris
import android.content.Context
import android.net.Uri
import android.view.KeyEvent
import androidx.core.view.inputmethod.EditorInfoCompat
import androidx.core.view.inputmethod.InputConnectionCompat
import androidx.core.view.inputmethod.InputContentInfoCompat
import com.jtn0123.audiowhisper.dictation.DictationUndo
import com.jtn0123.audiowhisper.rust.SpacingRules
import com.jtn0123.audiowhisper.suggest.AutoCorrectionUndo
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFileStorage
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.keyboard.IncognitoMode
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.nlp.PunctuationRule
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.text.composing.Appender
import dev.patrickgold.florisboard.ime.text.composing.Composer
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.florisboard.lib.android.showShortToast

class EditorInstance(context: Context) : AbstractEditorInstance(context) {
    companion object {
        private const val SPACE = " "
        private const val GIF_MIME_TYPE = "image/gif"
    }

    private val prefs by FlorisPreferenceStore
    private val appContext by context.appContext()
    private val clipboardManager by context.clipboardManager()
    private val keyboardManager by context.keyboardManager()
    private val subtypeManager by context.subtypeManager()
    private val nlpManager by context.nlpManager()

    private val activeState get() = keyboardManager.activeState
    val autoSpace = AutoSpaceState()
    val phantomSpace = PhantomSpaceState()
    val massSelection = MassSelectionState()

    override val isSpacePending: Boolean
        get() = phantomSpace.isActive

    /** AudioWhisper: the phrase dictation typed last, for [undoDictation]. */
    private var dictationUndo: DictationUndo? = null

    private fun currentInputConnection() = FlorisImeService.currentInputConnection()

    override fun handleStartInputView(editorInfo: FlorisEditorInfo, isRestart: Boolean) {
        if (!prefs.correction.rememberCapsLockState.get()) {
            activeState.inputShiftState = InputShiftState.UNSHIFTED
        }
        activeState.isActionsOverflowVisible = false
        activeState.isActionsEditorVisible = false
        super.handleStartInputView(editorInfo, isRestart)
        // AudioWhisper: editor positions the word learner kept, and tap positions, belong to the previous field.
        nlpManager.resetLearning()
        val keyboardMode = when (editorInfo.inputAttributes.type) {
            InputAttributes.Type.NUMBER -> {
                activeState.keyVariation = KeyVariation.NORMAL
                KeyboardMode.NUMERIC
            }
            InputAttributes.Type.PHONE -> {
                activeState.keyVariation = KeyVariation.NORMAL
                KeyboardMode.PHONE
            }
            InputAttributes.Type.TEXT -> {
                activeState.keyVariation = when (editorInfo.inputAttributes.variation) {
                    InputAttributes.Variation.EMAIL_ADDRESS,
                    InputAttributes.Variation.WEB_EMAIL_ADDRESS,
                    -> {
                        KeyVariation.EMAIL_ADDRESS
                    }
                    InputAttributes.Variation.PASSWORD,
                    InputAttributes.Variation.VISIBLE_PASSWORD,
                    InputAttributes.Variation.WEB_PASSWORD,
                    -> {
                        KeyVariation.PASSWORD
                    }
                    InputAttributes.Variation.URI -> {
                        KeyVariation.URI
                    }
                    else -> {
                        KeyVariation.NORMAL
                    }
                }
                KeyboardMode.CHARACTERS
            }
            else -> {
                activeState.keyVariation = KeyVariation.NORMAL
                KeyboardMode.CHARACTERS
            }
        }
        activeState.keyboardMode = keyboardMode
        activeState.isComposingEnabled = when (keyboardMode) {
            KeyboardMode.NUMERIC,
            KeyboardMode.PHONE,
            KeyboardMode.PHONE2,
            -> false
            else -> activeState.keyVariation != KeyVariation.PASSWORD &&
                prefs.suggestion.enabled.get()// &&
            //!instance.inputAttributes.flagTextAutoComplete &&
            //!instance.inputAttributes.flagTextNoSuggestions
        }
        activeState.isIncognitoMode = when (prefs.suggestion.incognitoMode.get()) {
            IncognitoMode.FORCE_OFF -> false
            IncognitoMode.FORCE_ON -> true
            IncognitoMode.DYNAMIC_ON_OFF -> {
                editorInfo.imeOptions.flagNoPersonalizedLearning || prefs.suggestion.forceIncognitoModeFromDynamic.get()
            }
        }
    }

    override fun handleSelectionUpdate(oldSelection: EditorRange, newSelection: EditorRange, composing: EditorRange) {
        autoSpace.setInactiveFromUpdate()
        phantomSpace.setInactiveFromUpdate()
        if (massSelection.isActive) {
            super.handleMassSelectionUpdate(newSelection, composing)
        } else {
            super.handleSelectionUpdate(oldSelection, newSelection, composing)
        }
    }

    override fun determineComposingEnabled(): Boolean {
        return nlpManager.isSuggestionOn()
    }

    override fun determineComposer(composerName: ExtensionComponentName): Composer {
        return keyboardManager.resources.composers.value?.get(composerName) ?: Appender
    }

    override fun shouldDetermineComposingRegion(editorInfo: FlorisEditorInfo): Boolean {
        return super.shouldDetermineComposingRegion(editorInfo) &&
            (phantomSpace.isInactive || phantomSpace.showComposingRegion)
    }

    /**
     * Sets the selection of the input editor to the specified [start] and [end] values. This method does nothing if
     * the input connection is not valid or if the input editor is raw.
     *
     * @param start The start of the selection (inclusive). May be any value ranging from -1 to positive infinity.
     * @param end The end of the selection (exclusive). May be any value ranging from -1 to positive infinity.
     *
     * @return True on success or if the selection is already at specified position, false otherwise.
     */
    fun setSelection(start: Int, end: Int): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        val selection = EditorRange.normalized(start, end)
        return super.setSelection(selection)
    }

    /** What the spacing rules ([Spacing]) look up, when they need it. */
    private val spacingEditor = object : SpacingEditor {
        override val autoSpacePunctuation: Boolean get() = prefs.correction.autoSpacePunctuation.get()
        override val isRawInputEditor: Boolean get() = activeInfo.isRawInputEditor
        override val keyVariation: KeyVariation get() = activeState.keyVariation
        override val selection: EditorRange get() = activeContent.selection
        override val locale: FlorisLocale get() = subtypeManager.activeSubtype.primaryLocale
        override val punctuationRule: PunctuationRule get() = nlpManager.getActivePunctuationRule()
        override fun textBeforeCursor(n: Int): String = activeContent.getTextBeforeCursor(n)
        override val currentWordText: String get() = activeContent.currentWordText
    }

    override fun commitChar(char: String): Boolean {
        val spaces = Spacing.commitChar(char, spacingEditor, autoSpace, phantomSpace)
        return super.commitChar(
            char = char,
            deletePreviousSpace = spaces.deletePreviousSpace,
            insertSpaceBeforeChar = spaces.insertSpaceBeforeChar,
            insertSpaceAfterChar = spaces.insertSpaceAfterChar,
        )
    }

    /**
     * Commits the given [text] to this editor instance and adjusts both the cursor position and
     * composing region, if any.
     *
     * This method overwrites any selected text and replaces it with given [text]. If there is no
     * text selected (selection is in cursor mode), then this method will insert the [text] after
     * the cursor, then set the cursor position to the first character after the inserted text.
     *
     * @param text The text to commit.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    override fun commitText(text: String): Boolean {
        val isPhantomSpaceActive = phantomSpace.determine(text)
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return if (isPhantomSpaceActive) {
            super.commitText("$SPACE$text")
        } else {
            super.commitText(text)
        }
    }

    /**
     * Completes the given [candidate] in the current composing region. Does nothing if the current
     * input editor is not rich or if the input connection is invalid.
     *
     * Current phantom space state is respected and a space char will be inserted accordingly.
     * Phantom space will be activated if the text is committed.
     *
     * @param candidate The candidate to complete in this editor.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun commitCompletion(candidate: SuggestionCandidate): Boolean {
        val text = candidate.text.toString()
        if (text.isEmpty() || activeInfo.isRawInputEditor) return false
        val content = activeContent
        return if (content.composing.isValid) {
            phantomSpace.setActive(showComposingRegion = false, candidate = candidate)
            super.finalizeComposingText(text)
        } else {
            val isPhantomSpaceActive = phantomSpace.determine(text)
            phantomSpace.setActive(showComposingRegion = false, candidate = candidate)
            return if (isPhantomSpaceActive) {
                super.commitText("$SPACE$text")
            } else {
                super.commitText(text)
            }.also {
                // handled in finalizeComposingText if content.composing.isValid
                updateLastCommitPosition()
            }
        }
    }

    /**
     * AudioWhisper: commits a dictated phrase as finished text. No word of it is left composing,
     * so the next space can't auto-correct a dictated name, and the phantom space is set as after
     * a gesture, so a word typed next gets its own space instead of running into the phrase.
     * [text] already carries any space it needs in front. [replacing] characters before the
     * cursor are deleted first (a full stop dictation put at a pause); an editor that can't
     * do that still gets the text.
     */
    fun commitDictation(text: String, replacing: Int = 0): Boolean {
        if (text.isEmpty()) return false
        val content = activeContent
        autoSpace.setInactive()
        phantomSpace.setActive(showComposingRegion = false)
        val replaced = replacing > 0 && replaceTextBeforeCursor(replacing, text)
        val committed = replaced || super.commitText(text)
        if (!committed) {
            phantomSpace.setInactive()
            return false
        }
        updateLastCommitPosition()
        val restore = if (replaced) content.textBeforeSelection.takeLast(replacing) else ""
        val start = content.selection.start - restore.length
        dictationUndo = DictationUndo(text, start, restore).takeIf { content.selection.isCursorMode && start >= 0 }
        return true
    }

    /**
     * AudioWhisper: a backspace straight after a dictated phrase takes the whole phrase back
     * (and puts back a full stop joining it took). False if there is none to take back.
     */
    fun undoDictation(): Boolean {
        val undo = dictationUndo ?: return false
        dictationUndo = null
        val content = activeContent
        if (!content.selection.isCursorMode) return false
        val revert = undo.revertAt(content.selection.start, content.textBeforeSelection) ?: return false
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return replaceTextBeforeCursor(revert.length, revert.text)
    }

    /** AudioWhisper: another key was pressed, so a backspace no longer takes back the last phrase. */
    fun forgetDictation() {
        dictationUndo = null
    }

    /**
     * Commit a word generated by a gesture.
     *
     * Ignores the current phantom space state and will insert a space depending on the character
     * before selection start. Phantom space will be activated if the text is committed.
     *
     * @param text The text to commit in this editor.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun commitGesture(text: String): Boolean {
        if (text.isEmpty() || activeInfo.isRawInputEditor) return false
        val isPhantomSpaceActive = phantomSpace.determine(text, forceActive = true)
        phantomSpace.setActive(showComposingRegion = true)
        return if (isPhantomSpaceActive) {
            super.commitText("$SPACE$text")
        } else {
            super.commitText(text)
        }.also {
            updateLastCommitPosition()
        }
    }

    /**
     * Commits the given [ClipboardItem]. If the clip data is text (incl. HTML), it delegates to [commitText].
     * If the item has a content URI (and the EditText supports it), the item is committed as rich data.
     * This allows for committing (e.g) images.
     *
     * @param item The ClipboardItem to commit
     *
     * @return True on success, false if something went wrong.
     */
    fun commitClipboardItem(item: ClipboardItem?): Boolean {
        if (item == null) return false
        val mimeTypes = item.mimeTypes
        return when (item.type) {
            ItemType.TEXT -> {
                commitText(item.text.toString()).also {
                    updateLastCommitPosition()
                }
            }
            ItemType.IMAGE, ItemType.VIDEO -> {
                item.uri ?: return false
                val id = ContentUris.parseId(item.uri)
                val file = ClipboardFileStorage.getFileForId(appContext, id)
                if (!file.exists()) return false
                val inputContentInfo = InputContentInfoCompat(
                    item.uri,
                    ClipDescription("clipboard media file", mimeTypes.toTypedArray()),
                    null,
                )
                val ic = currentInputConnection() ?: return false
                ic.finishComposingText()
                val flags = InputConnectionCompat.INPUT_CONTENT_GRANT_READ_URI_PERMISSION
                InputConnectionCompat.commitContent(ic, activeInfo.base, inputContentInfo, flags, null)
            }
        }.also {
            if (prefs.clipboard.historyHideOnPaste.get()) {
                keyboardManager.activeState.imeUiMode = ImeUiMode.TEXT
            }
        }
    }

    /** AudioWhisper: true if the app being typed in takes GIF images (rich content). */
    fun acceptsGifs(): Boolean =
        EditorInfoCompat.getContentMimeTypes(activeInfo.base).any { ClipDescription.compareMimeTypes(GIF_MIME_TYPE, it) }

    /** AudioWhisper: sends the GIF at [uri] (readable through our file provider) as rich content. */
    fun commitGif(uri: Uri, description: String, link: Uri?): Boolean {
        val ic = currentInputConnection() ?: return false
        val info = InputContentInfoCompat(uri, ClipDescription(description, arrayOf(GIF_MIME_TYPE)), link)
        ic.finishComposingText()
        val flags = InputConnectionCompat.INPUT_CONTENT_GRANT_READ_URI_PERMISSION
        return InputConnectionCompat.commitContent(ic, activeInfo.base, info, flags, null)
    }

    /**
     * AudioWhisper: undoes an auto-correction (see [AutoCorrectionUndo]). The phantom space is set
     * first so the restored word does not become a composing region and get corrected again.
     */
    fun revertAutoCorrection(revert: AutoCorrectionUndo.Revert): Boolean {
        autoSpace.setInactive()
        if (revert.phantomSpace) phantomSpace.setActive(showComposingRegion = false) else phantomSpace.setInactive()
        return replaceTextBeforeCursor(revert.length, revert.text).also { if (!it) phantomSpace.setInactive() }
    }

    /**
     * Executes a backward delete on this editor's text. If a text selection is active, all
     * characters inside this selection will be removed, else only the left-most character from
     * the cursor's position.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun deleteBackwards(unit: OperationUnit): Boolean {
        val content = activeContent
        if (unit == OperationUnit.CHARACTERS) {
            if (phantomSpace.isActive && content.currentWord.isValid && prefs.glide.immediateBackspaceDeletesWord.get()) {
                return deleteBackwards(OperationUnit.WORDS)
            }
        }
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return if (content.selection.isSelectionMode) {
            commitText("")
        } else runBlocking {
            deleteAroundCursor(unit, OperationScope.BEFORE_CURSOR, n = 1)
        }
    }

    /**
     * Executes a backward delete on this editor's text. If a text selection is active, all
     * characters inside this selection will be removed, else only the left-most character from
     * the cursor's position.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun deleteForwards(unit: OperationUnit): Boolean {
        val content = activeContent
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return if (content.selection.isSelectionMode) {
            commitText("")
        } else runBlocking {
            deleteAroundCursor(unit, OperationScope.AFTER_CURSOR, n = 1)
        }
    }

    fun setSelectionSurrounding(n: Int, unit: OperationUnit, scope: OperationScope): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        val content = activeContent
        val selection = content.selection
        val safeEditorBounds = content.safeEditorBounds
        if (selection.isNotValid) return false
        when (scope) {
            OperationScope.BEFORE_CURSOR -> {
                if (n <= 0) {
                    return setSelection(selection.end, selection.end)
                }
                val textToAnalyze = content.text.substring(0, content.localSelection.end)
                val length = runBlocking {
                    when (unit) {
                        OperationUnit.CHARACTERS -> breakIterators.measureLastUChars(textToAnalyze, n)
                        OperationUnit.WORDS -> breakIterators.measureLastUWords(textToAnalyze, n)
                    }
                }
                return setSelection((selection.end - length).coerceAtLeast(safeEditorBounds.start), selection.end)
            }
            OperationScope.AFTER_CURSOR -> {
                if (n <= 0) {
                    return setSelection(selection.start, selection.start)
                }
                val textToAnalyze = content.text.substring(content.localSelection.start)
                val length = runBlocking {
                    when (unit) {
                        OperationUnit.CHARACTERS -> breakIterators.measureUChars(textToAnalyze, n)
                        OperationUnit.WORDS -> breakIterators.measureUWords(textToAnalyze, n)
                    }
                }
                return setSelection(selection.start, (selection.start + length).coerceAtMost(safeEditorBounds.end))
            }
        }
    }

    /**
     * Performs a cut command on this editor instance and adjusts both the cursor position and
     * composing region, if any.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performClipboardCut(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        val text = activeContent.selectedText.ifBlank { currentInputConnection()?.getSelectedText(0) }
        if (text != null) {
            clipboardManager.addNewPlaintext(text.toString())
        } else {
            scope.launch { appContext.showShortToast("Failed to retrieve selected text requested to cut: Eiter selection state is invalid or an error occurred within the input connection.") }
        }
        return deleteBackwards(OperationUnit.CHARACTERS)
    }

    /**
     * Performs a copy command on this editor instance and adjusts both the cursor position and
     * composing region, if any.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performClipboardCopy(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        val text = activeContent.selectedText.ifBlank { currentInputConnection()?.getSelectedText(0) }
        if (text != null) {
            clipboardManager.addNewPlaintext(text.toString())
        } else {
            scope.launch { appContext.showShortToast("Failed to retrieve selected text requested to copy: Eiter selection state is invalid or an error occurred within the input connection.") }
        }
        val activeSelection = activeContent.selection
        return setSelection(activeSelection.end, activeSelection.end)
    }

    /**
     * Performs a paste command on this editor instance and adjusts both the cursor position and
     * composing region, if any.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performClipboardPaste(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return commitClipboardItem(clipboardManager.primaryClip).also { result ->
            if (!result) {
                scope.launch { appContext.showShortToast("Failed to paste item.") }
            }
        }
    }

    /**
     * Performs a select all on this editor instance and adjusts both the cursor position and
     * composing region, if any.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performClipboardSelectAll(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        val ic = currentInputConnection() ?: return false
        ic.finishComposingText()
        return if (activeInfo.isRawInputEditor) {
            sendDownUpKeyEvent(KeyEvent.KEYCODE_A, meta(ctrl = true))
        } else {
            ic.performContextMenuAction(android.R.id.selectAll)
        }
    }

    /**
     * Performs an enter key press on the current input editor.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performEnter(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return if (activeInfo.isRawInputEditor) {
            sendDownUpKeyEvent(KeyEvent.KEYCODE_ENTER)
        } else {
            commitText("\n")
        }
    }

    fun tryPerformEnterCommitRaw(): Boolean {
        return if (subtypeManager.activeSubtype.primaryLocale.language.startsWith("zh") && activeContent.composing.length > 0) {
            finalizeComposingText(activeContent.composingText)
        } else {
            false
        }
    }

    /**
     * Performs a given [action] on the current input editor.
     *
     * @param action The action to be performed on this editor instance.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performEnterAction(action: ImeOptions.Action): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        val ic = currentInputConnection() ?: return false
        return ic.performEditorAction(action.toInt())
    }

    /**
     * Undoes the last action.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performUndo(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return sendDownUpKeyEvent(KeyEvent.KEYCODE_Z, meta(ctrl = true))
    }

    /**
     * Redoes the last Undo action.
     *
     * @return True on success, false if an error occurred or the input connection is invalid.
     */
    fun performRedo(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return sendDownUpKeyEvent(KeyEvent.KEYCODE_Z, meta(ctrl = true, shift = true))
    }

    override fun reset() {
        super.reset()
        autoSpace.setInactive()
        phantomSpace.setInactive()
        massSelection.reset()
        dictationUndo = null
    }

    private fun PhantomSpaceState.determine(text: String, forceActive: Boolean = false): Boolean =
        Spacing.phantomSpaceBefore(this, text, forceActive, spacingEditor)

    /** Whether an auto-space was typed after punctuation; its rules are in Rust ([SpacingRules]). */
    class AutoSpaceState {
        private val state = AtomicInteger(SpacingRules.INACTIVE)

        /** The state as Rust reads it. */
        internal var bits: Int
            get() = state.get()
            set(value) = state.set(value)

        val isActive: Boolean
            get() = SpacingRules.isActive(state.get())

        val isInactive: Boolean
            get() = !isActive

        fun setActive(stayActiveNextUpdate: Boolean = true) {
            state.set(SpacingRules.autoSpaceActive(stayActiveNextUpdate))
        }

        fun setInactive() {
            state.set(SpacingRules.INACTIVE)
        }

        fun setInactiveFromUpdate() {
            state.updateAndGet { SpacingRules.afterUpdate(it) }
        }
    }

    /** Whether a phantom space is owed, and the suggestion it came from; its rules are in Rust ([SpacingRules]). */
    class PhantomSpaceState {
        private val state = AtomicInteger(SpacingRules.INACTIVE)
        var candidateForRevert: SuggestionCandidate? = null
            private set

        /** The state as Rust reads it. Setting it leaves [candidateForRevert] as it is. */
        internal var bits: Int
            get() = state.get()
            set(value) = state.set(value)

        val isActive: Boolean
            get() = SpacingRules.isActive(state.get())

        val isInactive: Boolean
            get() = !isActive

        val showComposingRegion: Boolean
            get() = SpacingRules.showsComposingRegion(state.get())

        fun setActive(
            showComposingRegion: Boolean,
            stayActiveNextUpdate: Boolean = true,
            candidate: SuggestionCandidate? = null,
        ) {
            state.set(SpacingRules.phantomSpaceActive(showComposingRegion, stayActiveNextUpdate))
            candidateForRevert = candidate
        }

        fun setInactive() {
            state.set(SpacingRules.INACTIVE)
            candidateForRevert = null
        }

        /** Sets the state Rust gave, which no suggestion goes with. */
        internal fun reset(bits: Int) {
            state.set(bits)
            candidateForRevert = null
        }

        fun setInactiveFromUpdate() {
            val prevStateValue = state.getAndUpdate { SpacingRules.afterUpdate(it) }
            if (!SpacingRules.keepsCandidate(prevStateValue)) {
                candidateForRevert = null
            }
        }
    }

    inner class MassSelectionState {
        private val state = AtomicInteger(0)

        val isActive: Boolean
            get() = state.get() > 0

        val isInactive: Boolean
            get() = !isActive

        fun begin() {
            state.incrementAndGet()
        }

        fun end() {
            if (state.decrementAndGet() == 0) {
                // We need to emulate a selection update to update the content if mass selection has ended
                handleSelectionUpdate(EditorRange.Unspecified, activeContent.selection, EditorRange.Unspecified)
            }
        }

        fun reset() {
            state.set(0)
        }
    }
}
