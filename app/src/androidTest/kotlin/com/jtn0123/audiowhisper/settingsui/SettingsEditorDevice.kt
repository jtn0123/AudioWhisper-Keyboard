package com.jtn0123.audiowhisper.settingsui

import android.os.SystemClock
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import org.json.JSONObject
import java.io.File

/** Direct endpoint checks. No keyboard, clipboard payload or field contents enter receipts. */
internal class SettingsEditorDevice(private val h: SettingsHarness) {
    private val fixture = "ab\uD83D\uDE80cdef"
    private fun host() = h.views(RustSettingsView::class.java).single()
    private fun endpoint() = h.views(SettingsInputView::class.java).single()
    fun run(report: JSONObject) {
        h.launch("settings/keyboard")
        val view = host()
        val heapBefore = android.os.Debug.getNativeHeapAllocatedSize()
        lateinit var connection: InputConnection
        h.main {
            check(view.edit("focus")) // Focus native model without showing an IME.
            connection = checkNotNull(endpoint().onCreateInputConnection(EditorInfo()))
            check(connection.setComposingText("ab", 1))
            check(view.editorText == "ab")
            check(view.save()[11] == "0" && view.save()[12] == "2")
            check(connection.setComposingText(fixture, 1))
            check(view.editorText == fixture)
            check(connection.finishComposingText())
            check(view.save()[11] == "-1")
            check(connection.setSelection(1, 4))
            check(view.anchor == 1 && view.selectionEnd == 4)
        }
        h.waitFor("selection handles absent") { h.find("Start of selection") != null && h.find("End of selection") != null }
        val handles = h.nodes().filter { h.label(it) in listOf("Start of selection", "End of selection") }
        check(handles.size == 2 && handles.all { it.isEnabled && it.isClickable })
        // Drag the actual native end handle through host MotionEvents. Rust hit-tests it.
        val node = checkNotNull(h.find("End of selection"))
        val bounds = android.graphics.Rect().also(node::getBoundsInScreen)
        h.main {
            val origin = IntArray(2).also(view::getLocationOnScreen)
            val x = bounds.exactCenterX() - origin[0]
            val y = bounds.exactCenterY() - origin[1]
            val time = SystemClock.uptimeMillis()
            for ((action, offset, dt) in listOf(Triple(MotionEvent.ACTION_DOWN, 0f, 0L), Triple(MotionEvent.ACTION_MOVE, 45f, 16L), Triple(MotionEvent.ACTION_UP, 45f, 32L))) {
                val event = MotionEvent.obtain(time, time+dt, action, x+offset, y, 0)
                view.onTouchEvent(event)
                event.recycle()
            }
            check(view.anchor == 1 && view.selectionEnd > 4)
            check(connection.setSelection(4, 4))
            check(connection.deleteSurroundingText(2, 1))
            check(view.editorText == "abdef")
            check(connection.commitText("Z", 1))
            check(view.editorText == "abZdef")
            check(connection.sendKeyEvent(KeyEvent(KeyEvent.ACTION_DOWN, KeyEvent.KEYCODE_DEL)))
            check(view.editorText == "abdef")
            check(connection.setSelection(0, view.editorText.length))
            check(connection.setComposingText(fixture, 1))
        }
        val before = view.save()
        val old = h.activity
        h.main { checkNotNull(old).recreate() }
        val runner = h.instrumentation as SettingsInstrumentation
        h.waitFor("recreated activity absent") { runner.resumedActivity != null && runner.resumedActivity !== old }
        h.activity = runner.resumedActivity
        h.waitFor("recreated native view absent") { h.views(RustSettingsView::class.java).size == 1 }
        val restored = host()
        h.main {
            val after = restored.save()
            val changed = listOf(1, 8, 9, 10, 11, 12, 13).filter { before[it] != after[it] }
            report.put("recreate_changed_slots", org.json.JSONArray(changed))
            report.put("before_meta", org.json.JSONArray(listOf(9,10,11,12,13).map { before[it] }))
            report.put("after_meta", org.json.JSONArray(listOf(9,10,11,12,13).map { after[it] }))
            check(changed.isEmpty())
            connection = checkNotNull(endpoint().onCreateInputConnection(EditorInfo()))
            check(connection.finishComposingText())
            check(connection.setSelection(0, restored.editorText.length))
            check(connection.commitText("", 1))
            check(restored.editorText.isEmpty())
            connection.closeConnection()
        }
        check(h.nodes().none { h.label(it) in listOf("Start of selection", "End of selection") })
        report.put("native_heap_before_bytes", heapBefore).put("native_heap_after_bytes", android.os.Debug.getNativeHeapAllocatedSize())
        report.put("input_connection_operations", 18).put("selection_handles", 2).put("handle_drag", true)
            .put("recreate_mid_composition", true).put("ime_changed", false)
    }
    fun prepareDeath(report: JSONObject) {
        // Finish instrumentation before ordinary process death. AMS tears down the task
        // when its instrumentation process dies, which is a different lifecycle boundary.
        File(h.instrumentation.targetContext.filesDir, "settings-death-arm").writeText("armed")
        report.put("ordinary_process_death_armed", true)
    }
}
