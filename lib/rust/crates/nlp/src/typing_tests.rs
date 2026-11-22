//! One typing event, one plan: each event family's counts, learning, tap teaching, saves, the
//! auto-correction undo, and the plan Android carries out.

use std::convert::Infallible;
use std::sync::Mutex;

use suggest::dictionary::Lookup;
use suggest::kept_words::KeptWords;
use suggest::learned_words::LearnedWords;
use suggest::raw::NOT_A_PROBABILITY;
use suggest::tap::TapOffset;
use suggest::tap_learning::Taps;
use suggest::word_learner::WordLearner;
use wire::nlp::{self as ops, candidate, context, snapshot};
use wire::typing::{event, plan, row};

use crate::typing::{Armed, EventError, Stores};
use crate::{Field, Session, number, text};

/// A dictionary that knows a few words.
struct Known;

impl Lookup for Known {
    type Error = Infallible;
    fn probability(&mut self, word: &[u16]) -> Result<i32, Infallible> {
        let known = ["the", "hello", "teh"].iter().any(|&w| text(w) == word);
        Ok(if known { 150 } else { NOT_A_PROBABILITY })
    }
}

/// The session and the stores an event changes.
struct World {
    session: Session,
    learner: WordLearner,
    history: Mutex<LearnedWords>,
    taps: Taps,
    kept: KeptWords,
    dictionary: bool,
}

/// A plain text field with suggestions and learning on, `window` its text, the cursor at its end
/// and the last `composing` units the word being typed.
fn context_row(window: &str, composing: usize) -> Vec<Field> {
    let end = window.encode_utf16().count();
    let mut c = vec![number(0); context::SIZE];
    c[context::INPUT_TYPE] = number(1);
    c[context::PROVIDER] = text("org.florisboard.nlp.providers.latin");
    c[context::LANGUAGE] = text("en");
    c[context::SUGGESTIONS] = number(1);
    c[context::LEARN] = number(1);
    c[context::SMARTBAR] = number(1);
    c[context::SELECTION_VALID] = number(1);
    c[context::SELECTION_START] = number(end);
    c[context::SELECTION_END] = number(end);
    c[context::COMPOSING_START] = number(end - composing);
    c[context::COMPOSING_END] = number(end);
    c[context::TEXT] = text(window);
    c[context::READY] = number(1);
    c
}

impl World {
    fn new() -> World {
        World {
            session: Session::default(),
            learner: WordLearner::default(),
            history: Mutex::new(LearnedWords::new(100, 100)),
            taps: Taps::new(64),
            kept: KeptWords::new(10),
            dictionary: true,
        }
    }

    /// The plan for event `kind` at time `now`, with `set` fields and, if given, a context.
    fn event(&mut self, kind: i32, now: i64, set: &[(usize, &str)], c: &[Field]) -> Vec<Field> {
        self.try_event(kind, now, set, c).unwrap()
    }

    fn try_event(
        &mut self,
        kind: i32,
        now: i64,
        set: &[(usize, &str)],
        c: &[Field],
    ) -> Result<Vec<Field>, EventError<Infallible>> {
        let mut f = vec![vec![]; row::SIZE];
        f[row::KIND] = number(kind);
        f[row::NOW] = number(now);
        for &(at, value) in set {
            f[at] = text(value);
        }
        f.extend_from_slice(c);
        let mut known = Known;
        let stores = Stores {
            learner: &mut self.learner,
            history: &self.history,
            taps: &mut self.taps,
            kept: &mut self.kept,
            lookup: self.dictionary.then_some(&mut known),
        };
        self.session.event(&f, stores)
    }

    /// The learned words as saved, without the header.
    fn learned(&self) -> String {
        let saved = String::from_utf16_lossy(&self.history.lock().unwrap().saved_text());
        saved.split_once('\n').unwrap().1.to_string()
    }

    /// The save timer that the last plan started runs out.
    fn saved(&mut self, now: i64) {
        assert_eq!(self.session.save_timing(now), 0, "a save was due");
    }

    fn count(&self, name: &str) -> u64 {
        self.session.counters.get(name).unwrap()
    }

