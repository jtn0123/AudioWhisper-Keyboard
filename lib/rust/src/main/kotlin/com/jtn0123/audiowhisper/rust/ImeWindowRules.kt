package com.jtn0123.audiowhisper.rust

/** What the keyboard's views are told as Android shows the input method window (awnative ime_window.rs). */
object ImeWindowRules {
    init { RustLibrary.load() }

    /**
     * When the window is shown: whether a view last [told] some window visibility must be told
     * again that it is visible, given the [window]'s own (both View.VISIBLE, INVISIBLE or GONE).
     */
    @JvmStatic external fun retellVisible(told: Int, window: Int): Boolean
}
