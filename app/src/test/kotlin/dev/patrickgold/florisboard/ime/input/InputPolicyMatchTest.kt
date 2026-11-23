package dev.patrickgold.florisboard.ime.input

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.*
import dev.patrickgold.florisboard.ime.input.reference.*
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import kotlin.random.Random
import kotlin.test.*

/** The reference's answers also go on golden tapes (docs/phase6-kotlin-references.md). */
class InputPolicyMatchTest {
    @Test fun `feedback plans match every feature combination and seeded volumes`() {
        val tape=GoldenTape.ofReference("input-policy-feedback")
        val r=Random(20261113)
        for(event in 0..4) for(code in KeyModels.codes()) repeat(128) { mask ->
            val v=longArrayOf(event.toLong(),code.toLong(),mask.toLong() and 1,(mask shr 1).toLong() and 1,
                (mask shr 2).toLong() and 1,(mask shr 3).toLong() and 1,(mask shr 4).toLong() and 1,
                (mask shr 5).toLong() and 1,r.nextInt(-10,111).toLong(),(mask shr 6).toLong() and 1,
                r.nextInt(2).toLong(),r.nextInt(2).toLong())
            val plan=ReferenceInputFeedback.plan(v)
            tape.answer("plan",plan.toList())
            assertContentEquals(plan,InputRules.feedback(v))
        }
        tape.finish()
    }
    @Test fun `rolling input sliding jitter and long press match Compose decisions`() {
        val tape=GoldenTape.ofReference("input-policy-pointer")
        val r=Random(20261113)
        for(code in KeyModels.codes()) {
            val suitable=ReferenceInputPointer.suitable(code)
            tape.answer("suitable",suitable)
            assertEquals(suitable,InputRules.pointer(floatArrayOf(code.toFloat())),"Rolling pointer rule")
            // AudioWhisper: a still space or delete goes up too, so a letter tapped over it comes after it.
            val still=code==KeyCode.SPACE||code==KeyCode.CJK_SPACE||code==KeyCode.DELETE
            assertEquals(ReferenceInputPointer.suitable(code),InputRules.pointer(floatArrayOf(code.toFloat(),0f)),"Moved pointer")
            assertEquals(ReferenceInputPointer.suitable(code)||still,InputRules.pointer(floatArrayOf(code.toFloat(),1f)),"Still pointer")
            for(bits in 0..7) {
                val hold=ReferenceInputPointer.hold(code,bits and 1!=0,bits and 2!=0,bits and 4!=0)
                tape.answer("hold",hold)
                assertEquals(hold,InputRules.hold(code,bits and 1!=0,bits and 2!=0,bits and 4!=0))
            }
        }
        repeat(50000) {
            val left=r.nextFloat()*1000;val top=r.nextFloat()*1000
            val width=r.nextFloat()*300;val height=r.nextFloat()*300
            val v=floatArrayOf(left+(r.nextFloat()*2-0.5f)*width,top+(r.nextFloat()*2-0.5f)*height,left,top,left+width,top+height)
            val slide=ReferenceInputPointer.slide(v)
            tape.answer("slide",slide)
            assertEquals(slide,InputRules.pointer(v))
        }
        tape.finish()
    }
    @Test fun `label and icon choices match raw editor flags shift states modes and key types`() {
        val tape=GoldenTape.ofReference("input-policy-visual")
        val r=Random(20261113)
        for(code in KeyModels.codes()) for(mode in 0..10) for(type in 0..9) {
            val label=ReferenceInputVisual.label(type,code,mode)
            tape.answer("label",label)
            assertEquals(label,InputRules.visual(longArrayOf(type.toLong(),code.toLong(),mode.toLong())))
            val raw=r.nextLong();val input=r.nextInt();val options=r.nextInt()
            val icon=ReferenceInputVisual.icon(code,raw,input,options,mode)
            tape.answer("icon",icon)
            assertEquals(icon,InputRules.visual(longArrayOf(code.toLong(),raw,input.toLong(),options.toLong(),mode.toLong())))
        }
        tape.finish()
    }
}
