//! JNI entry points for swipe typing's scorer, called by `GlideNative` in lib/rust's Kotlin.
//!
//! An index is built once per word list and layout. Each swipe then opens a query on it holding
//! the swipe and the keys as drawn, asks it for the candidates, and has it score them with their
//! frequencies.

use std::ptr;

use glide::{CharKeys, Gesture, GlideIndex, Key, Layout, Swipe, Tuning};
use jni_sys::{JNIEnv, jcharArray, jclass, jfloatArray, jint, jintArray, jlong};

use crate::handle::{free, into_handle, object};
use crate::jni::{self, Env, Result, illegal};

/// One swipe against an index.
#[derive(Debug)]
struct Query {
    index: *const GlideIndex,
    swipe: Gesture,
    /// Each point's time in milliseconds, or none.
    times: Vec<f32>,
    keys: Vec<Key>,
}

/// The layout's keys from their codes and bounds: centre x, centre y, width and height for each.
fn keys(env: &Env, codes: jintArray, bounds: jfloatArray) -> Result<Vec<Key>> {
    let (codes, bounds) = (
        env.read::<i32>(codes, "keyCodes")?,
        env.read::<f32>(bounds, "keyBounds")?,
    );
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

fn unsigned(values: Vec<i32>, name: &str) -> Result<Vec<u32>> {
    values
        .into_iter()
        .map(|v| u32::try_from(v).map_err(|_| illegal(format!("{name} has a negative value"))))
        .collect()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_distinctChars(
    env: *mut JNIEnv,
    _: jclass,
    chars: jcharArray,
) -> jcharArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let chars = env.read::<u16>(chars, "chars")?;
            env.make(&glide::distinct_units(&chars))
        })
    }
}

/// Indexes the words in `chars` (word `w` from `starts[w]` to `starts[w + 1]`) with `table`, four
/// Ints per character: the UTF-16 unit, its lower case, its path key and its end key.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_build(
    env: *mut JNIEnv,
    _: jclass,
    chars: jcharArray,
    starts: jintArray,
    table: jintArray,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, 0, |env| {
            let chars = env.read::<u16>(chars, "chars")?;
            let starts = unsigned(env.read::<i32>(starts, "starts")?, "starts")?;
            let table = env.read::<i32>(table, "table")?;
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
            let index =
                GlideIndex::build(&chars, &starts, &table).map_err(|e| illegal(e.to_string()))?;
            Ok(into_handle(index))
        })
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_free(
    _: *mut JNIEnv,
    _: jclass,
    index: jlong,
) {
    // SAFETY: the Kotlin side frees each index once, when nothing can reach it.
    unsafe { free::<GlideIndex>(index) }
}

/// Opens a query for the swipe through `xs` and `ys`, drawn on keys with these codes and bounds.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_query(
    env: *mut JNIEnv,
    _: jclass,
    index: jlong,
    xs: jfloatArray,
    ys: jfloatArray,
    key_codes: jintArray,
    key_bounds: jfloatArray,
) -> jlong {
    // SAFETY: the JVM passes its environment for this call, and the index is live (see `object`).
    unsafe {
        jni::run(env, 0, |env| {
            open_query(env, index, (xs, ys, None), key_codes, key_bounds)
        })
    }
}

/// As `query`, with each point's time in milliseconds in `times`, which `scoreTuned` weighs.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_queryTimed(
    env: *mut JNIEnv,
    _: jclass,
    index: jlong,
    xs: jfloatArray,
    ys: jfloatArray,
    times: jfloatArray,
    key_codes: jintArray,
    key_bounds: jfloatArray,
) -> jlong {
    // SAFETY: as for `query`.
    unsafe {
        jni::run(env, 0, |env| {
            open_query(env, index, (xs, ys, Some(times)), key_codes, key_bounds)
        })
    }
}

