//! Emoji palette policy. Supported glyph data and stored histories arrive from Android.
use crate::panels::{Command, Item, Panel, rect};
#[derive(Clone, Debug, PartialEq)]
pub struct EmojiValue {
    pub value: String,
    pub tone: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct EmojiGroup {
    pub category: i32,
    pub start: usize,
    pub count: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct EmojiPanel {
    pub values: Vec<EmojiValue>,
    pub groups: Vec<EmojiGroup>,
    pub pinned: Vec<usize>,
    pub recent: Vec<usize>,
    pub shown_pinned: Vec<usize>,
    pub shown_recent: Vec<usize>,
    pub history: bool,
    pub locked: bool,
    pub tone: i32,
    pub pinned_strategy: i32,
    pub recent_strategy: i32,
    pub category: i32,
    pub popup: Option<(usize, bool, bool)>,
    pub labels: Vec<String>,
    pub icons: Vec<String>,
    pub bottom: f32,
    pub gifs: bool,
}
impl EmojiPanel {
    pub fn refresh(&mut self) {
        self.shown_pinned = self.pinned.clone();
        self.shown_recent = self.recent.clone();
    }
    pub fn base(&self, g: &EmojiGroup) -> usize {
        g.start
            + crate::panel_geometry::base(
                &self.values[g.start..g.start + g.count]
                    .iter()
                    .map(|v| v.tone)
                    .collect::<Vec<_>>(),
                self.tone,
            )
    }
    pub fn variants(&self, index: usize) -> Vec<usize> {
        self.groups
            .iter()
            .find(|g| index >= g.start && index < g.start + g.count)
            .map(|g| {
                crate::panel_geometry::variants(
                    &self.values[g.start..g.start + g.count]
                        .iter()
                        .map(|v| v.tone)
                        .collect::<Vec<_>>(),
                    self.tone,
                )
                .into_iter()
                .map(|i| i + g.start)
                .collect()
            })
            .unwrap_or_default()
    }

    pub fn label(&self, i: usize) -> String {
        self.labels.get(i).cloned().unwrap_or_default()
    }
}
impl Panel {
    pub(crate) fn emoji_layout(&mut self, width: f32, height: f32) {
        let Some(e) = self.emoji.take() else {
            return;
        };
        let all = rect(0.0, 0.0, width, height);
        self.items
            .push(Item::new(-30, all, String::new(), "media", Command::None));
        let count = if e.history { 10 } else { 9 };
        for n in 0..count {
            let category = n + if e.history { 0 } else { 1 };
            let mut i = Item::new(
                300_000 + category,
                rect(
                    n as f32 * width / count as f32,
                    0.0,
                    width / count as f32,
                    self.row,
                ),
                e.label(category as usize),
                "media-emoji-tab",
                Command::Category(category),
            );
            i.icon = e.icons.get(category as usize).cloned().unwrap_or_default();
            i.selected = category == e.category;
            self.items.push(i);
        }
        let bottom = rect(0.0, height - e.bottom, width, e.bottom);
        self.items.push(Item::new(
            -31,
            bottom,
            String::new(),
            "media-bottom-row",
            Command::None,
        ));
        let button = 48.0 * self.density;
        for (id, code, x, label) in [
            (-32, keys::codes::IME_UI_MODE_TEXT, 0.0, "ABC"),
            (
                -33,
                keys::codes::DELETE,
                width - self.bottom_sizes[1].max(button),
                "",
            ),
        ] {
            if let Some(entry) = self.entries.iter().find(|a| a.code == code) {
                let mut i = Item::new(
                    id,
                    rect(
                        x,
                        bottom.top,
                        self.bottom_sizes[usize::from(code == keys::codes::DELETE)].max(button),
                        e.bottom,
                    ),
                    entry.label.clone(),
                    "media-bottom-row-button",
                    Command::Action(entry.id),
                );
                i.label = if code == keys::codes::IME_UI_MODE_TEXT {
                    "ABC"
                } else {
                    "Backspace"
                }
                .into();
                i.code = code;
                i.glyph = label.into();
                i.icon = entry.icon.clone();
                self.items.push(i);
            }
        }
        if e.gifs {
            let mut i = Item::new(
                -34,
                rect(button * 2.0, bottom.top, button, e.bottom),
                "GIF".into(),
                "media-bottom-row-button",
                Command::Mode(4),
            );
            i.glyph = "GIF".into();
            self.items.push(i);
            let mut i = Item::new(
                -35,
                rect(button, bottom.top, button, e.bottom),
                "Emoji".into(),
                "media-bottom-row-button",
                Command::Category(e.category),
            );
            i.icon = e.icons.get(1).cloned().unwrap_or_default();
            self.items.push(i);
        }
        let area = rect(0.0, self.row, width, height - self.row - e.bottom);
        let cells = crate::panel_geometry::cells(width, 42.0 * self.density);
        let columns = cells.len();
        let cell = cells[0].1;
        let mut y = self.row - self.scroll;
        if e.category == 0 {
            if e.locked || (e.shown_pinned.is_empty() && e.shown_recent.is_empty()) {
                let padding = 8.0 * self.density;
                self.items.push(Item::new(
                    -36,
                    rect(
                        padding,
                        area.top + padding,
                        width - 2.0 * padding,
                        self.message_heights[0].max(self.row),
                    ),
                    e.label(if e.locked { 10 } else { 11 }),
                    "media",
                    Command::None,
                ));
                if !e.locked {
                    self.items.push(Item::new(
                        -42,
                        rect(
                            padding,
                            area.top + padding * 2.0 + self.message_heights[0],
                            width - 2.0 * padding,
                            self.message_heights[1].max(self.row),
                        ),
                        e.label(12),
                        "media",
                        Command::None,
                    ));
                }
            } else {
                for (pinned, ids) in [(true, &e.shown_pinned), (false, &e.shown_recent)] {
                    if ids.is_empty() {
                        continue;
                    }
                    self.items.push(Item::new(
                        if pinned { -37 } else { -38 },
                        rect(0.0, y, width, self.emoji_header.max(self.row * 0.5)),
                        e.label(if pinned { 13 } else { 14 }),
                        "media-emoji-subheader",
                        Command::None,
                    ));
                    y += self.emoji_header.max(self.row * 0.5);
                    for (n, &index) in ids.iter().enumerate() {
                        let mut i = emoji_item(
                            &e,
                            index,
                            rect(
                                cells[n % columns].0,
                                y + (n / columns) as f32 * cell,
                                cell,
                                cell,
                            ),
                            Command::Emoji(index),
                        );
                        i.selected = pinned;
                        i.clip = area;
                        self.items.push(i);
                    }
                    y += ids.len().div_ceil(columns) as f32 * cell;
                }
            }
        } else {
            let ids: Vec<_> = e
                .groups
                .iter()
                .filter(|g| g.category == e.category)
                .map(|g| e.base(g))
                .collect();
            for (n, &index) in ids.iter().enumerate() {
                let mut i = emoji_item(
                    &e,
                    index,
                    rect(
                        cells[n % columns].0,
                        y + (n / columns) as f32 * cell,
                        cells[n % columns].1,
                        cells[n % columns].1,
                    ),
                    Command::Emoji(index),
                );
                i.clip = area;
                self.items.push(i);
            }
            y += ids.len().div_ceil(columns) as f32 * cell;
        }
        self.max_scroll = (y + self.scroll - area.bottom).max(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);
        if self.max_scroll > 0.0 {
            self.items.insert(
                1,
                Item::new(-15, area, "Emoji".into(), "panel-scroll", Command::Scroll),
            );
        }
        if let Some((index, history, pinned)) = e.popup {
            let commands: Vec<_> = if history {
                let mut v = vec![Command::History(if pinned { 2 } else { 1 }, index)];
                // Keep Compose's operator precedence, including the recent manual strategy.
                if crate::panel_geometry::history_moves(
                    pinned,
                    e.pinned_strategy,
                    e.recent_strategy,
                ) {
                    v.extend([Command::History(7, index), Command::History(8, index)]);
                }
                v.push(Command::History(4, index));
                v
            } else {
                e.variants(index).into_iter().map(Command::Emoji).collect()
            };
            let source = self
                .items
                .iter()
                .find(|i| i.command == Command::Emoji(index))
                .map(|i| i.bounds)
                .unwrap_or(area);
            let w = 42.0 * self.density;
            let popup_width = (commands.len() as f32 * w).min(width);
            let x = (source.left + source.width() / 2.0 - popup_width / 2.0)
                .clamp(0.0, width - popup_width);
            let top = (source.top - self.row).max(0.0);
            let popup = rect(x, top, popup_width, self.row);
            self.items.push(Item::new(
                -39,
                all,
                String::new(),
                "media",
                Command::Dismiss,
            ));
            self.items.push(Item::new(
                -40,
                popup,
                String::new(),
                "media-emoji-key-popup-box",
                Command::None,
            ));
            for (n, command) in commands.iter().enumerate() {
                let r = rect(x + n as f32 * w, top, w, self.row);
                let mut i = match command {
                    Command::Emoji(index) => emoji_item(&e, *index, r, command.clone()),
                    Command::History(op, _) => {
                        let (label, glyph) = match op {
                            1 => ("Pin", "⌖"),
                            2 => ("Unpin", "⌖"),
                            7 => ("Move left", "‹"),
                            8 => ("Move right", "›"),
                            _ => ("Delete", "⌫"),
                        };
                        let mut i = Item::new(
                            400_000 + n as i32,
                            r,
                            label.into(),
                            "media-emoji-key-popup-element",
                            command.clone(),
                        );
                        i.glyph = glyph.into();
                        i.icon = e
                            .icons
                            .get(match op {
                                1 | 2 => 10,
                                7 => 11,
                                8 => 12,
                                _ => 13,
                            })
                            .cloned()
                            .unwrap_or_default();
                        i
                    }
                    _ => continue,
                };
                i.id = 400_000 + n as i32;
                i.element = "media-emoji-key-popup-element".into();
                self.items.push(i);
            }
        }
        self.emoji = Some(e);
    }
}
fn emoji_item(e: &EmojiPanel, index: usize, bounds: keys::Rect, command: Command) -> Item {
    let value = e
        .values
        .get(index)
        .map(|v| v.value.clone())
        .unwrap_or_default();
    let mut i = Item::new(
        310_000 + index as i32,
        bounds,
        value.clone(),
        "media-emoji-key",
        command,
    );
    i.glyph = value;
    i
}
