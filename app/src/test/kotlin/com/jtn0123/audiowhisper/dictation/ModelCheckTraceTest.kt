package com.jtn0123.audiowhisper.dictation

import com.jtn0123.audiowhisper.golden.GoldenTape
import java.io.File
import java.nio.file.Files
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

/**
 * Seeded traces holding [ModelCheck], whose decisions are Rust's (dictation::model_check), to
 * main's ([ReferenceModelCheck]): a model's files are written whole, cut short, changed, deleted,
 * touched, made unreadable and remembered or forgotten, and after each event both check them,
 * quickly or in full, each with its own memory of the files that passed.
 *
 * Compared: the state, every key looked up, and every key remembered, in order. One intended
 * difference, counted: main remembered each file as it passed, this remembers them together once
 * the check is over, so a check cut short by process death hashes those files again next time.
 */
class ModelCheckTraceTest {
    /** Remembered keys, and every lookup and addition, in order. */
    private class Memory : VerifiedFiles {
        val keys = mutableSetOf<String>()
        val asked = mutableListOf<String>()
        val added = mutableListOf<String>()

        override fun contains(key: String) = (key in keys).also { asked += key }

        override fun add(key: String) {
            keys += key
            added += key
        }
    }

    /** The reference's answers (docs/phase6-kotlin-references.md): each check, and the keys kept. */
    private val tape = GoldenTape.ofReference("model-check-traces")

    /** Times files are written at: one apart, so a rewrite is never mistaken for the file before it. */
    private var writtenAt = 1_500_000_000_000L

    private fun written(file: File, bytes: ByteArray) {
        file.writeBytes(bytes)
        file.setLastModified(writtenAt)
        writtenAt += 1_000
    }

    /** [text] with the trace's own folder, which differs each run, put as `<dir>`. */
    private fun File.relative(text: String) = text.replace(path, "<dir>")

    @Test
    fun checksMatchMain() {
        for (seed in SEEDS) trace(seed)
        tape.finish()
    }

    private fun trace(seed: Long) {
        val random = Random(seed)
        val dir = Files.createTempDirectory("model-trace").toFile()
        val contents = List(FILES) { ByteArray(random.nextInt(1, 64)).also(random::nextBytes) }
        val expected = contents.mapIndexed { i, bytes ->
            ExpectedFile("f$i.onnx", bytes.size.toLong(), ReferenceModelCheck.sha256(File(dir, "x").apply { writeBytes(bytes) }))
        }
        File(dir, "x").delete()
        val main = Memory()
        val new = Memory()
        val seen = HashMap<String, Int>()
        try {
            repeat(EVENTS) { event ->
                val kind = step(random, dir, expected, contents, main, new)
                seen[kind] = (seen[kind] ?: 0) + 1
                assertEquals(main.keys, new.keys, "seed $seed event $event ($kind)")
                tape.record(listOf("keys") + main.keys.map { dir.relative(it) }.sorted())
            }
        } finally {
            dir.listFiles()?.forEach { it.setReadable(true) }
            dir.deleteRecursively()
        }
        println("Model check trace seed $seed: $EVENTS events, 0 differ, seen $seen")
        for (kind in KINDS) assertTrue((seen[kind] ?: 0) >= MIN_SEEN, "seed $seed saw $kind ${seen[kind]} times")
    }

    /** One event; checks compare both and say what they found. */
    private fun step(random: Random, dir: File, expected: List<ExpectedFile>, contents: List<ByteArray>, main: Memory, new: Memory): String {
        val i = random.nextInt(FILES)
        val file = File(dir, expected[i].name)
        return when (random.nextInt(20)) {
            0, 1 -> "written".also { file.setReadable(true); written(file, contents[i]) }
            2 -> "changed".also { file.setReadable(true); written(file, contents[i].copyOf().also { it[0] = (it[0] + 1).toByte() }) }
            3 -> "cut short".also { file.setReadable(true); written(file, contents[i].copyOf(contents[i].size - 1)) }
            4 -> "deleted".also { file.setReadable(true); file.delete() }
            5 -> "touched".also { file.setLastModified(1_000_000_000_000L + random.nextLong(1_000_000) * 1000) }
            6 -> "unreadable".also { file.setReadable(false) }
            7 -> "readable".also { file.setReadable(true) }
            8 -> "forgotten".also { main.keys.clear(); new.keys.clear() }
            9 -> "repaired".also { expected.indices.forEach { File(dir, expected[it].name).apply { setReadable(true); written(this, contents[it]) } } }
            in 10..13 -> {
                val state = outcome { ModelCheck.quick(dir, expected) }
                val reference = outcome { ReferenceModelCheck.quick(dir, expected) }
                assertEquals(reference, state)
                tape.record(listOf("quick", dir.relative(reference.toString())))
                "quick ${name(state)}"
            }
            else -> full(dir, expected, main, new)
        }
    }

    private fun full(dir: File, expected: List<ExpectedFile>, main: Memory, new: Memory): String {
        main.asked.clear(); main.added.clear(); new.asked.clear(); new.added.clear()
        val state = outcome { ModelCheck.full(dir, expected, new) }
        val reference = outcome { ReferenceModelCheck.full(dir, expected, main) }
        assertEquals(reference, state)
        assertEquals(main.asked, new.asked, "keys looked up")
        assertEquals(main.added, new.added, "keys remembered")
        tape.record(listOf("full", dir.relative(reference.toString()), "asked") + main.asked.map { dir.relative(it) } +
            "added" + main.added.map { dir.relative(it) })
        // Main's order differs only if a file passed before the last one was looked up.
        val batched = new.added.any { it != new.asked.last() }
        return if (batched) "full ${name(state)}, remembered at the end" else "full ${name(state)}"
    }

    /** What a check came to: its state, or what it threw, so a throw is a difference like any other. */
    private fun outcome(check: () -> ModelState): Any = try {
        check()
    } catch (e: RuntimeException) {
        "threw ${e::class.simpleName}"
    }

    private fun name(state: Any) = when (state) {
        ModelState.Ready -> "ready"
        is ModelState.Missing -> "missing"
        is ModelState.Damaged -> "damaged"
        else -> "threw"
    }

    private companion object {
        val SEEDS = longArrayOf(20261122, 7, 4_000_000_007)
        const val FILES = 4
        const val EVENTS = 50_000
        const val MIN_SEEN = 20
        val KINDS = listOf(
            "quick ready", "quick missing", "quick damaged", "full ready", "full missing", "full damaged",
            "full ready, remembered at the end", "full damaged, remembered at the end", "unreadable", "touched", "forgotten",
        )
    }
}
