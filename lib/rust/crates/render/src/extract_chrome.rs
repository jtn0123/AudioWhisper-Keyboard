//! Fullscreen extraction keeps the editable field Android-owned, with native chrome around it.
use crate::{
    panels::{Command, Item, Panel, rect},
    smartbar::inset,
    text::Text,
    theme::{Request, Theme},
};
use keys::Rect;
pub fn geometry(width: f32, height: f32, density: f32, button: [f32; 2]) -> [Rect; 2] {
    let w = button[0].min((width - 16.0 * density).max(0.0));
    let h = button[1].min(height);
    [
        rect(0.0, 0.0, (width - w - 16.0 * density).max(0.0), height),
        rect(width - w - 8.0 * density, (height - h) / 2.0, w, h),
    ]
}
impl Panel {
    pub fn measure_chrome(&mut self, theme: &mut Theme, text: &mut Text, r: &Request) {
        let Some(c) = &mut self.chrome else { return };
        if c.kind != 1 {
            return;
        }
        let style = theme.style(&r.element("extracted-landscape-input-action", 0));
        let label = c.labels.first().cloned().unwrap_or_default();
        c.button = [
            text.measure(&label, style.size, style.mono, style.weight)
                + 48.0 * self.density
                + style.padding[0]
                + style.padding[2]
                + style.margin[0]
                + style.margin[2],
            text.height(&label, style.size, style.mono, style.weight)
                + 16.0 * self.density
                + style.padding[1]
                + style.padding[3]
                + style.margin[1]
                + style.margin[3],
        ];
    }
    pub fn layout_extract(&mut self, width: f32, height: f32) {
        let Some(c) = &self.chrome else { return };
        let [field, button] = geometry(width, height, self.density, c.button);
        self.items.clear();
        self.items.push(Item::new(
            -80,
            rect(0.0, 0.0, width, height),
            String::new(),
            "extracted-landscape-input-layout",
            Command::None,
        ));
        self.items.push(Item::new(
            -81,
            field,
            String::new(),
            "extracted-landscape-input-layout",
            Command::None,
        ));
        self.items.push(Item::new(
            800_000,
            button,
            c.labels.first().cloned().unwrap_or_default(),
            "extracted-landscape-input-action",
            Command::Chrome(0),
        ));
    }
    pub fn extract_style(&mut self, theme: &mut Theme, request: &Request) {
        let Some(c) = &mut self.chrome else { return };
        if c.kind != 1 {
            return;
        }
        let Some(item) = self.items.get(1) else {
            return;
        };
        let style = &item.styles[0];
        let r = inset(inset(item.bounds, style.margin), style.padding);
        let field_style = theme.style(&request.element("extracted-landscape-input-field", 0));
        c.field = [
            r.left,
            r.top,
            r.right,
            r.bottom,
            f32::from_bits(field_style.foreground),
            field_style.size,
        ];
    }
}
