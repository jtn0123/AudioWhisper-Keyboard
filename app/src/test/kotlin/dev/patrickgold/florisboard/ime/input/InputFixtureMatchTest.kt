package dev.patrickgold.florisboard.ime.input

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.rust.InputProtocol.*
import dev.patrickgold.florisboard.ime.editor.EditorFlowMatchTest
import dev.patrickgold.florisboard.ime.input.reference.ReferenceInputDispatcher
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import kotlin.test.*

/** Logical keys from the public harness traces, between the separately matched touch/decoder
 * and editor engines. Provider replies are fixed synthetic fixtures, as in EditorFlowMatchTest. The
 * references' answers also go on golden tapes (docs/phase6-kotlin-references.md). */
class InputFixtureMatchTest {
    private class Trace(val tape: GoldenTape) : AutoCloseable {
        val editor=EditorFlowMatchTest.FlowPair(tape)
        val native=InputDispatch()
        val reference=ReferenceInputDispatcher(intArrayOf(KeyCode.DELETE))
        val compare=InputDispatchMatchTest()
        var time=1000L
        var state=0L
        init {native.call(INIT,KeyCode.DELETE.toLong())}
        fun call(op:Int,vararg args:Long):LongArray {
            val expected=reference.call(op,*args);val actual=native.call(op,*args)
            tape.answer("call $op",expected.toList())
            assertTrue(expected.contentEquals(actual),"Fixture lifecycle differs")
            return actual
        }
        fun deliver(phase:Int,code:Int) {
            val facts=call(FACTS,code.toLong(),time,300)
            val shift=call(FACTS,KeyCode.SHIFT.toLong(),time,300)[0]
            state=compare.route(tape,longArrayOf(phase.toLong(),code.toLong(),if(code>0)0 else 2,state,
                shift,facts[1],facts[2],facts[4],0,3,-1,if(Character.isAlphabetic(code))1 else 0),editor)
        }
        fun down(code:Int):Long {val p=call(DOWN,code.toLong(),time,1);if(p.isNotEmpty()){deliver(PHASE_DOWN,code);call(ACK_DOWN,code.toLong(),time)};return p.getOrElse(4){0}}
        fun up(code:Int) {val p=call(UP,code.toLong(),time);if(p.isNotEmpty()) {
            deliver(if(p[0]==EMIT_UP.toLong())PHASE_UP else PHASE_CANCEL,code)
            if(p[0]==EMIT_UP.toLong())call(ACK_UP,code.toLong(),time)
        }}
        fun press(code:Int,gap:Long=85) {time+=gap;down(code);time+=35;up(code)}
        fun type(value:String) {value.codePoints().forEach{press(it)}}
        override fun close() {native.close();editor.close()}
    }
    @Test fun `text payloads from every digest harness survive the shared dispatch and editor chain`() {
        // Public fixed fixtures in regression, suggestions, editor, modes, panels, clipboard/GIF and chrome.
        val payloads=listOf("teh","helo","Hi. A","hello","XaYl","cat","abcdabcd","aw-test-2",
            "😀😃😄😁👋🏻","12@#~3e","~`|4πe","123450","12+34e","123*#0","()*#1+","ca1#t")
        val tape=GoldenTape.ofReference("input-fixture-payloads")
        for(value in payloads) Trace(tape).use { t ->
            t.type(value)
            assertTrue(t.editor.pair.newIc.text==value,"Harness fixture text differs")
            assertEquals(value.length,t.editor.pair.newIc.start)
        }
        tape.finish()
    }
    @Test fun `public regression popup repeat caps rolling mode and IME action traces match`() {
        val tape=GoldenTape.ofReference("input-fixture-traces")
        Trace(tape).use { t ->
            for(c in "qwerasdfzxcv") repeat(25){t.press(c.code)}
            repeat(8) {
                t.down('e'.code)
                t.time+=500
                val g=t.call(FACTS,'e'.code.toLong(),t.time,300)[7]
                t.call(HOLD_RESULT,'e'.code.toLong(),1,0,g)
                t.call(CANCEL,'e'.code.toLong(),t.time)
                t.press('3'.code)
            }
            assertEquals(308,t.editor.pair.newIc.text.length)
        }
        Trace(tape).use { t ->
            t.press(KeyCode.SHIFT);t.press(KeyCode.SHIFT)
            assertEquals(3,KeyModels.read(t.state,2))
            t.type("AB");t.press(KeyCode.SHIFT);t.type("c")
            assertEquals(0,KeyModels.read(t.state,2))
            t.down('a'.code);t.time+=80
            // Rolling input commits the first suitable key before the next goes down.
            assertTrue(InputRules.pointer(floatArrayOf('a'.code.toFloat())))
            t.up('a'.code);t.down('s'.code);t.time+=80;t.up('s'.code)
            for(code in listOf(KeyCode.VIEW_SYMBOLS,KeyCode.VIEW_SYMBOLS2,KeyCode.VIEW_NUMERIC,
                KeyCode.VIEW_NUMERIC_ADVANCED,KeyCode.VIEW_PHONE,KeyCode.VIEW_PHONE2)) {
                t.press(code);t.press('1'.code);t.press(KeyCode.VIEW_CHARACTERS)
                assertEquals(0,KeyModels.read(t.state,0))
            }
            t.editor.run(EditorOp.GESTURE,"hello");t.editor.update()
            t.editor.run(EditorOp.COMPLETION,"help");t.editor.update()
            t.editor.run(EditorOp.TYPE_TEXT,"alpha beta gamma");t.editor.update()
            val generation=t.down(KeyCode.DELETE)
            t.time+=t.call(DELAYS,KeyCode.DELETE.toLong(),300,50,0)[0]
            val repeat=t.call(HOLD_RESULT,KeyCode.DELETE.toLong(),0,0,generation)
            repeat(40) {
                assertEquals(1L,t.call(REPEAT_RESULT,KeyCode.DELETE.toLong(),1,generation)[0])
                t.deliver(PHASE_REPEAT,repeat[1].toInt());t.time+=50
            }
            t.up(KeyCode.DELETE)
            assertEquals(0,t.editor.pair.newIc.text.length)
            t.press(KeyCode.ENTER)
            t.editor.pair.oldIc.performEditorAction(3)
            t.editor.pair.new.run(EditorOp.ACTION,"3")
            t.editor.pair.compare("search action")
            assertEquals(3,EditorAttributes.enterAction(1,3,false))
            assertEquals(-1,EditorAttributes.enterAction(0x20001,0x40000003,false))
        }
        tape.finish()
    }
}
