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

import dev.patrickgold.florisboard.ime.theme.ThemeExtensionComponentEditor
import dev.patrickgold.florisboard.lib.cache.CacheManager
import org.florisboard.lib.kotlin.io.subFile
import org.florisboard.lib.snygg.SnyggAnnotationRule
import org.florisboard.lib.snygg.SnyggJsonConfiguration
import org.florisboard.lib.snygg.SnyggSinglePropertySetEditor
import org.florisboard.lib.snygg.SnyggStylesheet
import org.florisboard.lib.snygg.SnyggStylesheetEditor

internal val PrettyPrintConfig = SnyggJsonConfiguration.of(
    prettyPrint = true,
    prettyPrintIndent = "  ",
)

internal val LenientConfig = SnyggJsonConfiguration.of(
    ignoreMissingSchema = true,
    ignoreInvalidSchema = true,
    ignoreUnsupportedSchema = true,
    ignoreInvalidRules = true,
    ignoreInvalidProperties = true,
    ignoreInvalidValues = true,
)

internal enum class StylesheetLoadingStrategy {
    TRY_LOAD_OR_ASK_ON_CONFLICT, // default state
    TRY_LOAD_OR_EMPTY, // user chose to not auto-fix errors
    TRY_LOAD_OR_PARSE_LENIENT; // user chose to auto-fix errors
}

internal fun loadThemeEditorStylesheet(
    workspace: CacheManager.ExtEditorWorkspace<*>, editor: ThemeExtensionComponentEditor,
    strategy: StylesheetLoadingStrategy,
): Pair<SnyggStylesheetEditor, Throwable?> {
    var failure: Throwable? = null
    val stylesheetPath = editor.stylesheetPath()
    editor.stylesheetPathOnLoad = stylesheetPath
    val stylesheetFile = workspace.extDir.subFile(stylesheetPath)
    val stylesheetEditor = if (stylesheetFile.exists()) {
        try {
            val stylesheetJson = stylesheetFile.readText()
            val config = when (strategy) {
                StylesheetLoadingStrategy.TRY_LOAD_OR_PARSE_LENIENT -> LenientConfig
                else -> PrettyPrintConfig
            }
            SnyggStylesheet.fromJson(stylesheetJson, config).getOrThrow().edit(CustomRuleComparator)
        } catch (error: Throwable) {
            failure = when (strategy) {
                StylesheetLoadingStrategy.TRY_LOAD_OR_ASK_ON_CONFLICT -> error
                else -> null
            }
            SnyggStylesheetEditor(SnyggStylesheet.SCHEMA_V2, comparator = CustomRuleComparator)
        }
    } else {
        SnyggStylesheetEditor(SnyggStylesheet.SCHEMA_V2, comparator = CustomRuleComparator)
    }
    stylesheetEditor.rules.putIfAbsent(SnyggAnnotationRule.Defines, SnyggSinglePropertySetEditor())
    return stylesheetEditor to failure
}
