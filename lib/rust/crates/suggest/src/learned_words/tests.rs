use super::*;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn strings(words: &[&[u16]]) -> Vec<String> {
    words.iter().map(|w| String::from_utf16_lossy(w)).collect()
}

fn add(learned: &mut LearnedWords, word: &str, previous: Option<&str>, times: usize) {
    let previous = previous.map(utf16);
    for _ in 0..times {
        learned.add(&utf16(word), previous.as_deref(), false);
    }
}

fn after(word: &str) -> Vec<u16> {
    utf16(word)
}

#[test]
fn a_word_becomes_the_users_once_used_twice() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    add(&mut learned, "Zorblax", None, 1);
    assert_eq!(learned.spelling_of(&utf16("zorblax")), None);
    assert!(learned.completions(&utf16("zor"), 2).is_empty());
    add(&mut learned, "Zorblax", None, 1);
    assert_eq!(strings(&learned.known_spellings()), ["Zorblax"]);
    assert_eq!(
        learned.spelling_of(&utf16("ZORBLAX")),
        Some(&utf16("Zorblax")[..])
    );
    assert_eq!(strings(&learned.completions(&utf16("ZOR"), 2)), ["Zorblax"]);
    assert!(learned.completions(&utf16("zorblax"), 2).is_empty());
    assert!(learned.completions(&utf16("zor"), 0).is_empty());
    assert!(learned.completions(&[], 2).is_empty());
}

#[test]
fn ranks_most_used_then_most_recent() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    add(&mut learned, "formica", None, 3);
    add(&mut learned, "formal", None, 2);
    add(&mut learned, "formula", None, 2);
    assert_eq!(
        strings(&learned.completions(&utf16("form"), 1)),
        ["formica", "formula", "formal"]
    );
    add(&mut learned, "later", Some("you"), 2);
    add(&mut learned, "there", Some("you"), 1);
    add(&mut learned, "Hey", Some(""), 1);
    let you = after("You");
    assert_eq!(
        strings(&learned.followers(Some(&you), false, 1, 3).unwrap()),
        ["later", "there"]
    );
    assert_eq!(
        strings(&learned.followers(Some(&you), false, 1, 1).unwrap()),
        ["later"]
    );
    assert_eq!(
        strings(&learned.followers(None, true, 1, 3).unwrap()),
        ["Hey"]
    );
    assert!(learned.followers(None, false, 1, -1).unwrap().is_empty());
    assert_eq!(
        learned.followers(Some(&you), false, 1, -1),
        Err(NegativeCount(-1))
    );
    assert_eq!(
        learned.pair_count(Some(&after("YOU")), true, &utf16("Later")),
        2
    );
    assert_eq!(learned.pair_count(None, false, &utf16("later")), 0);
}

#[test]
fn remove_takes_back_one_use() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    add(&mut learned, "later", Some("you"), 2);
    let you = after("you");
    learned.remove(&utf16("later"), Some(&you));
    assert_eq!(learned.spelling_of(&utf16("later")), None);
    assert_eq!(learned.pair_count(Some(&you), false, &utf16("later")), 1);
    learned.remove(&utf16("later"), Some(&you));
    assert_eq!(learned.pair_count(Some(&you), false, &utf16("later")), 0);
    learned.remove(&utf16("never"), Some(&utf16("seen")));
    // Nothing is left at zero.
    assert_eq!(
        String::from_utf16_lossy(&learned.saved_text()),
        "# AudioWhisper learned words, v1\n"
    );
}

#[test]
fn keeping_the_spelling_keeps_the_earlier_one() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    add(&mut learned, "iPhone", None, 1);
    learned.add(&utf16("IPhone"), None, true);
    assert_eq!(
        learned.spelling_of(&utf16("iphone")),
        Some(&utf16("iPhone")[..])
    );
    learned.add(&utf16("IPhone"), None, false);
    assert_eq!(
        learned.spelling_of(&utf16("iphone")),
        Some(&utf16("IPhone")[..])
    );
}

#[test]
fn stores_only_words() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    let too_long = "x".repeat(MAX_WORD_LENGTH + 1);
    for word in ["mp3", "a b", "tab\there", &too_long, "--", ""] {
        add(&mut learned, word, Some("previous"), 2);
        assert_eq!(learned.spelling_of(&utf16(word)), None, "{word}");
    }
    assert!(is_storable(&utf16(&"x".repeat(MAX_WORD_LENGTH))));
    assert!(is_storable(&utf16("rock’n’roll")));
    add(&mut learned, "don't", Some("mp3"), 2);
    assert_eq!(
        learned.pair_count(Some(&utf16("mp3")), false, &utf16("don't")),
        0
    );
}

