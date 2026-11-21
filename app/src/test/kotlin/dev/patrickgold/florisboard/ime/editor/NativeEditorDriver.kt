package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.rust.EditorSession
import kotlinx.coroutines.runBlocking
import java.util.Locale

/** The app's side of an InputConnection, as the native editor's plans are run against it. */
interface PlanConnection {
    /** Everything in the field. */
    val text: String
    fun getTextBeforeCursor(n: Int, flags: Int): String?
    fun getTextAfterCursor(n: Int, flags: Int): String?
    fun getSelectedText(flags: Int): String?
    fun apply(op: FakeInputConnection.Operation): Boolean
    /** The cursor updates (selection, then composing span) the app sends for what it just did. */
    fun cursorUpdates(): List<IntArray>
}

/** Only platform replies and verbatim InputConnection execution; no native expected-state model. */
class NativeEditorDriver<C : PlanConnection>(val ic: C) : AutoCloseable {
    val session=EditorSession()
    val breaks=ReferenceBreaks()
    init {session.call(EditorOp.COMPOSER_RULES,"a\u0301","á")}
    var raw=false
    var enabled=true
    var determine=true
    var phantom=false
    var input=0x4001
    fun configure() {session.call(EditorOp.CONFIG,if(raw)"1" else "0",if(enabled)"1" else "0",if(determine)"1" else "0",if(phantom)"1" else "0",input.toString())}
    fun run(operation: EditorOp, vararg fields: String): Boolean = runBlocking {
        configure()
        var response=session.call(operation,*fields)
        while(response[EditorPlan.STATUS]!=EditorReply.READY) {
            val query=EditorOp.entries.firstOrNull{it.code==response[EditorPlan.STATUS]}
            val args=when(query) {
                EditorOp.READ -> arrayOf(if(WireFlag.decode(response[EditorReadQuery.BEFORE]))ic.getTextBeforeCursor(256,0).orEmpty()else "",ic.getTextAfterCursor(128,0).orEmpty(),if(WireFlag.decode(response[EditorReadQuery.SELECTED]))ic.getSelectedText(0).orEmpty()else "")
                EditorOp.WORD -> ReferencePlatform.word(response[EditorWordQuery.BEFORE]).let{arrayOf(it.start.toString(),it.end.toString())}
                EditorOp.CAPS -> arrayOf(ReferenceCaps.getCapsMode(response[1],response[2].toInt(),response[3].toInt()).toString())
                EditorOp.CHARACTER -> arrayOf(breaks.measureUChars(response[EditorTextQuery.TEXT],1).toString())

                EditorOp.MEASURE,EditorOp.EXTENT -> with(EditorMeasure){arrayOf(breaks.measure(response[TEXT],response[COUNT].toInt(),Locale.ENGLISH,!WireFlag.decode(response[FORWARD]),WireFlag.decode(response[WORDS])).toString())}
                EditorOp.SPACING -> response[EditorTextQuery.TEXT].let{t->arrayOf(t.takeLast(breaks.measureLastUChars(t,1)),t.takeLast(breaks.measureLastUChars(t,3)))}
                else -> error("Unhandled query ${response[EditorPlan.STATUS]}")
            }
            response=session.call(query,*args)
        }
        // Check each generated plan independently, including nested high-level batches.
        var depth = 0
        val effects=response.drop(EditorPlan.SIZE).chunked(EditorEffectRow.SIZE)
        for (operation in effects) {
            when (operation[EditorEffectRow.EFFECT]) { EditorEffect.BEGIN -> depth++; EditorEffect.END -> depth-- }
            kotlin.test.assertTrue(depth >= 0, "Editor plan ends an unopened batch")
        }
        kotlin.test.assertEquals(0, depth, "Editor plan leaks a batch")
        val results=effects.map { with(EditorEffectRow){ic.apply(FakeInputConnection.Operation(it[EFFECT],it[FIRST].toInt(),it[SECOND].toInt(),it[VALUE]))} }
        WireFlag.decode(session.call(EditorOp.RESULTS,*results.map(WireFlag::encode).toTypedArray())[0])
    }
    fun snapshot(): List<String> = session.call(EditorOp.SNAPSHOT).toList()
    override fun close()=session.close()
}
