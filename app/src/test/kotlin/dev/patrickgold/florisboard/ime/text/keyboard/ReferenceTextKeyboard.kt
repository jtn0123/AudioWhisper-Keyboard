package dev.patrickgold.florisboard.ime.text.keyboard

import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyType
import kotlin.math.abs

/**
 * The Kotlin `TextKeyboard.layout` and `getKeyForPos` replaced by lib/rust/crates/keys, kept to test
 * against: verbatim, over an arrangement of keys alone.
 */
internal class ReferenceTextKeyboard(val arrangement: Array<Array<TextKey>>) {
    val rowCount: Int
        get() = arrangement.size

    fun getKeyForPos(pointerX: Float, pointerY: Float): TextKey? {
        for (key in keys()) {
            if (key.touchBounds.contains(pointerX, pointerY)) {
                return key
            }
        }
        return null
    }

    fun layout(
        keyboardWidth: Float,
        keyboardHeight: Float,
        desiredKey: Key,
        extendTouchBoundariesDownwards: Boolean,
    ) {
        if (arrangement.isEmpty()) return

        val desiredTouchBounds = desiredKey.touchBounds
        val desiredVisibleBounds = desiredKey.visibleBounds
        if (desiredTouchBounds.isEmpty() || desiredVisibleBounds.isEmpty()) return
        if (keyboardWidth.isNaN() || keyboardHeight.isNaN()) return
        val rowMarginH = abs(desiredTouchBounds.width - desiredVisibleBounds.width)
        val rowMarginV = (keyboardHeight - desiredTouchBounds.height * rowCount.toFloat()) / (rowCount - 1).coerceAtLeast(1).toFloat()

        for ((r, row) in rows().withIndex()) {
            val posY = (desiredTouchBounds.height + rowMarginV) * r
            val availableWidth = (keyboardWidth - rowMarginH) / desiredTouchBounds.width
            var requestedWidth = 0.0f
            var shrinkSum = 0.0f
            var growSum = 0.0f
            for (key in row) {
                requestedWidth += key.flayWidthFactor
                shrinkSum += key.flayShrink
                growSum += key.flayGrow
            }
            if (requestedWidth <= availableWidth) {
                // Requested with is smaller or equal to the available with, so we can grow
                val additionalWidth = availableWidth - requestedWidth
                var posX = rowMarginH / 2.0f
                for ((k, key) in row.withIndex()) {
                    val keyWidth = desiredTouchBounds.width * when (growSum) {
                        0.0f -> when (k) {
                            0, row.size - 1 -> key.flayWidthFactor + additionalWidth / 2.0f
                            else -> key.flayWidthFactor
                        }
                        else -> key.flayWidthFactor + additionalWidth * (key.flayGrow / growSum)
                    }
                    key.touchBounds.apply {
                        left = posX
                        top = posY
                        right = posX + keyWidth
                        bottom = posY + desiredTouchBounds.height
                    }
                    key.visibleBounds.apply {
                        left = key.touchBounds.left + abs(desiredTouchBounds.left - desiredVisibleBounds.left) + when {
                            growSum == 0.0f && k == 0 -> ((additionalWidth / 2.0f) * desiredTouchBounds.width)
                            else -> 0.0f
                        }
                        top = key.touchBounds.top + abs(desiredTouchBounds.top - desiredVisibleBounds.top)
                        right = key.touchBounds.right - abs(desiredTouchBounds.right - desiredVisibleBounds.right) - when {
                            growSum == 0.0f && k == row.size - 1 -> ((additionalWidth / 2.0f) * desiredTouchBounds.width)
                            else -> 0.0f
                        }
                        bottom = key.touchBounds.bottom - abs(desiredTouchBounds.bottom - desiredVisibleBounds.bottom)
                    }
                    posX += keyWidth
                    // After-adjust touch bounds for the row margin
                    key.touchBounds.apply {
                        if (k == 0) {
                            left = 0.0f
                        } else if (k == row.size - 1) {
                            right = keyboardWidth
                        }
                        if (extendTouchBoundariesDownwards && r + 1 == arrangement.size) {
                            bottom += height
                        }
                    }
                }
            } else {
                // Requested size too big, must shrink.
                val clippingWidth = requestedWidth - availableWidth
                var posX = rowMarginH / 2.0f
                for ((k, key) in row.withIndex()) {
                    val keyWidth = desiredTouchBounds.width * if (key.flayShrink == 0.0f) {
                        key.flayWidthFactor
                    } else {
                        key.flayWidthFactor - clippingWidth * (key.flayShrink / shrinkSum)
                    }
                    key.touchBounds.apply {
                        left = posX
                        top = posY
                        right = posX + keyWidth
                        bottom = posY + desiredTouchBounds.height
                    }
                    key.visibleBounds.apply {
                        left = key.touchBounds.left + abs(desiredTouchBounds.left - desiredVisibleBounds.left)
                        top = key.touchBounds.top + abs(desiredTouchBounds.top - desiredVisibleBounds.top)
                        right = key.touchBounds.right - abs(desiredTouchBounds.right - desiredVisibleBounds.right)
                        bottom = key.touchBounds.bottom - abs(desiredTouchBounds.bottom - desiredVisibleBounds.bottom)
                    }
                    posX += keyWidth
                    // After-adjust touch bounds for the row margin
                    key.touchBounds.apply {
                        if (k == 0) {
                            left = 0.0f
                        } else if (k == row.size - 1) {
                            right = keyboardWidth
                        }
                        if (extendTouchBoundariesDownwards && r + 1 == arrangement.size) {
                            bottom += height
                        }
                    }
                }
            }
        }
    }

