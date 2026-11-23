package com.jtn0123.audiowhisper.gif

import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.GifRules
import dev.patrickgold.florisboard.ime.text.gestures.SwipeAction
import dev.patrickgold.florisboard.ime.text.key.KeyType
import kotlin.random.Random

/**
 * Records what the GIF panel's rules in Rust (lib/rust/crates/gif, behind [SearchText], [GifRules]
 * and [GifSearchKeys]) answer, on a [GoldenTape] of the answers the Kotlin they replaced gave
 * (docs/phase6-kotlin-references.md): random queries and texts (emoji and half emoji, odd spaces,
 * apostrophes, İ and ß), typed words for the chips, every key code near FlorisBoard's with every
 * key type, delete-key swipes and failed calls. These are the functions the panel's session runs
 * in Rust; GifSearchTraceTest records the session itself, with its result cache and recents.
 */
class GifComparison(private val tape: GoldenTape) {
    /** How often each outcome worth reaching was reached, so a run that tests nothing shows. */
    val seen = sortedMapOf<String, Int>()

    private fun check(what: String, answer: Any?) = tape.record("$what: ${show(answer)}")

    private fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    /** The constants the panel shares with the Rust. */
    fun constants() {
        check("MAX_LENGTH", SearchText.MAX_LENGTH)
        check("categories", GifRules.categories)
        check("TRENDING", GifRules.trending)
        check("RECENTS", GifRules.recents)
    }

    /** Random queries edited, and texts whose last word is offered. */
    fun text(random: Random, count: Int) = repeat(count) {
        val query = pieces(random, if (random.nextInt(6) == 0) 60 else 8)
        val typed = pieces(random, 3)
        val sum = query + typed
        check("type", SearchText.type(query, typed))
        see("cut at the limit", sum.length > SearchText.MAX_LENGTH)
        check("typeWord", SearchText.typeWord(query, typed))
        val one = SearchText.deleteLast(query)
        check("deleteLast", one)
        see("an emoji deleted whole", query.length - one.length == 2)
        see("half an emoji deleted", query.lastOrNull()?.isSurrogate() == true && query.length - one.length == 1)
        val times = COUNTS.random(random)
        check("deleteLast", SearchText.deleteLast(query, times))
        val words = SearchText.deleteWords(query, times)
        check("deleteWords", words)
        see("words deleted", words.isNotEmpty() && words.length < query.length)
        val before = if (random.nextInt(20) == 0) null else query
        val word = SearchText.lastWord(before)
        check("lastWord", word)
        see(if (word == null) "no word offered" else "a word offered")
        see("a word with an apostrophe", word?.contains('\'') == true)
    }

    /** Random words just typed, and the chips offered after them. */
    fun chips(random: Random, count: Int) = repeat(count) {
        val typed = when (random.nextInt(4)) {
            0 -> null
            1 -> recase(random, (GifRules.categories + GifRules.recents).random(random))
            else -> pieces(random, 3)
        }
        val hasRecents = random.nextBoolean()
        val chips = GifRules.chips(typed, hasRecents)
        check("chips", chips)
        val moved = typed != null && chips.size == GifRules.categories.size + (if (hasRecents) 1 else 0)
        see(if (typed == null) "no word" else if (moved) "a chip moved first" else "a word put first")
    }

    /** Every key code near FlorisBoard's, with every key type, and delete-key swipes. */
    fun keys(random: Random, swipes: Int) {
        val codes = (-10_000..13_000) + listOf(Int.MIN_VALUE, Int.MAX_VALUE, -1, 0, 65_535, 65_536)
        for (code in codes) {
            for (type in KeyType.entries) {
                val route = GifSearchKeys.route(code, type)
                check("route", route.name)
                see("route ${route.name}")
            }
        }
        repeat(swipes) {
            val action = SwipeAction.entries.random(random)
            val units = if (random.nextInt(10) == 0) EDGES.random(random) else random.nextInt(-40, 40)
            val swipe = GifSearchKeys.swipeDelete(action, units)?.let { it.count to it.words }
            check("swipeDelete", swipe)
            see(if (swipe == null) "no swipe delete" else if (swipe.first > 0) "a swipe deletes" else "a swipe deletes nothing")
        }
    }

    /**
     * Failed GIPHY calls of every kind: every status near HTTP's, then an unreadable answer, a
     * connection that failed and one that timed out (both unreachable), in the order main's
     * GifSearch was asked about them.
     */
    fun problems() {
        val statuses = (-1..700) + listOf(Int.MIN_VALUE, Int.MAX_VALUE)
        val problems = statuses.map { GifRules.problem(GifRules.Failure.STATUS, it) } +
            listOf(GifRules.Failure.UNREADABLE, GifRules.Failure.UNREACHABLE, GifRules.Failure.UNREACHABLE).map { GifRules.problem(it) }
        for (problem in problems) {
            check("problem", problem)
            see("problem $problem")
        }
    }

    /** Up to [parts] random pieces of text. */
    private fun pieces(random: Random, parts: Int): String = buildString {
        repeat(random.nextInt(parts + 1)) { append(PARTS.random(random)) }
    }

    private fun recase(random: Random, text: String): String = when (random.nextInt(4)) {
        0 -> text.uppercase()
        1 -> text.lowercase()
        else -> text
    }

    private fun show(value: Any?): String = when (value) {
        is String -> "\"" + value.map { if (it.code in 0x20..0x7E) "$it" else "\\u%04X".format(it.code) }.joinToString("") + "\""
        is List<*> -> value.joinToString(", ", "[", "]") { show(it) }
        else -> value.toString()
    }

    private companion object {
        val PARTS = listOf(
            "cat", "Congrats", "happy", "dance", "a", "I", "OK", "x", "7", "42", "don't", "'", "''", "’", "İ",
            "ß", "ǅ", "Σ", "café", "é", "😀", "\uD800", "\uDC00", "-", "!", ".", ",", "?", ";", ":", ")", "…",
            " ", " ", " ", "  ", "\t", "\n", " ", "\u0085", " ", "　", "\u001C", "​", "ـ", "ـا", "ा", "中文",
            "\u00A0", "\u2007", "\u202F", "\u2028",
            "x".repeat(29), "y".repeat(30), "z".repeat(31),
        )
        val COUNTS = listOf(-2, -1, 0, 1, 1, 2, 3, 5, 9, 60, Int.MAX_VALUE / 1_000_000, Int.MIN_VALUE)
        val EDGES = listOf(Int.MIN_VALUE, Int.MAX_VALUE, Int.MIN_VALUE + 1, -1_000_000, 1_000_000)
    }
}
