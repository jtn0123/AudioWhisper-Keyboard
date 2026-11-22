//! Typed inputs: both states given the same taps, fresh, letter by letter, and after a decode.

use super::*;
use latin_golden::queries::Query;

/// What a decode is given: the typed code points and taps, as the oracle's queries have them.
struct Typed {
    codes: Vec<i32>,
    xs: Vec<i32>,
    ys: Vec<i32>,
    times: Option<Vec<i32>>,
    pointer_ids: Option<Vec<i32>>,
}

impl Typed {
    fn of(query: &Query) -> Self {
        Self {
            codes: query.code_points.clone(),
            xs: query
                .points
                .iter()
                .map(|it| it.map_or(-1, |it| it.0))
                .collect(),
            ys: query
                .points
                .iter()
                .map(|it| it.map_or(-1, |it| it.1))
                .collect(),
            times: Some(vec![0; query.code_points.len()]),
            pointer_ids: Some(vec![0; query.code_points.len()]),
        }
    }

    /// The first `length` taps.
    fn prefix(&self, length: usize) -> Self {
        Self {
            codes: self.codes[..length].to_vec(),
            xs: self.xs[..length].to_vec(),
            ys: self.ys[..length].to_vec(),
            times: self.times.as_ref().map(|it| it[..length].to_vec()),
            pointer_ids: None,
        }
    }

    fn input(&self) -> TypingInput<'_> {
        TypingInput {
            codes: &self.codes,
            xs: &self.xs,
            ys: &self.ys,
            times: self.times.as_deref(),
            pointer_ids: self.pointer_ids.as_deref(),
        }
    }

    #[cfg(feature = "oracle")]
    fn state_input<'a>(&'a self, locale: &'a [i32]) -> cpp::StateInput<'a> {
        cpp::StateInput {
            max_point_to_key_length: 1.0,
            codes: &self.codes,
            xs: &self.xs,
            ys: &self.ys,
            times: self.times.as_deref(),
            pointer_ids: self.pointer_ids.as_deref(),
            locale,
        }
    }
}

/// One typing state each side, kept across calls as the keyboard's session keeps one.
struct Typing {
    ours: ProximityInfoState,
    #[cfg(feature = "oracle")]
    theirs: cpp::State,
}

impl Typing {
    fn new() -> Self {
        Self {
            ours: ProximityInfoState::new(),
            #[cfg(feature = "oracle")]
            theirs: cpp::State::new(),
        }
    }
}

/// Initialises both states with the same input and records every value.
fn init_and_compare(
    (tape, random): (&mut Tape, &mut Random),
    board: &Board,
    typing: &mut Typing,
    typed: &Typed,
    locale: &[i32],
) {
    typing
        .ours
        .init_input_params(1.0, &board.ours, &typed.input(), locale);
    if typed.codes.is_empty() {
        // The C++ pushes a point read from outside its empty arrays; the Rust has none.
        assert_eq!(typing.ours.size(), 0);
        return;
    }
    #[cfg(feature = "oracle")]
    typing
        .theirs
        .init(board.theirs(), &typed.state_input(locale));
    let context = (&typed.codes, &typed.xs, &typed.ys, &typed.times, locale);
    let states = States {
        ours: &typing.ours,
        #[cfg(feature = "oracle")]
        theirs: &typing.theirs,
    };
    assert_states_match(tape, random, &states, &board.ours, &typed.codes, &context);
}

/// The AOSP dictionary's words, as the questions draw them.
fn words() -> Vec<Vec<char>> {
    let file = std::fs::read(latin_golden::aosp_dictionary_path()).expect("the AOSP dictionary");
    let dictionary =
        crate::trie::dictionary::Dictionary::open(file.into()).expect("a valid dictionary");
    let text = String::from_utf16(&crate::trie::jni::get_words(&dictionary)).expect("UTF-16 words");
    text.lines().map(|it| it.chars().collect()).collect()
}

#[test]
fn typed_inputs_match_the_goldens() {
    let board = Board::qwerty();
    let words = words();
    let locales = locales();
    let mut tape = tape("layout.typed_inputs");
    let mut random = Random::new(20_261_111);
    for round in 0..25_000 {
        let query = queries::query(&mut random, &words);
        let mut typed = Typed::of(&query);
        match round % 5 {
            0 => (typed.times, typed.pointer_ids) = (None, None),
            1 => typed.times = Some((0..typed.codes.len() as i32).map(|it| it * 90).collect()),
            _ => {}
        }
        let locale = &locales[round % locales.len()];
        let sides = (&mut tape, &mut random);
        init_and_compare(sides, &board, &mut Typing::new(), &typed, locale);
    }
    tape.finish();
}

