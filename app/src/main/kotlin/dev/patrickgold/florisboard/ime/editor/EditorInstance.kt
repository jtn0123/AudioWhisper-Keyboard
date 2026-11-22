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
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.clipboardManager
import dev.patrickgold.florisboard.ime.ImeUiMode
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardFileStorage
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem
import dev.patrickgold.florisboard.ime.clipboard.provider.ItemType
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.nlp.SuggestionCandidate
import dev.patrickgold.florisboard.ime.nlp.TypingEvents
import dev.patrickgold.florisboard.ime.text.composing.Appender
import dev.patrickgold.florisboard.ime.text.composing.Composer
import dev.patrickgold.florisboard.ime.text.key.KeyVariation
import dev.patrickgold.florisboard.keyboardManager
import dev.patrickgold.florisboard.lib.ext.ExtensionComponentName
import dev.patrickgold.florisboard.nlpManager
import dev.patrickgold.florisboard.subtypeManager
import java.util.concurrent.atomic.AtomicInteger
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showShortToast

/**
 * AudioWhisper: a cursor update ends the spacing the last edit set, unless it is [late] (the editor session's
 * judgement): news of an edit the keyboard has already followed with more, typing faster than the app reports,
 * leaves alone the spacing those later edits set, such as the space an undone correction owes.
 */
internal fun followUpdate(late: Boolean, autoSpace: EditorInstance.AutoSpaceState, phantomSpace: EditorInstance.PhantomSpaceState) {
    if (late) return
    autoSpace.setInactiveFromUpdate()
    phantomSpace.setInactiveFromUpdate()
}

