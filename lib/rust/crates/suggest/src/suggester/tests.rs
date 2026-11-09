use super::*;
use crate::dictionary::Lookup;
use crate::personal_words::Entry;
use crate::raw::{FLAG_APPROPRIATE_FOR_AUTO_CORRECTION, FLAG_EXACT_MATCH, KIND_CORRECTION};
use crate::raw::{KIND_PREDICTION, KIND_SHORTCUT, KIND_WHITELIST};

const CORRECTION: i32 = KIND_CORRECTION | FLAG_APPROPRIATE_FOR_AUTO_CORRECTION;
const EXACT: i32 = CORRECTION | FLAG_EXACT_MATCH;
const WHITELIST_SCORE: i32 = i32::MAX - 1;

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn texts(words: &[Vec<u16>]) -> Vec<String> {
    words
        .iter()
        .map(|word| String::from_utf16_lossy(word))
        .collect()
}

fn raw(word: &str, score: i32, kind_and_flags: i32) -> RawSuggestion {
    RawSuggestion {
        word: utf16(word),
        score,
        kind_and_flags,
    }
}

/// The real engine's results, scores and probabilities for these words with the bundled
/// en_US dictionary, as the Kotlin suggester's tests have them.
struct Fake {
    previous: PreviousWords,
    /// What each decode was for.
    purposes: Vec<Purpose>,
}

