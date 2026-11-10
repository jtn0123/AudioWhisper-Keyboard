//! `suggest/core/suggest_options`: the options array `BinaryDictionary.getSuggestions` is given,
//! read by position. A position past the array's end reads as `false` or 0. `useFullEditDistance`,
//! `enableSpaceAwareGesture` and `getAdditionalFeaturesBoolOption` are not ported: nothing in the
//! C++ reads the first, and the other two are gesture-only.

/// `SuggestOptions::IS_GESTURE`.
const IS_GESTURE: i32 = 0;
/// `SuggestOptions::BLOCK_OFFENSIVE_WORDS`.
const BLOCK_OFFENSIVE_WORDS: i32 = 2;
/// `SuggestOptions::WEIGHT_FOR_LOCALE_IN_THOUSANDS`.
const WEIGHT_FOR_LOCALE_IN_THOUSANDS: i32 = 4;

/// `SuggestOptions`: a view of the options array.
#[derive(Clone, Copy, Debug)]
pub struct SuggestOptions<'a> {
    /// `mOptions` and `mLength`.
    options: &'a [i32],
}

impl<'a> SuggestOptions<'a> {
    /// `SuggestOptions(options, length)`.
    pub fn new(options: &'a [i32]) -> Self {
        Self { options }
    }

    /// `SuggestOptions::isGesture`.
    pub fn is_gesture(&self) -> bool {
        self.bool_option(IS_GESTURE)
    }

    /// `SuggestOptions::blockOffensiveWords`.
    pub fn block_offensive_words(&self) -> bool {
        self.bool_option(BLOCK_OFFENSIVE_WORDS)
    }

    /// `SuggestOptions::weightForLocale`: the option is in thousands.
    pub fn weight_for_locale(&self) -> f32 {
        self.int_option(WEIGHT_FOR_LOCALE_IN_THOUSANDS) as f32 / 1000.0
    }

    /// `SuggestOptions::getBoolOption`.
    fn bool_option(&self, key: i32) -> bool {
        self.option(key).is_some_and(|it| it != 0)
    }

    /// `SuggestOptions::getIntOption`.
    fn int_option(&self, key: i32) -> i32 {
        self.option(key).unwrap_or(0)
    }

    /// `options[key]` for a key inside the array (`isValidKey`).
    fn option(&self, key: i32) -> Option<i32> {
        usize::try_from(key)
            .ok()
            .and_then(|it| self.options.get(it).copied())
    }
}