    fun keys(): Iterator<TextKey> {
        return TextKeyboardIterator(arrangement)
    }

    fun rows(): Iterator<Array<TextKey>> {
        return arrangement.iterator()
    }

    class TextKeyboardIterator internal constructor(
        private val arrangement: Array<Array<TextKey>>
    ) : Iterator<TextKey> {
        private var rowIndex: Int = 0
        private var keyIndex: Int = 0

        override fun hasNext(): Boolean {
            return rowIndex < arrangement.size && keyIndex < arrangement[rowIndex].size
        }

        override fun next(): TextKey {
            val next = arrangement[rowIndex][keyIndex]
            if (keyIndex + 1 == arrangement[rowIndex].size) {
                rowIndex++
                keyIndex = 0
            } else {
                keyIndex++
            }
            return next
        }
    }

    companion object {
        /** `TextKey.compute`'s flex factors for a visible key, verbatim: shrink, grow, width factor. */
        fun flex(keyboardMode: KeyboardMode, code: Int, type: KeyType): Triple<Float, Float, Float> {
            val flayShrink = when (keyboardMode) {
                KeyboardMode.NUMERIC,
                KeyboardMode.NUMERIC_ADVANCED,
                KeyboardMode.PHONE,
                KeyboardMode.PHONE2 -> 1.0f
                else -> when (code) {
                    KeyCode.SHIFT,
                    KeyCode.DELETE -> 1.5f
                    KeyCode.VIEW_CHARACTERS,
                    KeyCode.VIEW_SYMBOLS,
                    KeyCode.VIEW_SYMBOLS2,
                    KeyCode.ENTER -> 0.0f
                    else -> 1.0f
                }
            }
            val flayGrow = when (keyboardMode) {
                KeyboardMode.NUMERIC,
                KeyboardMode.PHONE,
                KeyboardMode.PHONE2 -> 0.0f
                KeyboardMode.NUMERIC_ADVANCED -> when (type) {
                    KeyType.NUMERIC -> 1.0f
                    else -> 0.0f
                }
                else -> when (code) {
                    KeyCode.SPACE, KeyCode.CJK_SPACE -> 1.0f
                    else -> 0.0f
                }
            }
            val flayWidthFactor = when (keyboardMode) {
                KeyboardMode.NUMERIC,
                KeyboardMode.PHONE,
                KeyboardMode.PHONE2 -> 2.68f
                KeyboardMode.NUMERIC_ADVANCED -> when (code) {
                    44, 46 -> 1.00f
                    KeyCode.VIEW_SYMBOLS, 61 -> 1.26f
                    else -> 1.56f
                }
                else -> when (code) {
                    KeyCode.SHIFT,
                    KeyCode.DELETE -> 1.56f
                    KeyCode.VIEW_CHARACTERS,
                    KeyCode.VIEW_SYMBOLS,
                    KeyCode.VIEW_SYMBOLS2,
                    KeyCode.ENTER -> 1.56f
                    else -> 1.00f
                }
            }
            return Triple(flayShrink, flayGrow, flayWidthFactor)
        }

        /**
         * `TextKeyboardLayout`'s desired key, verbatim but for its inputs: [rowBaseHeight] is
         * `keyboardRowBaseHeight.toPx()`, and the keyboard's mode and row count are passed in.
         */
        fun desiredKey(
            mode: KeyboardMode,
            rowCount: Int,
            keyboardWidth: Float,
            keyboardHeight: Float,
            rowBaseHeight: Float,
            keyMarginH: Float,
            keyMarginV: Float,
        ): TextKey = TextKey(data = TextKeyData.UNSPECIFIED).also { desiredKey ->
            desiredKey.touchBounds.apply {
                width = keyboardWidth / 10f
                height = when (mode) {
                    KeyboardMode.CHARACTERS,
                    KeyboardMode.NUMERIC_ADVANCED,
                    KeyboardMode.SYMBOLS,
                    KeyboardMode.SYMBOLS2 -> {
                        (keyboardHeight / rowCount)
                            .coerceAtMost(rowBaseHeight * 1.12f)
                    }
                    else -> rowBaseHeight
                }
            }
            desiredKey.visibleBounds.applyFrom(desiredKey.touchBounds).deflateBy(keyMarginH, keyMarginV)
        }
    }
}
