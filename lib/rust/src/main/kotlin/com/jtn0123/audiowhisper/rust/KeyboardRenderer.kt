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
    fun tooltipShown(pointer: Int, shown: Boolean) = RenderNative.tooltipShown(handle, pointer, shown)

    @Synchronized
    fun candidateBindings(tokens: IntArray): IntArray = RenderNative.candidateBindings(handle, tokens)

    @Synchronized
    fun gestures(values: FloatArray, actions: Array<String>) = RenderNative.gestures(handle, values, actions)
    @Synchronized
    fun longPress(pointer: Int, popup: Boolean, uninterrupted: Boolean): IntArray = RenderNative.longPress(handle, pointer, popup, uninterrupted)

    @Synchronized
    fun tick(time: Long): IntArray = RenderNative.tick(handle, time)
    @Synchronized
    fun nextDeadline(): Long = RenderNative.nextDeadline(handle)
    @Synchronized
    fun barSettings(delay: Long, changed: IntArray): IntArray = RenderNative.barSettings(handle, delay, changed)

    @Synchronized
    fun candidateLongPress(pointer: Int) = RenderNative.candidateLongPress(handle, pointer)

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
    external fun tooltipShown(handle: Long, pointer: Int, shown: Boolean)
    external fun candidateBindings(handle: Long, tokens: IntArray): IntArray
    external fun gestures(handle: Long, values: FloatArray, actions: Array<String>)
    external fun longPress(handle: Long, pointer: Int, popup: Boolean, uninterrupted: Boolean): IntArray
    external fun tick(handle: Long, time: Long): IntArray
    external fun nextDeadline(handle: Long): Long
    external fun barSettings(handle: Long, delay: Long, changed: IntArray): IntArray
    external fun candidateLongPress(handle: Long, pointer: Int)
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
