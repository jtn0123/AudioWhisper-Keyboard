/*
 * Copyright (C) 2021-2025 The FlorisBoard Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package dev.patrickgold.florisboard.ime.text.keyboard

import com.jtn0123.audiowhisper.rust.KeyGeometry
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.ime.keyboard.Keyboard
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.florisboard.ime.popup.PopupMapping
import dev.patrickgold.florisboard.lib.FlorisRect

class TextKeyboard(
    val arrangement: Array<Array<TextKey>>,
    override val mode: KeyboardMode,
    val extendedPopupMapping: PopupMapping?,
    val extendedPopupMappingDefault: PopupMapping?,
) : Keyboard() {
    val rowCount: Int
        get() = arrangement.size

    val keyCount: Int
        get() = arrangement.sumOf { it.size }

    /** The keys in row order, and how many each row has: how [KeyGeometry] takes them. */
    private val flatKeys: Array<TextKey> = arrangement.flatten().toTypedArray()
    private val rowSizes = IntArray(arrangement.size) { arrangement[it].size }

    /** Each key's touch bounds as of the last [layout], four Floats a key, for [getKeyForPos]. */
    @Volatile
    private var touchBounds = FloatArray(4 * flatKeys.size)

    /** The first key whose touch bounds hold the pointer, in Rust (lib/rust/crates/keys). */
    override fun getKeyForPos(pointerX: Float, pointerY: Float): TextKey? {
        val index = KeyGeometry.keyAt(rowSizes, touchBounds, pointerX, pointerY)
        return if (index < 0) null else flatKeys[index]
    }

    /**
     * AudioWhisper: [getKeyForPos], or the neighbour [retarget] picks for this user's aim or the
     * letters likely to come next. It gets the hit key's index, this layout packed as
     * [com.jtn0123.audiowhisper.suggest.TapTrail.retarget] takes it, and the pointer, and answers
     * the index of the key that types.
     */
    fun getAimedKeyForPos(
        pointerX: Float,
        pointerY: Float,
        retarget: (Int, IntArray, FloatArray, Float, Float) -> Int,
    ): TextKey? {
        val hit = KeyGeometry.keyAt(rowSizes, touchBounds, pointerX, pointerY)
        if (hit < 0) return null
        val touch = touchBounds
        val layout = IntArray(2 + rowSizes.size + flatKeys.size)
        layout[1] = rowSizes.size
        rowSizes.copyInto(layout, 2)
        val bounds = FloatArray(6 * flatKeys.size)
        for ((i, key) in flatKeys.withIndex()) {
            layout[2 + rowSizes.size + i] = key.computedData.code
            touch.copyInto(bounds, 6 * i, 4 * i, 4 * i + 4)
            bounds[6 * i + 4] = key.visibleBounds.width
            bounds[6 * i + 5] = key.visibleBounds.height
        }
        return flatKeys.getOrNull(retarget(hit, layout, bounds, pointerX, pointerY)) ?: flatKeys[hit]
    }

    /**
     * Lays the keys out from [desiredKey] in Rust (lib/rust/crates/keys): each row's keys grow or
     * shrink by their flex factors to fill the width, the rows spread down the height, and the end
     * keys' touch bounds reach the keyboard's edges.
     */
    override fun layout(
        keyboardWidth: Float,
        keyboardHeight: Float,
        desiredKey: Key,
        extendTouchBoundariesDownwards: Boolean,
    ) {
        val flex = FloatArray(3 * flatKeys.size)
        for ((i, key) in flatKeys.withIndex()) {
            flex[3 * i] = key.flayWidthFactor
            flex[3 * i + 1] = key.flayShrink
            flex[3 * i + 2] = key.flayGrow
        }
        val desired = FloatArray(8)
        desiredKey.touchBounds.writeTo(desired, 0)
        desiredKey.visibleBounds.writeTo(desired, 4)
        val bounds = KeyGeometry.layout(
            rowSizes, flex, desired, keyboardWidth, keyboardHeight, extendTouchBoundariesDownwards,
        ) ?: return
        val touch = FloatArray(4 * flatKeys.size)
        for ((i, key) in flatKeys.withIndex()) {
            key.touchBounds.readFrom(bounds, 8 * i)
            key.visibleBounds.readFrom(bounds, 8 * i + 4)
            bounds.copyInto(touch, 4 * i, 8 * i, 8 * i + 4)
        }
        touchBounds = touch
    }

    /**
     * Sizes [desiredKey], a new key, as the one to lay this keyboard out with, in Rust
     * (lib/rust/crates/keys): a tenth of [keyboardWidth] wide and a row high, with its visible
     * bounds inside the key margins. [rowBaseHeight] and the margins are in pixels.
     */
    fun sizeDesiredKey(
        desiredKey: Key,
        keyboardWidth: Float,
        keyboardHeight: Float,
        rowBaseHeight: Float,
        keyMarginH: Float,
        keyMarginV: Float,
    ) {
        val bounds = KeyGeometry.desiredKey(
            mode.value, keyboardWidth, keyboardHeight, rowCount, rowBaseHeight, keyMarginH, keyMarginV,
        )
        desiredKey.touchBounds.readFrom(bounds, 0)
        desiredKey.visibleBounds.readFrom(bounds, 4)
    }

    private fun FlorisRect.writeTo(values: FloatArray, at: Int) {
        values[at] = left
        values[at + 1] = top
        values[at + 2] = right
        values[at + 3] = bottom
    }

    private fun FlorisRect.readFrom(values: FloatArray, at: Int) {
        left = values[at]
        top = values[at + 1]
        right = values[at + 2]
        bottom = values[at + 3]
    }

    override fun keys(): Iterator<TextKey> {
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
}
