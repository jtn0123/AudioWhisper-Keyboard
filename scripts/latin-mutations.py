"""Break the LatinIME port in crates/latin, require a test failure, restore exact bytes, rerun green.

python3 -I scripts/latin-mutations.py SUITE OUTPUT [CASE...]
SUITE is `golden` (the golden tests alone) or `oracle` (the differential tests against LatinIME's
C++, `--features oracle`, which need the C++: run them from a checkout of the generator commit
docs/phase6-oracle-goldens.md names, or of an earlier one). Every mutation the oracle kills, the
goldens must kill too: that document has the table.
Host only, no device. Compilation errors do not count. A run that passes LIMIT is stopped and
reported with exit `timeout`. One green run of the untouched source follows the mutations, after
each one's bytes are checked restored.
"""
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys

sys.dont_write_bytecode = True  # importing mutation_cases would otherwise leave a __pycache__ in scripts/
sys.path.insert(0, str(Path(__file__).resolve().parent))
import mutation_cases

ROOT = Path(__file__).resolve().parents[1]
LATIN = 'lib/rust/crates/latin/src/'
SESSION = 'lib/rust/crates/suggest/src/session.rs'
# name: (file, before, after, what it breaks). What it breaks picks the tests: 'latin' for the
# decoder, 'session' for the production session around it (crates/suggest).
CASES = {
    # Bytes: integers, code points and strings as the dictionary stores them.
    'bytes-s24-sign-bit-kept': (
        LATIN + 'bytes/mod.rs',
        '-(((i32::from(first) & 0x7F) << 16)', '-(((i32::from(first) & 0xFF) << 16)', 'latin'),
    'bytes-u24-middle-byte': (
        LATIN + 'bytes/mod.rs',
        '(u32::from(read_u8(buffer, pos + 1)) << 8)\n        ^ u32::from(read_u8(buffer, pos + 2))',
        '(u32::from(read_u8(buffer, pos + 1)) << 7)\n        ^ u32::from(read_u8(buffer, pos + 2))', 'latin'),
    'bytes-space-read-as-three-bytes': (
        LATIN + 'bytes/mod.rs',
        'if first < MINIMUM_ONE_BYTE_CHARACTER_VALUE {', 'if first <= MINIMUM_ONE_BYTE_CHARACTER_VALUE {', 'latin'),
    'bytes-table-off-by-one': (
        LATIN + 'bytes/mod.rs',
        '.get(usize::from(first - MINIMUM_ONE_BYTE_CHARACTER_VALUE))',
        '.get(usize::from(first - MINIMUM_ONE_BYTE_CHARACTER_VALUE) + 1)', 'latin'),
    'bytes-skip-one-short': (
        LATIN + 'bytes/mod.rs',
        'while code_point != NOT_A_CODE_POINT && length < max_length {\n        code_point = read_code_point_advance(buffer, None, pos);',
        'while code_point != NOT_A_CODE_POINT && length + 1 < max_length {\n        code_point = read_code_point_advance(buffer, None, pos);',
        'latin'),
    'bytes-testing-version-moved': (
        LATIN + 'bytes/mod.rs',
        '399 => FormatVersion::Version4OnlyForTesting,', '400 => FormatVersion::Version4OnlyForTesting,', 'latin'),
    # Chars: LatinIME's own lower case and base letters.
    'chars-lower-by-whole-code-point': (
        LATIN + 'chars/mod.rs',
        'let key = c as u16;', 'let key = c.clamp(0, 0xFFFF) as u16;', 'latin'),
    'chars-ascii-z-not-upper': (
        LATIN + 'chars/mod.rs',
        "(i32::from(b'A')..=i32::from(b'Z')).contains(&c)", "(i32::from(b'A')..i32::from(b'Z')).contains(&c)",
        'latin'),
    'chars-e-acute-lowers-wrong': (
        LATIN + 'chars/tables.rs', '(0x00C9, 0x00E9),', '(0x00C9, 0x00E8),', 'latin'),
    'chars-nbsp-base-kept': (
        LATIN + 'chars/tables.rs', '/* U+00A0 */ 0x0020, 0x00A1,', '/* U+00A0 */ 0x00A0, 0x00A1,', 'latin'),
    # Header: the attributes and the policy read from them.
    'header-zero-demotion-divides': (
        LATIN + 'header/mod.rs', 'if demotion_rate <= 0 {', 'if demotion_rate < 0 {', 'latin'),
    'header-default-max-trigrams': (
        LATIN + 'header/mod.rs',
        'const DEFAULT_MAX_NGRAM_COUNTS: [i32; 3] = [10000, 30000, 30000];',
        'const DEFAULT_MAX_NGRAM_COUNTS: [i32; 3] = [10000, 30000, 3000];', 'latin'),
    'header-value-cut-short': (
        LATIN + 'header/mod.rs',
        'let terminal_index = value.len().min(out.len() - 1);',
        'let terminal_index = value.len().min(out.len() - 2);', 'latin'),
    'header-sentence-start-from-403': (
        LATIN + 'header/mod.rs',
        'self.format_version as i32 >= FormatVersion::Version402 as i32',
        'self.format_version as i32 > FormatVersion::Version402 as i32', 'latin'),
    'header-missing-date-is-zero': (
        LATIN + 'header/mod.rs',
        'date: read_int_attribute_value(&attributes, DATE_KEY, now),',
        'date: read_int_attribute_value(&attributes, DATE_KEY, 0),', 'latin'),
    'header-last-duplicate-wins': (
        LATIN + 'header/read_write.rs',
        '        attributes\n            .entry(key_buffer[..key_length].to_vec())\n'
        '            .or_insert_with(|| value_buffer[..value_length].to_vec());',
        '        attributes.insert(\n            key_buffer[..key_length].to_vec(),\n'
        '            value_buffer[..value_length].to_vec(),\n        );', 'latin'),
    'header-int-saturates': (
        LATIN + 'header/read_write.rs',
        '.wrapping_mul(10)\n            .wrapping_add(', '.saturating_mul(10)\n            .saturating_add(', 'latin'),
    'header-key-limit-255': (
        LATIN + 'header/read_write.rs',
        'const MAX_ATTRIBUTE_KEY_LENGTH: usize = 256;', 'const MAX_ATTRIBUTE_KEY_LENGTH: usize = 255;', 'latin'),
    # Layout: the keyboard, the letters near a tap, and the typed input's state.
    'layout-sweet-spot-knee': (
        LATIN + 'layout/touch_position_correction_utils.rs',
        'const R1: f32 = 0.25;', 'const R1: f32 = 0.26;', 'latin'),
    'layout-signalling-nan-kept': (
        LATIN + 'layout/touch_position_correction_utils.rs',
        'f32::from_bits(x.to_bits() | QUIET_BIT)', 'x', 'latin'),
    'layout-only-bare-en-is-english': (
        LATIN + 'layout/additional_proximity_chars.rs',
        'locale.starts_with(&LOCALE_EN)', 'locale == LOCALE_EN', 'latin'),
    'layout-u-lacks-o': (
        LATIN + 'layout/additional_proximity_chars.rs',
        "const ADDITIONAL_U: [i32; 4] = ['a' as i32, 'e' as i32, 'i' as i32, 'o' as i32];",
        "const ADDITIONAL_U: [i32; 4] = ['a' as i32, 'e' as i32, 'i' as i32, 'i' as i32];", 'latin'),
    'layout-first-duplicate-key-wins': (
        LATIN + 'layout/proximity_info.rs',
        'Ok(at) => self.lower_code_point_to_key[at].1 = index as i32,', 'Ok(_) => {}', 'latin'),
    'layout-wide-key-third': (
        LATIN + 'layout/proximity_info.rs',
        'let key_width_half_diff = key_width.wrapping_sub(self.most_common_key_width) / 2;',
        'let key_width_half_diff = key_width.wrapping_sub(self.most_common_key_width) / 3;', 'latin'),
    'layout-bottom-row-half-height': (
        LATIN + 'layout/proximity_info.rs',
        'center_y.wrapping_add(self.key_heights[key]) > self.keyboard_height',
        'center_y.wrapping_add(self.key_heights[key] / 2) > self.keyboard_height', 'latin'),
    'layout-cells-round-down': (
        LATIN + 'layout/proximity_info.rs',
        'let cell = size.wrapping_add(cells).wrapping_sub(1) / cells;', 'let cell = size / cells;', 'latin'),
    'layout-on-key-right-edge': (
        LATIN + 'layout/proximity_info_utils.rs',
        'let right = left.wrapping_add(width).wrapping_add(1);', 'let right = left.wrapping_add(width);', 'latin'),
    'layout-continuation-ignores-times': (
        LATIN + 'layout/proximity_info_state_utils.rs',
        '        if input.times.is_some() && input.time(index) != sampled.times[i] {\n'
        '            return false;\n        }\n', '', 'latin'),
    'layout-additional-duplicates': (
        LATIN + 'layout/proximity_info_utils.rs',
        '                if proximities[..insert_pos].contains(&ac) {\n                    continue;\n'
        '                }\n', '', 'latin'),
    # Search: the typing model, its results, options, and the digraph and space paths.
    'search-transposition-cost': (
        LATIN + 'search/scoring_params.rs',
        'pub const TRANSPOSITION_COST: f32 = 0.5608;', 'pub const TRANSPOSITION_COST: f32 = 0.5609;', 'latin'),
    'search-space-substitution-cost': (
        LATIN + 'search/scoring_params.rs',
        'pub const SPACE_SUBSTITUTION_COST: f32 = 0.33;', 'pub const SPACE_SUBSTITUTION_COST: f32 = 0.34;', 'latin'),
    'search-digraph-penalty': (
        LATIN + 'search/scoring_params.rs',
        'pub const DIGRAPH_PENALTY_FOR_EXACT_MATCH: f32 = 0.03;',
        'pub const DIGRAPH_PENALTY_FOR_EXACT_MATCH: f32 = 0.04;', 'latin'),
    'search-small-cache-threshold': (
        LATIN + 'search/scoring_params.rs',
        'pub const LOCALE_WEIGHT_THRESHOLD_FOR_SMALL_CACHE_SIZE: f32 = 0.99;',
        'pub const LOCALE_WEIGHT_THRESHOLD_FOR_SMALL_CACHE_SIZE: f32 = 0.98;', 'latin'),
    'search-capped-prediction-threshold': (
        LATIN + 'search/scoring_params.rs',
        'pub const THRESHOLD_NEXT_WORD_PROBABILITY_FOR_CAPPED: i32 = 120;',
        'pub const THRESHOLD_NEXT_WORD_PROBABILITY_FOR_CAPPED: i32 = 121;', 'latin'),
    'search-longer-word-wins-ties': (
        LATIN + 'search/result/suggestion_results.rs',
        'left.code_point_count < right.code_point_count', 'left.code_point_count > right.code_point_count',
        'latin'),
    'search-zero-weight-is-default': (
        LATIN + 'search/result/output_utils.rs',
        'let weight_to_output = if weight_of_lang_model_vs_spatial_model < 0.0 {',
        'let weight_to_output = if weight_of_lang_model_vs_spatial_model <= 0.0 {', 'latin'),
    'search-offensive-not-blocked': (
        LATIN + 'search/result/output_utils.rs',
        'should_block_offensive_words && word_attributes.is_possibly_offensive();',
        'should_block_offensive_words && word_attributes.is_not_a_word();', 'latin'),
    'search-multi-word-not-a-word-allowed': (
        LATIN + 'search/dicnode/utils.rs',
        '&& (word_attributes.is_blacklisted() || word_attributes.is_not_a_word()))',
        '&& word_attributes.is_blacklisted())', 'latin'),
    'search-no-partial-commit': (
        LATIN + 'search/jni.rs',
        'output.space_indices[index] = word.index_to_partial_commit();', 'output.space_indices[index] = 0;',
        'latin'),
    'search-ue-digraph-as-ua': (
        LATIN + 'search/digraph.rs',
        "second: 'e' as i32,\n        composite_glyph: 0x00FC,", "second: 'a' as i32,\n        composite_glyph: 0x00FC,",
        'latin'),
    # Trie: nodes, arrays, bigrams, shortcuts and word lookups.
    'trie-bigram-step-size': (
        LATIN + 'trie/probability.rs',
        '/ (1.5_f32 + MAX_BIGRAM_ENCODED_PROBABILITY as f32);', '/ (1.0_f32 + MAX_BIGRAM_ENCODED_PROBABILITY as f32);',
        'latin'),
    'trie-negative-bigram-offset-added': (
        LATIN + 'trie/bigram.rs',
        'origin.wrapping_sub(offset)\n    } else {', 'origin.wrapping_add(offset)\n    } else {', 'latin'),
    'trie-two-byte-children-as-three': (
        LATIN + 'trie/reading.rs',
        'FLAG_CHILDREN_POSITION_TYPE_TWOBYTES => i32::from(read_u16_advance(buffer, pos)),',
        'FLAG_CHILDREN_POSITION_TYPE_TWOBYTES => read_u24_advance(buffer, pos) as i32,', 'latin'),
    'trie-array-size-mask': (
        LATIN + 'trie/reading.rs',
        '(i32::from(first & 0x7F) << 8) ^ i32::from(read_u8_advance(buffer, pos))',
        '(i32::from(first & 0x3F) << 8) ^ i32::from(read_u8_advance(buffer, pos))', 'latin'),
    'trie-not-a-word-flag': (
        LATIN + 'trie/reading.rs',
        'const FLAG_IS_NOT_A_WORD: NodeFlags = 0x02;', 'const FLAG_IS_NOT_A_WORD: NodeFlags = 0x01;', 'latin'),
    'trie-whitelist-probability': (
        LATIN + 'trie/shortcut.rs',
        'const WHITELIST_SHORTCUT_PROBABILITY: i32 = 15;', 'const WHITELIST_SHORTCUT_PROBABILITY: i32 = 14;',
        'latin'),
    'trie-previous-word-lower-case-first': (
        LATIN + 'trie/ngram.rs',
        'if word_id != NOT_A_WORD_ID || !try_lower_case {', 'if word_id != NOT_A_WORD_ID || try_lower_case {',
        'latin'),
    'trie-force-lower-ignored': (
        LATIN + 'trie/dynamic.rs',
        'let search = if force_lower_case_search {\n            to_lower_case(raw)',
        'let search = if force_lower_case_search {\n            raw', 'latin'),
    # The production session (crates/suggest) the keyboard puts the decoder's questions to.
    'session-never-blocks-offensive': (
        SESSION,
        'let options = [0, 0, i32::from(query.block_offensive_words), 0, 1000];',
        'let options = [0, 0, 0, 0, 1000];', 'session'),
    # Typing's scoring turns the weight -1 into 1.0, so a session that reset it to 1.0 would change
    # nothing; a weight the session hands in, instead of letting the scoring choose, does.
    'session-weight-given-not-chosen': (
        SESSION, 'self.output.weight = -1.0;\n        let options', 'self.output.weight = 0.5;\n        let options',
        'session'),
}

