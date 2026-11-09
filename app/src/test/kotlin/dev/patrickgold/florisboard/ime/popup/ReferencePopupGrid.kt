package dev.patrickgold.florisboard.ime.popup

import dev.patrickgold.florisboard.lib.FlorisRect

/**
 * The Kotlin that `PopupUiController.extend`, `propagateMotionEvent` and `getElementOrNull`
 * were before their arithmetic moved to Rust (lib/rust/crates/popup/src/grid.rs), copied from
 * PopupUiController.kt for [PopupComparison] to hold the new code up against.
 *
 * The edits: the key's `visibleBounds` comes in as [FlorisRect] `keyBounds`, the keyboard's
 * `Size` as its width, and `key is TextKey` as [PopupKeys] that aren't null; instead of building
 * elements and render info, [extend] returns what they were built from; [propagateMotionEvent]
 * returns the active index rather than setting it (null for false); and each way through notes
 * itself in [branches], so the comparison can tell the runs reached them all.
 */
object ReferencePopupGrid {
    val branches = mutableListOf<String>()

    class Ext(
        val anchorLeft: Boolean,
        val anchorRight: Boolean,
        val anchorOffset: Int,
        val row0count: Int,
        val row1count: Int,
        val initUiIndex: Int,
        val popupIndices: IntArray,
        val rows: IntArray,
        val bounds: FlorisRect,
    )

    fun extend(keyBounds: FlorisRect, baseBounds: FlorisRect, sizeWidth: Float, popupKeys: PopupKeys<*>?): Ext? {
        branches.clear()
        val keyPopupDiffX = (keyBounds.width - baseBounds.width) / 2.0f

        // Anchor left if keyView is in left half of keyboardView, else anchor right
        val anchorLeft = keyBounds.left < sizeWidth / 2
        val anchorRight = !anchorLeft

        // Determine key counts for each row
        val n = when {
            popupKeys != null -> popupKeys.size
            else -> 0
        }
        val row1count: Int
        val row0count: Int
        when {
            n <= 0 -> return null
            n <= 5 -> {
                row1count = 0
                row0count = n
            }
            n > 5 && n % 2 == 1 -> {
                row1count = (n - 1) / 2
                row0count = (n + 1) / 2
            }
            else -> {
                row1count = n / 2
                row0count = n / 2
            }
        }

        // Calculate anchor offset (always positive int, direction depends on anchorLeft and
        // anchorRight state)
        val anchorOffset = when {
            row0count <= 1 -> 0
            else -> {
                var offset = when {
                    row0count % 2 == 1 -> (row0count - 1) / 2
                    row0count % 2 == 0 -> (row0count / 2) - 1
                    else -> 0
                }
                val availableSpace = when {
                    anchorLeft -> keyBounds.left + keyPopupDiffX
                    anchorRight -> sizeWidth -
                        (keyBounds.left + keyPopupDiffX + baseBounds.width)
                    else -> 0.0f
                }
                while (offset > 0) {
                    if (availableSpace >= offset * baseBounds.width) {
                        break
                    } else {
                        offset -= 1
                        branches += "offset cut"
                    }
                }
                offset
            }
        }

        val initUiIndex = when {
            anchorLeft -> anchorOffset + row1count
            anchorRight -> row0count - 1 - anchorOffset + row1count
            else -> 0
        }
        val popupIndices: IntArray
        val uiIndices = IntRange(0, (n - 1).coerceAtLeast(0))
        if (popupKeys != null) {
            popupIndices = IntArray(n)
            when (popupKeys.prioritized.size) {
                // only one key: use initial position
                1 -> {
                    popupIndices[initUiIndex] = PopupKeys.FIRST_PRIORITIZED
                }
                // two keys: use initial position and one to the right if available, otherwise one to the left
                2 -> {
                    popupIndices[initUiIndex] = PopupKeys.FIRST_PRIORITIZED
                    when {
                        initUiIndex + 1 < n -> { popupIndices[initUiIndex + 1] = PopupKeys.SECOND_PRIORITIZED; branches += "two: right" }
                        initUiIndex - 1 >= 0 -> { popupIndices[initUiIndex - 1] = PopupKeys.SECOND_PRIORITIZED; branches += "two: left" }
                    }
                }
                // two keys: use initial position and one to either sides if available
                // otherwise two to the right or two to the left with decreasing priority
                3 -> {
                    popupIndices[initUiIndex] = PopupKeys.FIRST_PRIORITIZED
                    when {
                        initUiIndex + 1 < n && initUiIndex - 1 >= 0 -> {
                            popupIndices[initUiIndex + 1] = PopupKeys.SECOND_PRIORITIZED
                            popupIndices[initUiIndex - 1] = PopupKeys.THIRD_PRIORITIZED
                            branches += "three: sides"
                        }
                        initUiIndex + 2 < n -> {
                            popupIndices[initUiIndex + 1] = PopupKeys.SECOND_PRIORITIZED
                            popupIndices[initUiIndex + 2] = PopupKeys.THIRD_PRIORITIZED
                            branches += "three: right"
                        }
                        initUiIndex - 2 >= 0 -> {
                            popupIndices[initUiIndex - 1] = PopupKeys.SECOND_PRIORITIZED
                            popupIndices[initUiIndex - 2] = PopupKeys.THIRD_PRIORITIZED
                            branches += "three: left"
                        }
                    }
                }
            }
            var offset = 0
            for (uiIndex in uiIndices) {
                if (popupIndices[uiIndex] < 0) {
                    offset++
                } else {
                    popupIndices[uiIndex] = uiIndex - offset
                }
            }
        } else {
            popupIndices = IntArray(n) { it }
        }

        val rows = IntArray(n)
        for (uiIndex in uiIndices) {
            val rowIndex = if (uiIndex < row1count && row1count > 0) { 1 } else { 0 }
            rows[uiIndex] = rowIndex
        }

        // Calculate layout params
        val extWidth = row0count * baseBounds.width
        val extHeight = when {
            row1count > 0 -> baseBounds.height * 0.4f * 2.0f
            else -> baseBounds.height * 0.4f
        }
        val x = ((keyBounds.width - baseBounds.width) / 2.0f) + when {
            anchorLeft -> -anchorOffset * baseBounds.width
            anchorRight -> -extWidth + baseBounds.width + anchorOffset * baseBounds.width
            else -> 0.0f
        } + keyBounds.left
        val y = -baseBounds.height - when {
            row1count > 0 -> (baseBounds.height * 0.4f).toInt()
            else -> 0
        } + keyBounds.bottom
        val extBounds = FlorisRect.new(
            left = x, top = y, right = x + extWidth, bottom = y + extHeight,
        )

        return Ext(anchorLeft, anchorRight, anchorOffset, row0count, row1count, initUiIndex, popupIndices, rows, extBounds)
    }

