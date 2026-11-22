package com.jtn0123.audiowhisper.rust

import java.nio.ByteBuffer
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.read
import kotlin.concurrent.write

/** C API owners. PCM calls borrow a direct, native-order Float32 buffer only until return. */
class SherpaAsr(modelDirectory: String) : AutoCloseable {
    private val allocation = Allocation(SherpaNative.createRecognizer(modelDirectory))
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    fun transcribe(pcm: ByteBuffer, samples: Int): String = allocation.transcribe(pcm, samples)
    fun cancel() = allocation.cancel()
    override fun close() = cleanable.clean()
    private class Allocation(private var handle: Long) : Runnable {
        private val lock = ReentrantReadWriteLock()
        fun transcribe(pcm: ByteBuffer, samples: Int): String = lock.read {
            check(handle != 0L) { "ASR closed" }; SherpaNative.transcribe(handle, pcm, samples)
        }
        fun cancel() = lock.read { if (handle != 0L) SherpaNative.cancel(handle) }
        override fun run() = lock.write { val owned = handle; handle = 0; SherpaNative.freeRecognizer(owned) }
    }
}

/** One batch per mic chunk: speech flag/end, raw boundaries, phrases with lead-in, and preview. */
class SherpaVad(model: String) : AutoCloseable {
    data class Batch(val detected: Boolean, val transition: Int, val end: Long, val boundaries: LongArray, val phrases: Array<FloatArray>, val preview: FloatArray?)
    private val allocation = Allocation(SherpaNative.createVad(model))
    private val cleanable = RustLibrary.cleaner.register(this, allocation)
    @Synchronized fun push(pcm: ByteBuffer, samples: Int): Batch = allocation.call(pcm, samples, 0)
    @Synchronized fun stop(keep: Boolean): Batch = allocation.call(null, 0, if (keep) 1 else 2)
    @Synchronized override fun close() = cleanable.clean()
    private class Allocation(private var handle: Long) : Runnable {
        @Synchronized fun call(pcm: ByteBuffer?, samples: Int, mode: Int): Batch {
            check(handle != 0L) { "VAD closed" }
            val payload = SherpaNative.vad(handle, pcm, samples, mode)
            val metadata = payload[0] as LongArray
            val count = metadata[2].toInt()
            return Batch(metadata[0] != 0L, metadata.last().toInt(), metadata[1], metadata.copyOfRange(3, metadata.lastIndex),
                Array(count) { payload[it + 1] as FloatArray }, payload[count + 1] as FloatArray?)
        }
        @Synchronized override fun run() { val owned = handle; handle = 0; SherpaNative.freeVad(owned) }
    }
}
