package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotEquals

class KeyboardRendererTest {
    @Test
    fun packedMotionAndGeometryDispatchThroughJni() {
        KeyboardRenderer(emptyArray()).use { renderer ->
            renderer.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","key":{"font-size":"20sp"}}""",
                emptyArray(), intArrayOf(), floatArrayOf(1f, 1f, 1f))
            renderer.update(floatArrayOf(100f, 80f, 20f),
                arrayOf("characters", "unshifted", "character", "character"), intArrayOf(),
                floatArrayOf(0f, 0f, 50f, 60f, 0f, 0f, 50f, 60f, 1f, 97f,
                    50f, 0f, 100f, 60f, 50f, 0f, 100f, 60f, 1f, 101f),
                arrayOf("a", "", "", "e", "", ""), emptyArray())
            assertContentEquals(intArrayOf(0, 0, 7),
                renderer.touch(intArrayOf(0, 0, 7), longArrayOf(1, 2), floatArrayOf(25f, 40f)))
            assertContentEquals(intArrayOf(2, 0, 7, 0, 1, 7),
                renderer.touch(intArrayOf(2, 0, 7), longArrayOf(1, 2, 3), floatArrayOf(25f, 40f, 75f, 40f)))
            renderer.popup(1, arrayOf("é", "è"), intArrayOf(233, 232))
            assertContentEquals(intArrayOf(3, 0, 7),
                renderer.touch(intArrayOf(1, 0, 7), longArrayOf(1, 4), floatArrayOf(75f, 40f)))
            assertFailsWith<IllegalArgumentException> { renderer.popup(2, arrayOf("é"), intArrayOf(233)) }
        }
    }

    @Test
    fun candidateRemovalAcknowledgesTheHoldWithoutLosingItsRelease() {
        KeyboardRenderer(emptyArray()).use { renderer ->
            renderer.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",
                emptyArray(), intArrayOf(), floatArrayOf(1f, 1f, 1f))
            renderer.smartbar(floatArrayOf(1f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 0f, 1f, 40f),
                IntArray(10), Array(10) { "" })
            renderer.update(floatArrayOf(400f, 200f, 40f), arrayOf("characters", "unshifted"),
                IntArray(3), floatArrayOf(), emptyArray(), arrayOf("one", "two", "three"))
            assertContentEquals(intArrayOf(8, 0, 7), renderer.touch(intArrayOf(0, 0, 7),
                longArrayOf(1, 2), floatArrayOf(50f, 20f)))
            renderer.candidateLongPress(7)
            assertContentEquals(intArrayOf(9, 0, 7), renderer.touch(intArrayOf(1, 0, 7),
                longArrayOf(1, 500_000_000), floatArrayOf(50f, 20f)))
        }
    }

    @Test
    fun candidateHoldRetainsItsOriginalPlatformHandleAcrossSuggestionUpdates() {
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",
                emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            r.smartbar(floatArrayOf(1f,0f,0f,0f,0f,0f,0f,0f,0f,0f,1f,40f),IntArray(10),Array(10) {""})
            r.update(floatArrayOf(400f,200f,40f),arrayOf("characters","unshifted"),
                intArrayOf(0),floatArrayOf(),emptyArray(),arrayOf("fixture"))
            r.candidateBindings(intArrayOf(17))
            r.touch(intArrayOf(0,0,7),longArrayOf(0,1),floatArrayOf(50f,20f))
            assertContentEquals(intArrayOf(17),r.candidateBindings(intArrayOf(23)))
            assertContentEquals(intArrayOf(RenderProtocol.CANDIDATE_HOLD,17,7),r.tick(300_000_001))
            assertContentEquals(intArrayOf(RenderProtocol.CANDIDATE_CANCEL,0,7),
                r.touch(intArrayOf(1,0,7),longArrayOf(0,400_000_000),floatArrayOf(50f,20f)))
            assertContentEquals(intArrayOf(),r.candidateBindings(intArrayOf(23)))
        }
    }

    @Test
    fun platformTooltipFailureDoesNotConsumeAnActionRelease() {
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",
                emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            r.smartbar(floatArrayOf(1f,2f,1f,0f,0f,0f,0f,1f,0f,0f,1f,40f),
                IntArray(10) {if(it%2==1) 1 else -1},Array(10) {""})
            r.update(floatArrayOf(400f,200f,40f),arrayOf("characters","unshifted"),
                intArrayOf(),floatArrayOf(),emptyArray(),emptyArray())
            assertContentEquals(intArrayOf(RenderProtocol.BAR_DOWN,0,7),r.touch(intArrayOf(0,0,7),longArrayOf(0,1),floatArrayOf(20f,20f)))
            assertContentEquals(intArrayOf(RenderProtocol.TOOLTIP_SHOW,0,7),r.tick(300_000_001))
            r.tooltipShown(7,false)
            assertContentEquals(intArrayOf(RenderProtocol.BAR_UP,0,7),r.touch(intArrayOf(1,0,7),longArrayOf(0,400_000_000),floatArrayOf(20f,20f)))
        }
    }

    @Test
    fun frameContinuationEndsWhenTrailExpiresAndHonorsTheVisibilitySetting() {
        for (show in listOf(false,true)) KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            r.update(floatArrayOf(100f,80f,20f),arrayOf("characters","unshifted","character"),intArrayOf(),
                floatArrayOf(0f,0f,100f,60f,0f,0f,100f,60f,1f,97f),arrayOf("a","",""),emptyArray())
            r.gestures(floatArrayOf(1f,32f,1f,0f,0f,if(show) 1f else 0f,200f,32f,1900f),Array(9) {"NO_ACTION"})
            r.touch(intArrayOf(0,0,0),longArrayOf(0,1_000_000),floatArrayOf(10f,40f))
            r.touch(intArrayOf(2,0,0),longArrayOf(0,17_000_000),floatArrayOf(80f,40f))
            kotlin.test.assertEquals(show,r.frame(20_000_000))
            r.touch(intArrayOf(1,0,0),longArrayOf(0,33_000_000),floatArrayOf(80f,40f))
            kotlin.test.assertFalse(r.frame(300_000_000))
            kotlin.test.assertFalse(r.frame(2_300_000_000))
        }
    }

    @Test
    fun theSurfaceKeepsRoomAboveTheKeyboardForATopRowKeysPopup() {
        KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet"}""",emptyArray(),intArrayOf(),floatArrayOf(1f,1f,1f))
            // One key 60 tall at the top, under a strip 20 tall; the desired key is 50 by 60.
            r.update(floatArrayOf(100f,80f,20f,50f,60f),arrayOf("characters","unshifted","character"),intArrayOf(),
                floatArrayOf(0f,0f,50f,60f,0f,0f,50f,60f,1f,101f),arrayOf("e","",""),emptyArray())
            // Portrait: a preview 2.5 keys (150) tall, a second row of trunc(0.4 x 150) = 60 above
            // it, from the key's bottom at 60, less the strip: 150 + 60 - 60 - 20.
            kotlin.test.assertEquals(130,r.headroom(false))
            // Landscape: 3 keys (180) and 72 more: 180 + 72 - 60 - 20.
            kotlin.test.assertEquals(172,r.headroom(true))
        }
    }

    @Test
    fun aPressedKeysBubbleDrawsOnlyWhileTheGestureSettingAsksForIt() {
        val red = 0xffff0000.toInt()
        for (bubble in listOf(false, true)) KeyboardRenderer(emptyArray()).use { r ->
            r.theme("""{"${'$'}schema":"https://schemas.florisboard.org/snygg/v2/stylesheet","window":{"background":"#0000ff"},""" +
                """"key":{"background":"#00ff00"},"key-popup-box":{"background":"#ff0000"}}""", emptyArray(), intArrayOf(),
                floatArrayOf(1f, 1f, 1f))
            // q above a, keys 50 by 60 under a strip 20 tall; the desired key is 50 by 60.
            r.update(floatArrayOf(100f, 140f, 20f, 50f, 60f), arrayOf("characters", "unshifted", "character", "character"),
                intArrayOf(), floatArrayOf(0f, 0f, 50f, 60f, 0f, 0f, 50f, 60f, 1f, 113f,
                    0f, 60f, 50f, 120f, 0f, 60f, 50f, 120f, 1f, 97f), arrayOf("q", "", "", "a", "", ""), emptyArray())
            r.gestures(floatArrayOf(1f, 32f, 0f, 0f, 0f, 0f, 200f, 32f, 1900f, 0f, 0f, if (bubble) 1f else 0f),
                Array(9) { "NO_ACTION" })
            r.touch(intArrayOf(0, 0, 7), longArrayOf(1, 2), floatArrayOf(25f, 110f))
            // Over a's lower half, under the bubble's label: the bubble's box, or the key itself.
            assertEquals(bubble, r.pixels()[100 * 100 + 25] == red)
            r.touch(intArrayOf(1, 0, 7), longArrayOf(1, 3), floatArrayOf(25f, 110f))
            assertNotEquals(red, r.pixels()[100 * 100 + 25], "released, the bubble goes")
        }
    }

    @Test
    fun gestureSettingsTakeTheBubbleAsATwelfthValueAndNoMore() {
        KeyboardRenderer(emptyArray()).use { r ->
            val twelve = floatArrayOf(1f, 32f, 0f, 0f, 0f, 0f, 200f, 32f, 1900f, 0f, 0f, 1f)
            r.gestures(twelve, Array(9) { "NO_ACTION" })
            assertFailsWith<IllegalArgumentException> { r.gestures(twelve + 1f, Array(9) { "NO_ACTION" }) }
        }
    }

    @Test
    fun closedHandleThrowsInsteadOfUsingFreedMemory() {
        val renderer = KeyboardRenderer(emptyArray())
        renderer.close()
        renderer.close()
        assertFailsWith<IllegalArgumentException> { renderer.touch(intArrayOf(3, 0), longArrayOf(), floatArrayOf()) }
    }
}
