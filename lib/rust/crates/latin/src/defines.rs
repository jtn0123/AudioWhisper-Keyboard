//! LatinIME's shared constants and enums (`defines.h`), with the C++'s values and types.

pub const MAX_WORD_LENGTH: usize = 48;
pub const MAX_RESULTS: usize = 18;
pub const MAX_PROXIMITY_CHARS_SIZE: usize = 16;
pub const ADDITIONAL_PROXIMITY_CHAR_DELIMITER_CODE: i32 = 2;

pub const S_INT_MAX: i32 = i32::MAX;
pub const S_INT_MIN: i32 = i32::MIN;
/// The C++'s `3.14159265f`, which is this float exactly.
pub const M_PI_F: f32 = std::f32::consts::PI;
pub const MAX_PERCENTILE: i32 = 100;

pub const NOT_A_CODE_POINT: i32 = -1;
pub const NOT_A_DISTANCE: i32 = -1;
pub const NOT_A_COORDINATE: i32 = -1;
pub const NOT_AN_INDEX: i32 = -1;
pub const NOT_A_PROBABILITY: i32 = -1;
pub const NOT_A_DICT_POS: i32 = S_INT_MIN;
pub const NOT_A_WORD_ID: i32 = S_INT_MIN;
pub const NOT_A_TIMESTAMP: i32 = -1;
pub const NOT_A_WEIGHT_OF_LANG_MODEL_VS_SPATIAL_MODEL: f32 = -1.0;
pub const NOT_A_FIRST_WORD_CONFIDENCE: i32 = S_INT_MIN;

pub const CONFIDENCE_FOR_AUTO_COMMIT: i32 = 1_000_000;
pub const DISTANCE_WEIGHT_FOR_AUTO_COMMIT: i32 = 80 * CONFIDENCE_FOR_AUTO_COMMIT / 100;
pub const LENGTH_WEIGHT_FOR_AUTO_COMMIT: i32 = CONFIDENCE_FOR_AUTO_COMMIT;
pub const SPACE_COUNT_WEIGHT_FOR_AUTO_COMMIT: i32 = 80 * CONFIDENCE_FOR_AUTO_COMMIT / 100;

pub const KEYCODE_SPACE: i32 = b' ' as i32;
pub const KEYCODE_SINGLE_QUOTE: i32 = b'\'' as i32;
pub const KEYCODE_HYPHEN_MINUS: i32 = b'-' as i32;

pub const CODE_POINT_BEGINNING_OF_SENTENCE: i32 = 0x110000;

pub const SUGGEST_INTERFACE_OUTPUT_SCALE: f32 = 1_000_000.0;
pub const MAX_PROBABILITY: i32 = 255;
pub const MAX_BIGRAM_ENCODED_PROBABILITY: i32 = 15;
pub const MAX_VALUE_FOR_WEIGHTING: i32 = 10_000_000;
pub const MAX_KEY_COUNT_IN_A_KEYBOARD: usize = 64;
pub const MAX_POINTER_COUNT: usize = 1;
pub const MAX_POINTER_COUNT_G: usize = 2;
pub const MAX_PREV_WORD_COUNT_FOR_N_GRAM: usize = 3;

/// `ProximityType`: how a touched key relates to a dictionary letter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProximityType {
    /// The same letter, possibly with another case or accent.
    MatchChar,
    /// A letter near it on the keyboard.
    ProximityChar,
    /// A letter the language adds as near (`additional_proximity_chars`).
    AdditionalProximityChar,
    SubstitutionChar,
    UnrelatedChar,
}

/// `DoubleLetterLevel`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoubleLetterLevel {
    NotADoubleLetter,
    ADoubleLetter,
    AStrongDoubleLetter,
}

/// `CorrectionType`: what a step of the search did to match the input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectionType {
    Match,
    Proximity,
    AdditionalProximity,
    Substitution,
    Omission,
    Insertion,
    Transposition,
    Completion,
    Terminal,
    TerminalInsertion,
    NewWordSpaceOmission,
    NewWordSpaceSubstitution,
}
