//! Complete MotionEvent packets are validated at the boundary; pointer identity survives reordering.
use crate::scene::Scene;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Motion {
    pub action: i32,
    pub index: usize,
    pub ids: Vec<i32>,
    /// Down time followed by historical times and the current time, in uptime nanoseconds.
    pub times: Vec<i64>,
    /// Historical samples then current sample: x/y for every pointer in ids order.
    pub coords: Vec<f32>,
}

impl Motion {
    pub fn valid(&self) -> bool {
        !self.ids.is_empty()
            && self.ids.len() <= 32
            && self.index < self.ids.len()
            && self.times.len() >= 2
            // Injected MotionEvents can carry a fresh downTime alongside older batched samples.
            // Compose reads the sample clock; downTime is metadata, not a sample-order guard.
            && self.times[1..].windows(2).all(|t| t[0] <= t[1])
            && self.coords.len() == (self.times.len() - 1) * self.ids.len() * 2
            && self.coords.iter().all(|v| v.is_finite())
            && self
                .ids
                .iter()
                .enumerate()
                .all(|(i, id)| !self.ids[..i].contains(id))
    }
}

/// Wire action kind and two payload integers, defined in protocol.rs. Glide points use f32 bits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action(pub i32, pub i32, pub i32);

#[derive(Debug)]
pub struct Touch {
    search: crate::panel_touch::PanelTouch,
    search_active: bool,
    panel: crate::panel_touch::PanelTouch,
    keys: crate::keyboard_touch::KeyboardTouch,
    glide: crate::glide_touch::GlideTouch,
    bar: crate::smartbar_touch::BarTouch,
    candidates: BTreeMap<i32, usize>,
    pub config: crate::gesture_config::GestureConfig,
    pub long_press_delay: i64,
    trail: crate::trail::Trail,
    trail_time: i64,
}
impl Default for Touch {
    fn default() -> Self {
        Self {
            search: Default::default(),
            search_active: false,
            panel: Default::default(),
            keys: Default::default(),
            glide: Default::default(),
            bar: Default::default(),
            candidates: Default::default(),
            config: Default::default(),
            long_press_delay: 300_000_000,
            trail: Default::default(),
            trail_time: 0,
        }
    }
}
impl Touch {
    pub fn animate(&mut self, scene: &mut Scene, now: i64) -> bool {
        self.trail.frame(
            scene,
            now,
            self.config.glide && self.config.show_trail,
            self.config.trail_duration,
        )
    }
    fn trail_actions(&mut self, actions: &[Action]) {
        self.trail.actions(actions, self.trail_time);
    }
    pub fn configure(&mut self, config: crate::gesture_config::GestureConfig) {
        self.keys.set_shift(config.shift_pressed);
        self.config = config;
    }
    pub fn tooltip_shown(&mut self, pointer: i32, shown: bool) {
        self.bar.tooltip_shown(pointer, shown);
    }
    pub fn candidate_tokens(&self) -> Vec<i32> {
        self.bar.candidate_tokens()
    }
    pub fn next_deadline(&self) -> Option<i64> {
        self.bar
            .next_deadline()
            .into_iter()
            .chain(self.panel.deadline())
            .min()
    }
    pub fn tick(&mut self, scene: &mut Scene, now: i64) -> Vec<Action> {
        let mut actions = self.bar.tick(scene, now);
        if let Some(p) = &mut scene.panel {
            actions.extend(self.panel.tick(p, now));
        }
        actions
    }
    pub fn invalidate_bar(&mut self, scene: &mut Scene, slots: &[i32]) -> Vec<Action> {
        self.bar.invalidate(scene, slots)
    }
    pub fn candidate_long_press(&mut self, scene: &mut Scene, pointer: i32) {
        self.bar.candidate_long_press(scene, pointer);
    }
    pub fn long_press(
        &mut self,
        scene: &Scene,
        pointer: i32,
        popup: bool,
        uninterrupted: bool,
    ) -> Vec<Action> {
        self.keys
            .long_press(scene, &self.config, pointer, popup, uninterrupted)
    }
    pub fn cancel(&mut self, scene: &mut Scene) -> Vec<Action> {
        let mut actions = self.glide.cancel();
        if let Some(p) = &mut scene.panel {
            actions.extend(self.panel.cancel(p));
        }
        if let Some(p) = &mut scene.search {
            actions.extend(self.search.cancel(p));
        }
        self.search_active = false;
        actions.extend(self.keys.cancel(scene));
        actions.extend(self.bar.cancel(scene));
        self.candidates.clear();
        scene.candidate_pressed.clear();
        actions
    }
    pub fn event(&mut self, scene: &mut Scene, event: &Motion) -> Vec<Action> {
        self.trail_time = event.times.last().copied().unwrap_or(self.trail_time);
        if event.action == 3 || !event.valid() {
            let actions = self.cancel(scene);
            self.trail_actions(&actions);
            return actions;
        }
        let mut actions = Vec::new();
        if event.action == 0 {
            actions.extend(self.cancel(scene));
        }
        if let Some(p) = &mut scene.search
            && (event.action == 0 && event.coords.last().copied().unwrap_or(f32::MAX) < scene.strip
                || self.search_active)
        {
            self.search_active = event.action != 1 && event.action != 3;
            actions.extend(self.search.event(p, event, self.long_press_delay));
            return actions;
        }
        if let Some(p) = &mut scene.panel {
            actions.extend(self.panel.event(p, event, self.long_press_delay));
            // The smartbar stays usable above the overflow. Editor covers it.
            if p.kind == 2 {
                actions.extend(self.bar.event(scene, event, self.long_press_delay));
            }
            return actions;
        }
        self.keys.velocity_event(event);
        let (glide, intercepted) = self.glide.event(scene, &self.keys, event, &self.config);
        actions.extend(glide);
        if intercepted {
            actions.extend(self.keys.cancel_active(scene));
            if event.action == 1 {
                actions.extend(self.keys.cancel(scene));
            }
            self.trail_actions(&actions);
            return actions;
        }
        actions.extend(self.bar.event(scene, event, self.long_press_delay.max(0)));
        actions.extend(self.keys.event(scene, event, &self.config));
        // Legacy candidate-only previews have no smartbar action table.
        if scene.smartbar.is_none() {
            self.candidate_event(scene, event, &mut actions);
        }
        self.bar.mark(scene);
        self.trail_actions(&actions);
        actions
    }
    fn candidate_event(&mut self, scene: &mut Scene, event: &Motion, actions: &mut Vec<Action>) {
        let current = event.coords.len() - event.ids.len() * 2;
        let id = event.ids[event.index];
        let x = event.coords[current + event.index * 2];
        let y = event.coords[current + event.index * 2 + 1];
        match event.action {
            0 | 5 => {
                if let Some(i) = scene.candidate_at(x, y) {
                    self.candidates.insert(id, i);
                }
            }
            2 => {
                for (i, &id) in event.ids.iter().enumerate() {
                    if self.candidates.get(&id).is_some_and(|&c| {
                        scene.candidate_at(
                            event.coords[current + i * 2],
                            event.coords[current + i * 2 + 1],
                        ) != Some(c)
                    }) {
                        self.candidates.remove(&id);
                    }
                }
            }
            1 | 6 => {
                if let Some(i) = self.candidates.remove(&id)
                    && scene.candidate_at(x, y) == Some(i)
                {
                    actions.push(Action(crate::protocol::CANDIDATE_COMMIT, i as i32, id));
                }
            }
            _ => {}
        }
        scene.candidate_pressed = self.candidates.values().copied().collect();
    }
}
