package dev.patrickgold.florisboard.ime.input

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import com.jtn0123.audiowhisper.rust.InputProtocol.*
import dev.patrickgold.florisboard.ime.editor.EditorFlowMatchTest
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import kotlin.random.Random
import kotlin.test.*

/** Rust's routing and dispatch answers are checked against golden tapes of the frozen references' (docs/phase6-kotlin-references.md). */
class InputDispatchMatchTest {
    private val repeatable=intArrayOf(KeyCode.ARROW_DOWN,KeyCode.ARROW_LEFT,KeyCode.ARROW_RIGHT,KeyCode.ARROW_UP,
        KeyCode.DELETE,KeyCode.FORWARD_DELETE,KeyCode.UNDO,KeyCode.REDO)
    private val codes=KeyModels.codes().distinct().toIntArray()+intArrayOf(97,101,46,0x1f600)
    private fun bit(value:Boolean)=if(value)1L else 0L

    internal fun route(tape:GoldenTape,input:LongArray,editor:EditorFlowMatchTest.FlowPair?=null):Long {
        val v=if(input.size==12)input+longArrayOf((input[3] ushr 15) and 1,(input[3] ushr 16) and 1,(input[3] ushr 17) and 1) else input
        val actual=InputRules.route(v)
        tape.answer("route",actual.toList())
        var state=v[3]
        for(offset in actual.indices.step(3)) {
            val action=actual[offset].toInt();val arg=actual[offset+1]
            when(action) {
                STATE->state=KeyModels.write(state,(arg ushr 32).toInt(),arg.toInt())
                INCOGNITO->state=KeyModels.write(state,9,(arg ushr 1).toInt())
                COMMIT_CHAR,COMMIT_TEXT->editor?.run(if(action==COMMIT_CHAR)EditorOp.TYPE_CHAR else EditorOp.TYPE_TEXT,if(v[1] in 32..0x10ffff) String(Character.toChars(v[1].toInt())) else "x")
                DELETE_BACK,DELETE_FORWARD->editor?.run(if(action==DELETE_BACK)EditorOp.BACKWARD else EditorOp.FORWARD,arg.toString())
                SPACE_KEY->editor?.run(EditorOp.TYPE_TEXT," ")
                ENTER_KEY->editor?.run(EditorOp.ENTER)
                SELECT_ALL->editor?.run(EditorOp.SELECT_ALL)
            }
        }
        tape.answer("state",state)
        editor?.update()
        return state
    }

    @Test fun `all routing codes and real key types match the frozen receiver`() {
        GoldenTape.check("input-routing-codes") { tape -> for(code in codes) for(phase in 0..3) for(mode in 0..10) for(kind in listOf(0,2,7)) {
            route(tape,longArrayOf(phase.toLong(),code.toLong(),kind.toLong(),mode.toLong(),0,0,0,1,0,3,-1,bit(code in 65..122)))
        } }
    }

    @Test fun `GIF search owns editing keys without reaching the app editor`() {
        GoldenTape.check("input-routing-gif") { tape -> for(code in codes) for(phase in 0..3) for(gif in 0L..2L) for(shift in 0L..3L) {
            route(tape,longArrayOf(phase.toLong(),code.toLong(),if(code>0)0 else 2,shift shl 8,0,0,0,1,0,3,gif,0))
        } }
    }

