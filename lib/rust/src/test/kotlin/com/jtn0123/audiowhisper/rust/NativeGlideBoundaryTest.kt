package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertFailsWith

class NativeGlideBoundaryTest {
    @Test fun malformedPacketsAreExceptionsAndNoContextDoesNotDereferenceClosedHandles() {
        assertContentEquals(intArrayOf(), NativeGlideNative.rank(0, 0, "", intArrayOf(), floatArrayOf(), 0, false))
        assertFailsWith<IllegalArgumentException> {
            NativeGlideNative.rank(-1, 0, "a", intArrayOf(2), floatArrayOf(0f), 1, false)
        }
        assertFailsWith<IllegalArgumentException> {
            NativeGlideNative.rank(-1, 0, "", intArrayOf(), floatArrayOf(0f), 1, false)
        }
        assertFailsWith<IllegalArgumentException> {
            NativeGlideNative.rank(-1, 0, "a", intArrayOf(1), floatArrayOf(0f), 0, false)
        }
    }
}
