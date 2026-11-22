//! The production typing engine. Owns the dictionary, spatial model, reusable search cache and
//! output slots; requests call `latin` directly, without a JVM callback. Ports LatinDictionaryEngine.

use std::sync::Arc;

use latin::defines::{MAX_RESULTS, MAX_WORD_LENGTH, NOT_A_PROBABILITY};
use latin::layout::{KeyboardGeometry, ProximityInfo};
use latin::search::{DicTraverseSession, Output, Query, suggest_into};
use latin::trie::{dictionary::Dictionary as LatinDictionary, jni::get_words, ngram::NgramContext};

use crate::dictionary::{Dictionary, Lookup, Purpose};
use crate::java::code_points;
use crate::language_model::{LanguageModel, Tuning};
use crate::learned_words::NegativeCount;
use crate::previous_words::PreviousWords;
use crate::qwerty::{self, CannotRoundNaN};
use crate::raw::RawSuggestion;
use crate::tap::TapOffset;

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    CannotRoundNaN,
    NegativeCount(i32),
}
impl From<NegativeCount> for Error {
    fn from(NegativeCount(count): NegativeCount) -> Self {
        Self::NegativeCount(count)
    }
}
impl From<CannotRoundNaN> for Error {
    fn from(_: CannotRoundNaN) -> Self {
        Self::CannotRoundNaN
    }
}

#[derive(Debug)]
pub struct Session {
    dictionary: LatinDictionary,
    traversal: DicTraverseSession,
    proximity: ProximityInfo,
    output: Output,
    model: Option<LanguageModel>,
}

impl Session {
    pub fn open(bytes: Arc<[u8]>) -> Option<Self> {
        let traversal = DicTraverseSession::new(bytes.len());
        Some(Self {
            dictionary: LatinDictionary::open(bytes)?,
            traversal,
            proximity: proximity()?,
            output: Output::new(0.0),
            model: None,
        })
    }

    /// Lets `model` predict next words and re-weigh candidates in context (see
    /// [`crate::language_model`]), as the scoreboard chose.
    pub fn attach(&mut self, model: Arc<ngram::Model>) {
        self.attach_tuned(model, Tuning::DEFAULT);
    }

    pub fn attach_tuned(&mut self, model: Arc<ngram::Model>, tuning: Tuning) {
        self.model = Some(LanguageModel::new(model, tuning));
    }

    pub fn language_model(&self) -> Option<&LanguageModel> {
        self.model.as_ref()
    }

    /// What the words before `word` say for it, for the swipe ranker: the model's
    /// [`LanguageModel::swipe_ngram`] if there is a model, else the dictionary's n-gram.
    pub fn context_probability(&self, previous: &PreviousWords, word: &[u16]) -> i32 {
        match &self.model {
            Some(model) => model.swipe_ngram(previous, word, self.probability(word)),
            None => self.ngram_probability(previous, word),
        }
    }

    pub fn probability(&self, word: &[u16]) -> i32 {
        if word.is_empty() {
            NOT_A_PROBABILITY
        } else {
            self.dictionary.probability(&code_points(word))
        }
    }

    pub fn ngram_probability(&self, previous: &PreviousWords, word: &[u16]) -> i32 {
        if previous.slot_count() == 0 || word.is_empty() {
            return NOT_A_PROBABILITY;
        }
        let (words, flags) = slots(previous);
        let words: Vec<_> = words.iter().map(|w| Some(w.as_slice())).collect();
        let context = NgramContext::new(&words, &flags);
        self.dictionary
            .ngram_probability(Some(&context), &code_points(word))
    }

    /// Ordered, control-free dictionary words. All filtering stays on the native side.
    pub fn words(&self) -> Vec<Vec<u16>> {
        get_words(&self.dictionary)
            .split(|&u| u == 10)
            .filter(|w| !w.is_empty())
            .map(<[u16]>::to_vec)
            .collect()
    }

    pub fn decode(
        &mut self,
        input: &[u16],
        previous: &PreviousWords,
        block: bool,
        taps: &[Option<TapOffset>],
    ) -> Result<Vec<RawSuggestion>, Error> {
        let codes = code_points(input);
        // Kotlin returns before resetting the session or touching any output slot.
        if codes.len() > MAX_WORD_LENGTH {
            return Ok(vec![]);
        }
        // resetForTyping precedes geometry conversion, even if a NaN tap then throws.
        self.output.weight = -1.0;
        let known_taps = taps.len() == codes.len();
        let points = codes
            .iter()
            .enumerate()
            .map(|(i, &cp)| qwerty::touch_point(cp, if known_taps { taps[i] } else { None }))
            .collect::<Result<Vec<_>, _>>()?;
        let (previous, beginning_of_sentence) = slots(previous);
        self.query(&Query {
            code_points: codes,
            points,
            block_offensive_words: block,
            previous,
            beginning_of_sentence,
        });
        let count = self.output.count.clamp(0, MAX_RESULTS as i32) as usize;
        let mut results = (0..count)
            .map(|i| {
                let start = i * MAX_WORD_LENGTH;
                let word = self.output.code_points[start..start + MAX_WORD_LENGTH]
                    .iter()
                    .take_while(|&&cp| cp != 0)
                    .flat_map(|&cp| utf16(cp))
                    .collect();
                RawSuggestion {
                    word,
                    score: self.output.scores[i],
                    kind_and_flags: self.output.types[i],
                }
            })
            .collect::<Vec<_>>();
        // Kotlin's sortByDescending is stable: equal scores keep native slot order.
        results.sort_by_key(|r| std::cmp::Reverse(r.score));
        Ok(results)
    }

