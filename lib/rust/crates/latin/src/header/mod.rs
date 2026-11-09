//! The dictionary's header (`dictionary/header`): the fixed fields before the attributes, the
//! `key=value` attributes after them, and the settings they hold. [`HeaderPolicy`] is the C++
//! class of that name (`header_policy.{h,cpp}`); the readers it uses are in `read_write`.
//!
//! The C++ class implements `DictionaryHeaderStructurePolicy`, a virtual interface with a single
//! implementation. Here the interface's methods are inherent methods of [`HeaderPolicy`], and the
//! interface itself is not ported.
//!
//! Where the C++ reads outside the buffer, the port reads 0 there (as `bytes` does) and stops at
//! the buffer's end; see `read_write::fetch_all_header_attributes`.

mod entry_counts;
mod read_write;

#[cfg(test)]
mod tests;

use std::time::{SystemTime, UNIX_EPOCH};

use crate::bytes::FormatVersion;
use crate::defines::MAX_VALUE_FOR_WEIGHTING;

pub use entry_counts::{EntryCounts, MutableEntryCounters, NgramType};
pub use read_write::{
    AttributeMap, fetch_all_header_attributes, get_flags, get_header_size,
    insert_characters_into_vector, read_bool_attribute_value, read_code_point_table,
    read_code_point_vector_attribute_value, read_int_attribute_value,
};

/// `HeaderPolicy::MULTIPLE_WORDS_DEMOTION_RATE_KEY`.
const MULTIPLE_WORDS_DEMOTION_RATE_KEY: &str = "MULTIPLE_WORDS_DEMOTION_RATE";
/// `HeaderPolicy::REQUIRES_GERMAN_UMLAUT_PROCESSING_KEY`.
const REQUIRES_GERMAN_UMLAUT_PROCESSING_KEY: &str = "REQUIRES_GERMAN_UMLAUT_PROCESSING";
/// `HeaderPolicy::IS_DECAYING_DICT_KEY`. The C++ has a TODO to rename the attribute.
const IS_DECAYING_DICT_KEY: &str = "USES_FORGETTING_CURVE";
/// `HeaderPolicy::DATE_KEY`.
const DATE_KEY: &str = "date";
/// `HeaderPolicy::LAST_DECAYED_TIME_KEY`.
const LAST_DECAYED_TIME_KEY: &str = "LAST_DECAYED_TIME";
/// `HeaderPolicy::NGRAM_COUNT_KEYS`, for the types in `NgramType::ASCENDING`. The C++ list also
/// names `QUADGRAM_COUNT`, which is never read.
const NGRAM_COUNT_KEYS: [&str; 3] = ["UNIGRAM_COUNT", "BIGRAM_COUNT", "TRIGRAM_COUNT"];
/// `HeaderPolicy::MAX_NGRAM_COUNT_KEYS`, for the types in `NgramType::ASCENDING`.
const MAX_NGRAM_COUNT_KEYS: [&str; 3] = [
    "MAX_UNIGRAM_ENTRY_COUNT",
    "MAX_BIGRAM_ENTRY_COUNT",
    "MAX_TRIGRAM_ENTRY_COUNT",
];
/// `HeaderPolicy::DEFAULT_MAX_NGRAM_COUNTS`, for the types in `NgramType::ASCENDING`.
const DEFAULT_MAX_NGRAM_COUNTS: [i32; 3] = [10000, 30000, 30000];
/// `HeaderPolicy::EXTENDED_REGION_SIZE_KEY`.
const EXTENDED_REGION_SIZE_KEY: &str = "EXTENDED_REGION_SIZE";
/// `HeaderPolicy::HAS_HISTORICAL_INFO_KEY`.
const HAS_HISTORICAL_INFO_KEY: &str = "HAS_HISTORICAL_INFO";
/// `HeaderPolicy::LOCALE_KEY`.
const LOCALE_KEY: &str = "locale";
/// `HeaderPolicy::FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID_KEY`.
const FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID_KEY: &str =
    "FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID";

/// `HeaderPolicy::DEFAULT_MULTIPLE_WORDS_DEMOTION_RATE`.
const DEFAULT_MULTIPLE_WORDS_DEMOTION_RATE: i32 = 100;
/// `HeaderPolicy::MULTIPLE_WORD_COST_MULTIPLIER_SCALE`.
const MULTIPLE_WORD_COST_MULTIPLIER_SCALE: f32 = 100.0;
/// `HeaderPolicy::DEFAULT_FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID`.
const DEFAULT_FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID: i32 = 3;

