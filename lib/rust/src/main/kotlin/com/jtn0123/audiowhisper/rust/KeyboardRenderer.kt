package com.jtn0123.audiowhisper.rust

/** Plain scene and MotionEvent packets. This wrapper has no Android dependency, including in JVM tests. */
class KeyboardRenderer(fontPaths: Array<String>) : AutoCloseable {
    private var handle = RenderNative.create(fontPaths)
    private val cleanable = RustLibrary.cleaner.register(this, Free(handle))

    @Synchronized
    fun theme(json: String, names: Array<String>, colors: IntArray, scale: FloatArray) =
        RenderNative.theme(handle, json, names, colors, scale)

    @Synchronized
    fun update(dimensions: FloatArray, metadata: Array<String>, candidateFlags: IntArray, geometry: FloatArray, labels: Array<String>, candidates: Array<String>) =
        RenderNative.update(handle, dimensions, metadata, candidateFlags, geometry, labels, candidates)

    @Synchronized
    fun smartbar(config: FloatArray, codes: IntArray, labels: Array<String>) = RenderNative.smartbar(handle, config, codes, labels)

    @Synchronized
    fun touch(metadata: IntArray, times: LongArray, coords: FloatArray): IntArray =
        RenderNative.touch(handle, metadata, times, coords)

    @Synchronized
    fun popup(key: Int, labels: Array<String>, codes: IntArray) = RenderNative.popup(handle, key, labels, codes)

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
    external fun theme(handle: Long, json: String, names: Array<String>, colors: IntArray, scale: FloatArray)
    external fun update(handle: Long, dimensions: FloatArray, metadata: Array<String>, candidateFlags: IntArray, geometry: FloatArray, labels: Array<String>, candidates: Array<String>)
    external fun smartbar(handle: Long, config: FloatArray, codes: IntArray, labels: Array<String>)
    external fun touch(handle: Long, metadata: IntArray, times: LongArray, coords: FloatArray): IntArray
    external fun popup(handle: Long, key: Int, labels: Array<String>, codes: IntArray)
    external fun attach(handle: Long, surface: Any, hybrid: Boolean)
    external fun frame(handle: Long, time: Long)
    external fun detach(handle: Long)
    external fun close(handle: Long)
}

/** Compose-compatible measurement; values are physical pixels, ids are platform action handles. */
object SmartbarRules {
    init { RustLibrary.load() }
    external fun layout(config: FloatArray, width: Float, intrinsic: FloatArray): FloatArray
    external fun ellipsis(advances: FloatArray, width: Float): Int
}
