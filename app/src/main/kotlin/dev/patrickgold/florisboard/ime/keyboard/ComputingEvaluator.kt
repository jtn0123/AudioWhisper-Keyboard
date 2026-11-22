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

package dev.patrickgold.florisboard.ime.keyboard

import com.jtn0123.audiowhisper.rust.InputProtocol.*
import android.content.Context
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowRightAlt
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.automirrored.filled.KeyboardReturn
import androidx.compose.material.icons.automirrored.filled.Redo
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.automirrored.filled.Undo
import androidx.compose.material.icons.automirrored.filled.VolumeUp
import androidx.compose.material.icons.automirrored.outlined.Assignment
import androidx.compose.material.icons.automirrored.outlined.Backspace
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.ContentCut
import androidx.compose.material.icons.filled.ContentPasteGo
import androidx.compose.material.icons.filled.DeleteSweep
import androidx.compose.material.icons.filled.Done
import androidx.compose.material.icons.filled.FontDownload
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material.icons.filled.KeyboardCapslock
import androidx.compose.material.icons.filled.KeyboardHide
import androidx.compose.material.icons.filled.KeyboardVoice
import androidx.compose.material.icons.filled.Language
import androidx.compose.material.icons.filled.MoreHoriz
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.SelectAll
import androidx.compose.material.icons.filled.SentimentSatisfiedAlt
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.SpaceBar
import androidx.compose.ui.graphics.vector.ImageVector
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.ime.core.DisplayLanguageNamesIn
import dev.patrickgold.florisboard.ime.core.Subtype
import dev.patrickgold.florisboard.ime.editor.FlorisEditorInfo
import dev.patrickgold.florisboard.ime.text.key.KeyCode
import dev.patrickgold.florisboard.lib.FlorisLocale
import dev.patrickgold.florisboard.lib.compose.vectorResource
import org.florisboard.lib.compose.icons.ForwardDelete

interface ComputingEvaluator {
    val version: Int

    val keyboard: Keyboard

    val editorInfo: FlorisEditorInfo

    val state: KeyboardState

    val subtype: Subtype

    fun context(): Context?

    fun displayLanguageNamesIn(): DisplayLanguageNamesIn

    fun evaluateEnabled(data: KeyData): Boolean

    fun evaluateVisible(data: KeyData): Boolean

    fun isSlot(data: KeyData): Boolean

    fun slotData(data: KeyData): KeyData?
}

object DefaultComputingEvaluator : ComputingEvaluator {
    override val version = -1

    override val keyboard = PlaceholderLoadingKeyboard

    override val editorInfo = FlorisEditorInfo.Unspecified

    override val state = KeyboardState.new()

    override val subtype = Subtype.DEFAULT

    override fun context(): Context? = null

    override fun displayLanguageNamesIn() = DisplayLanguageNamesIn.NATIVE_LOCALE

    override fun evaluateEnabled(data: KeyData): Boolean = true

    override fun evaluateVisible(data: KeyData): Boolean = true

    override fun isSlot(data: KeyData): Boolean = false

    override fun slotData(data: KeyData): KeyData? = null
}

private var cachedDisplayNameState = Triple(FlorisLocale.ROOT, DisplayLanguageNamesIn.SYSTEM_LOCALE, "")

/**
 * Compute language name with a cache to prevent repetitive calling of `locale.displayName()`, which invokes the
 * underlying `LocaleNative.getLanguageName()` method and in turn uses the rather slow ICU data table to look up the
 * language name. This only caches the last display name, but that's more than enough, as a one-time re-computation when
 * the subtype changes does not hurt, the repetitive computation for the same language hurts.
 */
