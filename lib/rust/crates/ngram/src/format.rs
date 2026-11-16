//! The model file. Everything is little-endian, and every section starts on a 4-byte boundary,
//! in this order:
//!
//! | Section | Size | What |
//! |---|---|---|
//! | header | [`HEADER_LEN`] bytes | [`Header`] |
//! | spelling starts | (words + 1) × u32 | where each word's UTF-16 units start in the text |
//! | text | units × u16 | the words' spellings, in UTF-16 order, so ids are that order |
//! | unigrams | (words + 1) × u32 | each word's, then the sentence start's: [`Unigram`] |
//! | bigram starts | (words + 2) × u32 | where the next words after each word, then after the sentence start, begin |
//! | bigrams | bigrams × u32 | a next word and its probability ([`pack`]), by previous word, then id |
//! | bigram backoffs | bigrams × u8 | the backoff after the two words each bigram makes |
//! | contexts | contexts × u32 | the bigrams that have trigrams, as their indexes, in order |
//! | trigram starts | (contexts + 1) × u32 | where the next words after each of those begin |
//! | trigrams | trigrams × u32 | as bigrams, by context, then id |
//!
//! Probabilities and backoffs are natural logs, quantised: a byte `q` stands for `-q × step`.

/// The file's first four bytes.
pub const MAGIC: [u8; 4] = *b"AWLM";
pub const VERSION: u32 = 1;
pub const HEADER_LEN: usize = 48;
/// Word ids are 24 bits in a packed n-gram.
pub const MAX_WORDS: u32 = 1 << 24;
/// A unigram flag: a sentence's first word spelled like this one is counted, and looked up, as its
/// lower-case spelling ("Will" at a sentence start is mostly "will").
pub const PREFERS_LOWER_AT_START: u8 = 1;

/// The header's fields, after the magic number and version.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Header {
    /// Words in the vocabulary; the sentence start's id is this.
    pub words: u32,
    /// UTF-16 units in the text.
    pub units: u32,
    pub bigrams: u32,
    /// Bigrams with trigrams after them.
    pub contexts: u32,
    pub trigrams: u32,
    /// The quantisation step of a log probability, in nats.
    pub step: f32,
    /// The quantisation step of a log backoff, in nats.
    pub backoff_step: f32,
    /// The dictionary's value scale (value = `a + b ln p`), so a probability can be compared
    /// with the dictionary's 0-255 values.
    pub scale_a: f32,
    pub scale_b: f32,
}

/// What the model holds for one word.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Unigram {
    /// The word's lowest-order (continuation) probability.
    pub probability: u8,
    /// The backoff after this word alone.
    pub backoff: u8,
    /// How often the corpus writes the word, as a probability.
    pub frequency: u8,
    pub flags: u8,
}

impl Unigram {
    pub fn to_bytes(self) -> [u8; 4] {
        [self.probability, self.backoff, self.frequency, self.flags]
    }

    pub fn from_bytes(bytes: [u8; 4]) -> Unigram {
        Unigram {
            probability: bytes[0],
            backoff: bytes[1],
            frequency: bytes[2],
            flags: bytes[3],
        }
    }
}

/// An n-gram's next word and quantised probability in one u32.
pub fn pack(word: u32, probability: u8) -> u32 {
    (word & (MAX_WORDS - 1)) | u32::from(probability) << 24
}

pub fn unpack(entry: u32) -> (u32, u8) {
    (entry & (MAX_WORDS - 1), (entry >> 24) as u8)
}

/// The byte for log probability `ln` at `step`: the nearest, within 0-255.
pub fn quantise(ln: f64, step: f64) -> u8 {
    (-ln / step).round().clamp(0.0, 255.0) as u8
}

pub fn ln(q: u8, step: f32) -> f32 {
    -f32::from(q) * step
}

/// Where each section starts, from the header's counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Layout {
    pub spelling_starts: usize,
    pub text: usize,
    pub unigrams: usize,
    pub bigram_starts: usize,
    pub bigrams: usize,
    pub backoffs: usize,
    pub contexts: usize,
    pub trigram_starts: usize,
    pub trigrams: usize,
    /// The file's length.
    pub end: usize,
}

fn align(at: usize) -> Option<usize> {
    at.checked_add(3).map(|n| n & !3)
}

impl Layout {
    pub fn of(header: &Header) -> Option<Layout> {
        let words = header.words as usize;
        let bigrams = header.bigrams as usize;
        let section = |at: usize, count: usize, size: usize| {
            count.checked_mul(size).and_then(|n| at.checked_add(n))
        };
        let spelling_starts = HEADER_LEN;
        let text = section(spelling_starts, words.checked_add(1)?, 4)?;
        let unigrams = align(section(text, header.units as usize, 2)?)?;
        let bigram_starts = section(unigrams, words.checked_add(1)?, 4)?;
        let bigram_entries = section(bigram_starts, words.checked_add(2)?, 4)?;
        let backoffs = section(bigram_entries, bigrams, 4)?;
        let context_entries = align(section(backoffs, bigrams, 1)?)?;
        let contexts = header.contexts as usize;
        let trigram_starts = section(context_entries, contexts, 4)?;
        let trigrams = section(trigram_starts, contexts.checked_add(1)?, 4)?;
        let end = section(trigrams, header.trigrams as usize, 4)?;
        Some(Layout {
            spelling_starts,
            text,
            unigrams,
            bigram_starts,
            bigrams: bigram_entries,
            backoffs,
            contexts: context_entries,
            trigram_starts,
            trigrams,
            end,
        })
    }
}

impl Header {
    pub fn to_bytes(&self) -> [u8; HEADER_LEN] {
        let mut bytes = [0; HEADER_LEN];
        let fields = [
            u32::from_le_bytes(MAGIC),
            VERSION,
            self.words,
            self.units,
            self.bigrams,
            self.contexts,
            self.trigrams,
            self.step.to_bits(),
            self.backoff_step.to_bits(),
            self.scale_a.to_bits(),
            self.scale_b.to_bits(),
        ];
        for (chunk, field) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(fields) {
            *chunk = field.to_le_bytes();
        }
        bytes
    }

    /// The header of a model file, if it is one this reader knows.
    pub fn read(bytes: &[u8]) -> Option<Header> {
        let field = |i: usize| {
            let at = i * 4;
            bytes
                .get(at..at + 4)
                .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
        };
        if bytes.len() < HEADER_LEN || bytes[..4] != MAGIC || field(1)? != VERSION {
            return None;
        }
        let header = Header {
            words: field(2)?,
            units: field(3)?,
            bigrams: field(4)?,
            contexts: field(5)?,
            trigrams: field(6)?,
            step: f32::from_bits(field(7)?),
            backoff_step: f32::from_bits(field(8)?),
            scale_a: f32::from_bits(field(9)?),
            scale_b: f32::from_bits(field(10)?),
        };
        let finite = [
            header.step,
            header.backoff_step,
            header.scale_a,
            header.scale_b,
        ]
        .iter()
        .all(|v| v.is_finite());
        (finite && header.words < MAX_WORDS && header.step > 0.0 && header.backoff_step > 0.0)
            .then_some(header)
    }
}