    fun propagateMotionEvent(keyBounds: FlorisRect, extRenderInfo: PopupUiController.ExtRenderInfo, xEvent: Float, yEvent: Float): Int? {
        branches.clear()
        val baseBounds = extRenderInfo.baseBounds
        val keyPopupDiffX = (keyBounds.width - baseBounds.width) / 2.0f

        val x = xEvent - keyBounds.left
        val y = yEvent - keyBounds.top
        val kX = x / baseBounds.width

        // Check if out of boundary on y-axis
        if (y < -baseBounds.height || y > 0.9f * baseBounds.height) {
            branches += "off: y"
            return null
        }

        extRenderInfo.apply {
            return when {
                anchorLeft -> when {
                    // check if out of boundary on x-axis
                    x < keyPopupDiffX - (anchorOffset + 1) * baseBounds.width ||
                        x > (keyPopupDiffX + (row0count + 1 - anchorOffset) * baseBounds.width) -> {
                        branches += "off: left x"
                        return null
                    }
                    // row 1
                    y < 0 && row1count > 0 -> when {
                        kX >= row1count - anchorOffset -> { branches += "left 1 end"; row1count - 1 }
                        kX < -anchorOffset -> { branches += "left 1 start"; 0 }
                        kX < 0 -> { branches += "left 1 before"; kX.toInt() - 1 + anchorOffset }
                        else -> { branches += "left 1 after"; kX.toInt() + anchorOffset }
                    }
                    // row 0
                    else -> when {
                        kX >= row0count - anchorOffset -> { branches += "left 0 end"; row1count + row0count - 1 }
                        kX < -anchorOffset -> { branches += "left 0 start"; row1count }
                        kX < 0 -> { branches += "left 0 before"; row1count + kX.toInt() - 1 + anchorOffset }
                        else -> { branches += "left 0 after"; row1count + kX.toInt() + anchorOffset }
                    }
                }
                anchorRight -> when {
                    // check if out of boundary on x-axis
                    x > keyBounds.width - keyPopupDiffX + (anchorOffset + 1) * baseBounds.width ||
                        x < (keyBounds.width - keyPopupDiffX - (row0count + 1 - anchorOffset) * baseBounds.width) -> {
                        branches += "off: right x"
                        return null
                    }
                    // row 1
                    y < 0 && row1count > 0 -> when {
                        kX >= anchorOffset -> { branches += "right 1 end"; row1count - 1 }
                        kX < -(row1count - 1 - anchorOffset) -> { branches += "right 1 start"; 0 }
                        kX < 0 -> { branches += "right 1 before"; row1count - 2 + kX.toInt() - anchorOffset }
                        else -> { branches += "right 1 after"; row1count - 1 + kX.toInt() - anchorOffset }
                    }
                    // row 0
                    else -> when {
                        kX >= anchorOffset -> { branches += "right 0 end"; row1count + row0count - 1 }
                        kX < -(row0count - 1 - anchorOffset) -> { branches += "right 0 start"; row1count }
                        kX < 0 -> { branches += "right 0 before"; row1count + row0count - 2 + kX.toInt() - anchorOffset }
                        else -> { branches += "right 0 after"; row1count + row0count - 1 + kX.toInt() - anchorOffset }
                    }
                }
                else -> -1
            }
        }
    }

    fun <T> getElementOrNull(elements: List<List<T>>, index: Int): T? {
        if (index < 0) {
            return null
        }
        var cachedIndex = index
        elements.asReversed().forEach { row ->
            if (cachedIndex >= row.size) {
                cachedIndex -= row.size
            } else {
                return row[cachedIndex]
            }
        }
        return null
    }
}
