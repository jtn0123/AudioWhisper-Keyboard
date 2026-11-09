//! `AdditionalProximityChars` (`suggest/core/layout/additional_proximity_chars.{h,cpp}`): letters
//! the language counts as near a vowel whatever the keyboard says. Only English has any.

/// `LOCALE_EN_US`: the two code points every English locale starts with.
const LOCALE_EN: [i32; 2] = ['e' as i32, 'n' as i32];
/// `EN_US_ADDITIONAL_A` and the others: the other four vowels.
const ADDITIONAL_A: [i32; 4] = ['e' as i32, 'i' as i32, 'o' as i32, 'u' as i32];
const ADDITIONAL_E: [i32; 4] = ['a' as i32, 'i' as i32, 'o' as i32, 'u' as i32];
const ADDITIONAL_I: [i32; 4] = ['a' as i32, 'e' as i32, 'o' as i32, 'u' as i32];
const ADDITIONAL_O: [i32; 4] = ['a' as i32, 'e' as i32, 'i' as i32, 'u' as i32];
const ADDITIONAL_U: [i32; 4] = ['a' as i32, 'e' as i32, 'i' as i32, 'o' as i32];

/// `AdditionalProximityChars::isEnLocale`: the locale's first two code points are `en`.
fn is_en_locale(locale: &[i32]) -> bool {
    locale.starts_with(&LOCALE_EN)
}

/// `AdditionalProximityChars::getAdditionalChars` (and `getAdditionalCharsSize`, its length): the
/// letters added as near to `c`, empty unless the locale is English and `c` a lower case vowel.
pub fn get_additional_chars(locale: &[i32], c: i32) -> &'static [i32] {
    if !is_en_locale(locale) {
        return &[];
    }
    match u8::try_from(c) {
        Ok(b'a') => &ADDITIONAL_A,
        Ok(b'e') => &ADDITIONAL_E,
        Ok(b'i') => &ADDITIONAL_I,
        Ok(b'o') => &ADDITIONAL_O,
        Ok(b'u') => &ADDITIONAL_U,
        _ => &[],
    }
}
