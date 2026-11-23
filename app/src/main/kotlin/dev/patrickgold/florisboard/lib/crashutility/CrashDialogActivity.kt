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

package dev.patrickgold.florisboard.lib.crashutility

import android.annotation.SuppressLint
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import android.widget.Toolbar
import androidx.activity.ComponentActivity
import androidx.core.net.toUri
import com.jtn0123.audiowhisper.rust.CrashFact
import com.jtn0123.audiowhisper.rust.CrashRules
import com.jtn0123.audiowhisper.settingsui.PlatformCrashScreen
import dev.patrickgold.florisboard.BuildConfig
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.florisboard.app.FlorisPreferenceStore
import org.florisboard.lib.android.stringRes
import dev.patrickgold.florisboard.lib.devtools.Devtools
import dev.patrickgold.florisboard.lib.devtools.LogTopic
import dev.patrickgold.florisboard.lib.devtools.flogWarning
import kotlin.properties.ReadOnlyProperty
import kotlin.reflect.KProperty

private class SafePreferenceInstanceWrapper : ReadOnlyProperty<Any?, FlorisPreferenceModel?> {
    val cachedPreferenceModel = try {
        FlorisPreferenceStore
    } catch (_: Throwable) {
        null
    }

    override fun getValue(thisRef: Any?, property: KProperty<*>): FlorisPreferenceModel? {
        return cachedPreferenceModel?.getValue(thisRef, property)
    }
}

class CrashDialogActivity : ComponentActivity() {
    private var stacktraces: List<CrashUtility.Stacktrace> = listOf()
    private var errorReport = ""
    private val prefs by SafePreferenceInstanceWrapper()

    private val stacktrace by lazy { findViewById<TextView>(R.id.stacktrace) }
    private val reportInstructions by lazy { findViewById<TextView>(R.id.report_instructions) }
    private val copyToClipboard by lazy { findViewById<Button>(R.id.copy_to_clipboard) }
    private val openBugReportForm by lazy { findViewById<Button>(R.id.open_bug_report_form) }
    private val close by lazy { findViewById<Button>(R.id.close) }

    @SuppressLint("InflateParams")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Without the library the report can't be made: the platform's crash screen says so, and
        // the reports stay on disk for when it loads.
        if (PlatformCrashScreen.showIfUnavailable(this)) return
        val layout = layoutInflater.inflate(R.layout.crash_dialog, null)
        setContentView(layout)

        val toolbar = layout.findViewById<Toolbar>(R.id.crash_dialog_toolbar)
        setActionBar(toolbar)

        stacktraces = CrashUtility.getUnhandledStacktraces(this)
        if (stacktraces.isEmpty()) flogWarning(LogTopic.CRASH_UTILITY) { "Stacktrace file list is empty." }
        errorReport = CrashRules.report(facts(), BuildConfig.DEBUG, stacktraces.map { it.name }.toTypedArray(),
            stacktraces.map { it.details }.toTypedArray())
        stacktrace.text = errorReport

        reportInstructions.text =
            reportInstructions.text.toString().format(
                resources.getString(R.string.crash_dialog__bug_report_template)
            )

        copyToClipboard.setOnClickListener {
            val clipboardManager = getSystemService(Context.CLIPBOARD_SERVICE)
            val toastMessage: String = if (clipboardManager != null && clipboardManager is ClipboardManager) {
                clipboardManager.setPrimaryClip(ClipData.newPlainText(errorReport, errorReport))
                resources.getString(R.string.crash_dialog__copy_to_clipboard_success)
            } else {
                resources.getString(R.string.crash_dialog__copy_to_clipboard_failure)
            }
            Toast.makeText(this, toastMessage, Toast.LENGTH_SHORT).show()
        }

        openBugReportForm.setOnClickListener {
            val browserIntent = Intent(
                Intent.ACTION_VIEW,
                resources.getString(R.string.florisboard__issue_tracker_url).toUri()
            )
            startActivity(browserIntent)
        }

        close.setOnClickListener {
            finish()
        }
    }

    /** What the report is made of, a `CrashFact` row: the build, the phone and the debug log. */
    private fun facts(): Array<String> {
        val facts = Array(CrashFact.SIZE) { "" }
        facts[CrashFact.VERSION_NAME] = BuildConfig.VERSION_NAME
        facts[CrashFact.VERSION_CODE] = BuildConfig.VERSION_CODE.toString()
        facts[CrashFact.COMMIT_URL] = stringRes(R.string.florisboard__commit_by_hash_url, "hash" to BuildConfig.BUILD_COMMIT_HASH)
        facts[CrashFact.CHANGELOG_URL] = stringRes(R.string.florisboard__changelog_url, "version" to BuildConfig.VERSION_NAME)
        facts[CrashFact.DEVICE] = Devtools.getDeviceName()
        facts[CrashFact.ANDROID] = Devtools.getAndroidVersion()
        facts[CrashFact.DEBUG_LOG] = Devtools.generateDebugLog(this, prefs)
        return facts
    }
}
