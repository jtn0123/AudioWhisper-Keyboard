//! Compose's naturally measured confirmation column and trailing ButtonDefaults row.
use crate::{
    panels::*,
    smartbar::inset,
    text::Text,
    theme::{Request, Theme},
};
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClearMetrics {
    pub root: [f32; 4],
    pub row: [f32; 4],
    pub messages: [f32; 2],
    pub buttons: [[f32; 2]; 2],
}
pub fn geometry(width: f32, area: keys::Rect, m: &ClearMetrics, filtered: bool) -> [keys::Rect; 4] {
    let width = width.min(area.width());
    let message = m.messages[usize::from(filtered)];
    let row = m.buttons[0][1].max(m.buttons[1][1]) + m.row[1] + m.row[3];
    let height = m.root[1] + message + row + m.root[3];
    let root = rect(
        area.left + (area.width() - width) / 2.0,
        area.top + (area.height() - height) / 2.0,
        width,
        height,
    );
    let inner = inset(root, m.root);
    let message = rect(inner.left, inner.top, inner.width(), message);
    let x = inner.right - m.row[2] - m.buttons[0][0] - m.buttons[1][0];
    let y = message.bottom + m.row[1];
    [
        root,
        message,
        rect(x, y, m.buttons[0][0], m.buttons[0][1]),
        rect(x + m.buttons[0][0], y, m.buttons[1][0], m.buttons[1][1]),
    ]
}
impl Panel {
    pub fn measure_clear(&mut self, theme: &mut Theme, text: &mut Text, r: &Request, width: f32) {
        let Some(c) = &mut self.clipboard else { return };
        let mut m = ClearMetrics {
            root: theme
                .style(&r.element("clipboard-clear-all-dialog", 0))
                .padding,
            row: theme
                .style(&r.element("clipboard-clear-all-dialog-buttons", 0))
                .padding,
            ..Default::default()
        };
        let message = theme.style(&r.element("clipboard-clear-all-dialog-message", 0));
        let w = (240.0 * self.density).min(width)
            - m.root[0]
            - m.root[2]
            - message.padding[0]
            - message.padding[2];
        for (i, height) in m.messages.iter_mut().enumerate() {
            let mut layout =
                text.layout_weight(&c.label(17 + i), message.size, message.mono, message.weight);
            layout.break_all_lines(Some(w.max(1.0)));
            *height = layout.height() + message.padding[1] + message.padding[3];
        }
        let button = theme.style(&r.element("clipboard-clear-all-dialog-button", 0));
        for (i, size) in m.buttons.iter_mut().enumerate() {
            *size = [
                text.measure(&c.label(19 + i), button.size, button.mono, button.weight)
                    + 48.0 * self.density
                    + button.padding[0]
                    + button.padding[2],
                text.height(&c.label(19 + i), button.size, button.mono, button.weight)
                    + 16.0 * self.density
                    + button.padding[1]
                    + button.padding[3],
            ];
        }
        c.clear_metrics = m;
    }
    pub(crate) fn clear_dialog(&mut self, grid: keys::Rect) {
        let c = self.clipboard.as_ref().unwrap();
        let [root, message, no, yes] =
            geometry(240.0 * self.density, grid, &c.clear_metrics, c.filter_row);
        self.items.push(Item::new(
            -120,
            root,
            String::new(),
            "clipboard-clear-all-dialog",
            Command::None,
        ));
        let mut item = Item::new(
            -121,
            message,
            c.label(if c.filter_row { 18 } else { 17 }),
            "clipboard-clear-all-dialog-message",
            Command::None,
        );
        item.multiline = true;
        self.items.push(item);
        for (n, bounds) in [no, yes].into_iter().enumerate() {
            let mut item = Item::new(
                -122 - n as i32,
                bounds,
                c.label(19 + n),
                "clipboard-clear-all-dialog-button",
                Command::Clipboard(10 + n as i32, 0),
            );
            item.clip = grid;
            self.items.push(item);
        }
    }
}
