//! JNI entry points for the shift key's states, double taps and the double-space full stop,
//! called by `ShiftNative` in lib/rust's Kotlin. States, events, behaviours and caps modes cross
//! as their ordinals; the facts an event reads cross as the bits `ShiftRules` packs.

use jni_sys::{JNI_FALSE, JNIEnv, jboolean, jclass, jint, jlong, jstring};
use shift::{Behavior, CapsMode, Event, Facts, State};

use crate::jni::{self, Result};

/// The fact bits, as `ShiftRules` packs them.
const SHIFT_PRESSED: jint = 1;
const ANY_PRESSED: jint = 2;
const CONSECUTIVE_DOWN: jint = 4;
const UNINTERRUPTED: jint = 8;
const AUTO_CAPITALIZATION: jint = 16;
const LOCALE_CAPITALIZES: jint = 32;

fn state(number: jint) -> Result<State> {
    match number {
        0 => Ok(State::Unshifted),
        1 => Ok(State::ShiftedManual),
        2 => Ok(State::ShiftedAutomatic),
        3 => Ok(State::CapsLock),
        _ => Err(jni::illegal("state")),
    }
}

fn state_number(state: State) -> jint {
    match state {
        State::Unshifted => 0,
        State::ShiftedManual => 1,
        State::ShiftedAutomatic => 2,
        State::CapsLock => 3,
    }
}

fn event(number: jint) -> Result<Event> {
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

fn behavior(number: jint) -> Result<Behavior> {
    match number {
        0 => Ok(Behavior::DoubleTap),
        1 => Ok(Behavior::Cycle),
        _ => Err(jni::illegal("behavior")),
    }
}

fn caps_mode(number: jint) -> Result<CapsMode> {
    match number {
        0 => Ok(CapsMode::None),
        1 => Ok(CapsMode::All),
        2 => Ok(CapsMode::Words),
        3 => Ok(CapsMode::Sentences),
        _ => Err(jni::illegal("caps mode")),
    }
}

fn facts(behavior_number: jint, caps_mode_number: jint, bits: jint) -> Result<Facts> {
    let has = |bit: jint| bits & bit != 0;
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
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ShiftNative_next(
    env: *mut JNIEnv,
    _: jclass,
    state_number_in: jint,
    event_number: jint,
    behavior_number: jint,
    caps_mode_number: jint,
    bits: jint,
) -> jint {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |_| {
            let facts = facts(behavior_number, caps_mode_number, bits)?;
            let next = shift::next(state(state_number_in)?, event(event_number)?, &facts);
            Ok(state_number(next))
        })
    }
}

/// Whether a key event follows the last within the timeout (`shift::is_consecutive`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ShiftNative_isConsecutive(
    _: *mut JNIEnv,
    _: jclass,
    last_code: jint,
    last_time: jlong,
    code: jint,
    now: jlong,
    timeout: jlong,
) -> jboolean {
    jni::boolean(shift::is_consecutive(
        last_code, last_time, code, now, timeout,
    ))
}

/// Whether a second space after `text` types a full stop (`shift::is_double_space_period`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_ShiftNative_isDoubleSpacePeriod(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, JNI_FALSE, |env| {
            let text = env.read_string(text, "text")?;
            Ok(jni::boolean(shift::is_double_space_period(&text)))
        })
    }
}
