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

import org.florisboard.lib.kotlin.UtilityOp
import org.florisboard.lib.kotlin.UtilityRules
import org.florisboard.lib.kotlin.UtilityVersion
import org.florisboard.lib.kotlin.UtilityVersionPlan
import org.florisboard.lib.kotlin.UtilityVersionStatus
import com.jtn0123.audiowhisper.rust.WireFlag
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
        return WireFlag.decode(UtilityRules.call(UtilityOp.SHOWS_CHANGELOG, *raw.toTypedArray())[0])
    }

    suspend fun updateVersionOnInstallAndLastUse(context: Context, prefs: FlorisPreferenceModel) {
        if (WireFlag.decode(UtilityRules.call(UtilityOp.UNSET_VERSION, prefs.internal.versionOnInstall.get())[0])) {
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
            val result = UtilityRules.call(UtilityOp.PARSE_VERSION, raw)
            if (result.isEmpty()) return null
            with(UtilityVersionPlan) { if (result[STATUS] == UtilityVersionStatus.NUMBER_ERROR) throw NumberFormatException("For input string: \"${result[BAD_NUMBER]}\"") }
            return with(UtilityVersionPlan) { VersionName(result[MAJOR].toInt(), result[MINOR].toInt(), result[PATCH].toInt()) }
        }

    }

    override fun toString(): String = UtilityRules.call(UtilityOp.FORMAT_VERSION,
        *fields(if (extraName == null) WireFlag.FALSE else WireFlag.TRUE + extraName))[0]

    /** This version as a [UtilityVersion] row, with [name] as its extra name. */
    private fun fields(name: String = extraName.orEmpty()) =
        with(UtilityVersion) { row(MAJOR to "$major", MINOR to "$minor", PATCH to "$patch", EXTRA_NAME to name, EXTRA_VALUE to extraValue?.toString().orEmpty()) }
    operator fun compareTo(other: VersionName): Int = UtilityRules.call(UtilityOp.COMPARE_VERSIONS,
        *fields(), *other.fields())[0].toInt()
}
