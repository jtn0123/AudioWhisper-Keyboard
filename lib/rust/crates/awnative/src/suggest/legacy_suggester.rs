//! Host/opt-in fixture adapter for the frozen callback suggester. Never shipped on Android.
use super::suggester::{stores, write_strip};
use super::{boolean, non_null};
use crate::jni::{self, Env, Error, Method, Result, illegal};
use jni_sys::{
    JNIEnv, jboolean, jclass, jfloat, jint, jintArray, jlong, jobject, jobjectArray, jstring,
    jvalue,
};
use std::ptr;
use suggest::dictionary::{Dictionary, Lookup, Purpose};
use suggest::previous_words::PreviousWords;
use suggest::raw::RawSuggestion;
use suggest::suggester::{Suggester, TypingState};

const PROBABILITY: (&std::ffi::CStr, &std::ffi::CStr) = (c"probability", c"(Ljava/lang/String;)I");
const NGRAM_PROBABILITY: (&std::ffi::CStr, &std::ffi::CStr) =
    (c"ngramProbability", c"(Ljava/lang/String;)I");
const DECODE: (&std::ffi::CStr, &std::ffi::CStr) = (c"decode", c"(Ljava/lang/String;ZZ)[I");

/// Kotlin's `WordProbabilities`, called back.
#[derive(Debug)]
pub(super) struct JavaLookup<'a> {
    env: &'a Env,
    probability: Method,
}

impl<'a> JavaLookup<'a> {
    /// `object`'s answers; `what` says which argument it is, if it is null.
    pub(super) fn new(env: &'a Env, object: jobject, what: &'static str) -> Result<Self> {
        let probability = env.method(object, what, PROBABILITY)?;
        Ok(JavaLookup { env, probability })
    }
}

impl Lookup for JavaLookup<'_> {
    type Error = Error;

    fn probability(&mut self, word: &[u16]) -> Result<i32> {
        ask(self.env, self.probability, word)
    }
}

/// Calls `method`, which takes a word and returns an int, with `word`.
fn ask(env: &Env, method: Method, word: &[u16]) -> Result<i32> {
    env.with_string(word, |word| env.call_int(method, &[jvalue { l: word }]))
}

/// Kotlin's `SuggestDictionary`, called back.
#[derive(Debug)]
struct JavaDictionary<'a> {
    lookup: JavaLookup<'a>,
    ngram_probability: Method,
    decode: Method,
}

impl<'a> JavaDictionary<'a> {
    fn new(env: &'a Env, object: jobject) -> Result<Self> {
        Ok(JavaDictionary {
            lookup: JavaLookup::new(env, object, "dictionary")?,
            ngram_probability: env.method(object, "dictionary", NGRAM_PROBABILITY)?,
            decode: env.method(object, "dictionary", DECODE)?,
        })
    }
}

impl Lookup for JavaDictionary<'_> {
    type Error = Error;

    fn probability(&mut self, word: &[u16]) -> Result<i32> {
        self.lookup.probability(word)
    }
}

impl Dictionary for JavaDictionary<'_> {
    fn ngram_probability(&mut self, word: &[u16]) -> Result<i32> {
        ask(self.lookup.env, self.ngram_probability, word)
    }

    fn decode(&mut self, input: &[u16], purpose: Purpose) -> Result<Vec<RawSuggestion>> {
        let env = self.lookup.env;
        let packed = env.with_string(input, |input| {
            let arguments = [
                jvalue { l: input },
                jvalue {
                    z: boolean(purpose.in_context()),
                },
                jvalue {
                    z: boolean(purpose.with_taps()),
                },
            ];
            env.call_object(self.decode, &arguments)
        })?;
        if packed.is_null() {
            return Err(Error::NullPointer("the decoded results"));
        }
        let results = env.read::<i32>(packed, "the decoded results");
        env.delete_local(packed);
        unpack(&results?)
    }
}

