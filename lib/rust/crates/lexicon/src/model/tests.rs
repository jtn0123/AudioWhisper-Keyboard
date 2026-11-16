//! The builder's files, read back by the keyboard's reader (`crates/ngram`).

use super::smooth::Tables;
use super::*;
use crate::dictionary::Scale;
use crate::text::utf16;
use crate::v202::{Entry, code_points};
use ngram::format::{Unigram, pack};
use ngram::{Context, Model};

const SCALE: Scale = Scale { a: 240.0, b: 11.0 };

fn entry(word: &str) -> Entry {
    Entry {
        word: code_points(word),
        probability: 100,
        not_a_word: false,
        possibly_offensive: false,
        shortcuts: Vec::new(),
        bigrams: Vec::new(),
    }
}

fn dictionary(words: &[&str]) -> Dictionary {
    let mut entries: Vec<Entry> = words.iter().map(|w| entry(w)).collect();
    let mut rude = entry("damn");
    rude.possibly_offensive = true;
    let mut not_a_word = entry("teh");
    not_a_word.not_a_word = true;
    entries.extend([rude, not_a_word]);
    Dictionary {
        flags: 0,
        attributes: Vec::new(),
        entries,
    }
}

const WORDS: &[&str] = &[
    "I", "am", "here", "you", "are", "there", "we", "will", "Will", "go", "now", "never", "seen",
];

const LINES: &[&str] = &[
    "I am here.",
    "I am here now.",
    "You are there.",
    "You are here, I am there.",
    "We will go now. Will you go?",
    "I will go, damn it. Damn.",
    "We are here; we will go there now.",
    "Will is here.",
    "teh teh teh teh",
];

fn built(options: Options) -> (Model, Vocabulary) {
    let d = dictionary(WORDS);
    let counts = crate::count::count(&d, LINES, 2);
    let counted = count(&d, &counts, LINES, 2).unwrap();
    let (bytes, summary) = build(&counted, SCALE, options).unwrap();
    assert_eq!(summary.words, counted.vocabulary.len());
    (Model::open(bytes).unwrap(), counted.vocabulary)
}

fn all() -> Options {
    Options {
        min_bigram: 1,
        min_trigram: 1,
    }
}

fn id(model: &Model, word: &str) -> u32 {
    model.word(&utf16(word)).unwrap()
}

fn after(model: &Model, words: &[&str], start: bool) -> Context {
    let newest_first: Vec<Vec<u16>> = words.iter().rev().map(|w| utf16(w)).collect();
    model.context(&newest_first, start)
}

#[test]
fn writes_tables_the_reader_reads_back() {
    let words: Vec<Vec<u16>> = ["ab", "b", "cd"].iter().map(|w| utf16(w)).collect();
    let unigram = |probability, backoff, frequency, flags| Unigram {
        probability,
        backoff,
        frequency,
        flags,
    };
    let tables = Tables {
        words,
        unigrams: vec![
            unigram(10, 1, 11, 0),
            unigram(20, 2, 21, ngram::format::PREFERS_LOWER_AT_START),
            unigram(30, 3, 31, 0),
            unigram(255, 4, 255, 0),
        ],
        bigram_starts: vec![0, 2, 2, 2, 3],
        bigrams: vec![pack(1, 5), pack(2, 6), pack(0, 7)],
        backoffs: vec![8, 9, 10],
        contexts: vec![0, 2],
        trigram_starts: vec![0, 1, 3],
        trigrams: vec![pack(2, 12), pack(1, 13), pack(2, 14)],
        step: 0.25,
        backoff_step: 0.5,
        scale: SCALE,
    };
    let bytes = write(&tables).unwrap();
    let m = Model::open(bytes).unwrap();
    let h = m.header();
    assert_eq!(
        (h.words, h.units, h.bigrams, h.contexts, h.trigrams),
        (3, 5, 3, 2, 3)
    );
    assert_eq!(
        (h.step, h.backoff_step, h.scale_a, h.scale_b),
        (0.25, 0.5, 240.0, 11.0)
    );
    for (i, word) in tables.words.iter().enumerate() {
        assert_eq!(m.spelling(i as u32).as_ref(), Some(word));
        assert_eq!(ngram::Spellings::exact(&m, word), Some(i as u32));
    }
    assert!(ngram::Spellings::prefers_lower_at_start(&m, 1));
    assert!(!ngram::Spellings::prefers_lower_at_start(&m, 0));
    let p = |c: Context, w| m.ln_probability(c, w).unwrap();
    let ctx = |newest, older| Context { newest, older };
    assert_eq!(p(Context::default(), 2), -7.5);
    assert_eq!(m.ln_frequency(2), Some(-7.75));
    assert_eq!(p(ctx(Some(0), None), 1), -1.25);
    assert_eq!(p(ctx(Some(0), None), 0), -0.5 - 2.5);
    assert_eq!(p(ctx(Some(3), None), 0), -1.75);
    assert_eq!(p(ctx(Some(3), None), 1), -2.0 - 5.0);
    // Bigram 0 (ab b) has trigram cd; bigram 2 (start ab) has b and cd.
    assert_eq!(p(ctx(Some(1), Some(0)), 2), -3.0);
    assert_eq!(p(ctx(Some(1), Some(0)), 0), -4.0 + -1.0 + -2.5);
    assert_eq!(p(ctx(Some(0), Some(3)), 1), -3.25);
    assert_eq!(p(ctx(Some(0), Some(3)), 2), -3.5);
    assert_eq!(p(ctx(Some(0), Some(3)), 0), -5.0 + -0.5 + -2.5);
    // Bigram 1 (ab cd) has no trigrams, only its backoff.
    assert_eq!(p(ctx(Some(2), Some(0)), 1), -4.5 - 1.5 - 5.0);
}

