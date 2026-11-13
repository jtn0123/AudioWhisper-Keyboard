package com.jtn0123.audiowhisper.render

import com.jtn0123.audiowhisper.rust.RenderProtocol.*
import android.view.View
import android.widget.FrameLayout
import kotlin.math.roundToInt

/** Android's tooltip panel, anchored to Rust's action bounds. The panel owns no input policy. */
internal class RendererActionTooltip(private val parent: FrameLayout) {
    private var anchor: View? = null
    private var hide: Runnable? = null
    private var visible = false

    fun show(index: Int, snapshot: KeyboardSnapshot): Boolean {
        clear()
        val packet = snapshot.smartbar.measure(parent.width.toFloat())
        val slot = (BAR_MEASUREMENT_HEADER until packet.size step BAR_SLOT_STRIDE).firstOrNull { packet[it] == BAR_SLOT_ACTION.toFloat() && packet[it+BAR_SLOT_ID].toInt() == index } ?: return false
        val text = snapshot.smartbar.tooltips.getOrNull(index)?.takeIf { it.isNotEmpty() } ?: return false
        val view = View(parent.context).apply {
            isHapticFeedbackEnabled = false
            tooltipText = text
            importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
        }
        val params = FrameLayout.LayoutParams((packet[slot+BAR_SLOT_RIGHT]-packet[slot+BAR_SLOT_LEFT]).roundToInt(),
            (packet[slot+BAR_SLOT_BOTTOM]-packet[slot+BAR_SLOT_TOP]).roundToInt()).apply {
            leftMargin = packet[slot+BAR_SLOT_LEFT].roundToInt()
            topMargin = packet[slot+BAR_SLOT_TOP].roundToInt()
        }
        parent.addView(view, params)
        anchor = view
        visible = view.performLongClick()
        return visible
    }
    fun up(): Boolean {
        val consumed = visible
        if (visible) hide = Runnable { clear() }.also { parent.postDelayed(it, 1500) }
        else clear()
        return consumed
    }
    fun clear() {
        hide?.let(parent::removeCallbacks)
        anchor?.let { it.tooltipText = null; parent.removeView(it) }
        anchor = null; hide = null; visible = false
    }
}