/// A query's handle, for the swipe through `xs` and `ys` (drawn at `times`, if given).
///
/// # Safety
/// `index` must be live (see `object`).
unsafe fn open_query(
    env: &Env,
    index: jlong,
    (xs, ys, times): (jfloatArray, jfloatArray, Option<jfloatArray>),
    key_codes: jintArray,
    key_bounds: jfloatArray,
) -> Result<jlong> {
    // SAFETY: as the caller promises.
    let index = unsafe { object::<GlideIndex>(index, "index")? };
    let (xs, ys) = (env.read::<f32>(xs, "xs")?, env.read::<f32>(ys, "ys")?);
    if xs.len() != ys.len() {
        return Err(illegal("xs and ys differ in length"));
    }
    let times = match times {
        Some(times) => env.read::<f32>(times, "times")?,
        None => Vec::new(),
    };
    if !times.is_empty() && times.len() != xs.len() {
        return Err(illegal("times and xs differ in length"));
    }
    let query = Query {
        index,
        swipe: Gesture::from_points(&xs, &ys),
        times,
        keys: keys(env, key_codes, key_bounds)?,
    };
    Ok(into_handle(query))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_freeQuery(
    _: *mut JNIEnv,
    _: jclass,
    query: jlong,
) {
    // SAFETY: the Kotlin side frees each query once, on closing it.
    unsafe { free::<Query>(query) }
}

/// The query's index and its keys as a layout of it.
///
/// # Safety
/// `query` must be live (see `object`); its index then is too, as the Kotlin query holds it.
unsafe fn open<'a>(query: jlong) -> Result<(&'a Query, &'a GlideIndex)> {
    // SAFETY: as the caller promises.
    let query = unsafe { object::<Query>(query, "query")? };
    // SAFETY: the index outlives every query on it.
    Ok((query, unsafe { &*query.index }))
}

/// The words worth scoring for the query's swipe, in order.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_candidates(
    env: *mut JNIEnv,
    _: jclass,
    query: jlong,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call, and the query is live (see `object`).
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (query, index) = open(query)?;
            let candidates =
                index.candidates(&query.swipe, &Layout::new(&query.keys, index.codes()));
            // Word numbers fit in an Int: they number a Java list.
            env.make(&candidates.iter().map(|&w| w as i32).collect::<Vec<_>>())
        })
    }
}

/// Scores `candidates` at their `frequencies`, writing the likeliest words and their weights to
/// `out_words` and `out_weights`, as many as `out_words` holds. Returns how many it wrote. Scores
/// as FlorisBoard's classifier did (`glide::Tuning::ORIGINAL`), which the reference tests check.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_score(
    env: *mut JNIEnv,
    _: jclass,
    query: jlong,
    candidates: jintArray,
    frequencies: jfloatArray,
    out_words: jintArray,
    out_weights: jfloatArray,
) -> jint {
    let to = (candidates, frequencies, out_words, out_weights);
    // SAFETY: the JVM passes its environment for this call, and the query is live (see `object`).
    unsafe { jni::run(env, 0, |env| score(env, query, to, &Tuning::ORIGINAL)) }
}

/// As `score`, as the keyboard scores: with the additions of `glide::Tuning::DEFAULT`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_GlideNative_scoreTuned(
    env: *mut JNIEnv,
    _: jclass,
    query: jlong,
    candidates: jintArray,
    frequencies: jfloatArray,
    out_words: jintArray,
    out_weights: jfloatArray,
) -> jint {
    let to = (candidates, frequencies, out_words, out_weights);
    // SAFETY: as for `score`.
    unsafe { jni::run(env, 0, |env| score(env, query, to, &Tuning::DEFAULT)) }
}

/// Scores the query's `candidates` at their `frequencies` with `tuning` into `out_words` and
/// `out_weights`; how many it wrote.
///
/// # Safety
/// `query` must be live (see `object`).
unsafe fn score(
    env: &Env,
    query: jlong,
    (candidates, frequencies, out_words, out_weights): (
        jintArray,
        jfloatArray,
        jintArray,
        jfloatArray,
    ),
    tuning: &Tuning,
) -> Result<jint> {
    // SAFETY: as the caller promises.
    let (query, index) = unsafe { open(query)? };
    let candidates = unsigned(env.read::<i32>(candidates, "candidates")?, "candidates")?;
    let frequencies = env.read::<f32>(frequencies, "frequencies")?;
    let max = env.read::<i32>(out_words, "outWords")?.len();
    let layout = Layout::new(&query.keys, index.codes());
    let swipe = Swipe {
        gesture: &query.swipe,
        times: &query.times,
    };
    let ranking = index
        .score_with(&candidates, &frequencies, swipe, &layout, max, tuning)
        .map_err(|e| illegal(e.to_string()))?;
    let words: Vec<i32> = ranking.words().iter().map(|&w| w as i32).collect();
    env.write(out_words, &words, "outWords")?;
    env.write(out_weights, ranking.weights(), "outWeights")?;
    Ok(words.len() as jint)
}
