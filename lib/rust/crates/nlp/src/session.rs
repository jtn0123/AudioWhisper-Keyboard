use crate::{Context, Field, flag_at, number};
use wire::flag;
use wire::nlp::{candidate, context, inline_begin, inline_view, snapshot};

#[derive(Clone, Debug)]
pub struct Candidate {
    pub id: Field,
    pub text: Field,
    pub kind: i64,
    pub auto: bool,
}
impl Candidate {
    /// Whether it is of `kind`, an `NlpCandidateKind`.
    pub fn is(&self, kind: i32) -> bool {
        self.kind == i64::from(kind)
    }
}
#[derive(Debug, Default)]
pub struct Session {
    pub context: Context,
    pub external_generation: u64,
    pub generation: u64,
    pub pending: bool,
    pub due: i64,
    pub candidates: Vec<Candidate>,
    pub automatic: Option<Candidate>,
    pub clipboard: Vec<Candidate>,
    pub clipboard_generation: u64,
    pub inline_generation: u64,
    pub inline_pending: bool,
    pub inline: Vec<Field>,
    pub save_due: Option<i64>,
    pub preview_time: i64,
    /// Words filled the row when the in-flight request began.
    pub held: bool,
    /// Typing emptied a row of words; the request for the new text always follows.
    pub carry: bool,
    /// The private accuracy counters (see crate::counters).
    pub counters: crate::counters::Counters,
    /// The auto-correction a backspace straight after undoes (see crate::typing).
    pub armed: Option<crate::typing::Armed>,
}
/// Context fields that change as the user types: selection validity and bounds, composing
/// bounds, text and offset. A change to only these is answered by a request for the new text.
const TYPING: [usize; 7] = [
    context::SELECTION_VALID,
    context::SELECTION_START,
    context::SELECTION_END,
    context::COMPOSING_START,
    context::COMPOSING_END,
    context::TEXT,
    context::OFFSET,
];
impl Session {
    pub fn configure(&mut self, context: Context) {
        if self.context != context {
            let typing = context.query()
                && self.context.0.len() == context.0.len()
                && (self.context.0.iter().zip(&context.0).enumerate())
                    .all(|(i, (a, b))| a == b || TYPING.contains(&i));
            let holding = self.words();
            self.invalidate();
            self.carry = typing && holding;
            self.external_generation = self.external_generation.wrapping_add(1);
            self.context = context;
            // An editor/privacy/subtype change invalidates platform inflations as well.
            self.inline_generation = self.inline_generation.wrapping_add(1);
            self.inline_pending = false;
            self.inline.clear();
        }
    }
    pub fn invalidate(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.pending = false;
        self.candidates.clear();
        self.automatic = None;
        self.clipboard.clear();
        self.clipboard_generation = self.clipboard_generation.wrapping_add(1);
        self.held = false;
        self.carry = false;
    }
    /// The main row shows words, or holds their place: typing emptied a row of words and the
    /// refill is due. The Smartbar keeps the suggestions' full width across that empty list.
    pub fn words(&self) -> bool {
        !self.visible().is_empty() || self.carry || self.pending && self.held
    }
    /// Begins a request: its generation, and how long to wait before running it, or -1 to not.
    pub fn begin(&mut self, now: i64) -> (u64, i64) {
        let held = self.words();
        let automatic = self.automatic.take();
        self.invalidate();
        self.automatic = automatic;
        self.pending = self.context.query();
        self.held = held && self.pending;
        // One short coalescing window, cancelled by the next input. No wall-clock identity.
        self.due = now.saturating_add(8);
        (self.generation, if self.pending { 8 } else { -1 })
    }
    pub(crate) fn accepts(&self, generation: u64) -> bool {
        self.pending && generation == self.generation
    }
    pub fn ready(&self, generation: u64, now: i64) -> bool {
        self.accepts(generation) && now >= self.due
    }
    pub fn complete(&mut self, generation: u64, values: Vec<Candidate>) -> bool {
        if !self.accepts(generation) {
            return false;
        }
        self.pending = false;
        self.candidates = self.filter(values);
        true
    }
    pub fn filter(&self, values: Vec<Candidate>) -> Vec<Candidate> {
        let mut result: Vec<Candidate> = vec![];
        let prefixed_emoji = (self.context.0.get(context::EMOJI_PREFIX))
            .is_some_and(|v| !v.is_empty())
            && self.context.emoji()
            && values.iter().any(|v| v.is(candidate::EMOJI));
        for mut value in values {
            if !self.context.query() {
                break;
            }
            if value.is(candidate::WORD) && (!self.context.words() || prefixed_emoji)
                || value.is(candidate::EMOJI) && !self.context.emoji()
            {
                continue;
            }
            // Stable first occurrence. Never sort decoder scores a second time.
            if result
                .iter()
                .any(|v| v.kind == value.kind && v.text == value.text)
            {
                continue;
            }
            value.auto &= self.context.autocorrect() && value.is(candidate::WORD);
            result.push(value);
        }
        result
    }
    pub fn visible(&self) -> &[Candidate] {
        if !self.context.on() {
            return &[];
        }
        if !self.clipboard.is_empty() {
            &self.clipboard
        } else {
            &self.candidates
        }
    }
    pub fn expanded(&self) -> i32 {
        if !self.context.flag(context::SMARTBAR)
            || self.pending && !self.context.selection()
            || self.carry
        {
            // Clearing stale words for an in-flight or due request must not flash the actions toolbar.
            return -1;
        }
        i32::from(self.visible().is_empty() && self.inline.is_empty() || self.context.selection())
    }
    pub fn snapshot(&self) -> Vec<Field> {
        let shown = self.visible();
        let auto = self
            .automatic
            .as_ref()
            .or_else(|| shown.iter().find(|v| v.auto))
            .map(|v| v.id.clone())
            .unwrap_or_default();
        let mut result = vec![vec![]; snapshot::SIZE];
        result[snapshot::GENERATION] = number(self.generation);
        result[snapshot::EXPANDED] = number(self.expanded());
        result[snapshot::AUTO] = auto;
        result[snapshot::COUNT] = number(shown.len());
        result.extend(shown.iter().map(|v| v.id.clone()));
        result.push(number(self.inline.len()));
        result.extend(self.inline.iter().cloned());
        result
    }
    /// An [`inline_begin`] row for `count` inline suggestions.
    pub fn inline_begin(&mut self, count: i64) -> Vec<Field> {
        self.inline_generation = self.inline_generation.wrapping_add(1);
        self.inline_pending = count > 0 && self.context.flag(context::INLINE_SUGGESTIONS);
        self.inline.clear();
        let mut row = vec![vec![]; inline_begin::SIZE];
        row[inline_begin::GENERATION] = number(self.inline_generation);
        row[inline_begin::PENDING] = flag::utf16(self.inline_pending);
        row[inline_begin::TIMEOUT] = number(2000);
        row
    }
    pub fn inline_complete(&mut self, generation: u64, fields: &[Field]) -> bool {
        if generation != self.inline_generation || !self.inline_pending {
            return false;
        }
        self.inline_pending = false;
        // Legacy order: stable pinned first, including null platform views.
        let mut values: Vec<_> = fields
            .as_chunks::<{ inline_view::SIZE }>()
            .0
            .iter()
            .collect();
        values.sort_by_key(|v| std::cmp::Reverse(flag_at(*v, inline_view::PINNED)));
        self.inline = values
            .into_iter()
            .map(|v| v[inline_view::ID].clone())
            .collect();
        true
    }
}
