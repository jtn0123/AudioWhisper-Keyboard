//! The in-process Latin session (`suggest::session`), for `LatinSessionNative`, whose generated
//! entry points call these. No callbacks run while the session is locked. Rows of strings carry
//! numbers in decimal and flags as `WireFlag`'s "1" and "0".
use super::suggester::{decimal, stores, strip_row};
use crate::{
    files::read_range,
    handle::{into_handle, locked},
    jni::{self, Result, illegal},
};
#[cfg(any(not(target_os = "android"), feature = "latin-fixture"))]
use jni_sys::{JNIEnv, jclass, jintArray, jlong};
#[cfg(any(not(target_os = "android"), feature = "latin-fixture"))]
use std::ptr;
use std::sync::{Arc, Mutex, MutexGuard};
use suggest::{
    auto_correction::Strength,
    previous_words::PreviousWords,
    session::{Error as SessionError, Session},
    suggester::{Suggester, TypingState},
    tap::TapOffset,
};

use crate::boundary::args::LatinSessionNativeSuggest;

impl From<SessionError> for jni::Error {
    fn from(error: SessionError) -> Self {
        match error {
            SessionError::CannotRoundNaN => illegal("Cannot round NaN value."),
            SessionError::NegativeCount(n) => suggest::learned_words::NegativeCount(n).into(),
        }
    }
}

fn previous(words: &[Vec<u16>], sentence_start: bool) -> PreviousWords {
    PreviousWords {
        words: words.to_vec(),
        is_beginning_of_sentence: sentence_start,
    }
}

/// Each tap as three floats: whether it is known, then its offset across and down.
fn taps(values: &[f32]) -> Result<Vec<Option<TapOffset>>> {
    let (chunks, rest) = values.as_chunks::<3>();
    if !rest.is_empty() {
        return Err(illegal("three values per tap"));
    }
    Ok(chunks
        .iter()
        .map(|v| (v[0] != 0.0).then_some(TapOffset { dx: v[1], dy: v[2] }))
        .collect())
}

/// `flag` as `WireFlag` encodes it.
fn flag(flag: bool) -> Vec<u16> {
    vec![if flag {
        u16::from(b'1')
    } else {
        u16::from(b'0')
    }]
}

fn session<'a>(handle: i64) -> Result<MutexGuard<'a, Session>> {
    // SAFETY: the Kotlin wrapper holds its live handle under the same monitor as close.
    unsafe { locked(handle, "session") }
}

/// A session over the dictionary `bytes`, or 0 if they aren't one.
pub fn open(bytes: &[i8]) -> i64 {
    let bytes: Vec<u8> = bytes.iter().map(|&b| b as u8).collect();
    Session::open(Arc::from(bytes)).map_or(0, |s| into_handle(Mutex::new(s)))
}

pub fn free(handle: i64) -> Result<()> {
    // SAFETY: the Kotlin cleaner frees this allocation exactly once, under its wrapper's lock.
    unsafe { crate::handle::free::<Mutex<Session>>(handle) };
    Ok(())
}

/// Reads the n-gram model at `[offset, offset + length)` of the file at `path` (an uncompressed
/// APK entry) into the session. False, with the session left as it was, if it can't be read or
/// isn't a model.
pub fn attach_model(handle: i64, path: &[u16], offset: i64, length: i64) -> Result<bool> {
    let path = String::from_utf16_lossy(path);
    let (Ok(offset), Ok(length)) = (u64::try_from(offset), usize::try_from(length)) else {
        return Ok(false);
    };
    let Ok(bytes) = read_range(&path, offset, length) else {
        return Ok(false);
    };
    let Some(model) = ngram::Model::open(bytes) else {
        return Ok(false);
    };
    session(handle)?.attach(Arc::new(model));
    Ok(true)
}

pub fn probability(handle: i64, word: &[u16]) -> Result<i32> {
    Ok(session(handle)?.probability(word))
}

pub fn ngram(
    handle: i64,
    word: &[u16],
    previous: &[Vec<u16>],
    sentence_start: bool,
) -> Result<i32> {
    Ok(session(handle)?.ngram_probability(&self::previous(previous, sentence_start), word))
}

/// The dictionary's words.
pub fn words(handle: i64) -> Result<Vec<Vec<u16>>> {
    Ok(session(handle)?.words())
}

/// The candidates for `input`, three strings each: the word, its score, and its kind and flags.
pub fn decode(
    handle: i64,
    input: &[u16],
    previous: &[Vec<u16>],
    sentence_start: bool,
    block: bool,
    taps: &[f32],
) -> Result<Vec<Vec<u16>>> {
    let previous = self::previous(previous, sentence_start);
    let taps = self::taps(taps)?;
    let results = session(handle)?.decode(input, &previous, block, &taps)?;
    let mut row = Vec::with_capacity(results.len() * 3);
    for result in results {
        row.extend([
            result.word,
            decimal(result.score),
            decimal(result.kind_and_flags),
        ]);
    }
    Ok(row)
}

/// The strip for the typed word: its auto-correction's and typed word's indexes, or -1, then its
/// words.
pub fn suggest(args: LatinSessionNativeSuggest) -> Result<Vec<Vec<u16>>> {
    // SAFETY: Kotlin holds both stores for the call.
    let (personal, learned) = unsafe { stores(args.personal, args.learned)? };
    let state = TypingState {
        typed: args.typed.to_vec(),
        previous_words: previous(args.previous, args.sentence_start),
        is_cursor_mid_word: args.mid_word,
    };
    let offsets = taps(args.taps)?;
    let mut session = session(args.handle)?;
    super::letter_odds::note(&session, &state);
    let mut dictionary = session.request(&state.previous_words, args.block, &offsets);
    let strip = Suggester {
        dictionary: &mut dictionary,
        personal,
        learned,
        threshold: Strength::from_ordinal(args.strength).threshold(),
    }
    .suggest(&state, args.max_count)?;
    Ok(strip_row(&strip))
}

/// Spell-checks `word` with no context: whether it is valid and whether that is confident, then
/// the suggestions.
pub fn spell(
    handle: i64,
    personal: i64,
    learned: i64,
    word: &[u16],
    block: bool,
    max_count: i32,
    threshold: f32,
) -> Result<Vec<Vec<u16>>> {
    // SAFETY: Kotlin holds both stores for the call.
    let (personal, learned) = unsafe { stores(personal, learned)? };
    let mut session = session(handle)?;
    let previous = PreviousWords::NONE;
    let mut dictionary = session.request(&previous, block, &[]);
    let check = Suggester {
        dictionary: &mut dictionary,
        personal,
        learned,
        threshold: Some(threshold),
    }
    .spell(word, max_count)?;
    let mut row = vec![flag(check.is_valid), flag(check.is_high_confidence)];
    row.extend(check.suggestions);
    Ok(row)
}

/// Test APKs/host tests alone can inspect untouched native slots. This symbol is absent from shipped Android libraries.
#[cfg(any(not(target_os = "android"), feature = "latin-fixture"))]
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_suggest_A1Probe_snapshot(
    env: *mut JNIEnv,
    _: jclass,
    handle: jlong,
) -> jintArray {
    // SAFETY: the test holds a live session wrapper while reading its handle, with no concurrent close.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let session = locked::<Session>(handle, "session")?;
            let o = session.output();
            let mut result = vec![o.count];
            for array in [&o.code_points, &o.scores, &o.space_indices, &o.types] {
                result.extend(array);
            }
            result.extend([o.auto_commit_confidence, o.weight.to_bits() as i32]);
            env.make(&result)
        })
    }
}
