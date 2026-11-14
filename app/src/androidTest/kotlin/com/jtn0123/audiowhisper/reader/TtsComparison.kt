package com.jtn0123.audiowhisper.reader

import androidx.annotation.Keep
import com.jtn0123.audiowhisper.rust.SherpaTts
import java.lang.ref.Reference
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.atomic.AtomicLong

/** Test APK only. Reflection is bound before timed synthesis; the production owner stays live. */
@Keep
internal object TtsComparison {
    fun bind(engine: SherpaTts): (String, (ByteBuffer) -> Boolean) -> Unit {
        val allocation = SherpaTts::class.java.getDeclaredField("allocation")
            .apply { isAccessible = true }.get(engine)
        val id = allocation.javaClass.getDeclaredField("id").apply { isAccessible = true }
            .get(allocation) as AtomicLong
        return { text, consume ->
            try { generate(id.get(), text, 42, Audio(consume)) }
            finally { Reference.reachabilityFence(engine) }
        }
    }
    @JvmStatic private external fun generate(handle: Long, text: String, seed: Int, audio: Audio)
    @Keep class Audio(private val consume: (ByteBuffer) -> Boolean) {
        fun audio(pcm: ByteBuffer) = consume(pcm.asReadOnlyBuffer().order(ByteOrder.nativeOrder()))
    }
}
