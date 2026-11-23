//! Measured clipboard list/grid, filter row and modal surfaces matching ClipboardInputLayout.
use crate::{
    clipboard_panel::*,
    panel_layout::intersection,
    panels::*,
    smartbar::inset,
    text::Text,
    theme::{Request, Theme},
};
impl Panel {
    pub fn measure_clipboard(
        &mut self,
        theme: &mut Theme,
        text: &mut Text,
        r: &Request,
        width: f32,
    ) {
        let Some(c) = &mut self.clipboard else {
            return;
        };
        c.content_padding = theme.style(&r.element("clipboard-content", 0)).padding;
        c.grid_padding = theme.style(&r.element("clipboard-grid", 0)).padding;
        let width = width - c.content_padding[0] - c.content_padding[2];
        let n = columns(width, self.density, c.columns);
        let cell = width / n as f32;
        for e in &mut c.entries {
            let mut q = r.element("clipboard-item", 0);
            q.key_type = match e.kind {
                2 => "image",
                3 => "video",
                _ => "text",
            }
            .into();
            let s = theme.style(&q);
            let inner = (cell - s.margin[0] - s.margin[2] - s.padding[0] - s.padding[2]).max(1.0);
            let mut l = text.layout_weight(&e.label, s.size, s.font, s.weight);
            l.break_all_lines(Some(inner));
            let body = if e.kind == 1 {
                l.height().min(s.size * 12.0)
            } else {
                inner / e.aspect.max(0.01)
            };
            let desc = if e.description.is_empty() {
                0.0
            } else {
                text.height(&e.description, 12.0 * self.density, false, 400)
            };
            e.height = body + desc + s.margin[1] + s.margin[3] + s.padding[1] + s.padding[3];
            e.full_height = e.height
                + if e.kind == 1 {
                    (l.height() - body).max(0.0)
                } else {
                    0.0
                };
            let date = theme.style(&r.element("clipboard-item-timestamp", 0));
            let mut l = text.layout_weight(&e.date, date.size, date.font, date.weight);
            l.break_all_lines(Some(width / 2.0 - date.padding[0] - date.padding[2]));
            e.date_height = l.height() + date.padding[1] + date.padding[3];
        }
        let s = theme.style(&r.element("clipboard-subheader", 0));
        c.group_height = text.height("RECENT", s.size, s.font, s.weight)
            + s.margin[1]
            + s.margin[3]
            + s.padding[1]
            + s.padding[3];
        let action = theme.style(&r.element("clipboard-item-action", 0));
        c.action_height = text.height(&c.label(13), action.size, action.font, action.weight)
            + action.padding[1]
            + action.padding[3];
        c.action_margin = theme.style(&r.element("clipboard-item-actions", 0)).margin;
        c.filter_height = 36.0 * self.density;
        let button = theme.style(&r.element("clipboard-history-disabled-button", 0));
        c.enable_size = [
            text.measure(&c.label(7), button.size, button.font, button.weight)
                + 32.0 * self.density,
            text.height(&c.label(7), button.size, button.font, button.weight) + 16.0 * self.density,
        ];
        c.message_heights = (0..c.labels.len())
            .map(|i| {
                let s = theme.style(&r.element(
                    match i {
                        5 => "clipboard-history-disabled-title",
                        6 => "clipboard-history-disabled-message",
                        8 => "clipboard-history-locked-title",
                        9 => "clipboard-history-locked-message",
                        _ => "window",
                    },
                    0,
                ));
                let mut l = text.layout_weight(&c.label(i), s.size, s.font, s.weight);
                l.break_all_lines(Some((width - s.padding[0] - s.padding[2]).max(1.0)));
                l.height() + s.padding[1] + s.padding[3]
            })
            .collect();
        self.measure_clear(theme, text, r, width);
    }
    pub fn clipboard_layout(&mut self, width: f32, height: f32) {
        let Some(c) = &self.clipboard else {
            return;
        };
        let d = self.density;
        let header = self.row;
        let full = rect(0.0, 0.0, width, height);
        self.items.push(Item::new(
            -60,
            full,
            String::new(),
            "clipboard-content",
            Command::None,
        ));
        self.items.push(Item::new(
            -61,
            rect(0.0, 0.0, width, header),
            String::new(),
            "clipboard-header",
            Command::None,
        ));
        let mut title = Item::new(
            -62,
            rect(header, 0.0, (width - 5.0 * header).max(0.0), header),
            c.label(0),
            "clipboard-header-text",
            Command::None,
        );
        title.glyph = title.label.clone();
        self.items.push(title);
        let modal = c.modal();
        for (slot, op, label, icon) in [
            (0, 0, "Back", 0),
            (
                1,
                1,
                if c.enabled {
                    "Disable clipboard history"
                } else {
                    "Enable clipboard history"
                },
                if c.enabled { 1 } else { 2 },
            ),
            (2, 2, "Clear history", 3),
            (3, 3, "Filter history", if c.filter_row { 5 } else { 4 }),
            (4, 12, "Delete", 6),
        ] {
            let x = if slot == 0 {
                0.0
            } else {
                width - (5 - slot) as f32 * header
            };
            let command = if op == 12 {
                Command::Action(
                    self.entries
                        .iter()
                        .find(|e| e.code == keys::codes::DELETE)
                        .map_or(-1, |e| e.id),
                )
            } else {
                Command::Clipboard(op, 0)
            };
            let mut i = Item::new(
                -63 - slot,
                rect(x, 0.0, header, header),
                label.into(),
                "clipboard-header-button",
                command,
            );
            i.icon = c.icons.get(icon).cloned().unwrap_or_default();
            i.glyph = if slot == 0 { "‹" } else { "" }.into();
            let available = if slot == 2 {
                c.entries.iter().any(|e| c.visible(e))
            } else {
                !c.entries.is_empty()
            };
            i.enabled = slot == 0
                || slot == 4
                || (!c.locked && !modal && (slot == 1 || (c.enabled && available)));
            i.clip = if slot > 0 && c.enabled && !c.locked && !c.entries.is_empty() {
                rect(0.0, 0.0, width, header)
            } else {
                full
            };
            if slot == 4 {
                i.code = keys::codes::DELETE;
            }
            self.items.push(i);
        }
        let area = inset(rect(0.0, header, width, height - header), c.content_padding);
        if c.locked || !c.enabled || c.entries.is_empty() {
            let (first, second) = if c.locked {
                (8, 9)
            } else if !c.enabled {
                (5, 6)
            } else {
                (3, 4)
            };
            let mut y = area.top;
            for (j, n) in [first, second].into_iter().enumerate() {
                let h = c.message_heights.get(n).copied().unwrap_or(24.0 * d);
                let element = if c.locked {
                    if j == 0 {
                        "clipboard-history-locked-title"
                    } else {
                        "clipboard-history-locked-message"
                    }
                } else if !c.enabled {
                    if j == 0 {
                        "clipboard-history-disabled-title"
                    } else {
                        "clipboard-history-disabled-message"
                    }
                } else {
                    "clipboard-content"
                };
                let mut i = Item::new(
                    -80 - j as i32,
                    rect(area.left, y, area.width(), h),
                    c.label(n),
                    element,
                    Command::None,
                );
                i.multiline = true;
                self.items.push(i);
                y += h + if j == 0 && !c.enabled { 8.0 * d } else { 0.0 };
            }
            if !c.locked && !c.enabled {
                self.items.push(Item::new(
                    -82,
                    rect(
                        area.right - c.enable_size[0],
                        y,
                        c.enable_size[0],
                        c.enable_size[1],
                    ),
                    c.label(7),
                    "clipboard-history-disabled-button",
                    Command::Clipboard(1, 0),
                ));
            }
            self.max_scroll = 0.0;
            return;
        }
        let mut top = area.top;
        if c.filter_row {
            for (n, label) in ["Text", "Images", "Videos"].iter().enumerate() {
                let mut i = Item::new(
                    -90 - n as i32,
                    rect(
                        area.left + n as f32 * 90.0 * d,
                        top,
                        90.0 * d,
                        c.filter_height,
                    ),
                    (*label).into(),
                    "clipboard-filter-chip",
                    Command::Clipboard(4, n + 1),
                );
                i.selected = c.filters & (1 << (n + 1)) != 0;
                self.items.push(i);
            }
            top += c.filter_height;
        }
        let grid = inset(
            rect(area.left, top, area.width(), area.bottom - top),
            c.grid_padding,
        );
        let n = columns(grid.width(), d, c.columns);
        let cell = grid.width() / n as f32;
        let mut lanes = vec![grid.top - self.scroll; n];
        for group in 0..3 {
            let entries: Vec<_> = c
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| c.visible(e) && c.group(e) == group)
                .collect();
            if entries.is_empty() {
                continue;
            }
            let y = lanes.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let mut i = Item::new(
                -100 - group as i32,
                rect(grid.left, y, grid.width(), c.group_height),
                c.label(10 + group).to_uppercase(),
                "clipboard-subheader",
                Command::None,
            );
            i.clip = grid;
            self.items.push(i);
            lanes.fill(y + c.group_height);
            for (index, e) in entries {
                let l = lane(&lanes);
                let mut i = Item::new(
                    e.semantic,
                    rect(grid.left + l as f32 * cell, lanes[l], cell, e.height),
                    e.label.clone(),
                    "clipboard-item",
                    Command::Clipboard(5, index),
                );
                i.clip = grid;
                i.multiline = true;
                i.secondary = e.description.clone();
                i.media = (e.kind != 1).then_some(e.id);
                i.key_type = match e.kind {
                    2 => "image",
                    3 => "video",
                    _ => "text",
                }
                .into();
                i.enabled = c.popup.is_none();
                self.items.push(i);
                lanes[l] += e.height;
            }
        }
        self.max_scroll =
            (lanes.iter().copied().fold(grid.top, f32::max) + self.scroll - grid.bottom).max(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);
        if self.max_scroll > 0.0 {
            self.items.push(Item::new(
                -15,
                grid,
                "Clipboard history".into(),
                "panel-scroll",
                Command::Scroll,
            ));
        }
        if modal {
            self.clipboard_modal(grid);
        }
        for i in &mut self.items {
            i.clip = intersection(i.clip, full);
        }
    }
    fn clipboard_modal(&mut self, grid: keys::Rect) {
        let c = self.clipboard.as_mut().unwrap();
        self.items.push(Item::new(
            -110,
            grid,
            "Dismiss".into(),
            "clipboard-scrim",
            Command::Clipboard(10, 0),
        ));
        if let Some(index) = c.popup {
            let Some(e) = c.entries.get(index) else {
                return;
            };
            let h = e.full_height.min(grid.height() - e.date_height);
            c.popup_max_scroll = (e.full_height - h).max(0.0);
            c.popup_scroll = c.popup_scroll.clamp(0.0, c.popup_max_scroll);
            let y = grid.top + (grid.height() - h - e.date_height) / 2.0;
            let mut i = Item::new(
                -111,
                rect(grid.left, y, grid.width() / 2.0, h),
                e.label.clone(),
                "clipboard-item-popup",
                Command::None,
            );
            i.key_type = match e.kind {
                2 => "image",
                3 => "video",
                _ => "text",
            }
            .into();
            i.multiline = true;
            i.secondary = e.description.clone();
            i.text_offset = c.popup_scroll;
            i.media = (e.kind != 1).then_some(e.id);
            self.items.push(i);
            let mut date = Item::new(
                -112,
                rect(grid.left, y + h, grid.width() / 2.0, e.date_height),
                e.date.clone(),
                "clipboard-item-timestamp",
                Command::None,
            );
            date.multiline = true;
            self.items.push(date);
            let total = c.action_height * 3.0 + c.action_margin[1] + c.action_margin[3];
            let y = grid.top + (grid.height() - total) / 2.0;
            let x = grid.left + grid.width() / 2.0;
            let half = grid.width() / 2.0;
            self.items.push(Item::new(
                -113,
                rect(x, y, half, total),
                String::new(),
                "clipboard-item-actions",
                Command::None,
            ));
            for (n, (op, label)) in [(7, if e.pinned { 14 } else { 13 }), (8, 15), (9, 16)]
                .iter()
                .enumerate()
            {
                let mut i = Item::new(
                    -114 - n as i32,
                    rect(
                        x + c.action_margin[0],
                        y + c.action_margin[1] + n as f32 * c.action_height,
                        half - c.action_margin[0] - c.action_margin[2],
                        c.action_height,
                    ),
                    c.label(*label),
                    "clipboard-item-action",
                    Command::Clipboard(*op, index),
                );
                i.multiline = true;
                i.clip = grid;
                i.icon = c.icons.get(7 + n).cloned().unwrap_or_default();
                self.items.push(i);
            }
        } else if c.clear {
            self.clear_dialog(grid);
        }
    }
}
