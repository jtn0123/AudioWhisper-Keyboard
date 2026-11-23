package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.golden.GoldenTape
import kotlin.random.Random

/**
 * Holds reading aloud's rules in Rust (lib/rust/crates/reader, behind [PageText], [FollowAlong]
 * and [SpeakerKey]) up against the Kotlin they replaced, [ReferenceReader], wherever this runs:
 * random pages of blocks and pieces of text (bullets, links, odd spaces, emoji and half emoji, İ
 * and ß), selections taken from them and roughened, runs of blocks read before a scroll, text to
 * cut for the speech engine, pieces to keep in view, and presses of the speaker key.
 * Self-contained, so a device check can copy it into the app and run it on a phone, whose Java
 * judges spaces, case and sentences differently from a computer's.
 *
 * [sigmaExact] says whether text may hold Σ, which lower-cases to ς at the end of a word and to σ
 * elsewhere. Rust decides that by ICU's rule, as Android does; the JVM uses its own, so off a
 * phone Σ is left out.
 */
class ReaderComparison(private val sigmaExact: Boolean, private val tape: GoldenTape) {
    /** How many comparisons failed, and the first few, described. */
    var failures = 0
        private set
    val examples = mutableListOf<String>()

    /** How many comparisons ran. */
    var checks = 0L
        private set

    /** How often each outcome worth reaching was reached, so a run that tests nothing shows. */
    val seen = sortedMapOf<String, Int>()

    private val parts = if (sigmaExact) PARTS else PARTS.filter { CAPITAL_SIGMA !in it }

    private fun check(what: String, input: Any?, kotlin: Any?, rust: Any?) {
        tape.record("$what: ${show(kotlin)}")
        checks++
        if (kotlin == rust) return
        failures++
        if (examples.size < MAX_EXAMPLES) examples += "$what(${show(input)}): Kotlin ${show(kotlin)}, Rust ${show(rust)}"
    }

    private fun see(what: String, happened: Boolean = true) {
        if (happened) seen[what] = (seen[what] ?: 0) + 1
    }

    /** What [body] gives, or the exception it throws, so both sides must throw alike. */
    private inline fun outcome(body: () -> Any?): Any? = try {
        body()
    } catch (e: IllegalArgumentException) {
        "threw ${e.javaClass.name}"
    }

    /** Random pages: where reading starts, whether all of it shows, and where a selection is. */
    fun pages(random: Random, count: Int) = repeat(count) {
        val blocks = List(random.nextInt(MAX_BLOCKS + 1)) { TextBlock(text(random), 0, 0) }.map { place(random, it) }
        val (top, bottom) = visible(random)
        val first = PageText.firstOnScreen(blocks, top, bottom)
        check("firstOnScreen", blocks, ReferenceReader.PageText.firstOnScreen(blocks, top, bottom), first)
        see(if (first == null) "none on screen" else "one on screen")
        val all = Screen(null, blocks, null, top, bottom, nodes = 0, web = false).allShown
        check("allShown", blocks, ReferenceReader.allShown(blocks, top, bottom), all)
        see(if (all) "all shown" else "not all shown")
        val selected = selection(random, blocks)
        val found = PageText.find(blocks, selected, top, bottom)
        check("find", listOf(selected, blocks), ReferenceReader.PageText.find(blocks, selected, top, bottom), found)
        see("selection found on screen", found != null && blocks[found.block].isShownIn(top, bottom))
        see("selection found off screen", found != null && !blocks[found.block].isShownIn(top, bottom))
        see("selection found mid-block", found != null && found.offset > 0)
        see("selection not found", found == null)
    }

    /** Random pieces a walk of the screen found, made into blocks. */
    fun joins(random: Random, count: Int) = repeat(count) {
        val pieces = List(random.nextInt(MAX_PIECES + 1)) { index ->
            val scrolls = random.nextInt(4) == 0
            place(random, TextBlock(piece(random), 0, 0, scrolls, Reveal(index)))
        }
        val joined = PageText.blocks(pieces)
        check("blocks", pieces, ReferenceReader.PageText.blocks(pieces).map(::describe), joined.map(::describe))
        for (block in joined) {
            val first = pieces.getOrNull((block.reveal as? Reveal)?.piece ?: -1) ?: continue
            see("pieces joined", block.text.length > first.text.trim().length)
            see("a bullet dropped", first.text.firstOrNull()?.let { it in "•◦▪▫‣⁃·" } == true)
        }
        see("a piece dropped", joined.size < pieces.size)
    }

