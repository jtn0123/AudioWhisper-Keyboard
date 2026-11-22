//! A keyboard with a space bar. The app's keyboard model has none, so `hasSpaceProximity` is
//! always false there and space substitution ("hevis" -> "he is") never runs; with a space bar
//! under the bottom row it does.

use latin_golden::queries::{self, Random};

use super::support::{Fixture, TestKeyboard, in_chunks, record, tap};

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

/// The generator's queries and words typed as one, letter by letter on one session and
/// whole on fresh ones, on a keyboard with a space bar: each matches the goldens, and the space
/// expansions make some answers two words.
#[test]
fn space_bar_keyboard_matches_the_goldens() {
    let keyboard = TestKeyboard::new(true);
    for (name, path) in latin_golden::dictionaries() {
        let fixture = Fixture::at(name, &path, Some(&keyboard));
        let short_words: Vec<&Vec<char>> = fixture
            .words
            .iter()
            .filter(|word| {
                (2..=7).contains(&word.len()) && word.iter().all(char::is_ascii_lowercase)
            })
            .collect();
        let mut random = Random::new(37);
        let inputs: Vec<_> = (0..2_000)
            .map(|round| {
                let mut query = queries::query(&mut random, &fixture.words);
                if round % 3 != 0 {
                    (query.code_points, query.points) = joined(&mut random, &short_words);
                }
                query
            })
            .collect();
        // Letter by letter on each thread's shared session, then the whole query on fresh ones.
        let outputs = in_chunks(
            &inputs,
            || fixture.session(),
            |session, _, query| fixture.type_then_ask(session, query),
        );
        let mut tape = fixture.tape("space_bar_keyboard");
        record(&mut tape, outputs.iter().flatten());
        tape.finish();
        let two_words = outputs
            .iter()
            .filter(|outputs| {
                let whole = outputs.last().expect("the whole query's output");
                whole.code_points[..whole.count as usize * crate::defines::MAX_WORD_LENGTH]
                    .contains(&i32::from(b' '))
            })
            .count();
        let calls = outputs.len();
        eprintln!("{two_words} of {calls} answers held two words");
        assert!(
            two_words > calls / 2,
            "only {two_words} of {calls} answers held two words"
        );
    }
}
