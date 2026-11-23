package com.jtn0123.audiowhisper.golden

import java.io.File
import kotlin.test.fail

/**
 * The answers a test records as it asks its questions, checked against a golden file of the
 * answers a frozen Kotlin reference gave to the same questions (docs/phase6-kotlin-references.md).
 *
 * A golden file holds no answer in full, only:
 *
 * * `count`: how many answers the test records;
 * * `digest`: every answer's hash, chained in order (FNV-1a over 64-bit words: an answer's
 *   number of parts, then each part's number of UTF-16 units and the units), so one different
 *   unit anywhere gives a different digest;
 * * `at N`: the chain after N answers, at each power of two, so a failure says which range of
 *   answers first differs.
 *
 * An answer is the text the test compared before (the reference's and Rust's, side by side), so
 * texts the questions were made of stay in the test, and no answer reaches a log.
 *
 * Golden files are read from the test classpath at `golden/<name>.txt` (each module's
 * `src/test/resources`). They are written only by a tape made with [ofReference], at the commit
 * that still had the reference, with `REFERENCE_GOLDEN=write` and the test run again; a tape of
 * the code under test refuses to write them, so a regression can never be recorded as the
 * reference.
 */
class GoldenTape private constructor(
    private val name: String,
    private val golden: Golden?,
    private val writing: Boolean,
) {
    private var count = 0L
    private var digest = OFFSET
    private val ats = mutableListOf<Pair<Long, Long>>()
    private var finished = false

    /** Records the next answer, and fails at once if the answers so far differ from the reference's. */
    fun record(answer: String) = record(listOf(answer))

    /**
     * Records the next answer, made of [parts]: each part's units are prefixed with their number,
     * so no two different lists of parts hash alike by running together.
     */
    fun record(parts: List<String>) {
        check(!finished) { "$name: recorded after the tape was finished" }
        var hash = mix(OFFSET, parts.size.toLong())
        for (part in parts) {
            hash = mix(hash, part.length.toLong())
            for (unit in part) hash = mix(hash, unit.code.toLong())
        }
        digest = mix(digest, hash)
        count++
        if (count and (count - 1) == 0L) {
            ats += count to digest
            val expected = golden?.ats?.get(count) ?: return
            if (expected != digest) {
                fail(
                    "$name: answers ${count / 2 + 1} to $count differ from the reference's " +
                        "(golden ${expected.hex()}, now ${digest.hex()})",
                )
            }
        }
    }

    /** Records each of [answers] in order, one answer each. */
    fun recordAll(answers: Iterable<String>) = answers.forEach { record(it) }

    /** Checks the count and the digest, or writes the golden file when the reference is writing it. */
    fun finish() {
        check(!finished) { "$name: finished twice" }
        finished = true
        if (writing) {
            val file = File(RESOURCES, "$name.txt")
            file.parentFile.mkdirs()
            file.writeText(text())
            return
        }
        val golden = golden ?: fail("$name: no golden file golden/$name.txt on the test classpath")
        if (golden.count != count) {
            fail("$name: ${golden.count} answers in the golden file, $count recorded")
        }
        if (golden.digest != digest) {
            fail("$name: the answers differ from the reference's after the last power of two")
        }
    }

    private fun text() = buildString {
        append("# ").append(name).append(": the answers the frozen Kotlin reference gave ")
        append("(docs/phase6-kotlin-references.md).\n")
        append("# Their count, the digest after the last, and after each power of two (at N).\n")
        append("count ").append(count).append('\n')
        append("digest ").append(digest.hex()).append('\n')
        for ((n, value) in ats) append("at ").append(n).append(' ').append(value.hex()).append('\n')
    }

    private class Golden(val count: Long, val digest: Long, val ats: Map<Long, Long>)

    companion object {
        private const val OFFSET = -0x340d631b7bdddcdbL // 0xcbf29ce484222325
        private const val PRIME = 0x100000001b3L
        private const val ENV = "REFERENCE_GOLDEN"
        private val RESOURCES = File("src/test/resources/golden")

        /** A tape of the code under test, checked against `golden/<name>.txt`. */
        operator fun invoke(name: String): GoldenTape {
            if (System.getenv(ENV) != null) {
                fail(
                    "$name: $ENV is set, but this tape records the code under test, not the reference. " +
                        "Golden files come from the reference at its generator commit " +
                        "(docs/phase6-kotlin-references.md).",
                )
            }
            return GoldenTape(name, read(name), writing = false)
        }

        /**
         * A tape of the frozen reference's answers: checked like any other, or, with
         * `REFERENCE_GOLDEN=write`, written to the module's `src/test/resources/golden/<name>.txt`.
         */
        fun ofReference(name: String): GoldenTape {
            val writing = System.getenv(ENV) == "write"
            return GoldenTape(name, if (writing) null else read(name), writing)
        }

        /** Runs [block] on a tape of the code under test, then finishes it. */
        inline fun <T> check(name: String, block: (GoldenTape) -> T): T {
            val tape = GoldenTape(name)
            return block(tape).also { tape.finish() }
        }

        /** One step of FNV-1a over the eight bytes of [word], low byte first. */
        private fun mix(hash: Long, word: Long): Long {
            var h = hash
            var w = word
            repeat(8) {
                h = (h xor (w and 0xff)) * PRIME
                w = w ushr 8
            }
            return h
        }

        private fun Long.hex() = java.lang.Long.toHexString(this).padStart(16, '0')

        private fun read(name: String): Golden? {
            val stream = Thread.currentThread().contextClassLoader.getResourceAsStream("golden/$name.txt")
                ?: return null
            var count = -1L
            var digest = 0L
            val ats = HashMap<Long, Long>()
            stream.bufferedReader().useLines { lines ->
                for (line in lines) {
                    if (line.startsWith("#") || line.isBlank()) continue
                    val parts = line.split(' ')
                    when (parts[0]) {
                        "count" -> count = parts[1].toLong()
                        "digest" -> digest = java.lang.Long.parseUnsignedLong(parts[1], 16)
                        "at" -> ats[parts[1].toLong()] = java.lang.Long.parseUnsignedLong(parts[2], 16)
                        else -> fail("$name: unreadable golden line ${parts[0]}")
                    }
                }
            }
            if (count < 0) fail("$name: the golden file has no count")
            return Golden(count, digest, ats)
        }
    }
}
