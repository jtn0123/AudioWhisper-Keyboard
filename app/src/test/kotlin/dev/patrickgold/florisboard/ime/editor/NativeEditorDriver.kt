package dev.patrickgold.florisboard.ime.editor

import com.jtn0123.audiowhisper.rust.EditorSession
import kotlinx.coroutines.runBlocking
import java.util.Locale

/** Only platform replies and verbatim InputConnection execution; no native expected-state model. */
class NativeEditorDriver(val ic: FakeInputConnection) : AutoCloseable {
    val session=EditorSession()
    val breaks=ReferenceBreaks()
    val composer=ReferenceComposer()
    var raw=false
    var enabled=true
    var determine=true
    var phantom=false
    var input=0x4001
    fun configure() {session.call("config",if(raw)"1" else "0",if(enabled)"1" else "0",if(determine)"1" else "0",if(phantom)"1" else "0",input.toString())}
    fun run(operation: String, vararg fields: String): Boolean = runBlocking {
        configure()
        var response=session.call(operation,*fields)
        while(response[0]!="ready") {
            val args=when(response[0]) {
                "read" -> arrayOf(if(response[1]=="1")ic.getTextBeforeCursor(256,0).orEmpty()else "",ic.getTextAfterCursor(128,0).orEmpty(),if(response[2]=="1")ic.getSelectedText(0).orEmpty()else "")
                "word" -> ReferencePlatform.word(response[1]).let{arrayOf(it.start.toString(),it.end.toString())}
                "caps" -> arrayOf(ReferenceCaps.getCapsMode(response[1],response[2].toInt(),response[3].toInt()).toString())
                "character" -> arrayOf(breaks.measureUChars(response[1],1).toString())
                "composer" -> composer.getActions(response[1].takeLast(composer.toRead.coerceAtLeast(response[3].toInt())),response[2]).let{arrayOf(it.first.toString(),it.second)}
                "measure","extent" -> arrayOf(breaks.measure(response[1],response[4].toInt(),Locale.ENGLISH,response[2]=="0",response[3]=="1").toString())
                "spacing" -> arrayOf(response[1].takeLast(breaks.measureLastUChars(response[1],1)),response[1].takeLast(breaks.measureLastUChars(response[1],3)))
                else -> error("Unhandled query ${response[0]}")
            }
            response=session.call(response[0],*args)
        }
        val results=response.drop(2).chunked(4).map { ic.apply(FakeInputConnection.Operation(it[0],it[1].toInt(),it[2].toInt(),it[3])) }
        session.call("results",*results.map{if(it)"1"else"0"}.toTypedArray())[0]=="1"
    }
    fun snapshot(): List<String> = session.call("snapshot").toList()
    override fun close()=session.close()
}
