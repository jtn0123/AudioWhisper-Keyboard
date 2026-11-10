//! `dictionary/structure/v2/patricia_trie_policy`: a version 2 dictionary, as the queries the
//! suggestion code makes of it. Positions are into the dictionary's body, which is the file after
//! its header, so the root is at 0 and a word's id is the position of its terminal node.

use std::sync::Arc;

use super::bigram::{BigramIterator, BigramListPolicy};
use super::code_points::PatriciaWalk;
use super::dynamic::DynamicPtReadingHelper;
use super::interface::{NgramListener, WordAttributes};
use super::multi_bigram::MultiBigramMap;
use super::node::{PtNodeParams, Ver2PtNodeArrayReader, Ver2PtNodeReader};
use super::probability::{backoff, compute_probability_for_bigram};
use super::shortcut::{ShortcutIterator, ShortcutListPolicy};
use crate::defines::{NOT_A_DICT_POS, NOT_A_PROBABILITY, NOT_A_WORD_ID};

/// `PatriciaTriePolicy`: a version 2 dictionary, read in place from its bytes.
#[derive(Clone, Debug)]
pub struct PatriciaTriePolicy {
    file: Arc<[u8]>,
    body_start: usize,
    code_point_table: Option<Vec<i32>>,
}

impl PatriciaTriePolicy {
    /// `PatriciaTriePolicy(mmappedBuffer)`, with the header's size and code point table given
    /// directly. The body starts after the header. A header size outside the file (the C++ reads
    /// past the end there) starts the body at the file's end, or at its start when negative.
    pub fn new(file: Arc<[u8]>, header_size: i32, code_point_table: Option<Vec<i32>>) -> Self {
        let body_start = usize::try_from(header_size).unwrap_or(0).min(file.len());
        Self {
            file,
            body_start,
            code_point_table,
        }
    }

    /// The dictionary's body: the file after its header.
    pub fn body(&self) -> &[u8] {
        &self.file[self.body_start..]
    }

    /// The header's code point table, if the dictionary has one (`getCodePointTable`).
    pub fn code_point_table(&self) -> Option<&[i32]> {
        self.code_point_table.as_deref()
    }

    /// `getRootPosition`.
    pub fn root_position(&self) -> i32 {
        0
    }

    /// `PatriciaTriePolicy::isValidPos`.
    pub fn is_valid_pos(&self, pos: i32) -> bool {
        super::node::is_valid_pos(self.body(), pos)
    }

