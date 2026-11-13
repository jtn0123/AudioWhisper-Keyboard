package com.jtn0123.audiowhisper.rust

import com.jtn0123.audiowhisper.rust.RenderProtocol.*

/** Test-only reference: 7c0b8183 BarTouch plus RendererSmartbarTouch's maps and timers.
 * Android callbacks are represented by their action traces. Hold delay is injected explicitly.
 */
internal class ReferenceRendererSmartbarTouch(private val slots: List<Slot>, private val delay: Long) {
    data class Slot(val action: Boolean, val index: Int, val left: Float, val right: Float)
    private data class Held(val slot: Slot, var deadline: Long?, var timedOut: Boolean = false)
    private val owners = sortedMapOf<Int, Held>()
    private fun hit(x: Float, y: Float) = slots.firstOrNull { y >= 0 && y < 40 && x >= it.left && x < it.right }
    fun deadline(): Long = owners.values.mapNotNull { it.deadline }.minOrNull() ?: -1
    fun tick(now: Long): IntArray = buildList {
        for ((id, h) in owners) if (h.deadline?.let { now >= it } == true) {
            h.deadline = null
            h.timedOut = true
            addAll(listOf(if (h.slot.action) TOOLTIP_SHOW else CANDIDATE_HOLD, h.slot.index, id))
        }
    }.toIntArray()
    fun invalidated(indices: IntArray): IntArray = buildList {
        for ((id, h) in owners.toMap()) if (h.slot.action && h.slot.index in indices) {
            addAll(listOf(BAR_CANCEL, h.slot.index, id)); owners.remove(id)
        }
    }.toIntArray()
    fun event(action: Int, index: Int, ids: IntArray, xy: FloatArray, time: Long): IntArray = buildList {
        fun end(id: Int, h: Held, valid: Boolean) {
            val type = if (h.slot.action) {
                if (valid && h.slot.index == BAR_TOGGLE_INDEX) BAR_TOGGLE
                else if (valid && h.timedOut) BAR_HELD_UP else if (valid) BAR_UP else BAR_CANCEL
            } else if (valid && !h.timedOut) CANDIDATE_UP else CANDIDATE_CANCEL
            addAll(listOf(type, h.slot.index, id))
        }
        fun cancelAll() { for ((id,h) in owners) end(id,h,false); owners.clear() }
        if (action == 3) { cancelAll(); return@buildList }
        if (action == 0) cancelAll()
        when (action) {
            0, 5 -> hit(xy[index*2],xy[index*2+1])?.let { slot ->
                owners[ids[index]] = Held(slot, if (slot.action && slot.index == BAR_TOGGLE_INDEX) null else time+delay)
                if (!slot.action || slot.index != BAR_TOGGLE_INDEX) addAll(listOf(if (slot.action) BAR_DOWN else CANDIDATE_DOWN,slot.index,ids[index]))
            }
            2 -> for ((p,id) in ids.withIndex()) owners[id]?.let { h ->
                if (hit(xy[p*2],xy[p*2+1]) != h.slot) { end(id,h,false); owners.remove(id) }
            }
            1, 6 -> owners.remove(ids[index])?.let { h -> end(ids[index],h,hit(xy[index*2],xy[index*2+1]) == h.slot) }
        }
        if (action == 1) cancelAll()
    }.toIntArray()
}
