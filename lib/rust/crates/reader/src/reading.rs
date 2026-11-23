//! A reading of the page aloud, step by step, as the keyboard's `ReadAloudService` sequenced it
//! in Kotlin: the stop button, then the voice (the neural one if it loads, saying so if it just
//! failed, else the phone's), then a look at the screen, then reading from where the reading
//! starts, a couple of blocks queued ahead, scrolling the list on when they run out.
//!
//! Each step answers one `ReadingEffect` for Kotlin to run, which answers back with [`done`],
//! [`answer`] or [`looked`]; Kotlin keeps the screens' Android objects and the voice, and this
//! keeps which blocks are read and what comes next. Kotlin stops calling when the reading is
//! cancelled; the end of a reading (the voice shut down, the button hidden) is Kotlin's, since it
//! must run on a cancelled coroutine.
//!
//! [`done`]: Reading::done
//! [`answer`]: Reading::answer
//! [`looked`]: Reading::looked

use std::collections::VecDeque;

use wire::reading::{effect, from, message};

use crate::page_text::{self, Edges, RECENT, Start};
use crate::play_queue::LOOKAHEAD;

mod look;

/// Walks of the app's window before there is nothing to read: a web page builds its tree the
/// first time it is asked, and after "Read from here" the app takes a moment to be back in front.
pub const LOOK_TRIES: u32 = 8;
/// The wait before each walk again, in ms.
pub const LOOK_RETRY_MS: i32 = 250;
/// Walks again of a web page whose text is all in sight, while it keeps growing.
pub const SETTLE_TRIES: u32 = 4;
/// The wait for a list to settle after a scroll, in ms.
pub const SETTLE_MS: i32 = 350;
/// Looks after a scroll on for where to carry on.
pub const TRIES_AFTER_SCROLL: u32 = 3;
/// Scrolls back to the top of a list at most.
pub const MAX_SCROLLS_BACK: u32 = 60;
/// The wait after each scroll back, in ms.
pub const SCROLL_BACK_MS: i32 = 120;

/// A `ReadingEffect` code and its two arguments.
pub type Effect = [i32; 3];

/// What a walk of the app's window found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Screen {
    /// The app's package.
    pub app: Option<Vec<u16>>,
    /// Each block's text.
    pub texts: Vec<Vec<u16>>,
    /// Each block's edges.
    pub edges: Vec<Edges>,
    /// Whether each block is in the list that scrolls.
    pub scrolls: Vec<bool>,
    /// The part of the screen the app shows.
    pub visible: Edges,
    /// How many nodes the walk saw.
    pub nodes: i32,
    /// Whether it is a web page.
    pub web: bool,
    /// Whether it has a list that scrolls.
    pub scroller: bool,
}

/// Where a reading is: the effect it waits on Kotlin to finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum At {
    Start,
    Shown,
    Took,
    ToldFailed(bool),
    Made,
    ToldUnavailable,
    Begun,
    Walked,
    Retrying,
    Kept(bool),
    SettleWaiting,
    SettleWalked,
    SettleKept,
    ToldNothing,
    Revealed,
    ScrolledBack(u32),
    BackWaited(u32),
    TopSettled,
    SpokeSelection,
    AwaitedSelection,
    Following,
    AwaitedAhead,
    Queued,
    AwaitedOne,
    ScrolledOn,
    WaitedOn(u32),
    AwaitedEnd,
    Unfollowed,
    Done,
}

/// What a look at the screen is for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Then {
    /// Where the reading starts.
    Start,
    /// The page once scrolled back to the top.
    Top,
    /// Where to carry on after a scroll on, at this try.
    Scrolled(u32),
}

/// A look at the screen: walks until one finds text, then settles a web page.
#[derive(Clone, Copy, Debug)]
struct Look {
    skip_own: bool,
    tries: u32,
    settled: u32,
    then: Then,
}

/// A reading, from its start to [`DONE`](effect::DONE).
#[derive(Debug)]
pub struct Reading {
    from: i32,
    selection: Vec<u16>,
    at: At,
    look: Look,
    screen: Screen,
    blocks: Vec<usize>,
    index: usize,
    offset: usize,
    read: VecDeque<Vec<u16>>,
    app: Option<Vec<u16>>,
}

impl Default for Reading {
    fn default() -> Reading {
        Reading::new()
    }
}

