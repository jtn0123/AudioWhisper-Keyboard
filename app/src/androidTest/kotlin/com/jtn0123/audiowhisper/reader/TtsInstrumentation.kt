package com.jtn0123.audiowhisper.reader

import android.app.Activity
import android.app.Instrumentation
import android.os.Bundle
import android.os.Debug
import com.jtn0123.audiowhisper.golden.DeviceGoldens
import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.SherpaTts
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.MessageDigest
import org.json.JSONArray
import org.json.JSONObject

/**
 * Files only: never constructs AudioTrack, TextToSpeech, AudioRecord or any playback voice. The
 * reference voice's audio, by its digests, is recorded on the golden tape [TAPE]
 * (docs/phase6-kotlin-references.md). `-e timings false` skips the latency trials, `-e stress
 * false` the cancel and respeak cycles. A failed comparison reports `failure_kind=parity`.
 */
class TtsInstrumentation : Instrumentation() {
    companion object {
        const val TAPE = "tts-pocket-loona"
        private const val PARITY = "parity"
    }

    private lateinit var options: Bundle
    override fun onCreate(arguments: Bundle?) { super.onCreate(arguments); options = arguments ?: Bundle(); start() }
    private fun hex(bytes: ByteArray) = bytes.joinToString("") { "%02x".format(it) }
    private fun bytes(samples: FloatArray): ByteArray = ByteBuffer.allocate(samples.size * 4)
        .order(ByteOrder.LITTLE_ENDIAN).also { it.asFloatBuffer().put(samples) }.array()
    private fun generate(text: String, index: Int, reference: Boolean, directory: File, deterministic: Boolean = true): JSONObject {
        val hash = MessageDigest.getInstance("SHA-256")
        val chunks = JSONArray()
        val chunkHashes = JSONArray()
        var first = -1L
        var count = 0
        val old = if (reference) checkNotNull(ReferenceVoiceEngine.load(targetContext, if (deterministic) 42 else null)) else null
        val native = if (!reference) SherpaTts(VoiceEngine.dir(targetContext).path) else null
        val nativeGenerate = native?.let { if (deterministic) TtsComparison.bind(it) else it::speak }
        val rate = old?.sampleRate ?: checkNotNull(native).sampleRate
        try {
            return File(directory, "$index-${if (reference) 1 else 0}.pcm").outputStream().use { output ->
                fun audio(pcm: ByteArray): Boolean {
                    hash.update(pcm); output.write(pcm); count += pcm.size / 4; chunks.put(pcm.size / 4)
                    chunkHashes.put(hex(MessageDigest.getInstance("SHA-256").digest(pcm)))
                    return true
                }
                val start = System.nanoTime()
                if (reference) old!!.speak(text) {
                    if (first < 0) first = System.nanoTime()
                    audio(bytes(it))
                }
                else nativeGenerate!!(text) { buffer ->
                    if (first < 0) first = System.nanoTime()
                    val pcm = ByteArray(buffer.remaining()); buffer.get(pcm); audio(pcm)
                }
                val end = System.nanoTime()
                check(count > 0 && first >= start)
                JSONObject().put("sha256", hex(hash.digest())).put("samples", count).put("sample_rate", rate)
                    .put("chunks", chunks).put("chunk_sha256", chunkHashes).put("start_ns", start).put("first_audio_ns", first).put("end_ns", end)
            }
        } finally { old?.close(); native?.close() }
    }
    private fun identity(record: JSONObject) = JSONObject().put("sha256", record.getString("sha256"))
        .put("samples", record.getInt("samples")).put("sample_rate", record.getInt("sample_rate"))
        .put("chunks", record.getJSONArray("chunks")).put("chunk_sha256", record.getJSONArray("chunk_sha256")).toString()

