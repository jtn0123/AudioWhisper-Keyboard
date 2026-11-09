// Hooks into LatinIME's HeaderPolicy and HeaderReadWriteUtils, for comparing crates/latin's
// `header` with them. A HeaderPolicy is a heap object: aw_hook_header_new makes one and
// aw_hook_header_free deletes it. An attribute map is a heap std::map, from aw_hook_fetch_attributes.
// The caller pads every buffer, since the C++ reads without bounds checks.
#include <cstdint>
#include <iterator>
#include <vector>

#include "dictionary/header/header_policy.h"
#include "dictionary/header/header_read_write_utils.h"
#include "dictionary/interface/dictionary_header_structure_policy.h"
#include "dictionary/utils/entry_counters.h"
#include "dictionary/utils/format_utils.h"
#include "utils/time_keeper.h"

using latinime::DictionaryHeaderStructurePolicy;
using latinime::FormatUtils;
using latinime::HeaderPolicy;
using latinime::HeaderReadWriteUtils;
using latinime::NgramType;
using latinime::TimeKeeper;

namespace {

using AttributeMap = DictionaryHeaderStructurePolicy::AttributeMap;

const HeaderPolicy *policy(const void *handle) {
    return static_cast<const HeaderPolicy *>(handle);
}

const AttributeMap *attributes(const void *handle) {
    return static_cast<const AttributeMap *>(handle);
}

// The number of code points in the attribute `key`, or 0 when it is missing.
int attributeSize(const AttributeMap *map, const char *key) {
    AttributeMap::key_type keyVector;
    HeaderReadWriteUtils::insertCharactersIntoVector(key, &keyVector);
    const AttributeMap::const_iterator it = map->find(keyVector);
    return it == map->end() ? 0 : static_cast<int>(it->second.size());
}

} // namespace

extern "C" {
// The policy reads the clock only for a missing date, so `now` is what that default is.
void *aw_hook_header_new(const uint8_t *buffer, int version, int now) {
    TimeKeeper::startTestModeWithForceCurrentTime(now);
    HeaderPolicy *created =
            new HeaderPolicy(buffer, static_cast<FormatUtils::FORMAT_VERSION>(version));
    TimeKeeper::stopTestMode();
    return created;
}
void aw_hook_header_free(void *handle) {
    delete static_cast<HeaderPolicy *>(handle);
}

int aw_hook_header_size(const void *handle) { return policy(handle)->getSize(); }
int aw_hook_header_format_version_number(const void *handle) {
    return policy(handle)->getFormatVersionNumber();
}
int aw_hook_header_is_valid(const void *handle) { return policy(handle)->isValid(); }
float aw_hook_header_multi_word_cost_multiplier(const void *handle) {
    return policy(handle)->getMultiWordCostMultiplier();
}
int aw_hook_header_is_decaying_dict(const void *handle) {
    return policy(handle)->isDecayingDict();
}
int aw_hook_header_requires_german_umlaut_processing(const void *handle) {
    return policy(handle)->requiresGermanUmlautProcessing();
}
int aw_hook_header_date(const void *handle) { return policy(handle)->getDate(); }
int aw_hook_header_last_decayed_time(const void *handle) {
    return policy(handle)->getLastDecayedTime();
}
// `ngramType` is a NgramType's value, and `max` is nonzero for the maximum counts.
int aw_hook_header_ngram_count(const void *handle, int ngramType, int max) {
    const NgramType type = static_cast<NgramType>(ngramType);
    if (max) {
        return policy(handle)->getMaxNgramCounts().getNgramCount(type);
    }
    return policy(handle)->getNgramCounts().getNgramCount(type);
}
int aw_hook_header_extended_region_size(const void *handle) {
    return policy(handle)->getExtendedRegionSize();
}
int aw_hook_header_has_historical_info_of_words(const void *handle) {
    return policy(handle)->hasHistoricalInfoOfWords();
}
int aw_hook_header_should_boost_exact_matches(const void *handle) {
    return policy(handle)->shouldBoostExactMatches();
}
int aw_hook_header_forgetting_curve_table_id(const void *handle) {
    return policy(handle)->getForgettingCurveProbabilityValuesTableId();
}
int aw_hook_header_supports_beginning_of_sentence(const void *handle) {
    return policy(handle)->supportsBeginningOfSentence();
}

// The table's pointer (null when there is none), and its length, from the attribute.
const int *aw_hook_header_code_point_table(const void *handle, int *length) {
    *length = attributeSize(policy(handle)->getAttributeMap(), "codePointTable");
    return policy(handle)->getCodePointTable();
}
// Copies up to `capacity` of the locale's code points into `out`; returns how many it has.
int aw_hook_header_locale(const void *handle, int *out, int capacity) {
    const std::vector<int> *locale = policy(handle)->getLocale();
    const int size = static_cast<int>(locale->size());
    for (int i = 0; i < size && i < capacity; ++i) {
        out[i] = (*locale)[i];
    }
    return size;
}
void aw_hook_header_read_value_or_question_mark(const void *handle, const char *key, int *out,
        int outSize) {
    policy(handle)->readHeaderValueOrQuestionMark(key, out, outSize);
}
const void *aw_hook_header_attribute_map(const void *handle) {
    return policy(handle)->getAttributeMap();
}

// Attribute maps, from a policy or from aw_hook_fetch_attributes. Entries come in std::map order.
int aw_hook_attribute_map_size(const void *map) {
    return static_cast<int>(attributes(map)->size());
}
void aw_hook_attribute_map_entry(const void *map, int index, const int **key, int *keyLength,
        const int **value, int *valueLength) {
    const auto it = std::next(attributes(map)->begin(), index);
    *key = it->first.data();
    *keyLength = static_cast<int>(it->first.size());
    *value = it->second.data();
    *valueLength = static_cast<int>(it->second.size());
}

// The static readers of HeaderReadWriteUtils.
int aw_hook_get_header_size(const uint8_t *buffer) {
    return HeaderReadWriteUtils::getHeaderSize(buffer);
}
int aw_hook_get_flags(const uint8_t *buffer) { return HeaderReadWriteUtils::getFlags(buffer); }
void *aw_hook_fetch_attributes(const uint8_t *buffer) {
    AttributeMap *fetched = new AttributeMap();
    HeaderReadWriteUtils::fetchAllHeaderAttributes(buffer, fetched);
    return fetched;
}
void aw_hook_attributes_free(void *map) { delete static_cast<AttributeMap *>(map); }
const int *aw_hook_read_code_point_table(void *map, int *length) {
    *length = attributeSize(attributes(map), "codePointTable");
    return HeaderReadWriteUtils::readCodePointTable(static_cast<AttributeMap *>(map));
}
// Copies up to `capacity` of the value's code points into `out`; returns how many it has.
int aw_hook_read_code_point_vector_attribute(const void *map, const char *key, int *out,
        int capacity) {
    const std::vector<int> value =
            HeaderReadWriteUtils::readCodePointVectorAttributeValue(attributes(map), key);
    const int size = static_cast<int>(value.size());
    for (int i = 0; i < size && i < capacity; ++i) {
        out[i] = value[i];
    }
    return size;
}
int aw_hook_read_bool_attribute(const void *map, const char *key, int defaultValue) {
    return HeaderReadWriteUtils::readBoolAttributeValue(attributes(map), key, defaultValue != 0);
}
int aw_hook_read_int_attribute(const void *map, const char *key, int defaultValue) {
    return HeaderReadWriteUtils::readIntAttributeValue(attributes(map), key, defaultValue);
}
}
