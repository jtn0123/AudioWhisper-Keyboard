package dev.patrickgold.florisboard.ime.input

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.InputRules
import com.jtn0123.audiowhisper.rust.InputProtocol.*
import dev.patrickgold.florisboard.ime.input.reference.ReferenceInputControl
import kotlin.random.Random
import kotlin.test.*

/** The reference's answers also go on golden tapes (docs/phase6-kotlin-references.md). */
class InputControlMatchTest {
    @Test fun `active subtype fallback and cyclic switching match duplicate missing and empty lists`() {
        val tape=GoldenTape.ofReference("input-control-subtypes")
        val r=Random(20261113)
        repeat(20000) {
            val ids=List(r.nextInt(20)){r.nextInt(8).toLong()}
            val active=r.nextInt(10).toLong()
            val subtype=ReferenceInputControl.subtype(active,-1,ids)
            tape.answer("subtype",subtype.toList())
            assertContentEquals(subtype,InputRules.control(longArrayOf(CONTROL_SUBTYPE.toLong(),active,-1,*ids.toLongArray())))
            assertContentEquals(longArrayOf(ids.indexOf(active).toLong()), InputRules.control(
                longArrayOf(CONTROL_FIND_SUBTYPE.toLong(),active,*ids.toLongArray())))
            val flags=ids.map{it==active}
            for(direction in listOf(-1L,1L)) {
                val cycle=ReferenceInputControl.cycle(flags,direction)
                tape.answer("cycle",cycle.toList())
                assertContentEquals(cycle,InputRules.control(longArrayOf(CONTROL_CYCLE.toLong(),direction,*flags.map{if(it)1L else 0L}.toLongArray())))
            }
        }
        tape.finish()
    }
    @Test fun `start finish evaluator snapshots and space recipes match`() {
        val tape=GoldenTape.ofReference("input-control-states")
        val r=Random(20261113)
        repeat(20000) {
            val raw=r.nextLong();val hide=r.nextBoolean();val start=r.nextInt(-1,20);val end=r.nextInt(-1,20)
            for(op in listOf(CONTROL_EVALUATOR,CONTROL_START,CONTROL_FINISH)) {
                val v=if(op==CONTROL_START)longArrayOf(op.toLong(),raw,if(hide)1 else 0,start.toLong(),end.toLong()) else longArrayOf(op.toLong(),raw)
                val state=ReferenceInputControl.state(op,raw,hide,start,end)
                tape.answer("state",state)
                assertEquals(state,InputRules.control(v)[0])
            }
            val mode=r.nextInt(11).toLong();val bits=List(5){r.nextBoolean()}
            val text=listOf("a ",". ","? ","  ","x","😀","\n ").random(r)
            val space=ReferenceInputControl.space(mode,bits[0],bits[1],bits[2],bits[3],bits[4],text)
            tape.answer("space",space.toList())
            assertContentEquals(space,InputRules.control(longArrayOf(CONTROL_SPACE.toLong(),mode,*bits.map{if(it)1L else 0L}.toLongArray()),text))
        }
        tape.finish()
    }
}
