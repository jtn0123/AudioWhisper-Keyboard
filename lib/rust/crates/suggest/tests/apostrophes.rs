//! Words typed without their apostrophe, on the dictionary and n-gram model the app ships: what
//! the language model decides where the letters also spell a word ("its", "ill"), and that
//! without it AOSP's decisions are unchanged.

use std::path::PathBuf;
use std::sync::Arc;

use suggest::auto_correction::Strength;
use suggest::personal_words::PersonalWords;
use suggest::previous_words::PreviousWords;
use suggest::session::Session;
use suggest::suggester::{Suggester, TypingState};

fn asset(name: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../app/src/main/assets/dicts")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn session(with_model: bool) -> Session {
    let mut session = Session::open(asset("main_en-US.dict").into()).expect("the dictionary opens");
    if with_model {
        session.attach(Arc::new(
            ngram::Model::open(asset("main_en-US.lm")).expect("the model opens"),
        ));
    }
    session
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// What `typed` after `before` becomes when the space is pressed.
fn ended(session: &mut Session, before: &str, typed: &str) -> String {
    let previous_words = PreviousWords::from(&utf16(before));
    let state = TypingState {
        typed: utf16(typed),
        previous_words: previous_words.clone(),
        is_cursor_mid_word: false,
    };
    let personal = PersonalWords::new(Vec::new());
    let mut request = session.request(&previous_words, true, &[]);
    let threshold = Strength::DEFAULT.threshold();
    let mut suggester = Suggester {
        dictionary: &mut request,
        personal: &personal,
        learned: None,
        threshold,
    };
    let strip = suggester.suggest(&state, 1).unwrap();
    let correction = strip.into_iter().find(|it| it.is_auto_correction);
    correction.map_or_else(
        || typed.to_string(),
        |it| String::from_utf16(&it.text).unwrap(),
    )
}

/// (before, typed, what it becomes).
fn check(session: &mut Session, cases: &[(&str, &str, &str)]) {
    for &(before, typed, expected) in cases {
        assert_eq!(
            ended(session, before, typed),
            expected,
            "{before:?} {typed:?}"
        );
    }
}

#[test]
fn contractions_that_spell_no_word_are_put_right_as_before() {
    let mut session = session(true);
    check(
        &mut session,
        &[
            ("I ", "dont", "don't"),
            ("", "im", "I'm"),
            ("I think ", "youre", "you're"),
            ("I think ", "theyre", "they're"),
            ("I ", "cant", "can't"),
            ("I ", "wont", "won't"),
            ("it ", "doesnt", "doesn't"),
            ("", "Ive", "I've"),
        ],
    );
}

#[test]
fn a_word_meant_as_typed_is_kept() {
    let mut session = session(true);
    check(
        &mut session,
        &[
            ("he is ", "ill", "ill"),
            ("she was ", "ill", "ill"),
            ("he ", "lets", "lets"),
            ("the dog wagged ", "its", "its"),
            ("we ", "were", "were"),
            ("they ", "were", "were"),
            ("very ", "well", "well"),
            ("the ", "shell", "shell"),
            ("go to ", "hell", "hell"),
            ("what the ", "hell", "hell"),
            ("the ", "shed", "shed"),
        ],
    );
}

#[test]
fn the_apostrophe_is_put_back_where_the_words_before_call_for_it() {
    let mut session = session(true);
    check(
        &mut session,
        &[
            ("I think ", "ill", "I'll"),
            ("", "Lets", "Let's"),
            ("", "Its", "It's"),
            ("I think ", "its", "it's"),
            ("I think ", "were", "we're"),
            ("I think ", "well", "we'll"),
            ("I think ", "shed", "she'd"),
            ("I think ", "hell", "he'll"),
        ],
    );
}

#[test]
fn without_the_model_aosp_decides_alone() {
    let mut session = session(false);
    check(
        &mut session,
        &[
            ("he is ", "ill", "I'll"),
            ("he ", "lets", "let's"),
            ("I think ", "its", "its"),
            ("I think ", "were", "were"),
            ("I ", "dont", "don't"),
        ],
    );
}