    /// Shows `words` in the suggestion row, the first the auto-correction if `auto`.
    fn show(&mut self, words: &[&str], auto: bool) {
        let mut rows = vec![];
        for (i, word) in words.iter().enumerate() {
            rows.extend([text(word), text(word), number(candidate::WORD)]);
            rows.push(number(i32::from(auto && i == 0)));
        }
        self.session.checked_call(ops::DIRECT, &rows).unwrap();
    }

    /// `word` typed after `before`, then a space finishing it as typed.
    fn finish(&mut self, before: &str, word: &str, now: i64) -> Vec<Field> {
        let window = format!("{before}{word}");
        let c = context_row(&window, word.len());
        let start = before.encode_utf16().count().to_string();
        let fields = [
            (row::TRIGGER, "-1"),
            (row::WORD, word),
            (row::BEFORE_COMPOSING, before),
            (row::COMPOSING_START, &start),
            (row::COMPOSING_VALID, "1"),
            (row::TEXT_BEFORE_SELECTION, &window),
            (row::AIM, "1"),
        ];
        self.event(event::FINISHED, now, &fields, &c)
    }
}

fn at(plan: &[Field], field: usize) -> String {
    String::from_utf16_lossy(&plan[field])
}

#[test]
fn a_key_and_a_tapped_character_count_and_save_nothing_yet() {
    let mut w = World::new();
    let p = w.event(event::KEY, 0, &[], &[]);
    assert_eq!(
        (at(&p, plan::SAVE), at(&p, plan::PUBLISH)),
        ("-1".into(), "0".into())
    );
    assert_eq!(p.len(), plan::SIZE, "no snapshot without a publish");
    w.taps
        .trail
        .pressed(116, Some(TapOffset { dx: 0.1, dy: 0.0 }));
    w.event(event::TYPED, 1, &[(row::WORD, "t")], &[]);
    assert_eq!(w.taps.offsets_for(&text("t"), false).len(), 1);
    w.event(event::TAP_DELETED, 2, &[], &[]);
    assert!(w.taps.offsets_for(&text("t"), false).is_empty());
}

#[test]
fn a_backspace_drops_the_swipe_counts_and_saves_once() {
    let mut w = World::new();
    w.session.configure(crate::Context(context_row("a ", 0)));
    w.event(
        event::SWIPED,
        0,
        &[(row::WORD, "hello")],
        &context_row("a ", 0),
    );
    assert_eq!(w.count("swiped"), 1);
    w.saved(5000);
    let p = w.event(
        event::BACKSPACE,
        5010,
        &[(row::SELECTION_START, "8"), (row::SELECTION_END, "8")],
        &[],
    );
    assert_eq!(w.count("swipe_replaced"), 1);
    assert_eq!(at(&p, plan::SAVE), "5000", "the counters changed");
    let p = w.event(event::BACKSPACE, 5020, &[], &[]);
    assert_eq!(at(&p, plan::SAVE), "-1", "a save is already due");
    // The swiped word was dropped: the next one doesn't keep it.
    w.event(
        event::SWIPED,
        5030,
        &[(row::WORD, "the")],
        &context_row("a ", 0),
    );
    assert!(
        !w.learned().contains("hello"),
        "the dropped swipe is never kept"
    );
}

#[test]
fn the_undo_key_takes_back_a_swipe() {
    let mut w = World::new();
    w.event(
        event::SWIPED,
        0,
        &[(row::WORD, "hello")],
        &context_row("a ", 0),
    );
    w.saved(5000);
    let p = w.event(event::UNDO, 5001, &[], &[]);
    assert_eq!(w.count("swipe_replaced"), 1);
    assert_eq!(at(&p, plan::SAVE), "5000");
}

#[test]
fn a_finished_word_is_learned_counted_and_publishes() {
    let mut w = World::new();
    let p = w.finish("a ", "teh", 100);
    assert!(w.learned().contains("teh"), "learned");
    assert_eq!((w.count("words"), w.count("kept_as_typed")), (1, 1));
    assert_eq!(
        (at(&p, plan::SAVE), at(&p, plan::SAVE_AGAIN)),
        ("5000".into(), "-1".into())
    );
    assert_eq!(at(&p, plan::PUBLISH), "1");
    assert_eq!(p.len() - plan::SIZE, w.session.snapshot().len());
    // Without the dictionary it is counted, never learned.
    let mut w = World::new();
    w.dictionary = false;
    w.finish("a ", "teh", 0);
    assert_eq!((w.learned().as_str(), w.count("words")), ("", 1));
}

