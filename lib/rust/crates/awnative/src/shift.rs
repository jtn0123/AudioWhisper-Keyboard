//! The shift key's states, double taps and the double-space full stop, for `ShiftNative` in
//! lib/rust's Kotlin. Its entry points are generated from crates/boundary's declaration
//! (src/declarations/shift.rs there) and call `next` here, or the shift crate directly. States,
//! events, behaviours and caps modes cross as their ordinals.

use shift::{Behavior, CapsMode, Event, Facts, State};

use crate::boundary::args::ShiftNativeNext;
use crate::jni::{self, Result};

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

/// The state `event` leaves `state` in (`shift::next`).
pub fn next(args: ShiftNativeNext) -> Result<i32> {
    let facts = Facts {
        behavior: behavior(args.behavior)?,
        caps_mode: caps_mode(args.caps_mode)?,
        shift_pressed: args.shift_pressed,
        any_pressed: args.any_pressed,
        consecutive_down: args.consecutive_down,
        uninterrupted: args.uninterrupted,
        auto_capitalization: args.auto_capitalization,
        locale_capitalizes: args.locale_capitalizes,
    };
    let next = shift::next(state(args.state)?, event(args.event)?, &facts);
    Ok(state_number(next))
}
