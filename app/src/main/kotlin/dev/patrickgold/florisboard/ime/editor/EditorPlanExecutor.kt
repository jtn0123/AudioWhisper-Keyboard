package dev.patrickgold.florisboard.ime.editor

import android.os.SystemClock
import android.view.InputDevice
import android.view.KeyCharacterMap
import android.view.KeyEvent
import android.view.inputmethod.InputConnection
import com.jtn0123.audiowhisper.rust.EditorSession
import dev.patrickgold.florisboard.ime.nlp.BreakIteratorGroup
import dev.patrickgold.florisboard.ime.text.composing.Composer
import dev.patrickgold.florisboard.ime.text.composing.WithRules
import dev.patrickgold.florisboard.lib.FlorisLocale

/** Interprets the native protocol verbatim. Branches select platform APIs, never editing policy. */
internal class EditorPlanExecutor(
    private val breaks: BreakIteratorGroup,
    private val locale: () -> FlorisLocale,
    private val composer: () -> Composer,
    private val word: suspend (String, Int) -> EditorRange,
) {
    private var loadedComposer: Composer? = null
    private fun supplyComposer(session: EditorSession) {
        val current = composer()
        if (loadedComposer !== current) {
            val rules = (current as? WithRules)?.let { table ->
                table.ruleOrder.flatMap { listOf(it, table.rules.getValue(it)) }
            }.orEmpty()
            session.call("composerRules", *rules.toTypedArray())
            loadedComposer = current
        }
    }
    suspend fun execute(session: EditorSession, ic: InputConnection, operation: String, fields: Array<out String>): Boolean {
        supplyComposer(session)
        var response = session.call(operation, *fields)
        while (response[0] != "ready") {
            val values = when (response[0]) {
                "read" -> arrayOf(
                    if (response[1] == "1") ic.getTextBeforeCursor(256, 0)?.toString().orEmpty() else "",
                    ic.getTextAfterCursor(128, 0)?.toString().orEmpty(),
                    if (response[2] == "1") ic.getSelectedText(0)?.toString().orEmpty() else "")
                "word" -> word(response[1], response[2].toInt()).let { arrayOf(it.start.toString(), it.end.toString()) }
                "character" -> arrayOf(breaks.measureUChars(response[1], 1, locale()).toString())
                "spacing" -> arrayOf(
                    response[1].takeLast(breaks.measureLastUChars(response[1], 1, locale())),
                    response[1].takeLast(breaks.measureLastUChars(response[1], 3, locale())))
                "measure", "extent" -> {
                    val count = response[4].toInt()
                    val length = when (response[2] + response[3]) {
                        "00" -> breaks.measureLastUChars(response[1], count, locale())
                        "01" -> breaks.measureLastUWords(response[1], count, locale())
                        "10" -> breaks.measureUChars(response[1], count, locale())
                        "11" -> breaks.measureUWords(response[1], count, locale())
                        else -> error("Unknown native measurement")
                    }
                    arrayOf(length.toString())
                }
                else -> error("Unknown native editor query")
            }
            response = session.call(response[0], *values)
        }
        val results = mutableListOf<String>()
        val eventTime = SystemClock.uptimeMillis()
        for (op in response.drop(2).chunked(4)) {
            val a = op[1].toInt()
            val b = op[2].toInt()
            val result = when (op[0]) {
                "begin" -> ic.beginBatchEdit()
                "end" -> ic.endBatchEdit()
                "finish" -> ic.finishComposingText()
                "composing" -> ic.setComposingRegion(a, b)
                "composingText" -> ic.setComposingText(op[3], a)
                "commit" -> ic.commitText(op[3], a)
                "delete" -> ic.deleteSurroundingText(a, b)
                "selection" -> ic.setSelection(a, b)
                "action" -> ic.performEditorAction(a)
                "contextMenu" -> ic.performContextMenuAction(a)
                "down", "up" -> ic.sendKeyEvent(KeyEvent(eventTime,
                    if (op[0] == "down") eventTime else SystemClock.uptimeMillis(),
                    if (op[0] == "down") KeyEvent.ACTION_DOWN else KeyEvent.ACTION_UP,
                    a, op[3].toIntOrNull() ?: 0, b, KeyCharacterMap.VIRTUAL_KEYBOARD, 0,
                    KeyEvent.FLAG_SOFT_KEYBOARD or KeyEvent.FLAG_KEEP_TOUCH_MODE, InputDevice.SOURCE_KEYBOARD))
                else -> error("Unknown native InputConnection operation")
            }
            results.add(if (result) "1" else "0")
        }
        return session.call("results", *results.toTypedArray())[0] == "1"
    }
}