    /// Raw decoder queries also serve the golden session test and test-only JNI replay.
    pub fn query(&mut self, query: &Query) -> &Output {
        self.output.weight = -1.0;
        let options = [0, 0, i32::from(query.block_offensive_words), 0, 1000];
        suggest_into(
            &self.dictionary,
            Some(&self.proximity),
            &mut self.traversal,
            query,
            &options,
            &mut self.output,
        );
        &self.output
    }

    pub fn output(&self) -> &Output {
        &self.output
    }

    pub fn request<'a>(
        &'a mut self,
        previous: &'a PreviousWords,
        block: bool,
        taps: &'a [Option<TapOffset>],
    ) -> Request<'a> {
        Request {
            session: self,
            previous,
            block,
            taps,
        }
    }
}

fn utf16(cp: i32) -> Vec<u16> {
    if cp <= 0xffff {
        vec![cp as u16]
    } else {
        let cp = cp as u32 - 0x10000;
        vec![0xd800 | (cp >> 10) as u16, 0xdc00 | (cp & 0x3ff) as u16]
    }
}

fn slots(previous: &PreviousWords) -> (Vec<Vec<i32>>, Vec<bool>) {
    let count = previous.slot_count();
    let words = (0..count)
        .map(|i| {
            previous
                .words
                .get(i)
                .map_or_else(Vec::new, |w| code_points(w))
        })
        .collect();
    let flags = (0..count).map(|i| i >= previous.words.len()).collect();
    (words, flags)
}

fn proximity() -> Option<ProximityInfo> {
    let keys = qwerty::keys().collect::<Vec<_>>();
    let xs = keys.iter().map(|k| k.x).collect::<Vec<_>>();
    let ys = keys.iter().map(|k| k.y).collect::<Vec<_>>();
    let codes = keys.iter().map(|k| k.code_point).collect::<Vec<_>>();
    let widths = vec![qwerty::KEY_WIDTH; keys.len()];
    let heights = vec![qwerty::KEY_HEIGHT; keys.len()];
    ProximityInfo::new(&KeyboardGeometry {
        keyboard_width: qwerty::KEYBOARD_WIDTH,
        keyboard_height: qwerty::KEYBOARD_HEIGHT,
        grid_width: qwerty::GRID_WIDTH,
        grid_height: qwerty::GRID_HEIGHT,
        most_common_key_width: qwerty::KEY_WIDTH,
        most_common_key_height: qwerty::KEY_HEIGHT,
        proximity_chars: &qwerty::proximity_grid(),
        key_count: keys.len() as i32,
        key_x_coordinates: &xs,
        key_y_coordinates: &ys,
        key_widths: &widths,
        key_heights: &heights,
        key_char_codes: &codes,
    })
    .ok()
}

#[derive(Debug)]
pub struct Request<'a> {
    session: &'a mut Session,
    previous: &'a PreviousWords,
    block: bool,
    taps: &'a [Option<TapOffset>],
}
impl Lookup for Request<'_> {
    type Error = Error;
    fn probability(&mut self, word: &[u16]) -> Result<i32, Error> {
        Ok(self.session.probability(word))
    }
}
impl Dictionary for Request<'_> {
    fn ngram_probability(&mut self, word: &[u16]) -> Result<i32, Error> {
        Ok(self.session.ngram_probability(self.previous, word))
    }
    fn ln_odds(&mut self, word: &[u16], other: &[u16]) -> Result<Option<f32>, Error> {
        let model = self.session.model.as_ref();
        Ok(model.and_then(|model| model.ln_odds(self.previous, word, other)))
    }
    fn decode(&mut self, input: &[u16], purpose: Purpose) -> Result<Vec<RawSuggestion>, Error> {
        let previous = if purpose.in_context() {
            self.previous
        } else {
            &PreviousWords::NONE
        };
        if purpose == Purpose::Predicting
            && let Some(model) = &self.session.model
        {
            return Ok(model.predictions(previous));
        }
        let taps = if purpose.with_taps() { self.taps } else { &[] };
        let mut results = self.session.decode(input, previous, self.block, taps)?;
        if purpose == Purpose::Typing
            && let Some(model) = &self.session.model
        {
            let session = &*self.session;
            let size = code_points(input).len();
            model.rescore(previous, size, &mut results, |word| {
                let unigram = session.probability(word);
                (unigram, session.ngram_probability(previous, word))
            });
        }
        Ok(results)
    }
}
