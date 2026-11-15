package com.jtn0123.audiowhisper.settingsui

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.content.res.Configuration
import android.view.ContextThemeWrapper
import android.view.View
import android.view.WindowInsets
import android.view.WindowInsetsController
import android.view.inputmethod.InputMethodManager
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import androidx.core.graphics.drawable.toDrawable
import com.jtn0123.audiowhisper.rust.NativeRuntime
import dev.patrickgold.florisboard.R

/**
 * Bootstrap UI: deliberately depends on neither native code nor Compose. It draws with the
 * device's own DayNight theme, so it follows the system's dark mode with no app preference read.
 */
internal object PlatformCrashScreen {
    fun show(activity: Activity, report: CrashReport) {
        val context = themed(activity)
        // Content first: the window's insets controller lives on its decor view, which
        // setContentView installs; asking before then dereferences a null decor.
        activity.setContentView(panel(context, report.summary, R.string.crash__copy_details) {
            activity.getSystemService(ClipboardManager::class.java)
                .setPrimaryClip(ClipData.newPlainText("AudioWhisper crash details", report.details))
        })
        activity.window.setBackgroundDrawable(color(context, android.R.attr.colorBackground).toDrawable())
        val light = WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS or WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS
        activity.window.insetsController?.setSystemBarsAppearance(if (isNight(activity)) 0 else light, light)
    }

    fun showIfUnavailable(activity: Activity): Boolean {
        val failure = NativeRuntime.failure ?: return false
        show(activity, CrashReport.nativeLoad(failure))
        return true
    }

    /** The keyboard's input view when the engine is missing: say so, and offer another keyboard. */
    fun inputView(context: Context): View {
        val themed = themed(context)
        return panel(themed, R.string.crash__keyboard_unavailable, R.string.crash__switch_keyboard) {
            context.getSystemService(InputMethodManager::class.java).showInputMethodPicker()
        }.apply { setBackgroundColor(color(themed, android.R.attr.colorBackground)) }
    }

    private fun themed(context: Context): Context =
        ContextThemeWrapper(context, android.R.style.Theme_DeviceDefault_DayNight)

    private fun isNight(context: Context) =
        context.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK == Configuration.UI_MODE_NIGHT_YES

    private fun color(context: Context, attribute: Int): Int =
        context.obtainStyledAttributes(intArrayOf(attribute)).run {
            try { getColor(0, 0) } finally { recycle() }
        }

    /** A message and one button, padded clear of the system bars and the display cutout. */
    private fun panel(context: Context, message: Int, action: Int, onClick: () -> Unit): LinearLayout {
        val padding = (24 * context.resources.displayMetrics.density).toInt()
        return LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            setPadding(padding, padding, padding, padding)
            setOnApplyWindowInsetsListener { view, insets ->
                val bars = insets.getInsets(WindowInsets.Type.systemBars() or WindowInsets.Type.displayCutout())
                view.setPadding(padding + bars.left, padding + bars.top, padding + bars.right, padding + bars.bottom)
                insets
            }
            addView(TextView(context).apply {
                setText(message)
                setTextAppearance(android.R.style.TextAppearance_DeviceDefault_Medium)
                setTextColor(context.obtainStyledAttributes(intArrayOf(android.R.attr.textColorPrimary)).run {
                    try { getColorStateList(0) } finally { recycle() }
                })
            })
            addView(Button(context).apply { setText(action); setOnClickListener { onClick() } })
        }
    }
}
