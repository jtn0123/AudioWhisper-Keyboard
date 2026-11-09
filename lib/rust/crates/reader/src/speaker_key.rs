//! The suggestion bar's speaker key, as Kotlin's `SpeakerKey` had it: a tap reads from the top of
//! the screen down, a hold reads from the top of the page, and pressing it while it reads stops
//! it. The reading starts when the finger lifts, since only then is a tap told from a hold.

/// A press this long, in milliseconds, is a hold.
pub const HOLD_MS: i64 = 400;

/// What a press or a lift of the key asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Stop,
    FromScreen,
    FromTop,
}

/// The key, between a press and its lift.
#[derive(Debug, Default)]
pub struct SpeakerKey {
    /// When the press that will start a reading came, in milliseconds.
    down_at: Option<i64>,
}

impl SpeakerKey {
    /// The press, at `now`: [`Action::Stop`] if a reading is on, and the lift that follows then
    /// does nothing.
    pub fn down(&mut self, reading: bool, now: i64) -> Option<Action> {
        if reading {
            self.down_at = None;
            return Some(Action::Stop);
        }
        self.down_at = Some(now);
        None
    }

    /// The lift, at `now`.
    pub fn up(&mut self, now: i64) -> Option<Action> {
        let at = self.down_at.take()?;
        // Kotlin's Long difference wraps.
        if now.wrapping_sub(at) >= HOLD_MS {
            Some(Action::FromTop)
        } else {
            Some(Action::FromScreen)
        }
    }

    /// The finger slid off: nothing is read.
    pub fn cancel(&mut self) {
        self.down_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tap_reads_from_the_screen() {
        let mut key = SpeakerKey::default();
        assert_eq!(key.down(false, 1_000), None);
        assert_eq!(key.up(1_120), Some(Action::FromScreen));
        key.down(false, 1_000);
        assert_eq!(key.up(1_000 + HOLD_MS - 1), Some(Action::FromScreen));
    }

    #[test]
    fn a_hold_reads_from_the_top_of_the_page() {
        let mut key = SpeakerKey::default();
        key.down(false, 1_000);
        assert_eq!(key.up(1_000 + HOLD_MS), Some(Action::FromTop));
    }

    #[test]
    fn a_press_while_reading_stops_and_its_lift_starts_nothing() {
        let mut key = SpeakerKey::default();
        key.down(false, 0);
        assert_eq!(key.down(true, 1_000), Some(Action::Stop));
        assert_eq!(key.up(1_000 + HOLD_MS), None);
    }

    #[test]
    fn a_finger_slid_off_reads_nothing() {
        let mut key = SpeakerKey::default();
        key.down(false, 1_000);
        key.cancel();
        assert_eq!(key.up(1_100), None);
        assert_eq!(key.up(1_200), None);
    }

    #[test]
    fn each_lift_answers_its_own_press() {
        let mut key = SpeakerKey::default();
        assert_eq!(key.up(5), None);
        key.down(false, 0);
        assert_eq!(key.up(HOLD_MS), Some(Action::FromTop));
        assert_eq!(key.up(HOLD_MS * 2), None);
    }

    #[test]
    fn time_wraps_as_a_kotlin_long_does() {
        let mut key = SpeakerKey::default();
        key.down(false, i64::MAX);
        assert_eq!(key.up(i64::MIN), Some(Action::FromScreen));
        key.down(false, i64::MIN);
        assert_eq!(key.up(i64::MAX), Some(Action::FromScreen));
        key.down(false, 0);
        assert_eq!(key.up(-1), Some(Action::FromScreen));
    }
}
