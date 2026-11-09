//! The decisions a walk of the screen and following along make, as Kotlin's `ScreenRules` had
//! them: whether a scrollable view goes sideways, how much of a box shows, which of the screen's
//! lists is the one that scrolls and what is inside it, and how to bring a web page's piece of
//! text into view.

use crate::follow_along::fits;

/// A rectangle on the screen, in pixels, as Android's `Rect` has it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Bounds {
    /// Where `self` and `other` overlap, as `Rect.intersect` finds it: a box with no height or
    /// width overlaps a box it is inside, and boxes that only touch don't.
    pub fn intersect(self, other: Bounds) -> Option<Bounds> {
        let overlaps = self.left < other.right
            && other.left < self.right
            && self.top < other.bottom
            && other.top < self.bottom;
        overlaps.then(|| Bounds {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        })
    }

    /// How much of `self` shows in `visible`, in square pixels. Kotlin's `Int` sides wrap.
    pub fn visible_area(self, visible: Bounds) -> i64 {
        self.intersect(visible).map_or(0, |it| {
            let width = i64::from(it.right.wrapping_sub(it.left));
            width.wrapping_mul(i64::from(it.bottom.wrapping_sub(it.top)))
        })
    }

    /// The part of `self` that shows in `visible`, or all of `visible` if none does.
    pub fn visible_part(self, visible: Bounds) -> Bounds {
        self.intersect(visible).unwrap_or(visible)
    }
}

/// Whether a scrollable view goes sideways (a carousel or a pager), so it is never the list: a
/// collection of one row and several `columns`, a view that only scrolls sideways (`across`, not
/// `up_down`), or a horizontal scroll view or pager that can't go up and down. `rows` and
/// `columns` are 0 for a view that isn't a collection; `class_name` is the view's class.
pub fn is_sideways(
    rows: i32,
    columns: i32,
    up_down: bool,
    across: bool,
    class_name: &[u16],
) -> bool {
    if rows == 1 && columns > 1 {
        return true;
    }
    if !up_down && across {
        return true;
    }
    let horizontal: Vec<u16> = "HorizontalScrollView".encode_utf16().collect();
    let pager: Vec<u16> = "ViewPager".encode_utf16().collect();
    !up_down
        && (class_name.ends_with(&horizontal)
            || class_name.windows(pager.len()).any(|it| it == pager))
}

/// The list, among scrollers that show `areas` square pixels, given which scroller each piece of
/// text is directly in (`piece_scrollers`; `None` for none): the one in sight that holds the most
/// text itself, the bigger on a tie, the first of equals. Not simply the biggest: a settings page
/// wraps its list in a scroll view whose only travel is folding the title away. Each scroller a
/// piece names must be one of `areas`'.
pub fn main_scroller(piece_scrollers: &[Option<usize>], areas: &[i64]) -> Option<usize> {
    let mut holds = vec![0u32; areas.len()];
    for &scroller in piece_scrollers.iter().flatten() {
        holds[scroller] += 1;
    }
    let mut best: Option<usize> = None;
    for (index, &area) in areas.iter().enumerate().filter(|&(_, &area)| area > 0) {
        let better = best.is_none_or(|it| (holds[index], area) > (holds[it], areas[it]));
        if better {
            best = Some(index);
        }
    }
    best
}

/// Whether `scroller` is `main` or inside it, given each scroller's `parents`. A scroller's
/// parent comes before it, so the walk up ends.
pub fn is_within(parents: &[Option<usize>], scroller: Option<usize>, main: Option<usize>) -> bool {
    let Some(main) = main else {
        return false;
    };
    let mut at = scroller;
    while let Some(index) = at {
        if index == main {
            return true;
        }
        at = parents[index];
    }
    false
}

/// How to bring a web page's piece into view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// It fits.
    Stay,
    /// "Show on screen", which knows where the keyboard is.
    ShowOnScreen,
    /// Turn the page a scroll forward at a time.
    TurnPages,
}

/// How to bring a web page's piece spanning `first` to `last` into a view from `top` to
/// `bottom`: it stays if it fits; it is shown on screen if it is above the view or there is no
/// list to turn (`can_turn`); otherwise the page is turned.
pub fn move_into_view(first: i32, last: i32, top: i32, bottom: i32, can_turn: bool) -> Move {
    if fits(first, last, top, bottom) {
        Move::Stay
    } else if first < top || !can_turn {
        Move::ShowOnScreen
    } else {
        Move::TurnPages
    }
}

/// What to do once the page has turned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AfterTurn {
    ShowOnScreen,
    Done,
    TurnAgain,
}

/// Turned past it, the piece is shown on screen; once it fits, turning stops; else the page
/// turns again.
pub fn after_turn(first: i32, last: i32, top: i32, bottom: i32) -> AfterTurn {
    if first < top {
        AfterTurn::ShowOnScreen
    } else if fits(first, last, top, bottom) {
        AfterTurn::Done
    } else {
        AfterTurn::TurnAgain
    }
}

/// Whether an app's own view from `top` to `bottom` is well inside a view from `view_top` to
/// `view_bottom`, so it is left alone. Kotlin's `Int` height wraps.
pub fn is_inside(top: i32, bottom: i32, view_top: i32, view_bottom: i32) -> bool {
    bottom.wrapping_sub(top) > 0 && top > view_top && bottom < view_bottom
}

#[cfg(test)]
mod tests;
