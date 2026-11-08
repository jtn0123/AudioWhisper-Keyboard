//! How the user capitalized the word being typed, and how suggestions are made to match it (the
//! same three cases as AOSP's `Suggest.getTransformedSuggestedWordInfo`). Was Kotlin's `WordCase`.
//!
//! Upper- and lower-casing is Kotlin's with `Locale.US`, the only locale the keyboard suggests in.

use chars::text::first_code_point_len;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WordCase {
    /// Lower case, or mixed like "iPhone": suggestions keep the dictionary's spelling.
    AsTyped,
    /// Only the first letter is upper case (often auto-capitalization): capitalize suggestions.
    Capitalized,
    /// Two or more letters, all upper case: upper-case suggestions.
    AllCaps,
}

impl WordCase {
    /// The case of `typed`.
    pub fn of(typed: &[u16]) -> WordCase {
        let letters = typed.iter().filter(|&&unit| chars::is_letter(unit));
        let (count, upper) = letters.fold((0, 0), |(count, upper), &unit| {
            (count + 1, upper + usize::from(chars::is_upper_case(unit)))
        });
        if count > 1 && upper == count {
            WordCase::AllCaps
        } else if upper == 1
            && typed
                .first()
                .is_some_and(|&unit| chars::is_upper_case(unit))
        {
            WordCase::Capitalized
        } else {
            WordCase::AsTyped
        }
    }

    /// `word` in this case.
    pub fn apply(self, word: &[u16]) -> Vec<u16> {
        match self {
            WordCase::AsTyped => word.to_vec(),
            WordCase::Capitalized => map_first(word, chars::uppercase),
            WordCase::AllCaps => chars::uppercase(word),
        }
    }
}

/// AOSP's `WordComposer.isMostlyCaps`: more than one capital letter; never auto-corrected.
pub fn is_mostly_caps(typed: &[u16]) -> bool {
    typed
        .iter()
        .filter(|&&unit| chars::is_upper_case(unit))
        .count()
        > 1
}

/// The form handed to the decoder. A leading capital is usually auto-capitalization, so "Im" at
/// the start of a sentence is looked up as "im" (whitelisted to "I'm"), and [`WordCase::apply`]
/// puts the capital back on the results.
pub fn decoder_input(typed: &[u16]) -> Vec<u16> {
    match WordCase::of(typed) {
        WordCase::AsTyped => typed.to_vec(),
        WordCase::Capitalized => map_first(typed, chars::lowercase),
        WordCase::AllCaps => chars::lowercase(typed),
    }
}

/// `word` with its first code point, on its own, put through `map`.
fn map_first(word: &[u16], map: fn(&[u16]) -> Vec<u16>) -> Vec<u16> {
    let end = first_code_point_len(word);
    let mut mapped = map(&word[..end]);
    mapped.extend_from_slice(&word[end..]);
    mapped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn string(units: Vec<u16>) -> String {
        String::from_utf16(&units).unwrap()
    }

    #[test]
    fn tells_the_case_apart() {
        for (typed, case) in [
            ("hello", WordCase::AsTyped),
            ("iPhone", WordCase::AsTyped),
            ("Hello", WordCase::Capitalized),
            ("I", WordCase::Capitalized),
            ("HELLO", WordCase::AllCaps),
            ("I'M", WordCase::AllCaps),
            ("HeLLo", WordCase::AsTyped),
            ("", WordCase::AsTyped),
            ("'Hi", WordCase::AsTyped),
        ] {
            assert_eq!(WordCase::of(&utf16(typed)), case, "{typed}");
        }
    }

    #[test]
    fn applies_the_case() {
        assert_eq!(
            string(WordCase::Capitalized.apply(&utf16("straße"))),
            "Straße"
        );
        assert_eq!(string(WordCase::Capitalized.apply(&utf16("ßa"))), "SSa");
        assert_eq!(string(WordCase::AllCaps.apply(&utf16("straße"))), "STRASSE");
        assert_eq!(string(WordCase::AsTyped.apply(&utf16("iPhone"))), "iPhone");
        assert_eq!(string(WordCase::Capitalized.apply(&utf16("𐐨x"))), "𐐀x");
        assert!(WordCase::Capitalized.apply(&[]).is_empty());
    }

    #[test]
    fn lower_cases_for_the_decoder() {
        assert_eq!(string(decoder_input(&utf16("Im"))), "im");
        assert_eq!(string(decoder_input(&utf16("HELLO"))), "hello");
        assert_eq!(string(decoder_input(&utf16("iPhone"))), "iPhone");
        assert_eq!(string(decoder_input(&utf16("İf"))), "i\u{307}f");
    }

    #[test]
    fn mostly_caps_is_two_capitals() {
        assert!(is_mostly_caps(&utf16("NASA")));
        assert!(is_mostly_caps(&utf16("McD")));
        assert!(!is_mostly_caps(&utf16("Hello")));
    }

    #[test]
    fn a_capitalized_word_lower_cases_only_its_first_letter() {
        // Ⅻ is no letter, so "AⅫ" counts as capitalized; its own case stays as typed.
        assert_eq!(string(decoder_input(&utf16("AⅫ"))), "aⅫ");
    }
}
