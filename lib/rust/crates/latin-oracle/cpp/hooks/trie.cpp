// Hooks into LatinIME's version 2 trie, for comparing crates/latin's `trie` with it. Two kinds:
// the dictionary's own structure policy (reached through Dictionary::getDictionaryStructurePolicy),
// and a probe: this file's own node reader over a copy of a dictionary body, for the node-level
// reads the dictionary keeps private. The Rust side pads every copy, since the C++ reads without
// bounds checks; the probe's copy is padded the same way, and its views report the real size.
#include <cstdint>
#include <vector>

#include "dictionary/interface/dictionary_structure_with_buffer_policy.h"
#include "dictionary/interface/ngram_listener.h"
#include "dictionary/interface/word_listener.h"
#include "dictionary/property/word_attributes.h"
#include "dictionary/structure/pt_common/patricia_trie_reading_utils.h"
#include "dictionary/structure/pt_common/pt_node_params.h"
#include "dictionary/structure/pt_common/shortcut/shortcut_list_reading_utils.h"
#include "dictionary/structure/v2/bigram/bigram_list_policy.h"
#include "dictionary/structure/v2/shortcut/shortcut_list_policy.h"
#include "dictionary/structure/v2/ver2_patricia_trie_node_reader.h"
#include "dictionary/structure/v2/ver2_pt_node_array_reader.h"
#include "dictionary/utils/binary_dictionary_bigrams_iterator.h"
#include "dictionary/utils/multi_bigram_map.h"
#include "suggest/core/dictionary/dictionary.h"
#include "utils/byte_array_view.h"
#include "utils/int_array_view.h"

using latinime::BigramListPolicy;
using latinime::BinaryDictionaryBigramsIterator;
using latinime::CodePointArrayView;
using latinime::Dictionary;
using latinime::DictionaryStructureWithBufferPolicy;
using latinime::MultiBigramMap;
using latinime::NgramListener;
using latinime::PatriciaTrieReadingUtils;
using latinime::ReadOnlyByteArrayView;
using latinime::ShortcutListPolicy;
using latinime::ShortcutListReadingUtils;
using latinime::Ver2ParticiaTrieNodeReader;
using latinime::Ver2PtNodeArrayReader;
using latinime::WordIdArrayView;
using latinime::WordListener;

namespace {

// Enough zeros past a probe's body for any read a valid walk makes.
const size_t PROBE_PADDING = 64 * 1024;
const int MAX_CODE_POINTS = 48;

const DictionaryStructureWithBufferPolicy *policyOf(const int64_t dict) {
    return reinterpret_cast<const Dictionary *>(dict)->getDictionaryStructurePolicy();
}

// Copies at most `capacity` values to `out`, and returns how many there were.
int copyOut(const std::vector<int> &values, int *const out, const int capacity) {
    const int count = static_cast<int>(values.size());
    for (int i = 0; i < count && i < capacity; ++i) {
        out[i] = values[i];
    }
    return count;
}

// Collects the words of a walk as [length, code points...] per word.
class WordRecorder : public WordListener {
 public:
    void onVisitWord(const CodePointArrayView word) override {
        mValues.push_back(word.size());
        for (const int codePoint : word) {
            mValues.push_back(codePoint);
        }
    }

    const std::vector<int> &values() const { return mValues; }

 private:
    std::vector<int> mValues;
};

// Collects the ngram entries of a walk as (probability, target word id) pairs.
class NgramRecorder : public NgramListener {
 public:
    void onVisitEntry(const int ngramProbability, const int targetWordId) override {
        mValues.push_back(ngramProbability);
        mValues.push_back(targetWordId);
    }

    const std::vector<int> &values() const { return mValues; }

