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

package dev.patrickgold.florisboard.app.settings.theme

import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.offset
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.unit.dp
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.enumDisplayEntriesOf
import dev.patrickgold.jetpref.material.ui.JetPrefAlertDialog
import org.florisboard.lib.android.stringRes
import org.florisboard.lib.compose.FlorisChip
import org.florisboard.lib.compose.FlorisIconButton
import org.florisboard.lib.compose.stringRes
import org.florisboard.lib.snygg.SnyggAttributes
import kotlin.reflect.KClass

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun <V : Any> EnumLikeAttributeBox(
    text: String,
    enumClass: KClass<V>,
    attribute: String,
    attributes: SnyggAttributes,
    setAttributes: (SnyggAttributes) -> Unit,
    level: SnyggLevel,
) {
    val allEntries = enumDisplayEntriesOf(enumClass)
    val (alreadyAddedEntries, notYetAddedEntries) = remember(attributes, attribute) {
        allEntries.partition { entry ->
            attributes[attribute]?.contains(entry.key.toString()) == true
        }
    }
    var showAddDialog by remember { mutableStateOf(false) }

    DialogProperty(
        text = text,
        trailingIconTitle = {
            FlorisIconButton(
                onClick = { showAddDialog = true },
                modifier = Modifier.offset(x = 12.dp),
                icon = Icons.Default.Add,
            )
        },
    ) {
        FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            for (entry in alreadyAddedEntries) {
                FlorisChip(
                    onClick = {
                        setAttributes(attributes.excluding(attribute to entry.key.toString()))
                    },
                    text = entry.label,
                )
            }
            if (alreadyAddedEntries.isEmpty()) {
                Text(
                    text = stringRes(R.string.settings__theme_editor__no_codes_defined),
                    fontStyle = FontStyle.Italic,
                )
            }
        }
    }

    if (showAddDialog) {
        JetPrefAlertDialog(
            title = stringRes(R.string.action__add),
            dismissLabel = stringRes(R.string.action__cancel),
            onDismiss = { showAddDialog = false },
        ) {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                for (entry in notYetAddedEntries) {
                    FlorisChip(
                        onClick = {
                            setAttributes(attributes.including(attribute to entry.key.toString()))
                            showAddDialog = false
                        },
                        text = when (level) {
                            SnyggLevel.DEVELOPER -> entry.key.toString()
                            else -> entry.label
                        },
                    )
                }
            }
            if (notYetAddedEntries.isEmpty()) {
                Text(
                    text = stringRes(R.string.settings__theme_editor__no_enum_value_to_add_anymore),
                    fontStyle = FontStyle.Italic,
                )
            }
        }
    }
}