#[test]
fn a_word_that_goes_on_finishes_nothing() {
    let mut w = World::new();
    let c = context_row("a can", 3);
    let fields = [
        (row::TRIGGER, "39"),
        (row::WORD, "can"),
        (row::COMPOSING_START, "2"),
        (row::COMPOSING_VALID, "1"),
    ];
    let p = w.event(event::FINISHED, 0, &fields, &c);
    assert_eq!((w.count("words"), at(&p, plan::SAVE).as_str()), (0, "-1"));
}

#[test]
fn incognito_neither_learns_nor_counts() {
    let mut w = World::new();
    let mut c = context_row("a teh", 3);
    c[context::INCOGNITO] = number(1);
    let fields = [
        (row::TRIGGER, "-1"),
        (row::WORD, "teh"),
        (row::COMPOSING_START, "2"),
        (row::COMPOSING_VALID, "1"),
    ];
    w.event(event::FINISHED, 0, &fields, &c);
    assert_eq!((w.learned().as_str(), w.count("words")), ("", 0));
}

#[test]
fn a_pick_goes_in_only_when_shown_and_learns_when_committed() {
    let mut w = World::new();
    let c = context_row("a te", 2);
    w.session.configure(crate::Context(c.clone()));
    w.show(&["the", "tea"], false);
    let asked = [
        (row::CANDIDATE, "tea"),
        (row::WORD, "tea"),
        (row::WORD_CANDIDATE, "1"),
        (row::TYPED, "te"),
    ];
    let p = w.event(event::PICKED, 0, &asked, &c);
    assert_eq!(
        (at(&p, plan::ACCEPTED), at(&p, plan::PUBLISH)),
        ("1".into(), "1".into())
    );
    assert_eq!(w.learned(), "", "only asked about, not committed");
    assert_eq!(w.count("picked_typing_middle"), 1);
    let mut committed = asked.to_vec();
    committed.extend([
        (row::COMMIT, "1"),
        (row::COMPOSING_START, "2"),
        (row::COMPOSING_VALID, "1"),
    ]);
    w.event(event::PICKED, 1, &committed, &c);
    assert!(w.learned().contains("tea"));
    let p = w.event(event::PICKED, 2, &[(row::WORD_CANDIDATE, "1")], &c);
    assert_eq!(at(&p, plan::ACCEPTED), "0", "no candidate shown");
}

#[test]
fn an_auto_correction_arms_its_undo_whether_or_not_it_went_in() {
    let mut w = World::new();
    let c = context_row("a teh", 3);
    let pick = [
        (row::WORD, "the"),
        (row::TYPED, "teh"),
        (row::COMPOSING_START, "2"),
        (row::COMMIT, "1"),
        (row::CORRECTS, "1"),
        (row::WORD_CANDIDATE, "1"),
    ];
    w.event(event::PICKED, 0, &pick, &c);
    let armed = Armed {
        typed: text("teh"),
        corrected: text("the"),
        start: 2,
    };
    assert_eq!(w.session.armed, Some(armed));
    w.event(event::DISARM, 1, &[], &[]);
    assert_eq!(w.session.armed, None);
    // Nothing to undo when the word was already right, or there was no word.
    w.event(
        event::PICKED,
        2,
        &[
            (row::TYPED, "the"),
            (row::WORD, "the"),
            (row::COMMIT, "1"),
            (row::CORRECTS, "1"),
        ],
        &c,
    );
    w.event(
        event::PICKED,
        3,
        &[(row::WORD, "the"), (row::COMMIT, "1"), (row::CORRECTS, "1")],
        &c,
    );
    assert_eq!(w.session.armed, None);
}

/// Auto-corrects "teh" to "the" at 2, then a space.
fn corrected(w: &mut World) {
    w.session.configure(crate::Context(context_row("a teh", 3)));
    w.show(&["the"], true);
    let pick = [
        (row::CANDIDATE, "the"),
        (row::WORD, "the"),
        (row::TYPED, "teh"),
        (row::COMPOSING_START, "2"),
        (row::COMPOSING_VALID, "1"),
        (row::COMMIT, "1"),
        (row::CORRECTS, "1"),
        (row::WORD_CANDIDATE, "1"),
    ];
    w.event(event::PICKED, 0, &pick, &context_row("a teh", 3));
}

