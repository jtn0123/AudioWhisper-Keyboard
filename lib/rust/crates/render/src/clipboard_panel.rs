//! Clipboard presentation state; Room ids remain opaque and content is never diagnostic data.
use crate::{panels::Panel, protocol::*, touch::Action};
#[derive(Clone, Debug, PartialEq)]
pub struct ClipEntry {
    pub id: i64,
    pub semantic: i32,
    pub timestamp: i64,
    pub kind: i32,
    pub pinned: bool,
    pub label: String,
    pub description: String,
    pub date: String,
    pub aspect: f32,
    pub height: f32,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipboardPanel {
    pub entries: Vec<ClipEntry>,
    pub next_semantic: i32,
    pub labels: Vec<String>,
    pub icons: Vec<String>,
    pub enabled: bool,
    pub locked: bool,
    pub columns: usize,
    pub now: i64,
    pub filters: u32,
    pub filter_row: bool,
    pub popup: Option<usize>,
    pub clear: bool,
    pub message_heights: Vec<f32>,
    pub header_height: f32,
    pub content_padding: [f32; 4],
    pub grid_padding: [f32; 4],
    pub group_height: f32,
    pub filter_height: f32,
    pub enable_size: [f32; 2],
}
impl ClipboardPanel {
    pub fn label(&self, n: usize) -> String {
        self.labels.get(n).cloned().unwrap_or_default()
    }
    pub fn visible(&self, e: &ClipEntry) -> bool {
        self.filters == 0 || self.filters & (1 << e.kind) != 0
    }
    pub fn group(&self, e: &ClipEntry) -> usize {
        if e.pinned {
            0
        } else if self.now.wrapping_sub(e.timestamp) < 300_000 {
            1
        } else {
            2
        }
    }
    pub fn modal(&self) -> bool {
        self.popup.is_some() || self.clear
    }
    pub fn update(&mut self, mut next: Self) {
        let unchanged = self.entries.len() == next.entries.len()
            && self.entries.iter().zip(&next.entries).all(|(a, b)| {
                a.id == b.id
                    && a.timestamp == b.timestamp
                    && a.pinned == b.pinned
                    && a.label == b.label
            });
        if !unchanged {
            self.popup = None;
        } else {
            for (a, b) in self.entries.iter().zip(&mut next.entries) {
                b.height = a.height;
                b.aspect = a.aspect;
            }
        }
        for entry in &mut next.entries {
            entry.semantic = self.entries.iter().find(|e| e.id == entry.id).map_or_else(
                || {
                    self.next_semantic += 1;
                    700_000 + self.next_semantic
                },
                |e| e.semantic,
            );
        }
        self.entries = next.entries;
        self.labels = next.labels;
        self.icons = next.icons;
        self.enabled = next.enabled;
        self.locked = next.locked;
        self.columns = next.columns;
        self.now = next.now;
        if !self.enabled || self.locked {
            self.popup = None;
            self.clear = false;
        }
    }
}
impl Panel {
    pub fn clipboard_activate(&mut self, op: i32, index: usize) -> Vec<Action> {
        let Some(c) = &mut self.clipboard else {
            return vec![];
        };
        if op == 0 {
            return vec![Action(PANEL_CLIPBOARD, 0, index as i32)];
        }
        if c.locked {
            return vec![];
        }
        match op {
            1 => return vec![Action(PANEL_CLIPBOARD, 1, 0)],
            2 if c.enabled && !c.modal() && !c.entries.is_empty() => c.clear = true,
            3 if c.enabled && !c.modal() && !c.entries.is_empty() => {
                c.filter_row = !c.filter_row;
                if !c.filter_row {
                    c.filters = 0;
                }
                self.scroll = 0.0;
            }
            4 if c.enabled && !c.modal() => {
                c.filters ^= 1 << index;
                self.scroll = 0.0;
            }
            5 if c.enabled && index < c.entries.len() => {
                c.popup = None;
                return vec![Action(PANEL_CLIPBOARD, 5, index as i32)];
            }
            6 if c.enabled && index < c.entries.len() => c.popup = Some(index),
            7..=9 if c.popup.is_some() => {
                let i = c.popup.take().unwrap();
                return vec![Action(PANEL_CLIPBOARD, op, i as i32)];
            }
            10 => {
                c.popup = None;
                c.clear = false;
            }
            11 if c.clear => {
                c.clear = false;
                return c
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| !e.pinned && c.visible(e))
                    .map(|(i, _)| Action(PANEL_CLIPBOARD, 8, i as i32))
                    .collect();
            }
            _ => {}
        }
        vec![Action(PANEL_CHANGED, 0, 0)]
    }
}
/// Compose StaggeredGridCells.Adaptive(160.dp), Fixed(n), no inter-cell spacing.
pub fn columns(width: f32, density: f32, fixed: usize) -> usize {
    if fixed == 0 {
        (width / (160.0 * density)).floor().max(1.0) as usize
    } else {
        fixed.max(1)
    }
}
/// Ties choose the first lane; full-span section headers join the lanes.
pub fn lane(heights: &[f32]) -> usize {
    heights
        .iter()
        .enumerate()
        .min_by(|(ia, a), (ib, b)| a.total_cmp(b).then(ia.cmp(ib)))
        .map_or(0, |(i, _)| i)
}
