package com.jtn0123.audiowhisper.rust

/** Frozen from EmojiHistoryHelper, before the stage 3B6 port. Values represent Emoji equality. */
internal fun referenceHistory(pinned: IntArray, recent: IntArray, c: IntArray): Array<IntArray> {
    val p = pinned.toMutableList(); val r = recent.toMutableList()
    val (enabled, ps, rs, pl, rl) = c.take(5)
    val op = c[5]; val value = c[6]; val offset = c[7]
    fun MutableList<Int>.addWithStrategy(strategy: Int) {
        if (strategy % 2 == 0) add(0, value) else add(value)
    }
    fun MutableList<Int>.takeWithStrategy(strategy: Int, max: Int): List<Int> {
        val n = if (max == 0) Int.MAX_VALUE else max
        return if (strategy % 2 == 0) take(n) else takeLast(n)
    }
    if (enabled != 0) {
        val pi = p.indexOf(value); val ri = r.indexOf(value)
        when (op) {
            0 -> {
                if (pi != -1) {
                    if (ps < 2) { p.removeAt(pi); p.addWithStrategy(ps) }
                } else if (ri != -1) {
                    if (rs < 2) { r.removeAt(ri); r.addWithStrategy(rs) }
                } else r.addWithStrategy(rs)
                return arrayOf(p.takeWithStrategy(ps, pl).toIntArray(), r.takeWithStrategy(rs, rl).toIntArray())
            }
            1 -> if (ri != -1) { r.removeAt(ri); p.addWithStrategy(ps) }
            2 -> if (pi != -1) { p.removeAt(pi); r.addWithStrategy(rs) }
            3 -> if (offset != 0) {
                val list = if (pi != -1) p else r
                val i = if (pi != -1) pi else ri
                if (i != -1) {
                    val j = (i + offset).coerceIn(list.indices)
                    val item = list.removeAt(i)
                    if (j == list.size) list.add(item) else list.add(j, item)
                }
            }
            4 -> if (pi != -1) p.removeAt(pi) else if (ri != -1) r.removeAt(ri)
            5 -> r.clear()
            6 -> p.clear()
        }
    }
    return arrayOf(p.toIntArray(), r.toIntArray())
}

/** Frozen marker/removal/insertion rules from QuickActionsEditorPanel. */
internal fun referenceArrange(c: IntArray, dynamic: IntArray, hidden: IntArray): Array<IntArray> {
    var sticky = c[0]
    val d = dynamic.toList().ifEmpty { listOf(-1) }.toMutableList()
    val h = hidden.toList().ifEmpty { listOf(-1) }.toMutableList()
    var dragging: Int? = null
    fun removeMarkers() {
        if (sticky == -2) sticky = -1
        d.remove(-2); if (d.isEmpty()) d.add(-1)
        h.remove(-2); if (h.isEmpty()) h.add(-1)
    }
    fun complete() {
        dragging?.let { action ->
            if (sticky == -2) sticky = action
            else if (d.indexOf(-2) >= 0) d[d.indexOf(-2)] = action
            else if (h.indexOf(-2) >= 0) h[h.indexOf(-2)] = action
        }
        dragging = null
    }
    for (op in c.drop(1).chunked(3)) {
        val group = op[1]; val index = op[2]
        val list = if (group == 1) d else h
        val target = if (group == 0 && index == 0) sticky else list.getOrNull(index)
        when (op[0]) {
            0 -> {
                val seen = mutableSetOf(sticky)
                val nd = dynamic.filter { seen.add(it) }
                val nh = hidden.filter { seen.add(it) }
                d.clear(); d.addAll(nd.ifEmpty { listOf(-1) })
                h.clear(); h.addAll(nh.ifEmpty { listOf(-1) })
            }
            1 -> if (dragging == null && target != null && target >= 0) {
                dragging = target
                if (group == 0) sticky = -2 else list[index] = -2
            }
            2 -> if (dragging != null && target != null && target != -2) {
                if (group == 0) {
                    if (target != -1) d.add(0, target)
                    removeMarkers(); sticky = -2
                } else {
                    removeMarkers()
                    if (target == -1) list[index] = -2 else list.add(index, -2)
                }
            }
            3 -> complete()
        }
    }
    complete()
    return arrayOf(intArrayOf(if (sticky >= 0) sticky else -1), d.filter { it >= 0 }.toIntArray(), h.filter { it >= 0 }.toIntArray())
}
