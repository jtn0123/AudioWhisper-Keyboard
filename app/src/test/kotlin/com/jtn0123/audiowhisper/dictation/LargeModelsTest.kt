package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.dictation.LargeModels.Kind
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals

class LargeModelsTest {
    private val freed = mutableListOf<Kind>()

    @AfterTest
    fun cleanUp() = Kind.entries.forEach(LargeModels::unregister)

    private fun registerBoth() = Kind.entries.forEach { kind -> LargeModels.register(kind) { freed += kind } }

    @Test
    fun `loading dictation frees the voice, and the reverse`() {
        registerBoth()
        LargeModels.loading(Kind.DICTATION)
        assertEquals(listOf(Kind.VOICE), freed)
        LargeModels.loading(Kind.VOICE)
        assertEquals(listOf(Kind.VOICE, Kind.DICTATION), freed)
    }

    @Test
    fun `nothing is asked of a model that has gone`() {
        registerBoth()
        LargeModels.unregister(Kind.VOICE)
        LargeModels.loading(Kind.DICTATION)
        assertEquals(emptyList(), freed)
    }
}
