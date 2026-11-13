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

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Edit
import androidx.compose.material3.AlertDialogDefaults
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ListItemDefaults
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.materialkolor.Contrast
import com.materialkolor.PaletteStyle
import com.materialkolor.dynamiccolor.ColorSpec
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.apptheme.Shapes
import dev.patrickgold.florisboard.ime.theme.FlorisImeUi
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponent
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import dev.patrickgold.florisboard.ime.theme.ThemeExtensionEditor
import dev.patrickgold.florisboard.lib.cache.CacheManager
import dev.patrickgold.florisboard.lib.compose.Validation
import dev.patrickgold.florisboard.lib.ext.ExtensionValidation
import dev.patrickgold.florisboard.lib.rememberValidationResult
import dev.patrickgold.jetpref.material.ui.JetPrefAlertDialog
import dev.patrickgold.jetpref.material.ui.JetPrefDropdown
import dev.patrickgold.jetpref.material.ui.JetPrefListItem
import dev.patrickgold.jetpref.material.ui.JetPrefTextField
import kotlinx.coroutines.launch
import org.florisboard.lib.android.showLongToast
import org.florisboard.lib.color.MaterialYouFlagsSaver
import org.florisboard.lib.compose.FlorisIconButton
import org.florisboard.lib.compose.florisVerticalScroll
import org.florisboard.lib.compose.stringRes
import org.florisboard.lib.snygg.SnyggElementRule
import org.florisboard.lib.snygg.SnyggRule
import org.florisboard.lib.snygg.SnyggSelector

