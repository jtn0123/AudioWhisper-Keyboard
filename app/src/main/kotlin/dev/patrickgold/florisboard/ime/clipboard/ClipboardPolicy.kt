package dev.patrickgold.florisboard.ime.clipboard

import com.jtn0123.audiowhisper.rust.ClipboardRules
import dev.patrickgold.florisboard.ime.clipboard.provider.ClipboardItem

/** Room objects never cross JNI. Rust returns positions in this exact immutable cohort. */
internal fun clipboardPlan(
    items: List<ClipboardItem>, now: Long = System.currentTimeMillis(), enabled: Boolean = true,
    limitEnabled: Boolean = false, limit: Int = 0, old: Boolean = false, minutes: Int = 0,
    sensitive: Boolean = false, seconds: Int = 0, newItem: ClipboardItem? = null,
): Array<IntArray> = ClipboardRules.plan(
    items.flatMap { listOf(it.creationTimestampMs, it.type.value.toLong(), if (it.isPinned) 1L else 0L,
        if (it.isSensitive) 1L else 0L, if (it.text != null) 1L else 0L) }.toLongArray(),
    (items.map { it.text ?: "" } + (newItem?.text ?: "")).toTypedArray(),
    longArrayOf(now, if (enabled) 1 else 0, if (limitEnabled) 1 else 0, limit.toLong(),
        if (old) 1 else 0, minutes.toLong(), if (sensitive) 1 else 0, seconds.toLong(), if (newItem?.text != null) 1 else 0),
)
