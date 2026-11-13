//! GIF grid, categories and transient send selection; Android supplies results and decoded frames.
use crate::{
    panels::{Command, Item, Panel, rect},
    protocol::*,
    touch::Action,
};
#[derive(Clone, Debug, PartialEq)]
pub struct GifEntry {
    pub id: i64,
    pub label: String,
    pub aspect: f32,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GifPanel {
    pub entries: Vec<GifEntry>,
    pub chips: Vec<String>,
    pub shown: String,
    pub labels: Vec<String>,
    pub phase: f32,
    pub status: i32,
    pub sending: Option<usize>,
    pub bottom: f32,
    pub chip_scroll: f32,
    pub chip_widths: Vec<f32>,
}
impl GifPanel {
    pub fn label(&self, i: usize) -> String {
        self.labels.get(i).cloned().unwrap_or_default()
    }
}
pub fn aspect(value: f32) -> f32 {
    value.clamp(0.5, 2.5)
}
impl Panel {
    pub fn gif_layout(&mut self, width: f32, height: f32) {
        let Some(g) = &self.gif else {
            return;
        };
        let d = self.density;
        let bottom = height - g.bottom;
        let header = self.row;
        let chips = 36.0 * d;
        self.items.push(Item::new(
            -200,
            rect(0.0, 0.0, width, height),
            String::new(),
            "media",
            Command::None,
        ));
        let mut search = Item::new(
            -201,
            rect(8.0 * d, 6.0 * d, width - 110.0 * d, header - 12.0 * d),
            if g.shown == "Trending" {
                g.label(0)
            } else {
                g.shown.clone()
            },
            "gif-search-field",
            Command::Gif(0, 0),
        );
        search.multiline = true;
        self.items.push(search);
        self.items.push(Item::new(
            -202,
            rect(width - 96.0 * d, 6.0 * d, 88.0 * d, header - 12.0 * d),
            g.label(1),
            "gif-attribution",
            Command::None,
        ));
        if g.shown != "Trending" {
            self.items.push(Item::new(
                -203,
                rect(width - 146.0 * d, 6.0 * d, 36.0 * d, header - 12.0 * d),
                g.label(4),
                "gif-close",
                Command::Gif(1, 0),
            ));
        }
        let mut x = -g.chip_scroll;
        for (index, label) in g.chips.iter().enumerate() {
            let w = g.chip_widths.get(index).copied().unwrap_or(90.0 * d);
            let mut i = Item::new(
                800_000 + index as i32,
                rect(x, header, w, chips),
                label.clone(),
                "clipboard-filter-chip",
                Command::Gif(2, index),
            );
            i.selected = *label == g.shown;
            i.clip = rect(0.0, header, width, chips);
            self.items.push(i);
            x += w;
        }
        let area = rect(
            4.0 * d,
            header + chips + 4.0 * d,
            width - 8.0 * d,
            bottom - header - chips - 4.0 * d,
        );
        if g.status == 1 && !g.entries.is_empty() {
            let rects = grid(
                area.width(),
                d,
                &g.entries.iter().map(|e| e.aspect).collect::<Vec<_>>(),
            );
            let mut extent = 0.0f32;
            for (index, (e, r)) in g.entries.iter().zip(rects).enumerate() {
                extent = extent.max(r.bottom + 4.0 * d);
                let mut i = Item::new(
                    810_000 + index as i32,
                    rect(
                        area.left + r.left,
                        area.top + r.top - self.scroll,
                        r.width(),
                        r.height(),
                    ),
                    e.label.clone(),
                    "gif-tile",
                    Command::Gif(3, index),
                );
                i.media = Some(e.id);
                i.clip = area;
                i.enabled = g.sending.is_none();
                self.items.push(i);
            }
            self.max_scroll = (extent - area.height()).max(0.0);
            self.scroll = self.scroll.clamp(0.0, self.max_scroll);
            if self.max_scroll > 0.0 {
                self.items.push(Item::new(
                    -15,
                    area,
                    "GIF results".into(),
                    "panel-scroll",
                    Command::Scroll,
                ));
            }
        } else {
            self.max_scroll = 0.0;
            let mut i = Item::new(
                -204,
                area,
                g.label(2),
                "gif-message",
                if g.status == 2 {
                    Command::Gif(4, 0)
                } else {
                    Command::None
                },
            );
            i.multiline = true;
            if g.status == 0 {
                i.label.clear();
                i.element = "gif-spinner".into();
                i.bounds = rect(
                    area.left + (area.width() - 28.0 * d) / 2.0,
                    area.top + (area.height() - 28.0 * d) / 2.0,
                    28.0 * d,
                    28.0 * d,
                );
            }
            self.items.push(i);
        }
        self.items.push(Item::new(
            -205,
            rect(0.0, bottom, width, g.bottom),
            String::new(),
            "media-bottom-row",
            Command::None,
        ));
        for (n, (label, op)) in [("ABC", 5), ("Emoji", 6), ("GIF", 7), ("Delete", 8)]
            .iter()
            .enumerate()
        {
            let w = if n == 0 {
                self.bottom_sizes[0].max(48.0 * d)
            } else {
                48.0 * d
            };
            let x = if n == 3 {
                width - w
            } else {
                n as f32 * 64.0 * d
            };
            let command = if *op == 8 {
                Command::Action(
                    self.entries
                        .iter()
                        .find(|e| e.code == keys::codes::DELETE)
                        .map_or(-1, |e| e.id),
                )
            } else {
                Command::Gif(*op, 0)
            };
            let mut i = Item::new(
                -206 - n as i32,
                rect(x, bottom, w, g.bottom),
                (*label).into(),
                "media-bottom-row-button",
                command,
            );
            if n == 3 {
                i.code = keys::codes::DELETE;
            }
            self.items.push(i);
        }
    }
    pub fn gif_activate(&mut self, op: i32, index: usize) -> Vec<Action> {
        let Some(g) = &mut self.gif else {
            return vec![];
        };
        if op == 3 {
            if index >= g.entries.len() || g.sending.is_some() {
                return vec![];
            }
            g.sending = Some(index);
        }
        if op == 2 && index >= g.chips.len() {
            return vec![];
        }
        vec![Action(PANEL_GIF, op, index as i32)]
    }
}

/// Compose's three-lane staggered grid with 4dp gutters; the caller owns outer padding.
pub fn grid(width: f32, d: f32, aspects: &[f32]) -> Vec<keys::Rect> {
    let cell = (width - 8.0 * d) / 3.0;
    let mut lanes = [0.0; 3];
    aspects
        .iter()
        .map(|&a| {
            let lane = crate::clipboard_panel::lane(&lanes);
            let h = cell / aspect(a);
            let r = rect(lane as f32 * (cell + 4.0 * d), lanes[lane], cell, h);
            lanes[lane] += h + 4.0 * d;
            r
        })
        .collect()
}