WORKSPACE = ROOT / 'lib/rust'
# The command each suite runs, by what the mutation breaks.
COMMANDS = {
    ('oracle', 'latin'): ['cargo', 'test', '--locked', '-p', 'latin', '--lib', '--features', 'oracle'],
    ('oracle', 'session'): ['cargo', 'test', '--locked', '-p', 'latin-oracle', '--test', 'native_session'],
    ('golden', 'latin'): ['cargo', 'test', '--locked', '-p', 'latin', '--lib', '--', 'goldens'],
    ('golden', 'session'): ['cargo', 'test', '--locked', '-p', 'suggest', '--test', 'native_session', '--',
                            'goldens'],
}
# Seconds a run may take; a mutation that makes a test loop for ever is stopped and reported.
LIMIT = 1200
TIMEOUT = 'timeout'
# A failed test's panic: where it was raised, relative to the workspace.
PANIC = re.compile(r"panicked at ([^:\s]+):(\d+):\d+:")


def run(output, name, command):
    """The command's exit status, or TIMEOUT when it ran past LIMIT and was stopped."""
    with (output / (name + '.log')).open('wb') as log:
        process = subprocess.Popen(command, cwd=WORKSPACE, stdout=log, stderr=subprocess.STDOUT,
                                   start_new_session=True)
        try:
            status = process.wait(timeout=LIMIT)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait()
            status = TIMEOUT
    (output / (name + '.exit')).write_text(str(status) + '\n')
    return status


