//! One typing event in, one plan out (`wire::typing`). The session counts it, teaches the tap
//! trail, learns the word it finished and arms or takes the auto-correction undo itself, in the
//! order the Kotlin fan-out did; the plan holds only what Android must do after: when to save,
//! whether to write the kept words, whether a tapped candidate goes in, how to undo a correction,
//! and the suggestion row to publish.

use std::sync::Mutex;

use suggest::dictionary::Lookup;
use suggest::kept_words::KeptWords;
use suggest::learned_words::LearnedWords;
use suggest::tap_learning::Taps;
use suggest::undo::{self, Revert};
use suggest::word_characters::continues_word;
use suggest::word_learner::WordLearner;
use wire::flag;
use wire::nlp::{context, learn, learn_plan};
use wire::typing::{event, plan, row};

use crate::{Context, Field, Session, flag_at, int, number};

/// The stores an event changes besides the session, each already locked.
pub struct Stores<'a, L: ?Sized> {
    pub learner: &'a mut WordLearner,
    /// The learned words, which the learner locks to change.
    pub history: &'a Mutex<LearnedWords>,
    pub taps: &'a mut Taps,
    pub kept: &'a mut KeptWords,
    /// The dictionary, while one is open: words are learned only then.
    pub lookup: Option<&'a mut L>,
}

impl<L: ?Sized> std::fmt::Debug for Stores<'_, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let open = self.lookup.is_some();
        f.debug_struct("Stores")
            .field("dictionary_open", &open)
            .finish_non_exhaustive()
    }
}

/// The auto-correction a backspace straight after it undoes: what was typed, what replaced it,
/// and where it starts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Armed {
    pub typed: Field,
    pub corrected: Field,
    pub start: i32,
}

/// Why an event could not be answered.
#[derive(Debug, PartialEq, Eq)]
pub enum EventError<E> {
    /// The fields did not make an event.
    Wire(&'static str),
    /// The dictionary failed to answer.
    Lookup(E),
}

/// What Android does after an event.
#[derive(Debug, Default)]
struct Plan {
    saves: Vec<i64>,
    kept: bool,
    accepted: bool,
    revert: Option<Revert>,
    publish: bool,
}

/// Whether events of `kind` take a context row after the event row.
fn with_context(kind: i32) -> bool {
    matches!(
        kind,
        event::FINISHED | event::PICKED | event::SWIPED | event::UNDO_CORRECTION
    )
}

/// The [`learn`] row of a word of `kind` that an event row `f` finished.
fn learn_row(kind: i32, f: &[Field]) -> Vec<Field> {
    let mut row = vec![vec![]; learn::SIZE];
    row[learn::KIND] = number(kind);
    row[learn::TRIGGER] = f[row::TRIGGER].clone();
    row[learn::WORD] = f[row::WORD].clone();
    row[learn::BEFORE_COMPOSING] = f[row::BEFORE_COMPOSING].clone();
    row[learn::COMPOSING_START] = f[row::COMPOSING_START].clone();
    row[learn::COMPOSING_VALID] = f[row::COMPOSING_VALID].clone();
    row[learn::TEXT_BEFORE_SELECTION] = f[row::TEXT_BEFORE_SELECTION].clone();
    row
}

impl Plan {
    /// The [`plan`] row, then the session's snapshot if it publishes.
    fn fields(self, session: &Session) -> Vec<Field> {
        let mut out = vec![vec![]; plan::SIZE];
        out[plan::SAVE] = number(self.saves.first().copied().unwrap_or(-1));
        out[plan::SAVE_AGAIN] = number(self.saves.get(1).copied().unwrap_or(-1));
        out[plan::KEPT] = flag::utf16(self.kept);
        out[plan::ACCEPTED] = flag::utf16(self.accepted);
        out[plan::REVERT] = number(self.revert.as_ref().map_or(-1, |r| r.length));
        if let Some(revert) = self.revert {
            out[plan::REVERT_TEXT] = revert.text;
            out[plan::PHANTOM] = flag::utf16(revert.phantom_space);
        } else {
            out[plan::PHANTOM] = flag::utf16(false);
        }
        out[plan::PUBLISH] = flag::utf16(self.publish);
        if self.publish {
            out.extend(session.snapshot());
        }
        out
    }
}

impl Session {
    /// The plan for the event in `f`, a [`row`] and, for the kinds that take one, a context.
    pub fn event<L: Lookup + ?Sized>(
        &mut self,
        f: &[Field],
        stores: Stores<'_, L>,
    ) -> Result<Vec<Field>, EventError<L::Error>> {
        let kind = int(f, row::KIND) as i32;
        let size = row::SIZE + if with_context(kind) { context::SIZE } else { 0 };
        if f.len() != size {
            return Err(EventError::Wire("wrong NLP event field count"));
        }
        if with_context(kind) && kind != event::UNDO_CORRECTION {
            self.configure(Context(f[row::SIZE..].to_vec()));
        }
        let mut plan = Plan {
            publish: with_context(kind) && kind != event::UNDO_CORRECTION,
            ..Plan::default()
        };
        self.answer(kind, f, stores, &mut plan)?;
        Ok(plan.fields(self))
    }

