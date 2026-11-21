package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import android.text.InputType
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.ExtractedText
import android.view.inputmethod.ExtractedTextRequest
import android.view.inputmethod.InputConnection
import android.view.inputmethod.SurroundingText
import com.jtn0123.audiowhisper.rust.*

/** Android input endpoint only. Rust owns the text, selection, composing span and edit operations. */
internal class SettingsInputView(context: Context) : View(context) {
    private lateinit var host: RustSettingsView
    constructor(context: Context, host: RustSettingsView) : this(context) { this.host = host }
    init { isFocusable = true; isFocusableInTouchMode = true; importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_NO }
    override fun onCheckIsTextEditor() = true
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection? {
        if (!::host.isInitialized) return null
        // A multi-line field (DictationScreen's vocabulary) gets what Compose's EditorInfo.update
        // makes of the default ImeAction there: no action, so Enter is a new line, and no
        // fullscreen mode.
        // A time field gets TimeInput's KeyboardOptions: a number keyboard, Next on the hour and
        // Done on the minute.
        val multiLine = host.multiLine
        val number = host.numberAction
        outAttrs.inputType = when {
            number != null -> InputType.TYPE_CLASS_NUMBER
            else -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_AUTO_CORRECT or
                (if (multiLine) InputType.TYPE_TEXT_FLAG_MULTI_LINE else 0)
        }
        outAttrs.imeOptions = when {
            number == "next" -> EditorInfo.IME_ACTION_NEXT or EditorInfo.IME_FLAG_NO_EXTRACT_UI
            multiLine -> EditorInfo.IME_ACTION_UNSPECIFIED or EditorInfo.IME_FLAG_NO_ENTER_ACTION or EditorInfo.IME_FLAG_NO_FULLSCREEN
            else -> EditorInfo.IME_ACTION_DONE or EditorInfo.IME_FLAG_NO_EXTRACT_UI
        }
        outAttrs.initialSelStart = host.anchor
        outAttrs.initialSelEnd = host.selectionEnd
        return object : BaseInputConnection(this, true) {
            override fun commitText(text: CharSequence?, newCursorPosition: Int) = host.edit(SettingsUiEdit.COMMIT, text?.toString().orEmpty(), newCursorPosition.toString())
            override fun setComposingText(text: CharSequence?, newCursorPosition: Int) = host.edit(SettingsUiEdit.COMPOSE, text?.toString().orEmpty(), newCursorPosition.toString())
            override fun setComposingRegion(start: Int, end: Int) = host.edit(SettingsUiEdit.REGION, start.toString(), end.toString())
            override fun finishComposingText() = host.edit(SettingsUiEdit.FINISH)
            override fun setSelection(start: Int, end: Int) = host.edit(SettingsUiEdit.SELECT, start.toString(), end.toString())
            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int) = host.edit(SettingsUiEdit.DELETE, beforeLength.toString(), afterLength.toString())
            override fun deleteSurroundingTextInCodePoints(beforeLength: Int, afterLength: Int) = host.edit(SettingsUiEdit.CODEPOINTS, beforeLength.toString(), afterLength.toString())
            override fun getTextBeforeCursor(n: Int, flags: Int): CharSequence = host.query(SettingsUiQuery.BEFORE, n)
            override fun getTextAfterCursor(n: Int, flags: Int): CharSequence = host.query(SettingsUiQuery.AFTER, n)
            override fun getSelectedText(flags: Int): CharSequence = host.query(SettingsUiQuery.SELECTED, 0)
            override fun getCursorCapsMode(reqModes: Int) = host.query(SettingsUiQuery.CAPS, reqModes).toInt()
            override fun getSurroundingText(beforeLength: Int, afterLength: Int, flags: Int): SurroundingText {
                val before = host.query(SettingsUiQuery.BEFORE, beforeLength)
                val selected = host.query(SettingsUiQuery.SELECTED, 0)
                val after = host.query(SettingsUiQuery.AFTER, afterLength)
                return SurroundingText(before + selected + after, before.length, before.length + selected.length, host.anchor - before.length)
            }
            override fun getExtractedText(request: ExtractedTextRequest?, flags: Int) = ExtractedText().apply {
                text = host.editorText; startOffset = 0; partialStartOffset = -1; partialEndOffset = -1
                selectionStart = host.anchor; selectionEnd = host.selectionEnd
            }
            // Rust holds back the cursor updates of a batch until it ends, as EditText does.
            override fun beginBatchEdit() = host.edit(SettingsUiEdit.BEGIN)
            override fun endBatchEdit() = host.edit(SettingsUiEdit.END)
            override fun sendKeyEvent(event: KeyEvent) = host.forwardKey(event)
            override fun performEditorAction(actionCode: Int) = host.forwardKey(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_ENTER))
            override fun closeConnection() { host.edit(SettingsUiEdit.CLOSE); super.closeConnection() }
        }
    }
}