impl Reading {
    /// A reading not started yet.
    pub fn new() -> Reading {
        Reading {
            from: from::SCREEN,
            selection: Vec::new(),
            at: At::Start,
            look: Look {
                skip_own: false,
                tries: 0,
                settled: 0,
                then: Then::Start,
            },
            screen: Screen::default(),
            blocks: Vec::new(),
            index: 0,
            offset: 0,
            read: VecDeque::new(),
            app: None,
        }
    }

    /// Starts reading `from` a `ReadingFrom`, with the `selection` for `SELECTION`.
    pub fn start(&mut self, from: i32, selection: Vec<u16>) -> Result<Effect, String> {
        if self.at != At::Start || !(from::SCREEN..=from::SELECTION).contains(&from) {
            return Err(format!("start from {from} at {:?}", self.at));
        }
        self.from = from;
        self.selection = selection;
        Ok(self.emit(At::Shown, effect::SHOW_BUTTON, 0, 0))
    }

    /// The effect that takes no answer is done.
    pub fn done(&mut self) -> Result<Effect, String> {
        Ok(match self.at {
            At::Shown => self.emit(At::Took, effect::TAKE_VOICE, 0, 0),
            At::ToldFailed(neural) => self.make_voice(neural),
            At::ToldUnavailable | At::ToldNothing | At::AwaitedSelection | At::Unfollowed => {
                self.emit(At::Done, effect::DONE, 0, 0)
            }
            At::Retrying => self.after_retry(),
            At::Kept(settle) => self.after_keep(settle),
            At::SettleWaiting => self.walk(At::SettleWalked),
            At::SettleKept => self.after_settle_keep(),
            At::Revealed => self.after_reveal(),
            At::BackWaited(scrolls) => self.scroll_back(scrolls),
            At::TopSettled => self.look(false, Then::Top),
            At::SpokeSelection => self.emit(At::AwaitedSelection, effect::AWAIT_BLOCKS, 0, 0),
            At::Following | At::Queued => self.next_block(),
            At::AwaitedAhead => self.queue(),
            At::AwaitedOne => self.scroll_on(),
            At::WaitedOn(tries) => self.look(false, Then::Scrolled(tries)),
            At::AwaitedEnd => self.emit(At::Unfollowed, effect::UNFOLLOW, 0, 0),
            _ => return Err(self.wrong("done")),
        })
    }

    /// The effect that asked a question answered `value`.
    pub fn answer(&mut self, value: i32) -> Result<Effect, String> {
        let yes = value != 0;
        Ok(match self.at {
            At::Took if value & 2 != 0 => {
                let at = At::ToldFailed(value & 1 != 0);
                self.emit(at, effect::TELL, message::VOICE_FAILED, 0)
            }
            At::Took => self.make_voice(yes),
            At::Made if yes => {
                self.emit(At::ToldUnavailable, effect::TELL, message::UNAVAILABLE, 1)
            }
            At::Made => self.emit(At::Begun, effect::BEGIN, 0, 0),
            At::Begun if yes => self.look(self.from == from::SELECTION, Then::Start),
            At::Begun => self.emit(At::Done, effect::DONE, 0, 0),
            At::ScrolledBack(scrolls) if yes => {
                self.emit(At::BackWaited(scrolls + 1), effect::WAIT, SCROLL_BACK_MS, 0)
            }
            At::ScrolledBack(_) => self.settle_top(),
            At::ScrolledOn if yes => self.emit(At::WaitedOn(1), effect::WAIT, SETTLE_MS, 0),
            At::ScrolledOn => self.end_reading(),
            _ => return Err(self.wrong("answer")),
        })
    }

    /// The walk found `screen`, or no window at all.
    pub fn looked(&mut self, screen: Option<Screen>) -> Result<Effect, String> {
        match self.at {
            At::Walked => Ok(self.walked(screen)),
            At::SettleWalked => Ok(self.settle_walked(screen)),
            _ => Err(self.wrong("looked")),
        }
    }

    fn wrong(&self, call: &str) -> String {
        format!("{call} at {:?}", self.at)
    }

    fn emit(&mut self, at: At, code: i32, first: i32, second: i32) -> Effect {
        self.at = at;
        [code, first, second]
    }

    fn make_voice(&mut self, neural: bool) -> Effect {
        self.emit(At::Made, effect::MAKE_VOICE, i32::from(neural), 0)
    }

