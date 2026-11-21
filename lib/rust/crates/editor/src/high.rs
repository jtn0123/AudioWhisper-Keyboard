//! Editor-level gestures expand into the same low-level plans, without a Kotlin policy loop.
use crate::{Field, Session, edit::measure_query, integer, number, text};
use spacing::{spaces, state};
use wire::editor::{self as ops, char_input, context, effect, spaces as wire_spaces, text_query};
use wire::flag;

/// The event an `EXTENT` answer finishes. Never sent.
const SELECT_EXTENT: &str = "selectExtent";

#[derive(Debug, Default)]
pub(crate) struct Context {
    pub candidate_mode: i32,
    pub next: Option<(String, Vec<Field>)>,
    pub prefix: Vec<crate::operations::Op>,
    pub auto: i32,
    pub phantom: i32,
    pub variation: i32,
    pub automatic: bool,
    pub language: Field,
    pub symbols: [Field; 4],
    pub before: Option<(Field, Field)>,
    pub event: Option<(String, Vec<Field>)>,
    pub immediate_word: bool,
    pub undo: Option<(Field, i32, Field)>,
}
impl Session {
    pub(crate) fn high(&mut self, operation: &str, fields: &[Field]) -> Vec<Field> {
        if operation == ops::SPACING {
            self.context.before = Some((fields[0].clone(), fields[1].clone()));
            let (op, args) = self.context.event.take().expect("spacing response");
            return self.high(&op, &args);
        }
        if operation == ops::CONTEXT {
            self.context.candidate_mode = wire_spaces::KEEP;
            self.context.auto = integer(fields, context::AUTO_SPACE);
            self.context.phantom = integer(fields, context::PHANTOM_SPACE);
            self.context.variation = integer(fields, context::KEY_VARIATION);
            self.context.automatic = flag::decode(&fields[context::AUTO_SPACE_PUNCTUATION]);
            self.context.language = fields[context::LANGUAGE].clone();
            self.context.symbols = [
                context::PRECEDING_AUTO_SPACE,
                context::FOLLOWING_AUTO_SPACE,
                context::PRECEDING_PHANTOM_SPACE,
                context::FOLLOWING_PHANTOM_SPACE,
            ]
            .map(|field| fields[field].clone());
            self.context.immediate_word = flag::decode(&fields[context::IMMEDIATE_WORD]);
            self.context.before = None;
            return vec![];
        }
        if operation == ops::SPACES {
            let mut row = vec![vec![]; wire_spaces::SIZE];
            row[wire_spaces::AUTO_SPACE] = number(self.context.auto);
            row[wire_spaces::PHANTOM_SPACE] = number(self.context.phantom);
            row[wire_spaces::CANDIDATE] = number(self.context.candidate_mode);
            return row;
        }
        let c = self.content().clone();
        let selection = c.selection();
        let value = fields.first().cloned().unwrap_or_default();
        let s = &self.context.symbols;
        let editor = self
            .context
            .before
            .as_ref()
            .map(|(one, three)| spaces::Editor {
                rule: spacing::rule::Rule {
                    preceding_auto_space: &s[0],
                    following_auto_space: &s[1],
                    preceding_phantom_space: &s[2],
                    following_phantom_space: &s[3],
                },
                language: &self.context.language,
                before_1: Some(one),
                before_3: Some(three),
                current_word: None,
            });
        let word = c.slice(c.word);
        let editor = editor.map(|e| spaces::Editor {
            current_word: Some(&word),
            ..e
        });
        let typing = spaces::Typing {
            auto_space_punctuation: self.context.automatic,
            raw_editor: self.raw,
            key_variation: self.context.variation,
            selection_valid: selection.valid(),
            selection_start: selection.start,
            input_type: self.input_type,
        };
        let needs_space = matches!(operation, ops::TYPE_TEXT | ops::COMPLETION | ops::GESTURE);
        let phantom = if needs_space {
            spaces::phantom_space_before(
                &typing,
                &value,
                self.context.phantom,
                operation == ops::GESTURE,
                editor.as_ref(),
            )
        } else {
            Ok(false)
        };
        let char_spaces = if operation == ops::TYPE_CHAR {
            spaces::commit_char(
                &typing,
                &value,
                self.context.auto,
                self.context.phantom,
                editor.as_ref(),
            )
            .map(Some)
        } else {
            Ok(None)
        };
        let (Ok(phantom), Ok(char_spaces)) = (phantom, char_spaces) else {
            self.context.event = Some((operation.to_owned(), fields.to_vec()));
            let mut query = vec![vec![]; text_query::SIZE];
            query[text_query::QUERY] = text(ops::SPACING);
            query[text_query::TEXT] = c.before();
            return query;
        };
        match operation {
            ops::TYPE_CHAR => {
                let spaces = char_spaces.expect("character spaces");
                self.context.auto = spaces.auto_space;
                self.set_phantom(spaces.phantom_space);
                let mut row = vec![vec![]; char_input::SIZE];
                row[char_input::TEXT] = value;
                row[char_input::DELETE_SPACE_BEFORE] = flag::utf16(spaces.delete_previous_space);
                row[char_input::SPACE_BEFORE] = flag::utf16(spaces.space_before);
                row[char_input::SPACE_AFTER] = flag::utf16(spaces.space_after);
                self.edit(ops::CHAR, &row)
            }
            ops::TYPE_TEXT => {
                self.context.auto = 0;
                self.set_phantom(0);
                let value = if phantom {
                    [vec![32], value].concat()
                } else {
                    value
                };
                self.edit(ops::COMMIT, &[value])
            }
            ops::COMPLETION | ops::GESTURE => {
                if value.is_empty() || self.raw {
                    return self.done(false, vec![]);
                }
                self.set_phantom(state::phantom_space_active(operation == ops::GESTURE, true));
                if operation == ops::COMPLETION {
                    self.context.candidate_mode = wire_spaces::ACCEPT;
                }
                if operation == ops::COMPLETION && c.composing().valid() {
                    self.edit(ops::FINALIZE, &[value])
                } else {
                    let output = self.edit(
                        ops::COMMIT,
                        &[if phantom {
                            [vec![32], value].concat()
                        } else {
                            value
                        }],
                    );
                    self.mark_commit();
                    output
                }
            }
            ops::BACKWARD | ops::FORWARD => {
                let words = integer(fields, 0) != 0
                    || operation == ops::BACKWARD
                        && state::is_active(self.context.phantom)
                        && c.word.valid()
                        && self.context.immediate_word;
                self.context.auto = 0;
                self.set_phantom(0);
                if selection.valid() && !selection.cursor_mode() {
                    self.edit(ops::COMMIT, &[vec![]])
                } else {
                    self.edit(
                        ops::DELETE,
                        &[
                            number(i32::from(operation == ops::FORWARD)),
                            number(i32::from(words)),
                            number(1),
                        ],
                    )
                }
            }
            ops::REVERT => {
                self.context.auto = 0;
                self.set_phantom(if integer(fields, 2) != 0 {
                    state::phantom_space_active(false, true)
                } else {
                    0
                });
                let output = self.edit(ops::REPLACE, &fields[..2]);
                if output.get(1) == Some(&number(0)) {
                    self.set_phantom(0);
                }
                output
            }
            ops::DICTATION => {
                if value.is_empty() {
                    return self.done(false, vec![]);
                }
                self.context.auto = 0;
                self.set_phantom(state::phantom_space_active(false, true));
                let length = integer(fields, 1);
                let replace = length > 0
                    && !self.raw
                    && selection.cursor_mode()
                    && c.before().len() >= length as usize;
                let before = c.before();
                let restore = if replace {
                    before[before.len() - length as usize..].to_vec()
                } else {
                    vec![]
                };
                let start = selection.start - restore.len() as i32;
                self.context.undo = if selection.cursor_mode() && start >= 0 {
                    Some((value.clone(), start, restore))
                } else {
                    None
                };
                let output = if replace {
                    self.edit(ops::REPLACE, &[number(length), value])
                } else {
                    self.edit(ops::COMMIT, &[value])
                };
                self.mark_commit();
                output
            }
            ops::UNDO_DICTATION => {
                let Some((typed, start, restore)) = self.context.undo.take() else {
                    return self.done(false, vec![]);
                };
                if !selection.cursor_mode()
                    || !dictation::reverts_at(&typed, start, selection.start, &c.before())
                {
                    return self.done(false, vec![]);
                }
                self.context.auto = 0;
                self.set_phantom(0);
                self.edit(ops::REPLACE, &[number(typed.len() as i32), restore])
            }
            ops::FORGET_DICTATION => {
                self.context.undo = None;
                vec![]
            }
            ops::DOUBLE_SPACE => {
                self.context.next = Some((ops::TYPE_TEXT.into(), vec![text(". ")]));
                self.high(ops::BACKWARD, &[number(0)])
            }
            ops::RAW_ENTER => {
                if value.starts_with(&text("zh"))
                    && c.composing().valid()
                    && c.composing().start != c.composing().end
                {
                    self.edit(ops::FINALIZE, &[c.slice(c.composing)])
                } else {
                    self.done(false, vec![])
                }
            }
            ops::ENTER => {
                self.context.auto = 0;
                self.set_phantom(0);
                if self.raw {
                    self.edit(ops::KEYS, &[number(66), number(0), number(1)])
                } else {
                    self.edit(ops::COMMIT, &[vec![10]])
                }
            }
            ops::SELECT_ALL => {
                self.context.auto = 0;
                self.set_phantom(0);
                let mut output = if self.raw {
                    crate::operations::keys(29, 0x3000, 1)
                } else {
                    vec![crate::operations::Op::new(
                        effect::CONTEXT_MENU,
                        16908319,
                        0,
                        &[],
                    )]
                };
                output.insert(0, crate::operations::Op::simple(effect::FINISH));
                self.done(true, output)
            }
            ops::SELECT_AROUND => {
                // ICU returns the requested extent; selection arithmetic is native.
                let n = integer(fields, 0);
                let after = integer(fields, 2) != 0;
                if !selection.valid() {
                    return self.done(false, vec![]);
                }
                self.context.auto = 0;
                self.set_phantom(0);
                if n <= 0 {
                    let pos = if after {
                        selection.start
                    } else {
                        selection.end
                    };
                    return self.edit(ops::SELECT, &[number(pos), number(pos)]);
                }
                self.context.event = Some((SELECT_EXTENT.into(), fields.to_vec()));
                let value = if after {
                    c.text[(c.selection.start as usize).min(c.text.len())..].to_vec()
                } else {
                    c.text[..(c.selection.end as usize).min(c.text.len())].to_vec()
                };
                measure_query(ops::EXTENT, value, after, fields[1].clone(), n)
            }
            ops::EXTENT => {
                let (_, args) = self.context.event.take().expect("selection extent");
                let length = integer(fields, 0);
                let after = integer(&args, 2) != 0;
                let end = if c.offset >= 0 {
                    c.offset + c.text.len() as i32
                } else {
                    0
                };
                let (start, end) = if after {
                    (selection.start, (selection.start + length).min(end))
                } else {
                    ((selection.end - length).max(0), selection.end)
                };
                self.edit(
                    ops::SELECT,
                    &[number(start.min(end)), number(start.max(end))],
                )
            }
            _ => self.done(false, vec![]),
        }
    }
    fn set_phantom(&mut self, bits: i32) {
        self.context.candidate_mode = wire_spaces::RESET;
        self.context.phantom = bits;
        self.space = state::is_active(bits);
        self.determine = !self.raw
            && self.input_type & 0x80000 == 0
            && (!self.space || state::shows_composing_region(bits));
    }
    fn mark_commit(&mut self) {
        if let Some(p) = self.pending.as_mut() {
            p.commit = true;
        } else {
            let s = self.content().selection();
            self.last = if s.valid() { s.start.max(s.end) } else { -1 };
        }
    }
}
