package com.jtn0123.audiowhisper.settingsui

import android.app.Activity
import android.content.ClipData
import android.content.ClipboardManager
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import com.jtn0123.audiowhisper.rust.NativeRuntime

/** Bootstrap UI: deliberately depends on neither native code nor Compose. */
internal object PlatformCrashScreen {
    fun show(activity: Activity, report: CrashReport) {
        val padding = (24 * activity.resources.displayMetrics.density).toInt()
        activity.setContentView(LinearLayout(activity).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(android.graphics.Color.WHITE)
            setPadding(padding, padding, padding, padding)
            addView(TextView(activity).apply { text = report.summary; textSize = 20f; setTextColor(android.graphics.Color.BLACK) })
            addView(Button(activity).apply {
                setText(dev.patrickgold.florisboard.R.string.crash__copy_details)
                setOnClickListener {
                    activity.getSystemService(ClipboardManager::class.java)
                        .setPrimaryClip(ClipData.newPlainText("AudioWhisper crash details", report.details))
                }
            })
        })
    }
    fun showIfUnavailable(activity: Activity): Boolean {
        val failure = NativeRuntime.failure ?: return false
        show(activity, CrashReport.nativeLoad(failure))
        return true
    }
}
