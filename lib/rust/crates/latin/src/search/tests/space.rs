//! A keyboard with a space bar. The app's keyboard model has none, so `hasSpaceProximity` is
//! always false there and space substitution ("hevis" -> "he is") never runs; with a space bar
//! under the bottom row it does, and both sides are built from the same arrays to compare it.

use latin_oracle::queries::{self, Random};

use super::support::{TestKeyboard, assert_same, convert, in_chunks, new_session, tap, words_of};
use crate::trie::dictionary::Dictionary;

/// The y of a bottom-row tap close enough to the space bar for its grid cell to list the space.
const NEAR_SPACE_Y: i32 = 440;

/// Two words typed as one: run together (space omission), or with a bottom-row letter tapped
/// next to the space bar between them (space substitution), each letter nudged.
fn joined(random: &mut Random, short_words: &[&Vec<char>]) -> (Vec<i32>, Vec<Option<(i32, i32)>>) {
    let mut code_points: Vec<i32> = random
        .pick(short_words)
        .iter()
        .map(|&it| it as i32)
        .collect();
    let mut points: Vec<Option<(i32, i32)>> =
        code_points.iter().map(|&it| tap(random, it)).collect();
    if random.below(2) == 0 {
        let letter = b"zxcvbnm"[random.below(7)];
        code_points.push(i32::from(letter));
        let (x, _) = tap(random, i32::from(letter)).expect("a letter key");
        points.push(Some((x, NEAR_SPACE_Y)));
    }
    for &letter in random.pick(short_words).iter() {
        code_points.push(letter as i32);
        points.push(tap(random, letter as i32));
    }
    code_points.truncate(crate::defines::MAX_WORD_LENGTH);
    points.truncate(code_points.len());
    (code_points, points)
}

/// The generator's queries and words typed as one, letter by letter on one session per side and
/// whole on fresh ones, on a keyboard with a space bar: each matches the C++, and the space
/// expansions make some answers two words.
#[test]
fn space_bar_keyboard_matches_the_cpp() {
    let keyboard = TestKeyboard::new(true);
    let (cpp_proximity, info) = (keyboard.cpp_proximity(), keyboard.info());
    for path in latin_oracle::dictionary_paths() {
        let cpp_dictionary = latin_oracle::Dictionary::at(&path);
        let dictionary = Dictionary::open(std::fs::read(&path).expect("the dictionary").into())
            .expect("a valid dictionary");
        let all_words = words_of(&cpp_dictionary);
        let short_words: Vec<&Vec<char>> = all_words
            .iter()
            .filter(|word| {
                (2..=7).contains(&word.len()) && word.iter().all(char::is_ascii_lowercase)
            })
            .collect();
        let mut random = Random::new(37);
        let inputs: Vec<_> = (0..2_000)
            .map(|round| {
                let mut query = queries::query(&mut random, &all_words);
                if round % 3 != 0 {
                    (query.code_points, query.points) = joined(&mut random, &short_words);
                }
                query
            })
            .collect();
        // Letter by letter on each thread's shared sessions, then the whole query on fresh ones.
        let two_words_by_round = in_chunks(
            &inputs,
            || {
                let cpp = latin_oracle::Session::new("en_US", &cpp_dictionary);
                (cpp, new_session())
            },
            |(cpp_session, session), round, query| {
                for length in 1..=query.code_points.len() {
                    let input = latin_oracle::Query {
                        code_points: query.code_points[..length].to_vec(),
                        points: query.points[..length].to_vec(),
                        ..query.clone()
                    };
                    let theirs =
                        latin_oracle::suggest(&cpp_dictionary, &cpp_proximity, cpp_session, &input);
                    let ours =
                        crate::search::suggest(&dictionary, &info, session, &convert(&input));
                    assert_same(
                        &ours,
                        &theirs,
                        &format!("{} space {round}/{length}", path.display()),
                    );
                }
                let fresh = latin_oracle::Session::new("en_US", &cpp_dictionary);
                let theirs = latin_oracle::suggest(&cpp_dictionary, &cpp_proximity, &fresh, query);
                let ours =
                    crate::search::suggest(&dictionary, &info, &mut new_session(), &convert(query));
                assert_same(&ours, &theirs, &format!("{} space {round}", path.display()));
                ours.code_points[..ours.count as usize * crate::defines::MAX_WORD_LENGTH]
                    .contains(&i32::from(b' '))
            },
        );
        let calls = two_words_by_round.len();
        let two_words = two_words_by_round.iter().filter(|&&it| it).count();
        eprintln!("{two_words} of {calls} answers held two words");
        assert!(
            two_words > calls / 2,
            "only {two_words} of {calls} answers held two words"
        );
    }
}
