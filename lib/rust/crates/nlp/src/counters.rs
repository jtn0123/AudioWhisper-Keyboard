//! Private accuracy counters: how often typing on this phone goes right and wrong, kept as
//! integers only. Nothing textual is kept or saved: no word, no character, no position. What the
//! counters remember between events (where the last finished word sits, so a backspace into it can
//! be noticed) lives in memory only and is forgotten with the field.
//!
//! The NLP session counts what it already sees (a word finished in "learn", a suggestion tapped in
//! "pick"); Kotlin passes the rest through the "tally" operation: a keystroke, a backspace, an
//! undo. The counts are saved as `name value` lines (see [`Counters::text`]) in the app's private
//! files; docs/rust.md says how to read them.

use crate::{Field, Session, int, number, text};

/// Keystrokes after a word is finished within which deleting back into it counts as a typo the
/// user left and then fixed.
pub const TYPO_KEYSTROKES: u32 = 6;

const HEADER: &str = "typing-counters 1";

/// Each counter's name in the saved file, in order. The suggestion slots are the row's left,
/// middle and right suggestion, then any further along a scrolling row.
pub const NAMES: [&str; 16] = [
    "words",
    "kept_as_typed",
    "autocorrected",
    "autocorrect_undone",
    "picked_typing_left",
    "picked_typing_middle",
    "picked_typing_right",
    "picked_typing_further",
    "picked_after_space_left",
    "picked_after_space_middle",
    "picked_after_space_right",
    "picked_after_space_further",
    "predicted",
    "swiped",
    "swipe_replaced",
    "typos_left",
];

/// Indexes into [`NAMES`].
const WORDS: usize = 0;
const KEPT_AS_TYPED: usize = 1;
const AUTOCORRECTED: usize = 2;
const AUTOCORRECT_UNDONE: usize = 3;
const PICKED_TYPING: usize = 4;
const PICKED_AFTER_SPACE: usize = 8;
const PREDICTED: usize = 12;
const SWIPED: usize = 13;
const SWIPE_REPLACED: usize = 14;
const TYPOS_LEFT: usize = 15;

/// What happened just now, to tell what the next event means. Never saved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Recent {
    /// A swiped word that a suggestion tapped or a backspace straight after replaces.
    swipe: bool,
    /// An auto-correction was the last thing done, so an undo now takes it back.
    correction: bool,
    /// The last word finished: where it starts and ends in the text, and how many keystrokes
    /// are left in which deleting into it counts as a typo left.
    word: Option<(i64, i64, u32)>,
    /// The last word counted as kept as typed, so one reported twice counts once.
    kept: Option<(i64, usize)>,
}

/// A suggestion tapped, as [`Counters::picked`] counts it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pick {
    /// The auto-correction, committed by a space or punctuation.
    Correction,
    /// A word in the suggestion row: its place there, whether a word was being typed, and if
    /// not, whether the cursor followed a space.
    Suggestion {
        slot: usize,
        typing: bool,
        after_space: bool,
    },
}

/// The counts, what they need to remember, and whether they changed since last saved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    counts: [u64; NAMES.len()],
    recent: Recent,
    changed: bool,
}

impl Counters {
    fn add(&mut self, at: usize) {
        self.counts[at] = self.counts[at].saturating_add(1);
        self.changed = true;
    }

    /// The count called `name` in [`NAMES`].
    pub fn get(&self, name: &str) -> Option<u64> {
        NAMES
            .iter()
            .position(|&n| n == name)
            .map(|at| self.counts[at])
    }

    /// A word was finished: kept as typed (`kind` 0), a suggestion committed (1) or swiped (2),
    /// starting at `start` in the text and `length` UTF-16 units long.
    pub fn finished(&mut self, kind: i64, start: i64, length: usize) {
        if kind == 0 {
            if self.recent.kept == Some((start, length)) {
                return;
            }
            self.recent.kept = Some((start, length));
            self.add(KEPT_AS_TYPED);
        }
        self.add(WORDS);
        if kind == 2 {
            self.add(SWIPED);
            self.recent.swipe = true;
            self.recent.word = None;
        } else {
            self.recent.swipe = false;
            let end = start.saturating_add(length as i64);
            self.recent.word = Some((start, end, TYPO_KEYSTROKES));
        }
    }

    /// A suggestion was tapped, or the auto-correction committed.
    pub fn picked(&mut self, pick: Pick) {
        match pick {
            Pick::Correction => {
                self.add(AUTOCORRECTED);
                self.recent.correction = true;
            }
            Pick::Suggestion {
                slot,
                typing,
                after_space,
            } => {
                let slot = slot.min(3);
                if typing {
                    self.add(PICKED_TYPING + slot);
                    // A suggestion for the swiped word, which is the word being typed, replaces it.
                    if std::mem::take(&mut self.recent.swipe) {
                        self.add(SWIPE_REPLACED);
                    }
                } else {
                    if after_space {
                        self.add(PICKED_AFTER_SPACE + slot);
                    }
                    self.add(PREDICTED);
                }
            }
        }
    }

    /// A key that types went down: a character, a space or Enter.
    pub fn key(&mut self) {
        self.recent.swipe = false;
        self.recent.correction = false;
        self.recent.kept = None;
        self.spend();
    }

    /// A backspace with the selection at `start`..`end`, before it deleted anything.
    pub fn deleted(&mut self, start: i64, end: i64) {
        self.recent.correction = false;
        self.recent.kept = None;
        if std::mem::take(&mut self.recent.swipe) {
            self.add(SWIPE_REPLACED);
            self.recent.word = None;
            return;
        }
        if let Some((word_start, word_end, _)) = self.recent.word {
            let into = if start < end {
                start < word_end && word_start < end
            } else {
                word_start < start && start <= word_end
            };
            if into {
                self.add(TYPOS_LEFT);
                self.recent.word = None;
                return;
            }
        }
        self.spend();
    }

