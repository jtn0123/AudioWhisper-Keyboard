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

package dev.patrickgold.florisboard.app

import android.content.Intent
import android.os.Bundle
import android.widget.FrameLayout
import androidx.activity.BackEventCompat
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.lifecycle.lifecycleScope
import com.jtn0123.audiowhisper.rust.SettingsUiBack
import com.jtn0123.audiowhisper.settingsui.RustSettingsView
import com.jtn0123.audiowhisper.settingsui.settingsFileWork
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.lib.util.AppVersionUtils
import org.florisboard.lib.kotlin.collectIn
import java.util.concurrent.atomic.AtomicBoolean

private const val SETTINGS_PROFILE_TAG = "aw-settings"

enum class AppTheme(val id: String) {
    AUTO("auto"),
    AUTO_AMOLED("auto_amoled"),
    LIGHT("light"),
    DARK("dark"),
    AMOLED_DARK("amoled_dark");
}

/**
 * The settings activity, a host for the Rust settings (stage4-compose-host.md). Rust draws every screen,
 * setup and its not-found page included, and picks the first one; this keeps the splash screen up until the
 * preferences load, and passes on the window insets, Back, the saved state and the intents that bring a
 * settings link or an extension file.
 */
open class FlorisAppActivity : ComponentActivity() {
    private var rustSettings: RustSettingsView? = null
    private var rustInputFocus = false
    private var rustState: Array<String>? = null
    private val prefs: FlorisPreferenceModel by lazy { val model by FlorisPreferenceStore; model }
    private val appContext by appContext()

    override fun onCreate(savedInstanceState: Bundle?) {
        if (android.util.Log.isLoggable(SETTINGS_PROFILE_TAG, android.util.Log.DEBUG)) android.util.Log.d(SETTINGS_PROFILE_TAG, "activity_start_ns=${System.nanoTime()}")
        if (com.jtn0123.audiowhisper.rust.NativeRuntime.failure != null) {
            super.onCreate(savedInstanceState)
            com.jtn0123.audiowhisper.settingsui.PlatformCrashScreen.showIfUnavailable(this)
            return
        }
        // Splash screen should be installed before calling super.onCreate()
        installSplashScreen().apply {
            setKeepOnScreenCondition { !appContext.preferenceStoreLoaded.value }
        }
        super.onCreate(savedInstanceState)
        rustState = savedInstanceState?.getStringArray("rust-settings-state")
        rustInputFocus = savedInstanceState?.getBoolean("rust-settings-input-focus") == true
        WindowCompat.setDecorFitsSystemWindows(window, false)

        // The host waits for the datastore model to load; until then the splash screen stays drawn.
        val isModelLoaded = AtomicBoolean(false)
        appContext.preferenceStoreLoaded.collectIn(lifecycleScope) { loaded ->
            if (!loaded || isModelLoaded.getAndSet(true)) return@collectIn
            AppVersionUtils.updateVersionOnInstallAndLastUse(this, prefs)
            val host = installRustSettings()
            if (android.util.Log.isLoggable(SETTINGS_PROFILE_TAG, android.util.Log.DEBUG)) {
                window.decorView.viewTreeObserver.registerFrameCommitCallback {
                    android.util.Log.d(SETTINGS_PROFILE_TAG, "activity_commit_ns=${System.nanoTime()}")
                }
            }
            // Rust picks the first screen (SettingsUiOp.START); a recreated activity restores its stack instead.
            if (rustState == null) route(host, intent, fresh = true)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        rustSettings?.let { host -> route(host, intent, fresh = false) }
    }

    /** Hands [intent]'s settings link or extension file to Rust ([SettingsEntry.request]). */
    private fun route(host: RustSettingsView, intent: Intent, fresh: Boolean) {
        val shared = intent.clipData?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.uri
        val link = SettingsEntry.route(intent.action, intent.dataString, intent.data, shared, settingsFileWork::keep)
        SettingsEntry.request(fresh, link)?.let { (operation, input) -> host.send(operation, *input) }
    }

    private fun installRustSettings(): RustSettingsView {
        val host = RustSettingsView(this).apply { restoreInputFocus = rustInputFocus }
        host.bind(this, prefs, rustState)
        rustSettings = host
        val root = FrameLayout(this).apply { addView(host, FrameLayout.LayoutParams(-1, -1)) }
        ViewCompat.setOnApplyWindowInsetsListener(root) { view, insets ->
            val bars = insets.getInsets(WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout() or WindowInsetsCompat.Type.ime())
            view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
            host.imeVisibility(insets.isVisible(WindowInsetsCompat.Type.ime()))
            insets
        }
        setContentView(root)
        ViewCompat.requestApplyInsets(root)
        val backCallback = object : OnBackPressedCallback(false) {
            override fun handleOnBackStarted(backEvent: BackEventCompat) = host.back(SettingsUiBack.STARTED, backEvent.progress)
            override fun handleOnBackProgressed(backEvent: BackEventCompat) = host.back(SettingsUiBack.PROGRESSED, backEvent.progress)
            override fun handleOnBackCancelled() = host.back(SettingsUiBack.CANCELLED)
            override fun handleOnBackPressed() = host.back(SettingsUiBack.PRESSED)
        }
        host.backAvailability = { backCallback.isEnabled = it }
        onBackPressedDispatcher.addCallback(this, backCallback)
        return host
    }

    override fun onSaveInstanceState(outState: Bundle) {
        rustSettings?.let { outState.putBoolean("rust-settings-input-focus", it.inputHasFocus) }
        rustSettings?.save()?.let { outState.putStringArray("rust-settings-state", it) }
        super.onSaveInstanceState(outState)
    }

    override fun onResume() {
        super.onResume()
        if (com.jtn0123.audiowhisper.rust.NativeRuntime.failure != null) return
        rustSettings?.refreshPlatform()
    }
}
