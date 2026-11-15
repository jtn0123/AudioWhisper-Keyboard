package com.jtn0123.audiowhisper.settingsui

import android.view.KeyEvent
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import org.json.JSONObject

/**
 * DictationScreen's vocabulary field through each renderer's own InputConnection: the same edits,
 * each checked in the stored preference (the screen saves every change), then the user's text put
 * back exactly. Only lengths and EditorInfo numbers enter the report, never the vocabulary.
 */
internal class SettingsVocabularyDevice(private val h: SettingsHarness) {
    private val prefs by FlorisPreferenceStore

    fun run(report: JSONObject, info: EditorInfo, connection: InputConnection) {
        val saved = prefs.dictation.vocabulary
        val original = saved.get()
        fun expect(value: String) = h.waitFor("vocabulary not saved") { saved.get() == value }
        fun edit(value: String, operation: () -> Boolean) {
            h.main { check(operation()) }
            expect(value)
        }
        val fixture = "ab🚀cdef"
        h.main { check(connection.setSelection(0, original.length)) }
        edit("") { connection.commitText("", 1) }
        edit("ab") { connection.setComposingText("ab", 1) }
        edit(fixture) { connection.setComposingText(fixture, 1) }
        h.main { check(connection.finishComposingText()) }
        // Enter in a multi-line field is a line break, not an action.
        edit("$fixture\n") {
            connection.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_ENTER)) &&
                connection.sendKeyEvent(KeyEvent(KeyEvent.ACTION_UP, KeyEvent.KEYCODE_ENTER))
        }
        edit("$fixture\nZ") { connection.commitText("Z", 1) }
        edit("$fixture\n") { connection.deleteSurroundingText(1, 0) }
        h.main { check(connection.setSelection(0, fixture.length + 1)) }
        edit(original) { connection.commitText(original, 1) }
        h.main { connection.closeConnection() }
        report.put("vocabulary", JSONObject().put("operations", 9).put("restored", saved.get() == original)
            .put("multi_line", info.inputType and android.text.InputType.TYPE_TEXT_FLAG_MULTI_LINE != 0)
            .put("input_type", info.inputType).put("ime_options", info.imeOptions))
    }
}
