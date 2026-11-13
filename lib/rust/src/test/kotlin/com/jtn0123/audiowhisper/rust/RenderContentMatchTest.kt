package com.jtn0123.audiowhisper.rust

import kotlin.math.roundToInt
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertContentEquals

/** Frozen TextKeyButton's isTelPadKey and Compose BiasAlignment.align rules. */
private fun referencePhoneLabel(mode: String, kind: String, v: FloatArray, hint: Boolean): FloatArray {
    if (mode != "phone" || kind != "numeric") return v.copyOf(4)
    val bias = if (hint) 0.5f else -0.5f
    val x = v[0] + (((v[2] - v[0] - v[4]) / 2f) * (1f + bias)).roundToInt()
    val y = v[1] + ((v[3] - v[1] - v[5]) / 2f).roundToInt()
    return floatArrayOf(x, y, x + v[4], y + v[5])
}

class RenderContentMatchTest {
    @Test fun phoneDigitsAndLetterHintsMatchComposeBiasAlignment() {
        val random = Random(0x3b05)
        for (mode in listOf("characters", "symbols", "symbols2", "numeric", "numeric_advanced", "phone", "phone2")) {
            for (kind in listOf("numeric", "character", "system_gui")) for (hint in listOf(false, true)) {
                repeat(100) {
                    val v = floatArrayOf(12f, 30f, random.nextInt(200, 900).toFloat(), 180f,
                        random.nextInt(10, 120).toFloat(), random.nextInt(20, 80).toFloat())
                    assertContentEquals(referencePhoneLabel(mode, kind, v, hint),
                        RenderContentRules.phoneLabel(mode, kind, v, hint), "$mode/$kind/$hint")
                }
            }
        }
    }
}
