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

// What outlived the Compose theme editor (compose-removal step 9): the preference model's types,
// and the configuration the extension editor and the native theme editor's glue write
// stylesheets with. Moved here unchanged from dev.patrickgold.florisboard.app.settings.theme.

import androidx.compose.ui.graphics.Color
import dev.patrickgold.jetpref.datastore.model.PreferenceSerializer
import org.florisboard.lib.snygg.SnyggJsonConfiguration

internal val PrettyPrintConfig = SnyggJsonConfiguration.of(
    prettyPrint = true,
    prettyPrintIndent = "  ",
)

object ColorPreferenceSerializer : PreferenceSerializer<Color> {
    override fun deserialize(value: String): Color {
        return Color(com.jtn0123.audiowhisper.rust.SettingsRules.color(value).toULong(16))
    }

    override fun serialize(value: Color): String = com.jtn0123.audiowhisper.rust.SettingsRules.color(value.value.toString(16))
}

/**
 * DisplayPreviewAfterDialogs indicates if the keyboard should auto-open after closing
 * any dialog. This is useful because the dialog always hides the keyboard and one may
 * not want to always press the preview field again.
 */
enum class DisplayKbdAfterDialogs {
    ALWAYS,
    NEVER,
    REMEMBER;
}

/**
 * SnyggLevel indicates if a rule property is intended to be edited by all users (BASIC) or only by advanced users
 * (ADVANCED). This level is intended for theme editor UIs to hide certain properties in a "basic" mode, for the Snygg
 * theme engine internally this level will be ignored completely.
 */
enum class SnyggLevel : Comparable<SnyggLevel> {
    /** A property is intended to be edited by all users **/
    BASIC,
    /** A property is intended to be edited by advanced users **/
    ADVANCED,
    /** A property is intended to be edited by developers **/
    DEVELOPER;
}
