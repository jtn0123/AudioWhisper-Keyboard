//! Search strip on the same surface as the text keys; Android's existing key dispatcher edits its Rust query rules.
use crate::{
    panels::{Command, Item, Panel, rect},
    text::Text,
    theme::{Request, Theme},
};
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Search {
    pub query: String,
    pub back: String,
    pub prompt: String,
    pub submit: String,
    pub back_icon: String,
    pub submit_icon: String,
    pub cursor: bool,
    pub deadline: i64,
}
impl Search {
    pub fn tick(&mut self, now: i64) -> bool {
        if now < self.deadline {
            return false;
        }
        self.cursor = !self.cursor;
        self.deadline = now + 530_000_000;
        true
    }
}
impl Panel {
    pub fn search_layout(&mut self, width: f32) {
        let Some(s) = &self.search else {
            return;
        };
        let d = self.density;
        let row = self.row;
        self.items.clear();
        self.items.push(Item::new(
            -300,
            rect(0.0, 0.0, width, row),
            String::new(),
            "smartbar",
            Command::None,
        ));
        let mut back = Item::new(
            -301,
            rect(0.0, 0.0, row, row),
            s.back.clone(),
            "clipboard-header-button",
            Command::Gif(9, 0),
        );
        back.icon = s.back_icon.clone();
        back.clip = rect(0.0, 0.0, width, row);
        self.items.push(back);
        let field = rect(row, 6.0 * d, width - 2.0 * row, row - 12.0 * d);
        self.items.push(Item::new(
            -302,
            field,
            String::new(),
            "gif-search-field",
            Command::None,
        ));
        let mut query = Item::new(
            -303,
            rect(
                field.left + 14.0 * d,
                field.top,
                field.width() - 28.0 * d,
                field.height(),
            ),
            if s.query.is_empty() {
                s.prompt.clone()
            } else {
                s.query.clone()
            },
            "search-query",
            Command::None,
        );
        query.multiline = true;
        self.items.push(query);
        let mut submit = Item::new(
            -304,
            rect(width - row, 0.0, row, row),
            s.submit.clone(),
            "clipboard-header-button",
            Command::Gif(10, 0),
        );
        submit.icon = s.submit_icon.clone();
        submit.clip = rect(0.0, 0.0, width, row);
        self.items.push(submit);
        self.items.push(Item::new(
            -305,
            rect(
                field.left + 14.0 * d,
                field.top + (field.height() - 18.0 * d) / 2.0,
                2.0 * d,
                18.0 * d,
            ),
            String::new(),
            "search-cursor",
            Command::None,
        ));
    }
    pub fn search_style(&mut self, theme: &mut Theme, text: &mut Text, r: &Request) {
        let Some(s) = &self.search else {
            return;
        };
        let d = self.density;
        let foreground = theme.style(&r.element("window", 0)).foreground;
        let mut cursor_x = self.row + 14.0 * d;
        for i in &mut self.items {
            if i.element == "gif-search-field" {
                for style in &mut i.styles {
                    style.background = 0x33808080;
                    style.shape = snygg::value::ShapeKind::Rounded;
                    style.percent = true;
                    style.corners = [50.0; 4];
                }
            }
            if i.element == "search-query" {
                i.text.size = 14.0 * d * theme.font_scale();
                if s.query.is_empty() {
                    i.bounds.left += 6.0 * d;
                    i.text.foreground = (i.text.foreground & 0x00ffffff) | 0x99000000;
                }
                let width = i.bounds.width() - 4.0 * d;
                let mut display = i.label.clone();
                if text.measure(&display, i.text.size, i.text.mono, i.text.weight) > width {
                    while !display.is_empty()
                        && text.measure(
                            &format!("{display}…"),
                            i.text.size,
                            i.text.mono,
                            i.text.weight,
                        ) > width
                    {
                        display.pop();
                    }
                    display.push('…');
                }
                cursor_x += if s.query.is_empty() {
                    0.0
                } else {
                    text.measure(&display, i.text.size, i.text.mono, i.text.weight)
                };
                let measured = text.measure(&display, i.text.size, i.text.mono, i.text.weight);
                let height = text.height("Ag", i.text.size, i.text.mono, i.text.weight);
                i.glyph = display;
                i.bounds.top += (i.bounds.height() - height) / 2.0;
                i.bounds.bottom = i.bounds.top + height;
                i.bounds.right = i.bounds.left + measured;
            }
            if i.element == "search-cursor" {
                i.bounds.left = cursor_x;
                i.bounds.right = cursor_x + 2.0 * d;
                for style in &mut i.styles {
                    style.background = if s.cursor { foreground } else { 0 };
                }
            }
        }
    }
}
