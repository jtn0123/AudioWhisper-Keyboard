//! Turns the decoder's results into the candidate strip and decides auto-correction, following
//! AOSP's `Suggest.getSuggestedWordsForNonBatchInput` and `AutoCorrectionUtils`, plus three narrow
//! rules of our own (see [`Suggester::is_allowed_to_auto_correct`]). Was Kotlin's
//! `LatinSuggester`, which still runs the decoder: this asks it through a [`Dictionary`].
//!
//! Upper- and lower-casing is Kotlin's with `Locale.US`, the only locale the keyboard suggests in.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::{Mutex, PoisonError};

use chars::starts_with_ignore_case;
use chars::text::code_point_count;
use chars::{equals_ignore_case, is_digit, is_letter, is_lower_case, is_upper_case, lowercase};

use crate::auto_correction::suggestion_exceeds_threshold;
use crate::dictionary::{Dictionary, Purpose};
use crate::learned_words::{KNOWN_AFTER, LearnedWords, NegativeCount};
use crate::personal_words::{MAX_IN_STRIP, PersonalWords};
use crate::previous_words::PreviousWords;
use crate::raw::{NOT_A_PROBABILITY, RawSuggestion};
use crate::word_case::{WordCase, decoder_input, is_mostly_caps};

/// How much more frequent (0-255 log scale) a suggestion must be to replace a valid word.
pub const RARER_BY: i32 = 50;

/// What surrounds the cursor when suggestions are asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypingState {
    /// The word being typed, left of the cursor; empty for next-word prediction.
    pub typed: Vec<u16>,
    pub previous_words: PreviousWords,
    /// The cursor sits inside a word (letters follow it); AOSP never auto-corrects then.
    pub is_cursor_mid_word: bool,
}

/// One entry of the candidate strip. At most one per strip is the auto-correction, the first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arranged {
    pub text: Vec<u16>,
    pub is_auto_correction: bool,
    /// The literal typed word, kept so the user can always tap it to keep what they typed.
    pub is_typed_word: bool,
}

impl Arranged {
    fn plain(text: Vec<u16>) -> Arranged {
        Arranged {
            text,
            is_auto_correction: false,
            is_typed_word: false,
        }
    }
}

/// A spell checker's verdict on one word.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpellCheck {
    pub is_valid: bool,
    pub suggestions: Vec<Vec<u16>>,
    pub is_high_confidence: bool,
}

impl SpellCheck {
    const VALID: SpellCheck = SpellCheck {
        is_valid: true,
        suggestions: Vec::new(),
        is_high_confidence: false,
    };
}

/// A decoder result and how it is shown, after matching the typed word's case.
#[derive(Debug)]
struct Ranked {
    raw: RawSuggestion,
    display: Vec<u16>,
}

/// What one request for suggestions draws on.
#[derive(Debug)]
pub struct Suggester<'a, D: ?Sized> {
    pub dictionary: &'a mut D,
    /// The user's personal dictionaries.
    pub personal: &'a PersonalWords,
    /// What the user has typed before, or `None` if the keyboard isn't learning. Locked for each
    /// question only, as the keyboard learns on another thread.
    pub learned: Option<&'a Mutex<LearnedWords>>,
    /// The auto-correction threshold (see [`crate::auto_correction`]).
    pub threshold: f32,
}

