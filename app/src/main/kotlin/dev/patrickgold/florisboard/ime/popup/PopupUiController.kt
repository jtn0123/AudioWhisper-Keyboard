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

package dev.patrickgold.florisboard.ime.popup

import android.content.Context
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.absoluteOffset
import androidx.compose.foundation.layout.requiredSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import com.jtn0123.audiowhisper.rust.PopupGrid
import dev.patrickgold.florisboard.ime.keyboard.ComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.DefaultComputingEvaluator
import dev.patrickgold.florisboard.ime.keyboard.Key
import dev.patrickgold.florisboard.ime.keyboard.KeyData
import dev.patrickgold.florisboard.ime.keyboard.computeImageVector
import dev.patrickgold.florisboard.ime.keyboard.computeLabel
import dev.patrickgold.florisboard.ime.media.emoji.EmojiSet
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.ime.text.key.KeyHintConfiguration
import dev.patrickgold.florisboard.ime.text.keyboard.TextKey
import dev.patrickgold.florisboard.ime.text.keyboard.TextKeyData
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.lib.FlorisRect
import dev.patrickgold.florisboard.lib.toIntOffset

@Composable
fun rememberPopupUiController(
    key1: Any?,
    key2: Any?,
    boundsProvider: (key: Key) -> FlorisRect,
    isSuitableForBasicPopup: (key: Key) -> Boolean,
    isSuitableForExtendedPopup: (key: Key) -> Boolean,
): PopupUiController {
    val context = LocalContext.current
    return remember(key1, key2) {
        PopupUiController(context, boundsProvider, isSuitableForBasicPopup, isSuitableForExtendedPopup)
    }
}

val ExceptionsForKeyCodes = listOf(
    KeyCode.ENTER,
    KeyCode.LANGUAGE_SWITCH,
    KeyCode.IME_UI_MODE_TEXT,
    KeyCode.IME_UI_MODE_MEDIA,
    KeyCode.IME_UI_MODE_CLIPBOARD,
    KeyCode.KANA_SWITCHER,
    KeyCode.CHAR_WIDTH_SWITCHER,
)

