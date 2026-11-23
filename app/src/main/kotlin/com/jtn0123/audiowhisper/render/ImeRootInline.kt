package com.jtn0123.audiowhisper.render

import android.content.Context
import android.graphics.Rect
import android.view.View
import android.view.ViewGroup
import android.widget.HorizontalScrollView
import android.widget.LinearLayout
import android.widget.inline.InlineContentView
import dev.patrickgold.florisboard.ime.nlp.NlpInlineAutofillSuggestion

/**
 * Android's inline autofill chips, in a row that scrolls where Rust's plan put it. Each chip is a
 * surface of the autofill service's: it is clipped to what the row shows, hidden when scrolled
 * out, and drawn above the window only while no popup is open, as Compose's row did it.
 */
internal class ImeInlineChips(context: Context) : HorizontalScrollView(context) {
    private val row = LinearLayout(context)
    private var shown = emptyList<InlineContentView>()
    private var onTop = true

    init {
        isHorizontalScrollBarEnabled = true
        addView(row, LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.MATCH_PARENT))
    }

    /** Shows the [suggestions] Android inflated, above the window unless [popups] are open. */
    fun update(suggestions: List<NlpInlineAutofillSuggestion>, popups: Int) {
        val views = suggestions.mapNotNull { it.view }
        if (views != shown) {
            row.removeAllViews()
            for (view in views) {
                (view.parent as? ViewGroup)?.removeView(view)
                row.addView(view, LinearLayout.LayoutParams(LayoutParams.WRAP_CONTENT, LayoutParams.WRAP_CONTENT))
            }
            shown = views
            scrollTo(0, 0)
        }
        onTop = popups == 0
        clip()
    }

    override fun onScrollChanged(l: Int, t: Int, oldl: Int, oldt: Int) {
        super.onScrollChanged(l, t, oldl, oldt)
        clip()
    }

    override fun onLayout(changed: Boolean, l: Int, t: Int, r: Int, b: Int) {
        super.onLayout(changed, l, t, r, b)
        clip()
    }

    private fun clip() {
        val (from, to) = scrollX to scrollX + width
        for (view in shown) {
            view.isZOrderedOnTop = onTop
            val bounds = Rect((from - view.left).coerceAtLeast(0), 0, (to - view.left).coerceAtMost(view.width), view.height)
            view.clipBounds = bounds
            view.visibility = if (bounds.isEmpty) View.INVISIBLE else View.VISIBLE
        }
    }
}
