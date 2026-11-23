package com.jtn0123.audiowhisper.suggest

import android.content.Context
import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.Learner
import com.jtn0123.audiowhisper.rust.ReaderRules
import com.jtn0123.audiowhisper.rust.ReaderTree
import com.jtn0123.audiowhisper.rust.ReaderWalk
import com.jtn0123.audiowhisper.reader.PageText
import com.jtn0123.audiowhisper.reader.ReferenceReader
import com.jtn0123.audiowhisper.reader.TextBlock
import java.io.File
import java.security.MessageDigest
import org.json.JSONArray
import org.json.JSONObject

/**
 * Synthetic in-memory stores only. No microphone, playback, user text or user dictionary. The
 * reader's answers are recorded on the golden tape [TAPE] (docs/phase6-kotlin-references.md).
 */
object BoundaryReplay {
    const val TAPE = "boundary-reader"

    @JvmStatic fun run(context: Context, goldens: GoldenTape.Store): String {
        // App startup clears cacheDir asynchronously; use the same private-file boundary as A1Replay.
        val file = File(context.filesDir, "boundary-fixture.dict")
        context.assets.open("dicts/main_en-US.dict").use { input -> file.outputStream().use { input.copyTo(it) } }
        try {
            val engine = checkNotNull(LatinDictionaryEngine.open(file.path, 0, file.length(), "en_US"))
            return engine.use { measure(it, goldens).toString() }
        } finally { check(file.delete()) { "boundary dictionary cleanup" } }
    }

    private fun measure(engine: LatinDictionaryEngine, goldens: GoldenTape.Store): JSONObject {
        val tape = GoldenTape.ofReference(TAPE, goldens)
        val random = kotlin.random.Random(20261114)
        repeat(2_000) {
            val pieces = List(random.nextInt(64)) { index ->
                TextBlock(listOf("a ", "• ", "\ud800", "hello", "\ud83d\ude00", "\u00a0", " ")[random.nextInt(7)],
                    index / 3 * 40, index / 3 * 40 + 20, random.nextBoolean(), {})
            }
            fun signature(blocks: List<TextBlock>) = blocks.map { listOf(it.text, it.top, it.bottom, it.scrolls) }
            val blocks = signature(ReferenceReader.PageText.blocks(pieces))
            check(blocks == signature(PageText.blocks(pieces))) { "reader blocks sample $it" }
            tape.record(listOf("blocks") + blocks.flatMap { block -> block.map { field -> field.toString() } })
            val found = ReferenceReader.PageText.find(pieces, "hello", 0, 640)
            check(found == PageText.find(pieces, "hello", 0, 640)) { "reader find sample $it" }
            tape.answer("find", found)
            val read = pieces.take(random.nextInt(pieces.size + 1)).map { it.text }
            val resumed = ReferenceReader.PageText.resumeAfter(pieces, read)
            check(resumed == PageText.resumeAfter(pieces, read)) { "reader resume sample $it" }
            tape.answer("resumeAfter", resumed)
        }
        tape.finish()
        val history = LearnedWords(null)
        val learner = Learner(history.store)
        val guesses = (0 until 32).map { GlideGuess(listOf("the", "hello", "world", "later")[it % 4], it / 10f) }
        val ranker = GlideRanker(history, engine)
        val previous = PreviousWords(listOf("hello"), false)
        val texts = List(512) { "paragraph $it. " }
        val edges = IntArray(1024) { it * 10 }
        val scrolls = BooleanArray(512) { true }
        val tree = object : ReaderTree {
            override fun facts(id: Int) = intArrayOf(0, id * 10, 100, id * 10 + 10,
                0, 0, 1, 0, 0, 0, 0, 0, if (id == 0) 512 else 0)
            override fun className(id: Int) = "android.widget.TextView"
            override fun text(id: Int) = "paragraph $id"
            override fun child(id: Int, index: Int) = index + 1
        }
        val result = JSONObject()
        result.put("reader_reference_comparisons", 6_000)
        val units = List(65_536) { "x${it.toChar()}z" }
        val unitEdges = IntArray(units.size * 2) { it * 4 }
        val roundTrip = ReaderRules.join(units, unitEdges, BooleanArray(units.size))
        check(roundTrip.size == units.size) { "UTF-16 cohort size" }
        for (i in units.indices) {
            val block = roundTrip[i]
            check(block.text == units[i] && block.first == i && block.top == unitEdges[i * 2] &&
                block.bottom == unitEdges[i * 2 + 1]) { "UTF-16 packet unit $i" }
        }
        result.put("reader_unit_roundtrips", units.size)
        val digest = MessageDigest.getInstance("SHA-256")
        fun fingerprint(value: String) { digest.update(value.toByteArray(Charsets.UTF_8)) }
        fun trial(name: String, count: Int, action: () -> Unit) {
            repeat(100) { action() }
            val samples = LongArray(20) {
                val start = System.nanoTime()
                repeat(count) { action() }
                (System.nanoTime() - start) / count
            }.sorted()
            result.put(name, JSONObject().put("p50_ns", samples[9]).put("p95_ns", samples[18])
                .put("samples_ns", JSONArray(samples)))
        }
        trial("learner_finished", 100) {
            learner.reset()
            learner.learnNative("HELLO", "say ", -1, true, engine.session)
            fingerprint(learner.learnNative("The", "", 0, false, engine.session).toString())
            learner.undo(0)
        }
        trial("glide_32", 100) { fingerprint(ranker.rank(guesses, previous).joinToString("|")) }
        trial("reader_join_512", 10) {
            fingerprint(ReaderRules.join(texts, edges, scrolls).joinToString("|") { "${it.first}:${it.text}:${it.top}:${it.bottom}" })
        }
        trial("reader_find_512", 10) {
            fingerprint(ReaderRules.find(texts, edges, "paragraph 400", 0, 10240)?.joinToString().orEmpty())
        }
        trial("reader_resume_512", 10) {
            fingerprint(ReaderRules.resumeAfter(texts, texts.takeLast(8)).toString())
        }
        trial("android_adapter_walk_513", 10) {
            fingerprint(ReaderWalk.walk(tree, intArrayOf(0, 0, 1080, 1600)).joinToString())
        }
        result.put("sha256", digest.digest().joinToString("") { "%02x".format(it) })
        return result
    }
}
