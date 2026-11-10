//! `suggest/core/dictionary/digraph_utils`: letters a language may type as two (the German
//! umlauts: "ue" for u-umlaut). A dictionary node holding the composite letter is also tried as
//! its two letters in turn.

use crate::chars;
use crate::defines::NOT_A_CODE_POINT;
use crate::header::HeaderPolicy;

/// `DigraphUtils::DigraphCodePointIndex`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DigraphCodePointIndex {
    /// `NOT_A_DIGRAPH_INDEX`.
    NotADigraphIndex,
    /// `FIRST_DIGRAPH_CODEPOINT`.
    FirstDigraphCodepoint,
    /// `SECOND_DIGRAPH_CODEPOINT`.
    SecondDigraphCodepoint,
}

/// `DigraphUtils::DigraphType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DigraphType {
    /// `DIGRAPH_TYPE_NONE`.
    None,
    /// `DIGRAPH_TYPE_GERMAN_UMLAUT`.
    GermanUmlaut,
}

/// `DigraphUtils::digraph_t`.
struct Digraph {
    /// `first`.
    first: i32,
    /// `second`.
    second: i32,
    /// `compositeGlyph`.
    composite_glyph: i32,
}

/// `DigraphUtils::GERMAN_UMLAUT_DIGRAPHS`.
const GERMAN_UMLAUT_DIGRAPHS: [Digraph; 3] = [
    // U+00E4 : LATIN SMALL LETTER A WITH DIAERESIS
    Digraph {
        first: 'a' as i32,
        second: 'e' as i32,
        composite_glyph: 0x00E4,
    },
    // U+00F6 : LATIN SMALL LETTER O WITH DIAERESIS
    Digraph {
        first: 'o' as i32,
        second: 'e' as i32,
        composite_glyph: 0x00F6,
    },
    // U+00FC : LATIN SMALL LETTER U WITH DIAERESIS
    Digraph {
        first: 'u' as i32,
        second: 'e' as i32,
        composite_glyph: 0x00FC,
    },
];

/// `DigraphUtils::USED_DIGRAPH_TYPES`.
const USED_DIGRAPH_TYPES: [DigraphType; 1] = [DigraphType::GermanUmlaut];

/// `DigraphUtils::hasDigraphForCodePoint`: whether the dictionary's language types
/// `composite_glyph_code_point` as two letters.
pub fn has_digraph_for_code_point(header: &HeaderPolicy, composite_glyph_code_point: i32) -> bool {
    let digraph_type = digraph_type_for_dictionary(header);
    digraph_for_digraph_type_and_code_point(digraph_type, composite_glyph_code_point).is_some()
}

/// `DigraphUtils::getDigraphTypeForDictionary`.
fn digraph_type_for_dictionary(header: &HeaderPolicy) -> DigraphType {
    if header.requires_german_umlaut_processing() {
        DigraphType::GermanUmlaut
    } else {
        DigraphType::None
    }
}

/// `DigraphUtils::getDigraphCodePointForIndex`: the first or second letter of the digraph of
/// `composite_glyph_code_point`, `NOT_A_CODE_POINT` when it has none or `index` is none.
pub fn digraph_code_point_for_index(
    composite_glyph_code_point: i32,
    index: DigraphCodePointIndex,
) -> i32 {
    if index == DigraphCodePointIndex::NotADigraphIndex {
        return NOT_A_CODE_POINT;
    }
    let Some(digraph) = digraph_for_code_point(composite_glyph_code_point) else {
        return NOT_A_CODE_POINT;
    };
    match index {
        DigraphCodePointIndex::FirstDigraphCodepoint => digraph.first,
        DigraphCodePointIndex::SecondDigraphCodepoint => digraph.second,
        DigraphCodePointIndex::NotADigraphIndex => NOT_A_CODE_POINT,
    }
}

/// `DigraphUtils::getAllDigraphsForDigraphTypeAndReturnSize`.
fn all_digraphs(digraph_type: DigraphType) -> &'static [Digraph] {
    match digraph_type {
        DigraphType::GermanUmlaut => &GERMAN_UMLAUT_DIGRAPHS,
        DigraphType::None => &[],
    }
}

/// `DigraphUtils::getDigraphForCodePoint`: over the types any dictionary may use.
fn digraph_for_code_point(composite_glyph_code_point: i32) -> Option<&'static Digraph> {
    USED_DIGRAPH_TYPES
        .iter()
        .find_map(|&it| digraph_for_digraph_type_and_code_point(it, composite_glyph_code_point))
}

/// `DigraphUtils::getDigraphForDigraphTypeAndCodePoint`.
fn digraph_for_digraph_type_and_code_point(
    digraph_type: DigraphType,
    composite_glyph_code_point: i32,
) -> Option<&'static Digraph> {
    let lower = chars::to_lower_case(composite_glyph_code_point);
    all_digraphs(digraph_type)
        .iter()
        .find(|it| it.composite_glyph == lower)
}