/// A word typed letter by letter on one state, as the keyboard does: each call's input starts
/// with the last one's, so the C++ keeps its points; sometimes a tap, a time or a letter before
/// the end changes, or the word is cut short or replaced.
#[test]
fn words_typed_letter_by_letter_match_the_goldens() {
    let board = Board::qwerty();
    let words = words();
    let mut tape = tape("layout.typed_letter_by_letter");
    let mut random = Random::new(99);
    let mut typing = Typing::new();
    let mut continued = 0;
    for round in 0..2_500 {
        let query = queries::query(&mut random, &words);
        let mut typed = Typed::of(&query);
        let times: Vec<i32> = (0..typed.codes.len() as i32)
            .map(|it| 40 + it * 120)
            .collect();
        typed.times = (round % 3 != 0).then_some(times);
        for length in 1..=typed.codes.len() {
            let mut prefix = typed.prefix(length);
            if length > 2 && random.chance(10) {
                let at = random.below(length);
                match random.below(3) {
                    0 => prefix.xs[at] += 1,
                    1 => prefix.codes[at] = b'q'.into(),
                    _ => {
                        if let Some(times) = prefix.times.as_mut() {
                            times[at] += 5;
                        }
                    }
                }
            }
            let sides = (&mut tape, &mut random);
            init_and_compare(sides, &board, &mut typing, &prefix, &EN_US);
            continued += usize::from(typing.ours.is_continuous_suggestion_possible());
        }
    }
    tape.finish();
    assert!(
        continued > 5_000,
        "only {continued} calls could continue the last"
    );
}

/// A word typed in full and then deleted back letter by letter (and the same input twice), on
/// one state: the points kept from the last call are never fewer than the new input's.
#[test]
fn words_deleted_letter_by_letter_match_the_goldens() {
    let board = Board::qwerty();
    let words = words();
    let mut tape = tape("layout.deleted_letter_by_letter");
    let mut random = Random::new(1234);
    let mut typing = Typing::new();
    let mut continued = 0;
    for _ in 0..1_500 {
        let query = queries::query(&mut random, &words);
        let typed = Typed::of(&query);
        let full = typed.codes.len();
        if full == 0 {
            continue;
        }
        for length in std::iter::once(full).chain((1..full).rev()) {
            let prefix = typed.prefix(length);
            let repeats = if length > 2 { 2 } else { 1 };
            for _ in 0..repeats {
                let sides = (&mut tape, &mut random);
                init_and_compare(sides, &board, &mut typing, &prefix, &EN_US);
                continued += usize::from(typing.ours.is_continuous_suggestion_possible());
            }
        }
    }
    tape.finish();
    assert!(
        continued > 500,
        "only {continued} calls could continue the last"
    );
}

/// The state a decode leaves behind in the C++ session, against the Rust one given the same
/// input, over a run of decodes on one session.
#[test]
fn states_after_a_decode_match_the_goldens() {
    let board = Board::qwerty();
    #[cfg(feature = "oracle")]
    let (proximity, dictionary) = (
        latin_oracle::Proximity::qwerty(),
        latin_oracle::Dictionary::aosp(),
    );
    #[cfg(feature = "oracle")]
    let session = latin_oracle::Session::new("en_US", &dictionary);
    let words = words();
    let mut tape = tape("layout.states_after_a_decode");
    let mut random = Random::new(5);
    let mut ours = ProximityInfoState::new();
    for _ in 0..400 {
        let mut query = queries::query(&mut random, &words);
        query.previous.clear();
        query.beginning_of_sentence.clear();
        if query.code_points.is_empty() {
            continue;
        }
        #[cfg(feature = "oracle")]
        latin_oracle::suggest(
            &dictionary,
            &proximity,
            &session,
            &crate::golden::oracle_query(&query),
        );
        let typed = Typed::of(&query);
        ours.init_input_params(1.0, &board.ours, &typed.input(), &EN_US);
        #[cfg(feature = "oracle")]
        let theirs = cpp::State::of_session(&session);
        let states = States {
            ours: &ours,
            #[cfg(feature = "oracle")]
            theirs: &theirs,
        };
        let info = &board.ours;
        assert_states_match(&mut tape, &mut random, &states, info, &typed.codes, &query);
    }
    tape.finish();
}

#[test]
fn typed_inputs_on_random_keyboards_match_the_goldens() {
    let mut tape = tape("layout.typed_on_random_keyboards");
    let mut random = Random::new(31);
    let locales = locales();
    for _ in 0..1_500 {
        let board = Board::of(random_keyboard(&mut random));
        let mut typing = Typing::new();
        for _ in 0..6 {
            let typed = random_typed(&mut random, &board.keyboard);
            let locale = &locales[random.below(locales.len())];
            let sides = (&mut tape, &mut random);
            init_and_compare(sides, &board, &mut typing, &typed, locale);
        }
    }
    tape.finish();
}

/// Taps anywhere on and around `keyboard`, some with no x, with times and pointers or not.
fn random_typed(random: &mut Random, keyboard: &Keyboard) -> Typed {
    let length = match random.below(10) {
        0 => MAX_WORD_LENGTH,
        _ => 1 + random.below(10),
    };
    let codes: Vec<i32> = (0..length).map(|_| support::pool_code(random)).collect();
    let (xs_range, ys_range) = (keyboard.x_range(30), keyboard.y_range(30));
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for _ in 0..length {
        if random.chance(15) {
            xs.push(-1);
            ys.push(random.below(100) as i32);
        } else {
            xs.push(xs_range.start + random.below(xs_range.len()) as i32);
            ys.push(ys_range.start + random.below(ys_range.len()) as i32);
        }
    }
    if random.chance(10) {
        (xs[0], ys[0]) = (-1, -1);
    }
    let times = random
        .chance(60)
        .then(|| (0..length as i32).map(|it| it * 30).collect());
    let pointer_ids = random.chance(15).then(|| {
        (0..length)
            .map(|_| i32::from(random.chance(30)))
            .collect::<Vec<i32>>()
    });
    Typed {
        codes,
        xs,
        ys,
        times,
        pointer_ids,
    }
}
