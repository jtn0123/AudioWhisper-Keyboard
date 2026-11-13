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
    fun decorations(labels: Array<String>, height: Float, flags: IntArray) = RenderNative.decorations(handle, labels, height, flags)

    @Synchronized
    fun chrome(config: FloatArray, labels: Array<String>, icons: Array<String>, metadata: Array<String> = arrayOf("characters", "unshifted")) = RenderNative.chrome(handle, config, labels, icons, metadata)
    @Synchronized
    fun keyPreview(code: Int, labels: Array<String>): IntArray = RenderNative.keyPreview(handle, code, labels)
    @Synchronized
    fun chromeInfo(): FloatArray = RenderNative.chromeInfo(handle)
    @Synchronized
    fun pixels(): IntArray = RenderNative.pixels(handle)

    @Synchronized
    fun subtypes(selected: Long, ids: LongArray, labels: Array<String>) = RenderNative.subtypes(handle, selected, ids, labels)

    @Synchronized
    fun search(density: Float, labels: Array<String>, now: Long) = RenderNative.search(handle,density,labels,now)
    @Synchronized
    fun gifSent() = RenderNative.gifSent(handle)
    @Synchronized
    fun gifs(config: FloatArray, labels: Array<String>, chips: Int, ids: LongArray) = RenderNative.gifs(handle,config,labels,chips,ids)
    @Synchronized
    fun mediaRepeat(id: Long, count: Int) = RenderNative.mediaRepeat(handle,id,count)
    @Synchronized
    fun mediaRequests(): LongArray = RenderNative.mediaRequests(handle)
    @Synchronized
    fun mediaUpload(id: Long, index: Int, width: Int, height: Int, delay: Long, last: Boolean, pixels: IntArray): Boolean =
        RenderNative.mediaUpload(handle,id,index,width,height,delay,last,pixels)
    @Synchronized
    fun clipboard(config: LongArray, records: LongArray, labels: Array<String>) = RenderNative.clipboard(handle,config,records,labels)

    @Synchronized
    fun emoji(config: IntArray, groups: IntArray, values: Array<String>, history: IntArray, labels: Array<String>) = RenderNative.emoji(handle, config, groups, values, history, labels)
    @Synchronized
    fun panels(config: FloatArray, data: IntArray, labels: Array<String>) = RenderNative.panels(handle, config, data, labels)
    @Synchronized
    fun panelArrangement(): Array<IntArray> = RenderNative.panelArrangement(handle).map { it as IntArray }.toTypedArray()

    @Synchronized
    fun smartbar(config: FloatArray, codes: IntArray, labels: Array<String>) = RenderNative.smartbar(handle, config, codes, labels)

    @Synchronized
    fun semanticConfig(names: Array<String>, values: Array<String>, text: Array<String>, popups: IntArray, removable: IntArray, checked: IntArray, editor: IntArray) =
        RenderNative.semanticConfig(handle, names, values, text, popups, removable, checked, editor)
    @Synchronized
    fun semanticSnapshot(): String = RenderNative.semanticSnapshot(handle)
    @Synchronized
    fun semanticAction(id: Int, action: Int): IntArray = RenderNative.semanticAction(handle, id, action)
    @Synchronized
    fun semanticCancel(): IntArray = RenderNative.semanticCancel(handle)

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
    fun popupGrid(prioritized: Int, landscape: Boolean) = RenderNative.popupGrid(handle, prioritized, landscape)

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
    external fun decorations(handle: Long, labels: Array<String>, height: Float, flags: IntArray)
    external fun chrome(handle: Long, config: FloatArray, labels: Array<String>, icons: Array<String>, metadata: Array<String>)
    external fun keyPreview(handle: Long, code: Int, labels: Array<String>): IntArray
    external fun chromeInfo(handle: Long): FloatArray
    external fun pixels(handle: Long): IntArray
    external fun subtypes(handle: Long, selected: Long, ids: LongArray, labels: Array<String>)
    external fun search(handle: Long, density: Float, labels: Array<String>, now: Long)
    external fun gifSent(handle: Long)
    external fun gifs(handle: Long, config: FloatArray, labels: Array<String>, chips: Int, ids: LongArray)
    external fun mediaRepeat(handle: Long, id: Long, count: Int)
    external fun mediaRequests(handle: Long): LongArray
    external fun mediaUpload(handle: Long, id: Long, index: Int, width: Int, height: Int, delay: Long, last: Boolean, pixels: IntArray): Boolean
    external fun clipboard(handle: Long, config: LongArray, records: LongArray, labels: Array<String>)
    external fun emoji(handle: Long, config: IntArray, groups: IntArray, values: Array<String>, history: IntArray, labels: Array<String>)
    external fun panels(handle: Long, config: FloatArray, data: IntArray, labels: Array<String>)
    external fun panelArrangement(handle: Long): Array<Any>
    external fun smartbar(handle: Long, config: FloatArray, codes: IntArray, labels: Array<String>)
    external fun semanticConfig(handle: Long, names: Array<String>, values: Array<String>, text: Array<String>, popups: IntArray, removable: IntArray, checked: IntArray, editor: IntArray)
    external fun semanticSnapshot(handle: Long): String
    external fun semanticAction(handle: Long, id: Int, action: Int): IntArray
    external fun semanticCancel(handle: Long): IntArray
    external fun touch(handle: Long, metadata: IntArray, times: LongArray, coords: FloatArray): IntArray
    external fun tooltipShown(handle: Long, pointer: Int, shown: Boolean)
    external fun candidateBindings(handle: Long, tokens: IntArray): IntArray
    external fun gestures(handle: Long, values: FloatArray, actions: Array<String>)
    external fun longPress(handle: Long, pointer: Int, popup: Boolean, uninterrupted: Boolean): IntArray
    external fun tick(handle: Long, time: Long): IntArray
    external fun nextDeadline(handle: Long): Long
    external fun barSettings(handle: Long, delay: Long, changed: IntArray): IntArray
    external fun candidateLongPress(handle: Long, pointer: Int)
    external fun popupGrid(handle: Long, prioritized: Int, landscape: Boolean)
    external fun popup(handle: Long, key: Int, labels: Array<String>, codes: IntArray)
    external fun attach(handle: Long, surface: Any, hybrid: Boolean)
    external fun frame(handle: Long, time: Long): Boolean
    external fun detach(handle: Long)
    external fun close(handle: Long)
}

/** Compose-compatible measurement; values are physical pixels, ids are platform action handles. */
object SmartbarRules {
    init { RustLibrary.load() }
    external fun layout(config: FloatArray, width: Float, intrinsic: FloatArray): FloatArray
    external fun ellipsis(advances: FloatArray, width: Float): Int
}
