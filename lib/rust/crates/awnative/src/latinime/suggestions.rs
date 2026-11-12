//! `BinaryDictionary.getSuggestionsNative`: the typing search, or with no input the next-word
//! predictions, written into the session's Java arrays as the C++ writes them.

use jni_sys::{JNIEnv, jbooleanArray, jclass, jfloatArray, jint, jintArray, jlong, jobjectArray};
use latin::defines::{MAX_PREV_WORD_COUNT_FOR_N_GRAM, MAX_RESULTS, MAX_WORD_LENGTH};
use latin::layout::ProximityInfo;
use latin::search::{DicTraverseSession, Output, Query, suggest_into};
use latin::trie::dictionary::Dictionary;

use super::flags;
use crate::handle::{locked, object};
use crate::jni::{self, Env, Result, illegal};

/// The Java arrays the results go to.
struct Outputs {
    count: jintArray,
    code_points: jintArray,
    scores: jintArray,
    space_indices: jintArray,
    types: jintArray,
    auto_commit_confidence: jintArray,
    weight: jfloatArray,
}

impl Outputs {
    /// What the arrays hold now, which the search overwrites only in part. Reject malformed
    /// lengths before running the search or writing any result buffers.
    fn read(&self, env: &Env) -> Result<Output> {
        let code_points = env.read::<i32>(self.code_points, "outputCodePoints")?;
        let scores = env.read::<i32>(self.scores, "outputScores")?;
        let confidence = env.read::<i32>(self.auto_commit_confidence, "outputAutoCommit")?;
        if code_points.len() != MAX_WORD_LENGTH * MAX_RESULTS
            || scores.len() != MAX_RESULTS
            || confidence.len() != 1
        {
            return Err(illegal("an output array has an invalid length"));
        }
        let space_indices = env.read::<i32>(self.space_indices, "outputIndices")?;
        let types = env.read::<i32>(self.types, "outputTypes")?;
        if space_indices.len() < MAX_RESULTS || types.len() < MAX_RESULTS {
            return Err(illegal("an output array is shorter than MAX_RESULTS"));
        }
        let weight = env.read::<f32>(self.weight, "inOutWeight")?;
        let Some(&weight) = weight.first() else {
            return Err(illegal("inOutWeight is empty"));
        };
        Ok(Output {
            count: 0,
            code_points,
            scores,
            space_indices,
            types,
            auto_commit_confidence: confidence[0],
            weight,
        })
    }

    fn write(&self, env: &Env, output: &Output) -> Result<()> {
        env.write(self.code_points, &output.code_points, "outputCodePoints")?;
        env.write(self.scores, &output.scores, "outputScores")?;
        env.write(self.space_indices, &output.space_indices, "outputIndices")?;
        env.write(self.types, &output.types, "outputTypes")?;
        let confidence = [output.auto_commit_confidence];
        env.write(self.auto_commit_confidence, &confidence, "outputAutoCommit")?;
        env.write(self.count, &[output.count], "outputSuggestionCount")?;
        env.write(self.weight, &[output.weight], "inOutWeight")
    }
}

/// The inputs as Java passes them.
struct Inputs {
    xs: jintArray,
    ys: jintArray,
    code_points: jintArray,
    size: jint,
    prev_words: jobjectArray,
    beginning_of_sentence: jbooleanArray,
    prev_word_count: jint,
}

impl Inputs {
    /// The query: the first `size` taps (cut to `MAX_WORD_LENGTH`, where the C++ overruns), with
    /// their coordinates as they are (`NOT_A_COORDINATE` included), and the first
    /// `prev_word_count` previous words, a null one empty.
    fn read(&self, env: &Env) -> Result<Query> {
        let size = usize::try_from(self.size).unwrap_or(0);
        let mut query = Query::default();
        if size > 0 {
            let xs = env.read::<i32>(self.xs, "xCoordinates")?;
            let ys = env.read::<i32>(self.ys, "yCoordinates")?;
            let code_points = env.read::<i32>(self.code_points, "inputCodePoints")?;
            if xs.len() < size || ys.len() < size || code_points.len() < size {
                return Err(illegal("inputSize is longer than an input array"));
            }
            let size = size.min(MAX_WORD_LENGTH);
            query.code_points = code_points[..size].to_vec();
            query.points = xs[..size]
                .iter()
                .zip(&ys[..size])
                .map(|(&x, &y)| Some((x, y)))
                .collect();
        }
        let count = usize::try_from(self.prev_word_count)
            .unwrap_or(0)
            .min(MAX_PREV_WORD_COUNT_FOR_N_GRAM);
        if count > 0 {
            let mut words = env.read_arrays::<i32>(self.prev_words, "prevWordCodePointArrays")?;
            if words.len() < count {
                return Err(illegal(
                    "prevWordCount is longer than prevWordCodePointArrays",
                ));
            }
            words.truncate(count);
            query.beginning_of_sentence = flags(env, &words, self.beginning_of_sentence)?;
            query.previous = words.into_iter().map(Option::unwrap_or_default).collect();
        }
        Ok(query)
    }
}

/// `BinaryDictionary.getSuggestionsNative`. The count is 0 first; closed dictionaries and
/// sessions throw. The times and pointer ids are ignored: the app passes zeros, which is what
/// the search uses.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_android_inputmethod_latin_BinaryDictionary_getSuggestionsNative(
    env: *mut JNIEnv,
    _: jclass,
    dict: jlong,
    proximity_info: jlong,
    session: jlong,
    xs: jintArray,
    ys: jintArray,
    _times: jintArray,
    _pointer_ids: jintArray,
    code_points: jintArray,
    size: jint,
    options: jintArray,
    prev_words: jobjectArray,
    beginning_of_sentence: jbooleanArray,
    prev_word_count: jint,
    out_count: jintArray,
    out_code_points: jintArray,
    out_scores: jintArray,
    out_space_indices: jintArray,
    out_types: jintArray,
    out_auto_commit_confidence: jintArray,
    in_out_weight: jfloatArray,
) {
    let inputs = Inputs {
        xs,
        ys,
        code_points,
        size,
        prev_words,
        beginning_of_sentence,
        prev_word_count,
    };
    let outputs = Outputs {
        count: out_count,
        code_points: out_code_points,
        scores: out_scores,
        space_indices: out_space_indices,
        types: out_types,
        auto_commit_confidence: out_auto_commit_confidence,
        weight: in_out_weight,
    };
    // SAFETY: the JVM passes this call's environment; the handles are 0 or live ones from
    // `openNative`, `setProximityInfoNative` and `setDicTraverseSessionNative`.
    unsafe {
        jni::run(env, (), |env| {
            env.write(outputs.count, &[0], "outputSuggestionCount")?;
            if dict == 0 {
                return Err(illegal("the dictionary is closed"));
            }
            if session == 0 {
                return Err(illegal("the session is closed"));
            }
            let options = env.read::<i32>(options, "suggestOptions")?;
            let mut output = outputs.read(env)?;
            let query = inputs.read(env)?;
            let dictionary = object::<Dictionary>(dict, "dictionary")?;
            let info = match proximity_info {
                0 => None,
                handle => Some(object::<ProximityInfo>(handle, "proximityInfo")?),
            };
            let mut session = locked::<DicTraverseSession>(session, "session")?;
            suggest_into(
                dictionary,
                info,
                &mut session,
                &query,
                &options,
                &mut output,
            );
            drop(session);
            outputs.write(env, &output)
        })
    }
}
