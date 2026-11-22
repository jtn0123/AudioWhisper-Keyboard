package com.jtn0123.audiowhisper.settingsui

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.database.ContentObserver
import android.os.Handler
import android.os.Looper
import android.provider.Settings
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import com.jtn0123.audiowhisper.rust.SettingsUiPermission
import com.jtn0123.audiowhisper.rust.SettingsUiSetupEffect
import com.jtn0123.audiowhisper.rust.SettingsUiSetupFact
import dev.patrickgold.florisboard.app.FlorisAppActivity

/**
 * The Android calls the native setup screen cannot make (stage4-setup.md): the notification
 * permission request and its state, the app's notification settings, bringing the app back to the
 * front, and noticing when the keyboard is enabled or selected. Rust decides when each happens and
 * what each answer means.
 */
internal class SettingsSetup(
    private val activity: ComponentActivity,
    private val imeChanged: () -> Unit,
    answer: (Array<String>) -> Unit,
) {
    // Registered outside the lifecycle so it can be made after onCreate; an answer that arrives
    // while the activity is recreated is delivered here as soon as the new one registers.
    private val request = activity.activityResultRegistry.register(REQUEST_KEY, ActivityResultContracts.RequestPermission()) { granted ->
        answer(SettingsUiPermission.row(SettingsUiPermission.GRANTED to granted.toString(), SettingsUiPermission.RATIONALE to rationale().toString()))
    }
    private val observer = object : ContentObserver(Handler(Looper.getMainLooper())) {
        override fun onChange(selfChange: Boolean) = imeChanged()
    }

    init {
        for (key in IME_KEYS) activity.contentResolver.registerContentObserver(Settings.Secure.getUriFor(key), false, observer)
    }

    /** The facts the setup screen reads: whether notifications are allowed and Android would ask again. */
    fun facts(): Array<String> = arrayOf(
        SettingsUiSetupFact.NOTIFICATIONS, granted().toString(),
        SettingsUiSetupFact.RATIONALE, rationale().toString(),
    )

    fun execute(plan: List<String>) {
        when (plan[0]) {
            SettingsUiSetupEffect.ASK -> request.launch(Manifest.permission.POST_NOTIFICATIONS)
            SettingsUiSetupEffect.NOTIFICATION_SETTINGS -> activity.startActivity(
                Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS).putExtra(Settings.EXTRA_APP_PACKAGE, activity.packageName),
            )
            // Compose's setup brought itself back this way once the keyboard was enabled.
            SettingsUiSetupEffect.RETURN -> activity.startActivity(
                Intent(activity, FlorisAppActivity::class.java).addFlags(
                    Intent.FLAG_ACTIVITY_RESET_TASK_IF_NEEDED or Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP,
                ),
            )
        }
    }

    fun release() {
        request.unregister()
        activity.contentResolver.unregisterContentObserver(observer)
    }

    // ContextCompat answers from areNotificationsEnabled below Android 13, where the permission
    // does not exist; there Android never asks, so a denial reads as final.
    private fun granted() =
        ContextCompat.checkSelfPermission(activity, Manifest.permission.POST_NOTIFICATIONS) == PackageManager.PERMISSION_GRANTED

    private fun rationale() = ActivityCompat.shouldShowRequestPermissionRationale(activity, Manifest.permission.POST_NOTIFICATIONS)

    private companion object {
        const val REQUEST_KEY = "aw-settings-notifications"
        val IME_KEYS = listOf(Settings.Secure.ENABLED_INPUT_METHODS, Settings.Secure.DEFAULT_INPUT_METHOD)
    }
}
