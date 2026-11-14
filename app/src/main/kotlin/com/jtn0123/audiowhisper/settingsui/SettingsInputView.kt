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

/** Android input endpoint only. Rust owns the text, selection, composing span and edit operations. */
internal class SettingsInputView(context: Context) : View(context) {
    private lateinit var host: RustSettingsView
    constructor(context: Context, host: RustSettingsView) : this(context) { this.host = host }
    init { isFocusable = true; isFocusableInTouchMode = true; importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_NO }
    override fun onCheckIsTextEditor() = true
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection? {
        if (!::host.isInitialized) return null
        outAttrs.inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_AUTO_CORRECT
        outAttrs.imeOptions = EditorInfo.IME_ACTION_DONE or EditorInfo.IME_FLAG_NO_EXTRACT_UI
        outAttrs.initialSelStart = host.anchor
        outAttrs.initialSelEnd = host.selectionEnd
        return object : BaseInputConnection(this, true) {
            override fun commitText(text: CharSequence?, newCursorPosition: Int) = host.edit("commit", text?.toString().orEmpty(), newCursorPosition.toString())
            override fun setComposingText(text: CharSequence?, newCursorPosition: Int) = host.edit("compose", text?.toString().orEmpty(), newCursorPosition.toString())
            override fun setComposingRegion(start: Int, end: Int) = host.edit("region", start.toString(), end.toString())
            override fun finishComposingText() = host.edit("finish")
            override fun setSelection(start: Int, end: Int) = host.edit("select", start.toString(), end.toString())
            override fun deleteSurroundingText(beforeLength: Int, afterLength: Int) = host.edit("delete", beforeLength.toString(), afterLength.toString())
            override fun deleteSurroundingTextInCodePoints(beforeLength: Int, afterLength: Int) = host.edit("codepoints", beforeLength.toString(), afterLength.toString())
            override fun getTextBeforeCursor(n: Int, flags: Int): CharSequence = host.query("before", n)
            override fun getTextAfterCursor(n: Int, flags: Int): CharSequence = host.query("after", n)
            override fun getSelectedText(flags: Int): CharSequence = host.query("selected", 0)
            override fun getCursorCapsMode(reqModes: Int) = host.query("caps", reqModes).toInt()
            override fun getSurroundingText(beforeLength: Int, afterLength: Int, flags: Int): SurroundingText {
                val before = host.query("before", beforeLength)
                val selected = host.query("selected", 0)
                val after = host.query("after", afterLength)
                return SurroundingText(before + selected + after, before.length, before.length + selected.length, host.anchor - before.length)
            }
            override fun getExtractedText(request: ExtractedTextRequest?, flags: Int) = ExtractedText().apply {
                text = host.editorText; startOffset = 0; partialStartOffset = -1; partialEndOffset = -1
                selectionStart = host.anchor; selectionEnd = host.selectionEnd
            }
            override fun beginBatchEdit() = true
            override fun endBatchEdit() = true
            override fun sendKeyEvent(event: KeyEvent) = host.forwardKey(event)
            override fun performEditorAction(actionCode: Int) = host.forwardKey(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_ENTER))
            override fun closeConnection() { host.edit("finish"); super.closeConnection() }
        }
    }
}
