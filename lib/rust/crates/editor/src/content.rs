use crate::{Field, integer, number};

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
    pub fn fields(&self) -> Vec<Field> {
        vec![
            self.text.clone(),
            number(self.offset),
            number(self.selection.start),
            number(self.selection.end),
            number(self.composing.start),
            number(self.composing.end),
            number(self.word.start),
            number(self.word.end),
        ]
    }
}
