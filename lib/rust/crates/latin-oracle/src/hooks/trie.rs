//! The version 2 trie, from `cpp/hooks/trie.cpp`: a dictionary's structure policy (word ids, code
//! points, attributes, ngram entries, shortcuts, the word list, the multi-bigram map), and a probe
//! of node-level reads over a copy of a dictionary body. Every call holds the oracle's lock.

use crate::Dictionary;

/// The first capacity a collected result is asked for, before a second call with the exact size.
const FIRST_CAPACITY: usize = 4096;

// SAFETY: cpp/hooks/trie.cpp defines these with the same C signatures; each takes pointers to
// slices or buffers the caller holds for the call.
unsafe extern "C" {
    fn aw_hook_trie_word_id(
        dict: i64,
        code_points: *const i32,
        length: i32,
        force_lower_case: i32,
    ) -> i32;
    fn aw_hook_trie_code_points(dict: i64, word_id: i32, max: i32, out: *mut i32) -> i32;
    fn aw_hook_trie_probability_of_word(
        dict: i64,
        prev: *const i32,
        prev_count: i32,
        word_id: i32,
    ) -> i32;
    fn aw_hook_trie_word_attributes(
        dict: i64,
        map: i64,
        prev: *const i32,
        prev_count: i32,
        word_id: i32,
        flags: *mut i32,
    ) -> i32;
    fn aw_hook_trie_ngram_entries(
        dict: i64,
        prev: *const i32,
        prev_count: i32,
        out: *mut i32,
        capacity: i32,
    ) -> i32;
    fn aw_hook_trie_words(dict: i64, out: *mut i32, capacity: i32) -> i32;
    fn aw_hook_trie_shortcuts(
        dict: i64,
        word_id: i32,
        max_depth: i32,
        out: *mut i32,
        capacity: i32,
    ) -> i32;
    fn aw_hook_trie_multi_bigram_new() -> i64;
    fn aw_hook_trie_multi_bigram_delete(map: i64);
    fn aw_hook_trie_multi_bigram_probability(
        map: i64,
        dict: i64,
        prev: *const i32,
        prev_count: i32,
        next: i32,
        unigram: i32,
    ) -> i32;
}

// SAFETY: cpp/fake_jni.cpp defines this with the signature `lib.rs` declares for the same symbol.
unsafe extern "C" {
    fn aw_oracle_ngram_probability(
        dictionary: i64,
        previous_flat: *const i32,
        previous_lengths: *const i32,
        beginning_of_sentence: *const u8,
        slots: i32,
        word: *const i32,
        length: i32,
    ) -> i32;
}

/// `BinaryDictionary.getNgramProbability` as the JNI gives it, where a `None` previous word is a
/// null Java array. `beginning[i]` is the beginning-of-sentence flag of slot `i`, most recent first.
pub fn ngram_probability_jni(
    dictionary: &Dictionary,
    previous: &[Option<Vec<i32>>],
    beginning: &[bool],
    word: &[i32],
) -> i32 {
    assert_eq!(
        previous.len(),
        beginning.len(),
        "one flag per previous word"
    );
    let lengths: Vec<i32> = previous
        .iter()
        .map(|slot| slot.as_ref().map_or(-1, |it| length(it)))
        .collect();
    let flat: Vec<i32> = previous
        .iter()
        .flatten()
        .flat_map(|slot| slot.iter().copied())
        .collect();
    let flags: Vec<u8> = beginning.iter().map(|&it| u8::from(it)).collect();
    let _lock = crate::lock();
    // SAFETY: each pointer is valid for the lengths given. The C++ reads a slot's words only when
    // its length is not -1, so the empty `flat` of an all-null list is never read.
    unsafe {
        aw_oracle_ngram_probability(
            dictionary.address(),
            flat.as_ptr(),
            lengths.as_ptr(),
            flags.as_ptr(),
            length(&lengths),
            word.as_ptr(),
            length(word),
        )
    }
}

