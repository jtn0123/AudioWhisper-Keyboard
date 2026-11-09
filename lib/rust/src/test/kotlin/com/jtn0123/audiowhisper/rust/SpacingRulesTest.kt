package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * The spacing rules through JNI, as the keyboard calls them: the flags as `Int`s, text looked up
 * only when a rule needs it, and bad input thrown. The rules themselves are tested in Rust and,
 * against the Kotlin they replace, in the app.
 */
class SpacingRulesTest {
    private class Editor(
        private val before: String,
        override val currentWordText: String = "",
        override val language: String = "en",
    ) : SpacingRules.Editor {
        val asked = mutableListOf<String>()

        override val rule: SpacingRules.Rule
            get() = SpacingRules.Rule(".,?!", "¿", ".,?!", "(").also { asked += "rule" }

        override fun textBeforeCursor(n: Int): String = before.takeLast(n).also { asked += "before $n" }
    }

    private fun typing(autoSpacePunctuation: Boolean = true, rawEditor: Boolean = false, keyVariation: Int = 2) =
        SpacingRules.Typing(autoSpacePunctuation, rawEditor, keyVariation, selectionValid = true, selectionStart = 3)

    @Test
    fun theFlagsAreKotlins() {
        assertEquals(0, SpacingRules.INACTIVE)
        assertEquals(5, SpacingRules.autoSpaceActive(true))
        assertEquals(1, SpacingRules.autoSpaceActive(false))
        assertEquals(7, SpacingRules.phantomSpaceActive(showComposingRegion = true, stayActiveNextUpdate = true))
        assertEquals(3, SpacingRules.phantomSpaceActive(showComposingRegion = true, stayActiveNextUpdate = false))
        assertTrue(SpacingRules.isActive(1))
        assertFalse(SpacingRules.isActive(6))
        assertTrue(SpacingRules.showsComposingRegion(2))
        assertEquals(3, SpacingRules.afterUpdate(7))
        assertEquals(0, SpacingRules.afterUpdate(3))
        assertTrue(SpacingRules.keepsCandidate(4))
        assertFalse(SpacingRules.keepsCandidate(3))
    }

    @Test
    fun aFullStopIsAutoSpacedAndACommaTakesTheSpaceBack() {
        val editor = Editor("cat", currentWordText = "cat")
        val stop = SpacingRules.commitChar(".", typing(), SpacingRules.INACTIVE, SpacingRules.INACTIVE, editor)
        assertTrue(stop.spaceAfter)
        assertFalse(stop.deletePreviousSpace)
        assertEquals(SpacingRules.autoSpaceActive(true), stop.autoSpace)
        assertEquals(SpacingRules.INACTIVE, stop.phantomSpace)
        val comma = SpacingRules.commitChar(",", typing(), stop.autoSpace, SpacingRules.INACTIVE, Editor("t. "))
        assertTrue(comma.deletePreviousSpace)
        val opening = SpacingRules.commitChar("¿", typing(), SpacingRules.INACTIVE, SpacingRules.INACTIVE, editor)
        assertTrue(opening.spaceBefore)
        assertFalse(opening.spaceAfter)
    }

    @Test
    fun textIsLookedUpOnlyWhenNeeded() {
        val off = Editor("cat")
        val letter = SpacingRules.commitChar("a", typing(autoSpacePunctuation = false), 0, 0, off)
        assertFalse(letter.spaceBefore)
        assertEquals(emptyList(), off.asked)
        val owed = Editor("cat")
        val phantom = SpacingRules.commitChar("a", typing(autoSpacePunctuation = false), 0, 5, owed)
        assertTrue(phantom.spaceBefore)
        assertEquals(listOf("rule", "before 1"), owed.asked)
        val both = Editor("cat")
        SpacingRules.commitChar("a", typing(), 0, 0, both)
        assertEquals(listOf("rule", "before 1", "before 3"), both.asked)
    }

    @Test
    fun aPhantomSpaceGoesBetweenWordsButNotInChinese() {
        assertTrue(SpacingRules.phantomSpaceBefore("d", typing(), 1, forceActive = false, Editor("cat")))
        assertFalse(SpacingRules.phantomSpaceBefore("d", typing(), 0, forceActive = false, Editor("cat")))
        assertTrue(SpacingRules.phantomSpaceBefore("d", typing(), 0, forceActive = true, Editor("cat")))
        assertFalse(SpacingRules.phantomSpaceBefore("d", typing(), 1, forceActive = false, Editor("cat", language = "zh")))
        assertTrue(SpacingRules.phantomSpaceBefore("(", typing(), 1, forceActive = false, Editor("cat.")))
        assertFalse(SpacingRules.phantomSpaceBefore("", typing(), 1, forceActive = false, Editor("cat")))
    }

    @Test
    fun theLanguagesAndTheFallbackRuleAreUpstreams() {
        assertFalse(SpacingRules.supportsAutoSpace("zh"))
        assertFalse(SpacingRules.supportsAutoSpace("jp"))
        assertTrue(SpacingRules.supportsAutoSpace("ja"))
        assertTrue(SpacingRules.supportsAutoSpace("en"))
        assertEquals(listOf("fallback", "Fallback", ".,?!", "", ".,?!", "", ".?!"), SpacingRules.fallbackRule)
    }

    @Test
    fun badInputThrows() {
        assertFailsWith<IllegalArgumentException> { SpacingNative.commitChar("a", IntArray(4), null, null) }
        assertFailsWith<IllegalArgumentException> { SpacingNative.commitChar("a", IntArray(5), "", IntArray(7)) }
        assertFailsWith<IllegalArgumentException> { SpacingNative.commitChar("a", IntArray(5), "ab", IntArray(8)) }
        assertFailsWith<IllegalArgumentException> { SpacingNative.commitChar("a", IntArray(5), "", IntArray(8) { -2 }) }
        assertFailsWith<IllegalArgumentException> { SpacingNative.phantomSpaceBefore("a", IntArray(5), "", IntArray(8) { -1 }) }
        assertFailsWith<NullPointerException> { SpacingNative.commitChar("a", IntArray(5), "", null) }
    }
}
