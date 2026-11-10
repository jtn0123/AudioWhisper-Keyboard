//! The pure logic of `latinime_BinaryDictionary_getSuggestions`
//! (`com_android_inputmethod_latin_BinaryDictionary.cpp`) and of
//! `SuggestionResults::outputSuggestions` / `JniDataUtils::outputCodePoints`: what the Java side
//! passes in, what the decoder does with it, and the arrays it reads back. [`suggest`] is shaped
//! like the oracle's `suggest`, over a [`DicTraverseSession`] the caller keeps between calls (the
//! node cache continues from keystroke to keystroke through it).
//!
//! Intentional differences from the JNI method: there is no null dictionary, session or array to
//! return early on (they are references); a query longer than `MAX_WORD_LENGTH` is cut to it (the
//! C++ overruns its arrays); and a gesture option (`isGesture`, which the app never sets and which
//! is not ported) returns no suggestions.

use crate::chars;
use crate::defines::{
    CODE_POINT_BEGINNING_OF_SENTENCE, MAX_PREV_WORD_COUNT_FOR_N_GRAM, MAX_RESULTS, MAX_WORD_LENGTH,
    NOT_A_COORDINATE, NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL,
};
use crate::layout::proximity_info::ProximityInfo;
use crate::layout::proximity_info_state::TypingInput;
use crate::search::dictionary::get_predictions;
use crate::search::session::DicTraverseSession;
use crate::search::suggest::get_suggestions;
use crate::search::suggest_options::SuggestOptions;
use crate::trie::dictionary::Dictionary;
use crate::trie::ngram::NgramContext;

/// `CODE_POINT_REPLACEMENT_CHARACTER`.
const CODE_POINT_REPLACEMENT_CHARACTER: i32 = 0xFFFD;

/// The suggest options the app passes (`NativeSuggestOptions`): the third is blocking offensive
/// words, the fifth the weight for the locale in thousands.
const OPTION_BLOCK_OFFENSIVE_WORDS: usize = 2;
/// The index of `SuggestOptions::WEIGHT_FOR_LOCALE_IN_THOUSANDS`.
const OPTION_WEIGHT_FOR_LOCALE_IN_THOUSANDS: usize = 4;
/// The length of the options array the app passes.
const OPTION_COUNT: usize = 5;

/// What the app hands the decoder for one call.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    /// The typed code points, at most `MAX_WORD_LENGTH`.
    pub code_points: Vec<i32>,
    /// Where each was touched, or `None` for no coordinate.
    pub points: Vec<Option<(i32, i32)>>,
    pub block_offensive_words: bool,
    /// The previous words, most recent first, at most three.
    pub previous: Vec<Vec<i32>>,
    /// One flag per previous word: the slot is the start of a sentence.
    pub beginning_of_sentence: Vec<bool>,
}

/// The output arrays after one call, as Java reads them back.
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
    /// The arrays as Java allocates them: all zero, the weight at its input.
    pub fn new(weight: f32) -> Self {
        Self {
            count: 0,
            code_points: vec![0; MAX_WORD_LENGTH * MAX_RESULTS],
            scores: vec![0; MAX_RESULTS],
            space_indices: vec![0; MAX_RESULTS],
            types: vec![0; MAX_RESULTS],
            auto_commit_confidence: 0,
            weight,
        }
    }
}

/// `BinaryDictionary.getSuggestions` as the app calls it: options `[0, 0, block, 0, 1000]` and the
/// weight NOT_A_WEIGHT, so the scoring chooses it.
pub fn suggest(
    dictionary: &Dictionary,
    info: &ProximityInfo,
    session: &mut DicTraverseSession,
    query: &Query,
) -> Output {
    let mut options = [0; OPTION_COUNT];
    options[OPTION_BLOCK_OFFENSIVE_WORDS] = i32::from(query.block_offensive_words);
    options[OPTION_WEIGHT_FOR_LOCALE_IN_THOUSANDS] = 1000;
    suggest_with(
        dictionary,
        info,
        session,
        query,
        &options,
        NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL,
    )
}

/// `latinime_BinaryDictionary_getSuggestions` with the options array and the in/out weight given.
/// The times and pointer ids are the zeros the app passes.
pub fn suggest_with(
    dictionary: &Dictionary,
    info: &ProximityInfo,
    session: &mut DicTraverseSession,
    query: &Query,
    options: &[i32],
    weight: f32,
) -> Output {
    let mut output = Output::new(weight);
    suggest_into(dictionary, Some(info), session, query, options, &mut output);
    output
}