/// The decoder's results as `SuggestDictionary.pack` packs them: for each, its score, its kind
/// and flags, its length, then its UTF-16 units.
fn unpack(packed: &[i32]) -> Result<Vec<RawSuggestion>> {
    let mut results = Vec::new();
    let mut rest = packed;
    while let [score, kind_and_flags, length, after @ ..] = rest {
        let length = usize::try_from(*length).map_err(|_| illegal("a negative length"))?;
        let (word, after) = (after.split_at_checked(length))
            .ok_or_else(|| illegal("a word runs past the results"))?;
        let word = (word.iter())
            .map(|&unit| u16::try_from(unit).map_err(|_| illegal("not a UTF-16 unit")))
            .collect::<Result<_>>()?;
        results.push(RawSuggestion {
            word,
            score: *score,
            kind_and_flags: *kind_and_flags,
        });
        rest = after;
    }
    if !rest.is_empty() {
        return Err(illegal("the results stop part way"));
    }
    Ok(results)
}

/// The candidate strip (`Suggester::suggest`) for `typed` after `previousWords`, at most
/// `maxCount` words, drawing on the stores behind `personal` and `learned` (0 for none) and asking
/// `dictionary`. `marks` gets the indexes of the auto-correction and the typed word, -1 for none.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LegacyFixtureNative_suggest(
    env: *mut JNIEnv,
    _: jclass,
    personal: jlong,
    learned: jlong,
    dictionary: jobject,
    typed: jstring,
    previous_words: jobjectArray,
    is_beginning_of_sentence: jboolean,
    is_cursor_mid_word: jboolean,
    max_count: jint,
    threshold: jfloat,
    marks: jintArray,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment; Kotlin holds both stores while it calls.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (personal, learned) = stores(personal, learned)?;
            let previous_words = PreviousWords {
                words: non_null(env.read_strings(previous_words, "previousWords")?)?,
                is_beginning_of_sentence: is_beginning_of_sentence != 0,
            };
            let state = TypingState {
                typed: env.read_string(typed, "typed")?,
                previous_words,
                is_cursor_mid_word: is_cursor_mid_word != 0,
            };
            let mut dictionary = JavaDictionary::new(env, dictionary)?;
            let mut suggester = Suggester {
                dictionary: &mut dictionary,
                personal,
                learned,
                threshold: Some(threshold),
            };
            let strip = suggester.suggest(&state, max_count)?;
            write_strip(env, &strip, marks)
        })
    }
}

/// The spell checker's verdict on `word` (`Suggester::spell`), suggesting at most `maxCount`
/// corrections. `verdict` gets whether the word is valid and whether the first correction is
/// confident, 1 or 0 each.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_LegacyFixtureNative_spell(
    env: *mut JNIEnv,
    _: jclass,
    personal: jlong,
    learned: jlong,
    dictionary: jobject,
    word: jstring,
    max_count: jint,
    threshold: jfloat,
    verdict: jintArray,
) -> jobjectArray {
    // SAFETY: the JVM passes its environment; Kotlin holds both stores while it calls.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let (personal, learned) = stores(personal, learned)?;
            let word = env.read_string(word, "word")?;
            let mut dictionary = JavaDictionary::new(env, dictionary)?;
            let mut suggester = Suggester {
                dictionary: &mut dictionary,
                personal,
                learned,
                threshold: Some(threshold),
            };
            let check = suggester.spell(&word, max_count)?;
            let flags = [check.is_valid, check.is_high_confidence].map(i32::from);
            env.write(verdict, &flags, "verdict")?;
            env.make_strings(&check.suggestions)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unpacks_the_decoders_results() {
        let packed = [5, 1, 2, 0x74, 0x6F, -1, 3, 0, 9, 8, 1, 0xD83D];
        let results = unpack(&packed).ok();
        let expected = vec![
            RawSuggestion {
                word: vec![0x74, 0x6F],
                score: 5,
                kind_and_flags: 1,
            },
            RawSuggestion {
                word: vec![],
                score: -1,
                kind_and_flags: 3,
            },
            RawSuggestion {
                word: vec![0xD83D],
                score: 9,
                kind_and_flags: 8,
            },
        ];
        assert_eq!(results, Some(expected));
        assert_eq!(unpack(&[]).ok(), Some(vec![]));
        for bad in [
            &[5, 1][..],
            &[5, 1, 3, 0x74],
            &[5, 1, -1],
            &[5, 1, 1, 0x10000],
            &[5, 1, 1, -1],
        ] {
            assert!(unpack(bad).is_err(), "{bad:?}");
        }
    }
}
