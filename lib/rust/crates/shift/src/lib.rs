//! The shift key and auto-capitalisation, as FlorisBoard's `KeyboardManager` had them: which
//! shift state each shift press, release, cancel, typed key and cursor move leaves the keyboard
//! in, whether a key's press or release came soon enough after the last to count as a double tap
//! (`InputEventDispatcher.isConsecutiveDown` and `isConsecutiveUp`), and whether a second space
//! should become a full stop.
//!
//! Kotlin still asks the dispatcher which keys are down and the editor where the cursor is, and
//! changes a word's case for the subtype's locale (`fixCase`), which needs locale case tables
//! (Turkish, Azeri, Lithuanian) that `chars` doesn't have.

#![forbid(unsafe_code)]

/// The keyboard's shift state, numbered as `InputShiftState` orders it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Unshifted,
    /// Shifted for one key by the shift key.
    ShiftedManual,
    /// Shifted for one key by auto-capitalisation.
    ShiftedAutomatic,
    CapsLock,
}

/// How the shift key reaches caps lock, numbered as `CapitalizationBehavior` orders it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Behavior {
    /// A second press soon after the first locks; any other press toggles.
    DoubleTap,
    /// Each press steps unshifted, shifted, locked, unshifted.
    Cycle,
}

/// What the editor asks to capitalise at the cursor, numbered as `InputAttributes.CapsMode`
/// orders it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapsMode {
    None,
    All,
    Words,
    Sentences,
}

/// What happened, numbered as `ShiftRules.Event` orders it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// The cursor or the editor changed (`reevaluateInputShiftState`).
    Reevaluate,
    ShiftDown,
    ShiftUp,
    CapsLock,
    ShiftCancel,
    /// A key that types was released: shift, unless locked or held, lasts one key.
    KeyTyped,
}

/// What Kotlin knew when the event came. Each event reads only some of them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Facts {
    pub behavior: Behavior,
    pub caps_mode: CapsMode,
    /// The shift key is down.
    pub shift_pressed: bool,
    /// Some key is down.
    pub any_pressed: bool,
    /// This shift press came soon after the last one (`isConsecutiveDown`).
    pub consecutive_down: bool,
    /// The last key to go down was shift (`isUninterruptedEventSequence`).
    pub uninterrupted: bool,
    /// The user turned auto-capitalisation on.
    pub auto_capitalization: bool,
    /// The subtype's language has capitals.
    pub locale_capitalizes: bool,
}

/// The shift state `event` leaves `state` in.
pub fn next(state: State, event: Event, facts: &Facts) -> State {
    match event {
        Event::Reevaluate => reevaluate(state, facts),
        Event::ShiftDown => shift_down(state, facts),
        Event::ShiftUp => {
            // A shift held while typing other keys shifted only those: let go, it ends.
            if state != State::CapsLock && !facts.any_pressed && !facts.uninterrupted {
                State::Unshifted
            } else {
                state
            }
        }
        Event::CapsLock => State::CapsLock,
        Event::ShiftCancel => State::Unshifted,
        Event::KeyTyped => {
            if state != State::CapsLock && !facts.shift_pressed {
                State::Unshifted
            } else {
                state
            }
        }
    }
}

/// Unless locked or held, shifted automatically where the editor capitalises, if the user and
/// the language want it, and otherwise not.
fn reevaluate(state: State, facts: &Facts) -> State {
    if state == State::CapsLock || facts.shift_pressed {
        return state;
    }
    let shift =
        facts.auto_capitalization && facts.locale_capitalizes && facts.caps_mode != CapsMode::None;
    if shift {
        State::ShiftedAutomatic
    } else {
        State::Unshifted
    }
}

fn shift_down(state: State, facts: &Facts) -> State {
    match facts.behavior {
        Behavior::DoubleTap => {
            if facts.consecutive_down {
                State::CapsLock
            } else if state == State::Unshifted {
                State::ShiftedManual
            } else {
                State::Unshifted
            }
        }
        Behavior::Cycle => match state {
            State::Unshifted => State::ShiftedManual,
            State::ShiftedManual => State::CapsLock,
            State::ShiftedAutomatic | State::CapsLock => State::Unshifted,
        },
    }
}

/// Whether a key `code` at `now` follows the last such event, of `last_code` at `last_time`,
/// within `timeout` milliseconds. Kotlin's `Long` difference wraps.
pub fn is_consecutive(last_code: i32, last_time: i64, code: i32, now: i64, timeout: i64) -> bool {
    last_code == code && now.wrapping_sub(last_time) < timeout
}

const FULL_STOP: u16 = b'.' as u16;
const EXCLAMATION: u16 = b'!' as u16;
const QUESTION: u16 = b'?' as u16;
const INTERROBANG: u16 = 0x203D;

/// Whether the two characters before the cursor, `text`, are a space after something that ends
/// no sentence, so that a second space types ". ": Kotlin's
/// `text.length == 2 && Regex("""([^.!?‽\s]\s)""").matches(text)`.
///
/// A regex reads a surrogate pair as one character, which leaves nothing for `\s`; but `\s`
/// never matches a low surrogate, so judging each unit alone gives the same answer. `\s` is the
/// regex's (`chars::is_regex_space`): ASCII whitespace on the JVM, all of Unicode's White_Space on
/// Android.
pub fn is_double_space_period(text: &[u16]) -> bool {
    let &[before, space] = text else {
        return false;
    };
    !matches!(before, FULL_STOP | EXCLAMATION | QUESTION | INTERROBANG)
        && !chars::is_regex_space(before)
        && chars::is_regex_space(space)
}

#[cfg(test)]
mod tests;
