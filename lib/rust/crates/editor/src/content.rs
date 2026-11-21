use crate::{Field, integer, number};
use wire::editor::state;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: i32,
    pub end: i32,
}
impl Default for Range {
    fn default() -> Self {
        Self::NONE
    }
}
impl Range {
    pub const NONE: Self = Self { start: -1, end: -1 };
    pub fn cursor(pos: i32) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }
    pub fn valid(self) -> bool {
        self.start >= 0 && self.end >= 0
    }
    pub fn cursor_mode(self) -> bool {
        self.valid() && self.start == self.end
    }
    pub fn shifted(self, offset: i32) -> Self {
        if self.valid() {
            Self {
                start: self.start.wrapping_add(offset),
                end: self.end.wrapping_add(offset),
            }
        } else {
            Self::NONE
        }
    }
    pub fn read(fields: &[Field], at: usize) -> Self {
        Self {
            start: integer(fields, at),
            end: integer(fields, at + 1),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    pub text: Field,
    pub offset: i32,
    pub selection: Range,
    pub composing: Range,
    pub word: Range,
}
impl Default for Content {
    fn default() -> Self {
        Self {
            text: vec![],
            offset: -1,
            selection: Range::NONE,
            composing: Range::NONE,
            word: Range::NONE,
        }
    }
}
impl Content {
    pub fn selection(&self) -> Range {
        if self.offset > 0 {
            self.selection.shifted(self.offset)
        } else {
            self.selection
        }
    }
    pub fn composing(&self) -> Range {
        if self.offset > 0 {
            self.composing.shifted(self.offset)
        } else {
            self.composing
        }
    }
    /// Whether a cursor update reporting `selection` and `composing` is this content's: the
    /// keyboard's own edit, arrived, with enough text before it known to take as the editor's.
    pub fn reported_by(&self, selection: Range, composing: Range) -> bool {
        self.selection() == selection
            && self.composing() == composing
            && self.before().len() as i32 >= 128.min(selection.start)
    }
    pub fn slice(&self, range: Range) -> Field {
        if !range.valid() {
            return vec![];
        }
        self.text
            .get(range.start as usize..range.end as usize)
            .unwrap_or_default()
            .to_vec()
    }
    pub fn before(&self) -> Field {
        self.slice(Range {
            start: 0,
            end: self.selection.start,
        })
    }
    pub fn after(&self) -> Field {
        self.slice(Range {
            start: self.selection.end,
            end: self.text.len() as i32,
        })
    }
    pub fn selected(&self) -> Field {
        self.slice(self.selection)
    }
    pub fn generated(selection: Range, before: &[u16], after: &[u16], selected: &[u16]) -> Self {
        let offset = selection.start - before.len() as i32;
        let local = Range {
            start: before.len() as i32,
            end: (before.len() + selected.len()) as i32,
        };
        if offset < 0 || selection.shifted(-offset) != local {
            return Self::default();
        }
        Self {
            text: [before, selected, after].concat(),
            offset,
            selection: local,
            ..Self::default()
        }
    }
    /// A `wire::editor::state` row with this content's fields, the rest empty.
    pub fn fields(&self) -> Vec<Field> {
        let mut row = vec![vec![]; state::SIZE];
        row[state::TEXT] = self.text.clone();
        row[state::OFFSET] = number(self.offset);
        row[state::SELECTION_START] = number(self.selection.start);
        row[state::SELECTION_END] = number(self.selection.end);
        row[state::COMPOSING_START] = number(self.composing.start);
        row[state::COMPOSING_END] = number(self.composing.end);
        row[state::WORD_START] = number(self.word.start);
        row[state::WORD_END] = number(self.word.end);
        row
    }
}