class EditorInstance(context: Context) : AbstractEditorInstance(context) {
    companion object {
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


    private fun currentInputConnection() = FlorisImeService.currentInputConnection()

    override fun handleStartInputView(editorInfo: FlorisEditorInfo, isRestart: Boolean) {
        if (!prefs.correction.rememberCapsLockState.get()) {
            activeState.inputShiftState = InputShiftState.UNSHIFTED
        }
        activeState.isActionsOverflowVisible = false
        activeState.isActionsEditorVisible = false
        val plan = EditorAttributes.keyboard(editorInfo.inputAttributes.raw, editorInfo.imeOptions.raw,
            prefs.suggestion.enabled.get(), prefs.suggestion.incognitoMode.get().ordinal,
            prefs.suggestion.forceIncognitoModeFromDynamic.get())
        activeState.keyboardMode = KeyboardMode.fromInt(plan[EditorKeyboard.MODE])
        activeState.keyVariation = KeyVariation.fromInt(plan[EditorKeyboard.KEY_VARIATION])
        activeState.isComposingEnabled = plan[EditorKeyboard.COMPOSING] != 0
        activeState.isIncognitoMode = plan[EditorKeyboard.INCOGNITO] != 0
        super.handleStartInputView(editorInfo, isRestart)
        // AudioWhisper: editor positions the word learner kept, and tap positions, belong to the previous field.
        nlpManager.resetLearning()
    }

    override fun handleSelectionUpdate(oldSelection: EditorRange, newSelection: EditorRange, composing: EditorRange) {
        followUpdate(isLateUpdate(newSelection, composing), autoSpace, phantomSpace)
        super.handleSelectionUpdate(oldSelection, newSelection, composing)
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

    fun setSelection(start: Int, end: Int): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        val selection = EditorRange.normalized(start, end)
        return super.setSelection(selection)
    }

    private fun edit(operation: EditorOp, vararg fields: String, candidate: SuggestionCandidate? = null): Boolean {
        val rule = nlpManager.getActivePunctuationRule()
        editorSession.call(EditorOp.CONTEXT, *with(EditorContext) { row(AUTO_SPACE to "${autoSpace.bits}", PHANTOM_SPACE to "${phantomSpace.bits}",
            KEY_VARIATION to "${activeState.keyVariation.value}", AUTO_SPACE_PUNCTUATION to WireFlag.encode(prefs.correction.autoSpacePunctuation.get()),
            LANGUAGE to subtypeManager.activeSubtype.primaryLocale.language, PRECEDING_AUTO_SPACE to rule.symbolsPrecedingAutoSpace,
            FOLLOWING_AUTO_SPACE to rule.symbolsFollowingAutoSpace, PRECEDING_PHANTOM_SPACE to rule.symbolsPrecedingPhantomSpace,
            FOLLOWING_PHANTOM_SPACE to rule.symbolsFollowingPhantomSpace, IMMEDIATE_WORD to WireFlag.encode(prefs.glide.immediateBackspaceDeletesWord.get())) })
        val result = executeEditor(operation, *fields)
        val spaces = editorSession.call(EditorOp.SPACES)
        autoSpace.bits = spaces[EditorSpaces.AUTO_SPACE].toInt()
        val phantom = spaces[EditorSpaces.PHANTOM_SPACE].toInt()
        when (spaces[EditorSpaces.CANDIDATE].toInt()) {
            EditorCandidate.KEEP -> phantomSpace.bits = phantom
            EditorCandidate.RESET -> phantomSpace.reset(phantom)
            EditorCandidate.ACCEPT -> phantomSpace.accept(phantom, candidate)
            else -> error("Unknown native candidate disposition")
        }
        return result
    }

    override fun commitChar(char: String): Boolean {
        return edit(EditorOp.TYPE_CHAR, char)
    }

    override fun commitText(text: String): Boolean {
        return edit(EditorOp.TYPE_TEXT, text)
    }

    fun commitCompletion(candidate: SuggestionCandidate): Boolean {
        return edit(EditorOp.COMPLETION, candidate.text.toString(), candidate = candidate)
    }

    fun commitDictation(text: String, replacing: Int = 0): Boolean {
        return edit(EditorOp.DICTATION, text, replacing.toString())
    }

    fun undoDictation(): Boolean {
        return edit(EditorOp.UNDO_DICTATION)
    }

    fun forgetDictation() {
        editorSession.call(EditorOp.FORGET_DICTATION)
    }

    fun commitGesture(text: String): Boolean {
        return edit(EditorOp.GESTURE, text)
    }

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

    fun acceptsGifs(): Boolean =
        EditorInfoCompat.getContentMimeTypes(activeInfo.base).any { ClipDescription.compareMimeTypes(GIF_MIME_TYPE, it) }

    fun commitGif(uri: Uri, description: String, link: Uri?): Boolean {
        val ic = currentInputConnection() ?: return false
        val info = InputContentInfoCompat(uri, ClipDescription(description, arrayOf(GIF_MIME_TYPE)), link)
        ic.finishComposingText()
        val flags = InputConnectionCompat.INPUT_CONTENT_GRANT_READ_URI_PERMISSION
        return InputConnectionCompat.commitContent(ic, activeInfo.base, info, flags, null)
    }

    internal fun revertAutoCorrection(revert: TypingEvents.Revert): Boolean {
        return edit(EditorOp.REVERT, revert.length.toString(), revert.text, WireFlag.encode(revert.phantomSpace))
    }

    fun deleteBackwards(unit: OperationUnit): Boolean {
        return edit(EditorOp.BACKWARD, unit.ordinal.toString())
    }

    fun deleteForwards(unit: OperationUnit): Boolean {
        return edit(EditorOp.FORWARD, unit.ordinal.toString())
    }

    fun setSelectionSurrounding(n: Int, unit: OperationUnit, scope: OperationScope): Boolean {
        return edit(EditorOp.SELECT_AROUND, n.toString(), unit.ordinal.toString(), scope.ordinal.toString())
    }

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

    fun performClipboardPaste(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return commitClipboardItem(clipboardManager.primaryClip).also { result ->
            if (!result) {
                scope.launch { appContext.showShortToast("Failed to paste item.") }
            }
        }
    }

    fun performClipboardSelectAll(): Boolean {
        return edit(EditorOp.SELECT_ALL)
    }

    fun performEnter(): Boolean {
        return edit(EditorOp.ENTER)
    }

    fun tryPerformEnterCommitRaw(): Boolean = edit(EditorOp.RAW_ENTER, subtypeManager.activeSubtype.primaryLocale.language)

    fun performEnterAction(action: ImeOptions.Action): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return executeEditor(EditorOp.ACTION, action.toInt().toString())
    }

    fun performUndo(): Boolean {
        autoSpace.setInactive()
        phantomSpace.setInactive()
        return sendDownUpKeyEvent(KeyEvent.KEYCODE_Z, meta(ctrl = true))
    }

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
    }

    class AutoSpaceState {
        private val state = AtomicInteger(SpacingRules.INACTIVE)

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

    class PhantomSpaceState {
        private val state = AtomicInteger(SpacingRules.INACTIVE)
        var candidateForRevert: SuggestionCandidate? = null
            private set

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

            internal fun accept(bits: Int, candidate: SuggestionCandidate?) {
            state.set(bits)
            candidateForRevert = candidate
        }

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
        val isActive: Boolean get() = WireFlag.decode(editorSession.call(EditorOp.MASS_STATE)[0])
        val isInactive: Boolean get() = !isActive
        fun begin() { editorSession.call(EditorOp.MASS_BEGIN) }
        fun end() {
            if (WireFlag.decode(editorSession.call(EditorOp.MASS_END)[0])) {
                handleSelectionUpdate(EditorRange.Unspecified, activeContent.selection, EditorRange.Unspecified)
            }
        }
        fun reset() { editorSession.call(EditorOp.MASS_RESET) }
    }
    fun commitDoubleSpacePeriod(): Boolean = edit(EditorOp.DOUBLE_SPACE)
}
