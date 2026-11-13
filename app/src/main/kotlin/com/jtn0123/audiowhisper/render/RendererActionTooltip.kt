package com.jtn0123.audiowhisper.render

import android.view.View
import android.view.ViewConfiguration
import android.widget.FrameLayout
import kotlin.math.roundToInt

/** Android's tooltip panel, anchored to Rust's action bounds. The panel owns no input policy. */
internal class RendererActionTooltip(private val parent: FrameLayout) {
    private var anchor: View? = null
    private var show: Runnable? = null
    private var hide: Runnable? = null
    private var visible = false

    fun down(index: Int, snapshot: KeyboardSnapshot, onShown: () -> Unit) {
        clear()
        val packet = snapshot.smartbar.measure(parent.width.toFloat())
        val slot = (8 until packet.size step 6).firstOrNull { packet[it] == 1f && packet[it+1].toInt() == index } ?: return
        val text = snapshot.smartbar.tooltips.getOrNull(index)?.takeIf { it.isNotEmpty() } ?: return
        val view = View(parent.context).apply {
            isHapticFeedbackEnabled = false
            tooltipText = text
            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
        }
        val params = FrameLayout.LayoutParams((packet[slot+4]-packet[slot+2]).roundToInt(),
            (packet[slot+5]-packet[slot+3]).roundToInt()).apply {
            leftMargin = packet[slot+2].roundToInt()
            topMargin = packet[slot+3].roundToInt()
        }
        parent.addView(view, params)
        anchor = view
        show = Runnable {
            visible = view.performLongClick()
            if (visible) onShown()
        }.also { parent.postDelayed(it, ViewConfiguration.getLongPressTimeout().toLong()) }
    }
    fun up(): Boolean {
        show?.let(parent::removeCallbacks)
        show = null
        val consumed = visible
        if (visible) hide = Runnable { clear() }.also { parent.postDelayed(it, 1500) }
        else clear()
        return consumed
    }
    fun clear() {
        show?.let(parent::removeCallbacks)
        hide?.let(parent::removeCallbacks)
        anchor?.let { it.tooltipText = null; parent.removeView(it) }
        anchor = null; show = null; hide = null; visible = false
    }
}