/// `FormatUtils::UNKNOWN_VERSION`.
const UNKNOWN_VERSION: i32 = -1;

/// `HeaderPolicy`: the dictionary's header, read once from the file's bytes.
#[derive(Debug)]
pub struct HeaderPolicy {
    format_version: FormatVersion,
    size: i32,
    attributes: AttributeMap,
    locale: Vec<i32>,
    multi_word_cost_multiplier: f32,
    requires_german_umlaut_processing: bool,
    is_decaying_dict: bool,
    date: i32,
    last_decayed_time: i32,
    ngram_counts: EntryCounts,
    max_ngram_counts: EntryCounts,
    extended_region_size: i32,
    has_historical_info_of_words: bool,
    forgetting_curve_probability_values_table_id: i32,
}

impl HeaderPolicy {
    /// `HeaderPolicy(dictBuf, formatVersion)`, reading the header from `buffer`, the whole
    /// dictionary file. A missing date or last decayed time is the current time.
    pub fn new(buffer: &[u8], version: FormatVersion) -> Self {
        Self::with_current_time(buffer, version, current_time())
    }

    /// [`HeaderPolicy::new`] with `now` as the current time. The C++ defaults to
    /// `TimeKeeper::peekCurrentTime()`, the time the app last set; the time is a parameter here
    /// so the answer does not depend on the clock.
    pub fn with_current_time(buffer: &[u8], version: FormatVersion, now: i32) -> Self {
        let attributes = fetch_all_header_attributes(buffer);
        Self {
            format_version: version,
            size: get_header_size(buffer),
            locale: read_code_point_vector_attribute_value(&attributes, LOCALE_KEY),
            multi_word_cost_multiplier: read_multi_word_cost_multiplier(&attributes),
            requires_german_umlaut_processing: read_bool_attribute_value(
                &attributes,
                REQUIRES_GERMAN_UMLAUT_PROCESSING_KEY,
                false,
            ),
            is_decaying_dict: read_bool_attribute_value(&attributes, IS_DECAYING_DICT_KEY, false),
            date: read_int_attribute_value(&attributes, DATE_KEY, now),
            last_decayed_time: read_int_attribute_value(&attributes, LAST_DECAYED_TIME_KEY, now),
            ngram_counts: read_ngram_counts(&attributes),
            max_ngram_counts: read_max_ngram_counts(&attributes),
            extended_region_size: read_int_attribute_value(
                &attributes,
                EXTENDED_REGION_SIZE_KEY,
                0,
            ),
            has_historical_info_of_words: read_bool_attribute_value(
                &attributes,
                HAS_HISTORICAL_INFO_KEY,
                false,
            ),
            forgetting_curve_probability_values_table_id: read_int_attribute_value(
                &attributes,
                FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID_KEY,
                DEFAULT_FORGETTING_CURVE_PROBABILITY_VALUES_TABLE_ID,
            ),
            attributes,
        }
    }

    /// `getFormatVersionNumber`: the version as its number in the file, or -1 for versions this
    /// code cannot read. The C++ logs an error for 2 and 201; the port logs nothing.
    pub fn format_version_number(&self) -> i32 {
        match self.format_version {
            FormatVersion::Version202
            | FormatVersion::Version4OnlyForTesting
            | FormatVersion::Version402
            | FormatVersion::Version403 => self.format_version as i32,
            FormatVersion::Version2 | FormatVersion::Version201 | FormatVersion::Unknown => {
                UNKNOWN_VERSION
            }
        }
    }

    /// `isValid`: a decaying dictionary must also have the historical information of its words.
    pub fn is_valid(&self) -> bool {
        !self.is_decaying_dict || self.has_historical_info_of_words
    }

    /// `getSize`: the header's size in bytes, where the words start.
    pub fn size(&self) -> i32 {
        self.size
    }

    /// `getMultiWordCostMultiplier`.
    pub fn multi_word_cost_multiplier(&self) -> f32 {
        self.multi_word_cost_multiplier
    }

    /// `isDecayingDict`.
    pub fn is_decaying_dict(&self) -> bool {
        self.is_decaying_dict
    }

    /// `requiresGermanUmlautProcessing`.
    pub fn requires_german_umlaut_processing(&self) -> bool {
        self.requires_german_umlaut_processing
    }

    /// `getDate`.
    pub fn date(&self) -> i32 {
        self.date
    }

    /// `getLastDecayedTime`.
    pub fn last_decayed_time(&self) -> i32 {
        self.last_decayed_time
    }