    @Test fun `50000 session events over 200 seeds compare lifecycle routing editor effects and state`() {
        var events=0
        var editorOperations=0
        GoldenTape.check("input-dispatch-sessions") { tape -> for(seed in 0 until 200) InputDispatch().use { native ->
            val r=Random(20261113+seed)
            native.call(INIT,*repeatable.map(Int::toLong).toLongArray())
            var now=0L
            var state=0L
            fun call(op:Int,vararg args:Long):LongArray {
                val actual=native.call(op,*args)
                tape.answer("call $op",actual.toList())
                return actual
            }
            EditorFlowMatchTest.FlowPair(tape).use { editor ->
                repeat(250) { index ->
                    now+=listOf(0L,1L,49L,50L,299L,300L,301L,749L,750L,751L).random(r)
                    val code=if(index%11==0)KeyCode.SHIFT else codes.random(r)
                    val operation=listOf(DOWN,UP,CANCEL,DOWN_UP,HOLD_RESULT,REPEAT_RESULT).random(r)
                    val plan=when(operation) {
                        DOWN->call(DOWN,code.toLong(),now,1)
                        HOLD_RESULT->call(HOLD_RESULT,code.toLong(),bit(r.nextBoolean()),bit(r.nextBoolean()),call(FACTS,code.toLong(),now,300)[7])
                        REPEAT_RESULT->call(REPEAT_RESULT,code.toLong(),bit(r.nextBoolean()),call(FACTS,code.toLong(),now,300)[7])
                        else->call(operation,code.toLong(),now)
                    }
                    call(DELAYS,code.toLong(),r.nextInt(100,600).toLong(),50,bit(r.nextBoolean()))
                    val facts=call(FACTS,code.toLong(),now,300)
                    val shift=call(FACTS,KeyCode.SHIFT.toLong(),now,300)[0]
                    fun deliver(phase:Int,deliveredCode:Int=code) {
                        val v=longArrayOf(phase.toLong(),deliveredCode.toLong(),if(deliveredCode>0)0 else 2,state,
                            shift,facts[1],facts[2],facts[4],bit(r.nextBoolean()),r.nextInt(5).toLong(),-1,bit(deliveredCode in 65..122))
                        state=route(tape,v,editor)
                    }
                    if(plan.isNotEmpty()) when(plan[0].toInt()) {
                        EMIT_DOWN->{deliver(PHASE_DOWN);call(ACK_DOWN,code.toLong(),now)}
                        EMIT_UP->{deliver(PHASE_UP);call(ACK_UP,code.toLong(),now)}
                        EMIT_CANCEL->deliver(PHASE_CANCEL)
                        EMIT_DOWN_UP->{deliver(PHASE_DOWN);call(ACK_DOWN,code.toLong(),now);deliver(PHASE_UP);call(ACK_UP,code.toLong(),now)}
                        EMIT_REPEAT->deliver(PHASE_REPEAT,plan[1].toInt())
                    }
                    // External mode/subtype refreshes may arrive while several keys are down.
                    if(index%17==0) state=KeyModels.write(state,0,r.nextInt(8))
                    if(index%29==0) state=KeyModels.write(state,3,r.nextInt(3))
                    events++
                }
                editorOperations+=editor.pair.operationCount
            }
        } }
        assertEquals(50000,events)
        println("input: events=$events seeds=200 editor_operations=$editorOperations differences=0")
    }

    @Test fun `double tap timeout and repeat delete start boundaries are observable`() {
        InputDispatch().use { d ->
            d.call(INIT,KeyCode.DELETE.toLong())
            d.call(DOWN,KeyCode.SHIFT.toLong(),100,1);d.call(ACK_DOWN,KeyCode.SHIFT.toLong(),100)
            assertEquals(1L,d.call(FACTS,KeyCode.SHIFT.toLong(),399,300)[2])
            assertEquals(0L,d.call(FACTS,KeyCode.SHIFT.toLong(),400,300)[2])
            assertEquals(0L,d.call(FACTS,KeyCode.SHIFT.toLong(),401,300)[2])
            assertContentEquals(longArrayOf(300,50,KeyCode.DELETE.toLong()),d.call(DELAYS,KeyCode.DELETE.toLong(),300,50,0))
            assertContentEquals(longArrayOf(300,250,KeyCode.DELETE_WORD.toLong()),d.call(DELAYS,KeyCode.DELETE.toLong(),300,50,1))
        }
    }
    @Test fun `cancelled timer cannot block or repeat a newer press of the same key`() {
        InputDispatch().use { d ->
            d.call(INIT,KeyCode.DELETE.toLong())
            val first=d.call(DOWN,KeyCode.DELETE.toLong(),1,1)[4]
            d.call(CANCEL,KeyCode.DELETE.toLong(),2)
            val next=d.call(DOWN,KeyCode.DELETE.toLong(),3,1)[4]
            assertNotEquals(first,next)
            assertTrue(d.call(HOLD_RESULT,KeyCode.DELETE.toLong(),1,0,first).isEmpty())
            assertContentEquals(longArrayOf(0),d.call(REPEAT_RESULT,KeyCode.DELETE.toLong(),1,first))
            assertEquals(EMIT_UP.toLong(),d.call(UP,KeyCode.DELETE.toLong(),4)[0])
        }
    }
    @Test fun `closed lifecycle and malformed packets reject access`() {
        val d=InputDispatch();d.close();d.close()
        assertFailsWith<IllegalStateException>{d.call(FACTS,0,0,300)}
        assertFailsWith<IllegalArgumentException>{InputRules.route(longArrayOf())}
        assertFailsWith<IllegalArgumentException>{InputRules.feedback(longArrayOf())}
        InputDispatch().use {assertFailsWith<IllegalArgumentException>{it.call(DOWN)}}
    }
}
