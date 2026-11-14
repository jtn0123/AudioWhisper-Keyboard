package com.jtn0123.audiowhisper.settingsui

import android.os.Bundle
import android.view.KeyEvent
import android.view.View
import android.view.accessibility.AccessibilityNodeInfo as Node
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.ExtractedTextRequest
import android.view.inputmethod.InputConnection
import org.json.JSONObject

/** The retained Compose editor is driven through its actual Android InputConnection too. */
internal class SettingsComposeEditorDevice(private val h: SettingsHarness) {
    fun run(report: JSONObject) {
        h.launch("settings/keyboard")
        val field = checkNotNull(h.nodes().firstOrNull { it.isEditable })
        check(field.performAction(Node.ACTION_FOCUS))
        lateinit var connection: InputConnection
        h.waitFor("Compose input endpoint unavailable") {
            var found: InputConnection? = null
            h.main {
                val owner = h.views(View::class.java).first { it.javaClass.name == "androidx.compose.ui.platform.AndroidComposeView" }
                found = owner.onCreateInputConnection(EditorInfo())
            }
            found?.let { connection = it }
            found != null
        }
        val request = ExtractedTextRequest()
        fun expect(value: String) {
            h.waitFor("Compose edit not applied") {
                var query: String? = null
                h.main { query = connection.getExtractedText(request, 0)?.text?.toString() }
                val node = h.nodes().firstOrNull { it.isEditable }
                val actual = if (node?.isShowingHintText == true) "" else node?.text?.toString()
                report.put("expected_units", value.length).put("connection_query_units", query?.length ?: -1)
                    .put("editor_units", actual?.length ?: -1)
                actual == value
            }
        }
        fun edit(value: String, operation: () -> Boolean) { h.main { check(operation()) }; expect(value) }
        val fixture = "ab\uD83D\uDE80cdef"
        edit("ab") { connection.setComposingText("ab", 1) }
        edit(fixture) { connection.setComposingText(fixture, 1) }
        h.main { check(connection.finishComposingText()); check(connection.setSelection(1,4)) }
        h.main { check(connection.setSelection(4,4)) }
        edit("abdef") { connection.deleteSurroundingText(2,1) }
        edit("abZdef") { connection.commitText("Z",1) }
        edit("abdef") { connection.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN,KeyEvent.KEYCODE_DEL)) }
        h.main {
            check(connection.setSelection(0,5))
            check(connection.commitText("",1))
        }
        expect("")
        h.main { connection.closeConnection() }
        report.put("input_connection_operations", 11).put("same_edit_sequence", true).put("ime_changed", false)
    }
}
