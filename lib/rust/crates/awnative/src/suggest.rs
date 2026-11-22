//! The typing suggestions' rules (the `suggest` crate), for `SuggestNative`, whose generated entry
//! points call these, and those of tap.rs and tap_aim.rs.

#[cfg(any(not(target_os = "android"), feature = "legacy-fixture"))]
mod legacy_suggester;
pub(crate) mod letter_odds;
pub(crate) mod native_glide;
pub(crate) mod session;
mod suggester;
pub(crate) mod swipe;
pub(crate) mod tap;
pub(crate) mod tap_aim;
pub(crate) mod word_learner;
pub(crate) mod word_stores;

use std::ptr;

use jni_sys::jobjectArray;
use suggest::learned_words::NegativeCount;
use suggest::raw::RawSuggestion;
use suggest::word_case::{self, WordCase};
use suggest::{auto_correction, previous_words, undo, word_characters};

use crate::jni::{Env, Error, Result, illegal};

/// Kotlin's `take` of a negative count throws this.
impl From<NegativeCount> for Error {
    fn from(NegativeCount(count): NegativeCount) -> Error {
        illegal(format!(
            "Requested element count {count} is less than zero."
        ))
    }
}

/// The cases' numbers, as `SuggestRules.Case` orders them.
const CASES: [WordCase; 3] = [WordCase::AsTyped, WordCase::Capitalized, WordCase::AllCaps];

pub fn is_cursor_mid_word(text_after_cursor: &[u16]) -> bool {
    word_characters::cursor_mid_word(text_after_cursor)
}

/// Whether typing `code_point` after a word continues it (`word_characters::continues_word`).
pub fn continues_word(code_point: i32) -> bool {
    word_characters::continues_word(code_point)
}

/// The case of `typed`, as its number in [`CASES`].
pub fn word_case(typed: &[u16]) -> i32 {
    let case = WordCase::of(typed);
    // There are three cases.
    CASES.iter().position(|&c| c == case).unwrap_or(0) as i32
}

/// `word` in the case numbered `case`.
pub fn apply_case(case: i32, word: &[u16]) -> Result<Vec<u16>> {
    let case = usize::try_from(case).ok().and_then(|i| CASES.get(i));
    let case = case.ok_or_else(|| illegal("no such case"))?;
    Ok(case.apply(word))
}

/// Whether `typed` has more than one capital (`word_case::is_mostly_caps`).
pub fn is_mostly_caps(typed: &[u16]) -> bool {
    word_case::is_mostly_caps(typed)
}

/// `typed` as the decoder is given it (`word_case::decoder_input`).
pub fn decoder_input(typed: &[u16]) -> Vec<u16> {
    word_case::decoder_input(typed)
}

/// The context before a word that starts after `text_before_word`, as `SuggestRules.Context`
/// reads it: whether a sentence starts there, as `WireFlag` spells it, then the previous words,
/// most recent first.
pub fn previous_words(text_before_word: &[u16]) -> Vec<Vec<u16>> {
    let previous = previous_words::PreviousWords::from(text_before_word);
    let start = wire::flag::encode(previous.is_beginning_of_sentence);
    [vec![start.encode_utf16().collect()], previous.words].concat()
}

/// AOSP's "modest" auto-correction threshold (`auto_correction::MODEST_THRESHOLD`).
pub fn modest_threshold() -> f32 {
    auto_correction::MODEST_THRESHOLD
}

/// The ordinal of the default auto-correction strength (`auto_correction::Strength::DEFAULT`).
pub fn default_strength() -> i32 {
    let all = auto_correction::Strength::ALL;
    let at = all
        .iter()
        .position(|&it| it == auto_correction::Strength::DEFAULT);
    at.map_or(0, |at| at as i32)
}

/// The auto-correction strengths' stored names, in the order of their ordinals
/// (`auto_correction::Strength::ALL`), so Kotlin's enum can be held to them.
pub fn strength_names() -> Vec<Vec<u16>> {
    auto_correction::Strength::ALL
        .iter()
        .map(|strength| strength.name().encode_utf16().collect())
        .collect()
}

/// The most previous words a context holds (`previous_words::MAX_COUNT`).
pub fn max_previous_words() -> i32 {
    previous_words::MAX_COUNT as i32
}

/// Whether auto-correcting `typed` to the decoder's `word` (with its `score` and `kind_and_flags`)
/// passes `threshold` (`auto_correction::suggestion_exceeds_threshold`).
pub fn exceeds_threshold(
    word: &[u16],
    score: i32,
    kind_and_flags: i32,
    typed: &[u16],
    threshold: f32,
) -> bool {
    let suggestion = RawSuggestion {
        word: word.to_vec(),
        score,
        kind_and_flags,
    };
    auto_correction::suggestion_exceeds_threshold(&suggestion, typed, threshold)
}

/// How much of `after` is right, weighted by `score` (`auto_correction::normalized_score`).
pub fn normalized_score(before: &[u16], after: &[u16], score: i32) -> f32 {
    auto_correction::normalized_score(before, after, score)
}

/// The edits from `before` to `after`, both code points (`auto_correction::edit_distance`).
/// Kotlin threw for a negative one, which isn't a code point; so does this.
pub fn edit_distance(before: &[i32], after: &[i32]) -> Result<i32> {
    Ok(auto_correction::edit_distance(
        &code_points(before)?,
        &code_points(after)?,
    ))
}

fn code_points(values: &[i32]) -> Result<Vec<u32>> {
    values
        .iter()
        .map(|&v| {
            u32::try_from(v)
                .map_err(|_| illegal(format!("Not a valid Unicode code point: 0x{v:X}")))
        })
        .collect()
}

/// How to undo auto-correcting `typed` to `corrected` at `start` (`undo::revert_at`), as
/// `SuggestRules.Revert` reads it: the length to replace, the text to put back, and whether a
/// phantom space follows; null if that is no longer the correction.
pub fn revert_at(
    env: &Env,
    typed: &[u16],
    corrected: &[u16],
    start: i32,
    cursor: i32,
    text_before_cursor: &[u16],
) -> Result<jobjectArray> {
    let Some(revert) = undo::revert_at((typed, corrected, start), cursor, text_before_cursor)
    else {
        return Ok(ptr::null_mut());
    };
    env.make_objects(&[
        &|| env.make(&[revert.length]),
        &|| env.make_string(&revert.text),
        &|| env.make(&[u8::from(revert.phantom_space)]),
    ])
}

/// How many previous-word slots a context of `word_count` words uses (`previous_words::slot_count`).
pub fn previous_slot_count(word_count: i32, sentence_start: bool) -> i32 {
    previous_words::slot_count(usize::try_from(word_count).unwrap_or(0), sentence_start) as i32
}
