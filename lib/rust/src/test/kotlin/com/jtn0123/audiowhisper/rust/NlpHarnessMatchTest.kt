package com.jtn0123.audiowhisper.rust

import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals

/** Real dictionary answers over public fixture prefix/editor snapshots; never a user's text. */
class NlpHarnessMatchTest {
    @Test fun everyPublicHarnessFixtureMatchesWithBundledDictionaryResults() {
        for (dictionary in TestDictionaries.both) everyFixtureMatches(dictionary)
    }

    private fun everyFixtureMatches(dictionary: File) {
        val personal = PersonalWordStore(emptyList(), IntArray(0), emptyList())
        val rows = requireNotNull(javaClass.getResourceAsStream("/nlp/harness.tsv")).bufferedReader().readLines()
        val names = mutableSetOf<String>()
        var checkpoints = 0
        requireNotNull(LatinSession.open(dictionary.readBytes())).use { decoder ->
            NlpMatchTest.Pairing().use { p ->
                for (row in rows) {
                    val f = row.split('\t')
                    names += f[0]
                    val units = if (f[4] == "-") emptyList() else f[4].chunked(4).map { h -> (h.substring(0, 2).toInt(16) or (h.substring(2).toInt(16) shl 8)).toChar() }
                    val content = units.joinToString("")
                    // Prefixes exercise every request while typing; final cursor checkpoints cover moves.
                    for (end in 0..content.length) {
                        val text = content.take(end)
                        val context = ReferenceNlp.context(text, f[1].toInt()).toMutableList().also {
                            it[4] = f[3]; it[5] = f[3]; it[7] = "1"
                            if (end == content.length) { it[10] = f[2]; it[11] = f.getOrNull(5) ?: f[2] }
                        }
                        p.call(NlpOp.CONTEXT, *context.toTypedArray())
                        val generation = p.call(NlpOp.BEGIN, checkpoints.toString())[0]
                        p.call(NlpOp.READY, generation, (checkpoints + 8).toString())
                        val typed = text.takeLastWhile { it.isLetterOrDigit() || it in "'-’" }
                        val strip = decoder.suggest(personal, null, typed, emptyList(), true, false, true, FloatArray(0), 8, SuggestRules.defaultStrength)
                        val flags = p.call(NlpOp.CANDIDATE_PLAN, *context.toTypedArray(), "0",
                            *strip.words.indices.map { if (it == strip.autoCorrection) "1" else "0" }.toTypedArray())
                        val values = strip.words.flatMapIndexed { i, word -> listOf("$checkpoints-$i", word, "0", flags[i]) }.toTypedArray()
                        p.call(NlpOp.COMPLETE, generation, *values)
                        p.call(NlpOp.AUTO, "-1", "0")
                        if (checkpoints % 7 == 0) {
                            val inline = p.call(NlpOp.INLINE_BEGIN, "2")[0]
                            p.call(NlpOp.INLINE_COMPLETE, inline, "regular", "0", "pinned", "1")
                            p.call(NlpOp.INLINE_BEGIN, "0")
                        }
                        checkpoints++
                    }
                    p.call(NlpOp.FIELD)
                }
            }
        }
        assertEquals(54, names.size)
        assertEquals(2056, checkpoints)
        println("NLP public fixture replay on ${dictionary.name}: ${names.size} traces, $checkpoints prefix/cursor checkpoints, zero differences")
    }
}
