//! Batch editor plans for a recognized phrase. Dictionary and editor I/O stay outside locks.
use crate::{Spoken, Typing, join, parse, punctuate};
use std::collections::VecDeque;
use wire::dictation::effect;
#[derive(Debug, Default)]
pub(super) struct Composition {
    pending: VecDeque<Spoken>,
    names: VecDeque<bool>,
    last: Option<Vec<u16>>,
}
fn text(s: impl ToString) -> Vec<u16> {
    s.to_string().encode_utf16().collect()
}
impl Composition {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn prepare(&mut self, phrase: &[u16]) -> Vec<Vec<u16>> {
        self.pending = parse(phrase).into();
        let words: Vec<Vec<u16>> = self
            .pending
            .iter()
            .filter_map(|part| match part {
                Spoken::Words(words) => Some(crate::phrases::first_word(words).to_vec()),
                _ => None,
            })
            .collect();
        let mut effects = vec![text(effect::NAMES), text(words.len())];
        effects.extend(words);
        effects
    }
    pub fn set_names(&mut self, fields: &[Vec<u16>]) {
        self.names = fields.iter().map(|s| wire::flag::decode(s)).collect();
    }
    pub fn plan(&mut self, before: Vec<u16>) -> Vec<Vec<u16>> {
        let mut effects = Vec::new();
        if let Some(part) = self.pending.pop_front() {
            let typing = match part {
                Spoken::Words(words) => {
                    let name = self.names.pop_front().unwrap_or(false);
                    join(&before, self.last.as_deref(), &words, |_| name)
                }
                Spoken::Mark(mark) => punctuate(&before, self.last.as_deref(), mark),
                Spoken::Break(line) => Typing {
                    replacing: 0,
                    text: line.encode_utf16().collect(),
                },
                Spoken::ScratchThat => {
                    self.last = None;
                    effects.push(text(effect::UNDO));
                    if !self.pending.is_empty() {
                        effects.push(text(effect::RESUME));
                    }
                    return effects;
                }
            };
            effects.extend([
                text(effect::INSERT),
                text(typing.replacing),
                typing.text.clone(),
            ]);
            self.last = Some(typing.text);
            // InputConnection may filter an edit. Re-read its actual context before
            // planning the next spoken operation rather than predicting accepted text.
            if !self.pending.is_empty() {
                effects.push(text(effect::RESUME));
            }
        }
        effects
    }
}
