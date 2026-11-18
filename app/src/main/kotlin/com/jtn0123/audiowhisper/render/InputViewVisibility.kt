package com.jtn0123.audiowhisper.render

import android.view.View
import com.jtn0123.audiowhisper.rust.ImeWindowRules

/**
 * The window visibility Android last gave the keyboard's input view, put right when the input
 * method window is shown. Hiding tells the input view its window is gone; showing again within
 * half a second tells it nothing, and its SurfaceViews, the Rust keyboard's among them, never ask
 * for a surface again (awnative ime_window.rs). [retell] passes a visibility down the view tree.
 */
internal class InputViewVisibility(private val retell: (Int) -> Unit) {
    /** Never told visible until Android says so. */
    private var told = View.GONE

    /** Every window visibility the view is given, by Android or by [shown]. */
    fun told(visibility: Int) { told = visibility }

    /** The input method window is being shown; [window] is the view's getWindowVisibility(). */
    fun shown(window: Int): Boolean {
        val retold = ImeWindowRules.retellVisible(told, window)
        if (retold) retell(View.VISIBLE)
        return retold
    }
}
