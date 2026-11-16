package com.jtn0123.audiowhisper.rust

import kotlin.math.roundToInt

/** Test-only transcription of Smartbar.kt, QuickActionsRow.kt and CandidatesRow.kt.
 * Intrinsics are supplied independently, as Compose's text measurement supplies them to Row.
 * AudioWhisper's roomy row and word shrink are restated from their spec, not from the Rust.
 */
internal object ReferenceSmartbar {
    /** While words show, a shared or extended main row drops its toggle, sticky, speaker and emoji,
     * unless dictation owns the row or read-aloud does through the speaker. */
    fun roomy(c: FloatArray): Boolean {
        fun fact(at: Int) = c.getOrElse(at) { 0f } != 0f
        val mode = c[1].toInt()
        val preview = fact(13)
        val shared = fact(3) && !preview
        val showInline = fact(12) && !preview && !(mode == 2 && shared)
        return fact(0) && (mode == 2 || mode == 3) && fact(14) && !(mode == 2 && shared) && !showInline &&
            !preview && !fact(15) && !(fact(16) && fact(6))
    }
    /** A word shrinks to the whole percent that fits, never below three quarters. */
    fun textScale(scale: Float, measured: Float, available: Float): Float =
        if (measured <= available || !measured.isFinite() || !available.isFinite()) scale
        else minOf(scale, maxOf(0.75f, kotlin.math.floor(scale * maxOf(available, 0f) / measured * 100f) / 100f))
    fun layout(c: FloatArray, width: Float, words: FloatArray): FloatArray {
        if (c[0] == 0f) return FloatArray(8)
        val mode = c[1].toInt()
        val flip = c[2] != 0f
        val preview = c.getOrElse(13) { 0f } != 0f
        val inline = c.getOrElse(12) { 0f } != 0f && !preview
        val shared = c[3] != 0f && !preview
        val showInline = inline && !(mode == 2 && shared)
        val roomy = roomy(c)
        val extra = mode == 3 && c[4] != 0f
        val placement = c[5].toInt()
        val speaker = c[6] != 0f
        val sticky = c[7] != 0f
        val dynamic = c[8].toInt()
        val display = c[9].toInt()
        val dp = c[10]
        val h = c[11].roundToInt().toFloat()
        val height = if (extra) h * 2 else h
        val top = if (extra && placement != 1) h else 0f
        val actions = mutableListOf<Float>()
        val candidates = mutableListOf<Float>()
        fun action(id: Int, left: Float, y: Float = top) {
            actions.addAll(listOf(1f, id.toFloat(), left, y, left+h, y+h))
        }
        var left = 0f
        var right = width
        var visible = 0
        var content = 0f
        if (mode >= 2 && !roomy) {
            val edge = (4*dp).roundToInt().toFloat()
            val pinned = h + 2*edge
            val read = if (speaker) pinned else 0f
            if (flip) {
                if (sticky || (mode == 2 && shared)) action(if (sticky) 0 else 3, edge)
                if (speaker) action(1, pinned+edge)
                action(2, width-h-pinned+edge)
                action(4, width-h)
                left = pinned+read
                right = maxOf(left, width-h-pinned)
            } else {
                action(4, 0f)
                val end = width-pinned-read
                action(2, end-pinned+edge)
                if (speaker) action(1, end+edge)
                if (sticky || (mode == 2 && shared)) action(if (sticky) 0 else 3, width-pinned+edge)
                left = h
                right = maxOf(h, end-pinned)
            }
        }
        fun quickActions(l: Float, r: Float, y: Float) {
            val overflow = sticky || mode != 2 || !shared
            val capacity = maxOf(0, ((r-l)/h).toInt() - if (overflow) 1 else 0)
            val entries = buildList {
                if (mode == 1 && sticky) add(0)
                addAll((5 until 5+dynamic).toList())
            }.take(capacity).toMutableList()
            visible = maxOf(0, capacity - if (mode == 1 && sticky) 1 else 0)
            if (overflow) entries.add(if (flip) 0 else entries.size, 3)
            val gap = maxOf(0f, r-l-h*entries.size)/(entries.size+1)
            entries.forEachIndexed { i, id -> action(id, (l+gap+(gap+h)*i).roundToInt().toFloat(), y) }
        }
        if (showInline) {
            // InlineSuggestionsUi fills this center instead of candidates or actions-only.
        } else if (mode == 1 || (mode == 2 && shared)) {
            quickActions(left, right, top)
        } else {
            val list = if (display == 0) words.take(3) else words.toList()
            val gap = dp.roundToInt().toFloat()
            val remaining = maxOf(0f, right-left-gap*maxOf(0,list.size-1))
            val unit = if (list.isEmpty()) 0 else (remaining/list.size).roundToInt()
            var remainder = remaining.roundToInt()-unit*list.size
            var unweightedRemaining = right-left
            val widths = list.map {
                if (list.size == 1) { if (display == 0) remaining else minOf(it,remaining) }
                else if (display == 0) {
                    val correction = remainder.compareTo(0)
                    remainder -= correction
                    (unit+correction).toFloat()
                }
                else {
                    val child = minOf(it,160*dp,if (display == 1) unweightedRemaining else Float.POSITIVE_INFINITY)
                    unweightedRemaining = maxOf(0f,unweightedRemaining-child-gap)
                    child
                }
            }
            content = widths.sum()+gap*maxOf(0,list.size-1)
            if (display == 1 && list.size>1) content = minOf(content,right-left)
            var x = left + if (list.size == 1) (remaining-widths[0])/2 else 0f
            widths.forEachIndexed { id,w ->
                candidates.addAll(listOf(0f,id.toFloat(),x,top,x+w,top+h))
                x += w+gap
                if (display == 1) x = minOf(x,right)
            }
        }
        if (extra) quickActions(0f,width,if (placement == 1) h else 0f)
        val platform = if (showInline) listOf(2f,0f,left,top,right,top+h) else emptyList()
        return (listOf(height, height-if (extra && placement == 2) h else 0f, visible.toFloat(),
            left,top,right,top+h,content)+actions+candidates+platform).toFloatArray()
    }
    fun ellipsis(advances: FloatArray, width: Float): Int =
        advances.indices.lastOrNull { advances[it] <= width }?.plus(1) ?: 0
}
