package com.jtn0123.audiowhisper.dictation

import android.app.Activity
import android.app.Instrumentation
import android.os.Bundle
import com.jtn0123.audiowhisper.golden.DeviceGoldens
import com.jtn0123.audiowhisper.golden.GoldenTape
import com.jtn0123.audiowhisper.rust.MicRules
import com.jtn0123.audiowhisper.rust.SherpaAsr
import com.jtn0123.audiowhisper.rust.SherpaVad
import com.k2fsa.sherpa.onnx.WaveReader
import java.io.File
import java.nio.file.Files
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.MessageDigest
import org.json.JSONArray
import org.json.JSONObject

/**
 * Test APK only. The report contains hashes/counts/sample positions and monotonic timestamps. Each
 * fixture's WAV and the reference's answers for it, by their digests, are recorded on the golden
 * tape [TAPE]-N (docs/phase6-kotlin-references.md).
 */
class DictationInstrumentation : Instrumentation() {
    companion object {
        const val TAPE = "dictation-fixture"
    }

    private lateinit var options: Bundle
    override fun onCreate(arguments: Bundle?) { super.onCreate(arguments); options = arguments ?: Bundle(); start() }
    private fun hash(text: String) = hash(text.toByteArray(Charsets.UTF_8))
    private fun hash(bytes: ByteArray) = MessageDigest.getInstance("SHA-256").digest(bytes).joinToString("") { "%02x".format(it) }
    private fun pcm(samples: FloatArray): ByteBuffer = ByteBuffer.allocateDirect(samples.size * 4)
        .order(ByteOrder.nativeOrder()).also { it.asFloatBuffer().put(samples) }
    private fun audioHash(samples: FloatArray): String {
        val bytes = ByteBuffer.allocate(samples.size * 4).order(ByteOrder.LITTLE_ENDIAN)
        bytes.asFloatBuffer().put(samples); return hash(bytes.array())
    }
    private fun transcript(text: String) = JSONObject().put("sha256", hash(text)).put("utf8_bytes", text.toByteArray().size)

