package com.jtn0123.audiowhisper.dictation

import dev.patrickgold.florisboard.R
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertSame
import kotlin.test.assertTrue

class ModelHolderTest {
    private class Model : AutoCloseable {
        var closed = false
        override fun close() {
            closed = true
        }
    }

    @Test
    fun `loads once and hands back the same model`() {
        var loads = 0
        val holder = ModelHolder { loads++; Model() }
        assertSame(holder.get(), holder.get())
        assertEquals(1, loads)
    }

    @Test
    fun `a failed load is not repeated for every phrase`() {
        var loads = 0
        val holder = ModelHolder<Model> { loads++; throw UnsatisfiedLinkError("no libsherpa-onnx-jni.so") }
        val first = assertFailsWith<ModelLoadException> { holder.get() }
        val second = assertFailsWith<ModelLoadException> { holder.get() }
        assertSame(first, second)
        assertTrue(first.cause is UnsatisfiedLinkError)
        assertEquals(1, loads)
    }

    @Test
    fun `retry tries the load again`() {
        var broken = true
        val holder = ModelHolder { if (broken) throw IllegalArgumentException("bad encoder") else Model() }
        assertFailsWith<ModelLoadException> { holder.get() }
        broken = false
        assertFailsWith<ModelLoadException> { holder.get() }
        holder.retry()
        holder.get()
    }

    @Test
    fun `close releases the model and the next get loads a new one`() {
        val holder = ModelHolder { Model() }
        assertFalse(holder.close())
        val first = holder.get()
        assertTrue(holder.close())
        assertTrue(first.closed)
        assertFalse(holder.close())
        assertFalse(holder.get() === first)
    }

    @Test
    fun `knows whether the next get has to load`() {
        val holder = ModelHolder { Model() }
        assertFalse(holder.isLoaded)
        holder.get()
        assertTrue(holder.isLoaded)
        holder.close()
        assertFalse(holder.isLoaded)
        val failing = ModelHolder<Model> { throw OutOfMemoryError() }
        assertFailsWith<ModelLoadException> { failing.get() }
        assertFalse(failing.isLoaded)
    }

    @Test
    fun `an ended dictation explains itself only when nothing was typed`() {
        assertEquals(null, endingMessage(typedAny = true, failures = 0))
        assertEquals(null, endingMessage(typedAny = true, failures = 2))
        assertEquals(R.string.dictation__not_caught, endingMessage(typedAny = false, failures = 0))
        assertEquals(R.string.dictation__failed, endingMessage(typedAny = false, failures = 1))
    }
}