    /** Random blocks read before a scroll, and the screen after it. */
    fun scrolls(random: Random, count: Int) = repeat(count) {
        val vocabulary = VOCABULARY.take(2 + random.nextInt(VOCABULARY.size - 1))
        val blocks = List(random.nextInt(MAX_BLOCKS + 1)) { TextBlock(vocabulary.random(random), 0, 100) }
        val read = List(if (random.nextInt(8) == 0) random.nextInt(MAX_READ) else random.nextInt(8)) { vocabulary.random(random) }
        val at = PageText.resumeAfter(blocks, read)
        check("resumeAfter", "${blocks.map { it.text }} after $read", ReferenceReader.PageText.resumeAfter(blocks, read), at)
        see(
            when (at) {
                0 -> "resumed at the start"
                blocks.size -> "nothing new"
                else -> "resumed part way"
            },
        )
        see("read more than the recent", read.size > PageText.RECENT)
    }

    /** Random text cut into what the speech engine takes. */
    fun speech(random: Random, count: Int) = repeat(count) {
        if (random.nextInt(LONG_ONE_IN) == 0) return@repeat longSpeech(random)
        val text = buildString { repeat(random.nextInt(MAX_SENTENCES + 1)) { append(sentence(random)) } }
        val max = MAXES.random(random).takeIf { it > 0 } ?: (1 + random.nextInt(MAX_RANDOM_MAX))
        val said = outcome { PageText.utterances(text, max) }
        check("utterances", "$text, $max", outcome { ReferenceReader.PageText.utterances(text, max) }, said)
        if (said !is List<*>) return@repeat see("utterances threw")
        see("sentences joined", said.size in 1..<text.count { it in ".!?" })
        see("a sentence cut", said.any { it.toString().length == max })
        see("nothing to say", said.isEmpty())
    }

    /** Text about as long as the speech engine takes, cut at the app's own limit. */
    private fun longSpeech(random: Random) {
        val text = buildString {
            while (length < ReferenceReader.PageText.MAX_UTTERANCE - LONG_SLACK) append(sentence(random))
            repeat(random.nextInt(2 * LONG_SLACK)) { append(if (random.nextInt(5) == 0) ' ' else 'x') }
        }
        val said = PageText.utterances(text)
        check("utterances", "${text.length} long", ReferenceReader.PageText.utterances(text), said)
        see("cut at the app's limit", said.size > 1)
        see("whole at the app's limit", said.size == 1)
    }

    /** The numbers the rules are built on. */
    fun constants() {
        check("MAX_UTTERANCE", null, ReferenceReader.PageText.MAX_UTTERANCE, PageText.MAX_UTTERANCE)
        check("RECENT", null, ReferenceReader.PageText.RECENT, PageText.RECENT)
        check("HOLD_MS", null, ReferenceReader.SpeakerKey.HOLD_MS, SpeakerKey.HOLD_MS)
    }

    /** Random pieces kept in view. */
    fun views(random: Random, count: Int) = repeat(count) {
        val (first, last, top, bottom) = List(4) { number(random) }
        val fits = FollowAlong.fits(first..last, top, bottom)
        check("fits", listOf(first, last, top, bottom), ReferenceReader.fits(first..last, top, bottom), fits)
        see(if (fits) "fits" else "doesn't fit")
        see("fits by its top half", fits && last > bottom)
    }

    /** Random presses, lifts and slides off a pair of speaker keys sharing a clock. */
    fun keys(random: Random, runs: Int, steps: Int) = repeat(runs) {
        var now = TIMES.random(random)
        val kotlin = ReferenceReader.SpeakerKey { now }
        val rust = SpeakerKey { now }
        repeat(steps) { step ->
            now += GAPS.random(random).takeIf { it != 0L } ?: random.nextLong()
            val (what, before, after) = when (random.nextInt(5)) {
                0, 1 -> random.nextBoolean().let { reading -> Triple("down($reading)", kotlin.down(reading), rust.down(reading)) }
                2, 3 -> Triple("up", kotlin.up(), rust.up())
                else -> Triple("cancel", kotlin.cancel(), rust.cancel())
            }
            check(what, "step $step at $now", before?.toString(), after?.toString())
            if (what != "cancel") see(after?.toString() ?: "no action")
        }
    }

    private fun TextBlock.isShownIn(visibleTop: Int, visibleBottom: Int) =
        PageText.allShown(listOf(this), visibleTop, visibleBottom)

    /** A block's text, its place, whether it scrolls and which piece it scrolls into view as. */
    private fun describe(block: TextBlock) =
        "${block.text}|${block.top}|${block.bottom}|${block.scrolls}|${(block.reveal as? Reveal)?.piece}"

    /** A piece's way into view, which says which piece it is. */
    private class Reveal(val piece: Int) : suspend () -> Unit {
        override suspend fun invoke() = Unit
    }

    private fun place(random: Random, block: TextBlock): TextBlock {
        val top = number(random)
        val bottom = when (random.nextInt(5)) {
            0 -> top
            1 -> number(random)
            else -> top + random.nextInt(MAX_HEIGHT)
        }
        return TextBlock(block.text, top, bottom, block.scrolls, block.reveal)
    }

