use crate::{
    Content, Field, Range, Session, integer, number,
    operations::{NEW_COMPOSING, Op, keys},
    session::Query,
    text,
};
use wire::editor::{self as ops, char_input, effect, measure, text_query};
use wire::flag;
impl Session {
    pub(crate) fn edit(&mut self, operation: &str, fields: &[Field]) -> Vec<Field> {
        let c = self.content().clone();
        let s = c.selection();
        match operation {
            ops::SELECT => {
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
                        Op::simple(effect::BEGIN),
                        Op::new(effect::SELECTION, selection.start, selection.end, &[]),
                        Op::simple(NEW_COMPOSING),
                        Op::simple(effect::END),
                    ],
                    false,
                    false,
                    false,
                    None,
                )
            }
            ops::COMMIT => self.commit(&fields[0]),
            ops::CHAR => {
                let value = &fields[char_input::TEXT];
                self.query = Query::Composer(
                    value.clone(),
                    [
                        char_input::DELETE_SPACE_BEFORE,
                        char_input::SPACE_BEFORE,
                        char_input::SPACE_AFTER,
                    ]
                    .map(|field| flag::decode(&fields[field])),
                );
                let mut query = vec![vec![]; text_query::SIZE];
                query[text_query::QUERY] = text(ops::CHARACTER);
                query[text_query::TEXT] = value.clone();
                query
            }
            ops::CHARACTER => {
                let Query::Composer(value, spaces) = self.query.clone() else {
                    return self.done(false, vec![]);
                };
                if integer(fields, 0) != value.len() as i32 || !s.cursor_mode() || self.raw {
                    self.query = Query::None;
                    return self.commit(&value);
                }
                let before = c.before();
                let count = self.composer_read.max(usize::from(spaces[0]));
                let previous = &before[before.len().saturating_sub(count)..];
                let action = self
                    .composer
                    .as_ref()
                    .and_then(|rules| rules.actions(previous, &value));
                let (remove, value) = action.map(|a| (a.delete, a.text)).unwrap_or((0, value));
                self.edit(ops::COMPOSER, &[number(remove), value])
            }
            ops::COMPOSER => {
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
                        Op::simple(effect::BEGIN),
                        Op::new(effect::COMPOSING, s.start - rm, s.start, &[]),
                        Op::new(effect::COMPOSING_TEXT, 1, 0, &value),
                        Op::simple(NEW_COMPOSING),
                        Op::simple(effect::END),
                    ],
                    false,
                    false,
                    false,
                    None,
                )
            }
            ops::FINALIZE => {
                let value = &fields[0];
                let composing = c.composing();
                if self.raw || !composing.valid() {
                    return self.done(
                        false,
                        vec![Op::simple(effect::BEGIN), Op::simple(effect::END)],
                    );
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
                        Op::simple(effect::BEGIN),
                        Op::new(effect::COMPOSING_TEXT, 1, 0, value),
                        Op::simple(effect::FINISH),
                        Op::simple(effect::END),
                    ],
                    false,
                    true,
                    false,
                    None,
                )
            }
            ops::REPLACE => {
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
                        Op::simple(effect::BEGIN),
                        Op::simple(effect::FINISH),
                        Op::new(effect::DELETE, length, 0, &[]),
                        Op::new(effect::COMMIT, 1, 0, value),
                        Op::simple(NEW_COMPOSING),
                        Op::simple(effect::END),
                    ],
                    false,
                    true,
                    false,
                    None,
                )
            }
            ops::DELETE => {
                let after = flag::decode(&fields[0]);
                let words = flag::decode(&fields[1]);
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
                measure_query(ops::MEASURE, value, after, flag::utf16(words), n)
            }
            ops::MEASURE => {
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
                        Op::simple(effect::BEGIN),
                        Op::simple(effect::FINISH),
                        Op::new(
                            effect::DELETE,
                            if after { 0 } else { length },
                            if after { length } else { 0 },
                            &[],
                        ),
                        Op::simple(NEW_COMPOSING),
                        Op::simple(effect::END),
                    ],
                    false,
                    false,
                    true,
                    None,
                )
            }
            ops::REFRESH => {
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
            ops::KEYS => self.done(
                integer(fields, 2) > 0,
                keys(integer(fields, 0), integer(fields, 1), integer(fields, 2)),
            ),
            ops::ACTION => self.done(
                true,
                vec![Op::new(effect::ACTION, integer(fields, 0), 0, &[])],
            ),
            _ => self.done(false, vec![]),
        }
    }
    fn commit(&mut self, value: &[u16]) -> Vec<Field> {
        let c = self.content().clone();
        let mut ops = vec![
            Op::simple(effect::BEGIN),
            Op::simple(effect::FINISH),
            Op::new(effect::COMMIT, 1, 0, value),
        ];
        if self.raw {
            ops.push(Op::simple(effect::END));
            return self.done(true, ops);
        }
        let mut before = c.before();
        before.extend(value);
        ops.extend([Op::simple(NEW_COMPOSING), Op::simple(effect::END)]);
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

/// A [`measure`] query, answered by `answer`: `count` characters or words of `value`.
pub(crate) fn measure_query(
    answer: &str,
    value: Field,
    forward: bool,
    words: Field,
    count: i32,
) -> Vec<Field> {
    let mut query = vec![vec![]; measure::SIZE];
    query[measure::QUERY] = text(answer);
    query[measure::TEXT] = value;
    query[measure::FORWARD] = flag::utf16(forward);
    query[measure::WORDS] = words;
    query[measure::COUNT] = number(count);
    query
}
