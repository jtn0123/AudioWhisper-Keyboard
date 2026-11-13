package com.jtn0123.audiowhisper.rust

/** Frozen from ClipboardManager / ClipboardHistory before the package-7 port. Tests only. */
internal fun referenceClipboard(records: LongArray, texts: Array<String>, c: LongArray): Array<IntArray> {
    val indices = records.indices.step(5).map { it / 5 }
    fun at(i: Int, field: Int) = records[i * 5 + field]
    val pinned = indices.filter { at(it, 2) != 0L }
    val unpinned = indices.filter { at(it, 2) == 0L }
    val recent = unpinned.filter { c[0] - at(it, 0) < 300_000 }
    val other = unpinned.filter { c[0] - at(it, 0) >= 300_000 }
    val nonPinned = recent + other
    val n = nonPinned.size - c[3].toInt()
    val limited = if (c[2] != 0L && n > 0) nonPinned.asReversed().filterIndexed { i, _ -> i < n } else emptyList()
    val expired = linkedSetOf<Int>()
    if (c[4] != 0L) {
        val expiry = c[0] - (c[5].toInt() * 60 * 1000)
        expired.addAll(nonPinned.filter { at(it, 0) < expiry })
    }
    if (c[6] != 0L) {
        val expiry = c[0] - (c[7].toInt() * 1000)
        expired.addAll(indices.filter { at(it, 3) != 0L && at(it, 0) < expiry })
    }
    fun text(i: Int): String? = if (at(i,4) == 0L) null else texts[i]
    val newText = if (c[8] == 0L) null else texts.last()
    val duplicate = if (c[1] != 0L) indices.firstOrNull { at(it,1) == 1L && text(it) == newText } else null
    return arrayOf(indices.sortedByDescending { at(it,0) }.toIntArray(), pinned.toIntArray(), recent.toIntArray(),
        other.toIntArray(), limited.toIntArray(), expired.toIntArray(),
        intArrayOf(duplicate ?: -1, duplicate?.let { at(it,2).toInt() } ?: 0), unpinned.toIntArray())
}

internal fun referenceMime(concrete: String, desired: String): Boolean {
    val length = desired.length
    if (length == 3 && desired == "*/*") return true
    val slash = desired.indexOf('/')
    if (slash > 0) {
        if (length == slash + 2 && desired[slash + 1] == '*') {
            if (desired.regionMatches(0, concrete, 0, slash + 1)) return true
        } else if (desired == concrete) return true
    }
    return false
}
