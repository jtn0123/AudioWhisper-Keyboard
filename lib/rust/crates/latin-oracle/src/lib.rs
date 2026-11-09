//! Test-only: LatinIME's real C++, the decoder the app ships in `libjni_latinime.so`, built for
//! this computer so the Rust port can be compared with it in `cargo test`.
//!
//! `build.rs` compiles every C++ file in `lib/latinime` with `cpp/fake_jni.cpp`, a JNI
//! environment just big enough for LatinIME's own JNI glue. Calls here go through that glue's
//! registered native methods with arrays shaped as the Java side (`BinaryDictionary`,
//! `DicTraverseSession`, `ProximityInfo`) and the app's `LatinDictionaryEngine` shape them, so the
//! oracle answers as the app's decoder does.
//!
//! The C++ isn't known to be safe to call from several threads, so every call takes one lock.
//! This crate and `awnative` are the only ones with `unsafe`; this one never ships.
#![cfg(not(target_os = "android"))]

use std::ffi::{CString, c_void};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};

use suggest::qwerty;

pub mod cli;
pub mod hooks;
pub mod queries;

/// The native MAX_WORD_LENGTH: the longest word, and the input arrays' size.
pub const MAX_WORD_LENGTH: usize = 48;
/// DicTraverseSession.MAX_RESULTS: how many suggestions come back at most.
pub const MAX_RESULTS: usize = 18;
/// The native MAX_PREV_WORD_COUNT_FOR_N_GRAM: the previous-word slots.
pub const MAX_PREVIOUS_WORDS: usize = 3;
/// The native NOT_A_PROBABILITY.
pub const NOT_A_PROBABILITY: i32 = -1;
/// DicTraverseSession.NOT_A_COORDINATE.
pub const NOT_A_COORDINATE: i32 = -1;

const OPTIONS: usize = 5;
const OPTION_BLOCK_OFFENSIVE_WORDS: usize = 2;
const OPTION_WEIGHT_FOR_LOCALE_IN_THOUSANDS: usize = 4;
const NOT_A_WEIGHT: f32 = -1.0;

type Sink = extern "C" fn(*mut c_void, *const u16, usize);

// SAFETY: these are the functions cpp/fake_jni.cpp defines, with the same C signatures.
unsafe extern "C" {
    fn aw_oracle_load() -> i32;
    fn aw_oracle_open(path: *const i8, offset: i64, size: i64) -> i64;
    fn aw_oracle_close(dictionary: i64);
    fn aw_oracle_probability(dictionary: i64, word: *const i32, length: i32) -> i32;
    fn aw_oracle_ngram_probability(
        dictionary: i64,
        previous_flat: *const i32,
        previous_lengths: *const i32,
        beginning_of_sentence: *const u8,
        slots: i32,
        word: *const i32,
        length: i32,
    ) -> i32;
    fn aw_oracle_words(dictionary: i64, sink: Sink, context: *mut c_void);
    fn aw_oracle_proximity(
        width: i32,
        height: i32,
        grid_width: i32,
        grid_height: i32,
        key_width: i32,
        key_height: i32,
        grid: *const i32,
        grid_length: i32,
        key_count: i32,
        xs: *const i32,
        ys: *const i32,
        widths: *const i32,
        heights: *const i32,
        codes: *const i32,
    ) -> i64;
    fn aw_oracle_proximity_release(proximity: i64);
    fn aw_oracle_session(locale: *const i8, dictionary: i64, dictionary_size: i64) -> i64;
    fn aw_oracle_session_release(session: i64);
    fn aw_oracle_suggest(
        dictionary: i64,
        proximity: i64,
        session: i64,
        xs: *const i32,
        ys: *const i32,
        times: *const i32,
        pointer_ids: *const i32,
        codes: *const i32,
        input_capacity: i32,
        input_size: i32,
        options: *const i32,
        option_count: i32,
        previous_flat: *const i32,
        previous_lengths: *const i32,
        beginning_of_sentence: *const u8,
        previous_slots: i32,
        previous_count: i32,
        out_count: *mut i32,
        out_code_points: *mut i32,
        code_point_capacity: i32,
        out_scores: *mut i32,
        out_space_indices: *mut i32,
        out_types: *mut i32,
        result_capacity: i32,
        out_auto_commit: *mut i32,
        in_out_weight: *mut f32,
    );
}

