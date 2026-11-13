//! Compose adaptive tile grids, headers and clipping.
use crate::{
    panels::*,
    theme::{Request, Theme},
};
use keys::Rect;
impl Panel {
    pub fn measure(
        &mut self,
        theme: &mut Theme,
        text: &mut crate::text::Text,
        request: &Request,
        width: f32,
    ) {
        self.measure_chrome(theme, text, request);
        if self.kind == 7 {
            self.measure_subtypes(theme, text, request);
            return;
        }
        if self.kind == 4 {
            self.measure_clipboard(theme, text, request, width);
            return;
        }
        if self.kind == 5
            && let Some(g) = &mut self.gif
        {
            let style = theme.style(&request.element("clipboard-filter-chip", 0));
            let body = theme.style(&request.element("media", 0));
            let size = 16.0 * self.density * theme.font_scale();
            let mut message = text.layout_weight(&g.label(2), size, body.mono, body.weight);
            message.break_all_lines(Some((width - 56.0 * self.density).max(1.0)));
            g.message_height = message.height().max(size);
            g.chip_widths = g
                .chips
                .iter()
                .map(|label| {
                    text.measure(label, style.size, style.mono, style.weight)
                        + style.padding[0]
                        + style.padding[2]
                        + style.margin[0]
                        + style.margin[2]
                })
                .collect();
        }
        let style = theme.style(&request.element("smartbar-actions-editor-subheader", 0));
        for g in 0..3 {
            self.subheaders[g] =
                text.height(&self.title(g + 2), style.size, style.mono, style.weight)
                    + style.padding[1]
                    + style.padding[3]
                    + style.margin[1]
                    + style.margin[3];
        }
        let header = theme.style(&request.element("media-emoji-subheader", 0));
        self.emoji_header = text.height("Recent", header.size, header.mono, header.weight)
            + header.margin[1]
            + header.margin[3];
        if let Some(e) = &self.emoji {
            let body = theme.style(&request.element("media", 0));
            for (n, index) in [11, 12].iter().enumerate() {
                let mut layout =
                    text.layout_weight(&e.label(*index), body.size, body.mono, body.weight);
                layout.break_all_lines(Some((width - 16.0 * self.density).max(1.0)));
                self.message_heights[n] = layout.height();
            }
        }
        let bottom = theme.style(&request.element("media-bottom-row-button", 0));
        self.bottom_sizes = [
            text.measure("ABC", bottom.size, bottom.mono, 700)
                + bottom.padding[0]
                + bottom.padding[2],
            24.0 * self.density + bottom.padding[0] + bottom.padding[2],
        ];
        let customize =
            theme.style(&request.element("smartbar-actions-overflow-customize-button", 0));
        self.customize_size = [
            text.measure(
                &self.title(0),
                customize.size,
                customize.mono,
                customize.weight,
            ) + 48.0 * self.density
                + customize.margin[0]
                + customize.margin[2],
            text.height(
                &self.title(0),
                customize.size,
                customize.mono,
                customize.weight,
            ) + 16.0 * self.density
                + customize.margin[1]
                + customize.margin[3],
        ];
        self.grid_margin = theme
            .style(&request.element(
                if self.kind == 2 {
                    "smartbar-actions-overflow"
                } else {
                    "smartbar-actions-editor-tile-grid"
                },
                0,
            ))
            .margin;
    }
    pub fn layout(&mut self, width: f32, height: f32, strip: f32) {
        if self.chrome.is_some() {
            self.layout_chrome(width, height);
            return;
        }
        self.items.clear();
        if self.kind == 7 {
            self.layout_subtypes(width, height);
            return;
        }
        if self.kind == 0 {
            return;
        }
        if self.kind == 5 {
            self.gif_layout(width, height);
            return;
        }
        if self.kind == 4 {
            self.clipboard_layout(width, height);
            return;
        }
        if self.kind == 1 {
            self.emoji_layout(width, height);
            return;
        }
        let editor = self.kind == 3;
        let start = if editor {
            let width = width - self.grid_margin[0] - self.grid_margin[2];
            let cells = crate::panel_geometry::cells(width, self.row * 1.8);
            let columns = cells.len();
            let cell = cells[0].1;
            let a = self
                .editor
                .as_ref()
                .map(|e| &e.arrangement)
                .unwrap_or(&self.arrangement);
            let rows = 1
                + a.dynamic.len().max(1).div_ceil(columns)
                + a.hidden.len().max(1).div_ceil(columns);
            let extent = 48.0 * self.density
                + self
                    .subheaders
                    .iter()
                    .map(|h| h.max(self.row * 0.75))
                    .sum::<f32>()
                + rows as f32 * cell;
            (height - extent).max(0.0)
        } else {
            strip
        };
        let area = rect(0.0, start, width, height - start);
        if editor {
            self.items.push(Item::new(
                -14,
                rect(0.0, 0.0, width, start),
                "Dismiss".into(),
                "panel-scrim",
                Command::Mode(2),
            ));
        }
        self.items.push(Item::new(
            -10,
            area,
            String::new(),
            if editor {
                "smartbar-actions-editor"
            } else {
                "smartbar-actions-overflow"
            },
            Command::None,
        ));
        let min = self.row * if editor { 1.8 } else { 2.2 };
        let margin = self.grid_margin;
        let mut cells = crate::panel_geometry::cells(width - margin[0] - margin[2], min);
        for cell in &mut cells {
            cell.0 += margin[0];
        }
        let columns = cells.len();
        let cell = cells[0].1;
        let mut y = start + margin[1] - self.scroll;
        if editor {
            let header = 48.0 * self.density;
            self.items.push(Item::new(
                -11,
                rect(0.0, start, width, header),
                self.title(1),
                "smartbar-actions-editor-header",
                Command::None,
            ));
            let mut back = Item::new(
                -12,
                rect(0.0, start, header, header),
                self.title(5),
                "smartbar-actions-editor-header-button",
                Command::Mode(2),
            );
            back.glyph = "‹".into();
            self.items.push(back);
            y += header;
            let a = self
                .editor
                .as_ref()
                .map(|e| &e.arrangement)
                .unwrap_or(&self.arrangement)
                .clone();
            for (group, list) in [vec![a.sticky], a.dynamic, a.hidden].iter().enumerate() {
                let count = list
                    .iter()
                    .filter(|&&v| v != crate::arrangement::EMPTY)
                    .count();
                let title = self.title(group + 2).replace("{n}", &count.to_string());
                let h = self.subheaders[group].max(self.row * 0.75);
                self.items.push(Item::new(
                    -20 - group as i32,
                    rect(margin[0], y, width - margin[0] - margin[2], h),
                    title,
                    "smartbar-actions-editor-subheader",
                    Command::None,
                ));
                y += h;
                self.tiles(list, group as i32, &cells, y, area);
                y += list.len().div_ceil(columns) as f32 * cell;
            }
            // Header stays fixed while the tile grid scrolls below it.
            for i in self
                .items
                .iter_mut()
                .filter(|i| i.element.contains("subheader") || i.element.ends_with("-tile"))
            {
                i.clip.top = (start + header).max(i.clip.top);
            }
        } else {
            let ids = self.arrangement.overflow(self.visible_actions).to_vec();
            self.tiles(&ids, 1, &cells, y, area);
            y += ids.len().div_ceil(columns) as f32 * cell;
            let mut customize = Item::new(
                -13,
                rect(
                    (width - self.customize_size[0]) / 2.0,
                    y,
                    self.customize_size[0],
                    self.customize_size[1],
                ),
                self.title(0),
                "smartbar-actions-overflow-customize-button",
                Command::Mode(3),
            );
            customize.glyph = customize.label.clone();
            self.items.push(customize);
            y += self.customize_size[1];
        }
        self.extent = y + self.scroll;
        self.max_scroll = (self.extent - height + margin[3]).max(0.0);
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);
        for i in &mut self.items {
            if i.id == -42 {
                i.text.italic = true;
            }
            if i.id == -14 {
                continue;
            }
            i.clip = intersection(
                i.clip,
                if editor {
                    area
                } else {
                    rect(
                        margin[0],
                        start + margin[1],
                        width - margin[0] - margin[2],
                        height - start - margin[1] - margin[3],
                    )
                },
            );
        }
        if self.max_scroll > 0.0 {
            self.items.insert(
                1,
                Item::new(-15, area, "Actions".into(), "panel-scroll", Command::Scroll),
            );
        }
        if let Some(e) = &self.editor
            && let Some(id) = e.dragging
            && let Some(entry) = self.entries.iter().find(|a| a.id == id)
        {
            let mut item = tile(
                entry,
                900_000,
                rect(
                    self.drag_position[0] - cell / 2.0,
                    self.drag_position[1] - cell / 2.0,
                    cell,
                    cell,
                ),
                "smartbar-actions-editor-tile",
                Command::None,
            );
            item.clip = area;
            self.items.push(item);
        }
    }
    fn tiles(&mut self, ids: &[i32], group: i32, cells: &[(f32, f32)], y: f32, area: Rect) {
        let editor = self.kind == 3;
        let columns = cells.len();
        let cell = cells[0].1;
        for (index, &id) in ids.iter().enumerate() {
            let bounds = rect(
                cells[index % columns].0,
                y + (index / columns) as f32 * cell,
                cells[index % columns].1,
                cells[index % columns].1,
            );
            let command = if editor {
                Command::Drag(group, index)
            } else {
                Command::Action(id)
            };
            let element = if editor {
                "smartbar-actions-editor-tile"
            } else {
                "smartbar-action-tile"
            };
            let mut item = if let Some(entry) = self.entries.iter().find(|a| a.id == id) {
                tile(
                    entry,
                    100_000 + group * 10_000 + index as i32,
                    bounds,
                    element,
                    command,
                )
            } else {
                Item::new(
                    100_000 + group * 10_000 + index as i32,
                    bounds,
                    String::new(),
                    element,
                    command,
                )
            };
            if editor {
                item.enabled = true;
            }
            item.clip = area;
            self.items.push(item);
        }
    }
    pub fn style(&mut self, theme: &mut Theme, request: &Request) {
        self.indicator =
            theme.style(&request.element("media-emoji-key-popup-extended-indicator", 0));
        for i in &mut self.items {
            let mut r = request.element(&i.element, 0);
            r.code = i.code;
            r.key_type = i.key_type.clone();
            r.auto_commit = i.selected;
            i.styles = [0, 1].map(|pressed| {
                let mut r = r.clone();
                r.selector = if !i.enabled {
                    4
                } else if pressed == 1 {
                    1
                } else {
                    0
                };
                theme.style(&r)
            });
            let selector = if !i.enabled {
                4
            } else if self.pressed == Some(i.id) {
                1
            } else {
                0
            };
            i.text = theme.style(&r.element(&format!("{}-text", i.element), selector));
            i.icon_style = theme.style(&r.element(&format!("{}-icon", i.element), selector));
            if i.id == -42 {
                i.text.italic = true;
            }
            if i.id == -14 {
                for style in &mut i.styles {
                    style.background = 0;
                }
            }
            if i.element == "media-bottom-row-button" {
                i.icon_style = i.styles[0].clone();
                i.icon_style.size = 24.0 * self.density;
                if i.code == keys::codes::IME_UI_MODE_TEXT {
                    for style in &mut i.styles {
                        style.weight = 700;
                    }
                }
            }
            if !matches!(i.command, Command::Action(_) | Command::Drag(..)) {
                i.text = i.styles[0].clone();
                i.icon_style = i.styles[0].clone();
            }
            if matches!(i.command, Command::Emoji(_)) {
                for style in &mut i.styles {
                    style.size = 22.0 * self.density * theme.font_scale();
                }
            }
            if i.element == "media-emoji-tab" {
                i.icon_style.size = 18.0 * self.density;
                if i.selected {
                    i.icon_style = theme.style(&r.element("media-emoji-tab", 2));
                    i.icon_style.size = 18.0 * self.density;
                }
            }
            if matches!(i.command, Command::None) {
                i.text = i.styles[0].clone();
            }
            if i.id == -42 {
                i.text.italic = true;
            }
            i.secondary_style = theme.style(&r.element("clipboard-item-description", 0));
        }
        self.extract_style();
        self.style_chrome(theme, request);
        self.style_subtypes(theme, request);
        if self.kind == 5 {
            for i in &mut self.items {
                if ["gif-search-field", "gif-tile"].contains(&i.element.as_str()) {
                    for style in &mut i.styles {
                        style.background = 0x33808080;
                        style.corners = [6.0 * self.density; 4];
                        style.shape = snygg::value::ShapeKind::Rounded;
                    }
                }
                if i.element == "gif-attribution" {
                    for style in &mut i.styles {
                        style.size = 11.0 * self.density * theme.font_scale();
                    }
                }
                if ["gif-search-field", "gif-message"].contains(&i.element.as_str()) {
                    i.text.size = 14.0 * self.density * theme.font_scale();
                }
                if i.element == "gif-close" {
                    i.glyph = "×".into();
                }
                if i.element == "gif-tile" {
                    i.glyph.clear();
                }
            }
        }
        if self.kind == 4 {
            for i in &mut self.items {
                if i.element == "clipboard-scrim" {
                    for s in &mut i.styles {
                        s.background = (theme.style(&request.element("window", 0)).background
                            & 0x00ffffff)
                            | 0xe0000000;
                    }
                }
            }
        }
        if self.kind == 3 {
            let root = theme.style(&request.element("smartbar-actions-editor", 0));
            for item in self.items.iter_mut().filter(|i| i.id == -11) {
                for style in &mut item.styles {
                    style.shape = root.shape;
                    style.corners = [root.corners[0], root.corners[1], 0.0, 0.0];
                    style.percent = root.percent;
                }
            }
        }
    }
}
fn tile(entry: &Entry, id: i32, bounds: Rect, element: &str, command: Command) -> Item {
    let mut item = Item::new(id, bounds, entry.label.clone(), element, command);
    item.glyph = entry.glyph.clone();
    item.icon = entry.icon.clone();
    item.code = entry.code;
    item.enabled = entry.enabled;
    item
}
pub fn intersection(a: Rect, b: Rect) -> Rect {
    Rect {
        left: a.left.max(b.left),
        right: a.right.min(b.right),
        top: a.top.max(b.top),
        bottom: a.bottom.min(b.bottom),
    }
}