class PopupUiController(
    val context: Context,
    val boundsProvider: (key: Key) -> FlorisRect,
    val isSuitableForBasicPopup: (key: Key) -> Boolean,
    val isSuitableForExtendedPopup: (key: Key) -> Boolean,
) {
    private var baseRenderInfo by mutableStateOf<BaseRenderInfo?>(null)
    private var extRenderInfo by mutableStateOf<ExtRenderInfo?>(null)

    private var activeElementIndex by mutableIntStateOf(-1)
    var evaluator: ComputingEvaluator = DefaultComputingEvaluator
    var keyHintConfiguration: KeyHintConfiguration = KeyHintConfiguration.HINTS_DISABLED

    /** Is true if the preview popup is visible to the user, else false */
    val isShowingPopup: Boolean
        get() = baseRenderInfo != null
    /** Is true if the extended popup is visible to the user, else false */
    val isShowingExtendedPopup: Boolean
        get() = extRenderInfo != null

    fun isSuitableForPopups(key: Key): Boolean {
        return isSuitableForBasicPopup(key) || isSuitableForExtendedPopup(key)
    }

    /**
     * Shows a preview popup for the passed [key]. Ignores show requests for keys which
     * key code is equal to or less than [KeyCode.SPACE].
     *
     * @param key Reference to the key currently controlling the popup.
     */
    fun show(key: Key) {
        if (!isSuitableForBasicPopup(key)) return

        baseRenderInfo = BaseRenderInfo(
            key = key,
            bounds = boundsProvider(key),
            shouldIndicateExtendedPopups = when (key) {
                is TextKey -> key.computedPopups.getPopupKeys(keyHintConfiguration).isNotEmpty()
                //is EmojiKey -> key.computedPopups.getPopupKeys(keyHintConfiguration).isNotEmpty()
                else -> false
            },
        )
    }

    /**
     * Extends the currently showing key preview popup if there are popup keys defined in the
     * key data of the passed [key]. Ignores extend requests for key views which key code
     * is equal to or less than [KeyCode.SPACE]. An exception is made for the codes defined in
     * [ExceptionsForKeyCodes], as they most likely have special keys bound to them.
     *
     * Layout of the extended key popup: (n = key.computedPopups.size)
     *   when n <= 5: single line, row0 only
     *     _ _ _ _ _
     *     K K K K K
     *   when n > 5 && n % 2 == 1: multi line, row0 has 1 more key than row1, empty space position
     *     is depending on the current anchor
     *     anchorLeft           anchorRight
     *     K K ... K _         _ K ... K K
     *     K K ... K K         K K ... K K
     *   when n > 5 && n % 2 == 0: multi line, both same length
     *     K K ... K K
     *     K K ... K K
     *
     * @param key Reference to the key currently controlling the popup.
     */
    fun extend(key: Key, size: Size) {
        if (!isSuitableForExtendedPopup(key)) return

        val baseBounds = baseRenderInfo?.bounds ?: boundsProvider(key)
        val popupKeys = when (key) {
            is TextKey -> key.computedPopups.getPopupKeys(keyHintConfiguration)
            //is EmojiKey -> key.computedPopups.getPopupKeys(keyHintConfiguration)
            else -> null
        }
        // AudioWhisper: the rows, anchor, places and bounds are laid out in Rust (ExtendedPopup.kt).
        val layout = extendedPopupLayout(key.visibleBounds, baseBounds, size.width, popupKeys) ?: return

        val elements: List<MutableList<Element>> = if (layout.row1Count > 0) {
            listOf(mutableListOf(), mutableListOf())
        } else {
            listOf(mutableListOf())
        }
        for (uiIndex in 0 until layout.count) {
            val adjustedIndex = layout.popupIndex(uiIndex)
            val keyData = popupKeys?.get(adjustedIndex) ?: TextKeyData.UNSPECIFIED
            elements[layout.row(uiIndex)].add(Element(
                data = keyData,
                label = evaluator.computeLabel(keyData),
                icon = evaluator.computeImageVector(keyData),
                orderedIndex = uiIndex,
                adjustedIndex = adjustedIndex,
            ))
        }

        extRenderInfo = ExtRenderInfo(
            elements = elements,
            baseBounds = baseBounds,
            bounds = FlorisRect.new(left = layout.left, top = layout.top, right = layout.right, bottom = layout.bottom),
            anchorLeft = layout.anchorLeft,
            anchorRight = !layout.anchorLeft,
            anchorOffset = layout.anchorOffset,
            row0count = layout.row0Count,
            row1count = layout.row1Count,
        )
        activeElementIndex = layout.initialIndex
    }

    /**
     * Updates the current selected key in extended popup according to the passed [xEvent] and [yEvent].
     * This function does nothing if the extended popup is not showing and will return false.
     *
     * @param key Reference to the key currently controlling the popup.
     * @param xEvent The x coordinate of the MotionEvent.
     * @param yEvent The y coordinate of the MotionEvent.
     *
     * @return True if the pointer movement is within the elements bounds, false otherwise.
     */
    fun propagateMotionEvent(key: Key, xEvent: Float, yEvent: Float): Boolean {
        if (!isShowingExtendedPopup) {
            return false
        }

        val extRenderInfo = extRenderInfo ?: return false
        // AudioWhisper: which key the finger is on is worked out in Rust (ExtendedPopup.kt).
        val index = extendedPopupIndex(key.visibleBounds, extRenderInfo, xEvent, yEvent)
        if (index == PopupGrid.OFF_THE_POPUP) {
            return false
        }
        activeElementIndex = index.toInt()
        return true
    }

    /**
     * Gets the [TextKeyData] of the currently active key. May be either the key of the popup preview
     * or one of the keys in extended popup, if shown. Returns null if [key] is not a subclass of [TextKey].
     *
     * @param key Reference to the key currently controlling the popup.
     *
     * @return The [TextKeyData] object of the currently active key or null.
     */
    fun getActiveKeyData(key: Key): KeyData? {
        return if (key is TextKey) {
            val extRenderInfo = extRenderInfo ?: return key.computedData
            val element = extendedPopupElementOrNull(extRenderInfo.elements, activeElementIndex)
            element?.data ?: key.computedData
        } else {
            null
        }
    }

    /**
     * Gets the [EmojiSet] of the currently active key. May be either the key of the popup
     * preview or one of the keys in extended popup, if shown. Returns null if [key] is noz a subclass of [EmojiKey].
     *
     * @param key Reference to the key currently controlling the popup.
     * @return The [EmojiSet] object of the currently active key or null.
     */
    fun getActiveEmojiKeyData(key: Key): KeyData? {
        return null
        //return if (key is EmojiKey) {
        //    val extRenderInfo = extRenderInfo ?: return null
        //    val element = extendedPopupElementOrNull(extRenderInfo.elements, activeElementIndex)
        //    if (element != null) {
        //        key.computedPopups.getPopupKeys(KeyHintConfiguration.HINTS_DISABLED).getOrNull(element.adjustedIndex) ?: key.computedData
        //    } else {
        //        key.computedData
        //    }
        //} else {
        //    null
        //}
    }

    fun hide() {
        baseRenderInfo = null
        extRenderInfo = null
        activeElementIndex = -1
    }

    @Composable
    fun RenderPopups(): Unit = with(LocalDensity.current) {
        val attributes = mapOf(
            FlorisImeUi.Attr.Mode to evaluator.keyboard.mode.toString(),
            FlorisImeUi.Attr.ShiftState to evaluator.state.inputShiftState.toString(),
        )
        baseRenderInfo?.let { renderInfo ->
            PopupBaseBox(
                modifier = Modifier
                    .requiredSize(renderInfo.bounds.size.toDpSize())
                    .absoluteOffset { renderInfo.bounds.topLeft.toIntOffset() },
                attributes = attributes,
                key = renderInfo.key,
                shouldIndicateExtendedPopups = renderInfo.shouldIndicateExtendedPopups && extRenderInfo == null,
            )
        }
        extRenderInfo?.let { renderInfo ->
            val baseBounds = renderInfo.baseBounds
            val elemWidth = baseBounds.width
            val elemHeight = baseBounds.height * 0.4f
            PopupExtBox(
                modifier = Modifier
                    .requiredSize(renderInfo.bounds.size.toDpSize())
                    .absoluteOffset { renderInfo.bounds.topLeft.toIntOffset() },
                attributes = attributes,
                elements = renderInfo.elements,
                elemArrangement = if (renderInfo.anchorLeft) {
                    Arrangement.Start
                } else {
                    Arrangement.End
                },
                elemWidth = elemWidth.toDp(),
                elemHeight = elemHeight.toDp(),
                activeElementIndex = activeElementIndex,
            )
        }
    }

    data class BaseRenderInfo(
        val key: Key,
        val bounds: FlorisRect,
        val shouldIndicateExtendedPopups: Boolean,
    )

    data class ExtRenderInfo(
        val elements: List<List<Element>>,
        val baseBounds: FlorisRect,
        val bounds: FlorisRect,
        val anchorLeft: Boolean,
        val anchorRight: Boolean,
        val anchorOffset: Int,
        val row0count: Int,
        val row1count: Int,
    )

    data class Element(
        val data: KeyData,
        val label: String?,
        val icon: ImageVector?,
        val orderedIndex: Int,
        val adjustedIndex: Int,
    )
}
