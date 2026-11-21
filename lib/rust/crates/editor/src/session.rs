use crate::{
    Content, Field, Range, attributes, integer, number,
    operations::{NEW_COMPOSING, Op},
    text,
};
use std::collections::VecDeque;
use wire::editor::{self as ops, config, cursor, effect, plan, read_query, state, word_query};
use wire::flag;

#[derive(Clone, Debug)]
pub(crate) struct Pending {
    pub content: Content,
    pub ops: Vec<Op>,
    pub publish: bool,
    pub commit: bool,
    pub delete: bool,
    pub compare: Option<Range>,
    pub word_done: bool,
    pub caps_done: bool,
}
#[derive(Clone, Debug, Default)]
pub(crate) enum Query {
    Read(Range, Option<Range>, bool),
    Delete(bool),
    Composer(Field, [bool; 3]),
    #[default]
    None,
}

#[derive(Debug, Default)]
pub struct Session {
    pub(crate) active: Content,
    pub(crate) queue: VecDeque<Content>,
    pub(crate) last: i32,
    pub(crate) caps: i32,
    pub(crate) raw: bool,
    pub(crate) enabled: bool,
    pub(crate) determine: bool,
    pub(crate) space: bool,
    pub(crate) input_type: i32,
    pub(crate) pending: Option<Pending>,
    pub(crate) query: Query,
    pub(crate) result: bool,
    pub(crate) operations: Vec<Op>,
    pub(crate) failures: usize,
    pub(crate) mass: i32,
    pub(crate) composer: Option<composing::rules::Rules>,
    pub(crate) composer_read: usize,
    pub(crate) context: crate::high::Context,
}
impl Session {
    pub fn new() -> Self {
        Self {
            last: -1,
            ..Self::default()
        }
    }
    pub(crate) fn content(&self) -> &Content {
        self.queue.back().unwrap_or(&self.active)
    }
    pub(crate) fn ready(&self) -> Vec<Field> {
        let mut output = vec![vec![]; plan::SIZE];
        output[plan::STATUS] = text(ops::READY);
        output[plan::RESULT] = flag::utf16(self.result);
        for op in &self.operations {
            output.extend(op.fields());
        }
        output
    }
    /// The state of `content`, and whether an expected state is waiting to be reported.
    fn state(&self, content: &Content, expected: bool) -> Vec<Field> {
        let mut row = content.fields();
        row[state::CAPS] = number(self.caps);
        row[state::LAST_COMMIT] = number(self.last);
        row[state::EXPECTED] = flag::utf16(expected);
        row[state::FAILURES] = number(self.failures as i32);
        row
    }
    pub(crate) fn done(&mut self, success: bool, ops: Vec<Op>) -> Vec<Field> {
        if let Some((event, args)) = self.context.next.take() {
            self.context.prefix.extend(ops);
            return self.high(&event, &args);
        }
        let mut combined = std::mem::take(&mut self.context.prefix);
        combined.extend(ops);
        self.result = success;
        self.operations = combined;
        self.ready()
    }
    pub(crate) fn generated(
        &mut self,
        content: Content,
        ops: Vec<Op>,
        publish: bool,
        commit: bool,
        delete: bool,
        compare: Option<Range>,
    ) -> Vec<Field> {
        self.pending = Some(Pending {
            content,
            ops,
            publish,
            commit,
            delete,
            compare,
            word_done: false,
            caps_done: false,
        });
        self.advance()
    }
    fn advance(&mut self) -> Vec<Field> {
        let pending = self.pending.as_mut().expect("pending content");
        let c = &mut pending.content;
        if !pending.word_done {
            pending.word_done = true;
            if self.determine && c.selection.cursor_mode() && !c.before().is_empty() {
                let mut query = vec![vec![]; word_query::SIZE];
                query[word_query::QUERY] = text(ops::WORD);
                query[word_query::BEFORE] = c.before();
                query[word_query::LAST_COMMIT] = number(self.last - c.offset);
                return query;
            }
        }
        if !pending.caps_done && pending.publish {
            pending.caps_done = true;
            if c.selection.valid() {
                let mut value = c.text.clone();
                let mut cursor = c.selection.start;
                if self.space {
                    value = c.before();
                    value.push(32);
                    cursor += 1;
                }
                self.caps = attributes::caps(crate::caps::flags(&value, cursor, self.input_type));
            } else {
                self.caps = 0;
            }
        }
        let pending = self.pending.take().expect("pending content");
        let c = pending.content;
        if pending.commit {
            self.last = if c.selection().valid() {
                c.selection().start.max(c.selection().end)
            } else {
                -1
            };
        }
        if pending.delete && c.selection().start.min(c.selection().end) < self.last {
            self.last = c.selection().start.min(c.selection().end);
        }
        let mut ops = pending.ops;
        for op in &mut ops {
            if op.name == NEW_COMPOSING {
                *op = Op::composing(c.composing());
            }
        }
        if let Some(composing) = pending.compare {
            if c.composing() != composing {
                ops.push(Op::composing(c.composing()));
            } else if !pending.publish {
                return self.done(true, vec![]);
            }
        }
        if pending.publish {
            self.active = c;
        } else {
            self.queue.push_back(c);
        }
        self.done(true, ops)
    }
    pub fn call(&mut self, operation: &str, fields: &[Field]) -> Vec<Field> {
        match operation {
            ops::EXTRACTED => self.extracted(fields),
            ops::CONFIG => {
                self.raw = flag::decode(&fields[config::RAW]);
                self.enabled = flag::decode(&fields[config::COMPOSING]);
                self.determine = flag::decode(&fields[config::DETERMINE_COMPOSING]);
                self.space = flag::decode(&fields[config::SPACE_PENDING]);
                self.input_type = integer(fields, config::INPUT_TYPE);
                vec![]
            }
            ops::COMPOSER_RULES => {
                self.composer_read = fields
                    .as_chunks::<2>()
                    .0
                    .iter()
                    .map(|p| p[0].len().saturating_sub(1))
                    .max()
                    .unwrap_or(0);
                self.composer = Some(composing::rules::Rules::new(
                    fields
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|p| (p[0].clone(), p[1].clone()))
                        .collect(),
                ));
                vec![]
            }
            ops::ARROW => self.arrow(fields),
            ops::MASS_BEGIN | ops::MASS_END | ops::MASS_STATE | ops::MASS_RESET => {
                self.mass_state(operation)
            }
            ops::INPUT_INFO => {
                self.active = Content::default();
                self.caps = attributes::caps(integer(fields, 0));
                vec![]
            }
            ops::UNAVAILABLE => {
                if fields.first().is_some_and(|s| {
                    matches!(
                        String::from_utf16_lossy(s).as_str(),
                        ops::START | ops::UPDATE | ops::MASS | ops::EXTRACTED
                    )
                }) {
                    self.active = Content::default();
                    self.caps = 0;
                }
                self.done(false, vec![])
            }
            ops::PUBLISHED => self.state(&self.active, false),
            ops::SNAPSHOT => self.state(self.content(), !self.queue.is_empty()),
            ops::RESET => {
                let composer = self.composer.take();
                let composer_read = self.composer_read;
                *self = Self::new();
                self.composer = composer;
                self.composer_read = composer_read;
                self.done(true, vec![])
            }
            ops::START => {
                let selection = Range::read(fields, 0);
                if self.raw || !selection.valid() || !flag::decode(&fields[2]) {
                    self.active = Content::default();
                    self.caps = 0;
                    return self.done(false, vec![]);
                }
                self.query = Query::Read(selection, None, true);
                read_query(true, true)
            }
            ops::LATE => {
                // An update reporting one of the keyboard's own edits that it has already followed
                // with more: news of the past, which must not undo what those later edits set,
                // such as the space an undone auto-correction owes. Asked before ops::UPDATE.
                let selection = Range::read(fields, cursor::SELECTION_START);
                let composing = Range::read(fields, cursor::COMPOSING_START);
                let own = (self.queue.iter()).position(|it| it.reported_by(selection, composing));
                let late = self.mass == 0 && own.is_some_and(|at| at + 1 < self.queue.len());
                vec![flag::utf16(late)]
            }
            ops::UPDATE => {
                if self.mass > 0 {
                    return self.call(ops::MASS, &fields[..cursor::SIZE]);
                }
                let selection = Range::read(fields, cursor::SELECTION_START);
                let composing = Range::read(fields, cursor::COMPOSING_START);
                if self.raw || !selection.valid() || !flag::decode(&fields[cursor::SIZE]) {
                    self.active = Content::default();
                    self.caps = 0;
                    return self.done(false, vec![]);
                }
                if selection.start.min(selection.end) < self.last {
                    self.last = -1;
                }
                while let Some(expected) = self.queue.pop_front() {
                    if expected.reported_by(selection, composing) {
                        let (mut value, mut cursor) =
                            (expected.text.clone(), expected.selection.start);
                        if self.space {
                            value = expected.before();
                            value.push(32);
                            cursor += 1;
                        }
                        self.caps =
                            attributes::caps(crate::caps::flags(&value, cursor, self.input_type));
                        self.active = expected;
                        return self.done(true, vec![]);
                    }
                }
                self.query = Query::Read(selection, Some(composing), false);
                read_query(selection.start > 0, selection.start != selection.end)
            }
            ops::READ => {
                let Query::Read(mut selection, compare, initial) = std::mem::take(&mut self.query)
                else {
                    return self.done(false, vec![]);
                };
                let before = &fields[0];
                let after = &fields[1];
                let selected = &fields[2];
                if initial {
                    if (selection.end - selection.start).abs() != selected.len() as i32 {
                        selection = Range {
                            start: before.len() as i32,
                            end: (before.len() + selected.len()) as i32,
                        };
                    } else if (selection.start > 0 || selection.end > 0)
                        && before.is_empty()
                        && after.is_empty()
                        && selected.is_empty()
                    {
                        selection = Range::cursor(0);
                    }
                }
                let ops = if initial {
                    vec![Op::simple(NEW_COMPOSING)]
                } else {
                    vec![]
                };
                self.generated(
                    Content::generated(selection, before, after, selected),
                    ops,
                    true,
                    false,
                    false,
                    compare,
                )
            }
            ops::WORD => {
                let p = self.pending.as_mut().expect("word response");
                p.content.word = Range::read(fields, 0);
                p.content.composing = if self.enabled {
                    p.content.word
                } else {
                    Range::NONE
                };
                self.advance()
            }
            ops::CAPS => {
                self.caps = attributes::caps(integer(fields, 0));
                self.advance()
            }
            ops::MASS => {
                let selection = Range::read(fields, cursor::SELECTION_START);
                self.caps = 0;
                self.active = Content {
                    selection,
                    ..Content::default()
                };
                if selection.start.min(selection.end) < self.last {
                    self.last = -1;
                }
                self.done(
                    true,
                    if Range::read(fields, cursor::COMPOSING_START).valid() {
                        vec![Op::simple(effect::FINISH)]
                    } else {
                        vec![]
                    },
                )
            }
            ops::LAST_COMMIT => {
                let s = self.content().selection();
                self.last = if s.valid() { s.start.max(s.end) } else { -1 };
                vec![]
            }
            ops::LAST_DELETE => {
                let s = self.content().selection();
                if s.start.min(s.end) < self.last {
                    self.last = s.start.min(s.end);
                }
                vec![]
            }
            ops::RESULTS => {
                self.failures += fields
                    .iter()
                    .zip(&self.operations)
                    .filter(|(v, op)| {
                        !flag::decode(v) && !matches!(op.name, effect::BEGIN | effect::END)
                    })
                    .count();
                let success = if self
                    .operations
                    .last()
                    .is_some_and(|op| matches!(op.name, effect::ACTION | effect::CONTEXT_MENU))
                {
                    self.result && fields.last().is_some_and(|v| flag::decode(v))
                } else {
                    self.result
                };
                vec![flag::utf16(success)]
            }
            ops::CONTEXT
            | ops::SPACES
            | ops::SPACING
            | ops::TYPE_CHAR
            | ops::TYPE_TEXT
            | ops::COMPLETION
            | ops::GESTURE
            | ops::BACKWARD
            | ops::FORWARD
            | ops::REVERT
            | ops::DICTATION
            | ops::UNDO_DICTATION
            | ops::FORGET_DICTATION
            | ops::DOUBLE_SPACE
            | ops::RAW_ENTER
            | ops::ENTER
            | ops::SELECT_ALL
            | ops::SELECT_AROUND
            | ops::EXTENT => self.high(operation, fields),
            _ => self.edit(operation, fields),
        }
    }
}

/// A query for the editor's text, and whether to read the text before the cursor and the selected
/// text.
pub(crate) fn read_query(before: bool, selected: bool) -> Vec<Field> {
    let mut query = vec![vec![]; read_query::SIZE];
    query[read_query::QUERY] = text(ops::READ);
    query[read_query::BEFORE] = flag::utf16(before);
    query[read_query::SELECTED] = flag::utf16(selected);
    query
}

#[cfg(test)]
mod tests;
