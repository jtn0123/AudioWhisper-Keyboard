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

import android.text.InputType
import com.jtn0123.audiowhisper.rust.*
import android.view.inputmethod.EditorInfo

/**
 * Class which holds the same information as an [EditorInfo.inputType] int but more accessible and
 * readable.
 *
 * @see EditorInfo.inputType for mask table
 */
data class InputAttributes(val raw: Int) {
    private val decoded = EditorAttributes.input(raw)
    val type: Type get() = Type.fromInt(decoded[EditorInputType.TYPE])
    val variation: Variation get() = Variation.fromInt(decoded[EditorInputType.VARIATION])
    val capsMode: CapsMode get() = CapsMode.fromInt(decoded[EditorInputType.CAPS_MODE])
    val flagNumberDecimal: Boolean get() = decoded[EditorInputType.NUMBER_DECIMAL] != 0
    val flagNumberSigned: Boolean get() = decoded[EditorInputType.NUMBER_SIGNED] != 0
    val flagTextAutoComplete: Boolean get() = decoded[EditorInputType.TEXT_AUTO_COMPLETE] != 0
    val flagTextAutoCorrect: Boolean get() = decoded[EditorInputType.TEXT_AUTO_CORRECT] != 0
    val flagTextImeMultiLine: Boolean get() = decoded[EditorInputType.TEXT_IME_MULTI_LINE] != 0
    val flagTextMultiLine: Boolean get() = decoded[EditorInputType.TEXT_MULTI_LINE] != 0
    val flagTextNoSuggestions: Boolean get() = decoded[EditorInputType.TEXT_NO_SUGGESTIONS] != 0

    companion object {
        fun wrap(inputType: Int) = InputAttributes(inputType)
    }

    enum class Type(private val value: Int) {
        NULL(EditorInfo.TYPE_NULL),
        DATETIME(EditorInfo.TYPE_CLASS_DATETIME),
        NUMBER(EditorInfo.TYPE_CLASS_NUMBER),
        PHONE(EditorInfo.TYPE_CLASS_PHONE),
        TEXT(EditorInfo.TYPE_CLASS_TEXT);

        companion object {
            fun fromInt(int: Int) = entries.firstOrNull { it.value == int } ?: NULL
        }

        fun toInt() = value
    }

    enum class Variation(private val value: Int) {
        NORMAL(0),
        DATE(1),
        EMAIL_ADDRESS(2),
        EMAIL_SUBJECT(3),
        FILTER(4),
        LONG_MESSAGE(5),
        PASSWORD(6),
        PERSON_NAME(7),
        PHONETIC(8),
        POSTAL_ADDRESS(9),
        SHORT_MESSAGE(10),
        TIME(11),
        URI(12),
        VISIBLE_PASSWORD(13),
        WEB_EDIT_TEXT(14),
        WEB_EMAIL_ADDRESS(15),
        WEB_PASSWORD(16);

        companion object {
            fun fromInt(int: Int) = entries.firstOrNull { it.value == int } ?: NORMAL
        }

        fun toInt() = value
    }

    enum class CapsMode(private val value: Int) {
        NONE(0),
        ALL(1),
        WORDS(2),
        SENTENCES(3);

        companion object {
            fun fromFlags(flags: Int): CapsMode = fromInt(EditorAttributes.caps(flags))

            fun fromInt(int: Int) = entries.firstOrNull { it.value == int } ?: NONE
        }

        fun toFlags(): Int = EditorAttributes.capsFlags(value)

        fun toInt() = value
    }
}
