//! The letter odds the suggester last worked out (`suggest::letter_odds`), kept for both hit
//! tests: a key going down near a border leans on them (`keys::resize`) while they still hold for
//! the word being typed (`TapTrail::at_word`). Suggesting runs off the main thread a keystroke
//! behind at times; the trail says when the odds are stale, and then nothing leans on them.
//!
//! Only the word being typed is kept, as the trail keeps it. Nothing here is logged.

use std::sync::{Mutex, PoisonError};

use suggest::letter_odds::Latest;
use suggest::session::Session;
use suggest::suggester::TypingState;
use suggest::tap::TapTrail;

static LATEST: Mutex<Option<Latest>> = Mutex::new(None);

/// The suggester is suggesting for `state`: the odds of each letter coming next, if the model
/// knows the word so far. A cursor inside a word has none: what follows it is already there.
pub(crate) fn note(session: &Session, state: &TypingState) {
    let odds = (!state.is_cursor_mid_word)
        .then(|| session.language_model())
        .flatten()
        .and_then(|model| model.letter_odds(&state.previous_words, &state.typed));
    let latest = odds.map(|odds| Latest::new(state.typed.clone(), odds));
    *LATEST.lock().unwrap_or_else(PoisonError::into_inner) = latest;
}

/// The prior of each key, by its code, for a key going down on `trail` (see
/// `LetterOdds::priors`); empty when the odds don't hold.
pub(crate) fn priors(trail: &TapTrail, codes: impl IntoIterator<Item = i32>) -> Vec<Option<f32>> {
    let latest = LATEST.lock().unwrap_or_else(PoisonError::into_inner);
    latest
        .as_ref()
        .and_then(|latest| latest.at(trail))
        .map_or_else(Vec::new, |odds| odds.priors(codes))
}
