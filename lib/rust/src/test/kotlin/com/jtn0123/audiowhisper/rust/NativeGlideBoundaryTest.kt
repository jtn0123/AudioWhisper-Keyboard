package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

class NativeGlideBoundaryTest {
    @Test fun mismatchedCostsAreExceptionsAndNoContextDoesNotDereferenceClosedHandles() {
        assertContentEquals(intArrayOf(), NativeGlideNative.rank(0, 0, arrayOf(), floatArrayOf(), arrayOf(), false))
        assertContentEquals(intArrayOf(0), NativeGlideNative.rank(0, 0, arrayOf("a"), floatArrayOf(0f), arrayOf("b"), false))
        assertFailsWith<IllegalArgumentException> {
            NativeGlideNative.rank(-1, 0, arrayOf("a"), floatArrayOf(), arrayOf(), false)
        }
        assertFailsWith<IllegalArgumentException> {
            NativeGlideNative.rank(-1, 0, arrayOf(), floatArrayOf(0f), arrayOf(), false)
        }
        assertFailsWith<IllegalArgumentException> {
            NativeGlideNative.rank(0, 0, arrayOf("a", "b"), floatArrayOf(0f, 0f), arrayOf("c"), false)
        }
    }
}
