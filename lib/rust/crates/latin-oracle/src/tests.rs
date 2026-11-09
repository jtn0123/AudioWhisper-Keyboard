use super::*;

fn code_points(word: &str) -> Vec<i32> {
    word.chars().map(|it| it as i32).collect()
}

fn typed(word: &str) -> Query {
    let code_points = code_points(word);
    let points = code_points
        .iter()
        .map(|&it| qwerty::key_center(it))
        .collect();
    Query {
        code_points,
        points,
        ..Query::default()
    }
}

fn words(output: &Output) -> Vec<String> {
    output
        .suggestions()
        .into_iter()
        .map(|(word, _, _)| {
            word.iter()
                .map(|&it| char::from_u32(it as u32).unwrap())
                .collect()
        })
        .collect()
}

#[test]
fn opens_the_bundled_dictionary_and_lists_its_words() {
    let dictionary = Dictionary::bundled();
    let text = String::from_utf16(&dictionary.words()).unwrap();
    let count = text.lines().count();
    assert!((150_000..170_000).contains(&count), "{count} words");
    assert!(text.lines().any(|it| it == "keyboard"));
}

#[test]
fn rejects_what_isnt_a_dictionary() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let size = std::fs::metadata(&manifest).unwrap().len() as i64;
    assert!(Dictionary::open(&manifest, 0, size).is_none());
}

#[test]
fn knows_common_words_and_pairs() {
    let dictionary = Dictionary::bundled();
    assert!(dictionary.probability(&code_points("the")) > 200);
    assert_eq!(
        dictionary.probability(&code_points("qzxv")),
        NOT_A_PROBABILITY
    );
    let pair = dictionary.ngram_probability(&[code_points("thank")], &[false], &code_points("you"));
    assert!(
        pair > dictionary.ngram_probability(
            &[code_points("thank")],
            &[false],
            &code_points("zebra")
        )
    );
}

#[test]
fn corrects_a_typo() {
    let dictionary = Dictionary::bundled();
    let proximity = Proximity::qwerty();
    let session = Session::new("en_US", &dictionary);
    let output = suggest(&dictionary, &proximity, &session, &typed("teh"));
    assert!(output.count > 0);
    assert!(
        words(&output).contains(&"the".to_string()),
        "{:?}",
        words(&output)
    );
    // The same question gets the same answer.
    assert_eq!(
        suggest(&dictionary, &proximity, &session, &typed("teh")),
        output
    );
}

#[test]
fn predicts_after_a_previous_word() {
    let dictionary = Dictionary::bundled();
    let proximity = Proximity::qwerty();
    let session = Session::new("en_US", &dictionary);
    let query = Query {
        previous: vec![code_points("thank")],
        beginning_of_sentence: vec![false],
        ..Query::default()
    };
    let output = suggest(&dictionary, &proximity, &session, &query);
    assert!(
        words(&output).contains(&"you".to_string()),
        "{:?}",
        words(&output)
    );
}

#[test]
fn query_lines_read_back() {
    let dictionary = Dictionary::bundled();
    let text = String::from_utf16(&dictionary.words()).unwrap();
    let words: Vec<Vec<char>> = text.lines().map(|it| it.chars().collect()).collect();
    let mut random = queries::Random::new(7);
    for _ in 0..2_000 {
        let query = queries::query(&mut random, &words);
        assert_eq!(queries::parse_query(&queries::query_line(&query)), query);
    }
}