/// `c_call` writes at most `capacity` ints and returns how many it had. Calls again with the exact
/// size when the first buffer was too small. The hooks have no side effects, so both calls agree.
pub(super) fn collect(mut call: impl FnMut(*mut i32, i32) -> i32) -> Vec<i32> {
    let mut capacity = FIRST_CAPACITY;
    loop {
        let mut buffer = vec![0_i32; capacity];
        let total = usize::try_from(call(buffer.as_mut_ptr(), capacity as i32))
            .expect("the hook returns a count");
        if total <= capacity {
            buffer.truncate(total);
            return buffer;
        }
        capacity = total;
    }
}

pub(super) fn length(slice: &[i32]) -> i32 {
    i32::try_from(slice.len()).expect("array too long for the C++")
}

/// The word id of `code_points` in `dictionary`, or NOT_A_WORD_ID.
pub fn word_id(dictionary: &Dictionary, code_points: &[i32], force_lower_case: bool) -> i32 {
    let _lock = crate::lock();
    // SAFETY: the dictionary is open for the call, and the slice is valid for `length` ints.
    unsafe {
        aw_hook_trie_word_id(
            dictionary.address(),
            code_points.as_ptr(),
            length(code_points),
            i32::from(force_lower_case),
        )
    }
}

/// The code points of `word_id` (at most `max` of them), with the count the C++ returns.
pub fn code_points(dictionary: &Dictionary, word_id: i32, max: usize) -> Vec<i32> {
    let _lock = crate::lock();
    let mut out = vec![0_i32; max];
    // SAFETY: `out` holds `max` ints, and the C++ writes no more than its count.
    let count = unsafe {
        aw_hook_trie_code_points(
            dictionary.address(),
            word_id,
            i32::try_from(max).expect("max fits"),
            out.as_mut_ptr(),
        )
    };
    out.truncate(usize::try_from(count).expect("a count"));
    out
}

/// `getProbabilityOfWord`: the probability of `word_id` after the previous word ids.
pub fn probability_of_word(dictionary: &Dictionary, prev_word_ids: &[i32], word_id: i32) -> i32 {
    let _lock = crate::lock();
    // SAFETY: the dictionary is open, and the slice is valid for `prev_word_ids.len()` ints.
    unsafe {
        aw_hook_trie_probability_of_word(
            dictionary.address(),
            prev_word_ids.as_ptr(),
            length(prev_word_ids),
            word_id,
        )
    }
}

/// The attributes of a word in context: its probability, and whether it is blacklisted, not a word
/// and possibly offensive. `map` is a multi-bigram map the lookup goes through, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attributes {
    pub probability: i32,
    pub is_blacklisted: bool,
    pub is_not_a_word: bool,
    pub is_possibly_offensive: bool,
}

/// `getWordAttributesInContext`, through `map` when given.
pub fn word_attributes(
    dictionary: &Dictionary,
    map: Option<&MultiBigramMap>,
    prev_word_ids: &[i32],
    word_id: i32,
) -> Attributes {
    let _lock = crate::lock();
    let mut flags = 0_i32;
    let map_address = map.map_or(0, MultiBigramMap::address);
    // SAFETY: the dictionary and the map (if any) are open, and `flags` is one int.
    let probability = unsafe {
        aw_hook_trie_word_attributes(
            dictionary.address(),
            map_address,
            prev_word_ids.as_ptr(),
            length(prev_word_ids),
            word_id,
            &mut flags,
        )
    };
    Attributes {
        probability,
        is_blacklisted: flags & 1 != 0,
        is_not_a_word: flags & 2 != 0,
        is_possibly_offensive: flags & 4 != 0,
    }
}

