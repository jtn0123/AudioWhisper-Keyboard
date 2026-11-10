//! `suggest/core/dictionary/error_type_utils`: the kinds of error a search path has made, as bit
//! flags. `ErrorType` is mainly decided by the correction type, but also by whether the correction
//! really changed anything.

/// `ErrorTypeUtils::ErrorType`.
pub type ErrorType = u32;

/// `ErrorTypeUtils::NOT_AN_ERROR`.
pub const NOT_AN_ERROR: ErrorType = 0x0;
/// `ErrorTypeUtils::MATCH_WITH_WRONG_CASE`.
pub const MATCH_WITH_WRONG_CASE: ErrorType = 0x1;
/// `ErrorTypeUtils::MATCH_WITH_MISSING_ACCENT`.
pub const MATCH_WITH_MISSING_ACCENT: ErrorType = 0x2;
/// `ErrorTypeUtils::MATCH_WITH_MISSING_EXPLICIT_ACCENT`.
pub const MATCH_WITH_MISSING_EXPLICIT_ACCENT: ErrorType = 0x4;
/// `ErrorTypeUtils::MATCH_WITH_WRONG_ACCENT`.
pub const MATCH_WITH_WRONG_ACCENT: ErrorType = 0x8;
/// `ErrorTypeUtils::MATCH_WITH_DIGRAPH`.
pub const MATCH_WITH_DIGRAPH: ErrorType = 0x10;
/// An omission of a letter that can be left out on purpose (an apostrophe).
pub const INTENTIONAL_OMISSION: ErrorType = 0x20;
/// Substitution, omission and transposition.
pub const EDIT_CORRECTION: ErrorType = 0x40;
/// `ErrorTypeUtils::PROXIMITY_CORRECTION`.
pub const PROXIMITY_CORRECTION: ErrorType = 0x80;
/// `ErrorTypeUtils::COMPLETION`.
pub const COMPLETION: ErrorType = 0x100;
/// `ErrorTypeUtils::NEW_WORD`.
pub const NEW_WORD: ErrorType = 0x200;

/// `ErrorTypeUtils::ERRORS_TREATED_AS_AN_EXACT_MATCH`.
const ERRORS_TREATED_AS_AN_EXACT_MATCH: ErrorType =
    NOT_AN_ERROR | MATCH_WITH_WRONG_CASE | MATCH_WITH_MISSING_ACCENT | MATCH_WITH_DIGRAPH;
/// `ErrorTypeUtils::ERRORS_TREATED_AS_A_PERFECT_MATCH`.
const ERRORS_TREATED_AS_A_PERFECT_MATCH: ErrorType = NOT_AN_ERROR;
/// `ErrorTypeUtils::ERRORS_TREATED_AS_AN_EXACT_MATCH_WITH_INTENTIONAL_OMISSION`.
const ERRORS_TREATED_AS_AN_EXACT_MATCH_WITH_INTENTIONAL_OMISSION: ErrorType =
    ERRORS_TREATED_AS_AN_EXACT_MATCH | INTENTIONAL_OMISSION;

/// `ErrorTypeUtils::isExactMatch`.
pub fn is_exact_match(contained_error_types: ErrorType) -> bool {
    contained_error_types & !ERRORS_TREATED_AS_AN_EXACT_MATCH == 0
}

/// `ErrorTypeUtils::isPerfectMatch`.
pub fn is_perfect_match(contained_error_types: ErrorType) -> bool {
    contained_error_types & !ERRORS_TREATED_AS_A_PERFECT_MATCH == 0
}

/// `ErrorTypeUtils::isExactMatchWithIntentionalOmission`.
pub fn is_exact_match_with_intentional_omission(contained_error_types: ErrorType) -> bool {
    contained_error_types & !ERRORS_TREATED_AS_AN_EXACT_MATCH_WITH_INTENTIONAL_OMISSION == 0
}

/// `ErrorTypeUtils::isMissingExplicitAccent`.
pub fn is_missing_explicit_accent(error_type: ErrorType) -> bool {
    error_type & MATCH_WITH_MISSING_EXPLICIT_ACCENT != 0
}

/// `ErrorTypeUtils::isEditCorrectionError`.
pub fn is_edit_correction_error(error_type: ErrorType) -> bool {
    error_type & EDIT_CORRECTION != 0
}

/// `ErrorTypeUtils::isProximityCorrectionError`.
pub fn is_proximity_correction_error(error_type: ErrorType) -> bool {
    error_type & PROXIMITY_CORRECTION != 0
}

/// `ErrorTypeUtils::isCompletion`.
pub fn is_completion(error_type: ErrorType) -> bool {
    error_type & COMPLETION != 0
}
