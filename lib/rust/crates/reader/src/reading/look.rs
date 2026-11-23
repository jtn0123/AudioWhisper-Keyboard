//! A look at the screen: walked until a walk finds text, waiting [`LOOK_RETRY_MS`] after each
//! that doesn't, at most [`LOOK_TRIES`] times. A web page whose text is all in sight may be a
//! short page or one whose tree is still filling in (Chrome hands over what is on screen first),
//! so it is walked again until it stops growing, at most [`SETTLE_TRIES`] times. Each walk taken
//! is kept for reading; one that doesn't grow is not.

use wire::reading::{effect, message};

use super::{
    At, Effect, LOOK_RETRY_MS, LOOK_TRIES, Look, Reading, SETTLE_TRIES, Screen, Start, Then,
};

impl Reading {
    /// Starts a look, skipping this app's own window if `skip_own`, for `then`.
    pub(super) fn look(&mut self, skip_own: bool, then: Then) -> Effect {
        self.look = Look {
            skip_own,
            tries: 0,
            settled: 0,
            then,
        };
        self.walk(At::Walked)
    }

    pub(super) fn walk(&mut self, at: At) -> Effect {
        self.emit(at, effect::WALK, i32::from(self.look.skip_own), 0)
    }

    pub(super) fn walked(&mut self, screen: Option<Screen>) -> Effect {
        match screen.filter(|it| !it.texts.is_empty()) {
            Some(screen) => {
                let settle = screen.web && screen.all_shown();
                self.screen = screen;
                self.emit(At::Kept(settle), effect::KEEP, 0, 0)
            }
            None => {
                self.look.tries += 1;
                self.emit(At::Retrying, effect::WAIT, LOOK_RETRY_MS, 0)
            }
        }
    }

    pub(super) fn after_retry(&mut self) -> Effect {
        if self.look.tries < LOOK_TRIES {
            self.walk(At::Walked)
        } else {
            self.looked_for(false)
        }
    }

    pub(super) fn after_keep(&mut self, settle: bool) -> Effect {
        if settle {
            self.emit(At::SettleWaiting, effect::WAIT, LOOK_RETRY_MS, 0)
        } else {
            self.looked_for(true)
        }
    }

    pub(super) fn settle_walked(&mut self, screen: Option<Screen>) -> Effect {
        let nodes = self.screen.nodes;
        match screen.filter(|it| it.nodes > nodes) {
            Some(screen) => {
                self.screen = screen;
                self.look.settled += 1;
                self.emit(At::SettleKept, effect::KEEP, 0, 0)
            }
            None => self.looked_for(true),
        }
    }

    pub(super) fn after_settle_keep(&mut self) -> Effect {
        if self.look.settled < SETTLE_TRIES {
            self.emit(At::SettleWaiting, effect::WAIT, LOOK_RETRY_MS, 0)
        } else {
            self.looked_for(true)
        }
    }

    /// The look is over: it `found` a screen, now the one kept, or not.
    fn looked_for(&mut self, found: bool) -> Effect {
        match self.look.then {
            Then::Start if found => self.start_reading(),
            Then::Start => self.emit(At::ToldNothing, effect::TELL, message::NOTHING, 0),
            // Not found: the page as it was before scrolling back.
            Then::Top => self.read_on(Start {
                block: 0,
                offset: 0,
            }),
            Then::Scrolled(tries) if found => self.resume(tries),
            Then::Scrolled(_) => self.end_reading(),
        }
    }
}

impl Screen {
    /// Every block is in sight.
    fn all_shown(&self) -> bool {
        crate::page_text::all_shown(&self.edges, self.visible)
    }
}