    /// The node reader over the body (`mPtNodeReader`).
    pub fn node_reader(&self) -> Ver2PtNodeReader<'_> {
        Ver2PtNodeReader::new(self.body(), self.code_point_table())
    }

    /// The node array reader over the body (`mPtNodeArrayReader`).
    pub fn array_reader(&self) -> Ver2PtNodeArrayReader<'_> {
        Ver2PtNodeArrayReader::new(self.body())
    }

    /// The bigram list policy over the body (`mBigramListPolicy`).
    pub fn bigram_policy(&self) -> BigramListPolicy<'_> {
        BigramListPolicy::new(self.body())
    }

    /// The shortcut list policy over the body (`mShortcutListPolicy`).
    pub fn shortcut_policy(&self) -> ShortcutListPolicy<'_> {
        ShortcutListPolicy::new(self.body())
    }

    /// `getTerminalPtNodePosFromWordId`: a word id is its terminal node's position.
    pub fn terminal_pt_node_pos_from_word_id(&self, word_id: i32) -> i32 {
        if word_id == NOT_A_WORD_ID {
            NOT_A_DICT_POS
        } else {
            word_id
        }
    }

    /// `getWordIdFromTerminalPtNodePos`.
    pub fn word_id_from_terminal_pt_node_pos(&self, pt_node_pos: i32) -> i32 {
        if pt_node_pos == NOT_A_DICT_POS {
            NOT_A_WORD_ID
        } else {
            pt_node_pos
        }
    }

    /// `getWordId`: the id of the word `code_points`, or NOT_A_WORD_ID. With
    /// `force_lower_case_search`, the search lower-cases each code point.
    pub fn get_word_id(&self, code_points: &[i32], force_lower_case_search: bool) -> i32 {
        let mut helper = DynamicPtReadingHelper::new(self.node_reader(), self.array_reader());
        helper.init_with_pt_node_array_pos(self.root_position());
        let pt_node_pos =
            helper.terminal_pt_node_position_of_word(code_points, force_lower_case_search);
        self.word_id_from_terminal_pt_node_pos(pt_node_pos)
    }

    /// `getProbability(unigramProbability, bigramProbability)`: the unigram when there is no bigram,
    /// the bigram's combined probability otherwise, and NOT_A_PROBABILITY when there is no unigram.
    pub fn get_probability(&self, unigram_probability: i32, bigram_probability: i32) -> i32 {
        if unigram_probability == NOT_A_PROBABILITY {
            NOT_A_PROBABILITY
        } else if bigram_probability == NOT_A_PROBABILITY {
            backoff(unigram_probability)
        } else {
            compute_probability_for_bigram(unigram_probability, bigram_probability)
        }
    }

    /// `getWordAttributesInContext`: the word's attributes after `prev_word_ids`. With a
    /// multi-bigram map, the map answers the bigram; without one, the bigram is looked up here.
    pub fn get_word_attributes_in_context(
        &self,
        prev_word_ids: &[i32],
        word_id: i32,
        multi_bigram_map: Option<&mut MultiBigramMap>,
    ) -> WordAttributes {
        if word_id == NOT_A_WORD_ID {
            return WordAttributes::default();
        }
        let pt_node_pos = self.terminal_pt_node_pos_from_word_id(word_id);
        let node = self.node_reader().fetch(pt_node_pos);
        if let Some(map) = multi_bigram_map {
            let probability =
                map.get_bigram_probability(self, prev_word_ids, word_id, node.probability());
            return self.word_attributes(probability, &node);
        }
        if !prev_word_ids.is_empty() {
            let bigram_probability = self.get_probability_of_word(prev_word_ids, word_id);
            if bigram_probability != NOT_A_PROBABILITY {
                return self.word_attributes(bigram_probability, &node);
            }
        }
        let probability = self.get_probability(node.probability(), NOT_A_PROBABILITY);
        self.word_attributes(probability, &node)
    }

    /// `getWordAttributes`: the attributes a node gives a word with `probability`.
    fn word_attributes(&self, probability: i32, node: &PtNodeParams) -> WordAttributes {
        WordAttributes::new(
            probability,
            false,
            node.is_not_a_word(),
            node.is_possibly_offensive(),
        )
    }

    /// `getProbabilityOfWord`: the word's probability after `prev_word_ids`, from the bigram of
    /// the first previous word, or from the unigram when there are none. NOT_A_PROBABILITY for a
    /// not-a-word node and for a word that doesn't follow the previous word in the dictionary.
    /// With previous words, there is no fallback to the unigram.
    pub fn get_probability_of_word(&self, prev_word_ids: &[i32], word_id: i32) -> i32 {
        if word_id == NOT_A_WORD_ID {
            return NOT_A_PROBABILITY;
        }
        let pt_node_pos = self.terminal_pt_node_pos_from_word_id(word_id);
        let node = self.node_reader().fetch(pt_node_pos);
        if node.is_not_a_word() {
            return NOT_A_PROBABILITY;
        }
        let Some(&first_prev) = prev_word_ids.first() else {
            return self.get_probability(node.probability(), NOT_A_PROBABILITY);
        };
        let mut bigrams = self.bigram_iterator(first_prev);
        while bigrams.has_next() {
            bigrams.next_entry();
            if bigrams.bigram_pos() == pt_node_pos && bigrams.probability() != NOT_A_PROBABILITY {
                return self.get_probability(node.probability(), bigrams.probability());
            }
        }
        NOT_A_PROBABILITY
    }

    /// `iterateNgramEntries`: every bigram of the first previous word, to `listener`, as its
    /// probability and target word id. A bigram that can't be read is visited once more, with
    /// NOT_A_PROBABILITY and the position it stopped at, as the C++ does.
    pub fn iterate_ngram_entries<L: NgramListener>(&self, prev_word_ids: &[i32], listener: &mut L) {
        let Some(&first_prev) = prev_word_ids.first() else {
            return;
        };
        let mut bigrams = self.bigram_iterator(first_prev);
        while bigrams.has_next() {
            bigrams.next_entry();
            listener.on_visit_entry(
                bigrams.probability(),
                self.word_id_from_terminal_pt_node_pos(bigrams.bigram_pos()),
            );
        }
    }

    /// The bigram iterator of the word `word_id`'s node.
    fn bigram_iterator(&self, word_id: i32) -> BigramIterator<'_> {
        let pos = self.bigram_position_of_pt_node(self.terminal_pt_node_pos_from_word_id(word_id));
        BigramIterator::new(self.bigram_policy(), pos)
    }

    /// `getBigramsPositionOfPtNode`.
    pub fn bigram_position_of_pt_node(&self, pt_node_pos: i32) -> i32 {
        if pt_node_pos == NOT_A_DICT_POS {
            return NOT_A_DICT_POS;
        }
        self.node_reader().fetch(pt_node_pos).bigram_pos()
    }

    /// `getShortcutPositionOfPtNode`.
    pub fn shortcut_position_of_pt_node(&self, pt_node_pos: i32) -> i32 {
        if pt_node_pos == NOT_A_DICT_POS {
            return NOT_A_DICT_POS;
        }
        self.node_reader().fetch(pt_node_pos).shortcut_pos()
    }

    /// `getShortcutIterator`: the shortcut targets of the word `word_id`.
    pub fn shortcut_iterator(&self, word_id: i32) -> ShortcutIterator<'_> {
        let pos =
            self.shortcut_position_of_pt_node(self.terminal_pt_node_pos_from_word_id(word_id));
        ShortcutIterator::new(self.shortcut_policy(), pos)
    }

    /// `getCodePointsAndReturnCodePointCount`: see [`PatriciaWalk`]. Returns the code point count,
    /// 0 when the word id is not a word, and writes the code points into `out`.
    pub fn get_code_points_and_return_code_point_count(
        &self,
        word_id: i32,
        max_code_point_count: usize,
        out: &mut [i32],
    ) -> usize {
        self.get_code_points_and_probability(word_id, max_code_point_count, out)
            .0
    }

    /// `getCodePointsAndProbabilityAndReturnCodePointCount`: the code points and unigram probability
    /// of the word `word_id`, as in the C++. The probability is NOT_A_PROBABILITY when the word
    /// isn't found.
    pub fn get_code_points_and_probability(
        &self,
        word_id: i32,
        max_code_point_count: usize,
        out: &mut [i32],
    ) -> (usize, i32) {
        PatriciaWalk::new(self).code_points_and_probability(word_id, max_code_point_count, out)
    }
}
