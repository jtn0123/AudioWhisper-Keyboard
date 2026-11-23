package dev.patrickgold.florisboard.ime.input

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.InputRules
import com.jtn0123.audiowhisper.rust.InputProtocol.*
import dev.patrickgold.florisboard.ime.input.reference.ReferenceInputDelete
import kotlin.test.*

/** The reference's answers also go on a golden tape (docs/phase6-kotlin-references.md). */
class InputDeleteMatchTest {
    @Test fun `delete continuation preserves ordered effects and short circuit undo behavior`() {
        val tape=GoldenTape.ofReference("input-delete-effects")
        for(bits in 0..31) {
            val words=bits and 1!=0;val forward=bits and 2!=0;val auto=bits and 4!=0
            val dictation=bits and 8!=0;val composing=bits and 16!=0
            val effects=mutableListOf<Long>()
            var stage=0L;var reply=false
            while(stage>=0) {
                val result=InputRules.control(longArrayOf(CONTROL_DELETE.toLong(),stage,if(words)1 else 0,
                    if(reply)1 else 0,if(composing)1 else 0,if(forward)1 else 0))
                stage=result[0]
                effects.addAll(result.drop(1))
                for(offset in (1 until result.size).step(2)) when(result[offset].toInt()) {
                    DELETE_UNDO_AUTO->reply=auto
                    DELETE_UNDO_DICTATION->reply=dictation
                }
            }
            val expected=ReferenceInputDelete.effects(words,forward,auto,dictation,composing)
            tape.answer("effects",expected)
            assertEquals(expected,effects)
        }
        tape.finish()
    }
}
