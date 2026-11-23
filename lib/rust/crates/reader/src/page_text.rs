//! The text of a page read aloud, as Kotlin's `PageText` had it: which block reading starts at,
//! how the pieces a walk of the screen found join into blocks, where a selection starts, where
//! reading carries on after a scroll, and how text is cut into what the speech engine takes.

use std::collections::HashSet;
use std::num::NonZeroUsize;

use chars::{is_blank, is_letter_or_digit, is_regex_space, is_whitespace, to_lower, trim};

/// Text-to-speech refuses anything longer than 4,000 characters; a margin under that.
pub const MAX_UTTERANCE: usize = 3_900;

/// How many of the last blocks read are kept to find the place again after a scroll.
pub const RECENT: usize = 50;

/// How many of the last blocks read are matched to find the place again after a scroll.
const ANCHOR: usize = 3;

/// What a list's bullet is made of, which a block doesn't start with: • ◦ ▪ ▫ ‣ ⁃ ·, a space and
/// a no-break space.
const BULLETS: [u16; 9] = [
    0x2022, 0x25E6, 0x25AA, 0x25AB, 0x2023, 0x2043, 0x00B7, SPACE, 0x00A0,
];

/// Punctuation that closes what is before it: , . ; : ! ? ) ] } ” ’.
const CLOSING: [u16; 11] = [
    b',' as u16,
    b'.' as u16,
    b';' as u16,
    b':' as u16,
    b'!' as u16,
    b'?' as u16,
    b')' as u16,
    b']' as u16,
    b'}' as u16,
    0x201D,
    0x2019,
];

const SPACE: u16 = b' ' as u16;
const LINE_FEED: u16 = b'\n' as u16;
const CARRIAGE_RETURN: u16 = b'\r' as u16;

/// Where a piece of text is on the screen, in pixels as the app reports them: a web page has its
/// whole text there, and a piece scrolled out of sight is squashed to no height at the edge it
/// went past.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    pub top: i32,
    pub bottom: i32,
}

impl Edges {
    /// Whether any of it shows in `visible`, the part of the screen the app shows.
    pub fn is_shown(self, visible: Edges) -> bool {
        self.bottom > self.top && self.bottom > visible.top && self.top < visible.bottom
    }
}

/// A piece of text a walk of the screen found: its `text`, its `edges`, and whether it `scrolls`,
/// being in the list that scrolls rather than in a bar fixed above or below it.
#[derive(Clone, Copy, Debug)]
pub struct Piece<'a> {
    pub text: &'a [u16],
    pub edges: Edges,
    pub scrolls: bool,
}

/// A block to read, joined from pieces starting with the one at `first`: their `text`, and the
/// `edges` of those in sight (of the first, if none is).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub first: usize,
    pub text: Vec<u16>,
    pub edges: Edges,
}

/// Where reading starts: a block, and a UTF-16 unit within its text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Start {
    pub block: usize,
    pub offset: usize,
}

/// The first of `blocks` that shows in `visible`, so a line cut off at the top is read whole.
pub fn first_on_screen(blocks: &[Edges], visible: Edges) -> Option<usize> {
    blocks.iter().position(|it| it.is_shown(visible))
}

/// Whether every one of `blocks` shows in `visible`: a short page, or a web page whose tree is
/// still being built.
pub fn all_shown(blocks: &[Edges], visible: Edges) -> bool {
    blocks.iter().all(|it| it.is_shown(visible))
}

/// The `pieces` a walk of the screen found, made into blocks to read: a paragraph that holds a
/// link or bold words arrives as several pieces, which are joined so it is spoken as one; list
/// bullets are dropped, and so is a piece with no words in it.
pub fn blocks(pieces: &[Piece]) -> Vec<Block> {
    let mut out = Vec::new();
    let mut first = 0;
    for next in 1..pieces.len() {
        if !continues(&pieces[next - 1], &pieces[next]) {
            out.extend(join(&pieces[first..next], first));
            first = next;
        }
    }
    out.extend(join(&pieces[first..], first));
    out
}

/// Whether `next` carries on `previous`'s sentence: a web page keeps the space at the edge of a
/// link or bold words ("has a " + "link"), and closing punctuation always belongs to what's
/// before. A piece in the list that scrolls never carries on one fixed above or below it.
fn continues(previous: &Piece, next: &Piece) -> bool {
    let (Some(&last), Some(&first)) = (previous.text.last(), next.text.first()) else {
        return false;
    };
    previous.scrolls == next.scrolls
        && (is_whitespace(last) || is_whitespace(first) || CLOSING.contains(&first))
}

/// The pieces of `run`, the first of which is at `first`, as one block; `None` if they hold no
/// letter or digit.
fn join(run: &[Piece], first: usize) -> Option<Block> {
    let text: Vec<u16> = run.iter().flat_map(|it| it.text).copied().collect();
    let bullets = text.iter().take_while(|it| BULLETS.contains(it)).count();
    let text = trim(&text[bullets..]);
    if !text.iter().any(|&it| is_letter_or_digit(it)) {
        return None;
    }
    let edges = run.first()?.edges;
    let shown = || {
        run.iter()
            .map(|it| it.edges)
            .filter(|it| it.bottom > it.top)
    };
    let edges = Edges {
        top: shown().map(|it| it.top).min().unwrap_or(edges.top),
        bottom: shown().map(|it| it.bottom).max().unwrap_or(edges.bottom),
    };
    let text = text.to_vec();
    Some(Block { first, text, edges })
}

