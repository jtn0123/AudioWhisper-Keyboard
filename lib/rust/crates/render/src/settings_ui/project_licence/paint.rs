//! Drawing the licence: the lines on screen with the selection's highlight behind them, the
//! handles, a horizontal scrollbar along the viewport's bottom, and the nodes TalkBack reads.
use super::super::{Action, App, Control, shown};
use super::{SCROLL_ID, TEXT_ID, bounds::utf16};
use crate::{
    android_text::Layout,
    panel_layout::intersection,
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

impl App {
    /// The screen's one row: the text, unwrapped and scrolled both ways inside `clip`, the
    /// viewport. Returns the row's height, the text's.
    pub(in crate::settings_ui) fn project_licence(
        &mut self,
        p: &mut Panel,
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        self.project_licence.viewport = clip;
        self.request_project();
        let (Some(layout), Some(source)) =
            (self.project_layout(text), self.project_licence.text.clone())
        else {
            self.scrollbar_exists[2] = false;
            return 0.0;
        };
        self.apply_restored();
        self.project_licence.max_x = (layout.width.ceil() - clip.width()).max(0.0);
        self.set_project_x(self.project_licence.scroll_x);
        // TextSelectionColors' background: the primary colour at 40%.
        let highlight = (self.colors[3] & 0x00ff_ffff) | 0x6600_0000;
        for b in self.project_boxes() {
            self.paint_clipped_box(p, b, highlight, 0.0, clip);
        }
        self.project_lines(p, &layout, &source, clip);
        self.project_handles(p);
        self.project_scrollbar(p, clip);
        self.project_nodes(&layout, &source, clip);
        layout.height.ceil()
    }
    /// The scrolls a recreated screen saved, once the text they scroll is laid out.
    fn apply_restored(&mut self) {
        if let Some([vertical, horizontal]) = self.project_licence.restored.take() {
            if let Some(scroll) = self.scrolls.last_mut() {
                *scroll = vertical;
            }
            self.project_licence.scroll_x = horizontal;
        }
    }
    /// The lines inside `clip`, each one 10 sp monospace item.
    fn project_lines(&mut self, p: &mut Panel, layout: &Layout, source: &str, clip: Rect) {
        let [ox, oy] = self.project_origin();
        let size = self.sp(10);
        for line in &layout.lines {
            let y = oy + line.top;
            let shown = source[line.range.clone()].trim_end_matches('\n');
            if y + line.height <= clip.top || y >= clip.bottom || shown.is_empty() {
                continue;
            }
            let at = rect(ox, y, line.width + size, line.height);
            self.paint_text(p, shown, at, size, self.colors[15], clip);
            let style = &mut p.items.last_mut().expect("paint_text pushed an item").text;
            style.font = crate::text::Font::Mono;
            style.line_height = 0.0;
            style.letter_spacing = 0.0;
            style.weight = 400;
        }
    }
    /// The horizontal scroller's node, and the text's with the whole text as its label.
    fn project_nodes(&mut self, layout: &Layout, source: &str, clip: Rect) {
        let [ox, oy] = self.project_origin();
        let node = |id: i32, bounds: Rect, label: String, role: &'static str| Control {
            id,
            bounds,
            label,
            summary: String::new(),
            role,
            enabled: true,
            checked: None,
            selected: false,
            action: Action::None,
            range: None,
        };
        self.controls
            .push(node(SCROLL_ID, clip, String::new(), "scroll"));
        let shown = intersection(clip, rect(ox, oy, layout.width, layout.height));
        if !source.is_empty() {
            self.controls
                .push(node(TEXT_ID, shown, source.to_owned(), "selectable"));
        }
    }
    /// The selection's handles, each only while its end is in view, as Compose hides a handle
    /// scrolled away. They hang below the line and are not clipped to the viewport, as Compose
    /// draws them in a popup.
    fn project_handles(&mut self, p: &mut Panel) {
        let (Some(range), Some(source)) = (self.project_range(), self.project_licence.text.clone())
        else {
            return;
        };
        let [w, h, d, _] = self.config;
        let v = self.project_licence.viewport;
        let diameter = 25.0 * d;
        for (start, at) in [(true, range.start), (false, range.end)] {
            let Some((x, y)) = self.project_point(at) else {
                continue;
            };
            let bottom = y + 1.0;
            if x < v.left || x > v.right || bottom < v.top || bottom > v.bottom {
                continue;
            }
            let left = if start { x - diameter } else { x };
            let bounds = rect(left, bottom, diameter, diameter);
            let window = rect(0.0, 0.0, w, h);
            self.paint_handle(p, (start, utf16(&source, at)), bounds, start, window);
        }
    }
    /// The horizontal scrollbar: Floris's 4 dp bar and its fade, along the viewport's bottom.
    fn project_scrollbar(&mut self, p: &mut Panel, clip: Rect) {
        let max = self.project_licence.max_x;
        self.scrollbar_exists[2] = max > 0.0;
        if max <= 0.0 {
            return;
        }
        let thickness = 4.0 * self.config[2];
        let width = clip.width() * clip.width() / (clip.width() + max);
        let left =
            clip.left + (clip.width() - width) * (shown(self.project_licence.scroll_x) / max);
        let bar = rect(left, clip.bottom - thickness, width, thickness);
        let alpha = (self.scrollbar_alpha(2) * 0.28 * 255.0).round() as u32;
        self.paint_box(p, bar, (self.colors[1] & 0x00ff_ffff) | (alpha << 24), 0.0);
    }
}
