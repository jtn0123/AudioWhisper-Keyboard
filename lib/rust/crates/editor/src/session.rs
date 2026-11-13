use crate::{Content, Field, Range, attributes, integer, number, operations::Op, text};
use std::collections::VecDeque;

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
        let mut output = vec![text("ready"), number(i32::from(self.result))];
        for op in &self.operations {
            output.extend(op.fields());
        }
        output
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
                return vec![text("word"), c.before(), number(self.last - c.offset)];
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
            if op.name == "newComposing" {
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
            "extracted" => self.extracted(fields),
            "config" => {
                self.raw = integer(fields, 0) != 0;
                self.enabled = integer(fields, 1) != 0;
                self.determine = integer(fields, 2) != 0;
                self.space = integer(fields, 3) != 0;
                self.input_type = integer(fields, 4);
                vec![]
            }
            "arrow" => self.arrow(fields),
            "massBegin" | "massEnd" | "massState" | "massReset" => self.mass_state(operation),
            "inputInfo" => {
                self.active = Content::default();
                self.caps = attributes::caps(integer(fields, 0));
                vec![]
            }
            "unavailable" => {
                if fields.first().is_some_and(|s| {
                    matches!(
                        String::from_utf16_lossy(s).as_str(),
                        "start" | "update" | "mass" | "extracted"
                    )
                }) {
                    self.active = Content::default();
                    self.caps = 0;
                }
                self.done(false, vec![])
            }
            "published" => {
                let mut out = self.active.fields();
                out.extend([
                    number(self.caps),
                    number(self.last),
                    number(0),
                    number(self.failures as i32),
                ]);
                out
            }
            "snapshot" => {
                let mut out = self.content().fields();
                out.extend([
                    number(self.caps),
                    number(self.last),
                    number(i32::from(!self.queue.is_empty())),
                    number(self.failures as i32),
                ]);
                out
            }
            "reset" => {
                *self = Self::new();
                self.done(true, vec![])
            }
            "start" => {
                let selection = Range::read(fields, 0);
                if self.raw || !selection.valid() || integer(fields, 2) == 0 {
                    self.active = Content::default();
                    self.caps = 0;
                    return self.done(false, vec![]);
                }
                self.query = Query::Read(selection, None, true);
                vec![text("read"), number(1), number(1)]
            }
            "update" => {
                if self.mass > 0 {
                    return self.call("mass", &fields[..4]);
                }
                let selection = Range::read(fields, 0);
                let composing = Range::read(fields, 2);
                if self.raw || !selection.valid() || integer(fields, 4) == 0 {
                    self.active = Content::default();
                    self.caps = 0;
                    return self.done(false, vec![]);
                }
                if selection.start.min(selection.end) < self.last {
                    self.last = -1;
                }
                while let Some(expected) = self.queue.pop_front() {
                    if expected.selection() == selection
                        && expected.composing() == composing
                        && expected.before().len() as i32 >= 128.min(selection.start)
                    {
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
                vec![
                    text("read"),
                    number(i32::from(selection.start > 0)),
                    number(i32::from(selection.start != selection.end)),
                ]
            }
            "read" => {
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
                    vec![Op::simple("newComposing")]
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
            "word" => {
                let p = self.pending.as_mut().expect("word response");
                p.content.word = Range::read(fields, 0);
                p.content.composing = if self.enabled {
                    p.content.word
                } else {
                    Range::NONE
                };
                self.advance()
            }
            "caps" => {
                self.caps = attributes::caps(integer(fields, 0));
                self.advance()
            }
            "mass" => {
                let selection = Range::read(fields, 0);
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
                    if Range::read(fields, 2).valid() {
                        vec![Op::simple("finish")]
                    } else {
                        vec![]
                    },
                )
            }
            "lastCommit" => {
                let s = self.content().selection();
                self.last = if s.valid() { s.start.max(s.end) } else { -1 };
                vec![]
            }
            "lastDelete" => {
                let s = self.content().selection();
                if s.start.min(s.end) < self.last {
                    self.last = s.start.min(s.end);
                }
                vec![]
            }
            "results" => {
                self.failures += fields
                    .iter()
                    .zip(&self.operations)
                    .filter(|(v, op)| v.as_slice() == [48] && !matches!(op.name, "begin" | "end"))
                    .count();
                let success = if self
                    .operations
                    .last()
                    .is_some_and(|op| matches!(op.name, "action" | "contextMenu"))
                {
                    self.result && fields.last().is_some_and(|v| v.as_slice() == [49])
                } else {
                    self.result
                };
                vec![number(i32::from(success))]
            }
            "context" | "spaces" | "spacing" | "typeChar" | "typeText" | "completion"
            | "gesture" | "backward" | "forward" | "revert" | "dictation" | "undoDictation"
            | "forgetDictation" | "doubleSpace" | "rawEnter" | "enter" | "selectAll"
            | "selectAround" | "extent" => self.high(operation, fields),
            _ => self.edit(operation, fields),
        }
    }
}