def assertion_panics(text):
    """How many of the panics in a cargo test log were raised by an assert macro's line."""
    count = 0
    for path, line in PANIC.findall(text):
        source = WORKSPACE / path
        if source.is_file():
            count += 'assert' in source.read_text().splitlines()[int(line) - 1]
    return count


def check(output, name, command):
    """The exit status, whether it compiled, how many tests failed and how many by an assert."""
    status = run(output, name, command)
    text = (output / (name + '.log')).read_text(errors='replace')
    compiled = 'error[E' not in text and 'error: could not compile' not in text
    failed = sum(int(it) for it in re.findall(r'test result: FAILED\. \d+ passed; (\d+) failed', text))
    return status, compiled, failed, assertion_panics(text) if compiled else 0


def mutate(output, suite, name):
    relative, before, after, area = CASES[name]
    path = ROOT / relative
    original = path.read_bytes()
    source = original.decode()
    assert source.count(before) == 1, name + ': the mutation target must be unique'
    try:
        mutation_cases.write(path, source.replace(before, after))
        status, compiled, failed, assertions = check(output, name, COMMANDS[(suite, area)])
        assert compiled, name + ': the mutation must compile'
    finally:
        mutation_cases.write(path, original)
    assert path.read_bytes() == original, name + ': byte restoration failed'
    return {'mutation': name, 'area': area, 'killed': status != 0, 'exit': status, 'failed_tests': failed,
            'assertion_panics': assertions, 'sha256': hashlib.sha256(original).hexdigest()}


def check_edits():
    return mutation_cases.table(CASES)


def main():
    suite = sys.argv[1]
    assert suite in ('oracle', 'golden'), 'SUITE is oracle or golden'
    output = Path(sys.argv[2]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    names = sys.argv[3:] or list(CASES)
    results = []
    for name in names:
        results.append(mutate(output, suite, name))
        print(json.dumps(results[-1]), flush=True)
    for area in sorted({CASES[name][3] for name in names}):
        status, compiled, failed, _ = check(output, 'restored-' + area, COMMANDS[(suite, area)])
        assert status == 0 and compiled and failed == 0, area + ': the restored source must pass'
    (output / 'mutations.json').write_text(json.dumps(results, indent=2) + '\n')


if __name__ == '__main__':
    mutation_cases.check_flag()
    main()