 private:
    std::vector<int> mValues;
};

// A node reader over a copy of a dictionary body, with the policies a dictionary builds over it.
// A copy of `size` bytes, followed by PROBE_PADDING zeros.
std::vector<uint8_t> paddedCopy(const uint8_t *const body, const size_t size) {
    std::vector<uint8_t> copy(body, body + size);
    copy.resize(size + PROBE_PADDING, 0);
    return copy;
}

struct Probe {
    std::vector<uint8_t> bytes;
    std::vector<int> table;
    const int *tablePointer;
    ReadOnlyByteArrayView view;
    BigramListPolicy bigrams;
    ShortcutListPolicy shortcuts;
    Ver2ParticiaTrieNodeReader nodes;
    Ver2PtNodeArrayReader arrays;

    Probe(const uint8_t *const body, const size_t size, const int *const tableValues,
            const int tableSize)
            : bytes(paddedCopy(body, size)),
              table(tableValues ? std::vector<int>(tableValues, tableValues + tableSize)
                                : std::vector<int>()),
              tablePointer(tableValues ? table.data() : nullptr),
              view(bytes.data(), size),
              bigrams(view),
              shortcuts(view),
              nodes(view, &bigrams, &shortcuts, tablePointer),
              arrays(view) {}
};

}  // namespace

