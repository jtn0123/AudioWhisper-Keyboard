package com.jtn0123.audiowhisper.reader

import com.jtn0123.audiowhisper.rust.ReaderTree

/** dc7256e5 ScreenWalk.visit/walk, with Android node access replaced by the same plain facts. */
internal object ReferenceScreenWalk {
    fun walk(tree: ReaderTree, visible: IntArray): IntArray {
        data class Piece(val id: Int, val top: Int, val bottom: Int, val scroller: Int)
        data class Scroller(val id: Int, val parent: Int, val area: Long)
        val pieces = arrayListOf<Piece>()
        val scrollers = arrayListOf<Scroller>()
        var nodes = 0
        var web = false
        fun visit(id: Int, scroller: Int, sideways: Boolean, depth: Int) {
            if (nodes >= 20_000 || depth > 250) return
            nodes++
            val f = tree.facts(id)
            if (f[4] != 0 || f[5] != 0) return
            if (sideways && f[6] == 0) return
            if (!web && tree.className(id) == "android.webkit.WebView") web = true
            var inner = scroller
            var across = sideways
            if (f[7] != 0) {
                if (ReferenceScreen.isSideways(f[8], f[9], f[10] != 0, f[11] != 0, tree.className(id))) across = true
                else {
                    val box = Box(f[0], f[1], f[2], f[3])
                    scrollers += Scroller(id, scroller, ReferenceScreen.visibleArea(box, Box(visible[0], visible[1], visible[2], visible[3])))
                    inner = scrollers.lastIndex
                }
            }
            val before = pieces.size
            for (index in 0 until f[12]) {
                val child = tree.child(id, index)
                if (child >= 0) visit(child, inner, across, depth + 1)
            }
            if (pieces.size == before && tree.text(id).isNotBlank()) pieces += Piece(id, f[1], f[3], scroller)
        }
        visit(0, -1, false, 0)
        val of = pieces.map { it.scroller }.toIntArray()
        val main = ReferenceScreen.mainScroller(of, scrollers.map { it.area }.toLongArray())
        val parents = scrollers.map { it.parent }.toIntArray()
        val inList = BooleanArray(of.size) { ReferenceScreen.isWithin(parents, of[it], main) }
        return intArrayOf(scrollers.getOrNull(main)?.id ?: -1, nodes, if (web) 1 else 0) +
            pieces.flatMapIndexed { index, p -> listOf(p.id, p.top, p.bottom, if (inList[index]) 1 else 0) }.toIntArray()
    }
}
