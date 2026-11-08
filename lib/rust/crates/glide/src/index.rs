//! The swipe words, indexed once per word list and layout: each word as the keys its path visits,
//! and the words filed by the keys they start and end on.

use std::collections::HashMap;
use std::fmt;

use crate::gesture::Gesture;
use crate::layout::Layout;

/// In a [`CharKeys`], a character with no key.
pub const NO_KEY: i32 = -1;

/// In a word's keys, marks a letter repeating the one before it ("pool"), which its path loops on.
const REPEAT: u16 = 0x8000;

/// What a word's character means on the layout being indexed. The keyboard works this out, so
/// case and accents follow Java's rules exactly: a word's path visits [`CharKeys::path_key`] for
/// each character that has one, and a word is filed under the [`CharKeys::end_key`] of its first
/// and last characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CharKeys {
    /// The UTF-16 code unit this describes.
    pub unit: u16,
    /// The unit lower-cased; a letter repeats the one before when these match.
    pub lower: u16,
    /// The code of the key a path visits for it, or [`NO_KEY`] to skip it.
    pub path_key: i32,
    /// The code of the key it files a word under when first or last, or [`NO_KEY`] to leave the
    /// word out.
    pub end_key: i32,
}

/// Why words could not be indexed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexError {
    /// The word starts do not run from 0 up to the number of characters.
    BadStarts,
    /// The words use more keys than a word's keys can name.
    TooManyKeys,
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndexError::BadStarts => {
                f.write_str("word starts must rise from 0 to the character count")
            }
            IndexError::TooManyKeys => write!(f, "the words use more than {} keys", REPEAT - 1),
        }
    }
}

impl std::error::Error for IndexError {}

/// The words a swipe is scored against. Read-only once built, so any number of threads can score
/// at once.
#[derive(Debug)]
pub struct GlideIndex {
    /// Word `w`'s keys are `keys[starts[w]..starts[w + 1]]`.
    starts: Vec<u32>,
    /// Each word's keys in order, as indexes into `codes`, with [`REPEAT`] on a repeated letter.
    keys: Vec<u16>,
    /// The key codes the words use.
    codes: Vec<i32>,
    /// The words starting and ending on each pair of keys, in word order.
    buckets: HashMap<(i32, i32), Vec<u32>>,
    /// Words spelled the same as an earlier one, and that one: a spelling ranks once.
    aliases: HashMap<u32, u32>,
}

impl GlideIndex {
    /// Indexes the words in `chars`: word `w` is `chars[starts[w]..starts[w + 1]]`, in UTF-16.
    /// `table` describes the characters; one missing from it has no key.
    pub fn build(chars: &[u16], starts: &[u32], table: &[CharKeys]) -> Result<Self, IndexError> {
        check_starts(starts, chars.len())?;
        let lookup = CharLookup::new(table);
        let mut index = GlideIndex {
            starts: Vec::with_capacity(starts.len()),
            keys: Vec::with_capacity(chars.len()),
            codes: Vec::new(),
            buckets: HashMap::new(),
            aliases: HashMap::new(),
        };
        let mut slots = HashMap::new();
        let mut first_spelled = HashMap::with_capacity(starts.len());
        for (w, bounds) in starts.windows(2).enumerate() {
            let word = &chars[bounds[0] as usize..bounds[1] as usize];
            let w = w as u32;
            index.starts.push(index.keys.len() as u32);
            index.add_keys(word, &lookup, &mut slots)?;
            if let Some(&first) = first_spelled.get(word) {
                index.aliases.insert(w, first);
            } else {
                first_spelled.insert(word, w);
            }
            if let (Some(&first), Some(&last)) = (word.first(), word.last()) {
                let (first, last) = (lookup.get(first).end_key, lookup.get(last).end_key);
                if first != NO_KEY && last != NO_KEY {
                    index.buckets.entry((first, last)).or_default().push(w);
                }
            }
        }
        index.starts.push(index.keys.len() as u32);
        Ok(index)
    }

