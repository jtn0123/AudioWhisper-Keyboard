//! The session with the n-gram model the app ships (`app/src/main/assets/dicts/main_en-US.lm`)
//! on the dictionary it ships, against the same session without it.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use suggest::dictionary::{Dictionary, Purpose};
use suggest::language_model::Tuning;
use suggest::learned_words::LearnedWords;
use suggest::personal_words::PersonalWords;
use suggest::previous_words::PreviousWords;
use suggest::raw::{KIND_PREDICTION, NOT_A_PROBABILITY, RawSuggestion};
use suggest::session::Session;
use suggest::suggester::{Suggester, TypingState};

fn asset(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../app/src/main/assets/dicts")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn model() -> Arc<ngram::Model> {
    Arc::new(ngram::Model::open(asset("main_en-US.lm")).expect("the shipped model opens"))
}

fn plain() -> Session {
    Session::open(asset("main_en-US.dict").into()).expect("the shipped dictionary opens")
}

fn with_model(tuning: Tuning) -> Session {
    let mut session = plain();
    session.attach_tuned(model(), tuning);
    session
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

fn text(results: &[RawSuggestion], count: usize) -> Vec<String> {
    (results.iter().take(count))
        .map(|r| String::from_utf16(&r.word).unwrap())
        .collect()
}

fn decode(
    session: &mut Session,
    before: &str,
    typed: &str,
    purpose: Purpose,
) -> Vec<RawSuggestion> {
    let previous = PreviousWords::from(&utf16(before));
    (session.request(&previous, true, &[]))
        .decode(&utf16(typed), purpose)
        .unwrap()
}

fn predicted(session: &mut Session, before: &str, count: usize) -> Vec<String> {
    text(&decode(session, before, "", Purpose::Predicting), count)
}

#[test]
fn predicts_what_usually_comes_next() {
    let mut session = with_model(Tuning::DEFAULT);
    assert_eq!(predicted(&mut session, "Thank ", 1), ["you"]);
    assert_eq!(predicted(&mut session, "How are ", 1), ["you"]);
    assert!(predicted(&mut session, "I don't ", 3).contains(&"know".to_string()));
    let see_you = predicted(&mut session, "See you ", 6);
    for word in ["again", "tomorrow"] {
        assert!(see_you.contains(&word.to_string()), "{see_you:?}");
    }
    let results = decode(&mut session, "on the ", "", Purpose::Predicting);
    assert!(results.iter().all(|r| r.kind() == KIND_PREDICTION));
    assert!(results.windows(2).all(|w| w[0].score >= w[1].score));
    assert!(results.len() > 6);
}

/// The dictionary knows next words for one word only, so after "you" it says the same whatever
/// came before; the model reads the word before too.
#[test]
fn predictions_follow_more_than_the_last_word() {
    let contexts = ["See you ", "Can you ", "What do you "];
    let mut without = plain();
    let first = predicted(&mut without, contexts[0], 3);
    for before in contexts {
        assert_eq!(predicted(&mut without, before, 3), first, "{before}");
    }
    let mut session = with_model(Tuning::DEFAULT);
    let tops: Vec<_> = (contexts.iter())
        .map(|before| predicted(&mut session, before, 3))
        .collect();
    for (i, a) in tops.iter().enumerate() {
        assert_eq!(a.len(), 3);
        for b in &tops[i + 1..] {
            assert_ne!(a, b);
        }
    }
    assert_eq!(predicted(&mut session, "Can you ", 1), ["tell"]);
}

#[test]
fn no_context_means_no_predictions() {
    let mut session = with_model(Tuning::DEFAULT);
    let results = (session.request(&PreviousWords::NONE, true, &[]))
        .decode(&[], Purpose::Predicting)
        .unwrap();
    assert!(results.is_empty());
    // After a word the model lacks, the commonest words.
    assert!(!predicted(&mut session, "Zyzzogeton ", 3).is_empty());
}

#[test]
fn typing_in_context_prefers_the_word_that_fits() {
    let mut without = plain();
    let mut session = with_model(Tuning::DEFAULT);
    let before = "I want to ";
    assert_eq!(
        text(&decode(&mut without, before, "fo", Purpose::Typing), 1),
        ["to"]
    );
    let typed = text(&decode(&mut session, before, "fo", Purpose::Typing), 4);
    let at = |word: &str| typed.iter().position(|w| w == word);
    assert!(at("go") < at("to"), "{typed:?}");
    assert_ne!(typed[0], "to");
}

/// The model only redoes the language term: with none of its lift taken, typing is what it was,
/// and the spell checker, which reads no context, never sees it.
#[test]
fn without_the_model_s_lift_typing_is_unchanged() {
    let mut without = plain();
    let mut untuned = with_model(Tuning {
        typing: 0.0,
        swipe: 0.0,
    });
    let mut session = with_model(Tuning::DEFAULT);
    for (before, typed) in [
        ("I want to ", "fo"),
        ("Thank ", "tou"),
        ("", "helo"),
        ("on the ", "tabke"),
    ] {
        let expected = decode(&mut without, before, typed, Purpose::Typing);
        assert_eq!(
            decode(&mut untuned, before, typed, Purpose::Typing),
            expected
        );
        let spelling = decode(&mut without, before, typed, Purpose::Spelling);
        assert_eq!(
            decode(&mut session, before, typed, Purpose::Spelling),
            spelling
        );
    }
}

#[test]
fn the_swipe_ranker_hears_the_model_or_else_the_dictionary() {
    let without = plain();
    let session = with_model(Tuning::DEFAULT);
    let previous = PreviousWords::from(&utf16("on the "));
    for word in ["table", "day", "the", "you"] {
        let word = utf16(word);
        assert_eq!(
            without.context_probability(&previous, &word),
            without.ngram_probability(&previous, &word)
        );
    }
    // The dictionary has no "the table"; the model finds it far likelier here than anywhere.
    let table = utf16("table");
    assert_eq!(
        without.ngram_probability(&previous, &table),
        NOT_A_PROBABILITY
    );
    assert!(session.context_probability(&previous, &table) > session.probability(&table));
    // "the the" is no likelier than "the".
    let the = utf16("the");
    assert_eq!(
        session.context_probability(&previous, &the),
        NOT_A_PROBABILITY
    );
}

/// The user's own next words still come before the model's.
#[test]
fn what_the_user_types_next_comes_first() {
    let mut session = with_model(Tuning::DEFAULT);
    let mut learned = LearnedWords::new(100, 100);
    for _ in 0..2 {
        learned.add(&utf16("Wombats"), Some(&utf16("see")), false);
    }
    let learned = Mutex::new(learned);
    let personal = PersonalWords::new(Vec::new());
    let previous_words = PreviousWords::from(&utf16("I see "));
    let state = TypingState {
        typed: Vec::new(),
        previous_words: previous_words.clone(),
        is_cursor_mid_word: false,
    };
    let mut request = session.request(&previous_words, true, &[]);
    let mut suggester = Suggester {
        dictionary: &mut request,
        personal: &personal,
        learned: Some(&learned),
        threshold: suggest::auto_correction::MODEST_THRESHOLD,
    };
    let strip = suggester.suggest(&state, 5).unwrap();
    let words: Vec<String> = (strip.iter())
        .map(|a| String::from_utf16(&a.text).unwrap())
        .collect();
    assert_eq!(words.len(), 5);
    assert_eq!(words[0], "Wombats");
    assert!(words[1..].iter().all(|w| w != "Wombats"));
}
