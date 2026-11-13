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

internal data class AccessibilityTarget(val id: Int, val bounds: IntArray, val popup: Boolean)

/** A captured popup drag can leave the host's rectangle while remaining on a popup choice. */
internal fun accessibilityHitTarget(
    x: Float, y: Float, width: Int, height: Int, targets: List<AccessibilityTarget>,
): Int? {
    val inside = x >= 0 && y >= 0 && x < width && y < height
    return targets.asReversed().firstOrNull { target ->
        val b = target.bounds
        (inside || target.popup) && x >= b[0] && y >= b[1] && x < b[2] && y < b[3]
    }?.id
}