    /// Appends `word`'s keys: one per character with a key, flagged when it repeats the letter
    /// before. A character with no key is skipped, and doesn't count as a letter to repeat.
    fn add_keys(
        &mut self,
        word: &[u16],
        lookup: &CharLookup,
        slots: &mut HashMap<i32, u16>,
    ) -> Result<(), IndexError> {
        let mut previous = 0u16;
        for &unit in word {
            let char_keys = lookup.get(unit);
            if char_keys.path_key == NO_KEY {
                continue;
            }
            let slot = match slots.get(&char_keys.path_key) {
                Some(&slot) => slot,
                None => {
                    let slot = u16::try_from(self.codes.len())
                        .ok()
                        .filter(|&slot| slot < REPEAT)
                        .ok_or(IndexError::TooManyKeys)?;
                    self.codes.push(char_keys.path_key);
                    slots.insert(char_keys.path_key, slot);
                    slot
                }
            };
            self.keys.push(if previous == char_keys.lower {
                slot | REPEAT
            } else {
                slot
            });
            previous = char_keys.lower;
        }
        Ok(())
    }

    pub fn word_count(&self) -> usize {
        self.starts.len() - 1
    }

    /// The key codes the words' keys index, for [`Layout::new`].
    pub fn codes(&self) -> &[i32] {
        &self.codes
    }

    /// The words filed under the pair of keys, in word order.
    pub(crate) fn bucket(&self, first: i32, last: i32) -> &[u32] {
        self.buckets.get(&(first, last)).map_or(&[], Vec::as_slice)
    }

    /// The first word spelled as `word` is; a spelling ranks once, as that word.
    pub(crate) fn spelling(&self, word: u32) -> u32 {
        if self.aliases.is_empty() {
            word
        } else {
            self.aliases.get(&word).copied().unwrap_or(word)
        }
    }

    fn word_keys(&self, word: u32) -> &[u16] {
        let w = word as usize;
        &self.keys[self.starts[w] as usize..self.starts[w + 1] as usize]
    }

    /// Writes `word`'s ideal path on `layout` into `plain`: the centre of each key in turn.
    pub(crate) fn plain_path(&self, word: u32, layout: &Layout, plain: &mut Gesture) {
        plain.clear();
        for &key in self.word_keys(word) {
            if let Some(bounds) = layout.slot((key & !REPEAT) as usize) {
                plain.push(bounds.center_x, bounds.center_y);
            }
        }
    }

    /// Writes `word`'s ideal paths on `layout` into `plain` and `loops`: `loops` circles a key
    /// for a repeated letter, so "pool" and "poll" differ. Returns whether it has any loops.
    pub(crate) fn paths(
        &self,
        word: u32,
        layout: &Layout,
        plain: &mut Gesture,
        loops: &mut Gesture,
    ) -> bool {
        plain.clear();
        loops.clear();
        let mut has_loops = false;
        for &key in self.word_keys(word) {
            // A key the words use can be missing from a layout that has just changed.
            let Some(bounds) = layout.slot((key & !REPEAT) as usize) else {
                continue;
            };
            let (x, y) = (bounds.center_x, bounds.center_y);
            if key & REPEAT != 0 {
                let dx = bounds.width / 4.0;
                let dy = bounds.height / 4.0;
                loops.push(x + dx, y + dy);
                loops.push(x + dx, y - dy);
                loops.push(x - dx, y - dy);
                loops.push(x - dx, y + dy);
                has_loops = true;
            } else {
                loops.push(x, y);
            }
            plain.push(x, y);
        }
        has_loops
    }
}

fn check_starts(starts: &[u32], chars: usize) -> Result<(), IndexError> {
    let rises = starts.windows(2).all(|pair| pair[0] <= pair[1]);
    if starts.first() == Some(&0)
        && starts.last() == Some(&(chars as u32))
        && rises
        && chars <= u32::MAX as usize
    {
        Ok(())
    } else {
        Err(IndexError::BadStarts)
    }
}