#[test]
fn the_vocabulary_is_the_dictionary_words_the_corpus_writes_less_the_fixed_ones() {
    let (m, _) = built(all());
    for word in ["I", "am", "Will", "will", "seen"] {
        let found = m.word(&utf16(word)).and_then(|id| m.spelling(id));
        let expected = (word != "seen").then(|| utf16(word));
        assert_eq!(found, expected, "{word}");
    }
    assert_eq!(m.word(&utf16("damn")), None);
    assert_eq!(m.word(&utf16("teh")), None);
    assert_eq!(m.word(&utf16("never")), None);
}

#[test]
fn every_context_s_probabilities_add_up_to_one() {
    let (m, _) = built(all());
    // Quantised to a byte; each probability is within half a step of its value.
    let tolerance = (m.header().step / 2.0).exp_m1() as f64 + 1e-6;
    let contexts = [
        after(&m, &[], false),
        after(&m, &[], true),
        after(&m, &["I"], true),
        after(&m, &["you"], false),
        after(&m, &["I", "am"], true),
        after(&m, &["we", "will"], false),
        after(&m, &["are", "here"], false),
    ];
    for context in contexts {
        let total: f64 = (0..m.words())
            .map(|w| f64::from(m.ln_probability(context, w).unwrap()).exp())
            .sum();
        assert!((total - 1.0).abs() < tolerance, "{context:?}: {total}");
    }
}

#[test]
fn predicts_what_the_corpus_writes_next() {
    let (m, _) = built(all());
    let words = |c: Context, n| -> Vec<String> {
        m.predict(c, n)
            .into_iter()
            .map(|(id, _)| String::from_utf16(&m.spelling(id).unwrap()).unwrap())
            .collect()
    };
    assert_eq!(words(after(&m, &["I"], true), 1), ["am"]);
    assert_eq!(words(after(&m, &["I", "am"], true), 1), ["here"]);
    // Once each after "you are", but more words come before "are here" than "are there".
    assert_eq!(
        words(after(&m, &["you", "are"], true), 2),
        ["here", "there"]
    );
    // Every word comes out once, the offensive one never.
    let everything = words(after(&m, &["will"], false), 100);
    assert_eq!(everything.len(), m.words() as usize);
    assert!(!everything.contains(&"damn".to_string()));
    assert_eq!(words(after(&m, &["will"], false), 1), ["go"]);
}

#[test]
fn a_capital_at_a_sentence_start_counts_as_the_usual_spelling() {
    let (m, vocabulary) = built(all());
    // "Will you go?" starts a sentence: counted as "will", which is likelier mid-sentence.
    let will = id(&m, "will");
    let start = after(&m, &[], true);
    assert_eq!(after(&m, &["Will"], true).newest, Some(will));
    assert_eq!(after(&m, &["Will"], false).newest, Some(id(&m, "Will")));
    // Both the builder's vocabulary and the file resolve alike.
    for word in WORDS.iter().chain(&["The", "WILL", "Never"]) {
        for initial in [false, true] {
            let units = utf16(word);
            assert_eq!(
                ngram::resolve(&vocabulary, &units, initial),
                ngram::resolve(&m, &units, initial),
                "{word}"
            );
        }
    }
    let p = |w| m.ln_probability(start, w).unwrap();
    assert!(p(will) > p(id(&m, "Will")));
}

#[test]
fn pruned_ngrams_fall_back_to_shorter_ones() {
    let (full, _) = built(all());
    let (pruned, _) = built(Options {
        min_bigram: 2,
        min_trigram: 2,
    });
    assert!(pruned.header().trigrams < full.header().trigrams);
    assert!(pruned.header().bigrams < full.header().bigrams);
    // "We will go now" once: pruned, so it backs off to "will go"/"go now" and costs more.
    let context = |m: &Model| after(m, &["will", "go"], false);
    let now = |m: &Model| m.ln_probability(context(m), id(m, "now")).unwrap();
    assert!(now(&pruned) < now(&full));
    // Seen twice ("I am here"), the trigram stays.
    let here = |m: &Model| {
        m.ln_probability(after(m, &["I", "am"], true), id(m, "here"))
            .unwrap()
    };
    let steps = (pruned.header().step + full.header().step) / 2.0;
    assert!((here(&pruned) - here(&full)).abs() <= steps + 1e-6);
}

/// The model the app ships knows only the shipped dictionary's words, never one it flags as
/// possibly offensive or not a word, so it can't predict one. It fits the 15 MB budget.
#[test]
fn the_shipped_model_knows_only_words_the_shipped_dictionary_may_suggest() {
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../app/src/main/assets/dicts");
    let bytes = std::fs::read(assets.join("main_en-US.lm")).unwrap();
    assert!(bytes.len() <= 15_000_000, "{} bytes", bytes.len());
    let m = Model::open(bytes).unwrap();
    let d = crate::v202::read(&std::fs::read(assets.join("main_en-US.dict")).unwrap()).unwrap();
    let entries: std::collections::HashMap<Vec<u16>, &Entry> = (d.entries.iter())
        .map(|e| (utf16(&crate::v202::string(&e.word)), e))
        .collect();
    let flagged = (d.entries.iter())
        .filter(|e| crate::dictionary::fixed(e))
        .count();
    assert!(flagged > 0);
    for id in 0..m.words() {
        let word = m.spelling(id).unwrap();
        let entry = entries.get(&word).expect("a dictionary word");
        assert!(!crate::dictionary::fixed(entry), "word {id}");
    }
}
