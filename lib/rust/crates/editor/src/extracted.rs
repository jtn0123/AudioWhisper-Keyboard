//! ExtractedText's selection is local to startOffset. Partial reports require a fresh IC window.
use crate::{Content, Field, Range, Session, integer, session::Query, session::read_query};
use wire::editor::extracted::{
    COMPOSING_START, PARTIAL_START, SELECTION_START, START_OFFSET, TEXT,
};
impl Session {
    pub(crate) fn extracted(&mut self, fields: &[Field]) -> Vec<Field> {
        self.queue.clear();
        let offset = integer(fields, START_OFFSET);
        let local = Range::read(fields, SELECTION_START);
        let selection = Range {
            start: offset.wrapping_add(local.start),
            end: offset.wrapping_add(local.end),
        };
        let composing = Range::read(fields, COMPOSING_START).shifted(offset);
        if self.raw || offset < 0 || !local.valid() {
            self.active = Content::default();
            self.caps = 0;
            return self.done(false, vec![]);
        }
        if selection.start.min(selection.end) < self.last {
            self.last = -1;
        }
        if integer(fields, PARTIAL_START) >= 0 {
            self.query = Query::Read(selection, Some(composing), false);
            return read_query(selection.start > 0, selection.start != selection.end);
        }
        let value = &fields[TEXT];
        let a = local.start.min(local.end) as usize;
        let b = local.start.max(local.end) as usize;
        if b > value.len() {
            self.active = Content::default();
            self.caps = 0;
            return self.done(false, vec![]);
        }
        let selection = Range {
            start: selection.start.min(selection.end),
            end: selection.start.max(selection.end),
        };
        let before = &value[a.saturating_sub(256)..a];
        let after = &value[b..(b + 128).min(value.len())];
        self.generated(
            Content::generated(selection, before, after, &value[a..b]),
            vec![],
            true,
            false,
            false,
            Some(composing),
        )
    }
}
