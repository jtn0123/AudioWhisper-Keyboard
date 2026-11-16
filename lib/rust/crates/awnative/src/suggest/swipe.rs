//! JNI entry points for swipe typing's rules (`suggest::glide_ranker` and `suggest::swipe_words`),
//! called by `SwipeNative`. Kotlin asks the dictionary and the learned words what they know and
//! passes it in; these turn it into frequencies, bonuses and orders.

use std::ptr;

use jni_sys::{JNIEnv, jboolean, jclass, jdouble, jfloat, jfloatArray, jint, jintArray};
use jni_sys::{jobjectArray, jstring};
use suggest::glide_ranker::{self, Context};
use suggest::swipe_words;

use super::{boolean, non_null};
use crate::jni::{self, Env, Result, illegal};

/// The most a guess's context can take off its cost (`glide_ranker::MAX_BONUS`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_maxGlideBonus(
    _: *mut JNIEnv,
    _: jclass,
) -> jfloat {
    glide_ranker::MAX_BONUS
}

/// How much a guess's context favours it (`glide_ranker::bonus`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_glideBonus(
    _: *mut JNIEnv,
    _: jclass,
    pair_count: jint,
    ngram: jint,
    probability: jint,
) -> jfloat {
    glide_ranker::bonus(Context {
        pair_count,
        ngram,
        probability,
    })
}

/// The guesses' order, best first, as their indexes (`glide_ranker::rank`): each guess has a cost
/// and, at the same index, its context's pair count, n-gram and probability.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_rankGlide(
    env: *mut JNIEnv,
    _: jclass,
    costs: jfloatArray,
    pair_counts: jintArray,
    ngrams: jintArray,
    probabilities: jintArray,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let costs: Vec<f32> = env.read(costs, "costs")?;
            let contexts = contexts(env, pair_counts, ngrams, probabilities)?;
            let order = glide_ranker::rank(&costs, &contexts)
                .ok_or_else(|| illegal("costs and contexts differ in length"))?;
            // There are no more guesses than a Java array holds.
            let order: Vec<i32> = order.into_iter().map(|i| i as i32).collect();
            env.make(&order)
        })
    }
}

fn contexts(
    env: &Env,
    pair_counts: jintArray,
    ngrams: jintArray,
    probabilities: jintArray,
) -> Result<Vec<Context>> {
    let pair_counts: Vec<i32> = env.read(pair_counts, "pairCounts")?;
    let ngrams: Vec<i32> = env.read(ngrams, "ngrams")?;
    let probabilities: Vec<i32> = env.read(probabilities, "probabilities")?;
    if ngrams.len() != pair_counts.len() || probabilities.len() != pair_counts.len() {
        return Err(illegal("costs and contexts differ in length"));
    }
    let contexts = pair_counts.into_iter().zip(ngrams).zip(probabilities);
    Ok(contexts
        .map(|((pair_count, ngram), probability)| Context {
            pair_count,
            ngram,
            probability,
        })
        .collect())
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_maxProbability(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    swipe_words::MAX_PROBABILITY
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_common(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    swipe_words::COMMON
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_costPerStep(
    _: *mut JNIEnv,
    _: jclass,
) -> jdouble {
    swipe_words::COST_PER_STEP
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_learnedProbability(
    _: *mut JNIEnv,
    _: jclass,
) -> jint {
    swipe_words::LEARNED_PROBABILITY
}

/// Whether a swipe can spell `word` (`swipe_words::is_swipeable`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_isSwipeable(
    env: *mut JNIEnv,
    _: jclass,
    word: jstring,
) -> jboolean {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, boolean(false), |env| {
            let word = env.read_string(word, "word")?;
            Ok(boolean(swipe_words::is_swipeable(&word)))
        })
    }
}

/// The indexes of the words a swipe can spell, in order. The words are `text` cut at `lengths`
/// ([`jni::cut`]), so a whole dictionary crosses in one string.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_swipeable(
    env: *mut JNIEnv,
    _: jclass,
    text: jstring,
    lengths: jintArray,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let text = env.read_string(text, "text")?;
            let lengths: Vec<i32> = env.read(lengths, "lengths")?;
            let words = jni::cut(&text, &lengths)?;
            // There are no more words than a Java array holds.
            let swipeable: Vec<i32> = (0..words.len())
                .filter(|&i| swipe_words::is_swipeable(words[i]))
                .map(|i| i as i32)
                .collect();
            env.make(&swipeable)
        })
    }
}

/// The classifier's frequency for a word of `probability` (`swipe_words::frequency_of`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_frequencyOf(
    _: *mut JNIEnv,
    _: jclass,
    probability: jint,
) -> jdouble {
    swipe_words::frequency_of(probability)
}

/// The classifier's frequency for a word the dictionary and the user's words give these
/// probabilities (`swipe_words::frequency`).
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_frequency(
    _: *mut JNIEnv,
    _: jclass,
    probability: jint,
    user_probability: jint,
) -> jdouble {
    swipe_words::frequency(probability, user_probability)
}

/// As `frequency`, as the keyboard weighs it (`swipe_words::tuned_frequency`): its curve, and
/// `boost` steps from the user's use of the word.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_tunedFrequency(
    _: *mut JNIEnv,
    _: jclass,
    probability: jint,
    user_probability: jint,
    boost: jint,
) -> jdouble {
    swipe_words::tuned_frequency(probability, user_probability, boost)
}

/// The user's words (`swipe_words::user_words`), packed: for each, where it first appears among
/// `learned` followed by `personalWords`, then its probability.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SwipeNative_userWords(
    env: *mut JNIEnv,
    _: jclass,
    learned: jobjectArray,
    personal_words: jobjectArray,
    personal_frequencies: jintArray,
) -> jintArray {
    // SAFETY: the JVM passes its environment for this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let learned = non_null(env.read_strings(learned, "learned")?)?;
            let words = non_null(env.read_strings(personal_words, "personalWords")?)?;
            let frequencies: Vec<i32> = env.read(personal_frequencies, "personalFrequencies")?;
            if frequencies.len() != words.len() {
                return Err(illegal("personal words and frequencies differ in length"));
            }
            let learned: Vec<&[u16]> = learned.iter().map(Vec::as_slice).collect();
            let personal: Vec<(&[u16], i32)> =
                words.iter().map(Vec::as_slice).zip(frequencies).collect();
            let mut packed = Vec::new();
            for (place, probability) in swipe_words::user_words(&personal, &learned) {
                let place = i32::try_from(place).map_err(|_| illegal("too many words"))?;
                packed.extend([place, probability]);
            }
            env.make(&packed)
        })
    }
}
