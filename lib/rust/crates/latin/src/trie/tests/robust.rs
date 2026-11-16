//! No-panic tests: the Rust queries run over truncated and corrupted copies of each dictionary,
//! where the C++ reads past the end (the Rust reads those bytes as 0; see `trie/mod.rs`). These check
//! only that the queries return, with an answer of some kind.

use std::sync::Arc;

use super::{Random, pairs, random_word};
use crate::defines::NOT_A_WORD_ID;
use crate::trie::dictionary::Dictionary;
use crate::trie::interface::WordListener;
use crate::trie::multi_bigram::MultiBigramMap;
use crate::trie::ngram::NgramContext;

/// Asks each kind of question of `dictionary`, with answers that are discarded.
fn exercise(dictionary: &Dictionary, random: &mut Random, words: &[Vec<i32>]) {
    let policy = dictionary.policy();
    let body_len = policy.body().len();
    let mut out = [0_i32; 48];
    let mut map = MultiBigramMap::new();
    for _ in 0..40 {
        let word = random_word(random, words);
        let id = policy.get_word_id(&word, random.chance(2));
        let prev = [random.below(body_len + 8) as i32];
        let _ = dictionary.probability(&word);
        let _ = dictionary.ngram_probability(
            Some(&NgramContext::new(&[Some(&word)], &[random.chance(2)])),
            &word,
        );
        let _ = policy.get_code_points_and_return_code_point_count(id, 48, &mut out);
        let _ = policy.get_word_attributes_in_context(&prev, id, Some(&mut map));
        let _ = policy.get_probability_of_word(&prev, id);
        let _ = policy.node_reader().fetch(prev[0]);
        let _ = policy.array_reader().read_info(prev[0]);
        let _ = policy.array_reader().read_forward_link(prev[0]);
        let mut shortcuts = policy.shortcut_iterator(id);
        if shortcuts.has_next_shortcut_target() {
            let _ = shortcuts.next_shortcut_target(48, &mut out);
        }
    }
}

/// `file` as a dictionary, or `None` if its header no longer identifies a version 2 one.
fn dictionary_of(file: Vec<u8>) -> Option<Dictionary> {
    Dictionary::open(Arc::from(file))
}

#[test]
fn truncated_dictionaries_answer_without_panicking() {
    for pair in pairs() {
        let full = std::fs::read(&pair.path).expect("the dictionary");
        let mut random = Random(0x7E11_CA7E);
        for _ in 0..150 {
            let cut = 12 + random.below(full.len() - 12);
            let Some(dictionary) = dictionary_of(full[..cut].to_vec()) else {
                continue;
            };
            exercise(&dictionary, &mut random, &pair.words);
        }
    }
}

#[test]
fn truncated_dictionaries_iterate_their_words_without_panicking() {
    for path in latin_oracle::dictionary_paths() {
        let full = std::fs::read(path).expect("the dictionary");
        let mut random = Random(0x017E_2A7E);
        for _ in 0..25 {
            let cut = 12 + random.below(full.len() - 12);
            let Some(dictionary) = dictionary_of(full[..cut].to_vec()) else {
                continue;
            };
            let mut words = Count(0);
            dictionary.iterate_words(&mut words);
        }
    }
}

#[test]
fn corrupted_dictionaries_answer_without_panicking() {
    for pair in pairs() {
        let full = std::fs::read(&pair.path).expect("the dictionary");
        let mut random = Random(0x0BAD_B17E);
        for _ in 0..60 {
            let mut file = full.clone();
            for _ in 0..16 {
                let at = 12 + random.below(file.len() - 12);
                file[at] = random.draw() as u8;
            }
            let Some(dictionary) = dictionary_of(file) else {
                continue;
            };
            exercise(&dictionary, &mut random, &pair.words);
        }
    }
}

#[test]
fn queries_past_the_end_of_the_body_answer_without_panicking() {
    for pair in pairs() {
        let policy = pair.ours.policy();
        let past = policy.body().len() as i32 + 1_000;
        for pos in [-1, i32::MIN, past, i32::MAX] {
            let _ = policy.node_reader().fetch(pos);
            let _ = policy.node_reader().read_info(pos);
            let _ = policy.array_reader().read_info(pos);
            let _ = policy.array_reader().read_forward_link(pos);
        }
        assert_eq!(
            policy.get_word_id(&[], false),
            NOT_A_WORD_ID,
            "the empty word is no word"
        );
    }
}

/// Counts the words a walk visits, without keeping them.
struct Count(usize);

impl WordListener for Count {
    fn on_visit_word(&mut self, _code_points: &[i32]) {
        self.0 += 1;
    }
}
