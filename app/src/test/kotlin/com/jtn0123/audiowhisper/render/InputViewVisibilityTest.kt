package com.jtn0123.audiowhisper.render

import android.view.View
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/**
 * Replays what Android 16's InputMethodService tells the keyboard's input view as the keyboard is
 * hidden and shown (hideWindow, removeImeSurface, showWindow), and what the view is left believing.
 */
class InputViewVisibilityTest {
    /** The input view, as its SurfaceViews see it: the last window visibility passed down. */
    private class InputView {
        var believed = View.GONE
        val visibility = InputViewVisibility { tell(it) }
        fun tell(visibility: Int) { believed = visibility; this.visibility.told(visibility) }
    }

    @Test
    fun showingAgainWithinHalfASecondLeavesTheKeyboardsSurfaceVisible() {
        val view = InputView()
        view.tell(View.VISIBLE) // The first show: the window became visible.
        view.tell(View.GONE) // hideWindow: the view alone is told; the window stays up 500 ms.
        // showWindow before removeImeSurface: the window was never hidden, so Android tells the
        // view nothing. This is the moment the keys went missing.
        assertTrue(view.visibility.shown(View.VISIBLE))
        assertEquals(View.VISIBLE, view.believed)
    }

    @Test
    fun showingAfterTheWindowReallyHidLeavesItToAndroid() {
        val view = InputView()
        view.tell(View.VISIBLE)
        view.tell(View.GONE) // hideWindow.
        view.tell(View.GONE) // removeImeSurface hid the window, and Android said so.
        // showWindow: the window is shown but not yet laid out, so it is not visible yet.
        assertFalse(view.visibility.shown(View.GONE))
        assertEquals(View.GONE, view.believed)
        view.tell(View.VISIBLE) // The next traversal tells the view itself.
        assertEquals(View.VISIBLE, view.believed)
    }

    @Test
    fun aViewAlreadyToldVisibleIsNotToldTwice() {
        var retold = 0
        val visibility = InputViewVisibility { retold++ }
        visibility.told(View.VISIBLE)
        assertFalse(visibility.shown(View.VISIBLE))
        assertEquals(0, retold)
    }
}
