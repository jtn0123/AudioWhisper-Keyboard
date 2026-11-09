//! The mic key, as Kotlin's `MicKey` had it: a tap toggles dictation, and a hold of [`HOLD_MS`]
//! or more dictates until it is let go, the way Gboard's and Samsung's mics work. Either way
//! dictation starts on the press, so the first words are heard and the model loads while they are
//! spoken.

/// A press this long, in milliseconds, is a hold: longer than a tap (about 100 ms), short enough
/// that holding feels immediate.
pub const HOLD_MS: i64 = 300;

/// What dictation is doing when the key goes down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Idle,
    Listening,
    Finishing,
}

/// What a press or a release asks dictation to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Nothing,
    Start,
    Finish,
    /// Finish: the key was let go after holding it to talk. Unlike a tap, nothing else marked the
    /// moment.
    EndHold,
    Toggle,
}

/// The press the key is in.
#[derive(Clone, Copy, Debug)]
struct Press {
    /// When it came, in milliseconds.
    at: i64,
    /// Dictation was idle, so the press started it.
    started: bool,
    /// Dictation was listening, so the release finishes it.
    finishes: bool,
}

/// The mic key, between a press and its release.
#[derive(Debug, Default)]
pub struct MicKey {
    press: Option<Press>,
}

impl MicKey {
    /// The key went down at `now` while dictation was `status`: [`Action::Start`] if it was idle.
    pub fn press(&mut self, status: Status, now: i64) -> Action {
        let started = status == Status::Idle;
        self.press = Some(Press {
            at: now,
            started,
            finishes: status == Status::Listening,
        });
        if started {
            Action::Start
        } else {
            Action::Nothing
        }
    }

    /// The key came up at `now`, or the finger slid off it. A release without a press seen acts
    /// as a tap did before, toggling.
    pub fn release(&mut self, now: i64) -> Action {
        let Some(press) = self.press.take() else {
            return Action::Toggle;
        };
        if press.finishes {
            Action::Finish
        // Kotlin's Long difference wraps.
        } else if press.started && now.wrapping_sub(press.at) >= HOLD_MS {
            Action::EndHold
        } else {
            Action::Nothing
        }
    }
}

#[cfg(test)]
mod tests;
