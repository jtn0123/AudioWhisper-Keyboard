//! A reading's steps (`reader::reading`), which window it reads (`reader::window`), and how long
//! the neural voice stays loaded (`reader::lifetime`) for `ReadingNative`, whose generated entry
//! points call these.

use std::sync::{Mutex, MutexGuard};

use reader::lifetime::{self, VoiceHold};
use reader::page_text::Edges;
use reader::reading::{Effect, Reading, Screen};
use reader::window;
use wire::reading::fact;

use crate::handle::{free as free_handle, into_handle, locked};
use crate::jni::{Result, illegal};
use crate::reader::read_edges;

/// A reading not started yet.
pub fn create() -> i64 {
    into_handle(Mutex::new(Reading::new()))
}

pub fn free(handle: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe { free_handle::<Mutex<Reading>>(handle) };
    Ok(())
}

fn reading<'a>(handle: i64) -> Result<MutexGuard<'a, Reading>> {
    // SAFETY: Kotlin holds the plan while it calls, so it hasn't been freed.
    unsafe { locked(handle, "reading") }
}

fn row(effect: std::result::Result<Effect, String>) -> Result<Vec<i32>> {
    effect.map(Vec::from).map_err(illegal)
}

/// Starts a reading (`Reading::start`).
pub fn start(handle: i64, from: i32, selection: Option<&[u16]>) -> Result<Vec<i32>> {
    let selection = selection.map(<[u16]>::to_vec).unwrap_or_default();
    row(reading(handle)?.start(from, selection))
}

/// `Reading::done`.
pub fn done(handle: i64) -> Result<Vec<i32>> {
    row(reading(handle)?.done())
}

/// `Reading::answer`.
pub fn answer(handle: i64, value: i32) -> Result<Vec<i32>> {
    row(reading(handle)?.answer(value))
}

/// A walk found a screen (`Reading::looked`).
pub fn looked(
    handle: i64,
    app: Option<&[u16]>,
    texts: &[Vec<u16>],
    edges: &[i32],
    scrolls: &[u8],
    facts: &[i32],
) -> Result<Vec<i32>> {
    let edges = read_edges(edges)?;
    if edges.len() != texts.len() || scrolls.len() != texts.len() {
        return Err(illegal("not as many edges and flags as texts"));
    }
    let fact = |at: i32| facts.get(at as usize).copied();
    let [
        Some(top),
        Some(bottom),
        Some(nodes),
        Some(web),
        Some(scroller),
    ] = [
        fact::VISIBLE_TOP,
        fact::VISIBLE_BOTTOM,
        fact::NODES,
        fact::WEB,
        fact::SCROLLER,
    ]
    .map(fact)
    else {
        return Err(illegal("a short facts row"));
    };
    let screen = Screen {
        app: app.map(<[u16]>::to_vec),
        texts: texts.to_vec(),
        edges,
        scrolls: scrolls.iter().map(|&it| it != 0).collect(),
        visible: Edges { top, bottom },
        nodes,
        web: web != 0,
        scroller: scroller != 0,
    };
    row(reading(handle)?.looked(Some(screen)))
}

/// A walk found no window (`Reading::looked`).
pub fn looked_nothing(handle: i64) -> Result<Vec<i32>> {
    row(reading(handle)?.looked(None))
}

fn flags(values: &[u8]) -> Vec<bool> {
    values.iter().map(|&it| it != 0).collect()
}

/// The app's window and the keyboard's (`window::choose`), -1 for none.
pub fn window(
    application: &[u8],
    input_method: &[u8],
    active: &[u8],
    focused: &[u8],
    layers: &[i32],
) -> Result<Vec<i32>> {
    let lengths = [
        input_method.len(),
        active.len(),
        focused.len(),
        layers.len(),
    ];
    if lengths.iter().any(|&n| n != application.len()) {
        return Err(illegal("windows' facts of different lengths"));
    }
    let (app, keyboard) = window::choose(
        &flags(application),
        &flags(input_method),
        &flags(active),
        &flags(focused),
        layers,
    );
    Ok([app, keyboard]
        .map(|it| it.map_or(-1, |i| i as i32))
        .to_vec())
}

/// `window::visible_bottom`.
pub fn visible_bottom(top: i32, bottom: i32, keyboard_top: i32) -> Result<i32> {
    Ok(window::visible_bottom(top, bottom, keyboard_top))
}

/// The neural voice's hold, not taken yet.
pub fn hold_create() -> i64 {
    into_handle(Mutex::new(VoiceHold::default()))
}

pub fn hold_free(handle: i64) -> Result<()> {
    // SAFETY: the synchronized wrapper frees its handle once, then sets it to zero.
    unsafe { free_handle::<Mutex<VoiceHold>>(handle) };
    Ok(())
}

fn hold<'a>(handle: i64) -> Result<MutexGuard<'a, VoiceHold>> {
    // SAFETY: Kotlin holds the hold while it calls, so it hasn't been freed.
    unsafe { locked(handle, "voice hold") }
}

/// `VoiceHold::take`.
pub fn hold_take(handle: i64, loaded: bool) -> Result<i32> {
    Ok(hold(handle)?.take(loaded))
}

/// `VoiceHold::checked`.
pub fn hold_checked(handle: i64, missing: bool) -> Result<i32> {
    Ok(hold(handle)?.checked(missing))
}

/// `VoiceHold::loaded`.
pub fn hold_loaded(handle: i64, ok: bool) -> Result<()> {
    hold(handle)?.loaded(ok);
    Ok(())
}

/// `VoiceHold::failed_just_now`.
pub fn hold_failed_just_now(handle: i64) -> Result<bool> {
    Ok(hold(handle)?.failed_just_now())
}

/// `VoiceHold::idle`.
pub fn hold_idle(handle: i64) -> Result<()> {
    hold(handle)?.idle();
    Ok(())
}

/// `VoiceHold::may_free`.
pub fn hold_may_free(handle: i64) -> Result<bool> {
    Ok(hold(handle)?.may_free())
}

/// `lifetime::route`.
pub fn route(service_up: bool, turned_on: bool) -> Result<i32> {
    Ok(lifetime::route(service_up, turned_on))
}

/// `lifetime::still_asked`.
pub fn still_asked(asked_at: i64, now: i64) -> Result<bool> {
    Ok(lifetime::still_asked(asked_at, now))
}

/// The lifetimes Kotlin times: the neural voice's idle wait.
pub fn lifetimes() -> Result<Vec<i64>> {
    Ok(vec![lifetime::VOICE_IDLE_MS])
}
