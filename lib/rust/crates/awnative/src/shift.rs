//! The shift key's states, double taps and the double-space full stop, for `ShiftNative` in
//! lib/rust's Kotlin. Its entry points are generated from crates/boundary's declaration
//! (src/declarations/shift.rs there) and call the functions here. States, events, behaviours and
//! caps modes cross as their ordinals; the facts an event reads cross as the bits `ShiftRules`
//! packs.

use shift::{Behavior, CapsMode, Event, Facts, State};

use crate::jni::{self, Result};

/// The fact bits, as `ShiftRules` packs them.
const SHIFT_PRESSED: i32 = 1;
const ANY_PRESSED: i32 = 2;
const CONSECUTIVE_DOWN: i32 = 4;
const UNINTERRUPTED: i32 = 8;
const AUTO_CAPITALIZATION: i32 = 16;
const LOCALE_CAPITALIZES: i32 = 32;

fn state(number: i32) -> Result<State> {
    match number {
        0 => Ok(State::Unshifted),
        1 => Ok(State::ShiftedManual),
        2 => Ok(State::ShiftedAutomatic),
        3 => Ok(State::CapsLock),
        _ => Err(jni::illegal("state")),
    }
}

fn state_number(state: State) -> i32 {
    match state {
        State::Unshifted => 0,
        State::ShiftedManual => 1,
        State::ShiftedAutomatic => 2,
        State::CapsLock => 3,
    }
}

fn event(number: i32) -> Result<Event> {
    match number {
        0 => Ok(Event::Reevaluate),
        1 => Ok(Event::ShiftDown),
        2 => Ok(Event::ShiftUp),
        3 => Ok(Event::CapsLock),
        4 => Ok(Event::ShiftCancel),
        5 => Ok(Event::KeyTyped),
        _ => Err(jni::illegal("event")),
    }
}

fn behavior(number: i32) -> Result<Behavior> {
    match number {
        0 => Ok(Behavior::DoubleTap),
        1 => Ok(Behavior::Cycle),
        _ => Err(jni::illegal("behavior")),
    }
}

fn caps_mode(number: i32) -> Result<CapsMode> {
    match number {
        0 => Ok(CapsMode::None),
        1 => Ok(CapsMode::All),
        2 => Ok(CapsMode::Words),
        3 => Ok(CapsMode::Sentences),
        _ => Err(jni::illegal("caps mode")),
    }
}

fn facts(behavior_number: i32, caps_mode_number: i32, bits: i32) -> Result<Facts> {
    let has = |bit: i32| bits & bit != 0;
    Ok(Facts {
        behavior: behavior(behavior_number)?,
        caps_mode: caps_mode(caps_mode_number)?,
        shift_pressed: has(SHIFT_PRESSED),
        any_pressed: has(ANY_PRESSED),
        consecutive_down: has(CONSECUTIVE_DOWN),
        uninterrupted: has(UNINTERRUPTED),
        auto_capitalization: has(AUTO_CAPITALIZATION),
        locale_capitalizes: has(LOCALE_CAPITALIZES),
    })
}

/// The state `event` leaves `state` in (`shift::next`).
pub fn next(
    state_number_in: i32,
    event_number: i32,
    behavior_number: i32,
    caps_mode_number: i32,
    bits: i32,
) -> Result<i32> {
    let facts = facts(behavior_number, caps_mode_number, bits)?;
    let next = shift::next(state(state_number_in)?, event(event_number)?, &facts);
    Ok(state_number(next))
}

/// Whether a key event follows the last within the timeout (`shift::is_consecutive`).
pub fn is_consecutive(
    last_code: i32,
    last_time: i64,
    code: i32,
    now: i64,
    timeout: i64,
) -> Result<bool> {
    Ok(shift::is_consecutive(
        last_code, last_time, code, now, timeout,
    ))
}

/// Whether a second space after `text` types a full stop (`shift::is_double_space_period`).
pub fn is_double_space_period(text: &[u16]) -> Result<bool> {
    Ok(shift::is_double_space_period(text))
}
