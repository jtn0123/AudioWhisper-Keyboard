//! The user dictionary the keyboard keeps itself (`floris_user_dictionary`), with the semantics
//! of the Room table it replaces: rows in the order of their ids, which only grow; an id of 0
//! asks for the next one; another id is kept, unless a row has it already.

/// One word of the dictionary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Word {
    pub id: i64,
    pub word: Vec<u16>,
    pub frequency: i32,
    /// A language tag, or none for every language.
    pub locale: Option<Vec<u16>>,
    pub shortcut: Option<Vec<u16>>,
}

/// Why a word was not added.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Taken(pub i64);

/// The user dictionary store's value.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Words {
    /// The id the next word asking for one gets, unless one has a larger id.
    pub next_id: i64,
    /// The fingerprints of the old databases imported into it ([`crate::legacy`]).
    pub imported: Vec<u64>,
    /// By id, smallest first.
    pub words: Vec<Word>,
}

impl Words {
    /// Every word, by id.
    pub fn all(&self) -> &[Word] {
        &self.words
    }

    /// The words that are `word` exactly in `locale` exactly (none matching none), by id.
    pub fn exact(&self, word: &[u16], locale: Option<&[u16]>) -> Vec<&Word> {
        let matches = |row: &&Word| row.word == word && row.locale.as_deref() == locale;
        self.words.iter().filter(matches).collect()
    }

    /// Adds `word` with its id, or the next one if that is 0; the id it got.
    pub fn insert(&mut self, mut word: Word) -> Result<i64, Taken> {
        if word.id == 0 {
            word.id = self.next_id.max(1);
        }
        if word.id == i64::MAX {
            return Err(Taken(word.id)); // SQLite's rowids end there, leaving no next one.
        }
        let at = match self.words.binary_search_by_key(&word.id, |row| row.id) {
            Ok(_) => return Err(Taken(word.id)),
            Err(at) => at,
        };
        let id = word.id;
        self.next_id = self.next_id.max(id + 1);
        self.words.insert(at, word);
        Ok(id)
    }

    /// Replaces the row with `word`'s id, if there is one.
    pub fn update(&mut self, word: Word) {
        if let Some(row) = self.words.iter_mut().find(|row| row.id == word.id) {
            *row = word;
        }
    }

    /// Removes the row `id`, if there is one.
    pub fn delete(&mut self, id: i64) {
        self.words.retain(|row| row.id != id);
    }
}
