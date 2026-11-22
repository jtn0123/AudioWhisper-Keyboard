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

import android.icu.lang.UCharacter
import com.jtn0123.audiowhisper.gif.GifSearch
import com.jtn0123.audiowhisper.gif.GifSearchKeys
import com.jtn0123.audiowhisper.reader.ReadAloud
import com.jtn0123.audiowhisper.rust.InputProtocol.*
import com.jtn0123.audiowhisper.rust.InputRules
import com.jtn0123.audiowhisper.rust.KeyModels
import dev.patrickgold.florisboard.FlorisImeService
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.editor.OperationUnit
import dev.patrickgold.florisboard.ime.onehanded.OneHandedMode
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogError
import dev.patrickgold.florisboard.lib.util.InputMethodUtils
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showShortToast

internal fun KeyboardManager.handleInputKeyDown(data: KeyData) = executeInput(PHASE_DOWN,data)
internal fun KeyboardManager.handleInputKeyUp(data: KeyData) = activeState.batchEdit {executeInput(PHASE_UP,data)}
internal fun KeyboardManager.handleInputKeyCancel(data: KeyData) = executeInput(PHASE_CANCEL,data)
internal fun KeyboardManager.handleInputKeyRepeat(data: KeyData) = activeState.batchEdit {executeInput(PHASE_REPEAT,data)}

/** Supply platform facts once, then execute Rust's ordered effects. Both renderers enter here. */
private fun KeyboardManager.executeInput(phase: Int,data: KeyData) {
    val facts=inputEventDispatcher.facts(data.code)
    val text=if(phase==PHASE_UP || phase==PHASE_REPEAT) data.asString(isForDisplay=false) else ""
    val plan=InputRules.route(longArrayOf(phase.toLong(),data.code.toLong(),data.type.ordinal.toLong(),
        activeState.rawValue.toLong(),if(inputEventDispatcher.isPressed(KeyCode.SHIFT)) 1 else 0,
        facts[1],facts[2],facts[4],prefs.keyboard.capitalizationBehavior.get().ordinal.toLong(),
        prefs.keyboard.utilityKeyAction.get().ordinal.toLong(),
        if(isTypingIntoGifSearch) GifSearchKeys.route(data.code,data.type).ordinal.toLong() else -1,
        if(text.isNotEmpty() && UCharacter.isUAlphabetic(UCharacter.codePointAt(text,0))) 1 else 0,
        if(prefs.suggestion.forceIncognitoModeFromDynamic.get()) 1 else 0,
        if(prefs.keyboard.oneHandedModeEnabled.get()) 1 else 0,
        if(prefs.smartbar.enabled.get()) 1 else 0))
    for (offset in plan.indices.step(3)) {
        val arg=plan[offset+1]
        when(plan[offset].toInt()) {
            STATE -> activeState.rawValue=KeyModels.write(activeState.rawValue.toLong(),(arg ushr 32).toInt(),arg.toInt()).toULong()
            MASS_BEGIN -> editorInstance.massSelection.begin()
            MASS_END -> editorInstance.massSelection.end()
            ARROW -> handleArrow(arg.toInt())
            STOP_READING -> ReadAloud.stop()
            MIC_DOWN -> FlorisImeService.pressMic()
            MIC_UP -> FlorisImeService.releaseMic()
            STOP_DICTATION -> FlorisImeService.stopDictating()
            READ_DOWN -> ReadAloud.keyDown()
            READ_UP -> ReadAloud.keyUp(appContext)
            READ_CANCEL -> ReadAloud.keyCancel()
            CLEAR_UNDO -> nlpManager.disarmCorrectionUndo()
            FORGET_DICTATION -> editorInstance.forgetDictation()
            GIF_DELETE_KEY -> GifSearch.deleteLast()
            GIF_DELETE_WORD_KEY -> GifSearch.deleteWord()
            GIF_SPACE -> GifSearch.type(" ")
            GIF_SUBMIT -> GifSearch.submit()
            GIF_TYPE -> GifSearch.type(text)
            GIF_STOP -> GifSearch.stopTyping()
            CUT -> editorInstance.performClipboardCut()
            COPY -> editorInstance.performClipboardCopy()
            PASTE -> editorInstance.performClipboardPaste()
            SELECT -> handleClipboardSelect()
            SELECT_ALL -> editorInstance.performClipboardSelectAll()
            CLEAR_HISTORY -> clipboardManager.clearHistory()
            CLEAR_FULL_HISTORY -> clipboardManager.clearFullHistory()
            CLEAR_CLIP -> {
                if(prefs.clipboard.clearPrimaryClipAffectsHistoryIfUnpinned.get()) {
                    clipboardManager.primaryClip?.let {clipboardManager.deleteClip(it,onlyIfUnpinned=true)}
                }
                clipboardManager.updatePrimaryClip(null)
                scope.launch {appContext.showShortToast(R.string.clipboard__cleared_primary_clip)}
            }
            ONE_HANDED -> scope.launch {
                if(arg.toInt()>=0) prefs.keyboard.oneHandedMode.set(OneHandedMode.entries[arg.toInt()])
                prefs.keyboard.oneHandedModeEnabled.set(arg ushr 32 != 0L)
            }
            DELETE_BACK -> handleBackwardDelete(OperationUnit.entries[arg.toInt()])
            DELETE_FORWARD -> handleForwardDelete(OperationUnit.entries[arg.toInt()])
            ENTER_KEY -> handleEnter()
            SHOW_UI -> FlorisImeService.showUi()
            HIDE_UI -> FlorisImeService.hideUi()
            SUBTYPE -> if(arg<0) subtypeManager.switchToPrevSubtype() else subtypeManager.switchToNextSubtype()
            NEXT_IME -> FlorisImeService.switchToNextInputMethod()
            PREV_IME -> FlorisImeService.switchToPrevInputMethod()
            UNDO_KEY -> { nlpManager.noteUndo(); editorInstance.performUndo() }
            REDO_KEY -> editorInstance.performRedo()
            SETTINGS_UI -> FlorisImeService.launchSettings()
            SPACE_KEY -> { nlpManager.noteKey(); handleSpace(data) }
            IME_PICKER -> InputMethodUtils.showImePicker(appContext)
            SMARTBAR -> scope.launch {prefs.smartbar.enabled.set(arg != 0L)}
            INCOGNITO -> scope.launch {handleToggleIncognitoMode(arg and 1 != 0L, (arg ushr 1).toInt())}
            AUTOCORRECT_TOGGLE -> handleToggleAutocorrect()
            AUTO_CORRECT -> autoCorrect(if(arg<0) null else UCharacter.codePointAt(text,0))
            COMMIT_TEXT -> editorInstance.commitText(text)
            TAP_TYPED -> nlpManager.noteTyped(text)
            COMMIT_CHAR -> editorInstance.commitChar(text)
            UNKNOWN_KEY -> flogError(LogTopic.KEY_EVENTS) {"Received unknown key"}
            FEEDBACK_REPEAT -> FlorisImeService.inputFeedbackController()?.keyRepeatedAction(data)
            else -> error("Unknown native input action")
        }
    }
}
