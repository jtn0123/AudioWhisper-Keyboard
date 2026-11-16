//! Native glide ownership and ordered callbacks into the existing GlideTypingManager.
use crate::{
    gesture_config::GestureConfig,
    keyboard_touch::KeyboardTouch,
    protocol::*,
    scene::Scene,
    touch::{Action, Motion},
};
use gestures::glide_start::{GlideStart, Touch};
#[derive(Debug)]
pub(crate) struct GlideTouch {
    detector: GlideStart,
    key_size: f32,
    /// Each point and its time, in microseconds after the touch went down.
    points: Vec<(f32, f32, i32)>,
    owns: bool,
    tracked_id: i32,
    active: bool,
    cancelled_outside: bool,
}
impl Default for GlideTouch {
    fn default() -> Self {
        Self {
            detector: GlideStart::new(32.0),
            key_size: 32.0,
            points: Vec::new(),
            owns: false,
            tracked_id: -1,
            active: false,
            cancelled_outside: false,
        }
    }
}
impl GlideTouch {
    pub fn cancel(&mut self) -> Vec<Action> {
        let out = if self.active {
            vec![Action(GLIDE_CANCEL, 0, 0)]
        } else {
            Vec::new()
        };
        self.detector = GlideStart::new(self.key_size);
        self.points.clear();
        self.active = false;
        self.owns = false;
        self.cancelled_outside = false;
        out
    }
    pub fn event(
        &mut self,
        scene: &Scene,
        keys: &KeyboardTouch,
        event: &Motion,
        c: &GestureConfig,
    ) -> (Vec<Action>, bool) {
        if !c.glide {
            return (self.cancel(), false);
        }
        if self.key_size != c.key_size {
            self.key_size = c.key_size;
            self.detector = GlideStart::new(c.key_size);
        }
        let current = event.coords.len() - event.ids.len() * 2;
        let x = event.coords[current + event.index * 2];
        let y = event.coords[current + event.index * 2 + 1];
        if event.action == 0 {
            self.owns = scene.key_at(x, y).is_some();
            self.tracked_id = event.ids[event.index];
            self.cancelled_outside = false;
        }
        if !self.owns {
            return (Vec::new(), false);
        }
        if self.cancelled_outside {
            return (Vec::new(), true);
        }
        let pointer = keys.pointers.iter().find(|p| p.id == 0);
        if pointer.is_some_and(|p| p.long) {
            return (Vec::new(), false);
        }
        let initial = pointer
            .and_then(|p| p.initial)
            .and_then(|i| scene.keys.get(i))
            .map(|k| k.code);
        // Android can extrapolate the current sample beyond the surface while the batch's
        // genuine historical points remain inside. Compose continues that glide. Confirm an
        // exit from an entirely outside MOVE batch, or an outside release of the tracked finger.
        // Delete, Shift and space retain their own Compose swipe policies.
        if event.ids[event.index] == self.tracked_id
            && matches!(event.action, 1 | 2 | 6)
            && !initial
                .is_some_and(|code| gestures::glide_start::SWIPE_GESTURE_KEYS.contains(&code))
            && event
                .coords
                .chunks_exact(event.ids.len() * 2)
                .all(|sample| {
                    let x = sample[event.index * 2];
                    let y = sample[event.index * 2 + 1];
                    x < 0.0
                        || x >= scene.width as f32
                        || y < scene.strip
                        || y >= scene.height as f32
                })
        {
            let actions = self.cancel();
            self.cancelled_outside = true;
            self.owns = true;
            return (actions, true);
        }
        let mut points = Vec::new();
        let mut times = Vec::new();
        if matches!(event.action, 0 | 5) {
            points.extend([x, y - scene.strip]);
            times.push(since_down(event, event.times.len().saturating_sub(1)));
        } else if event.action == 2 {
            let samples = event.coords.chunks_exact(event.ids.len() * 2);
            let count = samples.len();
            for (i, sample) in samples.enumerate() {
                points.extend([
                    sample[event.index * 2],
                    sample[event.index * 2 + 1] - scene.strip,
                ]);
                // Down time, then one time a sample; the last time for any without one.
                let at = if event.times.len() == count + 1 {
                    i + 1
                } else {
                    event.times.len().saturating_sub(1)
                };
                times.push(since_down(event, at));
            }
        }
        let touch = Touch::of(event.action, event.ids[event.index], &points)
            .expect("validated event points");
        let outcome = self.detector.touch(
            touch,
            event.times.last().copied().unwrap_or(0) / 1_000_000,
            c.density,
            initial,
        );
        if outcome.cleared_before {
            self.points.clear();
        }
        if outcome.took {
            let taken = points.as_chunks::<2>().0.iter().zip(times);
            self.points.extend(taken.map(|(p, t)| (p[0], p[1], t)));
        }
        let mut actions = Vec::new();
        if let Some(from) = outcome.emit_from {
            self.active = true;
            for &(x, y, t) in &self.points[from..] {
                if c.glide_times {
                    actions.push(Action(GLIDE_TIME, t, 0));
                }
                actions.push(Action(GLIDE_POINT, x.to_bits() as i32, y.to_bits() as i32));
            }
        }
        if outcome.complete {
            actions.push(Action(GLIDE_COMPLETE, 0, 0));
            self.active = false;
        }
        if outcome.cancelled {
            actions.push(Action(GLIDE_CANCEL, 0, 0));
            self.active = false;
        }
        if outcome.cleared_after {
            self.points.clear();
        }
        (actions, outcome.gesture)
    }
}

/// The time of `event`'s `at`th time after its down time (its first), in microseconds, as far as
/// an `i32` holds.
fn since_down(event: &Motion, at: usize) -> i32 {
    let (Some(&down), Some(&time)) = (event.times.first(), event.times.get(at)) else {
        return 0;
    };
    let micros = time.saturating_sub(down) / 1_000;
    micros.clamp(0, i64::from(i32::MAX)) as i32
}