/// A result: word, score, kind and flags.
type Decoded = (&'static str, i32, i32);

/// Each input's results.
const RESULTS: [(&str, &[Decoded]); 16] = [
    (
        "teh",
        &[("the", 477_893, CORRECTION), ("ten", 400_787, CORRECTION)],
    ),
    ("teh2", &[("the", 477_893, CORRECTION)]),
    (
        "im",
        &[
            ("I'm", WHITELIST_SCORE, KIND_WHITELIST),
            ("in", 571_761, CORRECTION),
        ],
    ),
    (
        "form",
        &[("form", 1_819_619, EXACT), ("from", 422_604, CORRECTION)],
    ),
    (
        "fro",
        &[("fro", 1_536_913, EXACT), ("from", 471_045, CORRECTION)],
    ),
    (
        "ok",
        &[("OK", 1_740_557, EXACT), ("on", 521_841, CORRECTION)],
    ),
    ("london", &[("London", 1_801_319, EXACT)]),
    (
        "yeah",
        &[("yeah", 1_753_136, EXACT), ("year", 235_587, CORRECTION)],
    ),
    ("lol", &[("oil", 227_445, CORRECTION)]),
    (
        "wor",
        &[
            ("WOR", 1_600_000, EXACT),
            ("work", 600_000, CORRECTION),
            ("word", 580_000, CORRECTION),
            ("worn", 570_000, CORRECTION),
        ],
    ),
    (
        "jello",
        &[
            ("jello", 1_500_000, EXACT),
            ("Jello", 1_400_000, CORRECTION),
        ],
    ),
    (
        "jelo",
        &[
            ("Jello", 600_000, CORRECTION),
            ("jello", 590_000, CORRECTION),
        ],
    ),
    ("u", &[("I", 484_164, CORRECTION)]),
    (
        "nit",
        &[
            ("not", WHITELIST_SCORE, KIND_WHITELIST),
            ("nit", 1_493_598, EXACT),
            ("not", 505_979, CORRECTION),
        ],
    ),
    (
        "",
        &[
            ("you", 90, KIND_PREDICTION),
            ("God", 80, KIND_PREDICTION),
            ("you", 70, KIND_PREDICTION),
        ],
    ),
    (
        "shrt",
        &[(
            "short",
            900_000,
            KIND_SHORTCUT | FLAG_APPROPRIATE_FOR_AUTO_CORRECTION,
        )],
    ),
];

fn results(input: &str) -> Vec<RawSuggestion> {
    let known = RESULTS.iter().find(|&&(known, _)| known == input);
    let results = known.map_or(&[][..], |&(_, results)| results);
    (results.iter())
        .map(|&(word, score, kind_and_flags)| raw(word, score, kind_and_flags))
        .collect()
}

const PROBABILITIES: [(&str, i32); 12] = [
    ("jello", 120),
    ("WOR", 79),
    ("work", 200),
    ("word", 190),
    ("worn", 60),
    ("OK", 150),
    ("on", 220),
    ("form", 158),
    ("from", 191),
    ("fro", 73),
    ("yeah", 135),
    ("nit", 59),
];

impl Lookup for Fake {
    type Error = NegativeCount;

    fn probability(&mut self, word: &[u16]) -> Result<i32, NegativeCount> {
        let word = String::from_utf16_lossy(word);
        let known = PROBABILITIES.iter().find(|(known, _)| *known == word);
        Ok(known.map_or(NOT_A_PROBABILITY, |&(_, probability)| probability))
    }
}

impl Dictionary for Fake {
    fn ngram_probability(&mut self, word: &[u16]) -> Result<i32, NegativeCount> {
        let previous = self
            .previous
            .words
            .first()
            .map(|word| String::from_utf16_lossy(word));
        let known = previous.as_deref() == Some("came") && word == utf16("from");
        Ok(if known { 194 } else { NOT_A_PROBABILITY })
    }

    fn decode(
        &mut self,
        input: &[u16],
        purpose: Purpose,
    ) -> Result<Vec<RawSuggestion>, NegativeCount> {
        self.purposes.push(purpose);
        let mut results = results(&String::from_utf16_lossy(input));
        results.sort_by_key(|it| std::cmp::Reverse(it.score));
        Ok(results)
    }
}

struct Words {
    personal: PersonalWords,
    learned: Option<Mutex<LearnedWords>>,
}

impl Words {
    fn none() -> Words {
        Words {
            personal: PersonalWords::new(Vec::new()),
            learned: None,
        }
    }

    fn personal(words: &[(&str, i32)]) -> Words {
        let entries = words.iter().map(|&(word, frequency)| Entry {
            word: utf16(word),
            frequency,
            shortcut: None,
        });
        Words {
            personal: PersonalWords::new(entries.collect()),
            learned: None,
        }
    }

    fn learned(words: &[(&str, Option<&str>, i32)]) -> Words {
        let mut learned = LearnedWords::new(100, 100);
        for &(word, previous, times) in words {
            let previous = previous.map(utf16);
            for _ in 0..times {
                learned.add(&utf16(word), previous.as_deref(), false);
            }
        }
        Words {
            personal: PersonalWords::new(Vec::new()),
            learned: Some(Mutex::new(learned)),
        }
    }

    fn suggester<'a>(&'a self, fake: &'a mut Fake) -> Suggester<'a, Fake> {
        Suggester {
            dictionary: fake,
            personal: &self.personal,
            learned: self.learned.as_ref(),
            threshold: crate::auto_correction::MODEST_THRESHOLD,
        }
    }

    /// The strip for `typed` between `before` and `after`, and what each decode was for.
    fn strip(
        &self,
        before: &str,
        typed: &str,
        after: &str,
        max: i32,
    ) -> (Vec<Arranged>, Vec<Purpose>) {
        let previous_words = PreviousWords::from(&utf16(before));
        let state = TypingState {
            typed: utf16(typed),
            previous_words: previous_words.clone(),
            is_cursor_mid_word: utf16(after)
                .first()
                .is_some_and(|&unit| chars::is_letter_or_digit(unit)),
        };
        let mut fake = Fake {
            previous: previous_words,
            purposes: Vec::new(),
        };
        let strip = self.suggester(&mut fake).suggest(&state, max).unwrap();
        (strip, fake.purposes)
    }

    fn words(&self, before: &str, typed: &str, after: &str) -> Vec<String> {
        let (strip, _) = self.strip(before, typed, after, 8);
        strip
            .into_iter()
            .map(|it| String::from_utf16_lossy(&it.text))
            .collect()
    }

    fn auto_correction(&self, before: &str, typed: &str, after: &str) -> Option<String> {
        let (strip, _) = self.strip(before, typed, after, 8);
        let correction = strip.into_iter().find(|it| it.is_auto_correction);
        correction.map(|it| String::from_utf16_lossy(&it.text))
    }

    fn spell(&self, word: &str, max: i32) -> Result<(bool, Vec<String>, bool), NegativeCount> {
        let mut fake = Fake {
            previous: PreviousWords::NONE,
            purposes: Vec::new(),
        };
        let check = self.suggester(&mut fake).spell(&utf16(word), max)?;
        assert!(
            fake.purposes
                .iter()
                .all(|&purpose| purpose == Purpose::Spelling)
        );
        Ok((
            check.is_valid,
            texts(&check.suggestions),
            check.is_high_confidence,
        ))
    }
}

#[test]
fn corrects_a_typo_and_keeps_the_typed_word_tappable() {
    let (strip, purposes) = Words::none().strip("", "teh", "", 8);
    assert_eq!(
        texts(&strip.iter().map(|it| it.text.clone()).collect::<Vec<_>>()),
        ["the", "teh", "ten"]
    );
    let flags: Vec<(bool, bool)> = strip
        .iter()
        .map(|it| (it.is_auto_correction, it.is_typed_word))
        .collect();
    assert_eq!(flags, [(true, false), (false, true), (false, false)]);
    assert_eq!(purposes, [Purpose::Typing]);
}

#[test]
fn matches_the_case_of_the_typed_word() {
    let none = Words::none();
    assert_eq!(none.auto_correction("", "Teh", "").as_deref(), Some("The"));
    assert_eq!(none.words("", "TEH", ""), ["TEH", "THE", "TEN"]);
    // Mostly caps is never corrected.
    assert_eq!(none.auto_correction("", "TEH", ""), None);
    assert_eq!(none.auto_correction("", "TeH", ""), None);
}

#[test]
fn the_whitelist_corrects_even_capitals_and_valid_words() {
    let none = Words::none();
    assert_eq!(none.auto_correction("", "im", "").as_deref(), Some("I'm"));
    assert_eq!(none.auto_correction("", "Im", "").as_deref(), Some("I'm"));
    assert_eq!(none.auto_correction("", "nit", "").as_deref(), Some("not"));
    assert_eq!(none.words("", "nit", ""), ["not", "nit"]);
    // Even a single letter, which nothing else corrects.
    assert_eq!(none.auto_correction("", "u", ""), None);
}

#[test]
fn leaves_valid_words_alone_unless_much_rarer_in_context() {
    let none = Words::none();
    assert_eq!(none.words("", "form", ""), ["form", "from"]);
    assert_eq!(none.auto_correction("", "form", ""), None);
    assert_eq!(
        none.auto_correction("came ", "fro", "").as_deref(),
        Some("from")
    );
    assert_eq!(none.auto_correction("and ", "fro", ""), None);
    assert_eq!(none.auto_correction("", "fro", ""), None);
}

#[test]
fn capitalizes_names_but_makes_no_acronyms() {
    let none = Words::none();
    assert_eq!(
        none.auto_correction("", "london", "").as_deref(),
        Some("London")
    );
    assert_eq!(none.auto_correction("", "ok", ""), None);
    assert_eq!(none.words("", "ok", ""), ["ok", "OK", "on"]);
}

#[test]
fn offers_each_word_once_in_one_spelling() {
    let none = Words::none();
    assert_eq!(none.words("", "jello", ""), ["jello"]);
    assert_eq!(none.words("", "jelo", ""), ["jello", "jelo"]);
    // Typed with a capital, every result has one: both spellings are one.
    assert_eq!(none.words("", "Jelo", ""), ["Jello", "Jelo"]);
}

#[test]
fn an_acronym_of_the_typed_letters_goes_after_the_words_being_typed() {
    let none = Words::none();
    assert_eq!(
        none.words("", "wor", ""),
        ["wor", "work", "word", "WOR", "worn"]
    );
    assert_eq!(none.auto_correction("", "wor", ""), None);
}

#[test]
fn offers_below_the_threshold_without_correcting() {
    let none = Words::none();
    assert_eq!(none.auto_correction("", "yeah", ""), None);
    assert_eq!(none.words("", "lol", ""), ["lol", "oil"]);
    assert_eq!(none.auto_correction("", "lol", ""), None);
    // Nor to a shortcut, however good.
    assert_eq!(none.words("", "shrt", ""), ["shrt", "short"]);
}

#[test]
fn never_corrects_with_digits_or_inside_a_word() {
    let none = Words::none();
    assert_eq!(none.auto_correction("", "teh2", ""), None);
    assert_eq!(none.auto_correction("", "teh", "re"), None);
    assert_eq!(
        none.auto_correction("", "teh", " and").as_deref(),
        Some("the")
    );
}

#[test]
fn predicts_the_next_word_without_correcting() {
    let none = Words::none();
    let (strip, purposes) = none.strip("thank ", "", "", 8);
    assert!(
        strip
            .iter()
            .all(|it| !it.is_auto_correction && !it.is_typed_word)
    );
    assert_eq!(
        texts(&strip.into_iter().map(|it| it.text).collect::<Vec<_>>()),
        ["you", "God"]
    );
    assert_eq!(purposes, [Purpose::Predicting]);
    let (strip, purposes) = none.strip("Hello, ", "", "", 8);
    assert_eq!((strip, purposes), (Vec::new(), Vec::new()));
}

#[test]
fn keeps_to_the_limit() {
    let none = Words::none();
    let first = |typed: &str, max: i32| {
        let (strip, _) = none.strip("", typed, "", max);
        texts(&strip.into_iter().map(|it| it.text).collect::<Vec<_>>())
    };
    assert_eq!(first("teh", 1), ["the"]);
    assert_eq!(first("form", 1), ["form"]);
    assert_eq!(first("teh", 2), ["the", "teh"]);
    assert!(first("teh", 0).is_empty());
    assert!(first("teh", -1).is_empty());
    assert_eq!(none.strip("thank ", "", "", 1).0.len(), 1);
}

#[test]
fn spell_checks_with_confidence() {
    let none = Words::none();
    assert_eq!(none.spell("form", 3), Ok((true, vec![], false)));
    assert_eq!(
        none.spell("teh", 3),
        Ok((false, vec!["the".into(), "ten".into()], true))
    );
    assert_eq!(none.spell("teh", 1), Ok((false, vec!["the".into()], true)));
    assert_eq!(none.spell("lol", 3), Ok((false, vec!["oil".into()], false)));
    assert_eq!(none.spell("zzz", 3), Ok((false, vec![], false)));
    // Kotlin's take(-1) throws, but only once it comes to suggesting.
    assert_eq!(none.spell("teh", -1), Err(NegativeCount(-1)));
    assert_eq!(none.spell("form", -1), Ok((true, vec![], false)));
}

#[test]
fn never_corrects_a_personal_word_away() {
    let personal = Words::personal(&[("teh", 200), ("iPhone", 250), ("formica", 120)]);
    assert_eq!(personal.words("", "teh", ""), ["teh", "the", "ten"]);
    assert_eq!(personal.auto_correction("", "teh", ""), None);
    assert_eq!(personal.spell("teh", 3), Ok((true, vec![], false)));
    // Typed in another case, it takes the user's spelling.
    assert_eq!(
        personal.auto_correction("", "iphone", "").as_deref(),
        Some("iPhone")
    );
    assert_eq!(personal.auto_correction("", "iphone", "s"), None);
    assert_eq!(personal.words("", "form", ""), ["form", "formica", "from"]);
    assert_eq!(personal.words("", "Form", ""), ["Form", "Formica", "From"]);
}

#[test]
fn offers_at_most_two_of_the_users_words() {
    let personal = Words::personal(&[("formica", 120), ("formal", 130), ("format", 140)]);
    assert_eq!(
        personal.words("", "form", ""),
        ["form", "format", "formal", "from"]
    );
    let learned = Words::learned(&[("worda", None, 2), ("wordb", None, 3), ("wordc", None, 4)]);
    assert_eq!(learned.words("", "wo", ""), ["wo", "wordc", "wordb"]);
}

#[test]
fn offers_used_words_the_dictionary_lacks() {
    let learned = Words::learned(&[
        ("wordle", None, 2),
        ("word", None, 2),
        ("wormhole", None, 1),
    ]);
    assert_eq!(learned.words("", "wo", ""), ["wo", "wordle"]);
}

#[test]
fn never_corrects_a_word_the_user_keeps_typing() {
    let twice = Words::learned(&[("teh", None, 2)]);
    assert_eq!(twice.auto_correction("", "teh", ""), None);
    assert_eq!(twice.spell("teh", 3), Ok((true, vec![], false)));
    let once = Words::learned(&[("teh", None, 1)]);
    assert_eq!(once.auto_correction("", "teh", "").as_deref(), Some("the"));
    // The dictionary knows its own words better than what was typed: "WOR" would be taken.
    let known = Words::learned(&[("WOR", None, 2)]);
    assert_eq!(known.auto_correction("", "wor", ""), None);
    let unknown = Words::learned(&[("Teh", None, 2)]);
    assert_eq!(
        unknown.auto_correction("", "teh", "").as_deref(),
        Some("Teh")
    );
}

#[test]
fn predicts_the_users_habits_first() {
    let learned = Words::learned(&[
        ("later", Some("you"), 2),
        ("there", Some("you"), 1),
        ("You", Some("thank"), 1),
    ]);
    assert_eq!(
        learned.words("see you ", "", ""),
        ["later", "you", "God", "there"]
    );
    assert_eq!(learned.words("thank ", "", ""), ["you", "God"]);
}