    /// Carries out the event `kind` of row `f`, filling in `plan`.
    fn answer<L: Lookup + ?Sized>(
        &mut self,
        kind: i32,
        f: &[Field],
        s: Stores<'_, L>,
        plan: &mut Plan,
    ) -> Result<(), EventError<L::Error>> {
        let now = int(f, row::NOW);
        match kind {
            event::KEY => self.counters.key(),
            event::TYPED => {
                self.counters.key();
                s.taps.typed(&f[row::WORD]);
            }
            event::TAP_DELETED => s.taps.deleted(),
            event::BACKSPACE => {
                s.learner.drop_swiped();
                s.taps.backspace();
                let (start, end) = (int(f, row::SELECTION_START), int(f, row::SELECTION_END));
                self.counters.deleted(start, end);
                self.check(s.taps, now, plan);
            }
            event::UNDO => {
                s.taps.rejected();
                self.counters.undo();
                self.check(s.taps, now, plan);
            }
            event::FINISHED => self.finished(f, s, plan)?,
            event::PICKED => self.picked(f, s, plan)?,
            event::SWIPED => {
                let lookup = s.lookup;
                self.learning(learn::SWIPED, f, s.learner, s.history, lookup, plan)?;
                s.taps.swiped();
                self.check(s.taps, now, plan);
            }
            event::DISARM => self.armed = None,
            event::UNDO_CORRECTION => self.undo_correction(f, s, plan),
            event::CORRECTION_UNDONE => {
                self.correction_undone();
                self.check(s.taps, now, plan);
            }
            event::FIELD => {
                self.field();
                s.learner.reset();
                s.taps.clear();
                plan.publish = true;
            }
            _ => return Err(EventError::Wire("unknown NLP event")),
        }
        Ok(())
    }

    /// The word being typed was finished and stays as typed: learn it, then let it teach where
    /// this user taps.
    fn finished<L: Lookup + ?Sized>(
        &mut self,
        f: &[Field],
        s: Stores<'_, L>,
        plan: &mut Plan,
    ) -> Result<(), EventError<L::Error>> {
        let lookup = s.lookup;
        self.learning(learn::TYPED, f, s.learner, s.history, lookup, plan)?;
        if self.aim(f) {
            s.taps.kept(&f[row::WORD]);
        }
        self.check(s.taps, int(f, row::NOW), plan);
        Ok(())
    }

    /// Whether the word an event row `f` finished, kept as typed, may teach where this user taps
    /// (`suggest::tap_learning`): the setting is on, the field takes corrected prose, nothing is
    /// incognito, and the word ended with a valid composing region, as "learn" asks of a kept
    /// word.
    pub(crate) fn aim(&self, f: &[Field]) -> bool {
        flag_at(f, row::AIM)
            && self.context.prose()
            && !self.context.flag(context::INCOGNITO)
            && !self.context.flag(context::RAW_EDITOR)
            && !continues_word(int(f, row::TRIGGER) as i32)
            && !f[row::WORD].is_empty()
            && flag_at(f, row::COMPOSING_VALID)
    }

