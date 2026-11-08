package com.jtn0123.audiowhisper.reader

import android.accessibilityservice.AccessibilityService
import android.graphics.Color
import android.graphics.PixelFormat
import android.graphics.drawable.GradientDrawable
import android.view.Gravity
import android.view.View
import android.view.WindowInsets
import android.view.WindowManager
import android.widget.TextView
import dev.patrickgold.florisboard.R

/**
 * A small "Stop reading" pill at the top of the screen while a reading is on, so it can be stopped
 * from any app with the keyboard closed. A plain view: no Compose for one button over other apps.
 */
internal class StopButton(private val service: AccessibilityService, private val onStop: () -> Unit) {
    private val windows = service.getSystemService(WindowManager::class.java)
    private var view: View? = null

    fun show() {
        if (view != null) return
        val density = service.resources.displayMetrics.density
        val button = TextView(service).apply {
            setText(R.string.read_aloud__stop)
            setTextColor(Color.WHITE)
            textSize = TEXT_SP
            val across = (PADDING_ACROSS_DP * density).toInt()
            val down = (PADDING_DOWN_DP * density).toInt()
            setPadding(across, down, across, down)
            setCompoundDrawablesRelativeWithIntrinsicBounds(R.drawable.ic_read_aloud_stop, 0, 0, 0)
            compoundDrawablePadding = (ICON_GAP_DP * density).toInt()
            background = GradientDrawable().apply {
                cornerRadius = CORNER_DP * density
                setColor(BACKGROUND)
            }
            elevation = ELEVATION_DP * density
            setOnClickListener { onStop() }
        }
        val statusBar = windows.currentWindowMetrics.windowInsets.getInsets(WindowInsets.Type.statusBars()).top
        val params = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_ACCESSIBILITY_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_LAYOUT_IN_SCREEN,
            PixelFormat.TRANSLUCENT,
        ).apply {
            gravity = Gravity.TOP or Gravity.CENTER_HORIZONTAL
            y = statusBar + (MARGIN_DP * density).toInt()
            title = service.getString(R.string.read_aloud__stop)
        }
        windows.addView(button, params)
        view = button
    }

    fun hide() {
        view?.let(windows::removeView)
        view = null
    }

    private companion object {
        const val TEXT_SP = 15f
        const val PADDING_ACROSS_DP = 16
        const val PADDING_DOWN_DP = 10
        const val ICON_GAP_DP = 8
        const val CORNER_DP = 24f
        const val ELEVATION_DP = 6f
        const val MARGIN_DP = 8
        const val BACKGROUND = 0xE6202124.toInt()
    }
}
