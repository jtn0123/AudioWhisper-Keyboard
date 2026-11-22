//! Compose TextKeyboardLayoutController's pointer lifecycle. Timed key dispatch stays in the
//! shared InputEventDispatcher; its long-press callback reports back to this owner.
use crate::{
    gesture_config::GestureConfig,
    protocol::*,
    scene::Scene,
    touch::{Action, Motion},
    velocity::Velocity,
};
use gestures::swipe::Swipes;
use keys::codes::*;
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub(crate) struct Pointer {
    pub id: i32,
    slot: usize,
    pub initial: Option<usize>,
    pub active: Option<usize>,
    pub moved: bool,
    pub long: bool,
    pub mass: bool,
}
#[derive(Debug, Default)]
pub(crate) struct KeyboardTouch {
    pub pointers: Vec<Pointer>,
    swipes: Swipes,
    velocity: Velocity,
    pressed_codes: BTreeSet<i32>,
}
impl KeyboardTouch {
    pub fn cancel_active(&mut self, scene: &mut Scene) -> Vec<Action> {
        let mut actions = Vec::new();
        for p in &mut self.pointers {
            if p.active.is_some() {
                Self::release(scene, p, true, &mut actions);
            }
        }
        scene.pressed.clear();
        scene.popup = None;
        scene.bubble = None;
        Self::record(&mut self.pressed_codes, scene, &actions);
        actions
    }
    pub fn cancel(&mut self, scene: &mut Scene) -> Vec<Action> {
        let mut actions = Vec::new();
        for p in &mut self.pointers {
            Self::release(scene, p, true, &mut actions);
        }
        scene.pressed.clear();
        scene.popup = None;
        scene.bubble = None;
        self.pointers.clear();
        self.swipes.reset();
        self.pressed_codes.clear();
        actions
    }
    pub fn long_press(
        &mut self,
        scene: &Scene,
        config: &GestureConfig,
        id: i32,
        popup: bool,
        uninterrupted: bool,
    ) -> Vec<Action> {
        let Some(p) = self.pointers.iter_mut().find(|p| p.id == id) else {
            return Vec::new();
        };
        let Some(key) = p.active.and_then(|k| scene.keys.get(k)) else {
            return Vec::new();
        };
        p.long = true;
        let mut actions = Vec::new();
        use input::protocol as ip;
        let plan = input::pointer::hold(
            key.code,
            popup,
            uninterrupted,
            !["NO_ACTION", "INSERT_SPACE"].contains(&config.action(PREF_SPACE_HOLD as usize)),
        );
        if plan & ip::HOLD_SPACE != 0 {
            actions.push(Action(SWIPE_ACTION, PREF_SPACE_HOLD, id));
        }
        if plan & ip::HOLD_CAPS != 0 {
            actions.push(Action(crate::protocol::CAPS_LOCK, 0, id));
        }
        if plan & ip::HOLD_PICKER != 0 {
            actions.push(Action(INPUT_METHOD_PICKER, 0, id));
        }
        if plan & ip::HOLD_POPUP != 0 {
            actions.push(Action(OPEN_POPUP, p.active.expect("active key") as i32, id));
        }
        if plan & ip::HOLD_FEEDBACK != 0 {
            actions.push(Action(LONG_PRESS_FEEDBACK, key.code, id));
        }
        if plan & ip::HOLD_BLOCK != 0 {
            actions.push(Action(LONG_PRESS_BLOCK, 0, id));
        }
        actions
    }
    pub fn event(
        &mut self,
        scene: &mut Scene,
        event: &Motion,
        config: &GestureConfig,
    ) -> Vec<Action> {
        let mut actions = Vec::new();
        let current = event.coords.len() - event.ids.len() * 2;
        let id = event.ids[event.index];
        let x = event.coords[current + event.index * 2];
        let y = event.coords[current + event.index * 2 + 1];
        match event.action {
            0 | 5 => {
                if event.action == 5 {
                    if let Some(i) = self.pointers.iter().position(|p| p.id == id) {
                        let mut p = self.pointers.remove(i);
                        Self::release(scene, &mut p, true, &mut actions);
                        self.swipes.cancel(id);
                    }
                    for p in &mut self.pointers {
                        let still = !p.moved && !p.long;
                        if p.active
                            .and_then(|k| scene.keys.get(k))
                            .is_some_and(|k| input::pointer::rolls(k.code, still))
                        {
                            self.swipes.cancel(p.id);
                            Self::release(scene, p, false, &mut actions);
                        }
                    }
                }
                // The Compose controller has four pointer slots; smartbar input is a separate owner.
                if self.pointers.len() < 4 && y >= scene.strip {
                    let key = scene.aimed_key_at(x, y);
                    self.swipes.down(id, x, y, config.density);
                    let slot = (0..4)
                        .find(|slot| self.pointers.iter().all(|p| p.slot != *slot))
                        .expect("free pointer slot");
                    let mut p = Pointer {
                        id,
                        slot,
                        initial: None,
                        active: None,
                        moved: false,
                        long: false,
                        mass: false,
                    };
                    Self::down(&mut p, key, &mut actions);
                    self.pointers.push(p);
                    self.pointers.sort_by_key(|p| p.slot);
                }
            }
            2 => {
                for (i, &id) in event.ids.iter().enumerate() {
                    let shift = self.pressed_codes.contains(&SHIFT);
                    let start = actions.len();
                    let Some(p) = self.pointers.iter_mut().find(|p| p.id == id) else {
                        continue;
                    };
                    let x = event.coords[current + i * 2];
                    let y = event.coords[current + i * 2 + 1];
                    let code = p
                        .initial
                        .and_then(|k| scene.keys.get(k))
                        .map(|k| k.code)
                        .unwrap_or(0);
                    let always = p.moved
                        && (code == SPACE
                            || code == CJK_SPACE
                            || (code == DELETE
                                && ["DELETE_CHARACTERS_PRECISELY", "SELECT_CHARACTERS_PRECISELY"]
                                    .contains(&config.action(PREF_DELETE_LEFT as usize))));
                    let handled = self
                        .swipes
                        .moved(id, x, y, config.density, config.distance, always)
                        .is_some_and(|swipe| {
                            crate::keyboard_swipe::handle(
                                scene,
                                p,
                                &swipe,
                                false,
                                config,
                                shift,
                                &mut actions,
                            )
                        });
                    if handled || p.moved {
                        p.moved = true;
                        if let Some(key) = p.active {
                            actions.push(Action(KEY_CANCEL, key as i32, id));
                        }
                    } else {
                        Self::moved(scene, p, x, y, &mut actions);
                    }
                    Self::record(&mut self.pressed_codes, scene, &actions[start..]);
                }
            }
            1 | 6 => {
                let pointers = std::mem::take(&mut self.pointers);
                for mut p in pointers {
                    if p.id == id {
                        let swipe = self.swipes.up(
                            id,
                            (x, y),
                            self.velocity.get(id),
                            config.density,
                            config.distance,
                            config.velocity,
                        );
                        let handled = swipe.is_some_and(|s| {
                            crate::keyboard_swipe::handle(
                                scene,
                                &mut p,
                                &s,
                                true,
                                config,
                                false,
                                &mut actions,
                            )
                        });
                        let cancel = handled || p.moved;
                        if cancel
                            && p.moved
                            && p.initial
                                .and_then(|k| scene.keys.get(k))
                                .is_some_and(|k| k.code == DELETE)
                            && (event.action == 6
                                || !["SELECT_CHARACTERS_PRECISELY", "SELECT_WORDS_PRECISELY"]
                                    .contains(&config.action(PREF_DELETE_LEFT as usize)))
                            && !config.gif
                        {
                            actions.push(Action(DELETE_SELECTION, 0, id));
                        }
                        Self::release(scene, &mut p, cancel, &mut actions);
                    } else if event.action == 1 {
                        self.swipes.cancel(p.id);
                        Self::release(scene, &mut p, true, &mut actions);
                    } else {
                        self.pointers.push(p);
                    }
                }
            }
            _ => actions.extend(self.cancel(scene)),
        }
        scene.pressed = self.pointers.iter().filter_map(|p| p.active).collect();
        self.hold_bubble(scene, config.bubble);
        Self::record(&mut self.pressed_codes, scene, &actions);
        actions
    }
    /// The bubble follows the key a finger holds that shows one, while the setting (`on`) is.
    /// A character key goes up when another finger comes down (`input::pointer::rolls`), so at
    /// most one finger holds such a key. The bubble is kept, styles and all, while that key is.
    fn hold_bubble(&self, scene: &mut Scene, on: bool) {
        let key = self
            .pointers
            .iter()
            .filter_map(|p| p.active)
            .find(|&k| on && scene.bubbles.get(k) == Some(&true));
        if scene.bubble.as_ref().map(|b| b.key) != key {
            scene.bubble = key.map(crate::press_bubble::Bubble::new);
        }
    }
    pub fn set_shift(&mut self, pressed: bool) {
        if pressed {
            self.pressed_codes.insert(SHIFT);
        } else {
            self.pressed_codes.remove(&SHIFT);
        }
    }
    fn record(pressed: &mut BTreeSet<i32>, scene: &Scene, actions: &[Action]) {
        for &Action(kind, index, _) in actions {
            if kind == KEY_CANCEL_CODE {
                pressed.remove(&index);
            } else if let Some(key) = scene.keys.get(index as usize) {
                if kind == KEY_DOWN {
                    pressed.insert(key.code);
                } else if kind == KEY_UP || kind == KEY_CANCEL {
                    pressed.remove(&key.code);
                }
            }
        }
    }
    pub fn velocity_event(&mut self, event: &Motion) {
        self.velocity.event(event);
    }
    fn down(p: &mut Pointer, key: Option<usize>, actions: &mut Vec<Action>) {
        if let Some(key) = key {
            if p.initial.is_none() {
                p.initial = Some(key);
            }
            p.active = Some(key);
            actions.push(Action(KEY_DOWN, key as i32, p.id));
        } else {
            p.active = None;
        }
    }
    fn moved(scene: &mut Scene, p: &mut Pointer, x: f32, y: f32, actions: &mut Vec<Action>) {
        let Some(key) = p.active.and_then(|k| scene.keys.get(k)) else {
            return;
        };
        if scene
            .popup
            .as_ref()
            .is_some_and(|pop| Some(pop.key) == p.active)
        {
            if let Some(selected) = scene.popup_pointer(x, y) {
                scene.popup.as_mut().expect("popup exists").selected = selected;
            } else {
                Self::release(scene, p, true, actions);
                Self::down(p, scene.key_at(x, y), actions);
            }
        } else {
            let r = key.bounds.visible;
            let y = y - scene.strip;
            if input::pointer::slide(x, y, r.left, r.top, r.right, r.bottom) {
                Self::release(scene, p, true, actions);
                Self::down(p, scene.key_at(x, y + scene.strip), actions);
            }
        }
    }
    fn release(scene: &mut Scene, p: &mut Pointer, cancel: bool, actions: &mut Vec<Action>) {
        if p.mass {
            p.mass = false;
            actions.push(Action(MASS_END, 0, p.id));
        }
        if let Some(key) = p.active.take() {
            let popup = scene.popup.as_ref().filter(|pop| pop.key == key);
            if !cancel && let Some(popup) = popup {
                actions.push(Action(
                    POPUP_SELECT,
                    scene.popup_choice_index(popup.selected) as i32,
                    p.id,
                ));
            } else {
                actions.push(Action(
                    if cancel || p.moved {
                        KEY_CANCEL
                    } else {
                        KEY_UP
                    },
                    key as i32,
                    p.id,
                ));
            }
            if scene.popup.as_ref().is_some_and(|pop| pop.key == key) {
                scene.popup = None;
            }
        }
        p.moved = false;
    }
}
