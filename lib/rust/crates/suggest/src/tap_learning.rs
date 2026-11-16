//! When the tap aim ([`crate::tap_aim`]) may learn, and what it learns: the taps of a word the
//! user typed and kept.
//!
//! A word teaches only when there is good evidence the user meant every key they tapped in it:
//!
//! * it was committed as typed (a space, punctuation or Enter finished it and no correction
//!   replaced it) — [`Taps::kept`];
//! * every character of it was typed since the last deletion, so no backspace fixed it;
//! * and it was then left alone: it waits until [`CONFIRM_CHARACTERS`] more characters are typed,
//!   or a next-word prediction is tapped, and a backspace, an undo or a suggestion picked for it
//!   in the meantime throws it away.
//!
//! Auto-corrected words don't teach: their taps were by definition not all meant (see
//! docs/decisions.md, "Tap aim").
//!
//! A tap is learned in the orientation the screen had when the word was kept.

use crate::java;
use crate::tap::{TapOffset, TapTrail};
use crate::tap_aim::{AimKey, Orientation, TapAim};

/// Characters of the next word that confirm the word before it was kept.
pub const CONFIRM_CHARACTERS: usize = 2;

/// A kept word's taps, waiting to be confirmed.
#[derive(Debug)]
struct Pending {
    orientation: Orientation,
    samples: Vec<(i32, TapOffset)>,
    typed_since: usize,
}

/// The tap trail, the learned aim, and the word waiting to teach it.
#[derive(Debug)]
pub struct Taps {
    pub trail: TapTrail,
    pub aim: TapAim,
    orientation: Orientation,
    pending: Option<Pending>,
    changed: bool,
}

impl Taps {
    pub fn new(capacity: usize) -> Self {
        Taps {
            trail: TapTrail::new(capacity),
            aim: TapAim::default(),
            orientation: Orientation::Portrait,
            pending: None,
            changed: false,
        }
    }

    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn set_orientation(&mut self, orientation: Orientation) {
        self.orientation = orientation;
    }

    /// `text` went into the editor from a key (see [`TapTrail::typed`]); typing on confirms the
    /// word before.
    pub fn typed(&mut self, text: &[u16]) {
        self.trail.typed(text);
        if let Some(pending) = self.pending.as_mut() {
            pending.typed_since += java::code_points(text).len();
            if pending.typed_since >= CONFIRM_CHARACTERS {
                self.confirm();
            }
        }
    }

    /// The last typed character was deleted (see [`TapTrail::deleted`]).
    pub fn deleted(&mut self) {
        self.trail.deleted();
    }

    /// A backspace, whatever it deleted: the word waiting may be what it is fixing.
    pub fn backspace(&mut self) {
        self.pending = None;
    }

    /// The last edit was undone, or the word waiting was replaced.
    pub fn rejected(&mut self) {
        self.pending = None;
    }

    /// Input moved to another field.
    pub fn clear(&mut self) {
        self.trail.clear();
        self.pending = None;
    }

    /// `word` was committed as typed. The word waiting before it is confirmed, and this one
    /// waits in its place if all of it was typed since the last deletion.
    pub fn kept(&mut self, word: &[u16]) {
        self.confirm();
        let Some(typed) = self.trail.typed_for(word) else {
            return;
        };
        if !self.trail.typed_since_deletion(typed.len()) {
            return;
        }
        let samples: Vec<_> = typed
            .into_iter()
            .filter_map(|(code_point, offset)| Some((code_point, offset?)))
            .collect();
        if !samples.is_empty() {
            self.pending = Some(Pending {
                orientation: self.orientation,
                samples,
                typed_since: 0,
            });
        }
    }

    /// A suggestion was tapped. With nothing being typed it is a next-word prediction: typing went
    /// on, and the word waiting was kept. With a word being typed it is kept only if typing went
    /// on since; otherwise the suggestion is replacing it.
    pub fn picked(&mut self, composing: bool) {
        let moved_on = self.pending.as_ref().is_some_and(|p| p.typed_since > 0);
        if !composing || moved_on {
            self.confirm();
        } else {
            self.pending = None;
        }
    }

    /// A swiped word went in after the word waiting: typing went on.
    pub fn swiped(&mut self) {
        self.confirm();
    }

    fn confirm(&mut self) {
        if let Some(pending) = self.pending.take() {
            self.aim.learn(pending.orientation, &pending.samples);
            self.changed = true;
        }
    }

    /// Where each character of `word` was tapped (see [`TapTrail::offsets_for`]), moved by the
    /// learned aim if `aimed`.
    pub fn offsets_for(&self, word: &[u16], aimed: bool) -> Vec<Option<TapOffset>> {
        let offsets = self.trail.offsets_for(word);
        if !aimed {
            return offsets;
        }
        offsets
            .into_iter()
            .zip(java::code_points(word))
            .map(|(offset, code_point)| {
                offset.map(|offset| self.aim.aimed(self.orientation, code_point, offset))
            })
            .collect()
    }

    /// See [`TapAim::retarget`], in the current orientation.
    pub fn retarget(
        &self,
        keys: &[AimKey],
        hit: usize,
        at: (f32, f32),
        key_at: impl Fn(f32, f32) -> Option<usize>,
    ) -> usize {
        self.aim.retarget(self.orientation, keys, hit, at, key_at)
    }

    /// Whether the aim changed since this was last asked, so it needs saving.
    pub fn take_changed(&mut self) -> bool {
        std::mem::take(&mut self.changed)
    }

    /// Erases the learned aim and the word waiting.
    pub fn forget(&mut self) {
        self.aim.clear();
        self.pending = None;
        self.changed = false;
    }

    /// Replaces the learned aim with the saved `text` (see [`TapAim::parse`]).
    pub fn load(&mut self, text: &[u16]) {
        self.aim = TapAim::parse(text);
        self.changed = false;
    }
}

#[cfg(test)]
mod tests;
