use crate::{
    Content, Field, Range, Session, integer, number,
    operations::{Op, keys},
    session::Query,
    text,
};
impl Session {
    pub(crate) fn edit(&mut self, operation: &str, fields: &[Field]) -> Vec<Field> {
        let c = self.content().clone();
        let s = c.selection();
        match operation {
            "select" => {
                let selection = Range::read(fields, 0);
                if self.raw {
                    return self.done(false, vec![]);
                }
                if s == selection {
                    return self.done(true, vec![]);
                }
                let local = selection.shifted(-c.offset);
                let changed = Content {
                    selection: local,
                    ..c.clone()
                };
                self.generated(
                    Content::generated(
                        selection,
                        &changed.before(),
                        &changed.after(),
                        &changed.selected(),
                    ),
                    vec![
                        Op::simple("begin"),
                        Op::new("selection", selection.start, selection.end, &[]),
                        Op::simple("newComposing"),
                        Op::simple("end"),
                    ],
                    false,
                    false,
                    false,
                    None,
                )
            }
            "commit" => self.commit(&fields[0]),
            "char" => {
                self.query = Query::Composer(
                    fields[0].clone(),
                    [
                        integer(fields, 1) != 0,
                        integer(fields, 2) != 0,
                        integer(fields, 3) != 0,
                    ],
                );
                vec![text("character"), fields[0].clone()]
            }
            "character" => {
                let Query::Composer(value, spaces) = self.query.clone() else {
                    return self.done(false, vec![]);
                };
                if integer(fields, 0) != value.len() as i32 || !s.cursor_mode() || self.raw {
                    self.query = Query::None;
                    return self.commit(&value);
                }
                vec![
                    text("composer"),
                    c.before(),
                    value,
                    number(i32::from(spaces[0])),
                ]
            }
            "composer" => {
                let Query::Composer(_, spaces) = std::mem::take(&mut self.query) else {
                    return self.done(false, vec![]);
                };
                let rm =
                    integer(fields, 0) + i32::from(spaces[0] && c.before().last() == Some(&32));
                let mut value = fields[1].clone();
                if spaces[1] {
                    value.insert(0, 32);
                }
                if spaces[2] {
                    value.push(32);
                }
                if rm <= 0 {
                    return self.commit(&value);
                }
                let mut before = c.before();
                before.truncate(before.len().saturating_sub(rm as usize));
                before.extend(&value);
                self.generated(
                    Content::generated(
                        Range::cursor(s.start.wrapping_sub(rm).wrapping_add(value.len() as i32)),
                        &before,
                        &c.after(),
                        &[],
                    ),
                    vec![
                        Op::simple("begin"),
                        Op::new("composing", s.start - rm, s.start, &[]),
                        Op::new("composingText", 1, 0, &value),
                        Op::simple("newComposing"),
                        Op::simple("end"),
                    ],
                    false,
                    false,
                    false,
                    None,
                )
            }
            "finalize" => {
                let value = &fields[0];
                let composing = c.composing();
                if self.raw || !composing.valid() {
                    return self.done(false, vec![Op::simple("begin")]);
                }
                let old = c.slice(c.composing);
                let mut before = c.before();
                if before.ends_with(&old) {
                    before.truncate(before.len() - old.len());
                }
                before.extend(value);
                self.generated(
                    Content::generated(
                        Range::cursor(
                            composing
                                .end
                                .wrapping_add((value.len() as i32).wrapping_sub(old.len() as i32)),
                        ),
                        &before,
                        &c.after(),
                        &[],
                    ),
                    vec![
                        Op::simple("begin"),
                        Op::new("composingText", 1, 0, value),
                        Op::simple("finish"),
                        Op::simple("end"),
                    ],
                    false,
                    true,
                    false,
                    None,
                )
            }
            "replace" => {
                let length = integer(fields, 0);
                let value = &fields[1];
                let mut before = c.before();
                if self.raw || !s.cursor_mode() || length < 0 || before.len() < length as usize {
                    return self.done(false, vec![]);
                }
                before.truncate(before.len() - length as usize);
                before.extend(value);
                self.generated(
                    Content::generated(
                        Range::cursor(
                            s.start
                                .wrapping_sub(length)
                                .wrapping_add(value.len() as i32),
                        ),
                        &before,
                        &c.after(),
                        &c.selected(),
                    ),
                    vec![
                        Op::simple("begin"),
                        Op::simple("finish"),
                        Op::new("delete", length, 0, &[]),
                        Op::new("commit", 1, 0, value),
                        Op::simple("newComposing"),
                        Op::simple("end"),
                    ],
                    false,
                    true,
                    false,
                    None,
                )
            }
            "delete" => {
                let after = integer(fields, 0) != 0;
                let words = integer(fields, 1) != 0;
                let n = integer(fields, 2);
                if n < 1 {
                    return self.done(false, vec![]);
                }
                let value = if after { c.after() } else { c.before() };
                if self.raw || value.is_empty() {
                    let output = self.done(
                        true,
                        keys(
                            if after { 112 } else { 67 },
                            if words { 0x3000 } else { 0 },
                            n,
                        ),
                    );
                    if s.start.min(s.end) < self.last {
                        self.last = s.start.min(s.end);
                    }
                    return output;
                }
                self.query = Query::Delete(after);
                vec![
                    text("measure"),
                    value,
                    number(i32::from(after)),
                    number(i32::from(words)),
                    number(n),
                ]
            }
            "measure" => {
                let Query::Delete(after) = std::mem::take(&mut self.query) else {
                    return self.done(false, vec![]);
                };
                let length = integer(fields, 0).max(0);
                let mut before = c.before();
                let mut following = c.after();
                if after {
                    following = following[(length as usize).min(following.len())..].to_vec();
                } else {
                    before.truncate(before.len().saturating_sub(length as usize));
                }
                let selection = s.shifted(if after { length } else { -length });
                self.generated(
                    Content::generated(selection, &before, &following, &c.selected()),
                    vec![
                        Op::simple("begin"),
                        Op::simple("finish"),
                        Op::new(
                            "delete",
                            if after { 0 } else { length },
                            if after { length } else { 0 },
                            &[],
                        ),
                        Op::simple("newComposing"),
                        Op::simple("end"),
                    ],
                    false,
                    false,
                    true,
                    None,
                )
            }
            "refresh" => {
                if self.raw {
                    return self.done(false, vec![]);
                }
                self.generated(
                    Content::generated(s, &c.before(), &c.after(), &c.selected()),
                    vec![],
                    false,
                    false,
                    false,
                    Some(c.composing()),
                )
            }
            "keys" => self.done(
                integer(fields, 2) > 0,
                keys(integer(fields, 0), integer(fields, 1), integer(fields, 2)),
            ),
            "action" => self.done(true, vec![Op::new("action", integer(fields, 0), 0, &[])]),
            _ => self.done(false, vec![]),
        }
    }
    fn commit(&mut self, value: &[u16]) -> Vec<Field> {
        let c = self.content().clone();
        let mut ops = vec![
            Op::simple("begin"),
            Op::simple("finish"),
            Op::new("commit", 1, 0, value),
        ];
        if self.raw {
            ops.push(Op::simple("end"));
            return self.done(true, ops);
        }
        let mut before = c.before();
        before.extend(value);
        ops.extend([Op::simple("newComposing"), Op::simple("end")]);
        self.generated(
            Content::generated(
                Range::cursor(c.selection().start.wrapping_add(value.len() as i32)),
                &before,
                &c.after(),
                &[],
            ),
            ops,
            false,
            false,
            false,
            None,
        )
    }
}
