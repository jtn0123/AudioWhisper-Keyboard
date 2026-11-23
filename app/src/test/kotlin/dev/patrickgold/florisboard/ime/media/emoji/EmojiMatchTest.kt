package dev.patrickgold.florisboard.ime.media.emoji

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Emoji data and suggestions in Rust (lib/rust/crates/emoji, through [EmojiData], [Emoji] and
 * [EmojiSuggestionProvider.suggestEmojis]) give the answers the Kotlin they replaced gave
 * (`ReferenceEmoji`, retired for golden digests of them): every bundled emoji file parsed and
 * searched, damaged files, random queries and composing texts, emoji values with tones, and
 * locales choosing among files.
 */
class EmojiMatchTest {
    private val files = File("src/main/assets/ime/media/emoji").listFiles()!!.filter { it.extension == "txt" }.sortedBy { it.name }

    private fun assertSeen(comparison: EmojiComparison, outcomes: List<String>) {
        for (outcome in outcomes) {
            assertTrue((comparison.seen[outcome] ?: 0) >= MIN_SEEN, "$outcome only ${comparison.seen[outcome]} times, seen ${comparison.seen}")
        }
    }

    @Test
    fun bundledFilesMatch() {
        assertEquals(listOf("de", "en", "es", "fr", "it", "pt", "root"), files.map { it.nameWithoutExtension })
        val comparison = GoldenTape.check("emoji-bundled") { tape ->
            EmojiComparison(tape).apply {
                val random = Random(SEED)
                for (bundled in files) file(bundled.readText(), random, queries = 600)
            }
        }
        assertEquals(files.size, comparison.seen["bundled file"])
        assertSeen(comparison, SUGGEST_OUTCOMES)
    }

    @Test
    fun damagedFilesMatch() {
        val comparison = GoldenTape.check("emoji-damaged") { tape ->
            EmojiComparison(tape).apply { damaged(Random(SEED), files.map { it.readText() }, runs = 2_000, queries = 8) }
        }
        assertSeen(comparison, SUGGEST_OUTCOMES + PARSE_OUTCOMES)
    }

    @Test
    fun queriesTonesAndAssetsMatch() {
        val comparison = GoldenTape.check("emoji-queries") { tape ->
            EmojiComparison(tape).apply {
                val random = Random(SEED)
                queries(random, runs = 20_000)
                tones(random, runs = 20_000)
                assets(random, runs = 20_000)
            }
        }
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
