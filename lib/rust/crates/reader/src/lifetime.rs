//! How long read-aloud's pieces live, decided here for Kotlin: the neural voice kept loaded
//! between readings ([`VoiceHold`]), and a reading asked for before the reading service is up
//! ([`route`], [`still_asked`]). Kotlin keeps the voice itself, the timer and the service.

use wire::reading::{hold, route as routes};

/// The wait after a reading before the neural voice (a few hundred MB) is freed, in ms.
pub const VOICE_IDLE_MS: i64 = 3 * 60_000;
/// How long a reading asked for waits for the service to come up before it is dropped, in ms.
pub const SERVICE_WAIT_MS: i64 = 5_000;

/// The neural voice's hold: whether a reading has it, and whether it wouldn't load.
#[derive(Debug, Default)]
pub struct VoiceHold {
    in_use: bool,
    /// Installed but wouldn't load (damaged, or not enough memory): remembered while the
    /// service lives, so each reading doesn't retry a load of hundreds of MB first.
    failed: bool,
    just_failed: bool,
}

impl VoiceHold {
    /// A reading takes the voice, `loaded` or not: [`hold::USE`] the one loaded, [`hold::NONE`],
    /// or [`hold::CHECK`] whether its files are there.
    pub fn take(&mut self, loaded: bool) -> i32 {
        self.in_use = true;
        if loaded {
            hold::USE
        } else if self.failed {
            hold::NONE
        } else {
            hold::CHECK
        }
    }

    /// After [`hold::CHECK`]: [`hold::NONE`] if the files are `missing`, else [`hold::LOAD`].
    pub fn checked(&self, missing: bool) -> i32 {
        if missing { hold::NONE } else { hold::LOAD }
    }

    /// The load is over, and gave a voice if `ok`.
    pub fn loaded(&mut self, ok: bool) {
        self.failed = !ok;
        self.just_failed = !ok;
    }

    /// Whether the last load failed: true once per failure, so it is said once.
    pub fn failed_just_now(&mut self) -> bool {
        std::mem::take(&mut self.just_failed)
    }

    /// A reading is over.
    pub fn idle(&mut self) {
        self.in_use = false;
    }

    /// Whether the voice may be freed, its idle wait over or its memory wanted: no reading has it.
    pub fn may_free(&self) -> bool {
        !self.in_use
    }
}

/// What to do with a reading asked for: [`routes::READ`] it if the service is up, else
/// [`routes::ASK`] the user to turn the service on if it isn't `turned_on`, else
/// [`routes::WAIT`] for it to come up.
pub fn route(service_up: bool, turned_on: bool) -> i32 {
    if service_up {
        routes::READ
    } else if !turned_on {
        routes::ASK
    } else {
        routes::WAIT
    }
}

/// Whether a reading asked for at `asked_at` is still wanted `now`, as the service comes up.
pub fn still_asked(asked_at: i64, now: i64) -> bool {
    now.wrapping_sub(asked_at) < SERVICE_WAIT_MS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_voice_that_wont_load_is_said_once_and_not_tried_again() {
        let mut hold = VoiceHold::default();
        assert_eq!(hold.take(false), hold::CHECK);
        assert_eq!(hold.checked(false), hold::LOAD);
        hold.loaded(false);
        assert!(hold.failed_just_now());
        assert!(!hold.failed_just_now());
        assert_eq!(hold.take(false), hold::NONE);
        assert_eq!(hold.take(true), hold::USE);
        assert_eq!(hold.checked(true), hold::NONE);
        hold.loaded(true);
        assert!(!hold.failed_just_now());
        assert_eq!(hold.take(false), hold::CHECK);
    }

    #[test]
    fn the_voice_is_freed_only_when_no_reading_has_it() {
        let mut hold = VoiceHold::default();
        assert!(hold.may_free());
        hold.take(true);
        assert!(!hold.may_free());
        hold.idle();
        assert!(hold.may_free());
    }

    #[test]
    fn a_reading_waits_five_seconds_for_a_service_turned_on() {
        assert_eq!(route(true, false), routes::READ);
        assert_eq!(route(false, false), routes::ASK);
        assert_eq!(route(false, true), routes::WAIT);
        assert!(still_asked(1_000, 5_999));
        assert!(!still_asked(1_000, 6_000));
        assert!(still_asked(1_000, 0));
    }
}