    private fun run(samples: FloatArray, reference: Boolean, measureOnly: Boolean = false): Pair<JSONObject, JSONObject> {
        val segments = JSONArray(); val previews = JSONArray(); val vad = JSONArray(); val flush = JSONArray()
        val modelDir = ModelFiles.dir(targetContext)
        val loadStart = System.nanoTime()
        val old = if (reference) ReferenceTranscriber(modelDir) else null
        val native = if (!reference) SherpaAsr(modelDir.path) else null
        val loadEnd = System.nanoTime()
        val session = if (measureOnly) null else LiveSessionComparison()
        var firstPartial = -1L
        val start = System.nanoTime()
        var flushing = false
        var acceptedEnd = 0
        fun decode(audio: FloatArray): String = old?.transcribe(audio) ?: native!!.transcribe(pcm(audio), audio.size).trim()
        fun phrase(audio: FloatArray) {
            val text = decode(audio)
            session?.accept(2, audio, text, acceptedEnd.toLong())
            val result = transcript(text).put("samples", audio.size).put("pcm_sha256", audioHash(audio)).put("emitted_at_samples", acceptedEnd)
            result.put("final_flush", flushing)
            segments.put(result); if (flushing) flush.put(result)
        }
        fun preview(audio: FloatArray) {
            val text = decode(audio)
            session?.accept(1, audio, text, acceptedEnd.toLong())
            previews.put(transcript(text).put("samples", audio.size).put("pcm_sha256", audioHash(audio)).put("emitted_at_samples", acceptedEnd))
            if (firstPartial < 0) firstPartial = System.nanoTime()
        }
        val refVad = if (reference) ReferencePhraseListener(targetContext.assets, {}, {}, ::phrase, ::preview) else null
        val model = File(targetContext.filesDir, "dictation-fixture-silero.onnx")
        if (!reference) targetContext.assets.open("silero_vad.onnx").use { input -> model.outputStream().use(input::copyTo) }
        val rustVad = if (!reference) SherpaVad(model.path) else null
        val boundaries = mutableListOf<Long>()
        try {
            var offset = 0
            while (offset < samples.size) {
                val chunk = samples.copyOfRange(offset, minOf(offset + 1600, samples.size))
                acceptedEnd = offset + chunk.size
                val detected = if (reference) refVad!!.push(chunk) else {
                    val batch = rustVad!!.push(pcm(chunk), chunk.size)
                    boundaries.addAll(batch.boundaries.toList()); batch.phrases.forEach(::phrase); batch.preview?.let(::preview)
                    batch.detected
                }
                session?.accept(if (detected) 4 else 5, atSamples = acceptedEnd.toLong())
                session?.accept(0, atSamples = acceptedEnd.toLong())
                offset += chunk.size
                vad.put(JSONArray().put(offset).put(detected))
            }
            flushing = true
            if (reference) refVad!!.stop(true) else {
                val batch = rustVad!!.stop(true); boundaries.addAll(batch.boundaries.toList()); batch.phrases.forEach(::phrase)
            }
            if (reference) boundaries.addAll(refVad!!.boundaries)
            session?.accept(3, atSamples = acceptedEnd.toLong())
            val end = System.nanoTime()
            val parity = JSONObject().put("segments", segments).put("previews", previews).put("vad", vad)
                .put("boundaries", JSONArray(boundaries)).put("flush", flush)
                .put("whole", transcript(decode(samples))).put("session_states", session?.states ?: JSONArray())
            val timing = JSONObject().put("start_ns", start).put("first_partial_ns", firstPartial).put("end_ns", end)
                .put("load_start_ns", loadStart).put("load_end_ns", loadEnd).put("audio_samples", samples.size).put("sample_rate", 16_000)
            return parity to timing
        } finally { old?.close(); native?.close(); rustVad?.close() }
    }
    override fun onStart() {
        val scratch = File(targetContext.filesDir, "dictation-fixture-audio")
        val reportFile = File(targetContext.filesDir, "dictation-comparison.json")
        val result = Bundle()
        try {
            val nativeOnly = options.getString("nativeOnly") == "true"
            val oracleFile = File(targetContext.filesDir, "dictation-oracle.json")
            val oracle = if (nativeOnly) JSONObject(oracleFile.readText()) else null
            check(!nativeOnly || options.getString("reuse") == "true")
            val files = if (options.getString("reuse") == "true") scratch.listFiles()!!.sortedBy { it.name }
                else DictationAudioFixtures.create(targetContext, scratch)
            val records = JSONArray()
            val goldens = DeviceGoldens(this, options)
            for ((index, file) in files.withIndex()) {
                sendStatus(0, Bundle().apply { putInt("fixture", index); putLong("start_ns", System.nanoTime()) })
                val samples = if (nativeOnly) {
                    check(hash(file.readBytes()) == oracle!!.getJSONArray("fixtures").getJSONObject(index).getString("wav_sha256"))
                    MicRules.pcm16(file.readBytes())
                } else WaveReader.readWave(file.path).also { check(it.sampleRate == 16_000) }.samples
                val old = if (nativeOnly) oracle!!.getJSONArray("fixtures").getJSONObject(index)
                    .getJSONObject("parity") to JSONObject() else run(samples, true)
                val native = run(samples, false)
                if (old.first.toString() != native.first.toString()) {
                    Files.write(reportFile.toPath(), JSONObject().put("identical", false).put("failure_kind", "parity")
                        .put("fixture", index).put("reference", old.first).put("native", native.first).toString(2).toByteArray())
                    result.putString("failure_kind", "parity"); result.putInt("fixture", index)
                    finish(Activity.RESULT_CANCELED, result); return
                }
                if (index == 0) {
                    check(old.first.getJSONObject("whole").getInt("utf8_bytes") > 0) { "Speech fixture has no transcription" }
                    check(old.first.getJSONArray("segments").length() > 0 && old.first.getJSONArray("previews").length() > 0) { "Speech fixture did not exercise segmentation and preview" }
                }
                if (index == 5) check(old.first.getJSONArray("flush").length() > 0) { "Clipped fixture did not exercise final flush" }
                val wav = hash(Files.readAllBytes(file.toPath()))
                GoldenTape.ofReference("$TAPE-$index", goldens).apply {
                    answer("wav", wav); answer("samples", samples.size); answer("parity", old.first); finish()
                }
                records.put(JSONObject().put("fixture", index).put("wav_sha256", wav)
                    .put("samples", samples.size).put("parity", native.first))
            }
            val production = ProductionListenerComparison.run(targetContext, files.first(), records.getJSONObject(0).getJSONObject("parity"))
            val timing = JSONArray()
            val timed = if (nativeOnly) MicRules.pcm16(files.first().readBytes())
                else WaveReader.readWave(files.first().path).samples
            repeat(if (options.getString("timings") == "false") 0 else 10) { trial ->
                sendStatus(0, Bundle().apply { putInt("trial", trial); putLong("start_ns", System.nanoTime()) })
                for (reference in if (nativeOnly) listOf(false) else if (trial % 2 == 0) listOf(true, false) else listOf(false, true)) {
                    val record = run(timed, reference, measureOnly = true)
                    val expected = JSONObject(records.getJSONObject(0).getJSONObject("parity").toString()).also { it.put("session_states", JSONArray()) }
                    check(record.first.toString() == expected.toString()) { "Timing run changed output" }
                    timing.put(record.second.put("trial", trial).put("reference", reference))
                }
            }
            val report = JSONObject().put("fixtures", records).put("timings", timing).put("production_listener", production).put("identical", true)
                .put("native_only", nativeOnly)
            if (nativeOnly) report.put("oracle_sha256", hash(oracleFile.readBytes()))
            val pending = File(reportFile.path + ".tmp"); Files.write(pending.toPath(), report.toString(2).toByteArray()); check(pending.renameTo(reportFile))
            result.putBoolean("dictation_complete", true); finish(Activity.RESULT_OK, result)
        } catch (failure: Throwable) {
            // Avoid exception messages that might contain decoded text. Numeric fixture is enough.
            result.putString("failure_type", failure.javaClass.name)
            result.putString("failure_frames", failure.stackTrace.take(6).joinToString(";") { "${it.className}.${it.methodName}:${it.lineNumber}" })
            finish(Activity.RESULT_CANCELED, result)
        }
    }
}
