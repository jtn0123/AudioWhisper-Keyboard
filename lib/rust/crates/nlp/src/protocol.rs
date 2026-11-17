use crate::session::Candidate;
use crate::{Context, Field, Session, int, number};

fn candidates(fields: &[Field]) -> Vec<Candidate> {
    fields
        .as_chunks::<4>()
        .0
        .iter()
        .map(|v| Candidate {
            id: v[0].clone(),
            text: v[1].clone(),
            kind: int(v, 2),
            auto: int(v, 3) != 0,
        })
        .collect()
}
impl Session {
    pub fn checked_call(&mut self, op: &str, f: &[Field]) -> Result<Vec<Field>, &'static str> {
        if let Some(result) = self.counter_call(op, f) {
            return result;
        }
        let count = match op {
            "context" | "policy" | "clipboardBeginFor" => Some(22),
            "typingPolicy" => Some(25),
            "candidatePlan" | "clipboardFor" | "beginFor" | "completeFor" => None,
            "language" | "next" | "force" | "previewStart" => Some(1),
            "spell" => Some(2),
            "begin" | "inlineBegin" | "inlineCancel" | "save" => Some(1),
            "ready" | "auto" => Some(2),
            "clear" | "snapshot" | "field" | "externalBegin" | "words" => Some(0),
            "typing" | "preview" | "glidePlan" => Some(3),
            "learn" => Some(7),
            "aim" => Some(4),
            "autoCandidate" => Some(48),
            "composing" => Some(4),
            "keep" | "pick" | "externalValid" => Some(1),
            "complete" | "direct" | "clipboard" | "inlineComplete" | "clipboardPlan" => None,
            _ => return Err("unknown NLP operation"),
        };
        if count.is_some_and(|n| f.len() != n) {
            return Err("wrong NLP field count");
        }
        if op == "complete" && (f.is_empty() || !(f.len() - 1).is_multiple_of(4))
            || matches!(op, "direct" | "clipboard") && !f.len().is_multiple_of(4)
            || op == "inlineComplete" && (f.is_empty() || !(f.len() - 1).is_multiple_of(2))
            || op == "clipboardPlan" && (f.len() < 8 || !(f.len() - 8).is_multiple_of(3))
        {
            return Err("incomplete NLP result");
        }
        if op == "candidatePlan" && f.len() < 23
            || op == "clipboardFor" && (f.len() < 45 || !(f.len() - 45).is_multiple_of(4))
        {
            return Err("incomplete NLP plan");
        }
        if op == "beginFor" && f.len() != 23
            || op == "completeFor" && (f.len() < 23 || !(f.len() - 23).is_multiple_of(4))
        {
            return Err("incomplete NLP request");
        }
        let boolean = |value| vec![number(i32::from(value))];
        Ok(match op {
            "previewStart" => {
                self.preview_time = int(f, 0);
                vec![]
            }
            "preview" => {
                let now = int(f, 0);
                let refresh = int(f, 1) != 0 && now.saturating_sub(self.preview_time) > int(f, 2);
                if refresh {
                    self.preview_time = now;
                }
                boolean(refresh)
            }
            "glidePlan" => {
                let count = int(f, 0).max(0);
                let commit = int(f, 2) != 0 && count > 0;
                vec![
                    number(i64::from(commit)),
                    number(int(f, 1).max(0).min(count)),
                    number(if commit { 0 } else { -1 }),
                ]
            }
            "beginFor" => {
                self.configure(Context(f[..22].to_vec()));
                let mut result = self.begin(int(f, 22));
                result.extend([
                    number(i32::from(self.context.words())),
                    number(i32::from(self.context.emoji())),
                ]);
                result.extend(self.snapshot());
                result
            }
            "completeFor" => {
                let generation = int(f, 22) as u64;
                // An obsolete executor cannot roll back context or cancel the newer pending request.
                let accepted = if self.accepts(generation) {
                    self.configure(Context(f[..22].to_vec()));
                    self.complete(generation, candidates(&f[23..]))
                } else {
                    false
                };
                let mut result = boolean(accepted);
                result.extend(self.snapshot());
                result
            }
            "candidatePlan" => {
                let allowed = Context(f[..22].to_vec()).autocorrect() && int(f, 22) == 0;
                f[23..]
                    .iter()
                    .map(|flag| number(i32::from(allowed && flag == &number(1))))
                    .collect()
            }
            "clipboardBeginFor" => {
                self.configure(Context(f.to_vec()));
                self.clipboard_generation = self.clipboard_generation.wrapping_add(1);
                self.clipboard.clear();
                let mut result = vec![number(self.clipboard_generation)];
                result.extend(self.snapshot());
                result
            }
            "clipboardFor" => {
                // Completions never configure the session: an old executor cannot roll it back.
                if self.context.0 == f[..22]
                    && f[..22] == f[22..44]
                    && int(f, 44) as u64 == self.clipboard_generation
                {
                    self.clipboard = if self.context.on() {
                        candidates(&f[45..])
                    } else {
                        vec![]
                    };
                }
                self.snapshot()
            }
            "externalBegin" => {
                self.external_generation = self.external_generation.wrapping_add(1);
                vec![number(self.external_generation)]
            }
            "externalValid" => {
                boolean(int(f, 0) as u64 == self.external_generation && self.context.query())
            }
            "composing" => {
                let start = int(f, 1);
                let end = int(f, 2);
                if int(f, 0) == 0 {
                    vec![number(-1), number(-1)]
                } else {
                    let colon = usize::try_from(start.saturating_sub(1))
                        .ok()
                        .and_then(|at| f[3].get(at))
                        == Some(&58);
                    vec![number(if colon { start - 1 } else { start }), number(end)]
                }
            }
            "autoCandidate" => {
                let exact = self.context.0 == f[..22] && f[..22] == f[22..44];
                let candidate = candidates(&f[44..]).pop().expect("one validated candidate");
                let accept =
                    exact && self.context.autocorrect() && candidate.kind == 0 && candidate.auto;
                if accept {
                    self.automatic = Some(candidate);
                }
                boolean(accept)
            }
            "policy" => Context(f.to_vec()).policy(),
            "language" => boolean(f[0] == crate::text("en")),
            "force" => boolean(false),
            "spell" => boolean(
                f[0] == crate::text("en")
                    && f[1].iter().any(|&u| chars::is_letter(u))
                    && !f[1].iter().any(|&u| chars::is_digit(u)),
            ),
            "next" => {
                let mut before = f[0].clone();
                if before.last().is_some_and(|&u| !chars::is_whitespace(u)) {
                    before.push(32);
                }
                vec![before]
            }
            "typingPolicy" => Context(f[..22].to_vec()).typing(&f[22], &f[23], &f[24]),
            "context" => {
                self.configure(Context(f.to_vec()));
                self.snapshot()
            }
            "begin" => self.begin(int(f, 0)),
            "ready" => boolean(self.ready(int(f, 0) as u64, int(f, 1))),
            "complete" => boolean(self.complete(int(f, 0) as u64, candidates(&f[1..]))),
            "direct" => {
                self.invalidate();
                self.candidates = self.filter(candidates(f));
                self.snapshot()
            }
            "clipboard" => {
                self.clipboard = if self.context.on() {
                    candidates(f)
                } else {
                    vec![]
                };
                self.snapshot()
            }
            "field" => {
                self.counters.field();
                self.external_generation = self.external_generation.wrapping_add(1);
                self.invalidate();
                self.inline_begin(0);
                self.snapshot()
            }
            "clear" => {
                self.invalidate();
                self.snapshot()
            }
            "snapshot" => self.snapshot(),
            "words" => boolean(self.words()),
            "typing" => self.context.typing(&f[0], &f[1], &f[2]),
            "auto" => {
                let trigger = int(f, 0) as i32;
                let visible = self.visible();
                // 1 means synchronously query the current Latin input, 0 means no commit.
                boolean(
                    self.context.autocorrect()
                        && int(f, 1) == 0
                        && !suggest::word_characters::continues_word(trigger)
                        && (visible.is_empty() || visible.iter().any(|v| v.kind == 0)),
                )
            }
            "keep" => boolean(self.context.remember_kept()),
            "pick" => {
                let automatic = self.automatic.as_ref().is_some_and(|v| v.id == f[0]);
                let accepted = automatic || self.visible().iter().any(|v| v.id == f[0]);
                if accepted {
                    self.count_pick(&f[0], automatic);
                }
                if automatic {
                    self.automatic = None;
                }
                boolean(accepted)
            }
            "learn" => {
                // Rust returns the whole learner operation and context, Kotlin executes its store call.
                let kind = int(f, 0);
                let finished = kind != 0
                    || !(suggest::word_characters::continues_word(int(f, 1) as i32)
                        || f[2].is_empty()
                        || int(f, 5) == 0);
                if finished {
                    self.count_finished(kind, f);
                }
                if !self.context.learn() || !finished {
                    vec![]
                } else {
                    let replaces = int(f, 5) != 0 && self.context.0[12] != self.context.0[13];
                    let mut before = if kind == 2 || !replaces {
                        f[6].clone()
                    } else {
                        f[3].clone()
                    };
                    if (kind == 2 || kind == 1 && !replaces)
                        && before.last().is_some_and(|&u| !chars::is_whitespace(u))
                    {
                        before.push(32);
                    }
                    vec![
                        number(kind),
                        f[2].clone(),
                        before,
                        if replaces {
                            f[4].clone()
                        } else {
                            self.context.0[10].clone()
                        },
                    ]
                }
            }
            // Whether the word `f[2]` that `f[1]` finished, kept as typed, may teach where this
            // user taps (`suggest::tap_learning`): the setting `f[0]` is on, the field takes
            // corrected prose, nothing is incognito, and the word ended with a valid composing
            // region `f[3]`, as "learn" asks of a kept word.
            "aim" => boolean(
                int(f, 0) != 0
                    && self.context.prose()
                    && !self.context.flag(7)
                    && !self.context.flag(1)
                    && !suggest::word_characters::continues_word(int(f, 1) as i32)
                    && !f[2].is_empty()
                    && int(f, 3) != 0,
            ),
            "save" => {
                let now = int(f, 0);
                if self.save_due.is_some_and(|due| now >= due) {
                    self.save_due = None;
                    vec![number(0)]
                } else if self.save_due.is_none() {
                    self.save_due = Some(now.saturating_add(5000));
                    vec![number(5000)]
                } else {
                    vec![number(-1)]
                }
            }
            "inlineBegin" => self.inline_begin(int(f, 0)),
            "inlineCancel" => {
                if int(f, 0) as u64 == self.inline_generation {
                    self.inline_pending = false;
                }
                self.snapshot()
            }
            "inlineComplete" => boolean(self.inline_complete(int(f, 0) as u64, &f[1..])),
            "clipboardPlan" => clipboard_plan(f),
            _ => unreachable!(),
        })
    }
}
fn clipboard_plan(f: &[Field]) -> Vec<Field> {
    // Android supplies regex spans in email/URL/phone order. Preserve earlier overlap precedence.
    if int(f, 0) == 0
        || f[1] == f[2]
        || !f[3].iter().all(|&u| chars::is_whitespace(u))
        || f[4].is_empty()
        || f[4].iter().all(|&u| chars::is_whitespace(u))
        || int(f, 5) >= int(f, 6)
    {
        return vec![];
    }
    let mut result = vec![number(-1)];
    if int(f, 7) != 0 {
        return result;
    }
    let spans: Vec<_> = f[8..].as_chunks::<3>().0.iter().collect();
    for (i, span) in spans.iter().enumerate() {
        if span[0] != f[4]
            && spans[..i].iter().all(|prev| {
                prev[0] != span[0]
                    && (int(*prev, 2) < int(*span, 1) || int(*span, 2) < int(*prev, 1))
            })
        {
            result.push(number(i));
        }
    }
    result
}