/// `iterateNgramEntries`: the (probability, target word id) of each bigram visit, in order.
pub fn ngram_entries(dictionary: &Dictionary, prev_word_ids: &[i32]) -> Vec<(i32, i32)> {
    let _lock = crate::lock();
    let flat = collect(|out, capacity| {
        // SAFETY: the dictionary is open; `out` holds `capacity` ints, and the slice is valid.
        unsafe {
            aw_hook_trie_ngram_entries(
                dictionary.address(),
                prev_word_ids.as_ptr(),
                length(prev_word_ids),
                out,
                capacity,
            )
        }
    });
    flat.as_chunks::<2>()
        .0
        .iter()
        .map(|&[a, b]| (a, b))
        .collect()
}

/// `iterateWords`: every word of the dictionary, in the order the C++ visits them.
pub fn words(dictionary: &Dictionary) -> Vec<Vec<i32>> {
    let _lock = crate::lock();
    let flat = collect(|out, capacity| {
        // SAFETY: the dictionary is open, and `out` holds `capacity` ints.
        unsafe { aw_hook_trie_words(dictionary.address(), out, capacity) }
    });
    let mut words = Vec::new();
    let mut rest = flat.as_slice();
    while let Some((&count, tail)) = rest.split_first() {
        let count = count as usize;
        words.push(tail[..count].to_vec());
        rest = &tail[count..];
    }
    words
}

/// One shortcut target as `BinaryDictionaryShortcutIterator` gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShortcutTarget {
    pub code_points: Vec<i32>,
    pub is_whitelist: bool,
    pub has_next: bool,
}

/// The shortcut targets of `word_id`, at most `max_depth` code points each.
pub fn shortcuts(dictionary: &Dictionary, word_id: i32, max_depth: usize) -> Vec<ShortcutTarget> {
    let _lock = crate::lock();
    let max_depth = i32::try_from(max_depth).expect("max depth fits");
    let flat = collect(|out, capacity| {
        // SAFETY: the dictionary is open, and `out` holds `capacity` ints.
        unsafe { aw_hook_trie_shortcuts(dictionary.address(), word_id, max_depth, out, capacity) }
    });
    let mut targets = Vec::new();
    let mut rest = flat.as_slice();
    while let [count, whitelist, next, tail @ ..] = rest {
        let count = *count as usize;
        targets.push(ShortcutTarget {
            code_points: tail[..count].to_vec(),
            is_whitelist: *whitelist != 0,
            has_next: *next != 0,
        });
        rest = &tail[count..];
    }
    targets
}

/// A `MultiBigramMap` in C++, freed when dropped.
#[derive(Debug)]
pub struct MultiBigramMap {
    handle: i64,
}

impl MultiBigramMap {
    /// `MultiBigramMap()`.
    pub fn new() -> Self {
        let _lock = crate::lock();
        // SAFETY: takes no arguments and returns a new map the drop frees.
        let handle = unsafe { aw_hook_trie_multi_bigram_new() };
        Self { handle }
    }

    fn address(&self) -> i64 {
        self.handle
    }

    /// `MultiBigramMap::getBigramProbability` for `next_word_id` after the previous word ids.
    pub fn probability(
        &mut self,
        dictionary: &Dictionary,
        prev_word_ids: &[i32],
        next_word_id: i32,
        unigram_probability: i32,
    ) -> i32 {
        let _lock = crate::lock();
        // SAFETY: the map and the dictionary are open, and the slice is valid for its length.
        unsafe {
            aw_hook_trie_multi_bigram_probability(
                self.handle,
                dictionary.address(),
                prev_word_ids.as_ptr(),
                length(prev_word_ids),
                next_word_id,
                unigram_probability,
            )
        }
    }
}

impl Default for MultiBigramMap {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MultiBigramMap {
    fn drop(&mut self) {
        let _lock = crate::lock();
        // SAFETY: the handle came from `aw_hook_trie_multi_bigram_new` and is freed only here.
        unsafe { aw_hook_trie_multi_bigram_delete(self.handle) };
    }
}

mod probe;
pub use probe::{Probe, ProbedNode, ProbedShortcut};
