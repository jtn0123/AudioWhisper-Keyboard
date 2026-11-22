//! Touch on the licence: `verticalScroll` around `horizontalScroll` around a SelectionContainer.
//! Whichever axis passes the 8 dp touch slop first owns the drag, the horizontal one when both
//! do at once, as the inner scroller sees the event first. Holding still for the 400 ms long
//! press timeout selects a word instead, and a drag that follows extends it; a handle drags
//! from its down. While a selection drag holds the finger near an edge the text scrolls on its
//! own, each frame, faster the deeper the finger is.
use super::super::{Action, App};
use crate::{panel_scroll::Fling, touch::Motion};
use std::ops::Range;

/// Android 12's `ViewConfiguration.getLongPressTimeout()`, which Compose's long press waits.
const LONG_PRESS: i64 = 400_000_000;
/// The band along each viewport edge where a selection drag scrolls, in dp.
const EDGE: f32 = 32.0;
/// The edge scroll's speed with the finger at the edge, in dp per second; it grows past it up
/// to four times.
const EDGE_SPEED: f32 = 600.0;

#[derive(Clone, Debug, Default, PartialEq)]
pub(in crate::settings_ui) enum Gesture {
    #[default]
    Idle,
    /// A down outside the text, which the screen's own handling owns.
    Elsewhere,
    /// A down on the text, waiting for the slop or the long press.
    Pressed,
    /// A drag scrolling one axis: true for the horizontal.
    Scroll(bool),
    /// Long pressed on the word, and extending from it by words.
    Words(Range<usize>),
    /// Dragging a handle: the end not dragged, and the endpoint's offset from the finger.
    Handle { other: usize, offset: [f32; 2] },
}
#[derive(Debug, Default)]
pub(in crate::settings_ui) struct Touch {
    pub gesture: Gesture,
    /// The down's position and time.
    pub down: (f32, f32, i64),
    /// The finger's last position.
    pub at: (f32, f32),
    /// The frame time the edge scroll last moved to; zero when it is not running.
    pub edge_time: i64,
}
impl Touch {
    /// Whether a long press or handle drag is moving the selection.
    pub fn selecting(&self) -> bool {
        matches!(self.gesture, Gesture::Words(_) | Gesture::Handle { .. })
    }
}
impl App {
    /// The licence's own touch handling; false hands the event to the screen's.
    pub(in crate::settings_ui) fn project_motion(&mut self, e: &Motion) -> bool {
        if !self.on_project_licence() {
            return false;
        }
        if !e.valid() || e.action == 3 || e.ids.len() != 1 {
            self.project_licence.touch = Default::default();
            self.project_licence.fling = None;
            return false;
        }
        let at = e.coords.len() - 2;
        let (x, y, t) = (e.coords[at], e.coords[at + 1], e.times[e.times.len() - 1]);
        match e.action {
            0 => self.project_down(x, y, t),
            2 => self.project_move(e, x, y, t),
            1 => self.project_up(x, y, t),
            _ => false,
        }
    }
    fn project_down(&mut self, x: f32, y: f32, t: i64) -> bool {
        self.project_licence.fling = None;
        self.project_licence.touch = Touch {
            down: (x, y, t),
            at: (x, y),
            ..Default::default()
        };
        let handle = self.controls.iter().rev().find_map(|c| match c.action {
            Action::SelectionHandle(start) if c.bounds.contains(x, y) => Some(start),
            _ => None,
        });
        let range = self.project_range();
        if let (Some(start), Some(r)) = (handle, range) {
            let (end, other) = if start {
                (r.start, r.end)
            } else {
                (r.end, r.start)
            };
            let Some(point) = self.project_point(end) else {
                return false;
            };
            let offset = [point.0 - x, point.1 - y];
            self.project_licence.touch.gesture = Gesture::Handle { other, offset };
        } else if self.project_licence.viewport.contains(x, y) {
            self.project_licence.touch.gesture = Gesture::Pressed;
        } else {
            self.project_licence.touch.gesture = Gesture::Elsewhere;
            return false;
        }
        self.fling = None;
        self.dragging = false;
        true
    }
    fn project_move(&mut self, e: &Motion, x: f32, y: f32, t: i64) -> bool {
        let touch = &self.project_licence.touch;
        let (down_x, down_y, down_t) = touch.down;
        match touch.gesture.clone() {
            Gesture::Idle | Gesture::Elsewhere => return false,
            Gesture::Pressed if t >= down_t + LONG_PRESS => {
                self.project_hold();
                self.project_licence.touch.at = (x, y);
                self.follow_finger();
            }
            Gesture::Pressed => {
                let slop = (8.0 * self.config[2] + 0.5).floor();
                let (dx, dy) = (x - down_x, y - down_y);
                if dx.abs() >= slop || dy.abs() >= slop {
                    let horizontal = dx.abs() >= slop;
                    self.project_licence.touch.gesture = Gesture::Scroll(horizontal);
                    let (from, total) = if horizontal {
                        (down_x, dx)
                    } else {
                        (down_y, dy)
                    };
                    self.scroll_tracker(horizontal)
                        .down(down_t.div_euclid(1_000_000), from);
                    self.dragging = !horizontal;
                    self.project_scroll(e, horizontal, total - total.signum() * slop);
                }
                self.project_licence.touch.at = (x, y);
            }
            Gesture::Scroll(horizontal) => {
                let (last_x, last_y) = touch.at;
                if (x, y) != (last_x, last_y) {
                    let delta = if horizontal { x - last_x } else { y - last_y };
                    self.project_scroll(e, horizontal, delta);
                    self.project_licence.touch.at = (x, y);
                }
            }
            Gesture::Words(_) | Gesture::Handle { .. } => {
                self.project_licence.touch.at = (x, y);
                self.follow_finger();
            }
        }
        true
    }
    fn project_up(&mut self, x: f32, y: f32, t: i64) -> bool {
        let gesture = std::mem::take(&mut self.project_licence.touch.gesture);
        self.project_licence.touch.edge_time = 0;
        let maximum = (8000.0 * self.config[2] + 0.5).floor();
        match gesture {
            Gesture::Idle | Gesture::Elsewhere => return false,
            Gesture::Pressed if t >= self.project_licence.touch.down.2 + LONG_PRESS => {
                self.project_long_press(x, y);
            }
            // A tap clears the selection, as SelectionContainer's tap does.
            Gesture::Pressed => self.project_licence.selection = None,
            Gesture::Scroll(true) => {
                self.project_licence.velocity.up(t.div_euclid(1_000_000));
                let velocity = -self.project_licence.velocity.velocity(maximum);
                let from = self.project_licence.scroll_x;
                self.project_licence.fling = Fling::new(t, from, self.config[2], velocity);
            }
            Gesture::Scroll(false) => {
                self.velocity.up(t.div_euclid(1_000_000));
                let velocity = -self.velocity.velocity(maximum);
                self.release_drag(t, velocity);
                self.dragging = false;
            }
            Gesture::Words(_) | Gesture::Handle { .. } => {}
        }
        true
    }
    /// The velocity tracker of one axis: the licence's own for the horizontal, the screen's for
    /// the vertical.
    fn scroll_tracker(
        &mut self,
        horizontal: bool,
    ) -> &mut crate::compose_velocity::ComposeVelocity {
        if horizontal {
            &mut self.project_licence.velocity
        } else {
            &mut self.velocity
        }
    }
    /// A scroll drag's move: every sample to the tracker, then the content follows the finger.
    fn project_scroll(&mut self, e: &Motion, horizontal: bool, delta: f32) {
        let axis = usize::from(!horizontal);
        let samples: Vec<(i64, f32)> = (0..e.times.len() - 1)
            .map(|s| (e.times[s + 1].div_euclid(1_000_000), e.coords[s * 2 + axis]))
            .collect();
        self.scroll_tracker(horizontal).change(&samples);
        if horizontal {
            self.set_project_x(self.project_licence.scroll_x - delta);
        } else {
            self.drag_by(-delta);
        }
    }
    /// The long press timeout passed with the finger still: select the word under it.
    fn project_hold(&mut self) {
        let (x, y) = self.project_licence.touch.at;
        self.project_licence.touch.gesture = match self.project_long_press(x, y) {
            Some(word) => Gesture::Words(word),
            None => Gesture::Idle,
        };
    }
    /// The selection follows the finger: by words after a long press, by characters on a handle.
    fn follow_finger(&mut self) {
        let (x, y) = self.project_licence.touch.at;
        match self.project_licence.touch.gesture.clone() {
            Gesture::Words(pressed) => self.project_extend(pressed, x, y),
            Gesture::Handle { other, offset } => {
                self.project_drag_handle(other, x + offset[0], y + offset[1]);
            }
            _ => {}
        }
    }
    /// Moves the horizontal scroll, clamped, and wakes its scrollbar when it moves.
    pub(super) fn set_project_x(&mut self, value: f32) {
        let v = value.clamp(0.0, self.project_licence.max_x);
        if v != self.project_licence.scroll_x {
            self.scrollbar_last[2] = -1;
        }
        self.project_licence.scroll_x = v;
    }
    /// The edge scroll's speed on each axis, in pixels per second, with the finger where it is:
    /// none away from the edges or where the text cannot scroll further.
    pub(super) fn edge_speed(&self) -> [f32; 2] {
        if !self.project_licence.touch.selecting() {
            return [0.0; 2];
        }
        let d = self.config[2];
        let band = EDGE * d;
        let v = self.project_licence.viewport;
        let (x, y) = self.project_licence.touch.at;
        let depth = |at: f32, low: f32, high: f32| {
            if at < low + band {
                -((low + band - at) / band).min(4.0)
            } else if at > high - band {
                ((at - high + band) / band).min(4.0)
            } else {
                0.0
            }
        };
        let room = |depth: f32, at: f32, max: f32| {
            if (depth < 0.0 && at > 0.0) || (depth > 0.0 && at < max) {
                depth * EDGE_SPEED * d
            } else {
                0.0
            }
        };
        [
            room(
                depth(x, v.left, v.right),
                self.project_licence.scroll_x,
                self.project_licence.max_x,
            ),
            room(
                depth(y, v.top, v.bottom),
                self.screen_scroll(),
                self.max_scroll,
            ),
        ]
    }
    /// A frame: the long press timeout, the horizontal fling and the edge scroll advance.
    pub(in crate::settings_ui) fn project_tick(&mut self, now: i64) {
        if !self.on_project_licence() {
            return;
        }
        let touch = &self.project_licence.touch;
        if touch.gesture == Gesture::Pressed && now >= touch.down.2 + LONG_PRESS {
            self.project_hold();
        }
        if let Some(f) = self.project_licence.fling.take() {
            let value = f.value(now);
            let done = now >= f.start + f.duration
                || (f.distance < 0.0 && value <= 0.0)
                || (f.distance > 0.0 && value >= self.project_licence.max_x);
            self.set_project_x(value);
            if !done {
                self.project_licence.fling = Some(f);
            }
        }
        self.edge_scroll(now);
    }
    fn edge_scroll(&mut self, now: i64) {
        let [sx, sy] = self.edge_speed();
        if sx == 0.0 && sy == 0.0 {
            self.project_licence.touch.edge_time = 0;
            return;
        }
        let last = self.project_licence.touch.edge_time;
        self.project_licence.touch.edge_time = now;
        if last <= 0 || now <= last {
            return;
        }
        let seconds = (now - last) as f32 / 1e9;
        self.set_project_x(self.project_licence.scroll_x + sx * seconds);
        self.set_scroll(self.screen_scroll() + sy * seconds);
        // The text moved under the still finger.
        self.follow_finger();
    }
}
