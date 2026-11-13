//! Semantic scene and activation policy. No Android types, speech, logging or editor contents.
use crate::{accessibility_labels::Labels, protocol::*, scene::Scene, touch::Action};
use keys::{Rect, codes};

#[derive(Clone, Debug, Default)]
pub struct Semantics {
    pub labels: Labels,
    pub popup_keys: Vec<bool>,
    pub action_text: Vec<String>,
    pub checked_codes: Vec<i32>,
    pub removable: Vec<bool>,
    pub popup_source: Option<usize>,
    pub editor: [i32; 2],
}
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    Key(usize),
    Bar(usize),
    Candidate(usize),
    Popup(usize),
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: i32,
    pub bounds: Rect,
    pub label: String,
    pub description: String,
    pub role: &'static str,
    pub state: String,
    pub enabled: bool,
    pub pressed: bool,
    pub selected: bool,
    pub checked: Option<bool>,
    pub locked: bool,
    pub actions: Vec<i32>,
    pub long_click_label: String,
    pub target: Target,
}
impl Node {
    fn new(id: i32, bounds: Rect, label: String, role: &'static str, target: Target) -> Self {
        Self {
            id,
            bounds,
            description: label.clone(),
            label,
            role,
            state: String::new(),
            enabled: true,
            pressed: false,
            selected: false,
            checked: None,
            locked: false,
            actions: vec![ACCESS_CLICK],
            long_click_label: String::new(),
            target,
        }
    }
}
impl Semantics {
    /// Key ids are physical scene slots, independent of case/code/label. Bar ids are configured
    /// action slots. Candidate ids use the host's opaque object tokens, never a text hash.
    /// Popup ids include their source slot, and survive selected-item changes.
    pub fn snapshot(&self, scene: &Scene, shift: &str) -> Vec<Node> {
        let mut nodes = Vec::new();
        for (i, k) in scene.keys.iter().enumerate() {
            if k.code == codes::UNSPECIFIED || k.code == codes::NOOP {
                continue;
            }
            let mut bounds = k.bounds.touch;
            bounds.top += scene.strip;
            bounds.bottom += scene.strip;
            // Touch targets intentionally extend beyond the last row for ordinary input.
            // Virtual keys must stay inside the view, away from system navigation controls.
            bounds.left = bounds.left.max(0.0);
            bounds.top = bounds.top.max(0.0);
            bounds.right = bounds.right.min(scene.width as f32);
            bounds.bottom = bounds.bottom.min(scene.height as f32);
            if bounds.right <= bounds.left || bounds.bottom <= bounds.top {
                continue;
            }
            let mut n = Node::new(
                i as i32 + 1,
                bounds,
                if k.code == codes::ENTER && k.label.is_empty() {
                    self.labels.enter(self.editor[0], self.editor[1])
                } else {
                    self.labels.key(k.code, &k.label, shift)
                },
                "key",
                Target::Key(i),
            );
            n.enabled = k.enabled;
            n.pressed = scene.pressed.contains(&i);
            if k.code == codes::SHIFT {
                n.checked = Some(shift != "unshifted");
                n.locked = shift == "caps_lock";
                n.state = self.labels.shift(shift);
            }
            if self.popup_keys.get(i) == Some(&true) || k.code == codes::SHIFT {
                n.actions.push(ACCESS_LONG_CLICK);
            }
            if !n.enabled {
                n.actions.clear();
            }
            nodes.push(n);
        }
        if let Some(bar) = &scene.smartbar {
            for slot in &bar.layout.actions {
                let Some(a) = bar.actions.get(slot.id) else {
                    continue;
                };
                let toggle = slot.id == BAR_TOGGLE_INDEX as usize;
                let expanded = if bar.config.layout == 3 {
                    bar.config.extended
                } else {
                    bar.config.shared
                };
                let label = if toggle {
                    if expanded {
                        self.labels.get("access_collapse", "Collapse actions")
                    } else {
                        self.labels.get("access_expand", "Expand actions")
                    }
                } else {
                    self.labels.action(
                        a.code,
                        self.action_text.get(slot.id).unwrap_or(&a.label),
                        a.lit,
                    )
                };
                let checkable = toggle
                    || [
                        codes::TOGGLE_INCOGNITO_MODE,
                        codes::TOGGLE_AUTOCORRECT,
                        codes::TOGGLE_COMPACT_LAYOUT,
                    ]
                    .contains(&a.code);
                let mut n = Node::new(
                    2000 + slot.id as i32,
                    slot.bounds,
                    label,
                    if checkable { "toggle" } else { "button" },
                    Target::Bar(slot.id),
                );
                n.enabled = a.enabled;
                n.pressed = bar.pressed.contains(&slot.id);
                n.checked = checkable.then_some(if toggle {
                    expanded
                } else {
                    self.checked_codes.contains(&a.code)
                });
                if a.code == codes::READ_ALOUD && a.lit != 2 {
                    n.actions.push(ACCESS_LONG_CLICK);
                    n.long_click_label = self.labels.get(
                        "quick_action__read_aloud__from_top",
                        "Read from the top of the page",
                    );
                }
                if !n.enabled {
                    n.actions.clear();
                }
                nodes.push(n);
            }
            for slot in &bar.layout.candidates {
                let Some(text) = scene.candidates.get(slot.id) else {
                    continue;
                };
                let mut r = slot.bounds;
                r.left = r.left.max(bar.layout.center.left);
                r.right = r.right.min(bar.layout.center.right);
                if r.right <= r.left {
                    continue;
                }
                let token = scene
                    .candidate_tokens
                    .get(slot.id)
                    .copied()
                    .unwrap_or(slot.id as i32);
                let mut n = Node::new(
                    -2 - token,
                    r,
                    text.clone(),
                    "candidate",
                    Target::Candidate(slot.id),
                );
                if let Some((_, secondary)) = bar.aux.get(slot.id)
                    && !secondary.is_empty()
                {
                    n.description = format!("{text}, {secondary}");
                }
                n.pressed = scene.candidate_pressed.contains(&slot.id);
                if self.removable.get(slot.id) == Some(&true) {
                    n.actions.push(ACCESS_LONG_CLICK);
                }
                nodes.push(n);
            }
        }
        if let Some(p) = &scene.popup {
            for (i, label) in p.labels.iter().enumerate() {
                if let Some(bounds) = scene.popup_choice_bounds(i) {
                    let mut n = Node::new(
                        1_000_000 + p.key as i32 * 512 + i as i32,
                        bounds,
                        self.labels.symbol(label),
                        "popup item",
                        Target::Popup(i),
                    );
                    n.selected = p.selected == i;
                    n.actions.push(ACCESS_DISMISS);
                    nodes.push(n);
                }
            }
        }
        nodes
    }
    pub fn activate(
        &mut self,
        scene: &mut Scene,
        shift: &str,
        id: i32,
        action: i32,
    ) -> Vec<Action> {
        let Some(n) = self.snapshot(scene, shift).into_iter().find(|n| n.id == id) else {
            return vec![];
        };
        if !n.enabled || !n.actions.contains(&action) {
            return vec![];
        }
        let mut out = Vec::new();
        match n.target {
            Target::Key(i) => {
                out.extend(self.dismiss(scene));
                if action == ACCESS_LONG_CLICK && scene.keys[i].code == codes::SHIFT {
                    out.push(Action(CAPS_LOCK, 0, -1));
                } else {
                    out.push(Action(KEY_DOWN, i as i32, -1));
                    if action == ACCESS_LONG_CLICK {
                        self.popup_source = Some(i);
                        scene.pressed = vec![i];
                        out.push(Action(OPEN_POPUP, i as i32, -1));
                    } else {
                        out.push(Action(KEY_UP, i as i32, -1));
                    }
                }
            }
            Target::Bar(i) => {
                out.extend(self.dismiss(scene));
                if i == BAR_TOGGLE_INDEX as usize {
                    out.push(Action(BAR_TOGGLE, i as i32, -1));
                } else if action == ACCESS_LONG_CLICK {
                    out.push(Action(ACCESS_READ_TOP, i as i32, -1));
                } else {
                    out.extend([Action(BAR_DOWN, i as i32, -1), Action(BAR_UP, i as i32, -1)]);
                }
            }
            Target::Candidate(i) => {
                out.extend(self.dismiss(scene));
                let token = scene.candidate_tokens.get(i).copied().unwrap_or(i as i32);
                out.push(Action(
                    if action == ACCESS_LONG_CLICK {
                        CANDIDATE_HOLD
                    } else {
                        CANDIDATE_UP
                    },
                    if action == ACCESS_LONG_CLICK {
                        token
                    } else {
                        i as i32
                    },
                    -1,
                ));
            }
            Target::Popup(i) => {
                if action == ACCESS_DISMISS {
                    out.extend(self.dismiss(scene));
                } else {
                    out.push(Action(POPUP_SELECT, scene.popup_choice_index(i) as i32, -1));
                    self.popup_source = None;
                    scene.popup = None;
                    scene.popup_geometry = None;
                    scene.pressed.clear();
                }
            }
        }
        out
    }
    pub fn dismiss(&mut self, scene: &mut Scene) -> Vec<Action> {
        let actions = self
            .popup_source
            .take()
            .map(|i| Action(KEY_CANCEL, i as i32, -1))
            .into_iter()
            .collect();
        scene.popup = None;
        scene.popup_geometry = None;
        scene.pressed.clear();
        actions
    }
}
