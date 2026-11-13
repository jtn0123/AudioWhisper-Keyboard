//! IME chrome: geometry and commands are independent of Android/Compose hosts.
use crate::{
    panels::{Command, Item, Panel, rect},
    theme::{Request, Theme},
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chrome {
    pub kind: i32,
    pub labels: Vec<String>,
    pub icons: Vec<String>,
    pub button: [f32; 2],
    pub field: [f32; 6],
}

/// Physical START/END follows the LTR outer IME row, including in RTL locales.
pub fn one_handed(enabled: bool, landscape: bool, scale: i32, end: bool) -> [f32; 3] {
    let keyboard = if !enabled || landscape {
        1.0
    } else {
        scale as f32 / 100.0
    };
    [
        keyboard,
        if enabled && !landscape && end {
            1.0 - keyboard
        } else {
            0.0
        },
        if enabled && !landscape && !end {
            1.0 - keyboard
        } else {
            0.0
        },
    ]
}
pub fn side_buttons(width: f32, height: f32) -> [keys::Rect; 2] {
    let close = width.min(height);
    [
        rect(0.0, 0.0, width, close),
        rect(0.0, close, width, (height - close).max(0.0)),
    ]
}
impl Panel {
    pub fn layout_chrome(&mut self, width: f32, height: f32) {
        if self.chrome.as_ref().is_some_and(|c| c.kind == 1) {
            self.layout_extract(width, height);
            return;
        }
        let Some(c) = &self.chrome else { return };
        self.items.clear();
        let root = rect(0.0, 0.0, width, height);
        self.items.push(Item::new(
            -80,
            root,
            String::new(),
            "one-handed-panel",
            Command::None,
        ));
        for (index, bounds) in side_buttons(width, height).into_iter().enumerate() {
            let mut item = Item::new(
                800_000 + index as i32,
                bounds,
                c.labels.get(index).cloned().unwrap_or_default(),
                "one-handed-panel-button",
                Command::Chrome(index as i32),
            );
            item.icon = c.icons.get(index).cloned().unwrap_or_default();
            self.items.push(item);
        }
    }
    pub fn style_chrome(&mut self, theme: &mut Theme, r: &Request) {
        if self.chrome.as_ref().is_some_and(|c| c.kind == 1) {
            return;
        }
        if self.chrome.is_none() {
            return;
        }
        for item in &mut self.items {
            item.icon_style =
                theme.style(&r.element(&item.element, usize::from(self.pressed == Some(item.id))));
            // The inherited font-size constrains icon height after fillMaxWidth, so its
            // aspect-preserving vector fits that size rather than filling the side panel.
        }
    }
}

/// Shared host sizing retains Compose's multiplication order and portrait-only compaction.
pub fn font_multiplier(base: i32, scale: i32, enabled: bool, portrait: bool) -> f32 {
    base as f32 / 100.0
        * if enabled && portrait {
            scale as f32 / 100.0
        } else {
            1.0
        }
}
pub fn row_heights(
    base: f32,
    portrait: i32,
    landscape: i32,
    scale: i32,
    enabled: bool,
    is_landscape: bool,
) -> [f32; 2] {
    let factor = if is_landscape {
        landscape as f32 / 100.0
    } else {
        portrait as f32 / 100.0 * if enabled { scale as f32 / 100.0 } else { 1.0 }
    };
    let row = base * factor;
    [row, row * 0.753]
}
