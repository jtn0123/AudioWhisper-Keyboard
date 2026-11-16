//! Copied bulk fields and native handles only; glide orchestration is in suggest::glide_ranker.
use crate::{
    handle::locked,
    jni::{self, Result, illegal},
};
use jni_sys::{JNIEnv, jboolean, jclass, jfloatArray, jint, jintArray, jlong, jstring};
use std::ptr;
use suggest::{
    glide_ranker::{self, Query},
    learned_words::LearnedWords,
    previous_words::PreviousWords,
    raw::NOT_A_PROBABILITY,
    session::Session,
};

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_NativeGlideNative_rank(
    env: *mut JNIEnv,
    _: jclass,
    session: jlong,
    history: jlong,
    text: jstring,
    lengths: jintArray,
    costs: jfloatArray,
    word_count: jint,
    sentence_start: jboolean,
) -> jintArray {
    // SAFETY: Kotlin holds the session monitor and a reachable history owner for this entire call.
    // Handles are live or zero after close; -1 is the explicit absent dictionary, never dereferenced.
    unsafe {
        jni::run(env, ptr::null_mut(), |e| {
            let text = e.read_string(text, "glide fields")?;
            let lengths = e.read::<i32>(lengths, "glide lengths")?;
            let fields = jni::cut(&text, &lengths)?;
            let costs = e.read::<f32>(costs, "glide costs")?;
            let count = usize::try_from(word_count).map_err(|_| illegal("negative glide count"))?;
            if count != costs.len() {
                return Err(illegal("glide words and costs differ in length"));
            }
            let (words, previous) = fields
                .split_at_checked(count)
                .ok_or_else(|| illegal("missing glide words"))?;
            let previous = PreviousWords {
                words: previous.iter().map(|w| w.to_vec()).collect(),
                is_beginning_of_sentence: sentence_start != 0,
            };
            let order =
                glide_ranker::rank_from(&costs, words, &previous, |query| -> Result<i32> {
                    Ok(match query {
                        Query::Ngram(p, w) => {
                            if session == -1 {
                                NOT_A_PROBABILITY
                            } else {
                                locked::<Session>(session, "session")?.context_probability(p, w)
                            }
                        }
                        Query::Probability(w) => {
                            if session == -1 {
                                NOT_A_PROBABILITY
                            } else {
                                locked::<Session>(session, "session")?.probability(w)
                            }
                        }
                        Query::Pair(p, w) => {
                            if history == 0 {
                                0
                            } else {
                                locked::<LearnedWords>(history, "learned words")?.pair_count(
                                    p.words.first().map(Vec::as_slice),
                                    p.is_beginning_of_sentence,
                                    w,
                                )
                            }
                        }
                    })
                })?
                .ok_or_else(|| illegal("glide words and costs differ in length"))?;
            e.make(&order.into_iter().map(|i| i as i32).collect::<Vec<_>>())
        })
    }
}