#[test]
fn the_least_recently_used_go_first() {
    let mut learned = LearnedWords::new(2, 1);
    add(&mut learned, "alpha", Some("one"), 2);
    add(&mut learned, "bravo", Some("two"), 2);
    add(&mut learned, "alpha", None, 2);
    add(&mut learned, "charlie", None, 2);
    assert_eq!(strings(&learned.known_spellings()), ["alpha", "charlie"]);
    assert_eq!(
        learned.pair_count(Some(&utf16("one")), false, &utf16("alpha")),
        0
    );
    assert_eq!(
        learned.pair_count(Some(&utf16("two")), false, &utf16("bravo")),
        2
    );
}

#[test]
fn counts_wrap_as_kotlin_ints_do() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    learned.merge(Saved::parse(&utf16(
        "w\t2147483647\tbig\np\t2147483647\t\tbig\n",
    )));
    add(&mut learned, "big", Some(""), 1);
    assert_eq!(learned.spelling_of(&utf16("big")), None);
    assert_eq!(learned.pair_count(None, true, &utf16("big")), i32::MIN);
    // One fewer than the smallest Int is the largest: still used, and plenty.
    learned.remove(&utf16("big"), None);
    assert_eq!(learned.spelling_of(&utf16("big")), Some(&utf16("big")[..]));
}

#[test]
fn saves_and_parses_everything() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    add(&mut learned, "Zorblax", Some(""), 2);
    add(&mut learned, "later", Some("You"), 3);
    let text = learned.saved_text();
    assert_eq!(
        String::from_utf16_lossy(&text),
        "# AudioWhisper learned words, v1\nw\t2\tZorblax\nw\t3\tlater\np\t2\t\tzorblax\np\t3\tyou\tlater\n"
    );
    let mut reloaded = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    reloaded.merge(Saved::parse(&text));
    assert_eq!(reloaded.saved_text(), text);
}

#[test]
fn merging_keeps_what_was_learned_since() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    add(&mut learned, "Later", Some("you"), 1);
    add(&mut learned, "new", None, 1);
    learned.merge(Saved::parse(&utf16(
        "w\t1\tlater\nw\t5\told\nw\t2\tLATER\np\t1\tyou\tlater\n",
    )));
    assert_eq!(
        String::from_utf16_lossy(&learned.saved_text()),
        "# AudioWhisper learned words, v1\nw\t5\told\nw\t3\tLater\nw\t1\tnew\np\t2\tyou\tlater\n"
    );
}

#[test]
fn merging_keeps_to_the_capacity() {
    let mut learned = LearnedWords::new(1, 1);
    add(&mut learned, "new", Some("brand"), 1);
    learned.merge(Saved::parse(&utf16("w\t1\told\np\t1\tvery\told\n")));
    assert_eq!(
        String::from_utf16_lossy(&learned.saved_text()),
        "# AudioWhisper learned words, v1\nw\t1\tnew\np\t1\tbrand\tnew\n"
    );
}

#[test]
fn skips_damaged_lines() {
    let saved = Saved::parse(&utf16(
        "# header\nw\tx\tword\nw\t2\tgood\np\t1\tonly-three\nq\t1\ta\tb\nw\t0\tzero\nw\t-1\tminus\nw\t+3\tplus\nw\t٣\tarabic\nw\t2\tgood\textra\np\t1\tmp3\tword\n\n",
    ));
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    learned.merge(saved);
    assert_eq!(
        String::from_utf16_lossy(&learned.saved_text()),
        "# AudioWhisper learned words, v1\nw\t2\tgood\nw\t3\tplus\nw\t3\tarabic\n"
    );
}

#[test]
fn the_previous_word_is_the_last_one_or_a_sentence_start() {
    let word = utf16("you");
    assert_eq!(previous_key(Some(&word), true), Some(&word[..]));
    assert_eq!(previous_key(None, true), Some(SENTENCE_START));
    assert_eq!(previous_key(None, false), None);
}

#[test]
fn every_change_is_counted_and_reading_is_not() {
    let mut learned = LearnedWords::new(WORD_CAPACITY, PAIR_CAPACITY);
    let mut seen = vec![learned.changes()];
    add(&mut learned, "Zorblax", None, 2);
    seen.push(learned.changes());
    learned.remove(&utf16("Zorblax"), None);
    seen.push(learned.changes());
    learned.merge(Saved::parse(&utf16("")));
    seen.push(learned.changes());
    learned.clear();
    seen.push(learned.changes());
    assert_eq!(seen, [0, 2, 3, 4, 5]);
    let _ = learned.completions(&utf16("zor"), 2);
    let _ = learned.spelling_of(&utf16("zorblax"));
    let _ = learned.known_spellings();
    assert_eq!(learned.changes(), 5);
}