extern "C" {

// The dictionary's word id of the word, or NOT_A_WORD_ID.
int aw_hook_trie_word_id(const int64_t dict, const int *const codePoints, const int length,
        const int forceLowerCase) {
    return policyOf(dict)->getWordId(CodePointArrayView(codePoints, length), forceLowerCase != 0);
}

// The code points of a word id: the count, with them written to `out` (MAX_CODE_POINTS long).
int aw_hook_trie_code_points(const int64_t dict, const int wordId, const int maxCodePoints,
        int *const out) {
    return policyOf(dict)->getCodePointsAndReturnCodePointCount(wordId, maxCodePoints, out);
}

// The probability of a word id after previous word ids, with no multi-bigram map.
int aw_hook_trie_probability_of_word(const int64_t dict, const int *const prevWordIds,
        const int prevWordCount, const int wordId) {
    return policyOf(dict)->getProbabilityOfWord(
            WordIdArrayView(prevWordIds, prevWordCount), wordId);
}

// The attributes of a word id after previous word ids, through `map` when it is not 0. The
// probability is returned, and `flags` gets bit 0 for blacklisted, 1 for not-a-word and 2 for
// possibly offensive.
int aw_hook_trie_word_attributes(const int64_t dict, const int64_t map, const int *const prevWordIds,
        const int prevWordCount, const int wordId, int *const flags) {
    MultiBigramMap *const multiBigramMap = reinterpret_cast<MultiBigramMap *>(map);
    const WordAttributes attributes = policyOf(dict)->getWordAttributesInContext(
            WordIdArrayView(prevWordIds, prevWordCount), wordId, multiBigramMap);
    *flags = (attributes.isBlacklisted() ? 1 : 0) | (attributes.isNotAWord() ? 2 : 0)
            | (attributes.isPossiblyOffensive() ? 4 : 0);
    return attributes.getProbability();
}

// The ngram entries of the first previous word, as (probability, target word id) pairs: the total
// count of ints the walk produced, with up to `capacity` of them written to `out`.
int aw_hook_trie_ngram_entries(const int64_t dict, const int *const prevWordIds,
        const int prevWordCount, int *const out, const int capacity) {
    NgramRecorder recorder;
    policyOf(dict)->iterateNgramEntries(WordIdArrayView(prevWordIds, prevWordCount), &recorder);
    return copyOut(recorder.values(), out, capacity);
}

// Every word of the dictionary, as [length, code points...] per word: the total count of ints
// written, with up to `capacity` of them written to `out`.
int aw_hook_trie_words(const int64_t dict, int *const out, const int capacity) {
    WordRecorder recorder;
    policyOf(dict)->iterateWords(&recorder);
    return copyOut(recorder.values(), out, capacity);
}

// The shortcut targets of a word id, as [length, whitelist, has next, code points...] per target,
// read through BinaryDictionaryShortcutIterator. The count of ints, as above.
int aw_hook_trie_shortcuts(const int64_t dict, const int wordId, const int maxDepth,
        int *const out, const int capacity) {
    std::vector<int> values;
    auto iterator = policyOf(dict)->getShortcutIterator(wordId);
    while (iterator.hasNextShortcutTarget()) {
        int codePoints[MAX_CODE_POINTS];
        int length = 0;
        bool isWhitelist = false;
        iterator.nextShortcutTarget(maxDepth, codePoints, &length, &isWhitelist);
        values.push_back(length);
        values.push_back(isWhitelist ? 1 : 0);
        values.push_back(iterator.hasNextShortcutTarget() ? 1 : 0);
        for (int i = 0; i < length; ++i) {
            values.push_back(codePoints[i]);
        }
    }
    return copyOut(values, out, capacity);
}

// A multi-bigram map: one per sequence of lookups.
int64_t aw_hook_trie_multi_bigram_new() {
    return reinterpret_cast<int64_t>(new MultiBigramMap());
}

void aw_hook_trie_multi_bigram_delete(const int64_t map) {
    delete reinterpret_cast<MultiBigramMap *>(map);
}

// MultiBigramMap::getBigramProbability for `nextWordId` after previous word ids.
int aw_hook_trie_multi_bigram_probability(const int64_t map, const int64_t dict,
        const int *const prevWordIds, const int prevWordCount, const int nextWordId,
        const int unigramProbability) {
    return reinterpret_cast<MultiBigramMap *>(map)->getBigramProbability(policyOf(dict),
            WordIdArrayView(prevWordIds, prevWordCount), nextWordId, unigramProbability);
}

// The probe over a copy of `size` bytes of body. `table` may be null.
int64_t aw_hook_probe_open(const uint8_t *const body, const size_t size, const int *const table,
        const int tableSize) {
    return reinterpret_cast<int64_t>(new Probe(body, size, table, tableSize));
}

void aw_hook_probe_close(const int64_t probe) {
    delete reinterpret_cast<Probe *>(probe);
}

// Layout of a node as the probe reports it, in ints: see the Rust side's PROBE_* constants.
// fetch (Ver2ParticiaTrieNodeReader::fetchPtNodeParamsInBufferFromPtNodePos) fills the node's
// getters; read_info (PatriciaTrieReadingUtils::readPtNodeInfo) fills the raw fields and flags.
const int FLAGS_SLOT = 3 + MAX_CODE_POINTS + 5;

void writeNode(const latinime::PtNodeParams &node, int *const out) {
    out[0] = node.isValid() ? 1 : 0;
    out[1] = node.getHeadPos();
    out[2] = node.getCodePointCount();
    for (int i = 0; i < MAX_CODE_POINTS; ++i) {
        out[3 + i] = i < node.getCodePointCount() ? node.getCodePointArrayView()[i] : 0;
    }
    int *const rest = out + 3 + MAX_CODE_POINTS;
    rest[0] = node.getProbability();
    rest[1] = node.getChildrenPos();
    rest[2] = node.getShortcutPos();
    rest[3] = node.getBigramsPos();
    rest[4] = node.getSiblingNodePos();
    rest[5] = -1;  // the flags: only the raw reading knows them
    rest[6] = node.isTerminal() ? 1 : 0;
    rest[7] = node.isNotAWord() ? 1 : 0;
    rest[8] = node.isPossiblyOffensive() ? 1 : 0;
    rest[9] = node.hasChildren() ? 1 : 0;
    rest[10] = node.hasBigrams() ? 1 : 0;
}

// Fills `out` (62 ints, the layout below) with the node at pos, as the dictionary's own reader returns it.
void aw_hook_probe_fetch(const int64_t probe, const int pos, int *const out) {
    const latinime::PtNodeParams node =
            reinterpret_cast<Probe *>(probe)->nodes.fetchPtNodeParamsInBufferFromPtNodePos(pos);
    writeNode(node, out);
}

// Fills `out` with the node at pos as the raw reading gives it, flags included (slot 5).
void aw_hook_probe_read_info(const int64_t probe, const int pos, int *const out) {
    Probe *const p = reinterpret_cast<Probe *>(probe);
    PatriciaTrieReadingUtils::NodeFlags flags = 0;
    int count = 0;
    int codePoints[MAX_CODE_POINTS];
    int probability = -1;
    int childrenPos = 0;
    int shortcutPos = 0;
    int bigramPos = 0;
    int siblingPos = 0;
    PatriciaTrieReadingUtils::readPtNodeInfo(p->view.data(), pos, &p->shortcuts, &p->bigrams,
            p->tablePointer, &flags, &count, codePoints, &probability, &childrenPos, &shortcutPos,
            &bigramPos, &siblingPos);
    const latinime::PtNodeParams node(pos, flags, count, codePoints, probability, childrenPos,
            shortcutPos, bigramPos, siblingPos);
    writeNode(node, out);
    out[FLAGS_SLOT] = flags;
}

// The node array at pos: 1 when it is valid, with its count and first node in out[0] and out[1].
int aw_hook_probe_array(const int64_t probe, const int pos, int *const out) {
    int count = 0;
    int first = 0;
    const bool valid = reinterpret_cast<Probe *>(probe)->arrays
            .readPtNodeArrayInfoAndReturnIfValid(pos, &count, &first);
    out[0] = count;
    out[1] = first;
    return valid ? 1 : 0;
}

// The forward link of the node array at pos: 1 when valid, with the link in out[0].
int aw_hook_probe_forward_link(const int64_t probe, const int pos, int *const out) {
    int next = 0;
    const bool valid = reinterpret_cast<Probe *>(probe)->arrays
            .readForwardLinkAndReturnIfValid(pos, &next);
    out[0] = next;
    return valid ? 1 : 0;
}

// The bigram visits of the list at pos, as BinaryDictionaryBigramsIterator makes them: the
// (probability, bigram position) pairs, with the total count of ints.
int aw_hook_probe_bigram_visits(const int64_t probe, const int pos, int *const out,
        const int capacity) {
    Probe *const p = reinterpret_cast<Probe *>(probe);
    std::vector<int> values;
    BinaryDictionaryBigramsIterator iterator(&p->bigrams, pos);
    while (iterator.hasNext()) {
        iterator.next();
        values.push_back(iterator.getProbability());
        values.push_back(iterator.getBigramPos());
    }
    return copyOut(values, out, capacity);
}

// The shortcut entries of the list at pos, as ShortcutListPolicy reads them: per entry, the
// probability, whitelist, has next, the target length, and the target. The total count of ints.
int aw_hook_probe_shortcut_entries(const int64_t probe, const int pos, const int maxDepth,
        int *const out, const int capacity) {
    Probe *const p = reinterpret_cast<Probe *>(probe);
    std::vector<int> values;
    if (pos == INT32_MIN) {  // NOT_A_DICT_POS
        return 0;
    }
    int at = p->shortcuts.getStartPos(pos);
    bool hasNext = true;
    while (hasNext) {
        int peek = at;
        const ShortcutListReadingUtils::ShortcutFlags flags =
                ShortcutListReadingUtils::getFlagsAndForwardPointer(p->view, &peek);
        int codePoints[MAX_CODE_POINTS];
        int length = 0;
        bool isWhitelist = false;
        p->shortcuts.getNextShortcut(maxDepth, codePoints, &length, &isWhitelist, &hasNext, &at);
        values.push_back(ShortcutListReadingUtils::getProbabilityFromFlags(flags));
        values.push_back(isWhitelist ? 1 : 0);
        values.push_back(hasNext ? 1 : 0);
        values.push_back(length);
        for (int i = 0; i < length; ++i) {
            values.push_back(codePoints[i]);
        }
    }
    return copyOut(values, out, capacity);
}

}  // extern "C"
