package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.random.Random

/**
 * The `chars` crate against this JVM's `java.lang`: [JavaTextComparison] over every character
 * and code point and random strings of the hard ones. The same comparison runs on a phone, whose
 * Java answers some characters differently (see lib/rust/crates/chars/src/tables.rs).
 */
class JavaTextTest {
    private fun assertNoFailures(comparison: JavaTextComparison) {
        assertEquals(emptyList(), comparison.examples, "${comparison.failures} of ${comparison.checks} failed")
    }

    @Test
    fun everyCharAnswersAsJavaDoes() = assertNoFailures(JavaTextComparison(sigmaExact = false).apply { everyChar() })

    @Test
    fun everyCodePointAnswersAsJavaDoes() =
        assertNoFailures(JavaTextComparison(sigmaExact = false).apply { everyCodePoint() })

    @Test
    fun everyCodePointCasesAsJavaDoes() =
        assertNoFailures(JavaTextComparison(sigmaExact = false).apply { everyCodePointAsAString() })

    @Test
    fun randomStringsOfTheHardestCharactersCaseAsJavaDoes() =
        assertNoFailures(JavaTextComparison(sigmaExact = false).apply { randomStrings(Random(SEED), 300_000) })

    @Test
    fun numbersParseAsKotlinDoes() =
        assertNoFailures(JavaTextComparison(sigmaExact = false).apply { numbers(Random(SEED), 100_000) })

    /** The one thing compared loosely: Rust takes ICU's rule for a final Σ, not the JVM's. */
    @Test
    fun sigmaFollowsIcu() {
        assertEquals("οδος οδος.", JavaText.lowercase("ΟΔΟΣ ΟΔΟΣ."))
        assertEquals("ασ'α", JavaText.lowercase("ΑΣ'Α"))
        assertEquals("α'ς", JavaText.lowercase("Α'Σ"))
    }

    @Test
    fun oddUtf16PassesThroughUnchanged() {
        val odd = "a\uD800b 😀 \u0000﻿"
        assertEquals(odd.uppercase(), JavaText.uppercase(odd))
        assertEquals(odd.lowercase(), JavaText.lowercase(odd))
        assertNull(JavaText.toIntOrNull(odd))
    }

    private companion object {
        const val SEED = 20261108L
    }
}