    /// `getNgramCounts`.
    pub fn ngram_counts(&self) -> &EntryCounts {
        &self.ngram_counts
    }

    /// `getMaxNgramCounts`.
    pub fn max_ngram_counts(&self) -> EntryCounts {
        self.max_ngram_counts
    }

    /// `getExtendedRegionSize`.
    pub fn extended_region_size(&self) -> i32 {
        self.extended_region_size
    }

    /// `hasHistoricalInfoOfWords`.
    pub fn has_historical_info_of_words(&self) -> bool {
        self.has_historical_info_of_words
    }

    /// `shouldBoostExactMatches`: only a dictionary that does not decay boosts exact matches.
    pub fn should_boost_exact_matches(&self) -> bool {
        !self.is_decaying_dict
    }

    /// `getAttributeMap`: every attribute of the header.
    pub fn attribute_map(&self) -> &AttributeMap {
        &self.attributes
    }

    /// `getForgettingCurveProbabilityValuesTableId`.
    pub fn forgetting_curve_probability_values_table_id(&self) -> i32 {
        self.forgetting_curve_probability_values_table_id
    }

    /// `getLocale`: the locale's code points, empty when the header has none.
    pub fn locale(&self) -> &[i32] {
        &self.locale
    }

    /// `supportsBeginningOfSentence`: the format versions from 402 on do.
    pub fn supports_beginning_of_sentence(&self) -> bool {
        self.format_version as i32 >= FormatVersion::Version402 as i32
    }

    /// `getCodePointTable`: the table of frequent code points, or `None` when there is none (see
    /// [`read_code_point_table`]).
    pub fn code_point_table(&self) -> Option<&[i32]> {
        read_code_point_table(&self.attributes)
    }

    /// `readHeaderValueOrQuestionMark`, for logging: writes the value of `key` into `out`, its code
    /// points followed by a 0 and cut to fit. A missing key writes `?` and a 0. Nothing is written
    /// to an empty `out`, and a one-entry `out` gets only the 0.
    pub fn read_header_value_or_question_mark(&self, key: &str, out: &mut [i32]) {
        if out.is_empty() {
            return;
        }
        if out.len() == 1 {
            out[0] = 0;
            return;
        }
        match self.attributes.get(&insert_characters_into_vector(key)) {
            None => {
                out[0] = i32::from(b'?');
                out[1] = 0;
            }
            Some(value) => {
                let terminal_index = value.len().min(out.len() - 1);
                out[..terminal_index].copy_from_slice(&value[..terminal_index]);
                out[terminal_index] = 0;
            }
        }
    }
}

/// `readMultipleWordCostMultiplier`: the scale over the demotion rate, or the largest weight when
/// the rate is not positive.
fn read_multi_word_cost_multiplier(attributes: &AttributeMap) -> f32 {
    let demotion_rate = read_int_attribute_value(
        attributes,
        MULTIPLE_WORDS_DEMOTION_RATE_KEY,
        DEFAULT_MULTIPLE_WORDS_DEMOTION_RATE,
    );
    if demotion_rate <= 0 {
        return MAX_VALUE_FOR_WEIGHTING as f32;
    }
    MULTIPLE_WORD_COST_MULTIPLIER_SCALE / demotion_rate as f32
}

/// `readNgramCounts`: the counts of the types in `NgramType::ASCENDING`, 0 where missing.
fn read_ngram_counts(attributes: &AttributeMap) -> EntryCounts {
    let mut counters = MutableEntryCounters::default();
    for ngram_type in NgramType::ASCENDING {
        let index = ngram_type as usize;
        let count = read_int_attribute_value(attributes, NGRAM_COUNT_KEYS[index], 0);
        counters.set_ngram_count(ngram_type, count);
    }
    counters.entry_counts()
}

/// `readMaxNgramCounts`: the maxima of the types in `NgramType::ASCENDING`, with the C++'s defaults.
fn read_max_ngram_counts(attributes: &AttributeMap) -> EntryCounts {
    let mut counters = MutableEntryCounters::default();
    for ngram_type in NgramType::ASCENDING {
        let index = ngram_type as usize;
        let max_count = read_int_attribute_value(
            attributes,
            MAX_NGRAM_COUNT_KEYS[index],
            DEFAULT_MAX_NGRAM_COUNTS[index],
        );
        counters.set_ngram_count(ngram_type, max_count);
    }
    counters.entry_counts()
}

/// The clock `TimeKeeper::setCurrentTime` reads, `time(0)`, as the C++'s `int` holds it.
fn current_time() -> i32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since_epoch| since_epoch.as_secs() as i32)
}
