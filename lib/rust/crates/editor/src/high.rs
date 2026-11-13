//! Editor-level gestures expand into the same low-level plans, without a Kotlin policy loop.
use crate::{Field, Session, integer, number, text};
use spacing::{spaces, state};

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
        if operation == "spacing" {
            self.context.before = Some((fields[0].clone(), fields[1].clone()));
            let (op, args) = self.context.event.take().expect("spacing response");
            return self.high(&op, &args);
        }
        if operation == "context" {
            self.context.candidate_mode = 0;
            self.context.auto = integer(fields, 0);
            self.context.phantom = integer(fields, 1);
            self.context.variation = integer(fields, 2);
            self.context.automatic = integer(fields, 3) != 0;
            self.context.language = fields[4].clone();
            self.context.symbols = std::array::from_fn(|i| fields[5 + i].clone());
            self.context.immediate_word = integer(fields, 9) != 0;
            self.context.before = None;
            return vec![];
        }
        if operation == "spaces" {
            return vec![
                number(self.context.auto),
                number(self.context.phantom),
                number(self.context.candidate_mode),
            ];
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
        };
        let needs_space = matches!(operation, "typeText" | "completion" | "gesture");
        let phantom = if needs_space {
            spaces::phantom_space_before(
                &typing,
                &value,
                self.context.phantom,
                operation == "gesture",
                editor.as_ref(),
            )
        } else {
            Ok(false)
        };
        let char_spaces = if operation == "typeChar" {
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
            return vec![text("spacing"), c.before()];
        };
        match operation {
            "typeChar" => {
                let spaces = char_spaces.expect("character spaces");
                self.context.auto = spaces.auto_space;
                self.set_phantom(spaces.phantom_space);
                self.edit(
                    "char",
                    &[
                        value,
                        number(i32::from(spaces.delete_previous_space)),
                        number(i32::from(spaces.space_before)),
                        number(i32::from(spaces.space_after)),
                    ],
                )
            }
            "typeText" => {
                self.context.auto = 0;
                self.set_phantom(0);
                let value = if phantom {
                    [vec![32], value].concat()
                } else {
                    value
                };
                self.edit("commit", &[value])
            }
            "completion" | "gesture" => {
                if value.is_empty() || self.raw {
                    return self.done(false, vec![]);
                }
                self.set_phantom(state::phantom_space_active(operation == "gesture", true));
                if operation == "completion" {
                    self.context.candidate_mode = 2;
                }
                if operation == "completion" && c.composing().valid() {
                    self.edit("finalize", &[value])
                } else {
                    let output = self.edit(
                        "commit",
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
            "backward" | "forward" => {
                let words = integer(fields, 0) != 0
                    || operation == "backward"
                        && state::is_active(self.context.phantom)
                        && c.word.valid()
                        && self.context.immediate_word;
                self.context.auto = 0;
                self.set_phantom(0);
                if selection.valid() && !selection.cursor_mode() {
                    self.edit("commit", &[vec![]])
                } else {
                    self.edit(
                        "delete",
                        &[
                            number(i32::from(operation == "forward")),
                            number(i32::from(words)),
                            number(1),
                        ],
                    )
                }
            }
            "revert" => {
                self.context.auto = 0;
                self.set_phantom(if integer(fields, 2) != 0 {
                    state::phantom_space_active(false, true)
                } else {
                    0
                });
                let output = self.edit("replace", &fields[..2]);
                if output.get(1) == Some(&number(0)) {
                    self.set_phantom(0);
                }
                output
            }
            "dictation" => {
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
                    self.edit("replace", &[number(length), value])
                } else {
                    self.edit("commit", &[value])
                };
                self.mark_commit();
                output
            }
            "undoDictation" => {
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
                self.edit("replace", &[number(typed.len() as i32), restore])
            }
            "forgetDictation" => {
                self.context.undo = None;
                vec![]
            }
            "doubleSpace" => {
                self.context.next = Some(("typeText".into(), vec![text(". ")]));
                self.high("backward", &[number(0)])
            }
            "rawEnter" => {
                if value.starts_with(&text("zh"))
                    && c.composing().valid()
                    && c.composing().start != c.composing().end
                {
                    self.edit("finalize", &[c.slice(c.composing)])
                } else {
                    self.done(false, vec![])
                }
            }
            "enter" => {
                self.context.auto = 0;
                self.set_phantom(0);
                if self.raw {
                    self.edit("keys", &[number(66), number(0), number(1)])
                } else {
                    self.edit("commit", &[vec![10]])
                }
            }
            "selectAll" => {
                self.context.auto = 0;
                self.set_phantom(0);
                let mut output = if self.raw {
                    crate::operations::keys(29, 0x3000, 1)
                } else {
                    vec![crate::operations::Op::new("contextMenu", 16908319, 0, &[])]
                };
                output.insert(0, crate::operations::Op::simple("finish"));
                self.done(true, output)
            }
            "selectAround" => {
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
                    return self.edit("select", &[number(pos), number(pos)]);
                }
                self.context.event = Some(("selectExtent".into(), fields.to_vec()));
                let value = if after {
                    c.text[(c.selection.start as usize).min(c.text.len())..].to_vec()
                } else {
                    c.text[..(c.selection.end as usize).min(c.text.len())].to_vec()
                };
                vec![
                    text("extent"),
                    value,
                    number(i32::from(after)),
                    fields[1].clone(),
                    number(n),
                ]
            }
            "extent" => {
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
                self.edit("select", &[number(start.min(end)), number(start.max(end))])
            }
            _ => self.done(false, vec![]),
        }
    }
    fn set_phantom(&mut self, bits: i32) {
        self.context.candidate_mode = 1;
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
