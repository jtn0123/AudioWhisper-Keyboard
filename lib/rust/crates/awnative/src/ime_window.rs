//! What the keyboard's views must be told when Android shows the input method window again.
//!
//! Android's InputMethodService hides the keyboard in two steps. First (hideWindow) it tells the
//! input view, not the window, that the window is gone: every SurfaceView inside gives up its
//! surface. Half a second later (removeImeSurface) it really hides the window. Showing the
//! keyboard (showWindow) only shows the window, and a window that was never really hidden tells
//! its views nothing. So a keyboard shown again within that half second was left with views that
//! believed their window gone: the SurfaceView the keys are drawn on never asked for a surface,
//! and the keyboard showed its background and no keys until the next slow hide.
//!
//! The rule: when the window is shown, a view last told anything but visible, in a window that is
//! visible, is told visible again. A window that really was hidden is not visible yet when it is
//! shown; Android tells its views itself once it is, and they are left alone.

/// Android's View.VISIBLE.
const VISIBLE: i32 = 0;

/// When the input method window is shown: whether the input view must be told again that its
/// window is visible. `told` is the window visibility the view was last given (View.VISIBLE,
/// INVISIBLE or GONE); `window` is the window's own, View.getWindowVisibility().
pub fn retell_visible(told: i32, window: i32) -> bool {
    window == VISIBLE && told != VISIBLE
}

#[cfg(test)]
mod tests {
    use super::retell_visible;

    const VISIBLE: i32 = 0;
    const INVISIBLE: i32 = 4;
    const GONE: i32 = 8;

    /// Hidden and shown again inside the half second: hideWindow told the view GONE, and the
    /// window, never hidden, is still visible. This is the case that drew no keys.
    #[test]
    fn a_quick_show_retells_the_view_its_window_is_visible() {
        assert!(retell_visible(GONE, VISIBLE));
        assert!(retell_visible(INVISIBLE, VISIBLE));
    }

    /// Hidden for longer: the window really was hidden, so it is not visible yet when shown, and
    /// Android tells the views itself once it is.
    #[test]
    fn a_window_that_really_was_hidden_is_left_to_android() {
        assert!(!retell_visible(GONE, GONE));
        assert!(!retell_visible(GONE, INVISIBLE));
    }

    /// A view already told its window is visible (the first show, or Android got there first)
    /// hears nothing twice.
    #[test]
    fn a_view_already_told_visible_is_not_told_again() {
        assert!(!retell_visible(VISIBLE, VISIBLE));
        assert!(!retell_visible(VISIBLE, GONE));
    }
}
