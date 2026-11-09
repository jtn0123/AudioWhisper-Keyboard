package com.jtn0123.audiowhisper.rust

import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

/**
 * Emoji data and suggestions through JNI, as the keyboard calls them: a file parsed into entries,
 * sets and skin tones whose strings Kotlin cuts from its own text, queries answered as positions,
 * and bad input thrown. The rules themselves are tested in Rust and, against the Kotlin they
 * replace, in the app.
 */
class EmojiRulesTest {
    private val ids = listOf("recently_used", "smileys", "people")

    private val file = "# comment\n[smileys]\n😀;grinning face;face|grin\n👋;waving hand;hand|wave\n" +
        "\t👋🏻; light ;x\n\t👋🏿;dark;y\n[people]\n🧑‍🦰;person; a | b \n"

    @Test
    fun aFileParsesIntoEntriesSetsAndSkinTones() {
        val table = EmojiTable.parse(file, ids)
        assertEquals(listOf("😀", "👋", "👋🏻", "👋🏿", "🧑‍🦰"), table.entries.map { it.value })
        assertEquals(listOf("grinning face", "waving hand", "waving hand", "waving hand", "person"), table.entries.map { it.name })
        assertEquals(listOf(listOf("face", "grin"), listOf("hand", "wave"), listOf("x"), listOf("y"), listOf("a", "b")), table.entries.map { it.keywords })
        assertEquals(listOf(0, 0, 1, 5, 0), table.entries.map { it.skinTone })
        assertEquals(listOf(0, 0, 0, 0, 1), table.entries.map { it.hairStyle })
        assertEquals(listOf(Triple(1, 0, 1), Triple(1, 1, 3), Triple(2, 4, 1)), table.sets.map { Triple(it.category, it.start, it.size) })
        assertEquals(6, table.bySkinTone.size)
        assertContentEquals(intArrayOf(0, 1, 4), table.bySkinTone[0])
        assertContentEquals(intArrayOf(0, 2, 4), table.bySkinTone[1])
        assertContentEquals(intArrayOf(0, 4), table.bySkinTone[2])
        assertContentEquals(intArrayOf(0, 3, 4), table.bySkinTone[5])
    }

    @Test
    fun anEmptyFileHasNothing() {
        val table = EmojiTable.parse("", ids)
        assertEquals(0, table.entries.size)
        assertEquals(0, table.sets.size)
        assertEquals(List(6) { 0 }, table.bySkinTone.map { it.size })
        assertContentEquals(intArrayOf(), table.suggest("", 0, 5))
    }

    @Test
    fun queriesAnswerWithPositionsBestFirst() {
        val table = EmojiTable.parse(file, ids)
        // grinning face (2/13 of its name, and the keyword grin) above waving hand (2/11).
        assertContentEquals(intArrayOf(0, 1), table.suggest("IN", 0, 8))
        assertContentEquals(intArrayOf(0), table.suggest("IN", 0, 1))
        assertContentEquals(intArrayOf(1), table.suggest("wav", 5, 8))
        assertContentEquals(intArrayOf(), table.suggest("zzz", 0, 8))
        assertFailsWith<IllegalArgumentException> { table.suggest("fa", 0, -1) }
        assertFailsWith<IllegalArgumentException> { table.suggest("fa", 6, 8) }
        assertFailsWith<IllegalArgumentException> { table.suggest("fa", -1, 8) }
    }

    @Test
    fun theQueryFollowsThePrefix() {
        assertEquals("smi", EmojiRules.validateQuery(":smi", ":", 3))
        assertNull(EmojiRules.validateQuery(":sm", ":", 3))
        assertNull(EmojiRules.validateQuery("smi", ":", 3))
        assertNull(EmojiRules.validateQuery(":smí", ":", 3))
        assertEquals("smile", EmojiRules.validateQuery("smile", "", 3))
        assertEquals("", EmojiRules.validateQuery("", "", 0))
    }

    @Test
    fun tonesAreTheFirstListed() {
        assertEquals(EmojiTones(0, 0), EmojiRules.tones("😀"))
        assertEquals(EmojiTones(3, 0), EmojiRules.tones("👋🏽"))
        assertEquals(EmojiTones(5, 3), EmojiRules.tones("🧑🏿‍🦳🧑‍🦲"))
        assertEquals(EmojiTones(0, 0), EmojiRules.tones("\u0000👋🏽"))
    }

    @Test
    fun theMostSpecificAssetWins() {
        val assets = listOf("de.txt", "en.txt", "en_US.txt", "root.txt")
        assertEquals(2, EmojiRules.resolveAsset(assets, "EN", "US", ""))
        assertEquals(1, EmojiRules.resolveAsset(assets, "en", "GB", "x"))
        assertEquals(EmojiRules.NO_ASSET, EmojiRules.resolveAsset(assets, "fr", "", ""))
        assertEquals(EmojiRules.NO_ASSET, EmojiRules.resolveAsset(emptyList(), "en", "", ""))
    }

    @Test
    fun badLengthsThrow() {
        assertFailsWith<IllegalArgumentException> { EmojiNative.parse("", "ab", intArrayOf(3)) }
        assertFailsWith<IllegalArgumentException> { EmojiNative.resolveAsset("a", intArrayOf(), arrayOf("en", "", "")) }
        assertFailsWith<IllegalArgumentException> { EmojiNative.resolveAsset("", intArrayOf(), arrayOf("en", "")) }
    }
}