impl<D> Suggester<'_, D>
where
    D: Dictionary + ?Sized,
    D::Error: From<NegativeCount>,
{
    /// The candidate strip for `state`: at most `max_count` words, auto-correction first.
    pub fn suggest(
        &mut self,
        state: &TypingState,
        max_count: i32,
    ) -> Result<Vec<Arranged>, D::Error> {
        let Some(max_count) = usize::try_from(max_count).ok().filter(|&count| count > 0) else {
            return Ok(Vec::new());
        };
        if state.typed.is_empty() {
            return self.predictions(&state.previous_words, max_count);
        }
        let typed = &state.typed[..];
        let input = decoder_input(typed);
        let results = self.dictionary.decode(&input, Purpose::Typing)?;
        let ranked = self.rank(typed, results)?;
        let own_spelling = self.own_spelling_of(typed)?;
        let auto_correction = self.auto_correction_for(state, &input, &ranked, own_spelling)?;
        let case = WordCase::of(typed);
        let own_words = self.own_completions(typed)?;
        let own_words = own_words.iter().map(|word| case.apply(word));
        let corrections = ranked
            .into_iter()
            .filter(|it| it.display != typed)
            .collect();
        let corrections = self.acronym_after_completions(typed, corrections)?;
        let corrections = corrections.into_iter().map(|it| it.display);
        let mut strip = arrange(auto_correction, typed, own_words.chain(corrections));
        strip.truncate(max_count);
        Ok(strip)
    }

    /// The user's own spelling of `typed`: their personal dictionaries', else what they have
    /// typed, for a word the dictionary lacks.
    fn own_spelling_of(&mut self, typed: &[u16]) -> Result<Option<Vec<u16>>, D::Error> {
        match self.personal.spelling_of(typed) {
            Some(spelling) => Ok(Some(spelling.to_vec())),
            None => self.learned_spelling_of(typed),
        }
    }

    /// The user's own words that continue `typed`: personal dictionary entries first, then words
    /// they have used that the dictionary lacks (it already offers its own, ranked by frequency).
    fn own_completions(&mut self, typed: &[u16]) -> Result<Vec<Vec<u16>>, D::Error> {
        let mut words: Vec<Vec<u16>> = (self.personal.completions(typed, MAX_IN_STRIP).iter())
            .map(|word| word.to_vec())
            .collect();
        let ranked: Option<Vec<Vec<u16>>> = self.with_learned(|learned| {
            let completions = learned.completions(typed, MAX_IN_STRIP);
            completions.iter().map(|word| word.to_vec()).collect()
        });
        // The first MAX_IN_STRIP it accepts: it isn't asked about the rest.
        let max_count = MAX_IN_STRIP as usize;
        let mut accepted = 0;
        for word in ranked.unwrap_or_default() {
            if accepted == max_count {
                break;
            }
            if self.is_unknown(&word)? {
                words.push(word);
                accepted += 1;
            }
        }
        let mut words = distinct_by(words, |word| lowercase(word));
        words.truncate(max_count);
        Ok(words)
    }

    /// A learned spelling counts only for a word the dictionary lacks: it knows its own words
    /// better.
    fn learned_spelling_of(&mut self, typed: &[u16]) -> Result<Option<Vec<u16>>, D::Error> {
        let spelling = self.with_learned(|learned| learned.spelling_of(typed).map(<[u16]>::to_vec));
        match spelling.flatten() {
            Some(spelling) if self.is_unknown(&spelling)? => Ok(Some(spelling)),
            _ => Ok(None),
        }
    }

    fn is_unknown(&mut self, word: &[u16]) -> Result<bool, D::Error> {
        Ok(self.dictionary.probability(word)? == NOT_A_PROBABILITY
            && self.dictionary.probability(&lowercase(word))? == NOT_A_PROBABILITY)
    }

    /// What the learned words say, read under their lock; `None` if there are none.
    fn with_learned<R>(&self, read: impl FnOnce(&LearnedWords) -> R) -> Option<R> {
        // A panic aborts, so no lock is ever poisoned; take it back if one somehow were.
        (self.learned).map(|words| read(&words.lock().unwrap_or_else(PoisonError::into_inner)))
    }

    /// What `state`'s word is auto-corrected to, if anything. One of the user's own words is
    /// never corrected away; typed in another case ("iphone") and unknown to the dictionary, it
    /// takes the user's spelling ("iPhone").
    fn auto_correction_for(
        &mut self,
        state: &TypingState,
        input: &[u16],
        ranked: &[Ranked],
        own_spelling: Option<Vec<u16>>,
    ) -> Result<Option<Vec<u16>>, D::Error> {
        let typed = &state.typed[..];
        let is_typed_valid = self.is_valid(typed, input, ranked)?;
        if let Some(spelling) = own_spelling {
            let corrects = spelling != typed && !is_typed_valid && is_correctable(state);
            return Ok(corrects.then_some(spelling));
        }
        let Some(top) = ranked.iter().find(|it| it.display != typed) else {
            return Ok(None);
        };
        let corrects = self.should_auto_correct(state, input, top, is_typed_valid)?;
        Ok(corrects.then(|| top.display.clone()))
    }

    /// Spell-checks `word` with no context, suggesting at most `max_count` corrections; a
    /// negative `max_count` fails, as Kotlin's `take` throws, once it comes to suggesting.
    pub fn spell(&mut self, word: &[u16], max_count: i32) -> Result<SpellCheck, D::Error> {
        if self.personal.spelling_of(word).is_some() || self.learned_spelling_of(word)?.is_some() {
            return Ok(SpellCheck::VALID);
        }
        let input = decoder_input(word);
        let results = self.dictionary.decode(&input, Purpose::Spelling)?;
        let ranked = self.rank(word, results)?;
        if self.is_valid(word, &input, &ranked)? {
            return Ok(SpellCheck::VALID);
        }
        let corrections: Vec<Ranked> = ranked.into_iter().filter(|it| it.display != word).collect();
        let state = TypingState {
            typed: word.to_vec(),
            previous_words: PreviousWords::NONE,
            is_cursor_mid_word: false,
        };
        let is_high_confidence = match corrections.first() {
            Some(top) => self.should_auto_correct(&state, &input, top, false)?,
            None => false,
        };
        let max_count = usize::try_from(max_count).map_err(|_| NegativeCount(max_count))?;
        let suggestions = corrections.into_iter().take(max_count).map(|it| it.display);
        Ok(SpellCheck {
            is_valid: false,
            suggestions: suggestions.collect(),
            is_high_confidence,
        })
    }

    /// Next words after `previous`: those the user has typed there more than once, then the
    /// dictionary's (it knows at most three per word), then those typed there once.
    fn predictions(
        &mut self,
        previous: &PreviousWords,
        max_count: usize,
    ) -> Result<Vec<Arranged>, D::Error> {
        if previous.slot_count() == 0 {
            return Ok(Vec::new());
        }
        // There are no more words than an Int counts.
        let max = i32::try_from(max_count).unwrap_or(i32::MAX);
        let habits = self.followers(previous, KNOWN_AFTER, max)?;
        let dictionary = self.dictionary.decode(&[], Purpose::Predicting)?;
        let once = self.followers(previous, 1, max)?;
        let words = habits
            .into_iter()
            .chain(dictionary.into_iter().map(|it| it.word))
            .chain(once);
        let words = distinct_by(words, |word| lowercase(word));
        Ok(words
            .into_iter()
            .take(max_count)
            .map(Arranged::plain)
            .collect())
    }

    fn followers(
        &self,
        previous: &PreviousWords,
        min_count: i32,
        max_count: i32,
    ) -> Result<Vec<Vec<u16>>, NegativeCount> {
        let first_word = previous.words.first().map(Vec::as_slice);
        let followers = self.with_learned(|learned| {
            let words = learned.followers(
                first_word,
                previous.is_beginning_of_sentence,
                min_count,
                max_count,
            )?;
            Ok(words.iter().map(|word| word.to_vec()).collect())
        });
        followers.unwrap_or(Ok(Vec::new()))
    }

    /// Case-matched, de-duplicated results, best first.
    fn rank(
        &mut self,
        typed: &[u16],
        results: Vec<RawSuggestion>,
    ) -> Result<Vec<Ranked>, D::Error> {
        let case = WordCase::of(typed);
        let ranked = (results.into_iter())
            .filter(|raw| !raw.word.is_empty())
            .map(|raw| Ranked {
                display: case.apply(&raw.word),
                raw,
            });
        let ranked = distinct_by(ranked, |it| it.display.clone());
        // Capitalized or all-caps typing already gave every result the same case.
        if typed.iter().any(|&unit| is_upper_case(unit)) {
            Ok(ranked)
        } else {
            self.one_spelling_per_word(typed, ranked)
        }
    }

    /// For a word typed in lower case, where results keep the dictionary's spelling: no second
    /// spelling of a typed word that is itself a word ("jello" doesn't also offer "Jello"), and
    /// one spelling per word, lower case preferred ("jelo" offers "jello", not both).
    fn one_spelling_per_word(
        &mut self,
        typed: &[u16],
        ranked: Vec<Ranked>,
    ) -> Result<Vec<Ranked>, D::Error> {
        let typed_is_word = ranked.iter().any(|it| it.display == typed)
            || self.dictionary.probability(typed)? != NOT_A_PROBABILITY;
        let kept = ranked.into_iter().filter(|it| {
            it.display == typed || !(typed_is_word && equals_ignore_case(&it.display, typed))
        });
        // Grouped by the lower-cased spelling, groups in the order first seen.
        let mut groups: Vec<Vec<Ranked>> = Vec::new();
        let mut places: HashMap<Vec<u16>, usize> = HashMap::new();
        for it in kept {
            let place = *places.entry(lowercase(&it.display)).or_insert_with(|| {
                groups.push(Vec::new());
                groups.len() - 1
            });
            groups[place].push(it);
        }
        Ok(groups.into_iter().map(lower_case_first).collect())
    }

    /// Strip order only (auto-correction is decided first): an acronym spelled with exactly the
    /// typed letters goes after the more frequent words that continue them, so "wor" shows "work"
    /// and "word" before "WOR". With nothing like that ahead of it, it stays ("ok" still offers
    /// "OK").
    fn acronym_after_completions(
        &mut self,
        typed: &[u16],
        mut corrections: Vec<Ranked>,
    ) -> Result<Vec<Ranked>, D::Error> {
        let Some(acronym) = corrections
            .iter()
            .position(|it| equals_ignore_case(&it.display, typed) && is_acronym(&it.display))
        else {
            return Ok(corrections);
        };
        let frequency = self
            .dictionary
            .probability(&corrections[acronym].raw.word)?;
        // The last of the others that is ahead of it, looking from the end, as Kotlin does.
        let mut last_ahead = None;
        for (index, it) in corrections.iter().enumerate().rev() {
            if index != acronym
                && starts_with_ignore_case(&it.display, typed)
                && self.dictionary.probability(&it.raw.word)? > frequency
            {
                last_ahead = Some(index);
                break;
            }
        }
        // Already behind all of them (or nothing to go behind): leave the order alone.
        if let Some(last_ahead) = last_ahead.filter(|&index| index > acronym) {
            let moved = corrections.remove(acronym);
            corrections.insert(last_ahead, moved);
        }
        Ok(corrections)
    }

    /// The typed word is a dictionary word (in the case the user typed, or with the first letter
    /// lowered).
    fn is_valid(
        &mut self,
        typed: &[u16],
        input: &[u16],
        ranked: &[Ranked],
    ) -> Result<bool, D::Error> {
        Ok(ranked.iter().any(|it| it.display == typed)
            || self.dictionary.probability(typed)? != NOT_A_PROBABILITY
            || (input != typed && self.dictionary.probability(input)? != NOT_A_PROBABILITY))
    }

    /// AOSP's gates (no correction mid-word, with digits, in mostly-caps words, to predictions or
    /// shortcuts).
    fn should_auto_correct(
        &mut self,
        state: &TypingState,
        input: &[u16],
        top: &Ranked,
        is_typed_valid: bool,
    ) -> Result<bool, D::Error> {
        if !is_correctable(state) {
            return Ok(false);
        }
        if top.raw.is_prediction() || top.raw.is_shortcut() {
            return Ok(false);
        }
        Ok(
            self.is_allowed_to_auto_correct(state, input, top, is_typed_valid)?
                && suggestion_exceeds_threshold(&top.raw, input, self.threshold),
        )
    }

    /// AOSP's `allowsToBeAutoCorrected`: a whitelisted top word, or a typed word of two or more
    /// letters that is not in the dictionary. Our additions: a lower-case word is never
    /// upper-cased into an acronym ("ok" stays, not "OK"), and a valid word is still corrected
    /// when it [`Self::is_much_rarer`] than the suggestion in this context.
    fn is_allowed_to_auto_correct(
        &mut self,
        state: &TypingState,
        input: &[u16],
        top: &Ranked,
        is_typed_valid: bool,
    ) -> Result<bool, D::Error> {
        if top.raw.is_whitelist() {
            return Ok(true);
        }
        let typed = &state.typed[..];
        if code_point_count(typed) <= 1 {
            return Ok(false);
        }
        if is_case_only_acronym(typed, &top.display) {
            return Ok(false);
        }
        Ok(!is_typed_valid || self.is_much_rarer(input, &top.raw, &state.previous_words)?)
    }

    /// A valid word that is a likely slip: the dictionary knows the suggestion after the previous
    /// word but not the typed word, and the suggestion is at least [`RARER_BY`] steps more
    /// frequent on the 0-255 log scale ("came fro" -> "came from", "a lit" -> "a lot").
    fn is_much_rarer(
        &mut self,
        input: &[u16],
        suggestion: &RawSuggestion,
        previous: &PreviousWords,
    ) -> Result<bool, D::Error> {
        if previous.words.is_empty() {
            return Ok(false);
        }
        if self.dictionary.ngram_probability(input)? != NOT_A_PROBABILITY {
            return Ok(false);
        }
        if self.dictionary.ngram_probability(&suggestion.word)? == NOT_A_PROBABILITY {
            return Ok(false);
        }
        let typed_probability = self.dictionary.probability(input)?.max(0);
        // An Int's subtraction, which wraps.
        let rise = (self.dictionary.probability(&suggestion.word)?).wrapping_sub(typed_probability);
        Ok(rise >= RARER_BY)
    }
}

