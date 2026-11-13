//! Orchestration over existing native stores. No trip through managed probability callbacks.
use suggest::{personal_words::PersonalWords, raw::NOT_A_PROBABILITY, session::Session};

pub fn is_name(word: &[u16], personal: &PersonalWords, dictionary: Option<&Session>) -> bool {
    let lower = chars::lowercase(word);
    if lower == word {
        return false;
    }
    if let Some(spelling) = personal.spelling_of(word) {
        return spelling.first().is_some_and(|&u| chars::is_upper_case(u));
    }
    let Some(dictionary) = dictionary else {
        return true;
    };
    let written = dictionary.probability(word);
    let lowered = dictionary.probability(&lower);
    written == NOT_A_PROBABILITY && lowered == NOT_A_PROBABILITY || written > lowered
}

pub fn unknown_words(dictionary: &Session, words: Vec<Vec<u16>>) -> Vec<Vec<u16>> {
    words
        .into_iter()
        .filter(|word| dictionary.probability(word) == NOT_A_PROBABILITY)
        .collect()
}