    private fun stress(text: String, expected: JSONObject): JSONObject {
        val directory = VoiceEngine.dir(targetContext).path
        val native = SherpaTts(directory)
        val generate = TtsComparison.bind(native)
        val heaps = JSONArray()
        try {
            repeat(20) { generate(text) { native.cancel(); false } } // Allocator/model-cache warmup.
            repeat(200) { cycle ->
                var received = 0
                generate(text) { pcm -> received += pcm.remaining() / 4; native.cancel(); false }
                check(received in 1 until expected.getInt("samples"))
                // An immediate new generation must remain usable after cancellation.
                var resumed = 0
                generate(text) { pcm ->
                    resumed += pcm.remaining() / 4
                    val firstChunk = ByteArray(pcm.remaining()); pcm.get(firstChunk)
                    check(hex(MessageDigest.getInstance("SHA-256").digest(firstChunk)) == expected.getJSONArray("chunk_sha256").getString(0))
                    false
                }
                check(resumed > 0)
                if (cycle % 10 == 0 || cycle == 199) {
                    Runtime.getRuntime().gc()
                    heaps.put(JSONObject().put("cycle", cycle).put("at_ns", System.nanoTime())
                        .put("native_heap_bytes", Debug.getNativeHeapAllocatedSize()))
                }
                sendStatus(0, Bundle().apply { putInt("cycle", cycle); putLong("at_ns", System.nanoTime()) })
            }
        } finally { native.close() }
        val firstHeap = (0 until minOf(5, heaps.length())).maxOf { heaps.getJSONObject(it).getLong("native_heap_bytes") }
        val lastHeap = (maxOf(0, heaps.length() - 5) until heaps.length()).maxOf { heaps.getJSONObject(it).getLong("native_heap_bytes") }
        // The caller persists these observations before requiring the same bounded plateau.
        // Exercise close from INSIDE a callback: physical release must wait for callback return.
        val closing = SherpaTts(directory)
        var callbackReturned = false
        TtsComparison.bind(closing)(text) { closing.close(); callbackReturned = true; false }
        check(callbackReturned)
        closing.close()
        // Return the complete observations. A plateau is reviewed from these samples; a crash-free
        // run alone is not a leak detector, and the native-owner tests establish lifetime ordering.
        return JSONObject().put("cycles", 200).put("immediate_respeaks", 200).put("warmup_cycles", 20)
            .put("close_inside_callback", true).put("heap_samples", heaps).put("heap_plateau", lastHeap <= firstHeap)
    }
    override fun onStart() {
        val result = Bundle()
        val directory = File(targetContext.filesDir, "tts-fixture-audio").apply { mkdirs() }
        try {
            val sentences = context.assets.open("tts-sentences.txt").bufferedReader().use { it.readLines() }
            check(sentences.size == 5)
            if (options.getString("randomnessControl") == "true") {
                val control = JSONArray()
                for (index in 0..3) control.put(JSONObject().put("reference", index < 2)
                    .put("identity", JSONObject(identity(generate(sentences[0], 90 + index, index < 2, directory, deterministic = false)))))
                File(targetContext.filesDir, "tts-randomness-control.json").writeText(control.toString(2))
                result.putBoolean("tts_randomness_control", true)
                finish(Activity.RESULT_OK, result)
                return
            }
            val records = JSONArray()
            val report = JSONObject().put("identical", false)
            val reportFile = File(targetContext.filesDir, "tts-comparison.json")
            val expected = arrayListOf<JSONObject>()
            val tape = GoldenTape.ofReference(TAPE, DeviceGoldens(this, options))
            for ((index, text) in sentences.withIndex()) {
                val old = generate(text, index, true, directory)
                val native = generate(text, index, false, directory)
                val matches = identity(old) == identity(native)
                records.put(JSONObject().put("fixture", index).put("identical", matches).put("reference", old).put("native", native))
                report.put("fixtures", records); reportFile.writeText(report.toString(2))
                check(matches) { PARITY }
                tape.answer("fixture $index", identity(old))
                expected += native
            }
            tape.finish()
            if (options.getString("timings") != "false") {
                val timings = JSONArray()
                repeat(10) { trial ->
                    for (reference in if (trial % 2 == 0) listOf(true, false) else listOf(false, true)) {
                        val record = generate(sentences[0], 10 + trial, reference, directory)
                        check(identity(record) == identity(expected[0])) { PARITY }
                        timings.put(JSONObject().put("trial", trial).put("reference", reference)
                            .put("start_ns", record.getLong("start_ns")).put("first_audio_ns", record.getLong("first_audio_ns"))
                            .put("end_ns", record.getLong("end_ns")))
                    }
                }
                report.put("timings", timings)
                reportFile.writeText(report.toString(2))
            }
            if (options.getString("stress") != "false") {
                val stress = stress(sentences[1], expected[1])
                report.put("stress", stress)
                reportFile.writeText(report.toString(2))
                check(stress.getBoolean("heap_plateau")) // Persist failure evidence; never relax the rule.
            }
            report.put("identical", true); reportFile.writeText(report.toString(2))
            result.putBoolean("tts_complete", true); finish(Activity.RESULT_OK, result)
        } catch (failure: Throwable) {
            result.putString("failure_type", failure.javaClass.name)
            // A golden tape fails with an AssertionError naming it.
            if (failure.message == PARITY || failure is AssertionError && failure.message.orEmpty().startsWith("$TAPE: ")) {
                result.putString("failure_kind", "parity")
            }
            result.putString("failure_frames", failure.stackTrace.take(6).joinToString(";") { "${it.className}.${it.methodName}:${it.lineNumber}" })
            finish(Activity.RESULT_CANCELED, result)
        }
    }
}
