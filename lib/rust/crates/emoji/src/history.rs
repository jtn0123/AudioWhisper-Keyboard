//! Value-identity history policy. Android owns serialization and the storage mutex.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    pub pinned: Vec<i32>,
    pub recent: Vec<i32>,
}
#[derive(Clone, Copy, Debug)]
pub struct Policy {
    pub enabled: bool,
    /// AUTO_PREPEND, AUTO_APPEND, MANUAL_PREPEND, MANUAL_APPEND.
    pub pinned: i32,
    pub recent: i32,
    pub pinned_limit: i32,
    pub recent_limit: i32,
}
fn add(list: &mut Vec<i32>, strategy: i32, value: i32) {
    let index = if strategy % 2 == 0 { 0 } else { list.len() };
    list.insert(index, value);
}
fn limit(list: &mut Vec<i32>, strategy: i32, max: i32) {
    if max == 0 || max as usize >= list.len() {
        return;
    }
    let max = max.max(0) as usize;
    if strategy % 2 == 0 {
        list.truncate(max);
    } else {
        list.drain(..list.len() - max);
    }
}
impl History {
    /// 0 used, 1 pin, 2 unpin, 3 move, 4 remove, 5 clear recent, 6 clear pinned.
    /// Pin/unpin deliberately do not trim: this is the original preference policy.
    pub fn apply(&mut self, p: Policy, op: i32, value: i32, offset: i32) {
        if !p.enabled {
            return;
        }
        let pinned = self.pinned.iter().position(|&v| v == value);
        let recent = self.recent.iter().position(|&v| v == value);
        match op {
            0 => {
                if let Some(i) = pinned {
                    if p.pinned < 2 {
                        self.pinned.remove(i);
                        add(&mut self.pinned, p.pinned, value);
                    }
                } else if let Some(i) = recent {
                    if p.recent < 2 {
                        self.recent.remove(i);
                        add(&mut self.recent, p.recent, value);
                    }
                } else {
                    add(&mut self.recent, p.recent, value);
                }
                limit(&mut self.pinned, p.pinned, p.pinned_limit);
                limit(&mut self.recent, p.recent, p.recent_limit);
            }
            1 => {
                if let Some(i) = recent {
                    self.recent.remove(i);
                    add(&mut self.pinned, p.pinned, value);
                }
            }
            2 => {
                if let Some(i) = pinned {
                    self.pinned.remove(i);
                    add(&mut self.recent, p.recent, value);
                }
            }
            3 | 4 => {
                let (list, index) = if pinned.is_some() {
                    (&mut self.pinned, pinned)
                } else {
                    (&mut self.recent, recent)
                };
                if let Some(i) = index {
                    let j = (i as i32)
                        .wrapping_add(offset)
                        .clamp(0, list.len() as i32 - 1) as usize;
                    let item = list.remove(i);
                    if op == 3 {
                        list.insert(j, item);
                    }
                }
            }
            5 => self.recent.clear(),
            6 => self.pinned.clear(),
            _ => {}
        }
    }
}