    /// The screen to read is found: start where the reading starts.
    fn start_reading(&mut self) -> Effect {
        let visible = self.screen.visible;
        match self.from {
            from::TOP => self.back_to_top(),
            from::SELECTION => {
                let blocks: Vec<(&[u16], Edges)> = (self.screen.texts.iter())
                    .map(Vec::as_slice)
                    .zip(self.screen.edges.iter().copied())
                    .collect();
                match page_text::find(&blocks, &self.selection, visible) {
                    Some(start) => self.read_on(start),
                    None => self.emit(At::SpokeSelection, effect::SPEAK_SELECTION, 0, 0),
                }
            }
            _ => {
                let first = page_text::first_on_screen(&self.screen.edges, visible);
                self.read_on(Start {
                    block: first.unwrap_or(0),
                    offset: 0,
                })
            }
        }
    }

    /// Back to the top: the first block of the list brought into sight, then a list (not a web
    /// page, which has all its text) scrolled back until it stops.
    fn back_to_top(&mut self) -> Effect {
        match self.screen.scrolls.iter().position(|&it| it) {
            Some(block) => self.emit(At::Revealed, effect::REVEAL, block as i32, 0),
            None => self.after_reveal(),
        }
    }

    fn after_reveal(&mut self) -> Effect {
        if self.screen.scroller {
            self.scroll_back(0)
        } else {
            self.read_on(Start {
                block: 0,
                offset: 0,
            })
        }
    }

    fn scroll_back(&mut self, scrolls: u32) -> Effect {
        if !self.screen.web && scrolls < MAX_SCROLLS_BACK {
            self.emit(At::ScrolledBack(scrolls), effect::SCROLL_BACK, 0, 0)
        } else {
            self.settle_top()
        }
    }

    fn settle_top(&mut self) -> Effect {
        self.emit(At::TopSettled, effect::WAIT, SETTLE_MS, 0)
    }

    /// Reads the screen's blocks from `start` to the end of the page.
    fn read_on(&mut self, start: Start) -> Effect {
        self.blocks = (0..self.screen.texts.len()).collect();
        self.index = start.block;
        self.offset = start.offset;
        self.read.clear();
        self.emit(At::Following, effect::FOLLOW, 0, 0)
    }

    fn next_block(&mut self) -> Effect {
        if self.index < self.blocks.len() {
            self.emit(
                At::AwaitedAhead,
                effect::AWAIT_BLOCKS,
                LOOKAHEAD as i32 - 1,
                0,
            )
        } else {
            self.emit(At::AwaitedOne, effect::AWAIT_BLOCKS, 1, 0)
        }
    }

    /// Queues the next block, remembering it if it is in the list, to find the place after a
    /// scroll.
    fn queue(&mut self) -> Effect {
        let block = self.blocks[self.index];
        self.index += 1;
        let offset = std::mem::take(&mut self.offset);
        if self.screen.scrolls[block] {
            self.read.push_back(self.screen.texts[block].clone());
            if self.read.len() > RECENT {
                self.read.pop_front();
            }
        }
        self.emit(At::Queued, effect::QUEUE, block as i32, offset as i32)
    }

    /// The blocks ran out: scroll the list on, if there is one.
    fn scroll_on(&mut self) -> Effect {
        if !self.screen.scroller {
            return self.end_reading();
        }
        self.app = self.screen.app.clone();
        self.emit(At::ScrolledOn, effect::SCROLL_FORWARD, 0, 0)
    }

    /// A look after scroll on `tries` found a screen: carry on in its list just after what was
    /// read, or look again, or stop if the app changed or nothing new came in.
    fn resume(&mut self, tries: u32) -> Effect {
        if self.screen.app != self.app {
            return self.end_reading();
        }
        let blocks: Vec<usize> = (0..self.screen.texts.len())
            .filter(|&i| self.screen.scrolls[i])
            .collect();
        let texts: Vec<&[u16]> = blocks
            .iter()
            .map(|&i| self.screen.texts[i].as_slice())
            .collect();
        let read: Vec<&[u16]> = self.read.iter().map(Vec::as_slice).collect();
        let index = page_text::resume_after(&texts, &read);
        if index < blocks.len() {
            self.blocks = blocks;
            self.index = index;
            self.next_block()
        } else if tries < TRIES_AFTER_SCROLL {
            self.emit(At::WaitedOn(tries + 1), effect::WAIT, SETTLE_MS, 0)
        } else {
            self.end_reading()
        }
    }

    fn end_reading(&mut self) -> Effect {
        self.emit(At::AwaitedEnd, effect::AWAIT_BLOCKS, 0, 0)
    }
}

#[cfg(test)]
mod tests;
