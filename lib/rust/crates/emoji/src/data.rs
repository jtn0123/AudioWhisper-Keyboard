//! An emoji file parsed as `EmojiData.loadEmojiDataMap` parsed it.
//!
//! The file is read line by line, as Java's `BufferedReader.readLine` cuts it (at `\n`, `\r` or
//! `\r\n`). A line starting with `#` is a comment, even `#️⃣;keycap: #;...`, which the bundled
//! files therefore never offer. `[id]` starts a category: the text between the first and the last
//! character is looked up among the category ids, and an unknown one skips what follows until the
//! next. Blank lines are skipped, without ending a set. Any other line is an emoji,
//! `value;name;keyword|keyword|...` (exactly three fields, or the line is dropped), each piece
//! trimmed of Kotlin's whitespace. An emoji line starting with a tab is a variant of the set
//! before it and takes that set's first name; any other emoji line, and every `[`, ends the set.
//!
//! [`EmojiData::by_skin_tone`] lists, for each skin tone, the emoji to show with that tone
//! chosen, category by category in the ids' order: a set's only emoji for every tone, or each of
//! a set's emoji under its own tone.

use crate::tones::{SKIN_TONES, tones};

const LF: u16 = b'\n' as u16;
const CR: u16 = b'\r' as u16;
const HASH: u16 = b'#' as u16;
const OPEN: u16 = b'[' as u16;
const TAB: u16 = b'\t' as u16;
const FIELD: u16 = b';' as u16;
const KEYWORD: u16 = b'|' as u16;

/// Where a piece of the file is: units `start..end` of its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn of(self, text: &[u16]) -> &[u16] {
        &text[self.start..self.end]
    }
}

/// One emoji: where its value, name and keywords are in the file, and its tones (see
/// [`tones`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub value: Span,
    pub name: Span,
    pub keywords: Vec<Span>,
    pub skin_tone: usize,
    pub hair_style: usize,
}

/// A set of emoji, a base and its variants: entries `start..start + len` in `category`, a
/// position among the ids the file was parsed with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Set {
    pub category: usize,
    pub start: usize,
    pub len: usize,
}

/// A parsed emoji file, which holds its text.
#[derive(Debug)]
pub struct EmojiData {
    pub text: Vec<u16>,
    /// Every emoji, in the file's order.
    pub entries: Vec<Entry>,
    /// Every set, in the file's order.
    pub sets: Vec<Set>,
    /// For each skin tone (in `EmojiSkinTone`'s order), the entries to show.
    pub by_skin_tone: [Vec<usize>; SKIN_TONES.len()],
}

/// The lines of `text` as `BufferedReader.readLine` reads them, without their ends.
fn lines(text: &[u16]) -> impl Iterator<Item = Span> + '_ {
    let mut at = 0;
    std::iter::from_fn(move || {
        if at >= text.len() {
            return None;
        }
        let start = at;
        let end = text[start..]
            .iter()
            .position(|&u| u == LF || u == CR)
            .map_or(text.len(), |p| start + p);
        at = end;
        if at < text.len() {
            at += if text[at] == CR && text.get(at + 1) == Some(&LF) {
                2
            } else {
                1
            };
        }
        Some(Span { start, end })
    })
}

/// `span` without Kotlin's whitespace at either end (`String.trim()`).
fn trim(text: &[u16], span: Span) -> Span {
    let piece = span.of(text);
    let start = piece
        .iter()
        .position(|&u| !chars::is_whitespace(u))
        .unwrap_or(piece.len());
    let end = piece
        .iter()
        .rposition(|&u| !chars::is_whitespace(u))
        .map_or(start, |i| i + 1);
    Span {
        start: span.start + start,
        end: span.start + end,
    }
}

/// `span` cut at each `separator` (Kotlin's `split`, so empty pieces stay).
fn split(text: &[u16], span: Span, separator: u16) -> Vec<Span> {
    let mut pieces = Vec::new();
    let mut start = span.start;
    for (i, &u) in span.of(text).iter().enumerate() {
        if u == separator {
            pieces.push(Span {
                start,
                end: span.start + i,
            });
            start = span.start + i + 1;
        }
    }
    pieces.push(Span {
        start,
        end: span.end,
    });
    pieces
}

/// The set being read: where its entries start, and its category.
#[derive(Clone, Copy)]
struct Open {
    start: usize,
    category: usize,
}

struct Parser<'a> {
    text: &'a [u16],
    entries: Vec<Entry>,
    sets: Vec<Set>,
    open: Option<Open>,
}

impl Parser<'_> {
    /// Ends the set being read, if there is one (`commitEmojiEditorList`).
    fn commit(&mut self) {
        if let Some(open) = self.open.take() {
            self.sets.push(Set {
                category: open.category,
                start: open.start,
                len: self.entries.len() - open.start,
            });
        }
    }

    /// An emoji line in `category`.
    fn emoji(&mut self, line: Span, category: usize) {
        let text = self.text;
        if text[line.start] != TAB {
            self.commit();
        }
        let &[value, name, keywords] = split(text, line, FIELD).as_slice() else {
            return;
        };
        let value = trim(text, value);
        let name = match self.open {
            Some(open) => self.entries[open.start].name,
            None => trim(text, name),
        };
        let keywords = split(text, keywords, KEYWORD)
            .into_iter()
            .map(|keyword| trim(text, keyword))
            .collect();
        let (skin_tone, hair_style) = tones(value.of(text));
        if self.open.is_none() {
            self.open = Some(Open {
                start: self.entries.len(),
                category,
            });
        }
        self.entries.push(Entry {
            value,
            name,
            keywords,
            skin_tone,
            hair_style,
        });
    }
}

impl EmojiData {
    /// Parses an emoji file's `text`, with `categories` the ids of `EmojiCategory`'s entries in
    /// its order.
    pub fn parse(text: Vec<u16>, categories: &[&[u16]]) -> EmojiData {
        let mut parser = Parser {
            text: &text,
            entries: Vec::new(),
            sets: Vec::new(),
            open: None,
        };
        let mut category: Option<usize> = None;
        for line in lines(&text) {
            let units = line.of(&text);
            match units.first() {
                Some(&HASH) => {}
                Some(&OPEN) => {
                    parser.commit();
                    // `line.slice(1 until (line.length - 1))`: empty for "[" itself.
                    let id = &units[1..(units.len() - 1).max(1)];
                    category = categories.iter().position(|&c| c == id);
                }
                _ => match category {
                    Some(category) if !chars::is_blank(units) => parser.emoji(line, category),
                    _ => {}
                },
            }
        }
        parser.commit();
        let (entries, sets) = (parser.entries, parser.sets);
        let by_skin_tone = by_skin_tone(&entries, &sets, categories.len());
        EmojiData {
            text,
            entries,
            sets,
            by_skin_tone,
        }
    }

    /// The text of a piece of the file.
    pub fn units(&self, span: Span) -> &[u16] {
        span.of(&self.text)
    }
}

fn by_skin_tone(
    entries: &[Entry],
    sets: &[Set],
    categories: usize,
) -> [Vec<usize>; SKIN_TONES.len()] {
    let mut by_tone: [Vec<usize>; SKIN_TONES.len()] = Default::default();
    for category in 0..categories {
        for set in sets.iter().filter(|set| set.category == category) {
            if set.len == 1 {
                // No variants: the base for every tone.
                for list in &mut by_tone {
                    list.push(set.start);
                }
                continue;
            }
            for entry in set.start..set.start + set.len {
                by_tone[entries[entry].skin_tone].push(entry);
            }
        }
    }
    by_tone
}

#[cfg(test)]
mod tests;
