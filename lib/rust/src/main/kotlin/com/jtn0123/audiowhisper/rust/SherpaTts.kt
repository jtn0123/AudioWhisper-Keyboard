package com.jtn0123.audiowhisper.rust

import java.io.File
import java.lang.ref.Reference
import java.nio.ByteBuffer
import java.nio.ByteOrder
import java.util.concurrent.atomic.AtomicLong

/** Rust owns generation/cancellation/model leases. Android owns files and synchronous playback. */
class SherpaTts(directory: String) : AutoCloseable {
    private val allocation = Allocation(TtsNative.create(directory, File(directory, "loona.wav").readBytes()))
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    val sampleRate: Int = TtsNative.rate(allocation.id.get())

    /** PCM is borrowed only until audio returns; copy it to keep it. Never retain the buffer. */
    fun speak(text: String, onAudio: (ByteBuffer) -> Boolean) {
        try { TtsNative.generate(allocation.id.get(), text, Callback(onAudio)) }
        finally { Reference.reachabilityFence(this) }
    }
    fun cancel() {
        try { allocation.id.get().takeIf { it != 0L }?.let(TtsNative::cancel) }
        finally { Reference.reachabilityFence(this) }
    }
    override fun close() = cleanable.clean()

    private class Allocation(handle: Long) : Runnable {
        val id = AtomicLong(handle)
        override fun run() { TtsNative.close(id.getAndSet(0)) }
    }
    private class Callback(val consume: (ByteBuffer) -> Boolean) : TtsAudio {
        override fun audio(pcm: ByteBuffer): Boolean = consume(pcm.asReadOnlyBuffer().order(ByteOrder.nativeOrder()))
    }
}

/** Internal named JNI callback. No callback can outlive its synchronous speak invocation. */
internal interface TtsAudio { fun audio(pcm: ByteBuffer): Boolean }
internal object TtsNative {
    init { RustLibrary.load() }
    @JvmStatic external fun create(directory: String, referenceWav: ByteArray): Long
    @JvmStatic external fun rate(handle: Long): Int
    @JvmStatic external fun generate(handle: Long, text: String, callback: TtsAudio)
    @JvmStatic external fun cancel(handle: Long)
    @JvmStatic external fun close(handle: Long)
}
