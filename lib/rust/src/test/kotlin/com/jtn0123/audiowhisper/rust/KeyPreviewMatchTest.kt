package com.jtn0123.audiowhisper.rust

import kotlin.math.ceil
import kotlin.math.roundToInt
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertTrue

class KeyPreviewMatchTest {
    /** Frozen Material Box(height(36.dp).widthIn(min=36.dp)) intrinsic label rule. */
    @Test fun intrinsicBoxMatchesComposeMinimum() {
        for (d in listOf(.75f,1f,2f,2.625f,3f)) for (w in listOf(0f,10f,36f,80.5f,200f)) {
            val h = (36f*d).roundToInt()
            assertContentEquals(intArrayOf(maxOf(h,ceil(w*d).toInt()),h),ImeChromeRules.previewSize(floatArrayOf(d,w*d)))
        }
    }
    @Test fun realCpuPixelsThroughJniAndThemeResolution() {
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key":{"background":"#ff0000","foreground":"#000000","shape":"rounded-corner(4dp, 4dp, 4dp, 4dp)","font-size":"16sp"}}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            val pixels=r.keyPreview(97,arrayOf("character","",""))
            assertContentEquals(intArrayOf(36,36),pixels.take(2).toIntArray())
            assertTrue(pixels.drop(2).any { it == 0xffff0000.toInt() })
            assertTrue(pixels.drop(2).any { it == 0 })
        }
    }
}
