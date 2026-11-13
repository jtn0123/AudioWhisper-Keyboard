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
        if self.chrome.is_none() {
            return;
        }
        for item in &mut self.items {
            item.icon_style =
                theme.style(&r.element(&item.element, usize::from(self.pressed == Some(item.id))));
            // SnyggIcon(fillMaxWidth) scales its intrinsic 24dp vector to the available width.
            item.icon_style.size = item.bounds.width();
        }
    }
}
