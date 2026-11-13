//! Compose action editor's marker algorithm; ids are value identities, never key codes.
pub const EMPTY: i32 = -1;
pub const MARKER: i32 = -2;
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Arrangement {
    pub sticky: i32,
    pub dynamic: Vec<i32>,
    pub hidden: Vec<i32>,
}
impl Arrangement {
    pub fn distinct(&mut self) {
        let mut seen = vec![self.sticky];
        self.dynamic.retain(|id| {
            if seen.contains(id) {
                false
            } else {
                seen.push(*id);
                true
            }
        });
        self.hidden.retain(|id| {
            if seen.contains(id) {
                false
            } else {
                seen.push(*id);
                true
            }
        });
    }
    pub fn overflow(&self, visible: usize) -> &[i32] {
        if self.dynamic.is_empty() {
            return &[];
        }
        let count = self
            .dynamic
            .len()
            .saturating_sub(visible)
            .min(self.dynamic.len() - 1);
        &self.dynamic[self.dynamic.len() - count..]
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Editor {
    pub arrangement: Arrangement,
    pub dragging: Option<i32>,
}
impl Editor {
    pub fn new(mut arrangement: Arrangement) -> Self {
        for list in [&mut arrangement.dynamic, &mut arrangement.hidden] {
            if list.is_empty() {
                list.push(EMPTY);
            }
        }
        Self {
            arrangement,
            dragging: None,
        }
    }
    fn item(&mut self, group: i32, index: usize) -> Option<&mut i32> {
        match group {
            0 if index == 0 => Some(&mut self.arrangement.sticky),
            1 => self.arrangement.dynamic.get_mut(index),
            2 => self.arrangement.hidden.get_mut(index),
            _ => None,
        }
    }
    pub fn begin(&mut self, group: i32, index: usize) {
        if self.dragging.is_some() {
            return;
        }
        if let Some(item) = self.item(group, index)
            && *item >= 0
        {
            let id = *item;
            *item = MARKER;
            self.dragging = Some(id);
        }
    }
    fn remove_markers(&mut self) {
        if self.arrangement.sticky == MARKER {
            self.arrangement.sticky = EMPTY;
        }
        for list in [&mut self.arrangement.dynamic, &mut self.arrangement.hidden] {
            if let Some(i) = list.iter().position(|&v| v == MARKER) {
                list.remove(i);
            }
            if list.is_empty() {
                list.push(EMPTY);
            }
        }
    }
    pub fn over(&mut self, group: i32, index: usize) {
        if self.dragging.is_none() {
            return;
        }
        let Some(&mut target) = self.item(group, index) else {
            return;
        };
        if target == MARKER {
            return;
        }
        if group == 0 {
            if target != EMPTY {
                self.arrangement.dynamic.insert(0, target);
            }
            self.remove_markers();
            self.arrangement.sticky = MARKER;
        } else {
            self.remove_markers();
            let list = if group == 1 {
                &mut self.arrangement.dynamic
            } else {
                &mut self.arrangement.hidden
            };
            if target == EMPTY {
                list[index] = MARKER;
            } else {
                list.insert(index, MARKER);
            }
        }
    }
    /// Compose completes (rather than rolls back) cancelled drags and saves on disposal.
    pub fn complete(&mut self) {
        if let Some(id) = self.dragging.take() {
            if self.arrangement.sticky == MARKER {
                self.arrangement.sticky = id;
            } else if let Some(item) = self
                .arrangement
                .dynamic
                .iter_mut()
                .chain(&mut self.arrangement.hidden)
                .find(|v| **v == MARKER)
            {
                *item = id;
            }
        }
    }
    pub fn saved(&mut self) -> Arrangement {
        self.complete();
        let mut a = self.arrangement.clone();
        if a.sticky < 0 {
            a.sticky = EMPTY;
        }
        a.dynamic.retain(|&v| v >= 0);
        a.hidden.retain(|&v| v >= 0);
        a
    }
}
