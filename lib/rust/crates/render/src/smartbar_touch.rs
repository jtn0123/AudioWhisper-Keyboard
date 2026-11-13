//! Smartbar pointer ownership: no sliding from a key into a toolbar action.
use crate::{
    scene::Scene,
    touch::{Action, Motion},
};
use std::collections::BTreeMap;
#[derive(Debug, Default)]
pub struct BarTouch {
    pointers: BTreeMap<i32, Pointer>,
}
#[derive(Debug)]
struct Pointer {
    action: bool,
    id: usize,
    x: f32,
    scrolling: bool,
    highlighted: bool,
}
impl BarTouch {
    pub fn cancel(&mut self, scene: &mut Scene) -> Vec<Action> {
        let actions = self
            .pointers
            .iter()
            .map(|(&p, t)| Action(if t.action { 7 } else { 10 }, t.id as i32, p))
            .collect();
        self.pointers.clear();
        self.mark(scene);
        actions
    }
    pub fn event(&mut self, scene: &mut Scene, event: &Motion) -> Vec<Action> {
        let Some(bar) = &mut scene.smartbar else {
            return Vec::new();
        };
        let mut actions = Vec::new();
        let current = event.coords.len() - event.ids.len() * 2;
        let id = event.ids[event.index];
        let x = event.coords[current + event.index * 2];
        let y = event.coords[current + event.index * 2 + 1];
        match event.action {
            0 | 5 => {
                if let Some((action, target)) = bar.layout.hit(x, y) {
                    self.pointers.insert(
                        id,
                        Pointer {
                            action,
                            id: target,
                            x,
                            scrolling: false,
                            highlighted: true,
                        },
                    );
                    actions.push(Action(if action { 5 } else { 8 }, target as i32, id));
                }
            }
            2 => {
                for (p, &id) in event.ids.iter().enumerate() {
                    if let Some(t) = self.pointers.get_mut(&id) {
                        let nx = event.coords[current + p * 2];
                        let ny = event.coords[current + p * 2 + 1];
                        if !t.action
                            && bar.config.mode == 2
                            && (nx - t.x).abs() > 8.0 * bar.config.density
                        {
                            if !t.scrolling {
                                actions.push(Action(10, t.id as i32, id));
                            }
                            t.scrolling = true;
                        }
                        if t.scrolling {
                            let max =
                                (bar.layout.content_width - bar.layout.center.width()).max(0.0);
                            bar.scroll = (bar.scroll + t.x - nx).clamp(0.0, max);
                            bar.relayout(scene.width as f32);
                            t.x = nx;
                        } else if bar.layout.hit(nx, ny) != Some((t.action, t.id)) {
                            let t = self.pointers.remove(&id).expect("owned pointer");
                            actions.push(Action(if t.action { 7 } else { 10 }, t.id as i32, id));
                        }
                    }
                }
            }
            1 | 6 => {
                if let Some(t) = self.pointers.remove(&id) {
                    let valid = !t.scrolling && bar.layout.hit(x, y) == Some((t.action, t.id));
                    actions.push(Action(
                        if t.action {
                            if valid { 6 } else { 7 }
                        } else if valid {
                            9
                        } else {
                            10
                        },
                        t.id as i32,
                        id,
                    ));
                }
            }
            _ => {}
        }
        self.mark(scene);
        actions
    }
    pub fn candidate_long_press(&mut self, scene: &mut Scene, pointer: i32) {
        if let Some(held) = self.pointers.get_mut(&pointer)
            && !held.action
        {
            held.highlighted = false;
        }
        self.mark(scene);
    }
    pub fn mark(&self, scene: &mut Scene) {
        if let Some(bar) = &mut scene.smartbar {
            bar.pressed = self
                .pointers
                .values()
                .filter(|t| t.action)
                .map(|t| t.id)
                .collect();
            scene.candidate_pressed = self
                .pointers
                .values()
                .filter(|t| !t.action && !t.scrolling && t.highlighted)
                .map(|t| t.id)
                .collect();
        }
    }
}