/// The one lock every call into the C++ holds, taken after JNI_OnLoad has registered the natives.
pub fn lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    static LOADED: OnceLock<i32> = OnceLock::new();
    let guard = LOCK.lock().unwrap_or_else(|it| it.into_inner());
    // SAFETY: JNI_OnLoad only registers natives into the fake environment; it runs once, locked.
    let version = *LOADED.get_or_init(|| unsafe { aw_oracle_load() });
    assert!(version > 0, "LatinIME's JNI_OnLoad failed");
    guard
}

fn length(slice: &[i32]) -> i32 {
    i32::try_from(slice.len()).expect("array too long for Java")
}

/// Words as Java's `int[][]` of `slots` entries: the words, then null slots.
struct WordArrays {
    flat: Vec<i32>,
    lengths: Vec<i32>,
}

impl WordArrays {
    fn new(words: &[Vec<i32>], slots: usize) -> Self {
        assert!(words.len() <= slots, "more words than slots");
        let lengths = (0..slots)
            .map(|slot| words.get(slot).map_or(-1, |it| length(it)))
            .collect();
        Self {
            flat: words.concat(),
            lengths,
        }
    }
}

/// The app's bundled dictionary, `app/src/main/assets/dicts/main_en-US.dict`.
pub fn bundled_dictionary_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../app/src/main/assets/dicts/main_en-US.dict")
}

/// A dictionary opened read-only, as `BinaryDictionary` opens one.
#[derive(Debug)]
pub struct Dictionary {
    handle: i64,
    size: i64,
}

impl Dictionary {
    /// The C++ object (a `latinime::Dictionary *`) as a number, for hooks that reach below the JNI methods.
    /// Call the hook holding [`hooks::lock`].
    pub fn address(&self) -> i64 {
        self.handle
    }

    /// The `size` bytes at `offset` in `path`, or `None` if they aren't a usable dictionary.
    pub fn open(path: &Path, offset: i64, size: i64) -> Option<Self> {
        let path = CString::new(path.to_str().expect("ASCII path")).expect("no NUL in path");
        let handle = {
            let _lock = lock();
            // SAFETY: `path` is a NUL-terminated string that outlives the call.
            unsafe { aw_oracle_open(path.as_ptr(), offset, size) }
        };
        // Built only when open, so a failed open never reaches Drop's close.
        if handle == 0 {
            None
        } else {
            Some(Self { handle, size })
        }
    }

    /// The app's bundled English dictionary.
    pub fn bundled() -> Self {
        let path = bundled_dictionary_path();
        let size = std::fs::metadata(&path)
            .expect("the bundled dictionary")
            .len();
        Self::open(&path, 0, i64::try_from(size).expect("size fits")).expect("a valid dictionary")
    }

    /// `BinaryDictionary.getProbability`.
    pub fn probability(&self, word: &[i32]) -> i32 {
        let _lock = lock();
        // SAFETY: `word` is valid for its length; the handle is open until drop.
        unsafe { aw_oracle_probability(self.handle, word.as_ptr(), length(word)) }
    }

    /// `BinaryDictionary.getNgramProbability`: `previous` most recent first, with one flag each.
    pub fn ngram_probability(
        &self,
        previous: &[Vec<i32>],
        beginning: &[bool],
        word: &[i32],
    ) -> i32 {
        assert_eq!(
            previous.len(),
            beginning.len(),
            "one flag per previous word"
        );
        let arrays = WordArrays::new(previous, previous.len());
        let flags: Vec<u8> = beginning.iter().map(|&it| u8::from(it)).collect();
        let _lock = lock();
        // SAFETY: every pointer is valid for the lengths given; the handle is open until drop.
        unsafe {
            aw_oracle_ngram_probability(
                self.handle,
                arrays.flat.as_ptr(),
                arrays.lengths.as_ptr(),
                flags.as_ptr(),
                length(&arrays.lengths),
                word.as_ptr(),
                length(word),
            )
        }
    }