@Composable
internal fun ComponentMetaEditorDialog(
    workspace: CacheManager.ExtEditorWorkspace<*>,
    editor: ThemeExtensionComponentEditor,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var showValidationErrors by rememberSaveable { mutableStateOf(false) }

    var id by rememberSaveable { mutableStateOf(editor.id) }
    val idValidation = rememberValidationResult(ExtensionValidation.ComponentId, id)
    var label by rememberSaveable { mutableStateOf(editor.label) }
    val labelValidation = rememberValidationResult(ExtensionValidation.ComponentLabel, label)
    var authors by rememberSaveable { mutableStateOf(editor.authors.joinToString("\n")) }
    val authorsValidation = rememberValidationResult(ExtensionValidation.ComponentAuthors, authors)
    var isNightTheme by rememberSaveable { mutableStateOf(editor.isNightTheme) }
    var materialYouFlags by rememberSaveable(stateSaver = MaterialYouFlagsSaver) { mutableStateOf(editor.materialYouFlags) }
    var stylesheetPath by rememberSaveable { mutableStateOf(editor.stylesheetPath) }
    val stylesheetPathValidation = rememberValidationResult(ExtensionValidation.ThemeComponentStylesheetPath, stylesheetPath)

    JetPrefAlertDialog(
        title = stringRes(R.string.ext__editor__metadata__title),
        confirmLabel = stringRes(R.string.action__apply),
        onConfirm = {
            val allFieldsValid = idValidation.isValid() &&
                labelValidation.isValid() &&
                authorsValidation.isValid() &&
                stylesheetPathValidation.isValid()
            if (!allFieldsValid) {
                showValidationErrors = true
            } else if (id != editor.id && (workspace.editor as? ThemeExtensionEditor)?.themes?.find { it.id == id.trim() } != null) {
                scope.launch { context.showLongToast("A theme with this ID already exists!") }
            } else {
                workspace.update {
                    editor.id = id.trim()
                    editor.label = label.trim()
                    editor.authors = authors.lines().map { it.trim() }.filter { it.isNotBlank() }
                    editor.isNightTheme = isNightTheme
                    editor.stylesheetPath = stylesheetPath.trim()
                    editor.materialYouFlags = materialYouFlags
                }
                onConfirm()
            }
        },
        dismissLabel = stringRes(R.string.action__cancel),
        onDismiss = onDismiss,
        contentPadding = PaddingValues(horizontal = 8.dp),
        scrollModifier = Modifier.florisVerticalScroll(),
    ) {
        Column {
            DialogProperty(text = stringRes(R.string.ext__meta__id)) {
                JetPrefTextField(
                    value = id,
                    onValueChange = { id = it },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Ascii),
                    singleLine = true,
                )
                Validation(showValidationErrors, idValidation)
            }
            DialogProperty(text = stringRes(R.string.ext__meta__label)) {
                JetPrefTextField(
                    value = label,
                    onValueChange = { label = it },
                    singleLine = true,
                )
                Validation(showValidationErrors, labelValidation)
            }
            DialogProperty(text = stringRes(R.string.ext__meta__authors)) {
                JetPrefTextField(
                    value = authors,
                    onValueChange = { authors = it },
                )
                Validation(showValidationErrors, authorsValidation)
            }
            JetPrefListItem(
                modifier = Modifier.toggleable(isNightTheme) { isNightTheme = it },
                text = stringRes(R.string.settings__theme_editor__component_meta_is_night_theme),
                trailing = {
                    Switch(checked = isNightTheme, onCheckedChange = null)
                },
                colors = ListItemDefaults.colors(containerColor = AlertDialogDefaults.containerColor)
            )
            DialogProperty(text = stringRes(R.string.settings__theme_editor__component_meta_stylesheet_path)) {
                JetPrefTextField(
                    value = stylesheetPath,
                    onValueChange = { stylesheetPath = it },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Ascii),
                    singleLine = true,
                    placeholderText = if (stylesheetPath.isEmpty()) {
                        ThemeExtensionComponent.defaultStylesheetPath(id.trim())
                    } else {
                        null
                    },
                )
                Validation(showValidationErrors, stylesheetPathValidation)
            }

            DialogProperty(text = stringRes(R.string.settings__theme_editor__component_meta_material_you__title)) {
                JetPrefDropdown(
                    modifier = Modifier.padding(bottom = 8.dp),
                    labelText = stringRes(R.string.settings__theme_editor__component_meta_material_you__palette_style),
                    optionsLabelProvider = {
                        it.name
                    },
                    options = PaletteStyle.entries,
                    onSelectOption = {
                        materialYouFlags = materialYouFlags.copy(paletteStyle = PaletteStyle.entries[it])
                    },
                    selectedOptionIndex = materialYouFlags.paletteStyle.ordinal
                )
                JetPrefDropdown(
                    modifier = Modifier.padding(bottom = 8.dp),
                    labelText = stringRes(R.string.settings__theme_editor__component_meta_material_you__color_contrast),
                    optionsLabelProvider = {
                        it.name
                    },
                    options = Contrast.entries,
                    onSelectOption = {
                        materialYouFlags = materialYouFlags.copy(contrastLevel = Contrast.entries[it])
                    },
                    selectedOptionIndex = materialYouFlags.contrastLevel.ordinal
                )
                JetPrefDropdown(
                    modifier = Modifier.padding(bottom = 8.dp),
                    labelText = stringRes(R.string.settings__theme_editor__component_meta_material_you__spec_version),
                    optionsLabelProvider = {
                        it.name
                    },
                    options = ColorSpec.SpecVersion.entries,
                    onSelectOption = {
                        materialYouFlags =
                            materialYouFlags.copy(specVersion = ColorSpec.SpecVersion.entries[it])
                    },
                    selectedOptionIndex = materialYouFlags.specVersion.ordinal
                )
            }
        }
    }
}

