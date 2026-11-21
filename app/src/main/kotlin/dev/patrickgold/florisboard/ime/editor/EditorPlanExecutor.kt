package dev.patrickgold.florisboard.ime.editor

import android.os.SystemClock
import android.view.InputDevice
import android.view.KeyCharacterMap
import android.view.KeyEvent
import android.view.inputmethod.InputConnection
import com.jtn0123.audiowhisper.rust.*
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
    private val uptimeMs: () -> Long = SystemClock::uptimeMillis,
) {
    private var loadedComposer: Composer? = null
    private fun supplyComposer(session: EditorCalls) {
        val current = composer()
        if (loadedComposer !== current) {
            val rules = (current as? WithRules)?.let { table ->
                table.ruleOrder.flatMap { listOf(it, table.rules.getValue(it)) }
            }.orEmpty()
            session.call(EditorOp.COMPOSER_RULES, *rules.toTypedArray())
            loadedComposer = current
        }
    }
    suspend fun execute(session: EditorSession, ic: InputConnection, operation: EditorOp, fields: Array<out String>): Boolean =
        session.transaction { executePlan(it, ic, operation, fields) }

    private suspend fun executePlan(session: EditorCalls, ic: InputConnection, operation: EditorOp, fields: Array<out String>): Boolean {
        supplyComposer(session)
        var response = session.call(operation, *fields)
        while (response[EditorPlan.STATUS] != EditorReply.READY) {
            val query = EditorOp.entries.firstOrNull { it.code == response[EditorPlan.STATUS] } ?: error("Unknown native editor query")
            val values = when (query) {
                EditorOp.READ -> arrayOf(
                    if (WireFlag.decode(response[EditorReadQuery.BEFORE])) ic.getTextBeforeCursor(256, 0)?.toString().orEmpty() else "",
                    ic.getTextAfterCursor(128, 0)?.toString().orEmpty(),
                    if (WireFlag.decode(response[EditorReadQuery.SELECTED])) ic.getSelectedText(0)?.toString().orEmpty() else "")
                EditorOp.WORD -> word(response[EditorWordQuery.BEFORE], response[EditorWordQuery.LAST_COMMIT].toInt())
                    .let { arrayOf(it.start.toString(), it.end.toString()) }
                EditorOp.CHARACTER -> arrayOf(breaks.measureUChars(response[EditorTextQuery.TEXT], 1, locale()).toString())
                EditorOp.SPACING -> response[EditorTextQuery.TEXT].let { text -> arrayOf(
                    text.takeLast(breaks.measureLastUChars(text, 1, locale())),
                    text.takeLast(breaks.measureLastUChars(text, 3, locale()))) }
                EditorOp.MEASURE, EditorOp.EXTENT -> with(EditorMeasure) {
                    val (text, count) = response[TEXT] to response[COUNT].toInt()
                    val length = when (WireFlag.decode(response[FORWARD]) to WireFlag.decode(response[WORDS])) {
                        false to false -> breaks.measureLastUChars(text, count, locale())
                        false to true -> breaks.measureLastUWords(text, count, locale())
                        true to false -> breaks.measureUChars(text, count, locale())
                        else -> breaks.measureUWords(text, count, locale())
                    }
                    arrayOf(length.toString())
                }
                else -> error("Unknown native editor query")
            }
            response = session.call(query, *values)
        }
        val results = mutableListOf<String>()
        val eventTime = uptimeMs()
        for (op in response.drop(EditorPlan.SIZE).chunked(EditorEffectRow.SIZE)) {
            val (effect, value) = op[EditorEffectRow.EFFECT] to op[EditorEffectRow.VALUE]
            val a = op[EditorEffectRow.FIRST].toInt()
            val b = op[EditorEffectRow.SECOND].toInt()
            val result = when (effect) {
                EditorEffect.BEGIN -> ic.beginBatchEdit()
                EditorEffect.END -> ic.endBatchEdit()
                EditorEffect.FINISH -> ic.finishComposingText()
                EditorEffect.COMPOSING -> ic.setComposingRegion(a, b)
                EditorEffect.COMPOSING_TEXT -> ic.setComposingText(value, a)
                EditorEffect.COMMIT -> ic.commitText(value, a)
                EditorEffect.DELETE -> ic.deleteSurroundingText(a, b)
                EditorEffect.SELECTION -> ic.setSelection(a, b)
                EditorEffect.ACTION -> ic.performEditorAction(a)
                EditorEffect.CONTEXT_MENU -> ic.performContextMenuAction(a)
                EditorEffect.DOWN, EditorEffect.UP -> ic.sendKeyEvent(KeyEvent(eventTime,
                    if (effect == EditorEffect.DOWN) eventTime else uptimeMs(),
                    if (effect == EditorEffect.DOWN) KeyEvent.ACTION_DOWN else KeyEvent.ACTION_UP,
                    a, value.toIntOrNull() ?: 0, b, KeyCharacterMap.VIRTUAL_KEYBOARD, 0,
                    KeyEvent.FLAG_SOFT_KEYBOARD or KeyEvent.FLAG_KEEP_TOUCH_MODE, InputDevice.SOURCE_KEYBOARD))
                else -> error("Unknown native InputConnection operation")
            }
            results.add(WireFlag.encode(result))
        }
        return WireFlag.decode(session.call(EditorOp.RESULTS, *results.toTypedArray())[0])
    }
}
