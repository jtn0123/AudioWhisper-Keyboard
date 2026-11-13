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

package dev.patrickgold.florisboard.lib.devtools

import android.content.Context
import android.os.Build
import dev.patrickgold.florisboard.BuildConfig
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.florisboard.extensionManager
import dev.patrickgold.florisboard.lib.titlecase
import dev.patrickgold.florisboard.lib.util.TimeUtils
import dev.patrickgold.florisboard.subtypeManager

@Suppress("MemberVisibilityCanBePrivate")
object Devtools {
    fun generateDebugLog(context: Context, prefs: FlorisPreferenceModel? = null): String {
        return buildString {
            append(generateDebugLogHeader(context, prefs))

        }
    }

    fun generateDebugLogHeader(context: Context, prefs: FlorisPreferenceModel? = null): String {
        return buildString {
            append(generateSystemInfoLog(context))
            appendLine()
            append(generateAppInfoLog(context))
            if (prefs != null) {
                appendLine()
                append(generateFeatureConfigLog(prefs))
            }
            appendLine()
            append(generateExtensionConfigLog(context))
            appendLine()
            append(generateActiveSubtypeConfigLog(context))
        }
    }

    fun generateSystemInfoLog(context: Context, withTitle: Boolean = true): String {
        return buildString {
            if (withTitle) appendLine("======= SYSTEM INFO =======")
            append("Time                : ").appendLine(TimeUtils.currentUtcTimestamp())
            append("Manufacturer        : ").appendLine(Build.MANUFACTURER)
            append("Model               : ").appendLine(Build.MODEL)
            append("Product             : ").appendLine(Build.PRODUCT)
            append("Android             : ").appendLine(getAndroidVersion(includeOemBuildId = true))
            append("ABIs                : ").appendLine(Build.SUPPORTED_ABIS.contentToString())
            append("Font scale          : ").appendLine(context.resources.configuration.fontScale)
            append("Locales             : ").appendLine(context.resources.configuration.locales.toLanguageTags())
        }
    }

    fun generateAppInfoLog(context: Context, withTitle: Boolean = true): String {
        return buildString {
            if (withTitle) appendLine("======= APP INFO =======")
            append("Package             : ").appendLine(BuildConfig.APPLICATION_ID)
            append("Name                : ").appendLine(context.resources.getString(R.string.floris_app_name))
            append("Version             : ").appendLine("${BuildConfig.VERSION_NAME} (${BuildConfig.VERSION_CODE})")
            append("Build type          : ").appendLine(BuildConfig.BUILD_TYPE)
            append("Build commit hash   : ").appendLine(BuildConfig.BUILD_COMMIT_HASH)
        }
    }

    fun generateFeatureConfigLog(prefs: FlorisPreferenceModel, withTitle: Boolean = true): String {
        return buildString {
            if (withTitle) appendLine("======= FEATURE CONFIG =======")
            append("Smartbar enabled            : ").appendLine(prefs.smartbar.enabled.get())
            append("Suggestions enabled         : ").appendLine(prefs.suggestion.enabled.get())
            append("Inline autofill enabled     : ").appendLine(prefs.suggestion.api30InlineSuggestionsEnabled.get())
            append("Glide enabled               : ").appendLine(prefs.glide.enabled.get())
            append("Internal clipboard enabled  : ").appendLine(prefs.clipboard.useInternalClipboard.get())
        }
    }

    fun generateExtensionConfigLog(context: Context, withTitle: Boolean = true): String {
        return buildString {
            if (withTitle) appendLine("======= EXTENSION CONFIG =======")
            appendLine("Theme extensions    : ")
            context.extensionManager().value.themes.value?.forEach { append("    ").appendLine(it.meta.id) }
            appendLine("Language Packs      : ")
            context.extensionManager().value.languagePacks.value?.forEach { append("    ").appendLine(it.meta.id) }
        }
    }

    fun generateActiveSubtypeConfigLog(context: Context, withTitle: Boolean = true): String {
        return buildString {
            if (withTitle) appendLine("======= ACTIVE SUBTYPE CONFIG =======")
            context.subtypeManager().value.let { subtypeManager ->
                appendLine("Active Subtype      : ")
                append("    ")
                appendLine(subtypeManager.activeSubtype.toShortString())
                appendLine("Installed Subtypes    : ")
                subtypeManager.subtypes.forEach { subtype ->
                    append("    ").appendLine(subtype.toShortString())
                }
            }

        }
    }

    fun getDeviceName(): String {
        val manufacturer = Build.MANUFACTURER
        val model = Build.MODEL
        val product = Build.PRODUCT
        return buildString {
            if (model.startsWith(manufacturer, ignoreCase = true)) {
                append(model.titlecase())
            } else {
                append(manufacturer.titlecase())
                append(" ")
                append(model.titlecase())
            }
            if (product.isNotBlank()) {
                append(" (")
                append(product)
                append(")")
            }
        }
    }

    fun getAndroidVersion(includeOemBuildId: Boolean = false): String {
        val fields = Build.VERSION_CODES::class.java.fields
        val codeName = fields.firstOrNull { it.getInt(Build.VERSION_CODES::class) == Build.VERSION.SDK_INT }?.name
            ?: return "Unknown"
        return buildString {
            append(Build.VERSION.RELEASE)
            append(" (cn=")
            append(codeName)
            append(" sdk=")
            append(Build.VERSION.SDK_INT)
            append(")")
            if (includeOemBuildId) {
                append(" [")
                append(Build.DISPLAY)
                append("]")
            }
        }
    }

}