@Composable
internal fun SnyggRuleRow(
    rule: SnyggRule,
    level: SnyggLevel,
    showEditBtn: Boolean,
    onEditRuleBtnClick: () -> Unit,
    onAddPropertyBtnClick: () -> Unit,
) {
    val context = LocalContext.current

    @Composable
    fun Selector(text: String) {
        Text(
            modifier = Modifier
                .padding(end = 8.dp)
                .background(MaterialTheme.colorScheme.primaryContainer, shape = Shapes.small),
            text = text,
            style = MaterialTheme.typography.bodyMedium,
            fontFamily = FontFamily.Monospace,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }

    @Composable
    fun AttributesList(text: String, list: String) {
        Text(
            text = "$text = $list",
            style = MaterialTheme.typography.bodyMedium,
            color = LocalContentColor.current.copy(alpha = 0.56f),
            fontFamily = FontFamily.Monospace,
            overflow = TextOverflow.Ellipsis,
        )
    }

    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 6.dp),
    ) {
        Column(
            modifier = Modifier
                .weight(1f)
                .padding(vertical = 8.dp, horizontal = 10.dp),
        ) {
            if (rule is SnyggElementRule) {
                Text(
                    text = context.translateElementName(rule, level),
                    style = MaterialTheme.typography.bodyMedium,
                    fontFamily = FontFamily.Monospace,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
                Row(modifier = Modifier.fillMaxWidth()) {
                    if (rule.selector == SnyggSelector.PRESSED) {
                        Selector(
                            text = when (level) {
                                SnyggLevel.DEVELOPER -> SnyggSelector.PRESSED.id
                                else -> stringRes(R.string.snygg__rule_selector__pressed)
                            }
                        )
                    }
                    if (rule.selector == SnyggSelector.FOCUS) {
                        Selector(
                            text = when (level) {
                                SnyggLevel.DEVELOPER -> SnyggSelector.FOCUS.id
                                else -> stringRes(R.string.snygg__rule_selector__focus)
                            }
                        )
                    }
                    if (rule.selector == SnyggSelector.HOVER) {
                        Selector(
                            text = when (level) {
                                SnyggLevel.DEVELOPER -> SnyggSelector.HOVER.id
                                else -> stringRes(R.string.snygg__rule_selector__hover)
                            }
                        )
                    }
                    if (rule.selector == SnyggSelector.DISABLED) {
                        Selector(
                            text = when (level) {
                                SnyggLevel.DEVELOPER -> SnyggSelector.DISABLED.id
                                else -> stringRes(R.string.snygg__rule_selector__disabled)
                            }
                        )
                    }
                }
                for ((attrKey, attrValue) in rule.attributes) {
                    AttributesList(text = attrKey, list = attrValue.toString())
                }
            } else {
                Text(text = rule.toString())
            }
        }
        if (showEditBtn) {
            FlorisIconButton(
                onClick = onEditRuleBtnClick,
                icon = Icons.Default.Edit,
                iconColor = MaterialTheme.colorScheme.primary,
                iconModifier = Modifier.size(ButtonDefaults.IconSize),
            )
        }
        FlorisIconButton(
            onClick = onAddPropertyBtnClick,
            icon = Icons.Default.Add,
            iconColor = MaterialTheme.colorScheme.secondary,
            iconModifier = Modifier.size(ButtonDefaults.IconSize),
        )
    }
}

@Composable
internal fun DialogProperty(
    text: String,
    modifier: Modifier = Modifier,
    trailingIconTitle: @Composable () -> Unit = { },
    content: @Composable () -> Unit,
) {
    Column(modifier = modifier.padding(bottom = 8.dp)) {
        Row(verticalAlignment = Alignment.CenterVertically) {
            Text(
                modifier = Modifier
                    .weight(1f)
                    .padding(vertical = 8.dp),
                text = text,
                style = MaterialTheme.typography.titleMedium,
            )
            trailingIconTitle()
        }
        content()
    }
}

internal object CustomRuleComparator : Comparator<SnyggRule> {
    override fun compare(a: SnyggRule, b: SnyggRule): Int =
        org.florisboard.lib.snygg.SnyggStylesheetEditor.compareRules(a, b, FlorisImeUi.elementNames)
}
