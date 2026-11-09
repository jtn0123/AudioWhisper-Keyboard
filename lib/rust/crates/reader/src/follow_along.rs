//! Keeping the text being read in sight, as Kotlin's `FollowAlong` decides it.

/// Whether a piece of text from `first` down to `last` (screen pixels, both in it) fits in a view
/// from `top` to `bottom`: all of it shows, or, if it is taller than the view, it starts in the
/// top half.
pub fn fits(first: i32, last: i32, top: i32, bottom: i32) -> bool {
    // Kotlin's Int sum wraps, and its division, as Rust's, rounds toward zero.
    first >= top && (last <= bottom || first < top.wrapping_add(bottom) / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The view the text is read in: the page from 200 down to the keyboard at 1,400.
    fn fits_view(first: i32, last: i32) -> bool {
        fits(first, last, 200, 1_400)
    }

    #[test]
    fn a_paragraph_wholly_in_sight_is_left_where_it_is() {
        assert!(fits_view(400, 600));
        assert!(fits_view(200, 1_400));
    }

    #[test]
    fn a_paragraph_behind_the_keyboard_is_brought_up() {
        assert!(!fits_view(1_300, 1_500));
        assert!(!fits_view(1_600, 1_800));
    }

    #[test]
    fn a_paragraph_above_the_view_is_brought_down() {
        assert!(!fits_view(100, 300));
        assert!(!fits_view(199, 300));
    }

    #[test]
    fn a_paragraph_taller_than_the_view_fits_once_it_starts_in_the_top_half() {
        assert!(fits_view(500, 2_000));
        assert!(fits_view(799, 2_000));
        assert!(!fits_view(800, 2_000));
        assert!(!fits_view(1_000, 2_500));
    }

    #[test]
    fn the_middle_is_found_as_kotlin_finds_it() {
        // (MAX + 1) wraps to MIN in Kotlin; half of it is far above any first line.
        assert!(!fits(0, 10, i32::MAX, 1));
        assert!(fits(i32::MAX, i32::MAX, 1, i32::MAX));
        // -3 / 2 is -1, not -2.
        assert!(!fits(-1, 10, -2, -1));
        assert!(fits(-2, 10, -2, -1));
    }
}
