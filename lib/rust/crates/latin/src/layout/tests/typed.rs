//! Typed inputs: both states given the same taps, fresh, letter by letter, and after a decode.

#![cfg(not(target_os = "android"))]

use super::*;

/// What a decode is given: the typed code points and taps, as the oracle's queries have them.
#[cfg(not(target_os = "android"))]
struct Typed {
    codes: Vec<i32>,
    xs: Vec<i32>,
    ys: Vec<i32>,
    times: Option<Vec<i32>>,
    pointer_ids: Option<Vec<i32>>,
}

#[cfg(not(target_os = "android"))]
impl Typed {
    fn of(query: &latin_oracle::Query) -> Self {
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

    fn input(&self) -> TypingInput<'_> {
        TypingInput {
            codes: &self.codes,
            xs: &self.xs,
            ys: &self.ys,
            times: self.times.as_deref(),
            pointer_ids: self.pointer_ids.as_deref(),
        }
    }

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

/// Initialises both states with the same input and checks every value.
#[cfg(not(target_os = "android"))]
fn init_and_compare(
    random: &mut Random,
    info: &ProximityInfo,
    cpp_info: cpp::Info,
    pair: (&mut ProximityInfoState, &mut cpp::State),
    typed: &Typed,
    locale: &[i32],
) {
    let (ours, theirs) = pair;
    if typed.codes.is_empty() {
        // The C++ pushes a point read from outside its empty arrays; the Rust has none.
        ours.init_input_params(1.0, info, &typed.input(), locale);
        assert_eq!(ours.size(), 0);
        return;
    }
    ours.init_input_params(1.0, info, &typed.input(), locale);
    theirs.init(cpp_info, &typed.state_input(locale));
    let context = (&typed.codes, &typed.xs, &typed.ys, &typed.times, locale);
    assert_states_match(random, ours, info, theirs, &typed.codes, &context);
}

#[cfg(not(target_os = "android"))]
fn words() -> Vec<Vec<char>> {
    let dictionary = latin_oracle::Dictionary::aosp();
    let text = String::from_utf16(&dictionary.words()).expect("UTF-16 words");
    text.lines().map(|it| it.chars().collect()).collect()
}

#[cfg(not(target_os = "android"))]
#[test]
fn typed_inputs_match_the_cpp() {
    let keyboard = qwerty();
    let proximity = latin_oracle::Proximity::qwerty();
    let cpp_info = cpp::Info::of(&proximity);
    let info = ProximityInfo::new(&keyboard.geometry()).expect("a valid keyboard");
    let words = words();
    let locales = locales();
    let mut random = Random::new(20_261_111);
    for round in 0..25_000 {
        let query = queries::query(&mut random, &words);
        let mut typed = Typed::of(&query);
        match round % 5 {
            0 => (typed.times, typed.pointer_ids) = (None, None),
            1 => typed.times = Some((0..typed.codes.len() as i32).map(|it| it * 90).collect()),
            _ => {}
        }
        let (mut ours, mut theirs) = (ProximityInfoState::new(), cpp::State::new());
        let locale = &locales[round % locales.len()];
        init_and_compare(
            &mut random,
            &info,
            cpp_info,
            (&mut ours, &mut theirs),
            &typed,
            locale,
        );
    }
}

/// A word typed letter by letter on one state, as the keyboard does: each call's input starts
/// with the last one's, so the C++ keeps its points; sometimes a tap, a time or a letter before
/// the end changes, or the word is cut short or replaced.
#[cfg(not(target_os = "android"))]
#[test]
fn words_typed_letter_by_letter_match_the_cpp() {
    let keyboard = qwerty();
    let proximity = latin_oracle::Proximity::qwerty();
    let cpp_info = cpp::Info::of(&proximity);
    let info = ProximityInfo::new(&keyboard.geometry()).expect("a valid keyboard");
    let words = words();
    let mut random = Random::new(99);
    let (mut ours, mut theirs) = (ProximityInfoState::new(), cpp::State::new());
    let mut continued = 0;
    for round in 0..2_500 {
        let query = queries::query(&mut random, &words);
        let mut typed = Typed::of(&query);
        let times: Vec<i32> = (0..typed.codes.len() as i32)
            .map(|it| 40 + it * 120)
            .collect();
        typed.times = (round % 3 != 0).then_some(times);
        let locale = EN_US;
        for length in 1..=typed.codes.len() {
            let mut prefix = Typed {
                codes: typed.codes[..length].to_vec(),
                xs: typed.xs[..length].to_vec(),
                ys: typed.ys[..length].to_vec(),
                times: typed.times.as_ref().map(|it| it[..length].to_vec()),
                pointer_ids: None,
            };
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
            init_and_compare(
                &mut random,
                &info,
                cpp_info,
                (&mut ours, &mut theirs),
                &prefix,
                &locale,
            );
            continued += usize::from(ours.is_continuous_suggestion_possible());
        }
    }
    assert!(
        continued > 5_000,
        "only {continued} calls could continue the last"
    );
}

/// A word typed in full and then deleted back letter by letter (and the same input twice), on
/// one state: the points kept from the last call are never fewer than the new input's.
#[cfg(not(target_os = "android"))]
#[test]
fn words_deleted_letter_by_letter_match_the_cpp() {
    let keyboard = qwerty();
    let proximity = latin_oracle::Proximity::qwerty();
    let cpp_info = cpp::Info::of(&proximity);
    let info = ProximityInfo::new(&keyboard.geometry()).expect("a valid keyboard");
    let words = words();
    let mut random = Random::new(1234);
    let (mut ours, mut theirs) = (ProximityInfoState::new(), cpp::State::new());
    let mut continued = 0;
    for _ in 0..1_500 {
        let query = queries::query(&mut random, &words);
        let typed = Typed::of(&query);
        let full = typed.codes.len();
        if full == 0 {
            continue;
        }
        for length in std::iter::once(full).chain((1..full).rev()) {
            let prefix = Typed {
                codes: typed.codes[..length].to_vec(),
                xs: typed.xs[..length].to_vec(),
                ys: typed.ys[..length].to_vec(),
                times: typed.times.as_ref().map(|it| it[..length].to_vec()),
                pointer_ids: None,
            };
            let pair = (&mut ours, &mut theirs);
            init_and_compare(&mut random, &info, cpp_info, pair, &prefix, &EN_US);
            continued += usize::from(ours.is_continuous_suggestion_possible());
            if length > 2 {
                let pair = (&mut ours, &mut theirs);
                init_and_compare(&mut random, &info, cpp_info, pair, &prefix, &EN_US);
                continued += usize::from(ours.is_continuous_suggestion_possible());
            }
        }
    }
    assert!(
        continued > 500,
        "only {continued} calls could continue the last"
    );
}

/// The state a decode leaves behind, in the C++ session, against the Rust one given the same
/// input, over a run of decodes on one session.
#[cfg(not(target_os = "android"))]
#[test]
fn sessions_leave_the_state_the_rust_state_makes() {
    let keyboard = qwerty();
    let proximity = latin_oracle::Proximity::qwerty();
    let cpp_info = cpp::Info::of(&proximity);
    let info = ProximityInfo::new(&keyboard.geometry()).expect("a valid keyboard");
    let dictionary = latin_oracle::Dictionary::aosp();
    let session = latin_oracle::Session::new("en_US", &dictionary);
    let words = words();
    let mut random = Random::new(5);
    let mut ours = ProximityInfoState::new();
    for _ in 0..400 {
        let mut query = queries::query(&mut random, &words);
        query.previous.clear();
        query.beginning_of_sentence.clear();
        if query.code_points.is_empty() {
            continue;
        }
        latin_oracle::suggest(&dictionary, &proximity, &session, &query);
        let typed = Typed::of(&query);
        ours.init_input_params(1.0, &info, &typed.input(), &EN_US);
        let theirs = cpp::State::of_session(&session);
        assert_states_match(&mut random, &ours, &info, &theirs, &typed.codes, &query);
        let _ = cpp_info;
    }
}

#[cfg(not(target_os = "android"))]
#[test]
fn typed_inputs_on_random_keyboards_match_the_cpp() {
    let mut random = Random::new(31);
    let locales = locales();
    for round in 0..1_500 {
        let keyboard = random_keyboard(&mut random);
        let theirs_keyboard = cpp::Keyboard::new(&keyboard.raw());
        let info = ProximityInfo::new(&keyboard.geometry()).expect("a valid keyboard");
        let (mut ours, mut theirs) = (ProximityInfoState::new(), cpp::State::new());
        for _ in 0..6 {
            let length = match random.below(10) {
                0 => MAX_WORD_LENGTH,
                _ => 1 + random.below(10),
            };
            let codes: Vec<i32> = (0..length)
                .map(|_| support::pool_code(&mut random))
                .collect();
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
            let typed = Typed {
                codes,
                xs,
                ys,
                times,
                pointer_ids,
            };
            let locale = &locales[random.below(locales.len())];
            init_and_compare(
                &mut random,
                &info,
                theirs_keyboard.info(),
                (&mut ours, &mut theirs),
                &typed,
                locale,
            );
        }
        let _ = round;
    }
}
