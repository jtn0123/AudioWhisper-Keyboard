package dev.patrickgold.florisboard.ime.input.reference

import dev.patrickgold.florisboard.ime.text.key.KeyCode

/** Original feedback branches, substituting recorded Android effect/constant values. */
internal object ReferenceInputFeedback {
    fun plan(v:LongArray):LongArray {
        val audioFactor=when(v[0].toInt()) {0->1.0;1,3->0.7;else->0.4}
        val hapticFactor=when(v[0].toInt()) {0->1.0;1,3->0.4;else->0.05}
        var effect = -1L
        val volume=(v[8]*audioFactor)/100.0
        if(v[2]!=0L && v[4]!=0L) {
            if(v[6]==0L || v[7]!=0L) {
                if(volume in 0.01..1.00) effect=when(v[1].toInt()) {
                    KeyCode.DELETE->7;KeyCode.ENTER->8;KeyCode.SPACE->6;else->5
                }
            }
        }
        var haptic = -1L
        if(v[3]!=0L && v[5]!=0L && v[11]!=0L) {
            haptic=if(v[9]!=0L) {if(hapticFactor<1.0)9 else 3} else 0
        }
        return longArrayOf(effect,volume.toFloat().toBits().toLong(),haptic,hapticFactor.toBits(),v[10])
    }
}