    /// A suggestion was tapped: count it, then, if it goes in, learn it and arm the undo of an
    /// auto-correction.
    fn picked<L: Lookup + ?Sized>(
        &mut self,
        f: &[Field],
        s: Stores<'_, L>,
        plan: &mut Plan,
    ) -> Result<(), EventError<L::Error>> {
        let id = &f[row::CANDIDATE];
        plan.accepted = !id.is_empty() && self.pick(id);
        let word = plan.accepted && flag_at(f, row::WORD_CANDIDATE);
        if word {
            s.taps.picked(!f[row::TYPED].is_empty());
        }
        self.check(s.taps, int(f, row::NOW), plan);
        if word && flag_at(f, row::COMMIT) {
            s.learner.drop_swiped();
            let lookup = s.lookup;
            self.learning(learn::PICKED, f, s.learner, s.history, lookup, plan)?;
        }
        if flag_at(f, row::COMMIT) && flag_at(f, row::CORRECTS) {
            self.arm(f);
        }
        Ok(())
    }

    /// Arms the undo of the auto-correction a pick in row `f` committed, if it changed a word.
    fn arm(&mut self, f: &[Field]) {
        let (typed, corrected) = (&f[row::TYPED], &f[row::WORD]);
        let start = int(f, row::COMPOSING_START) as i32;
        if !typed.is_empty() && start >= 0 && corrected != typed {
            self.armed = Some(Armed {
                typed: typed.clone(),
                corrected: corrected.clone(),
                start,
            });
        }
    }

    /// A backspace straight after an auto-correction: how to put back what was typed, keeping it
    /// from being corrected again and unlearning the correction.
    fn undo_correction<L: Lookup + ?Sized>(
        &mut self,
        f: &[Field],
        s: Stores<'_, L>,
        plan: &mut Plan,
    ) {
        let Some(armed) = self.armed.take() else {
            return;
        };
        if !flag_at(f, row::CURSOR_MODE) {
            return;
        }
        let cursor = int(f, row::SELECTION_START) as i32;
        let undone = (&armed.typed[..], &armed.corrected[..], armed.start);
        let Some(revert) = undo::revert_at(undone, cursor, &f[row::TEXT_BEFORE_SELECTION]) else {
            return;
        };
        self.configure(Context(f[row::SIZE..].to_vec()));
        plan.publish = true;
        plan.kept = s.kept.add(&armed.typed, self.context.remember_kept());
        if self.learns(s.lookup.is_some()) && s.learner.undo(s.history, armed.start) {
            self.schedule(int(f, row::NOW), plan);
        }
        plan.revert = Some(revert);
    }

    /// Counts and learns the word of `kind` that row `f` finished, if words are learned here and
    /// the dictionary is open.
    fn learning<L: Lookup + ?Sized>(
        &mut self,
        kind: i32,
        f: &[Field],
        learner: &mut WordLearner,
        history: &Mutex<LearnedWords>,
        lookup: Option<&mut L>,
        plan: &mut Plan,
    ) -> Result<(), EventError<L::Error>> {
        let row = self.learn(&learn_row(kind, f));
        if row.is_empty() || !self.learns(lookup.is_some()) {
            return Ok(());
        }
        let (word, before) = (&row[learn_plan::WORD], &row[learn_plan::BEFORE]);
        let changed = if kind == learn::SWIPED {
            learner.swiped(history, lookup, word, before)
        } else {
            let start = int(&row, learn_plan::START) as i32;
            learner.finished(history, lookup, word, before, start)
        };
        if changed.map_err(EventError::Lookup)? {
            self.schedule(int(f, row::NOW), plan);
        }
        Ok(())
    }

    /// Whether words are learned in the field now, with the dictionary `loaded` or not.
    fn learns(&self, loaded: bool) -> bool {
        let mut context = self.context.clone();
        if let Some(ready) = context.0.get_mut(context::READY) {
            *ready = flag::utf16(loaded);
        }
        context.learn()
    }

    /// Schedules a save if where this user taps or the counters changed.
    fn check(&mut self, taps: &mut Taps, now: i64, plan: &mut Plan) {
        let changed = taps.take_changed() | self.counters.take_changed();
        if changed {
            self.schedule(now, plan);
        }
    }

    /// Asks for a save at `now`, as the `SAVE` operation does.
    fn schedule(&mut self, now: i64, plan: &mut Plan) {
        let delay = self.save_timing(now);
        if delay >= 0 {
            plan.saves.push(delay);
        }
    }
}
