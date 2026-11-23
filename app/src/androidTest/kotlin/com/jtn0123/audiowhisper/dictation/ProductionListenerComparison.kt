package com.jtn0123.audiowhisper.dictation

import android.content.Context
import com.jtn0123.audiowhisper.rust.MicRules
import java.io.File
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.security.MessageDigest
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicInteger
import org.json.JSONArray
import org.json.JSONObject

/** Exercises the real recorder executor and production VAD adapter with file replay only. */
internal object ProductionListenerComparison {
    fun run(context: Context, wav: File, expected: JSONObject): JSONObject {
        val samples = MicRules.pcm16(wav.readBytes()).size
        val chunks = CountDownLatch((samples+1599)/1600)
        val errors = AtomicInteger()
        val phrases = JSONArray(); val previews = JSONArray()
        fun hash(audio: FloatArray): String {
            val bytes=ByteBuffer.allocate(audio.size*4).order(ByteOrder.LITTLE_ENDIAN)
            bytes.asFloatBuffer().put(audio)
            return MessageDigest.getInstance("SHA-256").digest(bytes.array()).joinToString(""){"%02x".format(it)}
        }
        val listener=PhraseListener(context, { chunks.countDown() }, { errors.incrementAndGet() },
            { phrases.put(JSONObject().put("pcm_sha256",hash(it)).put("samples",it.size)) },
            { previews.put(JSONObject().put("pcm_sha256",hash(it)).put("samples",it.size)) }, {}, { errors.incrementAndGet() })
        try {
            // The only producer factory is Replay.open; AudioRecord is never reachable here.
            listener.start { MicAudio.Replay.open(wav) }
            check(chunks.await(20,TimeUnit.SECONDS)) { "Production replay did not drain" }
        } finally { listener.stop(true) }
        check(errors.get()==0) { "Production replay model/mic error" }
        for ((actual,key) in listOf(phrases to "segments",previews to "previews")) {
            val reference=expected.getJSONArray(key)
            check(actual.length()==reference.length()) { "Production replay count mismatch" }
            repeat(actual.length()) { index ->
                check(actual.getJSONObject(index).getString("pcm_sha256")==reference.getJSONObject(index).getString("pcm_sha256")) { "Production replay PCM mismatch" }
                check(actual.getJSONObject(index).getInt("samples")==reference.getJSONObject(index).getInt("samples"))
            }
        }
        check(context.noBackupFilesDir.listFiles()?.none { it.name.startsWith("dictation-vad-") } != false)
        return JSONObject().put("phrases",phrases).put("previews",previews).put("errors",errors.get()).put("identical",true)
    }
}
