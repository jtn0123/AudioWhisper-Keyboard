//! Swipe typing's scorer (`glide`), for `GlideNative`, whose generated entry points call these.
//!
//! The words cross once, as one string and their lengths. An index is built from them once per
//! layout. Each swipe then opens a query on it holding the swipe and the keys as drawn, asks it
//! for the candidates, and has it score them with their frequencies. A query shares its index, so
//! the index lives until its last query is freed.

use std::sync::Arc;

use glide::{CharKeys, Gesture, GlideIndex, Key, Layout, Swipe, Tuning};
use jni_sys::jobjectArray;

use crate::boundary::Chars;
use crate::handle::{into_handle, object};
use crate::jni::{Env, Result, illegal};

/// Swipe typing's words: word `w` is `chars[starts[w]..starts[w + 1]]`.
#[derive(Debug)]
struct Words {
    chars: Vec<u16>,
    starts: Vec<u32>,
}

/// One swipe against an index.
#[derive(Debug)]
struct Query {
    index: Arc<GlideIndex>,
    swipe: Gesture,
    /// Each point's time in milliseconds, or none.
    times: Vec<f32>,
    keys: Vec<Key>,
}

/// The layout's keys from their codes and bounds: centre x, centre y, width and height for each.
fn keys(codes: &[i32], bounds: &[f32]) -> Result<Vec<Key>> {
    let (bounds, rest) = bounds.as_chunks::<4>();
    if bounds.len() != codes.len() || !rest.is_empty() {
        return Err(illegal("keyBounds needs four values per key"));
    }
    Ok(codes
        .iter()
        .zip(bounds)
        .map(|(&code, &[center_x, center_y, width, height])| Key {
            code,
            center_x,
            center_y,
            width,
            height,
        })
        .collect())
}

fn unsigned(values: &[i32], name: &str) -> Result<Vec<u32>> {
    values
        .iter()
        .map(|&v| u32::try_from(v).map_err(|_| illegal(format!("{name} has a negative value"))))
        .collect()
}

/// The words in `text`, one after another, `lengths` long.
pub fn new_words(text: &[u16], lengths: &[i32]) -> Result<i64> {
    let mut starts = Vec::with_capacity(lengths.len() + 1);
    let mut at = 0u32;
    starts.push(at);
    for length in unsigned(lengths, "lengths")? {
        at = at
            .checked_add(length)
            .ok_or_else(|| illegal("lengths overflow"))?;
        starts.push(at);
    }
    if at as usize != text.len() {
        return Err(illegal("lengths and text differ"));
    }
    let chars = text.to_vec();
    Ok(into_handle(Words { chars, starts }))
}

pub fn free_words(words: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each word list once, when nothing can reach it.
    unsafe { crate::handle::free::<Words>(words) };
    Ok(())
}

fn words<'a>(words: i64) -> Result<&'a Words> {
    // SAFETY: Kotlin holds the word list while it calls, so it hasn't been freed.
    unsafe { object(words, "words") }
}

/// How many words there are.
pub fn word_count(words: i64) -> Result<i32> {
    // A word list is a Java list's, so its length fits.
    Ok((self::words(words)?.starts.len() - 1) as i32)
}

/// Every UTF-16 unit the words use, once each, ascending.
pub fn distinct_chars(words: i64) -> Result<Chars> {
    Ok(Chars(glide::distinct_units(&self::words(words)?.chars)))
}

/// Indexes the words with `table`, four Ints per character: the UTF-16 unit, its lower case, its
/// path key and its end key.
pub fn build(words: i64, table: &[i32]) -> Result<i64> {
    let words = self::words(words)?;
    let (rows, rest) = table.as_chunks::<4>();
    if !rest.is_empty() {
        return Err(illegal("table needs four values per character"));
    }
    let table: Vec<CharKeys> = rows
        .iter()
        .map(|&[unit, lower, path_key, end_key]| CharKeys {
            unit: unit as u16,
            lower: lower as u16,
            path_key,
            end_key,
        })
        .collect();
    let index = GlideIndex::build(&words.chars, &words.starts, &table)
        .map_err(|e| illegal(e.to_string()))?;
    Ok(into_handle(Arc::new(index)))
}

pub fn free(index: i64) -> Result<()> {
    // SAFETY: the Kotlin side frees each index once, when nothing can reach it; a query holds its
    // own share.
    unsafe { crate::handle::free::<Arc<GlideIndex>>(index) };
    Ok(())
}

