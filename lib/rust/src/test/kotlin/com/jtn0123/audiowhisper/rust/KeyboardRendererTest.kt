package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

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
    fun closedHandleThrowsInsteadOfUsingFreedMemory() {
        val renderer = KeyboardRenderer(emptyArray())
        renderer.close()
        renderer.close()
        assertFailsWith<IllegalArgumentException> { renderer.touch(intArrayOf(3, 0), longArrayOf(), floatArrayOf()) }
    }
}
