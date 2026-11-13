use crate::{Context, Field, int, number};

#[derive(Clone, Debug)]
pub struct Candidate {
    pub id: Field,
    pub text: Field,
    pub kind: i64,
    pub auto: bool,
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
    pub inline_generation: u64,
    pub inline_pending: bool,
    pub inline: Vec<Field>,
    pub save_due: Option<i64>,
}
impl Session {
    pub fn configure(&mut self, context: Context) {
        if self.context != context {
            self.invalidate();
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
    }
    pub fn begin(&mut self, now: i64) -> Vec<Field> {
        let automatic = self.automatic.take();
        self.invalidate();
        self.automatic = automatic;
        self.pending = self.context.query();
        // One short coalescing window, cancelled by the next input. No wall-clock identity.
        self.due = now.saturating_add(8);
        vec![
            number(self.generation),
            number(if self.pending { 8 } else { -1 }),
        ]
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
        let prefixed_emoji = self.context.0.get(16).is_some_and(|v| !v.is_empty())
            && self.context.emoji()
            && values.iter().any(|v| v.kind == 1);
        for mut value in values {
            if !self.context.query() {
                break;
            }
            if value.kind == 0 && (!self.context.words() || prefixed_emoji)
                || value.kind == 1 && !self.context.emoji()
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
            value.auto &= self.context.autocorrect() && value.kind == 0;
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
        if !self.context.flag(8) {
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
        let mut result = vec![
            number(self.generation),
            number(self.expanded()),
            auto,
            number(shown.len()),
        ];
        result.extend(shown.iter().map(|v| v.id.clone()));
        result.push(number(self.inline.len()));
        result.extend(self.inline.iter().cloned());
        result
    }
    pub fn inline_begin(&mut self, count: i64) -> Vec<Field> {
        self.inline_generation = self.inline_generation.wrapping_add(1);
        self.inline_pending = count > 0 && self.context.flag(18);
        self.inline.clear();
        vec![
            number(self.inline_generation),
            number(i32::from(self.inline_pending)),
            number(2000),
        ]
    }
    pub fn inline_complete(&mut self, generation: u64, fields: &[Field]) -> bool {
        if generation != self.inline_generation || !self.inline_pending {
            return false;
        }
        self.inline_pending = false;
        // Legacy order: stable pinned first, including null platform views.
        let mut values: Vec<_> = fields.as_chunks::<2>().0.iter().collect();
        values.sort_by_key(|v| std::cmp::Reverse(int(*v, 1)));
        self.inline = values.into_iter().map(|v| v[0].clone()).collect();
        true
    }
}