fn undo(w: &mut World, cursor_mode: &str) -> Vec<Field> {
    let fields = [
        (row::CURSOR_MODE, cursor_mode),
        (row::SELECTION_START, "6"),
        (row::TEXT_BEFORE_SELECTION, "a the "),
    ];
    w.event(
        event::UNDO_CORRECTION,
        10,
        &fields,
        &context_row("a the ", 0),
    )
}

#[test]
fn a_backspace_after_a_correction_puts_back_what_was_typed() {
    let mut w = World::new();
    corrected(&mut w);
    assert!(w.learned().contains("the"));
    let p = undo(&mut w, "1");
    let revert = (
        at(&p, plan::REVERT),
        at(&p, plan::REVERT_TEXT),
        at(&p, plan::PHANTOM),
    );
    assert_eq!(revert, ("4".into(), "teh".into(), "1".into()));
    assert_eq!(
        (at(&p, plan::KEPT), at(&p, plan::PUBLISH)),
        ("1".into(), "1".into())
    );
    assert!(w.kept.contains(&text("teh")));
    assert!(!w.learned().contains("the"), "the correction is unlearned");
    assert_eq!(at(&undo(&mut w, "1"), plan::REVERT), "-1", "taken once");
    let p = w.event(event::CORRECTION_UNDONE, 11, &[], &[]);
    assert_eq!(w.count("autocorrect_undone"), 1);
    assert_eq!(
        at(&p, plan::SAVE),
        "-1",
        "the unlearning asked for the save"
    );
}

#[test]
fn an_undo_needs_a_cursor_and_the_correction_still_there() {
    let mut w = World::new();
    corrected(&mut w);
    let p = undo(&mut w, "0");
    assert_eq!(
        (at(&p, plan::REVERT), at(&p, plan::PUBLISH)),
        ("-1".into(), "0".into())
    );
    assert_eq!(w.session.armed, None, "a selection disarms it too");
    corrected(&mut w);
    let fields = [
        (row::CURSOR_MODE, "1"),
        (row::SELECTION_START, "6"),
        (row::TEXT_BEFORE_SELECTION, "a thx "),
    ];
    let p = w.event(
        event::UNDO_CORRECTION,
        10,
        &fields,
        &context_row("a thx ", 0),
    );
    assert_eq!(at(&p, plan::REVERT), "-1");
    assert!(!w.kept.contains(&text("teh")));
}

#[test]
fn a_new_field_forgets_and_publishes() {
    let mut w = World::new();
    w.taps
        .trail
        .pressed(116, Some(TapOffset { dx: 0.1, dy: 0.0 }));
    w.taps.typed(&text("t"));
    let p = w.event(event::FIELD, 0, &[], &[]);
    assert!(w.taps.offsets_for(&text("t"), false).is_empty());
    assert_eq!(at(&p, plan::PUBLISH), "1");
    let generation = String::from_utf16_lossy(&p[plan::SIZE + snapshot::GENERATION]);
    assert_eq!(generation, w.session.generation.to_string());
}

#[test]
fn a_save_that_was_due_and_not_run_asks_for_two() {
    let mut w = World::new();
    w.finish("a ", "teh", 0);
    // The timer for 5000 never ran; the next finished word asks at once, then again.
    let p = w.finish("a teh ", "hello", 6000);
    assert_eq!(
        (at(&p, plan::SAVE), at(&p, plan::SAVE_AGAIN)),
        ("0".into(), "5000".into())
    );
}

#[test]
fn a_malformed_event_is_refused() {
    let mut w = World::new();
    let refused = |r: Result<_, _>| r.unwrap_err();
    assert_eq!(
        refused(w.try_event(99, 0, &[], &[])),
        EventError::Wire("unknown NLP event")
    );
    let wrong = EventError::Wire("wrong NLP event field count");
    assert_eq!(refused(w.try_event(event::FINISHED, 0, &[], &[])), wrong);
    assert_eq!(
        refused(w.try_event(event::KEY, 0, &[], &context_row("", 0))),
        wrong
    );
}
