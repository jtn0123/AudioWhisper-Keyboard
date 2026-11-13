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
    points: Vec<(f32, f32)>,
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
        if matches!(event.action, 0 | 5) {
            points.extend([x, y - scene.strip]);
        } else if event.action == 2 {
            for sample in event.coords.chunks_exact(event.ids.len() * 2) {
                points.extend([
                    sample[event.index * 2],
                    sample[event.index * 2 + 1] - scene.strip,
                ]);
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
            self.points
                .extend(points.as_chunks::<2>().0.iter().map(|p| (p[0], p[1])));
        }
        let mut actions = Vec::new();
        if let Some(from) = outcome.emit_from {
            self.active = true;
            actions.extend(
                self.points[from..]
                    .iter()
                    .map(|&(x, y)| Action(GLIDE_POINT, x.to_bits() as i32, y.to_bits() as i32)),
            );
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