    /// `BinaryDictionary.getWords`: every word followed by `'\n'`, as UTF-16.
    pub fn words(&self) -> Vec<u16> {
        extern "C" fn copy(context: *mut c_void, text: *const u16, length: usize) {
            // SAFETY: `context` is the Vec below, and `text` is valid for `length` units.
            unsafe {
                (*context.cast::<Vec<u16>>())
                    .extend_from_slice(std::slice::from_raw_parts(text, length))
            }
        }
        let mut words = Vec::new();
        let _lock = lock();
        // SAFETY: `copy` only writes to `words`, which outlives the call.
        unsafe { aw_oracle_words(self.handle, copy, (&raw mut words).cast()) };
        words
    }
}

impl Drop for Dictionary {
    fn drop(&mut self) {
        let _lock = lock();
        // SAFETY: the handle came from aw_oracle_open and is closed only here.
        unsafe { aw_oracle_close(self.handle) }
    }
}

/// The app's QWERTY spatial model, as `LatinDictionaryEngine.qwertyProximityInfo` builds it.
#[derive(Debug)]
pub struct Proximity {
    handle: i64,
}

impl Proximity {
    /// The C++ object (a `latinime::ProximityInfo *`) as a number, for hooks that reach below the JNI methods.
    /// Call the hook holding [`hooks::lock`].
    pub fn address(&self) -> i64 {
        self.handle
    }

    pub fn qwerty() -> Self {
        let keys: Vec<qwerty::Key> = qwerty::keys().collect();
        let column = |f: &dyn Fn(&qwerty::Key) -> i32| keys.iter().map(f).collect::<Vec<i32>>();
        let (xs, ys, codes) = (
            column(&|it| it.x),
            column(&|it| it.y),
            column(&|it| it.code_point),
        );
        let widths = vec![qwerty::KEY_WIDTH; keys.len()];
        let heights = vec![qwerty::KEY_HEIGHT; keys.len()];
        let grid = qwerty::proximity_grid();
        let _lock = lock();
        // SAFETY: every array is valid for the length given and only read during the call.
        let handle = unsafe {
            aw_oracle_proximity(
                qwerty::KEYBOARD_WIDTH,
                qwerty::KEYBOARD_HEIGHT,
                qwerty::GRID_WIDTH,
                qwerty::GRID_HEIGHT,
                qwerty::KEY_WIDTH,
                qwerty::KEY_HEIGHT,
                grid.as_ptr(),
                length(&grid),
                length(&xs),
                xs.as_ptr(),
                ys.as_ptr(),
                widths.as_ptr(),
                heights.as_ptr(),
                codes.as_ptr(),
            )
        };
        Self { handle }
    }
}

impl Drop for Proximity {
    fn drop(&mut self) {
        let _lock = lock();
        // SAFETY: the handle came from aw_oracle_proximity and is released only here.
        unsafe { aw_oracle_proximity_release(self.handle) }
    }
}

/// A decoder session, as `DicTraverseSession`'s constructor makes one for a dictionary.
#[derive(Debug)]
pub struct Session {
    handle: i64,
}

impl Session {
    /// The C++ object (a `latinime::DicTraverseSession *`) as a number, for hooks that reach below the JNI methods.
    /// Call the hook holding [`hooks::lock`].
    pub fn address(&self) -> i64 {
        self.handle
    }

    pub fn new(locale: &str, dictionary: &Dictionary) -> Self {
        let locale = CString::new(locale).expect("no NUL in locale");
        let _lock = lock();
        // SAFETY: `locale` is NUL-terminated and outlives the call; the dictionary is open.
        let handle =
            unsafe { aw_oracle_session(locale.as_ptr(), dictionary.handle, dictionary.size) };
        Self { handle }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        let _lock = lock();
        // SAFETY: the handle came from aw_oracle_session and is released only here.
        unsafe { aw_oracle_session_release(self.handle) }
    }
}

/// What `LatinDictionaryEngine.decode` hands the decoder.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    /// The typed code points, at most [`MAX_WORD_LENGTH`].
    pub code_points: Vec<i32>,
    /// Where each was touched on the QWERTY model, or `None` for no coordinate.
    pub points: Vec<Option<(i32, i32)>>,
    pub block_offensive_words: bool,
    /// The previous words, most recent first, at most [`MAX_PREVIOUS_WORDS`].
    pub previous: Vec<Vec<i32>>,
    /// One flag per previous word: the slot is the start of a sentence.
    pub beginning_of_sentence: Vec<bool>,
}