/// The strip: the auto-correction, the typed word, then the `others`, each text once.
fn arrange(
    auto_correction: Option<Vec<u16>>,
    typed: &[u16],
    others: impl Iterator<Item = Vec<u16>>,
) -> Vec<Arranged> {
    let auto_correction = auto_correction.map(|text| Arranged {
        text,
        is_auto_correction: true,
        is_typed_word: false,
    });
    let typed = Arranged {
        text: typed.to_vec(),
        is_auto_correction: false,
        is_typed_word: true,
    };
    let strip = auto_correction
        .into_iter()
        .chain([typed])
        .chain(others.map(Arranged::plain));
    distinct_by(strip, |it| it.text.clone())
}

/// Of one word's spellings, the first with no capital, else the first.
fn lower_case_first(mut spellings: Vec<Ranked>) -> Ranked {
    let has_no_capital = |it: &Ranked| !it.display.iter().any(|&unit| is_upper_case(unit));
    let index = spellings.iter().position(has_no_capital).unwrap_or(0);
    spellings.swap_remove(index)
}

fn is_acronym(word: &[u16]) -> bool {
    let letters = word.iter().filter(|&&unit| is_letter(unit));
    letters.clone().count() > 1 && !letters.clone().any(|&unit| is_lower_case(unit))
}

