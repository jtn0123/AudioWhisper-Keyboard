/*
 * Copyright (C) 2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.app

import androidx.compose.runtime.Composable
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.settings.theme.DisplayKbdAfterDialogs
import dev.patrickgold.florisboard.app.settings.theme.SnyggLevel
import dev.patrickgold.florisboard.ime.core.DisplayLanguageNamesIn
import dev.patrickgold.florisboard.ime.input.InputShiftState
import dev.patrickgold.florisboard.ime.keyboard.KeyboardMode
import dev.patrickgold.jetpref.datastore.ui.ListPreferenceEntry
import dev.patrickgold.jetpref.datastore.ui.listPrefEntries
import dev.patrickgold.jetpref.material.ui.ColorRepresentation
import org.florisboard.lib.compose.stringRes
import org.florisboard.lib.kotlin.curlyFormat
import kotlin.reflect.KClass

private const val DEFAULT = ""

private val ENUM_DISPLAY_ENTRIES = mapOf<Pair<KClass<*>, String>, @Composable () -> List<ListPreferenceEntry<*>>>(
    ColorRepresentation::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = ColorRepresentation.HEX,
                label = stringRes(R.string.enum__color_representation__hex),
                description = stringRes(R.string.general__example_given).curlyFormat("example" to "#4caf50ff"),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = ColorRepresentation.RGB,
                label = stringRes(R.string.enum__color_representation__rgb),
                description = stringRes(R.string.general__example_given).curlyFormat("example" to "rgba(76, 175, 80, 1.0)"),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = ColorRepresentation.HSV,
                label = stringRes(R.string.enum__color_representation__hsv),
                description = stringRes(R.string.general__example_given).curlyFormat("example" to "hsva(122, 56, 68, 1.0)"),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    DisplayKbdAfterDialogs::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = DisplayKbdAfterDialogs.ALWAYS,
                label = stringRes(R.string.enum__display_kbd_after_dialogs__always),
                description = stringRes(R.string.enum__display_kbd_after_dialogs__always__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = DisplayKbdAfterDialogs.NEVER,
                label = stringRes(R.string.enum__display_kbd_after_dialogs__never),
                description = stringRes(R.string.enum__display_kbd_after_dialogs__never__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = DisplayKbdAfterDialogs.REMEMBER,
                label = stringRes(R.string.enum__display_kbd_after_dialogs__remember),
                description = stringRes(R.string.enum__display_kbd_after_dialogs__remember__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    DisplayLanguageNamesIn::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = DisplayLanguageNamesIn.SYSTEM_LOCALE,
                label = stringRes(R.string.enum__display_language_names_in__system_locale),
                description = stringRes(R.string.enum__display_language_names_in__system_locale__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = DisplayLanguageNamesIn.NATIVE_LOCALE,
                label = stringRes(R.string.enum__display_language_names_in__native_locale),
                description = stringRes(R.string.enum__display_language_names_in__native_locale__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
    InputShiftState::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = InputShiftState.UNSHIFTED,
                label = stringRes(R.string.enum__input_shift_state__unshifted),
            )
            entry(
                key = InputShiftState.SHIFTED_MANUAL,
                label = stringRes(R.string.enum__input_shift_state__shifted_manual),
            )
            entry(
                key = InputShiftState.SHIFTED_AUTOMATIC,
                label = stringRes(R.string.enum__input_shift_state__shifted_automatic),
            )
            entry(
                key = InputShiftState.CAPS_LOCK,
                label = stringRes(R.string.enum__input_shift_state__caps_lock),
            )
        }
    },
    KeyboardMode::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = KeyboardMode.CHARACTERS,
                label = stringRes(R.string.enum__keyboard_mode__characters),
            )
            entry(
                key = KeyboardMode.SYMBOLS,
                label = stringRes(R.string.enum__keyboard_mode__symbols),
            )
            entry(
                key = KeyboardMode.SYMBOLS2,
                label = stringRes(R.string.enum__keyboard_mode__symbols2),
            )
            entry(
                key = KeyboardMode.NUMERIC,
                label = stringRes(R.string.enum__keyboard_mode__numeric),
            )
            entry(
                key = KeyboardMode.NUMERIC_ADVANCED,
                label = stringRes(R.string.enum__keyboard_mode__numeric_advanced),
            )
            entry(
                key = KeyboardMode.PHONE,
                label = stringRes(R.string.enum__keyboard_mode__phone),
            )
            entry(
                key = KeyboardMode.PHONE2,
                label = stringRes(R.string.enum__keyboard_mode__phone2),
            )
        }
    },
    SnyggLevel::class to DEFAULT to {
        listPrefEntries {
            entry(
                key = SnyggLevel.BASIC,
                label = stringRes(R.string.enum__snygg_level__basic),
                description = stringRes(R.string.enum__snygg_level__basic__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = SnyggLevel.ADVANCED,
                label = stringRes(R.string.enum__snygg_level__advanced),
                description = stringRes(R.string.enum__snygg_level__advanced__description),
                showDescriptionOnlyIfSelected = true,
            )
            entry(
                key = SnyggLevel.DEVELOPER,
                label = stringRes(R.string.enum__snygg_level__developer),
                description = stringRes(R.string.enum__snygg_level__developer__description),
                showDescriptionOnlyIfSelected = true,
            )
        }
    },
)

@Composable
fun <V : Any> enumDisplayEntriesOf(
    enumClass: KClass<V>,
    variant: String = DEFAULT,
): List<ListPreferenceEntry<V>> {
    @Suppress("UNCHECKED_CAST")
    return ENUM_DISPLAY_ENTRIES[enumClass to variant]?.invoke()
        as List<ListPreferenceEntry<V>>
}
