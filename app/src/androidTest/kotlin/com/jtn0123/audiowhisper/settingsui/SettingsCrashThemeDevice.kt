package com.jtn0123.audiowhisper.settingsui

import android.content.Context
import android.content.res.Configuration
import android.graphics.drawable.ColorDrawable
import android.view.ContextThemeWrapper
import android.view.WindowInsetsController
import android.widget.TextView
import dev.patrickgold.florisboard.R
import org.json.JSONObject

/**
 * The crash screen draws with the platform's DayNight attributes, so it must match them in the
 * device's current mode, and those attributes must differ between the modes (or matching
 * them would prove nothing). Run once in light and once in dark mode.
 */
internal class SettingsCrashThemeDevice(private val h: SettingsHarness) {
    private val context = h.instrumentation.targetContext

    fun run(report: JSONObject) {
        val night = context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK ==
            Configuration.UI_MODE_NIGHT_YES
        val current = colors(context)
        val opposite = colors(context.createConfigurationContext(Configuration(context.resources.configuration).apply {
            uiMode = uiMode and Configuration.UI_MODE_NIGHT_MASK.inv() or
                if (night) Configuration.UI_MODE_NIGHT_NO else Configuration.UI_MODE_NIGHT_YES
        }))
        h.lastStep = "platform DayNight colours are the same in both modes"
        check(current.first != opposite.first && current.second != opposite.second)
        h.main {
            val window = checkNotNull(h.activity).window
            h.lastStep = "crash screen background is not the platform's"
            check((window.decorView.background as ColorDrawable).color == current.first)
            val text = h.views(TextView::class.java).single { it.text == context.getString(R.string.crash__native_load_summary) }
            h.lastStep = "crash screen text colour is not the platform's"
            check(text.currentTextColor == current.second)
            val light = window.insetsController?.systemBarsAppearance?.and(WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS)
            h.lastStep = "status bar icons do not follow the mode"
            check((light != 0) == !night)
        }
        report.put("crash_theme_night", night).put("crash_theme_platform", true)
    }

    /** Background and primary text colours of the device DayNight theme in [base]'s mode. */
    private fun colors(base: Context): Pair<Int, Int> {
        val themed = ContextThemeWrapper(base, android.R.style.Theme_DeviceDefault_DayNight)
        val background = themed.obtainStyledAttributes(intArrayOf(android.R.attr.colorBackground)).run {
            try { getColor(0, 0) } finally { recycle() }
        }
        val text = themed.obtainStyledAttributes(intArrayOf(android.R.attr.textColorPrimary)).run {
            try { checkNotNull(getColorStateList(0)).defaultColor } finally { recycle() }
        }
        return background to text
    }
}