/// [`suggest_with`] writing into arrays the caller holds, as the JNI method writes into Java's:
/// what the C++ doesn't write (past the last suggestion, past each word's terminator, the
/// confidence when there is no suggestion) keeps its value. `output.weight` is the weight in.
///
/// Without a keyboard (a null `proximityInfo`), only predictions run: the C++ reads through the
/// null pointer for typed input, so this leaves `output` as it is instead.
pub fn suggest_into(
    dictionary: &Dictionary,
    info: Option<&ProximityInfo>,
    session: &mut DicTraverseSession,
    query: &Query,
    options: &[i32],
    output: &mut Output,
) {
    output.count = 0;
    let suggest_options = SuggestOptions::new(options);
    if suggest_options.is_gesture() {
        return;
    }
    let size = query
        .code_points
        .len()
        .min(query.points.len())
        .min(MAX_WORD_LENGTH);
    let mut prev: [Option<&[i32]>; MAX_PREV_WORD_COUNT_FOR_N_GRAM] =
        [None; MAX_PREV_WORD_COUNT_FOR_N_GRAM];
    let prev_count = query.previous.len().min(MAX_PREV_WORD_COUNT_FOR_N_GRAM);
    for (slot, word) in prev.iter_mut().zip(&query.previous) {
        *slot = Some(word);
    }
    let ngram_context = NgramContext::new(&prev[..prev_count], &query.beginning_of_sentence);
    if size > 0 {
        let Some(info) = info else {
            return;
        };
        let mut xs = [NOT_A_COORDINATE; MAX_WORD_LENGTH];
        let mut ys = [NOT_A_COORDINATE; MAX_WORD_LENGTH];
        for (index, point) in query.points.iter().take(size).enumerate() {
            if let Some((x, y)) = point {
                (xs[index], ys[index]) = (*x, *y);
            }
        }
        let zeros = [0; MAX_WORD_LENGTH];
        let input = TypingInput {
            codes: &query.code_points[..size],
            xs: &xs[..size],
            ys: &ys[..size],
            times: Some(&zeros[..size]),
            pointer_ids: Some(&zeros[..size]),
        };
        get_suggestions(
            dictionary,
            info,
            session,
            &input,
            &ngram_context,
            suggest_options,
            output.weight,
        );
    } else {
        session.results.clear();
        get_predictions(dictionary.policy(), &ngram_context, &mut session.results);
    }
    write_output(session, output);
}

/// `SuggestionResults::outputSuggestions`: pops the results worst first into the arrays, so the
/// best ends last.
fn write_output(session: &mut DicTraverseSession, output: &mut Output) {
    let results = &mut session.results;
    let mut index = 0;
    while let Some(word) = results.top() {
        if index >= MAX_RESULTS {
            break;
        }
        let start = index * MAX_WORD_LENGTH;
        output_code_points(
            &mut output.code_points[start..start + MAX_WORD_LENGTH],
            word.code_points(),
        );
        output.scores[index] = word.score();
        output.space_indices[index] = word.index_to_partial_commit();
        output.types[index] = word.kind();
        if results.suggestion_count() == 1 {
            output.auto_commit_confidence = word.auto_commit_first_word_confidence();
        }
        index += 1;
        results.pop();
    }
    output.count = index as i32;
    output.weight = results.weight_of_lang_model_vs_spatial_model();
}

/// `JniDataUtils::outputCodePoints` with null termination: skips the beginning-of-sentence marker,
/// writes U+FFFD for what is not a code point or is a control code, and a 0 after the last unless
/// the word fills `out`.
pub(super) fn output_code_points(out: &mut [i32], code_points: &[i32]) {
    let mut count = 0;
    for &code_point in code_points.iter().take(out.len()) {
        let mut to_output = code_point;
        if !chars::is_in_unicode_space(code_point) {
            if code_point == CODE_POINT_BEGINNING_OF_SENTENCE {
                continue;
            }
            to_output = CODE_POINT_REPLACEMENT_CHARACTER;
        } else if (0x01..=0x1F).contains(&code_point) {
            to_output = CODE_POINT_REPLACEMENT_CHARACTER;
        }
        out[count] = to_output;
        count += 1;
    }
    if count < out.len() {
        out[count] = 0;
    }
}
