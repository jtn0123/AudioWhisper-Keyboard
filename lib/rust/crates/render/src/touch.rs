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
            && self.times.windows(2).all(|t| t[0] <= t[1])
            && self.coords.len() == (self.times.len() - 1) * self.ids.len() * 2
            && self.coords.iter().all(|v| v.is_finite())
            && self
                .ids
                .iter()
                .enumerate()
                .all(|(i, id)| !self.ids[..i].contains(id))
    }
}

/// Actions: 0 down, 1 up, 2 cancel, 3 popup selection, 4 candidate, with key/choice and pointer id.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Action(pub i32, pub i32, pub i32);

#[derive(Debug, Default)]
pub struct Touch {
    pointers: BTreeMap<i32, usize>,
    bar: crate::smartbar_touch::BarTouch,
}
const OUTSIDE: usize = usize::MAX;

impl Touch {
    pub fn cancel(&mut self, scene: &mut Scene) -> Vec<Action> {
        let mut result: Vec<_> = self
            .pointers
            .iter()
            .filter(|(_, key)| **key < scene.keys.len())
            .map(|(&id, &key)| Action(2, key as i32, id))
            .collect();
        result.extend(self.bar.cancel(scene));
        self.pointers.clear();
        scene.pressed.clear();
        scene.candidate_pressed.clear();
        scene.popup = None;
        result
    }

    pub fn event(&mut self, scene: &mut Scene, event: &Motion) -> Vec<Action> {
        if event.action == 3 {
            return self.cancel(scene);
        }
        let mut actions = Vec::new();
        if !event.valid() {
            return self.cancel(scene);
        }
        if event.action == 0 {
            actions.extend(self.cancel(scene));
        }
        actions.extend(self.bar.event(scene, event));
        let current = event.coords.len() - event.ids.len() * 2;
        let id = event.ids[event.index];
        let x = event.coords[current + event.index * 2];
        let y = event.coords[current + event.index * 2 + 1];
        match event.action {
            0 | 5 => {
                if event.action != 0 {
                    // Compose commits active character keys before a rolling second finger
                    // goes down, while retaining held modifiers such as Shift.
                    let released: Vec<_> = self
                        .pointers
                        .iter()
                        .filter_map(|(&id, &index)| {
                            scene
                                .keys
                                .get(index)
                                .filter(|key| {
                                    key.code > keys::codes::SPACE
                                        && key.code != keys::codes::CJK_SPACE
                                })
                                .map(|_| (id, index))
                        })
                        .collect();
                    for (id, index) in released {
                        self.pointers.remove(&id);
                        actions.push(Action(1, index as i32, id));
                    }
                    scene.popup = None;
                }
                if let Some(key) = scene.key_at(x, y) {
                    if let Some(old) = self.pointers.insert(id, key) {
                        actions.push(Action(2, old as i32, id));
                    }
                    actions.push(Action(0, key as i32, id));
                } else if scene.smartbar.is_some() && y < scene.strip {
                    // Smartbar owns this gesture, even when it later leaves the strip.
                } else if let Some(choice) = scene
                    .candidate_at(x, y)
                    .filter(|_| scene.smartbar.is_none())
                {
                    // Keep the candidate identity from DOWN, rather than accepting an UP from outside.
                    self.pointers.insert(id, scene.keys.len() + choice);
                } else {
                    self.pointers.insert(id, OUTSIDE);
                }
            }
            2 => {
                for sample in event.coords.chunks_exact(event.ids.len() * 2) {
                    for (p, &pointer) in event.ids.iter().enumerate() {
                        if let (Some(popup), Some(bounds)) =
                            (scene.popup.as_ref(), scene.popup_bounds())
                            && self.pointers.get(&pointer) == Some(&popup.key)
                        {
                            let selected = ((sample[p * 2] - bounds.left) / bounds.width()
                                * popup.labels.len() as f32)
                                .floor()
                                .max(0.0) as usize;
                            if let Some(popup) = scene.popup.as_mut() {
                                popup.selected = selected.min(popup.labels.len().saturating_sub(1));
                            }
                            continue;
                        }
                        let Some(&old) = self.pointers.get(&pointer) else {
                            continue;
                        };
                        if old >= scene.keys.len() && old != OUTSIDE {
                            if scene.candidate_at(sample[p * 2], sample[p * 2 + 1])
                                != Some(old - scene.keys.len())
                            {
                                self.pointers.remove(&pointer);
                            }
                            continue;
                        }
                        let new = scene.key_at(sample[p * 2], sample[p * 2 + 1]);
                        if new.unwrap_or(OUTSIDE) != old {
                            self.pointers.insert(pointer, new.unwrap_or(OUTSIDE));
                            if old != OUTSIDE {
                                actions.push(Action(2, old as i32, pointer));
                            }
                            if let Some(new) = new {
                                self.pointers.insert(pointer, new);
                                actions.push(Action(0, new as i32, pointer));
                            }
                        }
                    }
                }
            }
            1 | 6 => {
                if let Some(key) = self.pointers.remove(&id) {
                    if key == OUTSIDE {
                        // An outside pointer can return on MOVE, but releasing it commits nothing.
                    } else if key >= scene.keys.len() {
                        if scene.candidate_at(x, y) == Some(key - scene.keys.len()) {
                            actions.push(Action(4, (key - scene.keys.len()) as i32, id));
                        }
                    } else if let Some(popup) = scene.popup.take() {
                        if popup.key == key {
                            actions.push(Action(3, popup.selected as i32, id));
                        } else {
                            scene.popup = Some(popup);
                            actions.push(Action(1, key as i32, id));
                        }
                    } else {
                        actions.push(Action(1, key as i32, id));
                    }
                }
                if event.action == 1 {
                    actions.extend(self.cancel(scene));
                }
            }
            _ => actions.extend(self.cancel(scene)),
        }
        scene.pressed = self
            .pointers
            .values()
            .filter(|&&k| k < scene.keys.len())
            .copied()
            .collect();
        scene.candidate_pressed = self
            .pointers
            .values()
            .filter(|&&k| k >= scene.keys.len() && k != OUTSIDE)
            .map(|k| k - scene.keys.len())
            .collect();
        self.bar.mark(scene);
        actions
    }
}
