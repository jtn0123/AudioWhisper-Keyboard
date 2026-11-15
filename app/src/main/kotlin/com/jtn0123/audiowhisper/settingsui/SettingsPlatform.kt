package com.jtn0123.audiowhisper.settingsui

import android.content.res.Configuration
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.isUnspecified
import androidx.compose.ui.graphics.toArgb
import androidx.core.view.WindowCompat
import androidx.lifecycle.lifecycleScope
import dev.patrickgold.florisboard.app.AppTheme
import dev.patrickgold.florisboard.app.FlorisAppActivity
import dev.patrickgold.florisboard.app.FlorisPreferenceModel
import dev.patrickgold.florisboard.ime.media.emoji.EmojiHistoryHelper
import dev.patrickgold.florisboard.lib.util.InputMethodUtils
import dev.patrickgold.jetpref.datastore.model.PreferenceData
import kotlinx.coroutines.launch
import kotlinx.coroutines.channels.Channel
import org.florisboard.lib.android.systemVibratorOrNull
import org.florisboard.lib.android.vibrate
import org.florisboard.lib.color.neutralDynamicColorScheme

/** Android facts and effects. No navigation/widget policy is implemented here. */
internal class SettingsPlatform(private val activity: FlorisAppActivity, private val prefs: FlorisPreferenceModel, private val persisted: () -> Unit) {
    private val queue = Channel<List<List<String>>>(Channel.UNLIMITED)
    var pending = 0
        private set
    init { activity.lifecycleScope.launch {
        for (plan in queue) {
            for (effect in plan) {
                if (effect[0] == "deleteEmojiHistory") {
                    if (effect[1] == "pinned") EmojiHistoryHelper.deletePinned(prefs) else EmojiHistoryHelper.deleteHistory(prefs)
                    continue
                }
                val pref = prefs.declaredPreferenceEntries.values.single { it.key == effect[1] }
                if (effect[0] == "reset") pref.reset().getOrThrow() else write(pref, effect[2])
            }
            pending--
            if (pending == 0) persisted()
        }
    } }
    fun palette(): IntArray {
        val theme = prefs.other.settingsTheme.get()
        val night = activity.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES
        val dark = theme == AppTheme.DARK || theme == AppTheme.AMOLED_DARK || (night && (theme == AppTheme.AUTO || theme == AppTheme.AUTO_AMOLED))
        val accent = prefs.other.accentColor.get().let {
            if (it.isUnspecified) Color(activity.resources.getColor(android.R.color.system_accent1_500, activity.theme)) else it
        }
        val colors = neutralDynamicColorScheme(accent, dark, theme == AppTheme.AMOLED_DARK || theme == AppTheme.AUTO_AMOLED)
        WindowCompat.getInsetsController(activity.window, activity.window.decorView).isAppearanceLightStatusBars = !dark
        return intArrayOf(colors.background.toArgb(), colors.onSurface.toArgb(), colors.onSurfaceVariant.toArgb(),
            colors.primary.toArgb(), colors.primaryContainer.toArgb(), colors.surfaceContainerHighest.toArgb(),
            colors.onPrimary.toArgb(), colors.outline.toArgb(), colors.surface.toArgb(),
            colors.secondary.toArgb(), colors.outlineVariant.toArgb(), colors.surfaceContainerHigh.toArgb(), colors.surfaceContainer.toArgb())
    }
    fun dialogWindowWidth(): Float {
        val value = android.util.TypedValue()
        val attr = if (activity.resources.configuration.orientation == Configuration.ORIENTATION_PORTRAIT)
            android.R.attr.windowMinWidthMinor else android.R.attr.windowMinWidthMajor
        if (!android.app.Dialog(activity).context.theme.resolveAttribute(attr, value, true)) return 0f
        val metrics = activity.resources.displayMetrics
        return if (value.type == android.util.TypedValue.TYPE_FRACTION) value.getFraction(metrics.widthPixels.toFloat(), metrics.widthPixels.toFloat())
            else android.util.TypedValue.complexToDimension(value.data, metrics)
    }
    fun facts() = arrayOf(InputMethodUtils.isFlorisboardEnabled(activity).toString(), InputMethodUtils.isFlorisboardSelected(activity).toString())
    private val vibrator = activity.systemVibratorOrNull()
    /** InputFeedbackScreen's `vibrator != null && vibrator.hasVibrator()` and `hasAmplitudeControl()`. */
    fun vibratorFacts() = arrayOf((vibrator?.hasVibrator() == true).toString(), (vibrator?.hasAmplitudeControl() == true).toString())
    /** A haptic slider's `onPreviewSelectedValue`. */
    fun vibrate(duration: Int, strength: Int) { vibrator?.vibrate(duration, strength) }
    private fun <T : Any> raw(pref: PreferenceData<T>) = pref.serializer.serialize(pref.get()).orEmpty()
    fun snapshot() = prefs.declaredPreferenceEntries.values.flatMap { listOf(it.key, raw(it)) }.toTypedArray()
    private suspend fun <T : Any> write(pref: PreferenceData<T>, value: String) {
        pref.set(requireNotNull(pref.serializer.deserialize(value))).getOrThrow()
    }
    fun enqueue(effects: List<List<String>>) {
        if (effects.isEmpty()) return
        pending++
        queue.trySend(effects).getOrThrow()
    }
}
