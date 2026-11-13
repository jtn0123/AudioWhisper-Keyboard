package com.jtn0123.audiowhisper.rust

/** Plain scene and MotionEvent packets. This wrapper has no Android dependency, including in JVM tests. */
class KeyboardRenderer(fontPaths: Array<String>) : AutoCloseable {
    private var handle = RenderNative.create(fontPaths)
    private val cleanable = RustLibrary.cleaner.register(this, Free(handle))

    @Synchronized
    fun update(dimensions: FloatArray, colors: IntArray, geometry: FloatArray, labels: Array<String>, candidates: Array<String>) =
        RenderNative.update(handle, dimensions, colors, geometry, labels, candidates)

    @Synchronized
    fun touch(metadata: IntArray, times: LongArray, coords: FloatArray): IntArray =
        RenderNative.touch(handle, metadata, times, coords)

    @Synchronized
    fun popup(key: Int, labels: Array<String>) = RenderNative.popup(handle, key, labels)

    @Synchronized
    fun attach(surface: Any, hybrid: Boolean = false) = RenderNative.attach(handle, surface, hybrid)

    @Synchronized
    fun frame(time: Long) = RenderNative.frame(handle, time)

    @Synchronized
    fun detach() = RenderNative.detach(handle)

    @Synchronized
    override fun close() {
        handle = 0
        cleanable.clean()
    }

    private class Free(private val handle: Long) : Runnable {
        override fun run() = RenderNative.close(handle)
    }
}

internal object RenderNative {
    init { RustLibrary.load() }
    external fun create(fontPaths: Array<String>): Long
    external fun update(handle: Long, dimensions: FloatArray, colors: IntArray, geometry: FloatArray, labels: Array<String>, candidates: Array<String>)
    external fun touch(handle: Long, metadata: IntArray, times: LongArray, coords: FloatArray): IntArray
    external fun popup(handle: Long, key: Int, labels: Array<String>)
    external fun attach(handle: Long, surface: Any, hybrid: Boolean)
    external fun frame(handle: Long, time: Long)
    external fun detach(handle: Long)
    external fun close(handle: Long)
}