/// The decoder's output arrays after one call, as Java reads them back.
#[derive(Clone, Debug, PartialEq)]
pub struct Output {
    pub count: i32,
    pub code_points: Vec<i32>,
    pub scores: Vec<i32>,
    pub space_indices: Vec<i32>,
    pub types: Vec<i32>,
    pub auto_commit_confidence: i32,
    pub weight: f32,
}

impl Output {
    /// The suggestions as the app reads them: each word up to its first 0, with score and type.
    pub fn suggestions(&self) -> Vec<(Vec<i32>, i32, i32)> {
        let count = usize::try_from(self.count.clamp(0, MAX_RESULTS as i32)).expect("clamped");
        (0..count)
            .map(|index| {
                let word =
                    &self.code_points[index * MAX_WORD_LENGTH..(index + 1) * MAX_WORD_LENGTH];
                let end = word
                    .iter()
                    .position(|&it| it == 0)
                    .unwrap_or(MAX_WORD_LENGTH);
                (word[..end].to_vec(), self.scores[index], self.types[index])
            })
            .collect()
    }
}

/// `BinaryDictionary.getSuggestions` with the session reset as `resetForTyping` resets it.
pub fn suggest(
    dictionary: &Dictionary,
    proximity: &Proximity,
    session: &Session,
    query: &Query,
) -> Output {
    let size = query.code_points.len();
    assert!(
        size <= MAX_WORD_LENGTH,
        "the app never passes a longer word"
    );
    assert_eq!(query.points.len(), size, "one point per code point");
    assert!(
        query.previous.len() <= MAX_PREVIOUS_WORDS,
        "at most three previous words"
    );
    assert_eq!(
        query.previous.len(),
        query.beginning_of_sentence.len(),
        "one flag per word"
    );
    let mut codes = [0; MAX_WORD_LENGTH];
    let mut xs = [NOT_A_COORDINATE; MAX_WORD_LENGTH];
    let mut ys = [NOT_A_COORDINATE; MAX_WORD_LENGTH];
    codes[..size].copy_from_slice(&query.code_points);
    for (index, point) in query.points.iter().enumerate() {
        if let Some((x, y)) = point {
            (xs[index], ys[index]) = (*x, *y);
        }
    }
    let zeros = [0; MAX_WORD_LENGTH];
    let mut options = [0; OPTIONS];
    options[OPTION_BLOCK_OFFENSIVE_WORDS] = i32::from(query.block_offensive_words);
    options[OPTION_WEIGHT_FOR_LOCALE_IN_THOUSANDS] = 1000;
    let previous = WordArrays::new(&query.previous, MAX_PREVIOUS_WORDS);
    let mut beginning = [0u8; MAX_PREVIOUS_WORDS];
    for (slot, &flag) in query.beginning_of_sentence.iter().enumerate() {
        beginning[slot] = u8::from(flag);
    }
    let mut output = Output {
        count: 0,
        code_points: vec![0; MAX_WORD_LENGTH * MAX_RESULTS],
        scores: vec![0; MAX_RESULTS],
        space_indices: vec![0; MAX_RESULTS],
        types: vec![0; MAX_RESULTS],
        auto_commit_confidence: 0,
        weight: NOT_A_WEIGHT,
    };
    let _lock = lock();
    // SAFETY: every array is valid for the length passed with it, the outputs are writable for
    // theirs, and the handles are open until their owners drop.
    unsafe {
        aw_oracle_suggest(
            dictionary.handle,
            proximity.handle,
            session.handle,
            xs.as_ptr(),
            ys.as_ptr(),
            zeros.as_ptr(),
            zeros.as_ptr(),
            codes.as_ptr(),
            MAX_WORD_LENGTH as i32,
            size as i32,
            options.as_ptr(),
            OPTIONS as i32,
            previous.flat.as_ptr(),
            previous.lengths.as_ptr(),
            beginning.as_ptr(),
            MAX_PREVIOUS_WORDS as i32,
            query.previous.len() as i32,
            &raw mut output.count,
            output.code_points.as_mut_ptr(),
            length(&output.code_points),
            output.scores.as_mut_ptr(),
            output.space_indices.as_mut_ptr(),
            output.types.as_mut_ptr(),
            MAX_RESULTS as i32,
            &raw mut output.auto_commit_confidence,
            &raw mut output.weight,
        );
    }
    output
}

#[cfg(test)]
mod tests;
