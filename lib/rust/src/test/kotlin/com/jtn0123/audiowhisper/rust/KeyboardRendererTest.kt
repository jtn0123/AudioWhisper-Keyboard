package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

class KeyboardRendererTest {
    @Test
    fun packedMotionAndGeometryDispatchThroughJni() {
        KeyboardRenderer(emptyArray()).use { renderer ->
            renderer.update(floatArrayOf(100f, 80f, 20f, 16f, 0f),
                IntArray(17),
                floatArrayOf(0f, 0f, 50f, 60f, 0f, 0f, 50f, 60f, 20f, 0f, 1f, 10f, 0f, 97f,
                    50f, 0f, 100f, 60f, 50f, 0f, 100f, 60f, 20f, 0f, 1f, 10f, 0f, 101f),
                arrayOf("a", "", "", "e", "", ""), emptyArray())
            assertContentEquals(intArrayOf(0, 0, 7),
                renderer.touch(intArrayOf(0, 0, 7), longArrayOf(1, 2), floatArrayOf(25f, 40f)))
            assertContentEquals(intArrayOf(2, 0, 7, 0, 1, 7),
                renderer.touch(intArrayOf(2, 0, 7), longArrayOf(1, 2, 3), floatArrayOf(25f, 40f, 75f, 40f)))
            renderer.popup(1, arrayOf("é", "è"))
            assertContentEquals(intArrayOf(3, 0, 7),
                renderer.touch(intArrayOf(1, 0, 7), longArrayOf(1, 4), floatArrayOf(75f, 40f)))
            assertFailsWith<IllegalArgumentException> { renderer.popup(2, arrayOf("é")) }
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
