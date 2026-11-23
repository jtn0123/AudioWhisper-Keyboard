# Phase 6: LatinIME's C++ becomes golden files

2026-11-21. This is the first half of
[phase 6](all-rust.md#6-retire-the-references-m-c-half-done). The owner decided that the C++
oracle becomes fixed test data in cargo tests, and then the C++ is deleted
([decision](decisions.md#everything-moves-to-rust-2026-11-21)). That is done here. The other
half, the `Reference*.kt` test copies, is not.

Until now, `lib/latinime` (LatinIME's C++, built only on the host) and the `latin-oracle` crate
around it let `cargo test --features latin/oracle` compare the Rust port in `crates/latin` with
the C++ answer by answer, in `:lib:rust:oracleCheck`. Now every answer the C++ gave is recorded in
plain text files beside the tests. The tests check against those files in the everyday
`cargo test` that `:lib:rust:check` runs, and the C++ is gone.

## What the oracle was asked

Every test that compared with the C++, and what became of it. The question counts are the
answers each golden file holds, which is the number of comparisons the oracle test made (a
comparison of several fields is one answer). Two dictionaries were used, both committed: Google's
AOSP dictionary (the fixture `crates/latin/tests/fixtures/aosp_main_en-US.dict`) and the one the
app ships (`app/src/main/assets/dicts/main_en-US.dict`). "Both" below means one golden file each.
No test needed anything that cannot be committed.

| Test | What it asks | Inputs | Answers |
|---|---|---|---|
| bytes: `random_buffers_match_the_goldens` | Integer reads of 0 to 5 bytes, signed 24-bit reads, code point and string reads (with and without a code point table) and skipping a string, each with the position after. | 20,000 seeded random buffers of 1 to 40 bytes | 200,000 |
| bytes: `format_versions_match_the_goldens` | The format version read from a header. | Every 16-bit version, broken magic numbers and short buffers, and both dictionaries | 65,541 |
| chars: `every_code_point_matches_the_goldens` | Lower case, base letter and base lower case. | Every code point to 0x110100, and odd values past it | 3,343,114 |
| header: `each_dictionary_header_matches_the_goldens` | Every `HeaderPolicy` answer: size, version, validity, multi-word cost, decay, umlaut flag, dates, n-gram counts, locale and every attribute. | Both dictionaries' headers (both) | 134 each |
| header: `random_headers_match_the_goldens` | The same policy answers. | 3,000 seeded headers built from known keys and odd values | 402,000 |
| header: `random_buffers_match_the_goldens` | The same policy answers on whatever parses. | 3,000 seeded random buffers | 399,052 |
| header: `integer_and_bool_values_match_the_goldens` | Integer and boolean attribute values as the policy reads them. | Every key with a fixed list of number texts, and random ones | 24,120 |
| header: `static_readers_on_every_format_version_match_the_goldens` | The static header readers (size, flags, attributes) for each format version. | Fixed versions with seeded headers | 938 |
| layout: `qwerty_keyboard_matches_the_goldens` | Every `ProximityInfo` answer on the app's QWERTY model: key at a point, nearest keys, distances, sweet spots, proximities. | Every point of the keyboard | 15,028,280 |
| layout: `random_keyboards_match_the_goldens` | The same answers. | 200 seeded random keyboards, sampled points | 94,059,491 |
| layout: `sweet_spot_factor_matches_the_goldens` | The sweet spot factor and the distances it scales, to the bit. | Factors from -0.75 to 3.75 by 1/400, with NaN, infinities and -0 | 43,620 |
| layout: `typed_inputs_match_the_goldens` | Everything the search reads from a `ProximityInfoState` after `initInputParams`. | Seeded queries from the generator on the AOSP words, on the QWERTY model | 33,236,328 |
| layout: `words_typed_letter_by_letter_match_the_goldens` | The same state, kept across calls as a word grows, with taps and times changed before the end. | AOSP words, QWERTY model | 24,871,099 |
| layout: `words_deleted_letter_by_letter_match_the_goldens` | The same state as a word is deleted back. | AOSP words, QWERTY model | 31,122,372 |
| layout: `states_after_a_decode_match_the_goldens` | The state a full decode leaves in the session. | Seeded queries, AOSP dictionary, QWERTY model | 529,651 |
| layout: `typed_inputs_on_random_keyboards_match_the_goldens` | The typed state on other keyboards. | Seeded random keyboards and inputs | 16,477,347 |
| search: `seeded_queries_match_the_goldens` | Every field of the suggestions (count, code points, scores, space indices, types, auto-commit confidence, weight by bits), on a fresh session each. | Seeded queries from the generator (both) | 20,000 each |
| search: `predictions_match_the_goldens` | Next-word predictions: no input, after zero to three words or a sentence start. | Seeded queries with the input cleared (both) | 3,000 each |
| search: `sentence_starts_match_the_goldens` | The first word of a text, typed or not. | Seeded queries (both) | 4,000 each |
| search: `offensive_blocking_matches_the_goldens` | The same question with offensive words allowed and blocked. | 6,000 seeded queries, asked twice (both) | 12,000 each |
| search: `capitalised_input_matches_the_goldens` | A capital first letter, every letter, or one in the middle. | Seeded queries (both) | 6,000 each |
| search: `missing_coordinates_match_the_goldens` | Taps with no coordinates, typed letter by letter on one session. | 1,500 seeded queries (both) | 13,042 each |
| search: `the_longest_inputs_match_the_goldens` | Inputs of 47 and 48 taps, the longest the app sends. | Words run together, half with a typo (both) | 400 each |
| search: `weights_and_options_match_the_goldens` | The options array and the weight the app never changes. | Seeded queries, 9 weights, 8 locale weights (both) | 9,000 each |
| search: `weights_while_typing_match_the_goldens` | The same, letter by letter on one session. | 600 seeded queries (both) | 5,320 each |
| search: `words_typed_and_deleted_match_the_goldens` | A word typed and deleted back on one session, sometimes changed in the middle, so the cache must not continue. | 2,500 seeded words (both) | 39,415 each |
| search: `unrelated_queries_on_one_session_match_the_goldens` | Unrelated queries one after another, every fifth asked twice. | 8,000 seeded queries (both) | 9,600 each |
| search: `predictions_between_typing_match_the_goldens` | Predictions between taps, which must not disturb the cache. | 800 seeded queries (both) | 8,732 each |
| search: `umlaut_digraphs_match_the_goldens` | German umlaut digraphs ("ae" for "ä") on a dictionary whose header asks for them. | Both dictionaries with the attribute added; umlaut words spelt out (both) | 31,476 each |
| search: `space_bar_keyboard_matches_the_goldens` | Space omission and substitution on a keyboard with a space bar. | Two short words typed as one, and seeded queries (both) | 24,387 each |
| trie: `word_list_matches_the_goldens_in_content_and_order` | Every word a walk of the trie visits, in order. | Both dictionaries (both) | 160,669 each |
| trie: `joined_word_text_matches_the_goldens` | The word list as the JNI returns it, one UTF-16 text. | Both dictionaries (both) | 1 each |
| trie: `every_word_matches_the_goldens_in_id_probability_attributes_and_code_points` | Each word's id, probability, attributes and code points. | Every word (both) | 482,007 each |
| trie: `random_words_match_the_goldens_in_id_probability_and_attributes` | The same for words of every kind: typos, case variants, empty, odd code points, too long. | 40,000 seeded words (both) | 68,090 each |
| trie: `random_ids_and_missing_ids_match_the_goldens_code_points` | Code points of an id, or of no word. | 5,000 seeded ids (both) | 5,000 each |
| trie: `every_node_reads_the_same_as_the_goldens` | Every node read and fetched as the C++'s own reader read it: every field and the flags. | Every node of the trie (both) | 392,288 each |
| trie: `arrays_and_forward_links_match_the_goldens_at_random_positions` | Node arrays and their forward links. | Every node and 20,000 random positions (both) | 216,145 each |
| trie: `bigram_lists_of_every_node_match_the_goldens` | Every bigram list's visits. | Every node with bigrams (both) | 160,625 AOSP, 41,095 shipped |
| trie: `shortcut_lists_of_every_node_match_the_goldens` | Every shortcut list's entries. | Every node with shortcuts (both) | 99 each |
| trie: `random_bodies_read_their_nodes_arrays_and_lists_as_the_goldens_say` | Nodes, arrays, forward links and both lists at every position. | 2,000 seeded random bodies | 1,054,940 |
| trie: `probability_after_previous_words_matches_the_goldens` | The n-gram probability after up to three words, with the JNI's rules for null, empty and long slots. | 40,000 seeded contexts (both) | 40,000 each |
| trie: `real_bigram_pairs_match_the_goldens` | The probability of real bigrams. | Bigram targets from the dictionary (both) | 19,996 AOSP, 5,163 shipped |
| trie: `bigram_entries_of_every_word_match_the_goldens` | Every word's bigram entries. | Every word (both) | 160,669 each |
| trie: `word_attributes_in_context_match_the_goldens` | Word attributes after a previous word. | 15,000 seeded pairs (both) | 15,000 each |
| trie: `multi_bigram_cache_answers_as_the_goldens_across_long_sequences` | The multi-bigram cache's probabilities across long sequences. | 60 sequences of 60 steps (both) | 7,200 each |
| suggest: `native_session_matches_the_goldens_for_four_thousand_seeded_questions` (was `latin-oracle/tests/native_session.rs`) | The production `suggest::session::Session` answers, as the app reads them: each suggestion to its terminator, scores, types, spaces, confidence, weight; and the word list. | 4,000 seeded queries (both) | 4,001 each |


All told, the oracle tests compared 224,560,122 answers.

Six more tests lived in `latin-oracle` itself (`src/tests.rs`). They checked the harness: that it
opened both dictionaries and listed about 160,000 words, rejected a file that is not a
dictionary, knew "the" and "thank you", corrected "teh", predicted after "thank", and that its
query lines read back. They asked nothing of the Rust port, and went with the crate.

## The golden files

One plain text file per test and dictionary, named `<area>.<test>[.<dictionary>].txt`: 74 in
`lib/rust/crates/latin/tests/golden/` and 2 in `lib/rust/crates/suggest/tests/golden/`.
Together they are 199,342 bytes for 224,560,122 answers. The largest file is 4,125 bytes.

224 million answers in full would not fit in 1 MB, so a file keeps hashes, and a few answers in
full:

```text
# search.seeded_queries.aosp: LatinIME's C++ answers to this test's questions (docs/phase6-oracle-goldens.md).
# Answers, the digest after the last, after each power of two (at N), and some in full.
count 20000
digest 115df7e85e591d2e
at 1 de380f9026fe8d25
at 2 270b292df089cf76
...
sample 0 e085ff7b55207ae2 18: [83,78,67,70,32,...]/.../... c... w... <- suggestions
```

* **`count`**: how many answers the test records.
* **`digest`**: every answer's hash, chained in order. Each answer is hashed with FNV-1a over
  64-bit words: integers sign-extended, floats by their bits, every list prefixed with its length.
  Each step of the hash and of the chain is a bijection, so one different word anywhere gives a
  different digest.
* **`at N`**: the chain after N answers, at each power of two. A failure says which range of
  answers first differs ("answers 4096 to 8191 differ from LatinIME's").
* **`sample`**: the first four answers, and the last before each `at`, in full with their hash
  and the expression that made them. A failure on one shows the answer next to LatinIME's.

A test records every answer on its tape (`latin_golden::Tape`) as it asks. The tape checks as it
goes and checks the count and the digest at the end. It also fails if the test never finishes it.
Every input is synthetic and seeded, or comes from the two committed dictionaries. The samples
are answers to those inputs, so they hold nothing else either.

The questions are the ones the C++ answered. The seeded generator moved from `latin-oracle` to
`latin-golden` unchanged (`queries.rs`). The inputs the C++ used to supply, its word list and
bigram targets, now come from the Rust, which the same tests held equal to the C++'s.

**The dictionaries are pinned.** `latin_golden::dictionaries()` checks both files' fingerprints
on first use. If the shipped dictionary is rebuilt, the tests fail with "the shipped dictionary
… has changed since its golden files were written … write the *.shipped.txt goldens again".
They do not fail with a misleading "differs from LatinIME's".

## The generator commit

The generator commit is **`8549be045`** ("test(latin): golden files of LatinIME's
answers, written by the oracle tests"). It holds the generator and the files it wrote. It was
`5290a7a62` before this branch was rebased, and the baseline gate and run (b) below ran there.
The rebase brought in `be35dd777`, which changed only doc comments in other crates, and the
one replaced mutation case below.

At that commit every test that records on a tape also compares each answer with the C++ when
built with `--features latin/oracle`. So writing the files is a run in which every answer
matched. To write them again:

```bash
git worktree add ../latin-goldens 8549be045
cd ../latin-goldens/lib/rust
LATIN_GOLDEN=write cargo test --locked -p latin --lib --features oracle -- goldens
LATIN_GOLDEN=write cargo test --locked -p latin-oracle --test native_session
```

The first command writes `crates/latin/tests/golden/` and the second
`crates/suggest/tests/golden/`. Copy the files back. Writing took about two minutes for the first
and one for the second.

**Not from the current tree.** Here `LATIN_GOLDEN` makes every golden test fail on purpose:
writing from this tree would record the port's own answers under LatinIME's name, and a
regression would pass as the reference. Changing one of the port's answers on purpose is a
decision for `docs/decisions.md` first, not a rewrite of the files.

**If the shipped dictionary is rebuilt**, its `*.shipped.txt` files must be written again from the
C++. Copy the new `main_en-US.dict` over the checkout's. Set the fingerprint the failure prints
in `FINGERPRINTS` (`crates/latin-golden/src/lib.rs`), in the checkout and in the current tree.
Run the two commands, then copy the `*.shipped.txt` files back. The checkout's Rust must still
agree with the C++ on the new dictionary, which the write run asserts. The current Rust must then
match what was written. The AOSP fixture does not change.

## The proof: the goldens catch what the oracle caught

`scripts/latin-mutations.py` breaks the port in 50 ways, one line each, across bytes, chars,
the header, the layout, the search, the trie and the production session in `crates/suggest`. It
needs each mutation to compile and a test to fail, puts the exact bytes back, and ends with a
green run of the untouched source. It ran each case twice, with the C++ still present:

* **(a) The oracle tests.** These ran in a checkout of `e2c7ce50d`, the old `main` before any
  golden existed, so they are the comparison tests as they were. The command was
  `cargo test --locked -p latin --lib --features oracle`; for the two session cases it was
  `-p latin-oracle --test native_session`.
* **(b) The golden tests alone.** These ran in a checkout of the generator commit with no
  feature, so nothing could reach the C++. The command was
  `cargo test --locked -p latin --lib -- goldens`; for the session cases it was
  `-p suggest --test native_session -- goldens`.

"Failed" is the number of test functions that failed. A golden test stops at the first answer
that differs, and the oracle tests mostly did too, so the numbers say how many tests saw the
change, not how many answers.

| Mutation | File | Oracle (a) | Goldens (b) |
|---|---|---|---|
| `bytes-s24-sign-bit-kept` | `latin/src/bytes/mod.rs` | killed, 1 failed | killed, 1 failed |
| `bytes-u24-middle-byte` | `latin/src/bytes/mod.rs` | killed, 34 failed | killed, 37 failed |
| `bytes-space-read-as-three-bytes` | `latin/src/bytes/mod.rs` | killed, 19 failed | killed, 19 failed |
| `bytes-table-off-by-one` | `latin/src/bytes/mod.rs` | killed, 1 failed | killed, 1 failed |
| `bytes-skip-one-short` | `latin/src/bytes/mod.rs` | killed, 1 failed | killed, 1 failed |
| `bytes-testing-version-moved` | `latin/src/bytes/mod.rs` | killed, 1 failed | killed, 1 failed |
| `chars-lower-by-whole-code-point` | `latin/src/chars/mod.rs` | killed, 2 failed | killed, 1 failed |
| `chars-ascii-z-not-upper` | `latin/src/chars/mod.rs` | killed, 23 failed | killed, 23 failed |
| `chars-e-acute-lowers-wrong` | `latin/src/chars/tables.rs` | killed, 6 failed | killed, 6 failed |
| `chars-nbsp-base-kept` | `latin/src/chars/tables.rs` | killed, 1 failed | killed, 1 failed |
| `header-zero-demotion-divides` | `latin/src/header/mod.rs` | killed, 3 failed | killed, 2 failed |
| `header-default-max-trigrams` | `latin/src/header/mod.rs` | killed, 6 failed | killed, 5 failed |
| `header-value-cut-short` | `latin/src/header/mod.rs` | killed, 6 failed | killed, 5 failed |
| `header-sentence-start-from-403` | `latin/src/header/mod.rs` | killed, 4 failed | killed, 3 failed |
| `header-missing-date-is-zero` | `latin/src/header/mod.rs` | killed, 6 failed | killed, 4 failed |
| `header-last-duplicate-wins` | `latin/src/header/read_write.rs` | killed, 4 failed | killed, 3 failed |
| `header-int-saturates` | `latin/src/header/read_write.rs` | killed, 3 failed | killed, 2 failed |
| `header-key-limit-255` | `latin/src/header/read_write.rs` | killed, 3 failed | killed, 3 failed |
| `layout-sweet-spot-knee` | `latin/src/layout/touch_position_correction_utils.rs` | killed, 1 failed | killed, 1 failed |
| `layout-signalling-nan-kept` | `latin/src/layout/touch_position_correction_utils.rs` | killed, 1 failed | killed, 1 failed |
| `layout-only-bare-en-is-english` | `latin/src/layout/additional_proximity_chars.rs` | killed, 20 failed | killed, 20 failed |
| `layout-u-lacks-o` | `latin/src/layout/additional_proximity_chars.rs` | killed, 20 failed | killed, 20 failed |
| `layout-first-duplicate-key-wins` | `latin/src/layout/proximity_info.rs` | killed, 2 failed | killed, 2 failed |
| `layout-wide-key-third` | `latin/src/layout/proximity_info.rs` | killed, 3 failed | killed, 3 failed |
| `layout-bottom-row-half-height` | `latin/src/layout/proximity_info.rs` | killed, 19 failed | killed, 19 failed |
| `layout-cells-round-down` | `latin/src/layout/proximity_info.rs` | killed, 20 failed | killed, 20 failed |
| `layout-on-key-right-edge` | `latin/src/layout/proximity_info_utils.rs` | killed, 1 failed | killed, 1 failed |
| `layout-continuation-ignores-times` | `latin/src/layout/proximity_info_state_utils.rs` | killed, 2 failed | killed, 2 failed |
| `layout-additional-duplicates` | `latin/src/layout/proximity_info_utils.rs` | killed, 7 failed | killed, 7 failed |
| `search-transposition-cost` | `latin/src/search/scoring_params.rs` | killed, 13 failed | killed, 13 failed |
| `search-space-substitution-cost` | `latin/src/search/scoring_params.rs` | killed, 1 failed | killed, 1 failed |
| `search-digraph-penalty` | `latin/src/search/scoring_params.rs` | killed, 1 failed | killed, 1 failed |
| `search-small-cache-threshold` | `latin/src/search/scoring_params.rs` | killed, 1 failed | killed, 1 failed |
| `search-capped-prediction-threshold` | `latin/src/search/scoring_params.rs` | killed, 13 failed | killed, 13 failed |
| `search-longer-word-wins-ties` | `latin/src/search/result/suggestion_results.rs` | killed, 14 failed | killed, 14 failed |
| `search-zero-weight-is-default` | `latin/src/search/result/output_utils.rs` | killed, 2 failed | killed, 2 failed |
| `search-offensive-not-blocked` | `latin/src/search/result/output_utils.rs` | killed, 11 failed | killed, 11 failed |
| `search-multi-word-not-a-word-allowed` | `latin/src/search/dicnode/utils.rs` | killed, 13 failed | killed, 13 failed |
| `search-no-partial-commit` | `latin/src/search/jni.rs` | killed, 14 failed | killed, 14 failed |
| `search-ue-digraph-as-ua` | `latin/src/search/digraph.rs` | killed, 1 failed | killed, 1 failed |
| `trie-bigram-step-size` | `latin/src/trie/probability.rs` | killed, 19 failed | killed, 18 failed |
| `trie-negative-bigram-offset-added` | `latin/src/trie/bigram.rs` | killed, 22 failed | killed, 21 failed |
| `trie-two-byte-children-as-three` | `latin/src/trie/reading.rs` | killed, 26 failed | killed, 33 failed |
| `trie-array-size-mask` | `latin/src/trie/reading.rs` | killed, 2 failed | killed, 2 failed |
| `trie-not-a-word-flag` | `latin/src/trie/reading.rs` | killed, 23 failed | killed, 30 failed |
| `trie-whitelist-probability` | `latin/src/trie/shortcut.rs` | killed, 13 failed | killed, 13 failed |
| `trie-previous-word-lower-case-first` | `latin/src/trie/ngram.rs` | killed, 13 failed | killed, 13 failed |
| `trie-force-lower-ignored` | `latin/src/trie/dynamic.rs` | killed, 14 failed | killed, 14 failed |
| `session-never-blocks-offensive` | `suggest/src/session.rs` | killed, 1 failed | killed, 1 failed |
| `session-weight-given-not-chosen` | `suggest/src/session.rs` | killed, 1 failed | killed, 1 failed |

**The oracle (a) killed 50 of 50, and the goldens (b) killed 50 of 50.** The goldens killed every
case the oracle killed, so no golden case had to be added. By area there are 6 bytes cases,
4 chars, 8 header, 11 layout, 11 search, 8 trie and 2 session. None timed out. Every run compiled,
its bytes were checked restored, and both suites' runs of the untouched source passed.

One case was replaced after the first runs. `session-weight-not-reset` set the session's weight
to 1.0 instead of −1. The oracle let it survive, and so did the goldens. Both were right to:
typing's scoring turns −1 into 1.0 (`adjusted_weight_of_lang_model_vs_spatial_model`), so the
mutation changed nothing. `session-weight-given-not-chosen` sets 0.5, which the scoring uses as
given. Both suites ran it on its own.

The deletion commits change only how the tests record, not what. `same!(tape, ours, theirs)`
became `record!(tape, ours)`. Every golden test passes on the final tree against the files the
generator commit wrote, with the same count and digest, so the final tests record the same answers
in the same order. Whatever changes an answer under (b) still fails them.

To check that, eight cases ran again on the final tree, with the C++ gone. In each area they are
the ones the fewest golden tests caught: `bytes-table-off-by-one`, `chars-nbsp-base-kept`,
`header-zero-demotion-divides`, `layout-signalling-nan-kept`, `search-ue-digraph-as-ua`,
`trie-array-size-mask` and both session cases. All eight were killed, each by as many tests as
under (b), and the untouched final tree passed.

Running `python3 -I scripts/latin-mutations.py golden OUT` today repeats (b). The oracle suite
needs a checkout of the generator commit, or of an earlier one with today's script copied in, as
run (a) did.

## What was deleted

Each in its own commit, after the generator commit:

| What | Commit | Size |
|---|---|---|
| Every `#[cfg(feature = "oracle")]` block, and the test support only they used. That covers the C++ half of each fixture, `TemporaryFile`, the C++ handles and readers, and the `same!` comparisons, which became `record!`. | `87acaa53f` | 24 files, 1,115 lines out, 458 in |
| `cargoOracleTest`, `cargoOracleClippy` and `oracleCheck` in `lib/rust/build.gradle.kts`, and `:lib:rust:oracleCheck` in CI and the README | `5c56d1e9f` | 3 files, 29 lines out, 14 in |
| The `oracle` feature and the optional `latin-oracle` dependency in `crates/latin/Cargo.toml` | `78dd9da2a` | 2 files, 9 lines |
| `[profile.dev.package.latin-oracle]` | `82e07a486` | 1 file, 5 lines out, 2 in |
| `lib/rust/crates/latin-oracle`: the C++ built for the host with `cc`, its FFI hooks, six self-tests and `tests/native_session.rs`. The query generator moved to `latin-golden` first. Also `default-members` in `lib/rust/Cargo.toml`, which existed to leave this crate out, and the lock entries `latin-oracle`, `cc`, `find-msvc-tools` and `shlex`. `--locked` still passes. | `04cd9e212` | 26 crate files: 4,322 lines, 164,198 bytes |
| `lib/latinime`: LatinIME's C++ sources and headers | `5cc238fe3` | 142 files: 16,904 lines, 715,866 bytes |

`lib/latinime` held no licence file of its own. The Apache-2.0 attribution for the port moves
to `lib/rust/crates/latin/NOTICE` and an in-app licence entry,
`app/src/main/config/libraries/aosp-latinime.json` (`6aa243114`). The licence text is the
repository's `LICENSE`. All told, the ten commits before this record remove 22,578 lines and add
5,012, which includes the 76 golden files.

Out of scope, and kept: the `Reference*.kt` test copies,
`app/src/test/java/com/android/inputmethod/latin/{BinaryDictionary,NativeLibrary}.java`,
`LatinImeJniTest`, `AutoCorrectionTest` and `crates/awnative/src/latinime.rs`. None of them uses
the C++. `NativeLibrary` loads `libawnative`, the Rust library. `LatinImeJniTest` drives the Java
classes whose natives `libawnative` registers under LatinIME's class names. `AutoCorrectionTest`
calls no native code.

## The gate

Before:

```bash
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :lib:rust:oracleCheck :app:checkNativeDeclarations :app:checkJvmLedger assembleDebug :app:compileDebugAndroidTestKotlin
```

After:

```bash
./gradlew --stacktrace test :app:lintDebug :lib:rust:check :app:checkNativeDeclarations :app:checkJvmLedger assembleDebug :app:compileDebugAndroidTestKotlin
```

`:lib:rust:check` keeps everything `oracleCheck` covered:

| Check | Before | After |
|---|---|---|
| Format | `cargoFmtCheck`: `cargo fmt --all --check` | the same |
| Clippy | `cargoClippy`: `cargo clippy --all-targets --locked -- -D warnings` on the default members (every crate but `latin-oracle`); `cargoOracleClippy` added `--workspace --features latin/oracle` | `cargoClippy`: `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Tests | `cargoTest`: `cargo test --locked` on the default members; `cargoOracleTest`: `cargo test --workspace --features latin/oracle --locked` | `cargoTest`: `cargo test --workspace --locked`, the goldens included |
| Android clippy | `cargoAndroidClippy`, `cargoAndroidHybridClippy` | the same |

Nothing new had to be wired in. With `latin-oracle` and `default-members` gone, `--workspace`
is every crate. The `oracle` feature added nothing but that crate and the comparisons, both
deleted. So clippy and the tests now see all the code they saw under either task before.

## Cargo test counts

Results are summed from every `test result:` line of the gate log (passed, failed, ignored):

| Run | Commit | Passed | Failed | Ignored | Made of |
|---|---|---|---|---|---|
| Before, at the old `main` | `e2c7ce50d` | 2,195 | 0 | 0 | `cargoTest` 1,065 + `cargoOracleTest` 1,130 (split worked out below) |
| Before, at the generator commit | `5290a7a62` | 2,269 | 0 | 0 | `cargoTest` 1,131 + `cargoOracleTest` 1,138 |
| After | `847eff4ec` | 1,131 | 0 | 0 | `cargoTest` 1,131 |

The split at `e2c7ce50d` is worked out, not measured. `cargoOracleTest` ran every test: the
generator commit's 1,138 less the 8 it added, which is 1,130. `cargoTest` left out
`latin-oracle`'s 7 and the 58 below, which leaves 1,065. The two add up to the 2,195 measured.

From 2,195 to 2,269, two things changed:

* At `e2c7ce50d`, 58 of the 94 tests in `latin` compiled only with the oracle feature, so they
  ran only in `cargoOracleTest`. Recording on tapes made them run without it, so `cargoTest`
  gained 58.
* The generator commit added 8 test functions: `latin-golden`'s 7 and `suggest`'s session
  goldens. These ran in both tasks, which is 16 more.

From 2,269 to 1,131, the second run of everything with the feature is gone. That is
`cargoOracleTest`'s 1,138, which included `latin-oracle`'s six self-tests and its
`native_session`. No test that `cargoTest` ran is lost. The source has 1,135 `#[test]` functions
at the generator commit and 1,128 after, and the 7 that went are `latin-oracle`'s. The everyday
check now runs all 94 `latin` tests. Before, it ran 36 of them.

The rest of the gate, before (the baseline gate, at the generator commit) and after:

| | Before | After |
|---|---|---|
| JVM tests | 956 in 233 files, 0 failures | 956 in 233 files, 0 failures |
| Native declarations | 389, 0 hand-written, 0 mismatched | 389, 0 hand-written, 0 mismatched |
| JVM ledger | 436 files, 436 rows | 436 files, 436 rows |
| Lint | AndroidGradlePluginVersion ×4, NewerVersionAvailable ×4, ChromeOsAbiSupport ×1, GradleDependency ×1 | the same, and no errors |

## Owed

* **The Kotlin half of phase 6.** The `Reference*.kt` test copies remain, each until the wrapper
  it guards is gone ([all-rust.md](all-rust.md#6-retire-the-references-m-c-half-done)).
* **An x86-64 or Linux run of the goldens.** The goldens were written and have only been checked
  on macOS arm64, as CI also runs. The port's floating point is plain IEEE `f32` arithmetic plus
  four `mul_add` calls, which round correctly on every target, and it calls no `libm` function.
  So another host should give the same bits, but none has been tried. This is listed in
  [owed-checks.md](owed-checks.md#not-device-checks-but-still-owed).
* **A rebuilt shipped dictionary needs the generator commit.** No test is owed now. But the
  `*.shipped.txt` goldens can only be written again from a checkout of the generator commit, as
  [above](#the-generator-commit), because the C++ is gone. Once the decoder is meant to differ
  from LatinIME, they come from the Rust itself after review
  ([decision](decisions.md#latinimes-c-answers-are-kept-as-digests-on-both-dictionaries-2026-11-21)).
* `scripts/latin-mutations.py`, like the other mutation scripts, had no `--check` mode to catch
  a case whose line has been renamed. It has one now, and `:app:checkMutationCases` runs it in
  the gate ([rust.md, checks](rust.md#checks)).
