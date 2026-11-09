package com.jtn0123.audiowhisper.reader

import android.graphics.Rect
import android.view.accessibility.AccessibilityNodeInfo
import android.view.accessibility.AccessibilityNodeInfo.AccessibilityAction
import com.jtn0123.audiowhisper.rust.ReaderRules
import kotlinx.coroutines.delay

/**
 * Keeps the text being read in sight, inside [view]: the part of the list that scrolls the app
 * shows above the keyboard.
 *
 * A web page has its whole text in the tree, with where each piece really is on the page in its
 * extras; Chrome's own "show on screen" counts the part behind the keyboard as shown, so the page
 * is turned instead, a scroll forward at a time, until the block about to be read fits. An app's
 * own views are moved with "show on screen", which knows where the keyboard is.
 */
internal class FollowAlong(private val scroller: AccessibilityNodeInfo?, private val view: Rect) {
    suspend fun show(node: AccessibilityNodeInfo) {
        if (!node.refresh()) return
        val span = span(node) ?: return showOwnView(node)
        when (ScreenRules.move(span.first, span.last, view.top, view.bottom, canTurn = scroller != null)) {
            ScreenRules.Move.STAY -> Unit
            ScreenRules.Move.SHOW_ON_SCREEN -> node.performAction(SHOW_ON_SCREEN)
            ScreenRules.Move.TURN_PAGES -> turnPages(node, checkNotNull(scroller))
        }
    }

    private suspend fun turnPages(node: AccessibilityNodeInfo, scroller: AccessibilityNodeInfo) {
        repeat(MAX_TURNS) {
            if (!scroller.performAction(AccessibilityAction.ACTION_SCROLL_FORWARD.id)) return
            delay(TURN_MS)
            val span = node.takeIf { it.refresh() }?.let(::span) ?: return
            when (ScreenRules.afterTurn(span.first, span.last, view.top, view.bottom)) {
                ScreenRules.AfterTurn.SHOW_ON_SCREEN -> {
                    node.performAction(SHOW_ON_SCREEN)
                    return
                }
                ScreenRules.AfterTurn.DONE -> return
                ScreenRules.AfterTurn.TURN_AGAIN -> Unit
            }
        }
    }

    /** A view cut off by an edge, or gone past it, is moved back; one well inside is left alone. */
    private fun showOwnView(node: AccessibilityNodeInfo) {
        val bounds = Rect().also(node::getBoundsInScreen)
        if (!ScreenRules.isInside(bounds.top, bounds.bottom, view.top, view.bottom)) node.performAction(SHOW_ON_SCREEN)
    }

    internal companion object {
        private const val SHOW_ON_SCREEN = android.R.id.accessibilityActionShowOnScreen
        private const val UNCLIPPED_TOP = "AccessibilityNodeInfo.unclippedTop"
        private const val UNCLIPPED_BOTTOM = "AccessibilityNodeInfo.unclippedBottom"

        /** A forward scroll moves a web page about half a screen; this covers a long way down. */
        private const val MAX_TURNS = 8
        private const val TURN_MS = 200L

        /**
         * A piece spanning [span] fits in a view from [top] to [bottom] when all of it shows, or, if
         * it is taller than the view, once it starts in the top half.
         */
        fun fits(span: IntRange, top: Int, bottom: Int): Boolean = ReaderRules.fits(span.first, span.last, top, bottom)

        /** Where a web page's piece really is, out of sight too; null for an app's own views. */
        private fun span(node: AccessibilityNodeInfo): IntRange? {
            val extras = node.extras
            if (!extras.containsKey(UNCLIPPED_TOP) || !extras.containsKey(UNCLIPPED_BOTTOM)) return null
            return extras.getInt(UNCLIPPED_TOP)..extras.getInt(UNCLIPPED_BOTTOM)
        }
    }
}
