//! Native panel state and evaluated Android data. Layout and interaction stay in Rust.
use crate::{
    arrangement::{Arrangement, Editor},
    style::Style,
};
use keys::Rect;
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: i32,
    pub code: i32,
    pub enabled: bool,
    pub insert_text: bool,
    pub label: String,
    pub glyph: String,
    pub icon: String,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Command {
    Emoji(usize),
    Category(i32),
    History(i32, usize),
    Dismiss,
    Scroll,
    Action(i32),
    Mode(i32),
    Drag(i32, usize),
    None,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    pub id: i32,
    pub bounds: Rect,
    pub clip: Rect,
    pub label: String,
    pub glyph: String,
    pub icon: String,
    pub element: String,
    pub command: Command,
    pub enabled: bool,
    pub selected: bool,
    pub code: i32,
    pub styles: [Style; 2],
    pub text: Style,
    pub icon_style: Style,
}
impl Item {
    pub fn new(id: i32, bounds: Rect, label: String, element: &str, command: Command) -> Self {
        Self {
            id,
            bounds,
            clip: bounds,
            label,
            glyph: String::new(),
            icon: String::new(),
            element: element.into(),
            command,
            enabled: true,
            selected: false,
            code: 0,
            styles: Default::default(),
            text: Default::default(),
            icon_style: Default::default(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Panel {
    /// 2 overflow, 3 action editor. Emoji uses 1.
    pub kind: i32,
    pub emoji: Option<crate::emoji_panel::EmojiPanel>,
    pub density: f32,
    pub row: f32,
    pub repeat_delay: i64,
    pub gesture_delay: i64,
    pub delete_words: bool,
    pub visible_actions: usize,
    pub entries: Vec<Entry>,
    pub arrangement: Arrangement,
    pub editor: Option<Editor>,
    pub scroll: f32,
    pub overflow_scroll: f32,
    pub emoji_header: f32,
    pub emoji_scrolls: [f32; 10],
    pub message_heights: [f32; 2],
    pub extent: f32,
    pub subheaders: [f32; 3],
    pub grid_margin: [f32; 4],
    pub bottom_sizes: [f32; 2],
    pub customize_size: [f32; 2],
    pub indicator: Style,
    pub max_scroll: f32,
    pub items: Vec<Item>,
    pub titles: Vec<String>,
    pub pressed: Option<i32>,
    pub drag_position: [f32; 2],
}
impl Panel {
    pub fn set_kind(&mut self, kind: i32) {
        if kind != self.kind {
            if kind == 3 {
                self.overflow_scroll = self.scroll;
            }
            self.scroll = if kind == 2 && self.kind == 3 {
                self.overflow_scroll
            } else {
                0.0
            };
            self.pressed = None;
            self.kind = kind;
            if kind == 3 {
                self.editor = Some(Editor::new(self.arrangement.clone()));
            }
        }
    }
    pub fn hit(&self, x: f32, y: f32) -> Option<&Item> {
        self.items.iter().rev().find(|i| {
            i.enabled
                && !matches!(i.command, Command::None | Command::Scroll)
                && inside(crate::smartbar::inset(i.bounds, i.styles[0].margin), x, y)
                && inside(i.clip, x, y)
        })
    }
    pub fn drag_at(&self, x: f32, y: f32) -> Option<Command> {
        let mut closest = None;
        for item in &self.items {
            if !matches!(item.command, Command::Drag(..))
                || item.bounds.bottom < item.clip.top
                || item.bounds.top > item.clip.bottom
            {
                continue;
            }
            if y >= item.bounds.top && y <= item.bounds.bottom {
                if x >= item.bounds.left && x <= item.bounds.right {
                    return Some(item.command.clone());
                }
                closest = Some(item.command.clone());
            }
        }
        closest
    }
    pub fn title(&self, i: usize) -> String {
        self.titles.get(i).cloned().unwrap_or_default()
    }
}
pub fn inside(r: Rect, x: f32, y: f32) -> bool {
    x >= r.left && x < r.right && y >= r.top && y < r.bottom
}
pub fn rect(x: f32, y: f32, width: f32, height: f32) -> Rect {
    Rect {
        left: x,
        top: y,
        right: x + width,
        bottom: y + height,
    }
}