/// Opens a query for the swipe through `xs` and `ys`, drawn on keys with these codes and bounds.
pub fn query(
    index: i64,
    xs: &[f32],
    ys: &[f32],
    key_codes: &[i32],
    key_bounds: &[f32],
) -> Result<i64> {
    open_query(index, (xs, ys, &[]), key_codes, key_bounds)
}

/// As `query`, with each point's time in milliseconds in `times`, which `scoreTuned` weighs.
pub fn query_timed(
    index: i64,
    xs: &[f32],
    ys: &[f32],
    times: &[f32],
    key_codes: &[i32],
    key_bounds: &[f32],
) -> Result<i64> {
    open_query(index, (xs, ys, times), key_codes, key_bounds)
}

/// A query's handle, for the swipe through `xs` and `ys` (drawn at `times`, if any).
fn open_query(
    index: i64,
    (xs, ys, times): (&[f32], &[f32], &[f32]),
    key_codes: &[i32],
    key_bounds: &[f32],
) -> Result<i64> {
    // SAFETY: Kotlin holds the index while it calls, so it hasn't been freed.
    let index = unsafe { object::<Arc<GlideIndex>>(index, "index")? };
    if xs.len() != ys.len() {
        return Err(illegal("xs and ys differ in length"));
    }
    if !times.is_empty() && times.len() != xs.len() {
        return Err(illegal("times and xs differ in length"));
    }
    let query = Query {
        index: Arc::clone(index),
        swipe: Gesture::from_points(xs, ys),
        times: times.to_vec(),
        keys: keys(key_codes, key_bounds)?,
    };
    Ok(into_handle(query))
}

pub fn free_query(query: i64) -> Result<()> {
    // SAFETY: the synchronized query zeros its handle on close and frees it exactly once.
    unsafe { crate::handle::free::<Query>(query) };
    Ok(())
}

fn open<'a>(query: i64) -> Result<&'a Query> {
    // SAFETY: the synchronized query keeps its handle live and serializes close; a closed one
    // passes 0, which throws.
    unsafe { object(query, "query") }
}

/// The words worth scoring for the query's swipe, in order.
pub fn candidates(query: i64) -> Result<Vec<i32>> {
    let query = open(query)?;
    let index = &query.index;
    let candidates = index.candidates(&query.swipe, &Layout::new(&query.keys, index.codes()));
    // Word numbers fit in an Int: they number a Java list.
    Ok(candidates.iter().map(|&w| w as i32).collect())
}

/// The `max` likeliest of `candidates` at their `frequencies`, as `[words, weights]`, scored as
/// FlorisBoard's classifier did (`glide::Tuning::ORIGINAL`), which the reference tests check.
pub fn score(
    env: &Env,
    query: i64,
    candidates: &[i32],
    frequencies: &[f32],
    max: i32,
) -> Result<jobjectArray> {
    scored(
        env,
        query,
        (candidates, frequencies, max),
        &Tuning::ORIGINAL,
    )
}

/// As `score`, as the keyboard scores: with the additions of `glide::Tuning::DEFAULT`.
pub fn score_tuned(
    env: &Env,
    query: i64,
    candidates: &[i32],
    frequencies: &[f32],
    max: i32,
) -> Result<jobjectArray> {
    scored(env, query, (candidates, frequencies, max), &Tuning::DEFAULT)
}

/// The query's `candidates` scored at their `frequencies` with `tuning`, at most `max`.
fn scored(
    env: &Env,
    query: i64,
    (candidates, frequencies, max): (&[i32], &[f32], i32),
    tuning: &Tuning,
) -> Result<jobjectArray> {
    let query = open(query)?;
    let index = &query.index;
    let candidates = unsigned(candidates, "candidates")?;
    let max = usize::try_from(max).map_err(|_| illegal("max is negative"))?;
    let layout = Layout::new(&query.keys, index.codes());
    let swipe = Swipe {
        gesture: &query.swipe,
        times: &query.times,
    };
    let ranking = index
        .score_with(&candidates, frequencies, swipe, &layout, max, tuning)
        .map_err(|e| illegal(e.to_string()))?;
    let words: Vec<i32> = ranking.words().iter().map(|&w| w as i32).collect();
    env.make_objects(&[&|| env.make(&words), &|| env.make(ranking.weights())])
}
