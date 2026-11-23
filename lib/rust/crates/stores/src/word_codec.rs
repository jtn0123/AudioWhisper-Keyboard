//! The user dictionary store's payload, kind `WORD`, version 1:
//!
//! | What | How |
//! |---|---|
//! | next id | `i64` |
//! | imported databases | count, then each fingerprint as a `u64` |
//! | words | count, then each: id `i64`, frequency `i32`, word text, locale and shortcut each optional |

use crate::codec::{Error, Reader, Writer};
use crate::store::Payload;
use crate::words::{Word, Words};

/// The fewest bytes a word takes: its numbers, an empty word and two absent texts.
const WORD_BYTES: usize = 8 + 4 + 4 + 1 + 1;

impl Payload for Words {
    const KIND: [u8; 4] = *b"WORD";
    const VERSION: u32 = 1;

    fn encode(&self, writer: &mut Writer) {
        writer.i64(self.next_id);
        writer.count(self.imported.len());
        self.imported.iter().for_each(|&print| writer.u64(print));
        writer.count(self.words.len());
        self.words.iter().for_each(|word| write_word(writer, word));
    }

    fn decode(reader: &mut Reader<'_>, _version: u32) -> Result<Self, Error> {
        let next_id = reader.i64()?;
        let imported = (0..reader.count(8)?)
            .map(|_| reader.u64())
            .collect::<Result<_, _>>()?;
        let words: Vec<Word> = (0..reader.count(WORD_BYTES)?)
            .map(|_| read_word(reader))
            .collect::<Result<_, _>>()?;
        let ordered = words.windows(2).all(|pair| pair[0].id < pair[1].id);
        if !ordered || words.last().is_some_and(|word| word.id >= next_id) {
            return Err(Error::Damaged("word ids"));
        }
        Ok(Words {
            next_id,
            imported,
            words,
        })
    }
}

pub(crate) fn write_word(writer: &mut Writer, word: &Word) {
    writer.i64(word.id);
    writer.i32(word.frequency);
    writer.text(&word.word);
    writer.optional_text(word.locale.as_deref());
    writer.optional_text(word.shortcut.as_deref());
}

fn read_word(reader: &mut Reader<'_>) -> Result<Word, Error> {
    Ok(Word {
        id: reader.i64()?,
        frequency: reader.i32()?,
        word: reader.text()?,
        locale: reader.optional_text()?,
        shortcut: reader.optional_text()?,
    })
}