internal fun computeLanguageDisplayName(locale: FlorisLocale, displayLanguageNamesIn: DisplayLanguageNamesIn): String {
    val (cachedLocale, cachedDisplayLanguageNamesIn, cachedDisplayName) = cachedDisplayNameState
    if (cachedLocale == locale && cachedDisplayLanguageNamesIn == displayLanguageNamesIn) {
        return cachedDisplayName
    }
    val displayName = when (displayLanguageNamesIn) {
        DisplayLanguageNamesIn.SYSTEM_LOCALE -> locale.displayName()
        DisplayLanguageNamesIn.NATIVE_LOCALE -> locale.displayName(locale)
    }
    cachedDisplayNameState = Triple(locale, displayLanguageNamesIn, displayName)
    return displayName
}

fun ComputingEvaluator.computeLabel(data: KeyData): String? {
    val token = com.jtn0123.audiowhisper.rust.InputRules.visual(longArrayOf(
        data.type.ordinal.toLong(), data.code.toLong(), keyboard.mode.toInt().toLong()))
    return when (token) {
        0 -> null
        1 -> data.asString(isForDisplay = true)
        2 -> computeLanguageDisplayName(subtype.primaryLocale, displayLanguageNamesIn())
        else -> keyLabelResource(token, context())
    }
}

/** The label resource tokens [com.jtn0123.audiowhisper.rust.InputRules.visual] names. */
val KEY_LABEL_RESOURCES = intArrayOf(KeyCode.PHONE_PAUSE, KeyCode.PHONE_WAIT, KeyCode.IME_UI_MODE_TEXT,
    KeyCode.VIEW_CHARACTERS, KeyCode.VIEW_NUMERIC, KeyCode.VIEW_NUMERIC_ADVANCED, KeyCode.VIEW_PHONE,
    KeyCode.VIEW_PHONE2, KeyCode.VIEW_SYMBOLS, KeyCode.VIEW_SYMBOLS2, KeyCode.HALF_SPACE, KeyCode.KESHIDA)

/** The label of resource [token], one of [KEY_LABEL_RESOURCES]. */
fun keyLabelResource(token: Int, context: Context?): String? = context?.getString(keyLabelResourceId(token))

/** The string resource that names resource [token], one of [KEY_LABEL_RESOURCES]. */
fun keyLabelResourceId(token: Int): Int = when (token) {
    KeyCode.PHONE_PAUSE -> R.string.key__phone_pause
    KeyCode.PHONE_WAIT -> R.string.key__phone_wait
    KeyCode.IME_UI_MODE_TEXT, KeyCode.VIEW_CHARACTERS -> R.string.key__view_characters
    KeyCode.VIEW_NUMERIC, KeyCode.VIEW_NUMERIC_ADVANCED -> R.string.key__view_numeric
    KeyCode.VIEW_PHONE -> R.string.key__view_phone
    KeyCode.VIEW_PHONE2 -> R.string.key__view_phone2
    KeyCode.VIEW_SYMBOLS -> R.string.key__view_symbols
    KeyCode.VIEW_SYMBOLS2 -> R.string.key__view_symbols2
    KeyCode.HALF_SPACE -> R.string.key__view_half_space
    KeyCode.KESHIDA -> R.string.key__view_keshida
    else -> error("Unknown native label resource")
}

fun ComputingEvaluator.computeImageVector(data: KeyData): ImageVector? = keyIconVector(
    com.jtn0123.audiowhisper.rust.InputRules.visual(longArrayOf(data.code.toLong(), state.rawValue.toLong(),
        editorInfo.inputAttributes.raw.toLong(), editorInfo.imeOptions.raw.toLong(), keyboard.mode.toInt().toLong())),
    context(),
)

