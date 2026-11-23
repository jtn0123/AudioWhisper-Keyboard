package dev.patrickgold.florisboard.ime.media.emoji

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Emoji data and suggestions in Rust (lib/rust/crates/emoji, through [EmojiData], [Emoji] and
 * [EmojiSuggestionProvider.suggestEmojis]) do exactly what the Kotlin they replaced did
 * ([ReferenceEmoji]): every bundled emoji file parsed and searched, damaged files, random queries
 * and composing texts, emoji values with tones, and locales choosing among files.
 */
class EmojiMatchTest {
    private val files = File("src/main/assets/ime/media/emoji").listFiles()!!.filter { it.extension == "txt" }.sortedBy { it.name }

    private fun assertSeen(comparison: EmojiComparison, outcomes: List<String>) {
        assertEquals(emptyList(), comparison.differences)
        for (outcome in outcomes) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times, seen ${comparison.seen}")
        }
    }

    @Test
    fun bundledFilesMatch() {
        assertEquals(listOf("de", "en", "es", "fr", "it", "pt", "root"), files.map { it.nameWithoutExtension })
        val tape = GoldenTape.ofReference("emoji-bundled")
        val comparison = EmojiComparison(tape)
        val random = Random(SEED)
        for (file in files) comparison.file(file.name, file.readText(), random, queries = 600)
        tape.finish()
        assertEquals(files.size, comparison.seen["bundled file"])
        assertSeen(comparison, SUGGEST_OUTCOMES)
    }

    @Test
    fun damagedFilesMatch() {
        val tape = GoldenTape.ofReference("emoji-damaged")
        val comparison = EmojiComparison(tape)
        comparison.damaged(Random(SEED), files.map { it.readText() }, runs = 2_000, queries = 8)
        tape.finish()
        assertSeen(comparison, SUGGEST_OUTCOMES + PARSE_OUTCOMES)
    }

    @Test
    fun queriesTonesAndAssetsMatch() {
        val tape = GoldenTape.ofReference("emoji-queries")
        val comparison = EmojiComparison(tape)
        val random = Random(SEED)
        comparison.queries(random, runs = 20_000)
        comparison.tones(random, runs = 20_000)
        comparison.assets(random, runs = 20_000)
        tape.finish()
        val tones = EmojiSkinTone.entries.map { "skin $it" } + EmojiHairStyle.entries.map { "hair $it" }
        val assets = listOf("none", "language", "country", "variant").map { "asset: $it" }
        val noData = listOf("no data: none", "no data: threw")
        assertSeen(comparison, listOf("query: none", "query: valid") + noData + tones + assets)
    }

    private companion object {
        const val SEED = 20_261_108
        const val MIN_SEEN = 100
        val SUGGEST_OUTCOMES = listOf(
            "suggest: no query", "suggest: none", "suggest: some", "suggest: max", "suggest: threw",
            "rank: none", "rank: some", "rank: max", "rank: threw",
        )
        val PARSE_OUTCOMES = listOf("parse: single", "parse: variants", "parse: empty name", "parse: nul", "rank: a NaN took a place")
    }
}
