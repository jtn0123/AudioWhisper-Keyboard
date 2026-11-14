/*
 * Copyright (C) 2020-2025 The FlorisBoard Contributors
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

package dev.patrickgold.florisboard.lib.util

import org.florisboard.lib.kotlin.UtilityRules
import android.content.Context
import dev.patrickgold.florisboard.app.FlorisPreferenceModel

object AppVersionUtils {
    private fun getRawVersionName(context: Context): String {
        return try {
            context.packageManager.getPackageInfo(context.packageName, 0).versionName!!
        } catch (e: Exception) {
            "undefined"
        }
    }

    fun shouldShowChangelog(context: Context, prefs: FlorisPreferenceModel): Boolean {
        // Preserve public parser exceptions, then ask Rust for the complete changelog rule.
        val raw = listOf(prefs.internal.versionOnInstall.get(), prefs.internal.versionLastChangelog.get(), getRawVersionName(context))
        raw.forEach { VersionName.fromString(it) }
        return UtilityRules.call(12, *raw.toTypedArray())[0] == "1"
    }

    suspend fun updateVersionOnInstallAndLastUse(context: Context, prefs: FlorisPreferenceModel) {
        if (UtilityRules.call(13, prefs.internal.versionOnInstall.get())[0] == "1") {
            prefs.internal.versionOnInstall.set(getRawVersionName(context))
        }
        prefs.internal.versionLastUse.set(getRawVersionName(context))
    }

    suspend fun updateVersionLastChangelog(context: Context, prefs: FlorisPreferenceModel) {
        prefs.internal.versionLastChangelog.set(getRawVersionName(context))
    }
}

data class VersionName(
    val major: Int,
    val minor: Int,
    val patch: Int,
    val extraName: String? = null,
    val extraValue: Int? = null
) {
    companion object {
        val DEFAULT: VersionName = VersionName(0, 0, 0)
        const val DEFAULT_RAW: String = "0.0.0"

        fun fromString(raw: String): VersionName? {
            val result = UtilityRules.call(9, raw)
            if (result.isEmpty()) return null
            if (result[0] == "number-error") throw NumberFormatException("For input string: \"${result[1]}\"")
            return VersionName(result[1].toInt(), result[2].toInt(), result[3].toInt())
        }

    }

    override fun toString(): String = UtilityRules.call(11, major.toString(), minor.toString(), patch.toString(),
        if (extraName == null) "0" else "1$extraName", extraValue?.toString().orEmpty())[0]

    private fun fields() = listOf(major.toString(), minor.toString(), patch.toString(), extraName.orEmpty(), extraValue?.toString().orEmpty())
    operator fun compareTo(other: VersionName): Int = UtilityRules.call(10,
        *(fields() + other.fields()).toTypedArray())[0].toInt()
}
