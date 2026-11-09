//! The auto-space and phantom-space flags, as `EditorInstance.AutoSpaceState` and
//! `PhantomSpaceState` keep them in an `Int`: active, showing the composing region (a phantom
//! space only), and staying active through the next selection update.
//!
//! A flag set by the keyboard's own typing must survive the selection update that typing causes,
//! so setting it marks it to stay active once; the update after that clears it.

/// Neither flag set.
pub const INACTIVE: i32 = 0;

const IS_ACTIVE: i32 = 0x1;
const SHOW_COMPOSING_REGION: i32 = 0x2;
const STAY_ACTIVE_NEXT_UPDATE: i32 = 0x4;

/// Whether the space is owed (`isActive`).
pub fn is_active(state: i32) -> bool {
    state & IS_ACTIVE != 0
}

/// Whether the word before a phantom space still shows as being composed (`showComposingRegion`).
pub fn shows_composing_region(state: i32) -> bool {
    state & SHOW_COMPOSING_REGION != 0
}

/// An active auto-space (`AutoSpaceState.setActive`).
pub fn auto_space_active(stay_active_next_update: bool) -> i32 {
    IS_ACTIVE
        | if stay_active_next_update {
            STAY_ACTIVE_NEXT_UPDATE
        } else {
            0
        }
}

/// An active phantom space (`PhantomSpaceState.setActive`).
pub fn phantom_space_active(show_composing_region: bool, stay_active_next_update: bool) -> i32 {
    IS_ACTIVE
        | if show_composing_region {
            SHOW_COMPOSING_REGION
        } else {
            0
        }
        | if stay_active_next_update {
            STAY_ACTIVE_NEXT_UPDATE
        } else {
            0
        }
}

/// The state after the editor's selection changed (`setInactiveFromUpdate`): one marked to stay
/// active loses the mark and keeps the rest; any other is cleared.
pub fn after_update(state: i32) -> i32 {
    if state & STAY_ACTIVE_NEXT_UPDATE != 0 {
        state & !STAY_ACTIVE_NEXT_UPDATE
    } else {
        INACTIVE
    }
}

/// Whether a phantom space in `state` before a selection update keeps the suggestion it came
/// from, to take back: only if it was marked to stay active.
pub fn keeps_candidate(state: i32) -> bool {
    state & STAY_ACTIVE_NEXT_UPDATE != 0
}

#[cfg(test)]
mod tests;
