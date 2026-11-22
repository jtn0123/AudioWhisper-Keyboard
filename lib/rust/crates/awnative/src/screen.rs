//! The decisions a walk of the screen and following along make (the `reader` crate's `screen`),
//! for `ScreenWalkRules`' generated entry points. A scroller is its index in the walk's list of
//! them, and -1 is none. Two boxes cross as one `int[8]`: a box's left, top, right and bottom,
//! then the visible box's.

use reader::screen::{self, AfterTurn, Bounds, Move};

use crate::jni::{Result, boolean, illegal};

/// No scroller.
const NONE: i32 = -1;

/// The scroller `value` names among `count`: `None` for -1, else one of them.
fn scroller(value: i32, count: usize, name: &str) -> Result<Option<usize>> {
    if value == NONE {
        return Ok(None);
    }
    (usize::try_from(value).ok().filter(|&it| it < count))
        .map(Some)
        .ok_or_else(|| illegal(format!("{name} {value} is not one of {count} scrollers")))
}

/// The scrollers `values` name among `count`.
fn scrollers(values: &[i32], count: usize, name: &str) -> Result<Vec<Option<usize>>> {
    values.iter().map(|&it| scroller(it, count, name)).collect()
}

/// Each scroller's parent, which must come before it, so a walk up its parents ends.
fn parents(values: &[i32]) -> Result<Vec<Option<usize>>> {
    (values.iter().enumerate())
        .map(|(index, &it)| scroller(it, index, "a parent"))
        .collect()
}

fn index(value: Option<usize>) -> i32 {
    value.map_or(NONE, |it| it as i32)
}

/// A box and the visible one, from `boxes`.
fn read_boxes(values: &[i32]) -> Result<(Bounds, Bounds)> {
    let [left, top, right, bottom, v_left, v_top, v_right, v_bottom] = values[..] else {
        return Err(illegal(format!("{} sides, not 8", values.len())));
    };
    let bounds = |left, top, right, bottom| Bounds {
        left,
        top,
        right,
        bottom,
    };
    Ok((
        bounds(left, top, right, bottom),
        bounds(v_left, v_top, v_right, v_bottom),
    ))
}

/// How much of the first box shows in the second, in square pixels (`Bounds::visible_area`).
pub fn visible_area(boxes: &[i32]) -> Result<i64> {
    let (bounds, visible) = read_boxes(boxes)?;
    Ok(bounds.visible_area(visible))
}

/// The part of the first box that shows in the second, or all of the second
/// (`Bounds::visible_part`): its left, top, right and bottom.
pub fn visible_part(boxes: &[i32]) -> Result<Vec<i32>> {
    let (bounds, visible) = read_boxes(boxes)?;
    let part = bounds.visible_part(visible);
    Ok(vec![part.left, part.top, part.right, part.bottom])
}

/// The list among scrollers that show `areas`, given the scroller each piece of text is directly
/// in (`screen::main_scroller`), or -1.
pub fn main_scroller(values: &[i32], areas: &[i64]) -> Result<i32> {
    let pieces = scrollers(values, areas.len(), "a piece's scroller")?;
    Ok(index(screen::main_scroller(&pieces, areas)))
}

/// For each piece of text, whether the scroller it is directly in is `main` or inside it
/// (`screen::is_within`).
pub fn in_list(values: &[i32], parents_of: &[i32], main: i32) -> Result<Vec<u8>> {
    let parents = parents(parents_of)?;
    let main = scroller(main, parents.len(), "main")?;
    let pieces = scrollers(values, parents.len(), "a piece's scroller")?;
    Ok((pieces.iter())
        .map(|&it| boolean(screen::is_within(&parents, it, main)))
        .collect())
}

/// How to bring a web page's piece into view (`screen::move_into_view`), as `Move`'s order in
/// lib/rust's Kotlin has it.
pub fn move_into_view(first: i32, last: i32, top: i32, bottom: i32, can_turn: bool) -> i32 {
    match screen::move_into_view(first, last, top, bottom, can_turn) {
        Move::Stay => 0,
        Move::ShowOnScreen => 1,
        Move::TurnPages => 2,
    }
}

/// What to do once a web page has turned (`screen::after_turn`), as `AfterTurn`'s order in
/// lib/rust's Kotlin has it.
pub fn after_turn(first: i32, last: i32, top: i32, bottom: i32) -> i32 {
    match screen::after_turn(first, last, top, bottom) {
        AfterTurn::ShowOnScreen => 0,
        AfterTurn::Done => 1,
        AfterTurn::TurnAgain => 2,
    }
}
