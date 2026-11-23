package com.jtn0123.audiowhisper.golden

import java.io.File
import java.io.InputStream
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
 * Golden files are kept in a [Store]: on the JVM, [Classpath], which reads `golden/<name>.txt` from
 * the test classpath (each module's `src/test/resources`); on a device, the test APK's assets
 * (androidTest's `DeviceGoldens`). They are written only by a tape made with [ofReference], at the
 * commit that still had the reference, with the store asked to write (`REFERENCE_GOLDEN=write`,
 * or the instrumentation argument `referenceGolden write`) and the test run again; a tape of the
 * code under test refuses to write them, so a regression can never be recorded as the reference.
 */
class GoldenTape private constructor(
    private val name: String,
    private val store: Store,
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

    /**
     * Records [value], what [what] answered, as the two parts [what] and the value's text, so
     * null is told apart from any value that prints as "null". The value's `toString()` must be
     * its content, as a number's, a string's, a list's or a data class's is.
     */
    fun answer(what: String, value: Any?) = record(listOf(what, if (value == null) "null" else "=$value"))

    /** Records each of [answers] in order, one answer each. */
    fun recordAll(answers: Iterable<String>) = answers.forEach { record(it) }

    /** Checks the count and the digest, or writes the golden file when the reference is writing it. */
    fun finish() {
        check(!finished) { "$name: finished twice" }
        finished = true
        if (writing) {
            store.write(name, text())
            return
        }
        val golden = golden ?: fail("$name: no golden file golden/$name.txt in ${store.where}")
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

    /**
     * Where golden files are read and written, and what the run asks of them: [Classpath] on the
     * JVM, the test APK's assets and the app's files on a device (androidTest's `DeviceGoldens`).
     */
    interface Store {
        /** What the run asks of golden files: null to check them, "write" to write them from the reference. */
        val request: String?

        /** Where the golden files are read from, as a failure names it. */
        val where: String

        /** The golden file `golden/<name>.txt`, or null when there is none. */
        fun read(name: String): InputStream?

        /** Writes the golden file `golden/<name>.txt` as [text]. */
        fun write(name: String, text: String)
    }

    /**
     * The JVM's store: `golden/<name>.txt` on the test classpath, written to the module's
     * `src/test/resources/golden`, asked for by `REFERENCE_GOLDEN`.
     */
    object Classpath : Store {
        override val request: String? get() = System.getenv(ENV)
        override val where = "the test classpath"

        override fun read(name: String): InputStream? =
            Thread.currentThread().contextClassLoader.getResourceAsStream("golden/$name.txt")

        override fun write(name: String, text: String) {
            val file = File(RESOURCES, "$name.txt")
            file.parentFile.mkdirs()
            file.writeText(text)
        }
    }

    companion object {
        private const val OFFSET = -0x340d631b7bdddcdbL // 0xcbf29ce484222325
        private const val PRIME = 0x100000001b3L
        private const val ENV = "REFERENCE_GOLDEN"
        private val RESOURCES = File("src/test/resources/golden")

        /** A tape of the code under test, checked against `golden/<name>.txt` in [store]. */
        operator fun invoke(name: String, store: Store = Classpath): GoldenTape {
            store.request?.let { request ->
                fail(
                    "$name: golden files are asked for ($request), but this tape records the code under test, " +
                        "not the reference. Golden files come from the reference at its generator commit " +
                        "(docs/phase6-kotlin-references.md).",
                )
            }
            return GoldenTape(name, store, read(name, store), writing = false)
        }

        /**
         * A tape of the frozen reference's answers: checked like any other, or, when [store] is asked
         * to write, written to it (on the JVM, the module's `src/test/resources/golden/<name>.txt`).
         */
        fun ofReference(name: String, store: Store = Classpath): GoldenTape {
            val writing = store.request == "write"
            return GoldenTape(name, store, if (writing) null else read(name, store), writing)
        }

        /** Runs [block] on a tape of the code under test, checked against [store], then finishes it. */
        inline fun <T> check(name: String, store: Store = Classpath, block: (GoldenTape) -> T): T {
            val tape = GoldenTape(name, store)
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

        private fun read(name: String, store: Store): Golden? {
            val stream = store.read(name) ?: return null
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
