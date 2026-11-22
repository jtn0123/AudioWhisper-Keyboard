use crate::session::Candidate;
use crate::{Context, Field, Session, flag_at, int, number};
use suggest::word_characters::continues_word;
use wire::flag;
use wire::nlp::{
    self as ops, aim, begin, candidate, clipboard_plan, clipboard_span, composing, composing_range,
    context, glide_plan, inline_view, learn, learn_plan, typing,
};

/// The requested and current contexts at the head of a completion.
const TWO_CONTEXTS: usize = 2 * context::SIZE;

fn candidates(fields: &[Field]) -> Vec<Candidate> {
    fields
        .as_chunks::<{ candidate::SIZE }>()
        .0
        .iter()
        .map(|v| Candidate {
            id: v[candidate::ID].clone(),
            text: v[candidate::TEXT].clone(),
            kind: int(v, candidate::KIND),
            auto: flag_at(v, candidate::AUTO),
        })
        .collect()
}
/// Whether `rows` is whole rows of `size` after `head` fields.
fn rows(fields: &[Field], head: usize, size: usize) -> bool {
    fields.len() >= head && (fields.len() - head).is_multiple_of(size)
}
impl Session {
    pub fn checked_call(&mut self, op: &str, f: &[Field]) -> Result<Vec<Field>, &'static str> {
        if let Some(result) = self.counter_call(op, f) {
            return result;
        }
        let count = match op {
            ops::CONTEXT | ops::POLICY | ops::CLIPBOARD_BEGIN_FOR => Some(context::SIZE),
            ops::TYPING_POLICY => Some(context::SIZE + typing::SIZE),
            ops::CANDIDATE_PLAN | ops::CLIPBOARD_FOR | ops::BEGIN_FOR | ops::COMPLETE_FOR => None,
            ops::LANGUAGE | ops::NEXT | ops::FORCE | ops::PREVIEW_START => Some(1),
            ops::SPELL => Some(2),
            ops::BEGIN | ops::INLINE_BEGIN | ops::INLINE_CANCEL | ops::SAVE => Some(1),
            ops::READY | ops::AUTO => Some(2),
            ops::CLEAR | ops::SNAPSHOT | ops::FIELD | ops::EXTERNAL_BEGIN | ops::WORDS => Some(0),
            ops::TYPING => Some(typing::SIZE),
            ops::PREVIEW | ops::GLIDE_PLAN => Some(3),
            ops::LEARN => Some(learn::SIZE),
            ops::AIM => Some(aim::SIZE),
            ops::AUTO_CANDIDATE => Some(TWO_CONTEXTS + candidate::SIZE),
            ops::COMPOSING => Some(composing::SIZE),
            ops::KEEP | ops::PICK | ops::EXTERNAL_VALID => Some(1),
            ops::COMPLETE | ops::DIRECT | ops::CLIPBOARD => None,
            ops::INLINE_COMPLETE | ops::CLIPBOARD_PLAN => None,
            _ => return Err("unknown NLP operation"),
        };
        if count.is_some_and(|n| f.len() != n) {
            return Err("wrong NLP field count");
        }
        if op == ops::COMPLETE && !rows(f, 1, candidate::SIZE)
            || matches!(op, ops::DIRECT | ops::CLIPBOARD) && !rows(f, 0, candidate::SIZE)
            || op == ops::INLINE_COMPLETE && !rows(f, 1, inline_view::SIZE)
            || op == ops::CLIPBOARD_PLAN && !rows(f, clipboard_plan::SIZE, clipboard_span::SIZE)
        {
            return Err("incomplete NLP result");
        }
        if op == ops::CANDIDATE_PLAN && f.len() <= context::SIZE
            || op == ops::CLIPBOARD_FOR && !rows(f, TWO_CONTEXTS + 1, candidate::SIZE)
        {
            return Err("incomplete NLP plan");
        }
        if op == ops::BEGIN_FOR && f.len() != context::SIZE + 1
            || op == ops::COMPLETE_FOR && !rows(f, context::SIZE + 1, candidate::SIZE)
        {
            return Err("incomplete NLP request");
        }
        Ok(self.dispatch(op, f))
    }

    /// Answers `op`, whose fields `f` were checked.
    fn dispatch(&mut self, op: &str, f: &[Field]) -> Vec<Field> {
        let boolean = |value| vec![flag::utf16(value)];
        match op {
            ops::PREVIEW_START => {
                self.preview_time = int(f, 0);
                vec![]
            }
            ops::PREVIEW => {
                let now = int(f, 0);
                let refresh = flag_at(f, 1) && now.saturating_sub(self.preview_time) > int(f, 2);
                if refresh {
                    self.preview_time = now;
                }
                boolean(refresh)
            }
            ops::GLIDE_PLAN => glide(int(f, 0), int(f, 1), flag_at(f, 2)),
            ops::BEGIN_FOR => {
                self.configure(Context(f[..context::SIZE].to_vec()));
                let (generation, delay) = self.begin(int(f, context::SIZE));
                let mut result = vec![vec![]; begin::SIZE];
                result[begin::GENERATION] = number(generation);
                result[begin::DELAY] = number(delay);
                result[begin::WORDS] = flag::utf16(self.context.words());
                result[begin::EMOJI] = flag::utf16(self.context.emoji());
                result.extend(self.snapshot());
                result
            }
            ops::COMPLETE_FOR => {
                let generation = int(f, context::SIZE) as u64;
                // An obsolete executor cannot roll back context or cancel the newer pending request.
                let accepted = if self.accepts(generation) {
                    self.configure(Context(f[..context::SIZE].to_vec()));
                    self.complete(generation, candidates(&f[context::SIZE + 1..]))
                } else {
                    false
                };
                let mut result = boolean(accepted);
                result.extend(self.snapshot());
                result
            }
            ops::CANDIDATE_PLAN => {
                let kept = flag_at(f, context::SIZE);
                let allowed = Context(f[..context::SIZE].to_vec()).autocorrect() && !kept;
                f[context::SIZE + 1..]
                    .iter()
                    .map(|auto| flag::utf16(allowed && flag::decode(auto)))
                    .collect()
            }
            ops::CLIPBOARD_BEGIN_FOR => {
                self.configure(Context(f.to_vec()));
                self.clipboard_generation = self.clipboard_generation.wrapping_add(1);
                self.clipboard.clear();
                let mut result = vec![number(self.clipboard_generation)];
                result.extend(self.snapshot());
                result
            }
            ops::CLIPBOARD_FOR => {
                let (requested, current) = f[..TWO_CONTEXTS].split_at(context::SIZE);
                // Completions never configure the session: an old executor cannot roll it back.
                if self.context.0 == requested
                    && requested == current
                    && int(f, TWO_CONTEXTS) as u64 == self.clipboard_generation
                {
                    self.clipboard = if self.context.on() {
                        candidates(&f[TWO_CONTEXTS + 1..])
                    } else {
                        vec![]
                    };
                }
                self.snapshot()
            }
            ops::EXTERNAL_BEGIN => {
                self.external_generation = self.external_generation.wrapping_add(1);
                vec![number(self.external_generation)]
            }
            ops::EXTERNAL_VALID => {
                boolean(int(f, 0) as u64 == self.external_generation && self.context.query())
            }
            ops::COMPOSING => composing_range(f),
            ops::AUTO_CANDIDATE => {
                let (requested, current) = f[..TWO_CONTEXTS].split_at(context::SIZE);
                let exact = self.context.0 == requested && requested == current;
                let offered = candidates(&f[TWO_CONTEXTS..])
                    .pop()
                    .expect("one validated candidate");
                let accept = exact
                    && self.context.autocorrect()
                    && offered.is(candidate::WORD)
                    && offered.auto;
                if accept {
                    self.automatic = Some(offered);
                }
                boolean(accept)
            }
            ops::POLICY => Context(f.to_vec()).policy(),
            ops::LANGUAGE => boolean(f[0] == crate::text("en")),
            ops::FORCE => boolean(false),
            ops::SPELL => boolean(
                f[0] == crate::text("en")
                    && f[1].iter().any(|&u| chars::is_letter(u))
                    && !f[1].iter().any(|&u| chars::is_digit(u)),
            ),
            ops::NEXT => {
                let mut before = f[0].clone();
                if before.last().is_some_and(|&u| !chars::is_whitespace(u)) {
                    before.push(32);
                }
                vec![before]
            }
            ops::TYPING_POLICY => Context(f[..context::SIZE].to_vec()).typing(&f[context::SIZE..]),
            ops::CONTEXT => {
                self.configure(Context(f.to_vec()));
                self.snapshot()
            }
            ops::BEGIN => {
                let (generation, delay) = self.begin(int(f, 0));
                vec![number(generation), number(delay)]
            }
            ops::READY => boolean(self.ready(int(f, 0) as u64, int(f, 1))),
            ops::COMPLETE => boolean(self.complete(int(f, 0) as u64, candidates(&f[1..]))),
            ops::DIRECT => {
                self.invalidate();
                self.candidates = self.filter(candidates(f));
                self.snapshot()
            }
            ops::CLIPBOARD => {
                self.clipboard = if self.context.on() {
                    candidates(f)
                } else {
                    vec![]
                };
                self.snapshot()
            }
            ops::FIELD => {
                self.field();
                self.snapshot()
            }
            ops::CLEAR => {
                self.invalidate();
                self.snapshot()
            }
            ops::SNAPSHOT => self.snapshot(),
            ops::WORDS => boolean(self.words()),
            ops::TYPING => self.context.typing(f),
            ops::AUTO => {
                let trigger = int(f, 0) as i32;
                let visible = self.visible();
                // A pending space means synchronously query the current Latin input: no commit.
                boolean(
                    self.context.autocorrect()
                        && !flag_at(f, 1)
                        && !continues_word(trigger)
                        && (visible.is_empty() || visible.iter().any(|v| v.is(candidate::WORD))),
                )
            }
            ops::KEEP => boolean(self.context.remember_kept()),
            ops::PICK => boolean(self.pick(&f[0])),
            ops::LEARN => self.learn(f),
            ops::AIM => boolean(self.aim(f)),
            ops::SAVE => vec![number(self.save_timing(int(f, 0)))],
            ops::INLINE_BEGIN => self.inline_begin(int(f, 0)),
            ops::INLINE_CANCEL => {
                if int(f, 0) as u64 == self.inline_generation {
                    self.inline_pending = false;
                }
                self.snapshot()
            }
            ops::INLINE_COMPLETE => boolean(self.inline_complete(int(f, 0) as u64, &f[1..])),
            ops::CLIPBOARD_PLAN => clipboard_plan(f),
            _ => unreachable!(),
        }
    }

    /// Whether the candidate `id` may be committed: it is the auto-correction or shown. Counts it.
    pub(crate) fn pick(&mut self, id: &[u16]) -> bool {
        let automatic = self.automatic.as_ref().is_some_and(|v| v.id == id);
        let accepted = automatic || self.visible().iter().any(|v| v.id == id);
        if accepted {
            self.count_pick(id, automatic);
        }
        if automatic {
            self.automatic = None;
        }
        accepted
    }

    /// Input moved to another field.
    pub(crate) fn field(&mut self) {
        self.counters.field();
        self.external_generation = self.external_generation.wrapping_add(1);
        self.invalidate();
        self.inline_begin(0);
    }

    /// How long to wait at `now` before saving: 5000 ms when no save is due, 0 when one is due
    /// already (and no longer), or -1 while one waits.
    pub(crate) fn save_timing(&mut self, now: i64) -> i64 {
        if self.save_due.is_some_and(|due| now >= due) {
            self.save_due = None;
            0
        } else if self.save_due.is_none() {
            self.save_due = Some(now.saturating_add(5000));
            5000
        } else {
            -1
        }
    }

    /// A [`learn`] row: Rust returns the whole learner operation and context, as a [`learn_plan`]
    /// row; Kotlin executes its store call.
    pub(crate) fn learn(&mut self, f: &[Field]) -> Vec<Field> {
        let kind = int(f, learn::KIND) as i32;
        let composing = flag_at(f, learn::COMPOSING_VALID);
        let finished = kind != learn::TYPED
            || !(continues_word(int(f, learn::TRIGGER) as i32)
                || f[learn::WORD].is_empty()
                || !composing);
        if finished {
            self.count_finished(kind, f);
        }
        if !self.context.learn() || !finished {
            return vec![];
        }
        let c = &self.context.0;
        let replaces = composing && c[context::COMPOSING_START] != c[context::COMPOSING_END];
        let swiped = kind == learn::SWIPED;
        let mut before = if swiped || !replaces {
            f[learn::TEXT_BEFORE_SELECTION].clone()
        } else {
            f[learn::BEFORE_COMPOSING].clone()
        };
        if (swiped || kind == learn::PICKED && !replaces)
            && before.last().is_some_and(|&u| !chars::is_whitespace(u))
        {
            before.push(32);
        }
        let mut plan = vec![vec![]; learn_plan::SIZE];
        plan[learn_plan::KIND] = number(kind);
        plan[learn_plan::WORD] = f[learn::WORD].clone();
        plan[learn_plan::BEFORE] = before;
        plan[learn_plan::START] = if replaces {
            f[learn::COMPOSING_START].clone()
        } else {
            c[context::SELECTION_START].clone()
        };
        plan
    }

    /// Whether the word an [`aim`] row's trigger finished, kept as typed, may teach where this
    /// user taps (`suggest::tap_learning`): the setting is on, the field takes corrected prose,
    /// nothing is incognito, and the word ended with a valid composing region, as "learn" asks of
    /// a kept word.
    pub(crate) fn aim(&self, f: &[Field]) -> bool {
        flag_at(f, aim::SETTING)
            && self.context.prose()
            && !self.context.flag(context::INCOGNITO)
            && !self.context.flag(context::RAW_EDITOR)
            && !continues_word(int(f, aim::TRIGGER) as i32)
            && !f[aim::WORD].is_empty()
            && flag_at(f, aim::COMPOSING_VALID)
    }
}
/// A [`glide_plan`] row for `count` suggestions, at most `max` shown, the first committed if
/// `commit`.
fn glide(count: i64, max: i64, commit: bool) -> Vec<Field> {
    let count = count.max(0);
    let commit = commit && count > 0;
    let mut plan = vec![vec![]; glide_plan::SIZE];
    plan[glide_plan::FIRST] = number(i64::from(commit));
    plan[glide_plan::END] = number(max.max(0).min(count));
    plan[glide_plan::COMMIT] = number(if commit { 0 } else { -1 });
    plan
}
/// A [`composing`] row's [`composing_range`]: a word after a colon starts at the colon.
fn composing_range(f: &[Field]) -> Vec<Field> {
    let mut range = vec![number(-1); composing_range::SIZE];
    if int(f, composing::STATUS) != 0 {
        let start = int(f, composing::START);
        let colon = usize::try_from(start.saturating_sub(1))
            .ok()
            .and_then(|at| f[composing::TEXT].get(at))
            == Some(&58);
        range[composing_range::START] = number(if colon { start - 1 } else { start });
        range[composing_range::END] = number(int(f, composing::END));
    }
    range
}
/// A [`clipboard_plan`] row, then a [`clipboard_span`] row for each match: nothing, or -1 for the
/// item, then the place of each match worth suggesting.
fn clipboard_plan(f: &[Field]) -> Vec<Field> {
    use clipboard_plan::{
        AGE, CONTENT_TEXT, ENABLED, ITEM_ID, ITEM_TEXT, LAST_ID, SENSITIVE, TIMEOUT,
    };
    let item = &f[ITEM_TEXT];
    // Android supplies regex spans in email/URL/phone order. Preserve earlier overlap precedence.
    if !flag_at(f, ENABLED)
        || f[ITEM_ID] == f[LAST_ID]
        || !f[CONTENT_TEXT].iter().all(|&u| chars::is_whitespace(u))
        || item.is_empty()
        || item.iter().all(|&u| chars::is_whitespace(u))
        || int(f, AGE) >= int(f, TIMEOUT)
    {
        return vec![];
    }
    let mut result = vec![number(-1)];
    if flag_at(f, SENSITIVE) {
        return result;
    }
    use clipboard_span::{END, START, VALUE};
    let spans: Vec<_> = f[clipboard_plan::SIZE..]
        .as_chunks::<{ clipboard_span::SIZE }>()
        .0
        .iter()
        .collect();
    for (i, span) in spans.iter().enumerate() {
        if &span[VALUE] != item
            && spans[..i].iter().all(|prev| {
                prev[VALUE] != span[VALUE]
                    && (int(*prev, END) < int(*span, START) || int(*span, END) < int(*prev, START))
            })
        {
            result.push(number(i));
        }
    }
    result
}