/// Where `selected` starts in `blocks`, each a text and its edges: in the block that contains it,
/// preferring one that shows in `visible` (the same words can appear twice on a page). A selection
/// across paragraphs is found by its first line. Matching ignores case and runs of spaces, which
/// apps report differently.
pub fn find(blocks: &[(&[u16], Edges)], selected: &[u16], visible: Edges) -> Option<Start> {
    let needle = normalize(first_line(selected)?);
    let mut first = None;
    for (block, &(text, edges)) in blocks.iter().enumerate() {
        let Some(offset) = index_in(text, &needle) else {
            continue;
        };
        let start = Start { block, offset };
        if edges.is_shown(visible) {
            return Some(start);
        }
        first = first.or(Some(start));
    }
    first
}

/// The first line of `text` that isn't blank, trimmed: Kotlin's
/// `lineSequence().map(String::trim).firstOrNull(String::isNotEmpty)`.
fn first_line(text: &[u16]) -> Option<&[u16]> {
    (text.split(|&it| it == LINE_FEED || it == CARRIAGE_RETURN))
        .map(trim)
        .find(|it| !it.is_empty())
}

/// `text` with each run of what a regex's `\s` matches made one space, in lower case: Kotlin's
/// `replace(Regex("\\s+"), " ").lowercase(Locale.ROOT)`.
fn normalize(text: &[u16]) -> Vec<u16> {
    let mut spaced = Vec::with_capacity(text.len());
    let mut in_run = false;
    for &unit in text {
        let space = is_regex_space(unit);
        if !(space && in_run) {
            spaced.push(if space { SPACE } else { unit });
        }
        in_run = space;
    }
    chars::lowercase(&spaced)
}

/// Where `needle`, made by [`normalize`], starts in `text`, as an index into `text` itself.
///
/// `text` is folded nearly as `needle` was, but not quite, as in Kotlin: its spaces are what
/// `Char.isWhitespace` says, not what a regex's `\s` matches, and it is lowered a unit at a time
/// (`lowercaseChar`), so İ is i here where `needle` has i and a combining dot.
fn index_in(text: &[u16], needle: &[u16]) -> Option<usize> {
    let mut folded = Vec::with_capacity(text.len());
    let mut origin = Vec::with_capacity(text.len());
    let mut last_space = false;
    for (index, &unit) in text.iter().enumerate() {
        let space = is_whitespace(unit);
        if space && last_space {
            continue;
        }
        origin.push(index);
        folded.push(if space { SPACE } else { to_lower(unit) });
        last_space = space;
    }
    // `needle` is never empty: it is a line that isn't, normalised. Were it, nothing matches.
    let at = (folded.windows(needle.len().max(1))).position(|it| it == needle)?;
    Some(origin[at])
}

/// Where to carry on in `blocks`, the screen's text after a scroll, given the texts `read` so far:
/// just past the last few blocks read, found in order. If none of them is there any more (the
/// scroll went further than the screen), the first block that wasn't read recently.
/// `blocks.len()` means there is nothing new.
pub fn resume_after(blocks: &[&[u16]], read: &[&[u16]]) -> usize {
    if read.is_empty() {
        return 0;
    }
    for length in (1..=ANCHOR.min(read.len())).rev() {
        let anchor = &read[read.len() - length..];
        if let Some(start) = blocks.windows(length).rposition(|it| it == anchor) {
            return start + length;
        }
    }
    let recent: HashSet<&[u16]> = read[read.len().saturating_sub(RECENT)..]
        .iter()
        .copied()
        .collect();
    (blocks.iter())
        .position(|it| !recent.contains(it))
        .unwrap_or(blocks.len())
}

/// The text these are the `sentences` of, cut into pieces the speech engine takes: sentences,
/// joined while they fit in `max` units, and a sentence longer than that cut at a space.
pub fn utterances(sentences: &[&[u16]], max: NonZeroUsize) -> Vec<Vec<u16>> {
    let max = max.get();
    let mut pieces = Vec::new();
    let mut current = Vec::new();
    for &sentence in sentences {
        if !current.is_empty() && current.len() + sentence.len() > max {
            pieces.push(trim(&current).to_vec());
            current.clear();
        }
        if sentence.len() > max {
            split_long(sentence, max, &mut pieces);
        } else {
            current.extend_from_slice(sentence);
        }
    }
    if !is_blank(&current) {
        pieces.push(trim(&current).to_vec());
    }
    pieces.retain(|it| !is_blank(it));
    pieces
}

/// `sentence`, longer than `max`, cut into pieces no longer, added to `out`: each at the last
/// space that leaves at most `max` units before it, else after `max` units.
fn split_long(sentence: &[u16], max: usize, out: &mut Vec<Vec<u16>>) {
    let mut rest = trim(sentence);
    while rest.len() > max {
        let cut = (rest[..=max].iter().rposition(|&it| it == SPACE))
            .filter(|&it| it > 0)
            .unwrap_or(max);
        out.push(trim(&rest[..cut]).to_vec());
        rest = trim(&rest[cut..]);
    }
    if !rest.is_empty() {
        out.push(rest.to_vec());
    }
}

#[cfg(test)]
mod tests;
