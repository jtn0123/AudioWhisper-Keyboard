package dev.patrickgold.florisboard.ime.popup

import com.jtn0123.audiowhisper.rust.PopupGrid
import com.jtn0123.audiowhisper.rust.PopupLayout
import com.jtn0123.audiowhisper.rust.PopupRect
import dev.patrickgold.florisboard.lib.FlorisRect

/*
 * AudioWhisper: the extended popup's arithmetic, which PopupUiController did itself, is in Rust
 * (lib/rust/crates/popup/src/grid.rs). These keep the app's classes out of lib/rust.
 */

private fun FlorisRect.toPopupRect() = PopupRect(left, top, right, bottom)

/**
 * The extended popup of [popupKeys] for the key at [keyBounds] whose preview popup is
 * [baseBounds], on a keyboard [keyboardWidth] wide; null if there are no keys.
 */
internal fun extendedPopupLayout(
    keyBounds: FlorisRect,
    baseBounds: FlorisRect,
    keyboardWidth: Float,
    popupKeys: PopupKeys<*>?,
): PopupLayout? = PopupGrid.extend(
    count = popupKeys?.size ?: 0,
    prioritized = popupKeys?.prioritized?.size ?: 0,
    key = keyBounds.toPopupRect(),
    base = baseBounds.toPopupRect(),
    keyboardWidth = keyboardWidth,
)

/**
 * The place of the popup [ext] that the finger at ([x], [y]) is on, for the key at [keyBounds];
 * [PopupGrid.OFF_THE_POPUP] once the finger has left it.
 */
internal fun extendedPopupIndex(keyBounds: FlorisRect, ext: PopupUiController.ExtRenderInfo, x: Float, y: Float): Long {
    val base = ext.baseBounds
    return PopupGrid.pointer(
        keyBounds.left, keyBounds.top, keyBounds.right, keyBounds.bottom,
        base.left, base.top, base.right, base.bottom,
        ext.anchorLeft, ext.anchorOffset, ext.row0count, ext.row1count,
        x, y,
    )
}

/** The element numbered [index] of these rows of [elements], counting from the last row back, or null. */
internal fun <T> extendedPopupElementOrNull(elements: List<List<T>>, index: Int): T? {
    val place = PopupGrid.elementAt(IntArray(elements.size) { elements[it].size }, index)
    if (place < 0) {
        return null
    }
    return elements[(place ushr 32).toInt()][place.toInt()]
}
