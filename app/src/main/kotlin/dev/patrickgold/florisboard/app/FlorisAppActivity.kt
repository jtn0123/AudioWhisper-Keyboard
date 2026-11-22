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
import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.BackEventCompat
import androidx.activity.OnBackPressedCallback
import androidx.activity.result.contract.ActivityResultContracts
import android.widget.FrameLayout
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.compose.runtime.DisposableEffect
import com.jtn0123.audiowhisper.rust.SettingsUiBack
import com.jtn0123.audiowhisper.settingsui.RustSettingsView
import com.jtn0123.audiowhisper.settingsui.settingsFileWork
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.displayCutoutPadding
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.core.net.toUri
import androidx.core.view.WindowCompat
import androidx.lifecycle.lifecycleScope
import androidx.navigation.NavController
import androidx.navigation.compose.rememberNavController
import dev.patrickgold.florisboard.R
import dev.patrickgold.florisboard.app.apptheme.FlorisAppTheme
import dev.patrickgold.florisboard.appContext
import dev.patrickgold.florisboard.lib.compose.LocalPreviewFieldController
import dev.patrickgold.florisboard.lib.compose.PreviewKeyboardField
import dev.patrickgold.florisboard.lib.compose.rememberPreviewFieldController
import dev.patrickgold.florisboard.lib.util.AppVersionUtils
import dev.patrickgold.jetpref.datastore.model.observeAsState
import dev.patrickgold.jetpref.datastore.ui.ProvideDefaultDialogPrefStrings
import org.florisboard.lib.compose.ProvideLocalizedResources
import org.florisboard.lib.compose.conditional
import org.florisboard.lib.compose.stringRes
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

val LocalNavController = staticCompositionLocalOf<NavController> {
    error("LocalNavController not initialized")
}

open class FlorisAppActivity : ComponentActivity() {
    protected open val allowRustSettings = true
    private var rustSettings: RustSettingsView? = null
    private var rustInputFocus = false
    private var rustState: Array<String>? = null
    private var contentShown = false
    private var fallbackReady = false
    private val composeFallback = registerForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        result.data?.getStringExtra("native-route")?.let { rustSettings?.navigate(it) }
        rustSettings?.refreshPlatform()
    }
    private val prefs: FlorisPreferenceModel by lazy { val model by FlorisPreferenceStore; model }
    private val appContext by appContext()
    private var appTheme by mutableStateOf(AppTheme.AUTO)

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

        prefs.other.settingsTheme.asFlow().collectIn(lifecycleScope) {
            appTheme = it
        }

        // We defer the setContent call until the datastore model is loaded, until then the splash screen stays drawn
        val isModelLoaded = AtomicBoolean(false)
        appContext.preferenceStoreLoaded.collectIn(lifecycleScope) { loaded ->
            if (!loaded || isModelLoaded.getAndSet(true)) return@collectIn
            AppVersionUtils.updateVersionOnInstallAndLastUse(this, prefs)
            // Rust draws settings and setup; it picks which opens first (SettingsUiOp.START).
            if (allowRustSettings) {
                installRustSettings()
            } else setContent {
                ProvideLocalizedResources(
                    this,
                    appName = R.string.floris_app_name,
                ) {
                    FlorisAppTheme(theme = appTheme) {
                        Surface(color = MaterialTheme.colorScheme.background) {
                            AppContent()
                        }
                    }
                }
            }
            if (android.util.Log.isLoggable(SETTINGS_PROFILE_TAG, android.util.Log.DEBUG)) {
                window.decorView.viewTreeObserver.registerFrameCommitCallback {
                    android.util.Log.d(SETTINGS_PROFILE_TAG, "activity_commit_ns=${System.nanoTime()}")
                }
            }
            contentShown = true
            val host = rustSettings
            if (host != null && rustState == null) route(host, intent, fresh = true)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        rustSettings?.let { host -> route(host, intent, fresh = false) }
    }

    /** A fresh activity starts on the screen Rust picks for [intent]; a later link or file opens its screen. */
    private fun route(host: RustSettingsView, intent: Intent, fresh: Boolean) {
        val shared = intent.clipData?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.uri
        val link = SettingsEntry.route(intent.action, intent.dataString, intent.data, shared, settingsFileWork::keep)
        if (fresh) host.start(link) else if (link != null) host.navigate(link)
    }

    private fun installRustSettings() {
        val host = RustSettingsView(this).apply { restoreInputFocus = rustInputFocus }
        host.bind(this, prefs, rustState) { route, native ->
            val target = Intent(Intent.ACTION_VIEW, "ui://florisboard/$route".toUri())
                .addCategory(Intent.CATEGORY_BROWSABLE).setClass(this, ComposeSettingsFallbackActivity::class.java)
                .putExtra("native-routes", native.toTypedArray())
            composeFallback.launch(target)
        }
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

    @Composable
    private fun AppContent() {
        val navController = rememberNavController()
        val previewFieldController = rememberPreviewFieldController()
        DisposableEffect(navController) {
            val listener = NavController.OnDestinationChangedListener { _, destination, _ ->
                if (!allowRustSettings && fallbackReady) {
                    // The routes Rust draws, as it listed them: hand any of them back.
                    val native = intent?.getStringArrayExtra("native-routes").orEmpty()
                    val route = native.firstOrNull { destination.hasDeepLink("ui://florisboard/$it".toUri()) }
                    if (route != null) { if (route != "settings/home") setResult(RESULT_OK, Intent().putExtra("native-route", route)); finish() }
                }
            }
            navController.addOnDestinationChangedListener(listener)
            onDispose { navController.removeOnDestinationChangedListener(listener) }
        }

        val isImeSetUp by prefs.internal.isImeSetUp.observeAsState()

        CompositionLocalProvider(
            LocalNavController provides navController,
            LocalPreviewFieldController provides previewFieldController,
        ) {
            ProvideDefaultDialogPrefStrings(
                confirmLabel = stringRes(R.string.action__ok),
                dismissLabel = stringRes(R.string.action__cancel),
                neutralLabel = stringRes(R.string.action__default),
            ) {
                Column(
                    modifier = Modifier
                        //.statusBarsPadding()
                        .navigationBarsPadding()
                        .conditional(LocalConfiguration.current.orientation == Configuration.ORIENTATION_LANDSCAPE) {
                            displayCutoutPadding()
                        }
                        .imePadding(),
                ) {
                    Routes.AppNavHost(
                        modifier = Modifier.weight(1.0f),
                        navController = navController,
                        startDestination = if (isImeSetUp) Routes.Settings.Home::class else Routes.Setup.Screen::class,
                    )
                    PreviewKeyboardField(previewFieldController)
                }
            }
        }

        // Its NavController opens activity.intent itself when the graph is set.
        LaunchedEffect(Unit) { fallbackReady = true }
    }
}