/// AOSP never auto-corrects mid-word, a word with digits, or a mostly-caps word.
fn is_correctable(state: &TypingState) -> bool {
    !state.is_cursor_mid_word
        && !state.typed.iter().any(|&unit| is_digit(unit))
        && !is_mostly_caps(&state.typed)
}

fn is_case_only_acronym(typed: &[u16], display: &[u16]) -> bool {
    equals_ignore_case(display, typed)
        && !display.iter().any(|&unit| is_lower_case(unit))
        && typed.iter().any(|&unit| is_lower_case(unit))
}

/// `items` without those whose `key` an earlier one had, as Kotlin's `distinctBy`.
fn distinct_by<T, K: Hash + Eq>(
    items: impl IntoIterator<Item = T>,
    key: impl Fn(&T) -> K,
) -> Vec<T> {
    let mut seen = HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(key(item)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::Lookup;
    use crate::personal_words::Entry;
    use crate::raw::{FLAG_APPROPRIATE_FOR_AUTO_CORRECTION, FLAG_EXACT_MATCH, KIND_CORRECTION};
    use crate::raw::{KIND_PREDICTION, KIND_SHORTCUT, KIND_WHITELIST};

    const CORRECTION: i32 = KIND_CORRECTION | FLAG_APPROPRIATE_FOR_AUTO_CORRECTION;
    const EXACT: i32 = CORRECTION | FLAG_EXACT_MATCH;
    const WHITELIST_SCORE: i32 = i32::MAX - 1;

    fn utf16(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn texts(words: &[Vec<u16>]) -> Vec<String> {
        words
            .iter()
            .map(|word| String::from_utf16_lossy(word))
            .collect()
    }

    fn raw(word: &str, score: i32, kind_and_flags: i32) -> RawSuggestion {
        RawSuggestion {
            word: utf16(word),
            score,
            kind_and_flags,
        }
    }

    /// The real engine's results, scores and probabilities for these words with the bundled
    /// en_US dictionary, as the Kotlin suggester's tests have them.
    struct Fake {
        previous: PreviousWords,
        /// What each decode was for.
        purposes: Vec<Purpose>,
    }

    /// A result: word, score, kind and flags.
    type Decoded = (&'static str, i32, i32);

    /// Each input's results.
    const RESULTS: [(&str, &[Decoded]); 16] = [
        (
            "teh",
            &[("the", 477_893, CORRECTION), ("ten", 400_787, CORRECTION)],
        ),
        ("teh2", &[("the", 477_893, CORRECTION)]),
        (
            "im",
            &[
                ("I'm", WHITELIST_SCORE, KIND_WHITELIST),
                ("in", 571_761, CORRECTION),
            ],
        ),
        (
            "form",
            &[("form", 1_819_619, EXACT), ("from", 422_604, CORRECTION)],
        ),
        (
            "fro",
            &[("fro", 1_536_913, EXACT), ("from", 471_045, CORRECTION)],
        ),
        (
            "ok",
            &[("OK", 1_740_557, EXACT), ("on", 521_841, CORRECTION)],
        ),
        ("london", &[("London", 1_801_319, EXACT)]),
        (
            "yeah",
            &[("yeah", 1_753_136, EXACT), ("year", 235_587, CORRECTION)],
        ),
        ("lol", &[("oil", 227_445, CORRECTION)]),
        (
            "wor",
            &[
                ("WOR", 1_600_000, EXACT),
                ("work", 600_000, CORRECTION),
                ("word", 580_000, CORRECTION),
                ("worn", 570_000, CORRECTION),
            ],
        ),
        (
            "jello",
            &[
                ("jello", 1_500_000, EXACT),
                ("Jello", 1_400_000, CORRECTION),
            ],
        ),
        (
            "jelo",
            &[
                ("Jello", 600_000, CORRECTION),
                ("jello", 590_000, CORRECTION),
            ],
        ),
        ("u", &[("I", 484_164, CORRECTION)]),
        (
            "nit",
            &[
                ("not", WHITELIST_SCORE, KIND_WHITELIST),
                ("nit", 1_493_598, EXACT),
                ("not", 505_979, CORRECTION),
            ],
        ),
        (
            "",
            &[
                ("you", 90, KIND_PREDICTION),
                ("God", 80, KIND_PREDICTION),
                ("you", 70, KIND_PREDICTION),
            ],
        ),
        (
            "shrt",
            &[(
                "short",
                900_000,
                KIND_SHORTCUT | FLAG_APPROPRIATE_FOR_AUTO_CORRECTION,
            )],
        ),
    ];

    fn results(input: &str) -> Vec<RawSuggestion> {
        let known = RESULTS.iter().find(|&&(known, _)| known == input);
        let results = known.map_or(&[][..], |&(_, results)| results);
        (results.iter())
            .map(|&(word, score, kind_and_flags)| raw(word, score, kind_and_flags))
            .collect()
    }

    const PROBABILITIES: [(&str, i32); 12] = [
        ("jello", 120),
        ("WOR", 79),
        ("work", 200),
        ("word", 190),
        ("worn", 60),
        ("OK", 150),
        ("on", 220),
        ("form", 158),
        ("from", 191),
        ("fro", 73),
        ("yeah", 135),
        ("nit", 59),
    ];

    impl Lookup for Fake {
        type Error = NegativeCount;

        fn probability(&mut self, word: &[u16]) -> Result<i32, NegativeCount> {
            let word = String::from_utf16_lossy(word);
            let known = PROBABILITIES.iter().find(|(known, _)| *known == word);
            Ok(known.map_or(NOT_A_PROBABILITY, |&(_, probability)| probability))
        }
    }

    impl Dictionary for Fake {
        fn ngram_probability(&mut self, word: &[u16]) -> Result<i32, NegativeCount> {
            let previous = self
                .previous
                .words
                .first()
                .map(|word| String::from_utf16_lossy(word));
            let known = previous.as_deref() == Some("came") && word == utf16("from");
            Ok(if known { 194 } else { NOT_A_PROBABILITY })
        }

        fn decode(
            &mut self,
            input: &[u16],
            purpose: Purpose,
        ) -> Result<Vec<RawSuggestion>, NegativeCount> {
            self.purposes.push(purpose);
            let mut results = results(&String::from_utf16_lossy(input));
            results.sort_by_key(|it| std::cmp::Reverse(it.score));
            Ok(results)
        }
    }

    struct Words {
        personal: PersonalWords,
        learned: Option<Mutex<LearnedWords>>,
    }

    impl Words {
        fn none() -> Words {
            Words {
                personal: PersonalWords::new(Vec::new()),
                learned: None,
            }
        }

        fn personal(words: &[(&str, i32)]) -> Words {
            let entries = words.iter().map(|&(word, frequency)| Entry {
                word: utf16(word),
                frequency,
                shortcut: None,
            });
            Words {
                personal: PersonalWords::new(entries.collect()),
                learned: None,
            }
        }

        fn learned(words: &[(&str, Option<&str>, i32)]) -> Words {
            let mut learned = LearnedWords::new(100, 100);
            for &(word, previous, times) in words {
                let previous = previous.map(utf16);
                for _ in 0..times {
                    learned.add(&utf16(word), previous.as_deref(), false);
                }
            }
            Words {
                personal: PersonalWords::new(Vec::new()),
                learned: Some(Mutex::new(learned)),
            }
        }

        fn suggester<'a>(&'a self, fake: &'a mut Fake) -> Suggester<'a, Fake> {
            Suggester {
                dictionary: fake,
                personal: &self.personal,
                learned: self.learned.as_ref(),
                threshold: crate::auto_correction::MODEST_THRESHOLD,
            }
        }

        /// The strip for `typed` between `before` and `after`, and what each decode was for.
        fn strip(
            &self,
            before: &str,
            typed: &str,
            after: &str,
            max: i32,
        ) -> (Vec<Arranged>, Vec<Purpose>) {
            let previous_words = PreviousWords::from(&utf16(before));
            let state = TypingState {
                typed: utf16(typed),
                previous_words: previous_words.clone(),
                is_cursor_mid_word: utf16(after)
                    .first()
                    .is_some_and(|&unit| chars::is_letter_or_digit(unit)),
            };
            let mut fake = Fake {
                previous: previous_words,
                purposes: Vec::new(),
            };
            let strip = self.suggester(&mut fake).suggest(&state, max).unwrap();
            (strip, fake.purposes)
        }

        fn words(&self, before: &str, typed: &str, after: &str) -> Vec<String> {
            let (strip, _) = self.strip(before, typed, after, 8);
            strip
                .into_iter()
                .map(|it| String::from_utf16_lossy(&it.text))
                .collect()
        }

        fn auto_correction(&self, before: &str, typed: &str, after: &str) -> Option<String> {
            let (strip, _) = self.strip(before, typed, after, 8);
            let correction = strip.into_iter().find(|it| it.is_auto_correction);
            correction.map(|it| String::from_utf16_lossy(&it.text))
        }

        fn spell(&self, word: &str, max: i32) -> Result<(bool, Vec<String>, bool), NegativeCount> {
            let mut fake = Fake {
                previous: PreviousWords::NONE,
                purposes: Vec::new(),
            };
            let check = self.suggester(&mut fake).spell(&utf16(word), max)?;
            assert!(
                fake.purposes
                    .iter()
                    .all(|&purpose| purpose == Purpose::Spelling)
            );
            Ok((
                check.is_valid,
                texts(&check.suggestions),
                check.is_high_confidence,
            ))
        }
    }

    #[test]
    fn corrects_a_typo_and_keeps_the_typed_word_tappable() {
        let (strip, purposes) = Words::none().strip("", "teh", "", 8);
        assert_eq!(
            texts(&strip.iter().map(|it| it.text.clone()).collect::<Vec<_>>()),
            ["the", "teh", "ten"]
        );
        let flags: Vec<(bool, bool)> = strip
            .iter()
            .map(|it| (it.is_auto_correction, it.is_typed_word))
            .collect();
        assert_eq!(flags, [(true, false), (false, true), (false, false)]);
        assert_eq!(purposes, [Purpose::Typing]);
    }

    #[test]
    fn matches_the_case_of_the_typed_word() {
        let none = Words::none();
        assert_eq!(none.auto_correction("", "Teh", "").as_deref(), Some("The"));
        assert_eq!(none.words("", "TEH", ""), ["TEH", "THE", "TEN"]);
        // Mostly caps is never corrected.
        assert_eq!(none.auto_correction("", "TEH", ""), None);
        assert_eq!(none.auto_correction("", "TeH", ""), None);
    }

    #[test]
    fn the_whitelist_corrects_even_capitals_and_valid_words() {
        let none = Words::none();
        assert_eq!(none.auto_correction("", "im", "").as_deref(), Some("I'm"));
        assert_eq!(none.auto_correction("", "Im", "").as_deref(), Some("I'm"));
        assert_eq!(none.auto_correction("", "nit", "").as_deref(), Some("not"));
        assert_eq!(none.words("", "nit", ""), ["not", "nit"]);
        // Even a single letter, which nothing else corrects.
        assert_eq!(none.auto_correction("", "u", ""), None);
    }

    #[test]
    fn leaves_valid_words_alone_unless_much_rarer_in_context() {
        let none = Words::none();
        assert_eq!(none.words("", "form", ""), ["form", "from"]);
        assert_eq!(none.auto_correction("", "form", ""), None);
        assert_eq!(
            none.auto_correction("came ", "fro", "").as_deref(),
            Some("from")
        );
        assert_eq!(none.auto_correction("and ", "fro", ""), None);
        assert_eq!(none.auto_correction("", "fro", ""), None);
    }

    #[test]
    fn capitalizes_names_but_makes_no_acronyms() {
        let none = Words::none();
        assert_eq!(
            none.auto_correction("", "london", "").as_deref(),
            Some("London")
        );
        assert_eq!(none.auto_correction("", "ok", ""), None);
        assert_eq!(none.words("", "ok", ""), ["ok", "OK", "on"]);
    }

    #[test]
    fn offers_each_word_once_in_one_spelling() {
        let none = Words::none();
        assert_eq!(none.words("", "jello", ""), ["jello"]);
        assert_eq!(none.words("", "jelo", ""), ["jello", "jelo"]);
        // Typed with a capital, every result has one: both spellings are one.
        assert_eq!(none.words("", "Jelo", ""), ["Jello", "Jelo"]);
    }

    #[test]
    fn an_acronym_of_the_typed_letters_goes_after_the_words_being_typed() {
        let none = Words::none();
        assert_eq!(
            none.words("", "wor", ""),
            ["wor", "work", "word", "WOR", "worn"]
        );
        assert_eq!(none.auto_correction("", "wor", ""), None);
    }

    #[test]
    fn offers_below_the_threshold_without_correcting() {
        let none = Words::none();
        assert_eq!(none.auto_correction("", "yeah", ""), None);
        assert_eq!(none.words("", "lol", ""), ["lol", "oil"]);
        assert_eq!(none.auto_correction("", "lol", ""), None);
        // Nor to a shortcut, however good.
        assert_eq!(none.words("", "shrt", ""), ["shrt", "short"]);
    }

    #[test]
    fn never_corrects_with_digits_or_inside_a_word() {
        let none = Words::none();
        assert_eq!(none.auto_correction("", "teh2", ""), None);
        assert_eq!(none.auto_correction("", "teh", "re"), None);
        assert_eq!(
            none.auto_correction("", "teh", " and").as_deref(),
            Some("the")
        );
    }

    #[test]
    fn predicts_the_next_word_without_correcting() {
        let none = Words::none();
        let (strip, purposes) = none.strip("thank ", "", "", 8);
        assert!(
            strip
                .iter()
                .all(|it| !it.is_auto_correction && !it.is_typed_word)
        );
        assert_eq!(
            texts(&strip.into_iter().map(|it| it.text).collect::<Vec<_>>()),
            ["you", "God"]
        );
        assert_eq!(purposes, [Purpose::Predicting]);
        let (strip, purposes) = none.strip("Hello, ", "", "", 8);
        assert_eq!((strip, purposes), (Vec::new(), Vec::new()));
    }

    #[test]
    fn keeps_to_the_limit() {
        let none = Words::none();
        let first = |typed: &str, max: i32| {
            let (strip, _) = none.strip("", typed, "", max);
            texts(&strip.into_iter().map(|it| it.text).collect::<Vec<_>>())
        };
        assert_eq!(first("teh", 1), ["the"]);
        assert_eq!(first("form", 1), ["form"]);
        assert_eq!(first("teh", 2), ["the", "teh"]);
        assert!(first("teh", 0).is_empty());
        assert!(first("teh", -1).is_empty());
        assert_eq!(none.strip("thank ", "", "", 1).0.len(), 1);
    }

    #[test]
    fn spell_checks_with_confidence() {
        let none = Words::none();
        assert_eq!(none.spell("form", 3), Ok((true, vec![], false)));
        assert_eq!(
            none.spell("teh", 3),
            Ok((false, vec!["the".into(), "ten".into()], true))
        );
        assert_eq!(none.spell("teh", 1), Ok((false, vec!["the".into()], true)));
        assert_eq!(none.spell("lol", 3), Ok((false, vec!["oil".into()], false)));
        assert_eq!(none.spell("zzz", 3), Ok((false, vec![], false)));
        // Kotlin's take(-1) throws, but only once it comes to suggesting.
        assert_eq!(none.spell("teh", -1), Err(NegativeCount(-1)));
        assert_eq!(none.spell("form", -1), Ok((true, vec![], false)));
    }

    #[test]
    fn never_corrects_a_personal_word_away() {
        let personal = Words::personal(&[("teh", 200), ("iPhone", 250), ("formica", 120)]);
        assert_eq!(personal.words("", "teh", ""), ["teh", "the", "ten"]);
        assert_eq!(personal.auto_correction("", "teh", ""), None);
        assert_eq!(personal.spell("teh", 3), Ok((true, vec![], false)));
        // Typed in another case, it takes the user's spelling.
        assert_eq!(
            personal.auto_correction("", "iphone", "").as_deref(),
            Some("iPhone")
        );
        assert_eq!(personal.auto_correction("", "iphone", "s"), None);
        assert_eq!(personal.words("", "form", ""), ["form", "formica", "from"]);
        assert_eq!(personal.words("", "Form", ""), ["Form", "Formica", "From"]);
    }

    #[test]
    fn offers_at_most_two_of_the_users_words() {
        let personal = Words::personal(&[("formica", 120), ("formal", 130), ("format", 140)]);
        assert_eq!(
            personal.words("", "form", ""),
            ["form", "format", "formal", "from"]
        );
        let learned = Words::learned(&[("worda", None, 2), ("wordb", None, 3), ("wordc", None, 4)]);
        assert_eq!(learned.words("", "wo", ""), ["wo", "wordc", "wordb"]);
    }

    #[test]
    fn offers_used_words_the_dictionary_lacks() {
        let learned = Words::learned(&[
            ("wordle", None, 2),
            ("word", None, 2),
            ("wormhole", None, 1),
        ]);
        assert_eq!(learned.words("", "wo", ""), ["wo", "wordle"]);
    }

    #[test]
    fn never_corrects_a_word_the_user_keeps_typing() {
        let twice = Words::learned(&[("teh", None, 2)]);
        assert_eq!(twice.auto_correction("", "teh", ""), None);
        assert_eq!(twice.spell("teh", 3), Ok((true, vec![], false)));
        let once = Words::learned(&[("teh", None, 1)]);
        assert_eq!(once.auto_correction("", "teh", "").as_deref(), Some("the"));
        // The dictionary knows its own words better than what was typed: "WOR" would be taken.
        let known = Words::learned(&[("WOR", None, 2)]);
        assert_eq!(known.auto_correction("", "wor", ""), None);
        let unknown = Words::learned(&[("Teh", None, 2)]);
        assert_eq!(
            unknown.auto_correction("", "teh", "").as_deref(),
            Some("Teh")
        );
    }

    #[test]
    fn predicts_the_users_habits_first() {
        let learned = Words::learned(&[
            ("later", Some("you"), 2),
            ("there", Some("you"), 1),
            ("You", Some("thank"), 1),
        ]);
        assert_eq!(
            learned.words("see you ", "", ""),
            ["later", "you", "God", "there"]
        );
        assert_eq!(learned.words("thank ", "", ""), ["you", "God"]);
    }
}