    private fun visible(random: Random): Pair<Int, Int> =
        if (random.nextInt(8) == 0) number(random) to number(random) else random.nextInt(-200, 600).let { it to it + random.nextInt(2_000) }

    private fun number(random: Random): Int =
        if (random.nextInt(4) == 0) NUMBERS.random(random) else random.nextInt(-500, 3_000)

    private fun text(random: Random) = buildString { repeat(random.nextInt(MAX_PARTS + 1)) { append(parts.random(random)) } }

    private fun piece(random: Random): String = when (random.nextInt(6)) {
        0 -> BULLETS.random(random) + text(random)
        1 -> CLOSINGS.random(random) + text(random)
        2 -> text(random) + " "
        else -> text(random)
    }

    private fun sentence(random: Random): String =
        text(random) + if (random.nextInt(5) == 0) "" else ENDINGS.random(random)

    /** Part of a block's text, roughened as apps report it, after a line or two; or anything. */
    private fun selection(random: Random, blocks: List<TextBlock>): String {
        if (blocks.isEmpty() || random.nextInt(5) == 0) return text(random)
        val text = blocks.random(random).text
        val start = random.nextInt(text.length + 1)
        val words = text.substring(start, random.nextInt(start, text.length + 1))
        val rough = words.map { char ->
            when (random.nextInt(8)) {
                0 -> char.uppercaseChar()
                1 -> char.lowercaseChar()
                2 -> if (char.isWhitespace()) SPACES.random(random) else "$char"
                else -> "$char"
            }.toString()
        }.joinToString("")
        return LINES.random(random) + rough + LINES.random(random)
    }

    private fun show(value: Any?): String = when (value) {
        is String -> value.map { if (it.code in 0x21..0x7E) "$it" else "\\u%04X".format(it.code) }.joinToString("", "\"", "\"")
        is TextBlock -> "${show(value.text)}@${value.top}..${value.bottom}${if (value.scrolls) " scrolls" else ""}"
        is List<*> -> value.joinToString(", ", "[", "]", transform = ::show)
        else -> value.toString()
    }

    private companion object {
        const val MAX_EXAMPLES = 10
        const val MAX_BLOCKS = 10
        const val MAX_PIECES = 14
        const val MAX_PARTS = 7
        const val MAX_SENTENCES = 8
        const val MAX_READ = 70

        /** One text to speak in this many is about as long as the speech engine takes. */
        const val LONG_ONE_IN = 10

        /** How far either side of the speech engine's limit a long text's length falls. */
        const val LONG_SLACK = 30
        const val MAX_HEIGHT = 400
        const val MAX_RANDOM_MAX = 60
        const val CAPITAL_SIGMA = 'Σ'

        val PARTS = listOf(
            "the", "quick", "Brown", "FOX", "ok", "Copy", "a", "I", "x", "7", "42", "İstanbul", "STRASSE", "ß",
            "ǅ", "ΣΟΦΊΑ", "Σ", "café", "é", "😀", "\uD800", "\uDC00", "-", "—", "|", "'", "\"", "Mr.",
            "e.g.", "U.S.", "3.5", ".", ",", "!", "?", ")", "”", "’", "•", "·",
            " ", " ", " ", "  ", "\t", "\n", "\r", "\r\n", " ", "\u0085", " ", " ", " ", " ",
            "　", "\u000B", "\u000C", "\u001C", "\u001F", "​", "﻿",
        )

        val BULLETS = listOf("•", "◦", "▪", "▫", "‣", "⁃", "·", "• ", " ", "  ", "• ◦ ")
        val CLOSINGS = listOf(",", ".", ";", ":", "!", "?", ")", "]", "}", "”", "’", " ", " ", "\n")
        val ENDINGS = listOf(". ", "! ", "? ", ".", "?! ", "...", ".\n", ".\" ", ".  ", "\n\n", "; ")
        val SPACES = listOf(" ", "  ", "\t", " ", " ", "\u0085", "\u000B", "\n", "\r\n", " \t ")
        val LINES = listOf("", "", "", "\n", " \n", "\r\n", "\n\n", "\r", "   \n", "\nmore", "x\n")
        val VOCABULARY = listOf("a", "b", "Copy", "c", "one", "two", "", "x", "y", "Reply", "z", "three")
        val NUMBERS = listOf(0, 1, -1, 2, -2, 3, -3, Int.MAX_VALUE, Int.MIN_VALUE, Int.MAX_VALUE - 1, Int.MIN_VALUE + 1, 1_000, 1_001)
        val MAXES = listOf(0, 0, 0, 1, 2, 3, 5, 12, 20, 3_900, Int.MAX_VALUE)
        val TIMES = listOf(0L, 1_000L, Long.MAX_VALUE - 100, Long.MIN_VALUE + 100, -5L)
        val GAPS = listOf(0L, 0L, 1L, 120L, 399L, 400L, 401L, 1_000L, -1L, -400L, Long.MAX_VALUE, Long.MIN_VALUE)
    }
}
