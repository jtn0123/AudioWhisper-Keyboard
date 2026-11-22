package com.jtn0123.audiowhisper.reader

import androidx.annotation.Keep
import com.jtn0123.audiowhisper.rust.SherpaTts
import java.lang.ref.Reference
import java.nio.ByteBuffer
import java.nio.ByteOrder

/** Test APK only. Reflection is bound before timed synthesis; the production owner stays live. */
@Keep
internal object TtsComparison {
    fun bind(engine: SherpaTts): (String, (ByteBuffer) -> Boolean) -> Unit {
        val handle = SherpaTts::class.java.getDeclaredField("handle").apply { isAccessible = true }
        return { text, consume ->
            try { generate(handle.getLong(engine), text, 42, Audio(consume)) }
            finally { Reference.reachabilityFence(engine) }
        }
    }
    @JvmStatic private external fun generate(handle: Long, text: String, seed: Int, audio: Audio)
    @Keep class Audio(private val consume: (ByteBuffer) -> Boolean) {
        fun audio(pcm: ByteBuffer) = consume(pcm.asReadOnlyBuffer().order(ByteOrder.nativeOrder()))
    }
}
