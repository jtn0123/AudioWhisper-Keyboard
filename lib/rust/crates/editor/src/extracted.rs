//! ExtractedText's selection is local to startOffset. Partial reports require a fresh IC window.
use crate::{Content, Field, Range, Session, integer, number, session::Query, text};
impl Session {
    pub(crate) fn extracted(&mut self, fields: &[Field]) -> Vec<Field> {
        self.queue.clear();
        let offset = integer(fields, 2);
        let local = Range::read(fields, 0);
        let selection = Range {
            start: offset.wrapping_add(local.start),
            end: offset.wrapping_add(local.end),
        };
        let composing = Range::read(fields, 6).shifted(offset);
        if self.raw || offset < 0 || !local.valid() {
            self.active = Content::default();
            self.caps = 0;
            return self.done(false, vec![]);
        }
        if selection.start.min(selection.end) < self.last {
            self.last = -1;
        }
        if integer(fields, 3) >= 0 {
            self.query = Query::Read(selection, Some(composing), false);
            return vec![
                text("read"),
                number(i32::from(selection.start > 0)),
                number(i32::from(selection.start != selection.end)),
            ];
        }
        let value = &fields[5];
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
