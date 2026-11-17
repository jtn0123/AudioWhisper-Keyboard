package com.jtn0123.audiowhisper.render

/** Translate a lifted accessible hold into the same popup selection as ordinary input. */
internal fun accessibilityReleaseTarget(
    down: Int,
    hit: Int,
    held: Boolean,
    popup: Boolean,
    hitIsPopup: Boolean,
    selectedPopup: Int?,
): Int? = when {
    !held -> down.takeIf { it != -1 }
    !popup -> null
    hitIsPopup -> hit
    hit == down -> selectedPopup
    else -> null
}

/** [layer] is Rust's draw order: higher is drawn on top. Targets come in reading order. */
internal data class AccessibilityTarget(val id: Int, val bounds: IntArray, val popup: Boolean, val layer: Int)

/**
 * The target drawn on top at a point, not the last one read: a popup is read after its key but
 * drawn over the keys below it. A captured popup drag can leave the host's rectangle while
 * remaining on a popup choice.
 */
internal fun accessibilityHitTarget(
    x: Float, y: Float, width: Int, height: Int, targets: List<AccessibilityTarget>,
): Int? {
    val inside = x >= 0 && y >= 0 && x < width && y < height
    return targets.withIndex().filter { (_, target) ->
        val b = target.bounds
        (inside || target.popup) && x >= b[0] && y >= b[1] && x < b[2] && y < b[3]
    }.maxWithOrNull(compareBy({ it.value.layer }, { it.index }))?.value?.id
}