/// [`CharKeys`] by code unit.
struct CharLookup {
    /// Each unit's place in `table`, or `u16::MAX` for none.
    places: Vec<u16>,
    table: Vec<CharKeys>,
}

impl CharLookup {
    const NONE: CharKeys = CharKeys {
        unit: 0,
        lower: 0,
        path_key: NO_KEY,
        end_key: NO_KEY,
    };

    fn new(table: &[CharKeys]) -> Self {
        let mut places = vec![u16::MAX; 1 << 16];
        // A unit described twice keeps its last description; a table that long has duplicates.
        let table: Vec<CharKeys> = table.iter().take(u16::MAX as usize).copied().collect();
        for (place, char_keys) in table.iter().enumerate() {
            places[char_keys.unit as usize] = place as u16;
        }
        CharLookup { places, table }
    }

    fn get(&self, unit: u16) -> CharKeys {
        match self.places[unit as usize] {
            u16::MAX => Self::NONE,
            place => self.table[place as usize],
        }
    }
}

/// Every distinct code unit in `chars`, ascending: the characters a [`CharKeys`] table describes.
pub fn distinct_units(chars: &[u16]) -> Vec<u16> {
    let mut seen = vec![false; 1 << 16];
    for &unit in chars {
        seen[unit as usize] = true;
    }
    (0..=u16::MAX).filter(|&unit| seen[unit as usize]).collect()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::gesture::MAX_POINTS;
    use crate::layout::Key;

    /// A table for lower-case ASCII letters on keys of their own code, plus `extra`.
    pub(crate) fn ascii_table(extra: &[CharKeys]) -> Vec<CharKeys> {
        let mut table: Vec<CharKeys> = (b'a'..=b'z')
            .map(|c| CharKeys {
                unit: c as u16,
                lower: c as u16,
                path_key: c as i32,
                end_key: c as i32,
            })
            .collect();
        table.extend_from_slice(extra);
        table
    }

    pub(crate) fn pack(words: &[&str]) -> (Vec<u16>, Vec<u32>) {
        let mut chars = Vec::new();
        let mut starts = vec![0];
        for word in words {
            chars.extend(word.encode_utf16());
            starts.push(chars.len() as u32);
        }
        (chars, starts)
    }

    pub(crate) fn index(words: &[&str], table: &[CharKeys]) -> GlideIndex {
        let (chars, starts) = pack(words);
        GlideIndex::build(&chars, &starts, table).unwrap()
    }

    /// Keys in a row, 10 apart, for the codes given.
    pub(crate) fn row(codes: &[i32]) -> Vec<Key> {
        codes
            .iter()
            .enumerate()
            .map(|(i, &code)| Key {
                code,
                center_x: i as f32 * 10.0 + 5.0,
                center_y: 5.0,
                width: 10.0,
                height: 10.0,
            })
            .collect()
    }

    type Points = Vec<(f32, f32)>;

    fn path(index: &GlideIndex, word: u32, keys: &[Key]) -> (Points, Points, bool) {
        let layout = Layout::new(keys, index.codes());
        let (mut plain, mut loops) = (Gesture::new(), Gesture::new());
        let has_loops = index.paths(word, &layout, &mut plain, &mut loops);
        let points = |g: &Gesture| (0..g.len()).map(|i| (g.x(i), g.y(i))).collect();
        (points(&plain), points(&loops), has_loops)
    }

    #[test]
    fn a_repeated_letter_loops_round_its_key() {
        let index = index(&["ab", "abb"], &ascii_table(&[]));
        let keys = row(&['a' as i32, 'b' as i32]);

        let (plain, loops, has_loops) = path(&index, 0, &keys);
        assert_eq!(plain, vec![(5.0, 5.0), (15.0, 5.0)]);
        assert_eq!(loops, plain);
        assert!(!has_loops);

        let (plain, loops, has_loops) = path(&index, 1, &keys);
        assert_eq!(plain, vec![(5.0, 5.0), (15.0, 5.0), (15.0, 5.0)]);
        assert_eq!(
            loops,
            vec![
                (5.0, 5.0),
                (15.0, 5.0),
                (17.5, 7.5),
                (17.5, 2.5),
                (12.5, 2.5),
                (12.5, 7.5)
            ]
        );
        assert!(has_loops);
    }

    #[test]
    fn a_character_without_a_key_is_skipped_and_does_not_break_a_repeat() {
        let apostrophe = CharKeys {
            unit: '\'' as u16,
            lower: '\'' as u16,
            path_key: NO_KEY,
            end_key: NO_KEY,
        };
        let index = index(&["a'a"], &ascii_table(&[apostrophe]));

        let (plain, _, has_loops) = path(&index, 0, &row(&['a' as i32]));
        assert_eq!(plain.len(), 2);
        assert!(has_loops);
    }

    #[test]
    fn case_and_accents_follow_the_table() {
        // As the keyboard describes them: "É" lower-cases to "é", which has no key of its own and
        // so uses "e"'s; "E" repeats "e", but "é" doesn't.
        let accented = |unit: char, lower: char| CharKeys {
            unit: unit as u16,
            lower: lower as u16,
            path_key: 'e' as i32,
            end_key: 'e' as i32,
        };
        let upper_e = CharKeys {
            unit: 'E' as u16,
            lower: 'e' as u16,
            path_key: 'e' as i32,
            end_key: 'e' as i32,
        };
        let table = ascii_table(&[accented('é', 'é'), accented('É', 'é'), upper_e]);
        let index = index(&["Ée", "eE"], &table);
        let keys = row(&['e' as i32]);

        assert!(!path(&index, 0, &keys).2);
        assert!(path(&index, 1, &keys).2);
        assert_eq!(index.bucket('e' as i32, 'e' as i32), &[0, 1]);
    }

    #[test]
    fn words_are_filed_by_their_first_and_last_keys() {
        let path_only = CharKeys {
            unit: 'x' as u16,
            lower: 'x' as u16,
            path_key: 'x' as i32,
            end_key: NO_KEY,
        };
        let mut table = ascii_table(&[]);
        table.retain(|c| c.unit != 'x' as u16);
        table.push(path_only);
        let index = index(&["cat", "cut", "cab", "tax", "xyz", ""], &table);

        assert_eq!(index.bucket('c' as i32, 't' as i32), &[0, 1]);
        assert_eq!(index.bucket('c' as i32, 'b' as i32), &[2]);
        assert!(index.bucket('t' as i32, 'x' as i32).is_empty());
        assert!(index.bucket('x' as i32, 'z' as i32).is_empty());
        assert_eq!(index.word_count(), 6);
    }

    #[test]
    fn a_spelling_ranks_as_its_first_word() {
        let index = index(&["to", "too", "to"], &ascii_table(&[]));

        assert_eq!(index.spelling(0), 0);
        assert_eq!(index.spelling(1), 1);
        assert_eq!(index.spelling(2), 0);
    }

    #[test]
    fn bad_starts_are_refused() {
        let chars: Vec<u16> = "abc".encode_utf16().collect();
        for starts in [&[][..], &[1, 3], &[0, 2], &[0, 2, 1, 3], &[0, 4]] {
            assert_eq!(
                GlideIndex::build(&chars, starts, &[]).unwrap_err(),
                IndexError::BadStarts,
                "{starts:?}"
            );
        }
        assert!(GlideIndex::build(&[], &[0], &[]).is_ok());
    }

    #[test]
    fn a_path_stops_at_the_points_a_gesture_holds() {
        let long = "ab".repeat(MAX_POINTS);
        let index = index(&[&long], &ascii_table(&[]));

        assert_eq!(
            path(&index, 0, &row(&['a' as i32, 'b' as i32])).0.len(),
            MAX_POINTS
        );
    }

    #[test]
    fn distinct_units_are_listed_once_in_order() {
        let chars: Vec<u16> = "banana é".encode_utf16().collect();

        assert_eq!(
            distinct_units(&chars),
            vec![' ' as u16, 'a' as u16, 'b' as u16, 'n' as u16, 'é' as u16]
        );
    }
}
