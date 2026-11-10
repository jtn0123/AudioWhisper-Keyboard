//! German umlaut digraphs (`DigraphUtils`): a dictionary whose header asks for umlaut processing
//! tries each node holding "ä", "ö" or "ü" as "ae", "oe" or "ue" too. The bundled dictionary does
//! not ask for it, so the tests give it a header that does, on both sides.

use std::path::PathBuf;

use latin_oracle::queries;

use super::support::{assert_same, convert, in_chunks, new_session, qwerty_info, tap, words};
use crate::trie::dictionary::Dictionary;

/// The attribute `HeaderPolicy::requiresGermanUmlautProcessing` reads, set to 1, in the v2
/// header's string encoding (one byte per ASCII character, each string ended by 0x1F).
const UMLAUT_ATTRIBUTE: &[u8] = b"REQUIRES_GERMAN_UMLAUT_PROCESSING\x1f1\x1f";
/// Where the header's attributes start, and where its size is (`HeaderReadWriteUtils`).
const HEADER_OPTIONS_POSITION: usize = 12;
const HEADER_SIZE_POSITION: usize = 8;

/// The bundled dictionary with [`UMLAUT_ATTRIBUTE`] first among its attributes and its header
/// size grown to match; the body, which is read relative to its own start, is unchanged.
fn umlaut_dictionary() -> Vec<u8> {
    let bundled = std::fs::read(latin_oracle::bundled_dictionary_path()).expect("the dictionary");
    let size_field = HEADER_SIZE_POSITION..HEADER_SIZE_POSITION + 4;
    let header_size = u32::from_be_bytes(bundled[size_field.clone()].try_into().expect("4 bytes"));
    let mut bytes = bundled[..HEADER_OPTIONS_POSITION].to_vec();
    bytes.extend_from_slice(UMLAUT_ATTRIBUTE);
    bytes.extend_from_slice(&bundled[HEADER_OPTIONS_POSITION..]);
    let grown = header_size + UMLAUT_ATTRIBUTE.len() as u32;
    bytes[size_field].copy_from_slice(&grown.to_be_bytes());
    bytes
}

/// The file the C++ side opens, removed when dropped.
struct TemporaryFile(PathBuf);

impl Drop for TemporaryFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// The digraphs typed out: "ä" as "ae", "ö" as "oe", "ü" as "ue".
fn spelt_out(word: &[char]) -> Vec<i32> {
    let mut typed = Vec::new();
    for &letter in word {
        match letter {
            'ä' => typed.extend([i32::from(b'a'), i32::from(b'e')]),
            'ö' => typed.extend([i32::from(b'o'), i32::from(b'e')]),
            'ü' => typed.extend([i32::from(b'u'), i32::from(b'e')]),
            other => typed.push(other as i32),
        }
    }
    typed
}

/// Words with an umlaut typed with its digraph spelt out, and the generator's queries, against
/// a dictionary that processes umlauts: each matches the C++, and some answers hold an umlaut
/// that only the digraph could have reached.
#[test]
fn umlaut_digraphs_match_the_cpp() {
    let bytes = umlaut_dictionary();
    let file = TemporaryFile(
        std::env::temp_dir().join(format!("latin-search-umlaut-{}.dict", std::process::id())),
    );
    std::fs::write(&file.0, &bytes).expect("a temporary file");
    let size = i64::try_from(bytes.len()).expect("size fits");
    let cpp_dictionary = latin_oracle::Dictionary::open(&file.0, 0, size).expect("opens");
    let dictionary = Dictionary::open(bytes.into()).expect("opens");
    assert!(dictionary.header().requires_german_umlaut_processing());
    let (cpp_proximity, info) = (latin_oracle::Proximity::qwerty(), qwerty_info());
    let all_words = words();
    let umlaut_words: Vec<&Vec<char>> = all_words
        .iter()
        .filter(|word| word.iter().any(|it| matches!(it, 'ä' | 'ö' | 'ü')))
        .collect();
    assert!(
        umlaut_words.len() > 20,
        "{} umlaut words",
        umlaut_words.len()
    );
    let mut random = queries::Random::new(31);
    let inputs: Vec<_> = (0..3_000)
        .map(|round| {
            let mut query = queries::query(&mut random, &all_words);
            if round % 3 != 0 {
                query.code_points = spelt_out(random.pick(&umlaut_words));
                query.code_points.truncate(crate::defines::MAX_WORD_LENGTH);
                query.points = query
                    .code_points
                    .iter()
                    .map(|&it| tap(&mut random, it))
                    .collect();
            }
            query
        })
        .collect();
    let umlauts = [0xE4, 0xF6, 0xFC];
    // Letter by letter on each thread's shared sessions, then the whole word on fresh ones.
    let reached_by_round = in_chunks(
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
                let ours = crate::search::suggest(&dictionary, &info, session, &convert(&input));
                assert_same(&ours, &theirs, &format!("umlaut {round}/{length}"));
            }
            let fresh = latin_oracle::Session::new("en_US", &cpp_dictionary);
            let theirs = latin_oracle::suggest(&cpp_dictionary, &cpp_proximity, &fresh, query);
            let ours =
                crate::search::suggest(&dictionary, &info, &mut new_session(), &convert(query));
            assert_same(&ours, &theirs, &format!("umlaut {round}"));
            query.code_points.iter().all(|it| !umlauts.contains(it))
                && ours.code_points[..ours.count as usize * 48]
                    .iter()
                    .any(|it| umlauts.contains(it))
        },
    );
    let calls = reached_by_round.len();
    let reached = reached_by_round.iter().filter(|&&it| it).count();
    eprintln!(
        "{} umlaut words; {reached} of {calls} answers reached an umlaut",
        umlaut_words.len()
    );
    assert!(
        reached > calls / 10,
        "only {reached} of {calls} answers reached an umlaut"
    );
}
