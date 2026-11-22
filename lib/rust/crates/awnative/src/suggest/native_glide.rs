//! The glide ranking (`suggest::glide_ranker`), for `NativeGlideNative`, whose generated entry
//! point calls this. It asks the session and the learned words through their handles.

use std::sync::MutexGuard;

use suggest::glide_ranker::{self, Query};
use suggest::learned_words::LearnedWords;
use suggest::previous_words::PreviousWords;
use suggest::raw::NOT_A_PROBABILITY;
use suggest::session::Session;

use crate::handle::locked;
use crate::jni::{Result, illegal};

/// The indexes of `words`, whose glide `costs` these are, best first after `previous`. `session`
/// is lent under its owner's lock (0 once closed), or -1 for no dictionary; `history` is the
/// learned words, or 0 for none.
pub fn rank(
    session: i64,
    history: i64,
    words: &[Vec<u16>],
    costs: &[f32],
    previous: &[Vec<u16>],
    sentence_start: bool,
) -> Result<Vec<i32>> {
    let words: Vec<&[u16]> = words.iter().map(Vec::as_slice).collect();
    let previous = PreviousWords {
        words: previous.to_vec(),
        is_beginning_of_sentence: sentence_start,
    };
    let ask = |query: Query| -> Result<i32> {
        Ok(match query {
            Query::Ngram(..) | Query::Probability(_) if session == -1 => NOT_A_PROBABILITY,
            Query::Ngram(p, w) => guarded::<Session>(session, "session")?.context_probability(p, w),
            Query::Probability(w) => guarded::<Session>(session, "session")?.probability(w),
            Query::Pair(..) if history == 0 => 0,
            Query::Pair(p, w) => guarded::<LearnedWords>(history, "learned words")?.pair_count(
                p.words.first().map(Vec::as_slice),
                p.is_beginning_of_sentence,
                w,
            ),
        })
    };
    let order = glide_ranker::rank_from(costs, &words, &previous, ask)?
        .ok_or_else(|| illegal("glide words and costs differ in length"))?;
    // Each is an index into a Java array, so it fits.
    Ok(order.into_iter().map(|i| i as i32).collect())
}

fn guarded<'a, T>(handle: i64, what: &str) -> Result<MutexGuard<'a, T>> {
    // SAFETY: Kotlin holds the session's lock and keeps the history reachable for the whole call,
    // so neither has been freed; a closed session's handle is 0, which `locked` refuses.
    unsafe { locked(handle, what) }
}
