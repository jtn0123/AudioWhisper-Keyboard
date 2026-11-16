//! Turns the decoder's results into the candidate strip and decides auto-correction, following
//! AOSP's `Suggest.getSuggestedWordsForNonBatchInput` and `AutoCorrectionUtils`, plus three narrow
//! rules of our own (see [`Suggester::is_allowed_to_auto_correct`]). Was Kotlin's
//! `LatinSuggester`; the production [`Dictionary`] is an in-process native session.
//!
//! Upper- and lower-casing is Kotlin's with `Locale.US`, the only locale the keyboard suggests in.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use chars::starts_with_ignore_case;
use chars::text::code_point_count;
use chars::{equals_ignore_case, is_upper_case, lowercase};

use crate::auto_correction::suggestion_exceeds_threshold;
use crate::dictionary::{Dictionary, Purpose};
use crate::learned_words::{KNOWN_AFTER, LearnedWords, NegativeCount};
use crate::personal_words::{MAX_IN_STRIP, PersonalWords};
use crate::previous_words::PreviousWords;
use crate::raw::{NOT_A_PROBABILITY, RawSuggestion};
use crate::word_case::{WordCase, decoder_input};

mod strip;
use strip::{
    arrange, distinct_by, is_acronym, is_case_only_acronym, is_correctable, lower_case_first,
};

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
    /// The auto-correction threshold (see [`crate::auto_correction`]), or `None` to never
    /// auto-correct (the setting's [`Off`](crate::auto_correction::Strength::Off)).
    pub threshold: Option<f32>,
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
        if self.threshold.is_none() {
            return Ok(None);
        }
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
        let Some(threshold) = self.threshold else {
            return Ok(false);
        };
        Ok(
            self.is_allowed_to_auto_correct(state, input, top, is_typed_valid)?
                && suggestion_exceeds_threshold(&top.raw, input, threshold),
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
        Ok(!is_typed_valid || self.is_much_rarer(typed, input, &top.raw, &state.previous_words)?)
    }

    /// A valid word that is a likely slip: the dictionary knows the suggestion after the previous
    /// word but not the typed word, and the suggestion is at least [`RARER_BY`] steps more
    /// frequent on the 0-255 log scale ("came fro" -> "came from", "a lit" -> "a lot"). The typed
    /// word is asked about as the dictionary holds it: the decoder's lower-cased `input`, or as
    /// typed when only that is a word, so a name typed right ("South Korea") is not taken for an
    /// unknown word and corrected to another ("South Korean").
    fn is_much_rarer(
        &mut self,
        typed: &[u16],
        input: &[u16],
        suggestion: &RawSuggestion,
        previous: &PreviousWords,
    ) -> Result<bool, D::Error> {
        if previous.words.is_empty() {
            return Ok(false);
        }
        let word = if input != typed && self.dictionary.probability(input)? == NOT_A_PROBABILITY {
            typed
        } else {
            input
        };
        if self.dictionary.ngram_probability(word)? != NOT_A_PROBABILITY {
            return Ok(false);
        }
        if self.dictionary.ngram_probability(&suggestion.word)? == NOT_A_PROBABILITY {
            return Ok(false);
        }
        let typed_probability = self.dictionary.probability(word)?.max(0);
        // An Int's subtraction, which wraps.
        let rise = (self.dictionary.probability(&suggestion.word)?).wrapping_sub(typed_probability);
        Ok(rise >= RARER_BY)
    }
}

#[cfg(test)]
mod tests;
