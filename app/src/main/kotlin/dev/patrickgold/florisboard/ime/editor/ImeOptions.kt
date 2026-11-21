/*
 * Copyright (C) 2022-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.ime.editor

import android.view.inputmethod.EditorInfo
import com.jtn0123.audiowhisper.rust.*

/**
 * Class which holds the same information as an [EditorInfo.imeOptions] int but more accessible and
 * readable.
 *
 * @see EditorInfo.imeOptions for mask table
 */
data class ImeOptions(val raw: Int) {
    private val decoded = EditorAttributes.options(raw)
    val action: Action get() = Action.fromInt(decoded[EditorImeOptions.ACTION])
    val flagForceAscii: Boolean get() = decoded[EditorImeOptions.FORCE_ASCII] != 0
    val flagNavigateNext: Boolean get() = decoded[EditorImeOptions.NAVIGATE_NEXT] != 0
    val flagNavigatePrevious: Boolean get() = decoded[EditorImeOptions.NAVIGATE_PREVIOUS] != 0
    val flagNoAccessoryAction: Boolean get() = decoded[EditorImeOptions.NO_ACCESSORY_ACTION] != 0
    val flagNoEnterAction: Boolean get() = decoded[EditorImeOptions.NO_ENTER_ACTION] != 0
    val flagNoExtractUi: Boolean get() = decoded[EditorImeOptions.NO_EXTRACT_UI] != 0
    val flagNoFullScreen: Boolean get() = decoded[EditorImeOptions.NO_FULLSCREEN] != 0
    val flagNoPersonalizedLearning: Boolean get() = decoded[EditorImeOptions.NO_PERSONALIZED_LEARNING] != 0

    companion object {
        fun wrap(imeOptions: Int) = ImeOptions(imeOptions)
    }

    enum class Action(private val value: Int) {
        UNSPECIFIED(EditorInfo.IME_ACTION_UNSPECIFIED),
        DONE(EditorInfo.IME_ACTION_DONE),
        GO(EditorInfo.IME_ACTION_GO),
        NEXT(EditorInfo.IME_ACTION_NEXT),
        NONE(EditorInfo.IME_ACTION_NONE),
        PREVIOUS(EditorInfo.IME_ACTION_PREVIOUS),
        SEARCH(EditorInfo.IME_ACTION_SEARCH),
        SEND(EditorInfo.IME_ACTION_SEND);

        companion object {
            fun fromInt(int: Int) = entries.firstOrNull { it.value == int } ?: NONE
        }

        fun toInt() = value
    }
}