    /// One keystroke less in which deleting into the last word counts.
    fn spend(&mut self) {
        if let Some((_, _, left)) = &mut self.recent.word {
            *left -= 1;
            if *left == 0 {
                self.recent.word = None;
            }
        }
    }

    /// The backspace straight after an auto-correction put back what was typed.
    pub fn undone(&mut self) {
        self.add(AUTOCORRECT_UNDONE);
        self.recent = Recent::default();
    }

    /// The undo key: it takes back an auto-correction or a swipe made just before.
    pub fn undo(&mut self) {
        if self.recent.correction {
            self.add(AUTOCORRECT_UNDONE);
        }
        if self.recent.swipe {
            self.add(SWIPE_REPLACED);
        }
        self.recent = Recent::default();
    }

    /// Input moved to another field: nothing just done relates to it.
    pub fn field(&mut self) {
        self.recent = Recent::default();
    }

    /// Every count back to zero.
    pub fn reset(&mut self) {
        *self = Counters::default();
    }

    /// Whether a count changed since this was last asked.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// The counts as saved: a header, then one `name value` line per counter. Integers only.
    pub fn text(&self) -> String {
        let mut out = format!("{HEADER}\n");
        for (name, value) in NAMES.iter().zip(self.counts) {
            out.push_str(&format!("{name} {value}\n"));
        }
        out
    }

    /// Replaces the counts with saved `text`. Lines it doesn't know are skipped, and so is
    /// everything if the header is not this version's.
    pub fn load(&mut self, text: &str) {
        self.reset();
        let mut lines = text.lines();
        if lines.next() != Some(HEADER) {
            return;
        }
        for line in lines {
            let Some((name, value)) = line.split_once(' ') else {
                continue;
            };
            let at = NAMES.iter().position(|&n| n == name);
            if let (Some(at), Ok(value)) = (at, value.parse()) {
                self.counts[at] = value;
            }
        }
    }
}

impl Session {
    /// Counting happens only where suggestions for words do and the user is not incognito.
    fn counting(&self) -> bool {
        self.context.words() && !self.context.flag(7)
    }

    /// The word being typed, read from the context's text window, if it is all there.
    fn composing_text(&self) -> Option<&[u16]> {
        let c = &self.context.0;
        let offset = int(c, 20);
        let start = usize::try_from(int(c, 12) - offset).ok()?;
        let end = usize::try_from(int(c, 13) - offset).ok()?;
        c.get(14)?.get(start..end)
    }

    /// The "learn" operation's fields `f` finished a word (see protocol.rs).
    pub(crate) fn count_finished(&mut self, kind: i64, f: &[Field]) {
        if !self.counting() {
            return;
        }
        let c = &self.context.0;
        let replaces = int(f, 5) != 0 && c[12] != c[13];
        let start = if kind == 0 || replaces {
            int(f, 4)
        } else {
            int(c, 10)
        };
        self.counters.finished(kind, start, f[2].len());
    }

    /// The candidate `id` was accepted by "pick": the auto-correction if `automatic`.
    pub(crate) fn count_pick(&mut self, id: &[u16], automatic: bool) {
        if !self.counting() {
            return;
        }
        if automatic {
            let typed = self.composing_text();
            let corrected = self.automatic.as_ref().map(|v| v.text.as_slice());
            if corrected != typed {
                self.counters.picked(Pick::Correction);
            }
            return;
        }
        let visible = self.visible();
        let Some(slot) = visible.iter().position(|v| v.id == id) else {
            return;
        };
        if visible[slot].kind != 0 {
            return;
        }
        let c = &self.context.0;
        let typing = c[12] != c[13];
        let before = int(c, 10) - int(c, 20) - 1;
        let after_space = usize::try_from(before)
            .ok()
            .and_then(|at| c.get(14)?.get(at))
            .is_some_and(|&u| chars::is_whitespace(u));
        self.counters.picked(Pick::Suggestion {
            slot,
            typing,
            after_space,
        });
    }

    /// The counters' own operations, or None for any other.
    pub(crate) fn counter_call(
        &mut self,
        op: &str,
        f: &[Field],
    ) -> Option<Result<Vec<Field>, &'static str>> {
        let count = match op {
            "tally" => 3,
            "tallyLoad" => 1,
            "tallyText" | "tallyChanged" | "tallyReset" => 0,
            _ => return None,
        };
        if f.len() != count {
            return Some(Err("wrong NLP field count"));
        }
        Some(Ok(match op {
            "tally" => return Some(self.tally(f)),
            "tallyLoad" => {
                self.counters.load(&String::from_utf16_lossy(&f[0]));
                vec![]
            }
            "tallyText" => vec![text(&self.counters.text())],
            "tallyChanged" => vec![number(i32::from(self.counters.take_changed()))],
            _ => {
                self.counters.reset();
                vec![]
            }
        }))
    }

    /// An event Kotlin passes through: "key", "deleted" with the selection's start and end,
    /// "undone" (a backspace undid the auto-correction) or "undo" (the undo key).
    fn tally(&mut self, f: &[Field]) -> Result<Vec<Field>, &'static str> {
        let event = String::from_utf16_lossy(&f[0]);
        match event.as_str() {
            "key" => self.counters.key(),
            "deleted" => self.counters.deleted(int(f, 1), int(f, 2)),
            "undone" if self.counting() => self.counters.undone(),
            "undone" => self.counters.field(),
            "undo" => self.counters.undo(),
            _ => return Err("unknown tally event"),
        }
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests;
