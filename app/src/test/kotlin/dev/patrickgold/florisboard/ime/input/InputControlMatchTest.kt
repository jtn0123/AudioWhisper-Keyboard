package dev.patrickgold.florisboard.ime.input

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.InputRules
import com.jtn0123.audiowhisper.rust.InputProtocol.*
import kotlin.random.Random
import kotlin.test.*

/** Rust's answers are checked against golden tapes of the frozen reference's (docs/phase6-kotlin-references.md). */
class InputControlMatchTest {
    @Test fun `active subtype fallback and cyclic switching match duplicate missing and empty lists`() {
        GoldenTape.check("input-control-subtypes") { tape ->
            val r=Random(20261113)
            repeat(20000) {
                val ids=List(r.nextInt(20)){r.nextInt(8).toLong()}
                val active=r.nextInt(10).toLong()
                tape.answer("subtype",InputRules.control(longArrayOf(CONTROL_SUBTYPE.toLong(),active,-1,*ids.toLongArray())).toList())
                assertContentEquals(longArrayOf(ids.indexOf(active).toLong()), InputRules.control(
                    longArrayOf(CONTROL_FIND_SUBTYPE.toLong(),active,*ids.toLongArray())))
                val flags=ids.map{it==active}
                for(direction in listOf(-1L,1L)) {
                    tape.answer("cycle",InputRules.control(longArrayOf(CONTROL_CYCLE.toLong(),direction,*flags.map{if(it)1L else 0L}.toLongArray())).toList())
                }
            }
        }
    }
    @Test fun `start finish evaluator snapshots and space recipes match`() {
        GoldenTape.check("input-control-states") { tape ->
            val r=Random(20261113)
            repeat(20000) {
                val raw=r.nextLong();val hide=r.nextBoolean();val start=r.nextInt(-1,20);val end=r.nextInt(-1,20)
                for(op in listOf(CONTROL_EVALUATOR,CONTROL_START,CONTROL_FINISH)) {
                    val v=if(op==CONTROL_START)longArrayOf(op.toLong(),raw,if(hide)1 else 0,start.toLong(),end.toLong()) else longArrayOf(op.toLong(),raw)
                    tape.answer("state",InputRules.control(v)[0])
                }
                val mode=r.nextInt(11).toLong();val bits=List(5){r.nextBoolean()}
                val text=listOf("a ",". ","? ","  ","x","😀","\n ").random(r)
                tape.answer("space",InputRules.control(longArrayOf(CONTROL_SPACE.toLong(),mode,*bits.map{if(it)1L else 0L}.toLongArray()),text).toList())
            }
        }
    }
}