/** The icon a key's icon [token] draws, from [com.jtn0123.audiowhisper.rust.InputRules.visual]. */
fun keyIconVector(icon: Int, context: Context?): ImageVector? {
    return when (icon) {
        KeyCode.ARROW_LEFT -> {
            Icons.AutoMirrored.Filled.KeyboardArrowLeft
        }
        KeyCode.ARROW_RIGHT -> {
            Icons.AutoMirrored.Filled.KeyboardArrowRight
        }
        KeyCode.ARROW_UP -> {
            Icons.Default.KeyboardArrowUp
        }
        KeyCode.ARROW_DOWN -> {
            Icons.Default.KeyboardArrowDown
        }
        KeyCode.CLIPBOARD_COPY -> {
            Icons.Default.ContentCopy
        }
        KeyCode.CLIPBOARD_CUT -> {
            Icons.Default.ContentCut
        }
        KeyCode.CLIPBOARD_PASTE -> {
            Icons.Default.ContentPasteGo
        }
        KeyCode.CLIPBOARD_SELECT_ALL -> {
            Icons.Default.SelectAll
        }
        KeyCode.CLIPBOARD_CLEAR_PRIMARY_CLIP -> {
            Icons.Default.DeleteSweep
        }
        KeyCode.COMPACT_LAYOUT_TO_LEFT,
        KeyCode.COMPACT_LAYOUT_TO_RIGHT,
        KeyCode.TOGGLE_COMPACT_LAYOUT -> {
            context?.vectorResource(id = R.drawable.ic_accessibility_one_handed)
        }
        KeyCode.VOICE_INPUT -> {
            Icons.Default.KeyboardVoice
        }
        KeyCode.READ_ALOUD -> {
            Icons.AutoMirrored.Filled.VolumeUp
        }
        KeyCode.IME_HIDE_UI -> {
            Icons.Default.KeyboardHide
        }
        KeyCode.DELETE -> {
            Icons.AutoMirrored.Outlined.Backspace
        }
        ICON_RETURN -> Icons.AutoMirrored.Filled.KeyboardReturn
        ICON_DONE -> Icons.Default.Done
        ICON_ARROW -> Icons.AutoMirrored.Filled.ArrowRightAlt
        ICON_SEARCH -> Icons.Default.Search
        ICON_SEND -> Icons.AutoMirrored.Filled.Send
        KeyCode.FORWARD_DELETE -> {
            Icons.AutoMirrored.Default.ForwardDelete
        }
        KeyCode.IME_UI_MODE_MEDIA -> {
            Icons.Default.SentimentSatisfiedAlt
        }
        KeyCode.IME_UI_MODE_CLIPBOARD -> {
            Icons.AutoMirrored.Outlined.Assignment
        }
        KeyCode.LANGUAGE_SWITCH -> {
            Icons.Default.Language
        }
        KeyCode.SETTINGS -> {
            Icons.Default.Settings
        }
        ICON_CAPS -> Icons.Default.KeyboardCapslock
        KeyCode.SHIFT -> Icons.Default.KeyboardArrowUp
        KeyCode.SPACE -> Icons.Default.SpaceBar
        KeyCode.UNDO -> {
            Icons.AutoMirrored.Filled.Undo
        }
        KeyCode.REDO -> {
            Icons.AutoMirrored.Filled.Redo
        }
        KeyCode.TOGGLE_ACTIONS_OVERFLOW -> {
            Icons.Default.MoreHoriz
        }
        KeyCode.TOGGLE_INCOGNITO_MODE -> context?.vectorResource(R.drawable.ic_incognito)
        ICON_INCOGNITO_OFF -> context?.vectorResource(R.drawable.ic_incognito_off)
        KeyCode.TOGGLE_AUTOCORRECT -> {
            Icons.Default.FontDownload
        }
        KeyCode.CHAR_WIDTH_FULL -> {
            context?.vectorResource(R.drawable.ic_keyboard_char_width_switcher_full)
        }
        KeyCode.CHAR_WIDTH_HALF -> {
            context?.vectorResource(R.drawable.ic_keyboard_char_width_switcher_half)
        }
        KeyCode.DRAG_MARKER -> {
            null
        }
        KeyCode.NOOP -> {
            Icons.Default.Close
        }
        else -> null
    }
}
