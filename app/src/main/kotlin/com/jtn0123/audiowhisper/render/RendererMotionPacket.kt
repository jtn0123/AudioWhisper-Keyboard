package com.jtn0123.audiowhisper.render

import android.view.MotionEvent

/** Complete pointer/history transport, shared by keyboard and modal/side chrome. */
internal data class RendererMotionPacket(val metadata: IntArray, val times: LongArray, val coords: FloatArray)
internal fun MotionEvent.rendererPacket(): RendererMotionPacket {
    val pointers = pointerCount
    val metadata = IntArray(2 + pointers)
    metadata[0] = actionMasked
    metadata[1] = actionIndex
    for (p in 0 until pointers) metadata[2 + p] = getPointerId(p)
    val times = LongArray(historySize + 2)
    times[0] = downTime * 1_000_000
    val coords = FloatArray((historySize + 1) * pointers * 2)
    for (h in 0..historySize) {
        times[h + 1] = if (h < historySize) getHistoricalEventTimeNanos(h) else eventTimeNanos
        for (p in 0 until pointers) {
            coords[(h * pointers + p) * 2] = if (h < historySize) getHistoricalX(p, h) else getX(p)
            coords[(h * pointers + p) * 2 + 1] = if (h < historySize) getHistoricalY(p, h) else getY(p)
        }
    }
    return RendererMotionPacket(metadata, times, coords)
}
