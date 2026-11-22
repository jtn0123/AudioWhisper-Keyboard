//! Text boundaries for the selection and for TalkBack's traversal: ICU word segments, as
//! Android's `WordIterator` finds them, grapheme clusters, paragraphs, and UTF-16 offsets.
use icu_segmenter::{WordSegmenter, options::WordBreakInvariantOptions};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

/// A text's word segments in order: each range and whether it is a word (letters or digits)
/// rather than spaces or punctuation.
#[derive(Debug, Default)]
pub struct Words(Vec<(Range<usize>, bool)>);
impl Words {
    pub fn new(text: &str) -> Self {
        let segmenter =
            WordSegmenter::new_for_non_complex_scripts(WordBreakInvariantOptions::default());
        let mut segments = vec![];
        let mut start = 0;
        for (end, kind) in segmenter.segment_str(text).iter_with_word_type() {
            if end > start {
                segments.push((start..end, kind.is_word_like()));
            }
            start = end;
        }
        Self(segments)
    }
    /// The word a long press at `offset` selects, as Compose's `getWordBoundary` finds it: the
    /// word the offset is in or at the start of, else the word ending at it. Between words it is
    /// the one character there, as `ensureAtLeastOneChar` keeps a selection from collapsing.
    pub fn at(&self, text: &str, offset: usize) -> Range<usize> {
        let i = self.0.partition_point(|(r, _)| r.end <= offset);
        if let Some((r, true)) = self.0.get(i)
            && r.start <= offset
        {
            return r.clone();
        }
        if let Some((r, true)) = i.checked_sub(1).and_then(|j| self.0.get(j))
            && r.end == offset
        {
            return r.clone();
        }
        match next_char(text, offset) {
            Some(end) => offset..end,
            None => prev_char(text, offset).map_or(offset..offset, |start| start..offset),
        }
    }
    /// TalkBack's next word: the first word starting at or after `offset`.
    pub fn following(&self, offset: usize) -> Option<Range<usize>> {
        self.0
            .iter()
            .find(|(r, word)| *word && r.start >= offset)
            .map(|(r, _)| r.clone())
    }
    /// TalkBack's previous word: the last word ending at or before `offset`.
    pub fn preceding(&self, offset: usize) -> Option<Range<usize>> {
        self.0
            .iter()
            .rev()
            .find(|(r, word)| *word && r.end <= offset)
            .map(|(r, _)| r.clone())
    }
}
/// The grapheme boundary after `offset`, if any.
pub fn next_char(text: &str, offset: usize) -> Option<usize> {
    text.get(offset..)?
        .graphemes(true)
        .next()
        .map(|g| offset + g.len())
}
/// The grapheme boundary before `offset`, if any.
pub fn prev_char(text: &str, offset: usize) -> Option<usize> {
    text.get(..offset)?
        .graphemes(true)
        .next_back()
        .map(|g| offset - g.len())
}
/// TalkBack's next paragraph, as Compose's `ParagraphTextSegmentIterator` finds it: newlines at
/// `offset` are passed over, then the paragraph runs to the next newline.
pub fn next_paragraph(text: &str, offset: usize) -> Option<Range<usize>> {
    let start = offset
        + text
            .get(offset..)?
            .bytes()
            .take_while(|b| *b == b'\n')
            .count();
    if start >= text.len() {
        return None;
    }
    let end = text[start..].find('\n').map_or(text.len(), |at| start + at);
    Some(start..end)
}
/// TalkBack's previous paragraph: newlines before `offset` are passed over, then the paragraph
/// runs back to the newline before it.
pub fn prev_paragraph(text: &str, offset: usize) -> Option<Range<usize>> {
    let head = text.get(..offset)?;
    let end = head.trim_end_matches('\n').len();
    if end == 0 {
        return None;
    }
    let start = text[..end].rfind('\n').map_or(0, |at| at + 1);
    Some(start..end)
}
/// A byte offset as UTF-16 units, the unit Android's text APIs count in.
pub fn utf16(text: &str, byte: usize) -> usize {
    text.get(..byte.min(text.len()))
        .map_or(0, |head| head.encode_utf16().count())
}
/// A UTF-16 offset as a byte offset, at the character boundary it falls on or after.
pub fn byte(text: &str, unit: usize) -> usize {
    let mut units = 0;
    for (at, ch) in text.char_indices() {
        if units >= unit {
            return at;
        }
        units += ch.len_utf16();
    }
    text.len()
}
