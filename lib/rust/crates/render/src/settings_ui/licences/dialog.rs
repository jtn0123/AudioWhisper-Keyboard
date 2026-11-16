//! AboutLibraries' `LicenseDialog`: a Dialog without the platform width, holding a Surface
//! (shapes.medium, background, at most 560 dp wide with a 16 dp by 8 dp margin) whose Column has
//! the scrolling license text (8 dp padding, bodyLarge) above a TextButton aligned to the end
//! (8 dp by 4 dp padding). The text is laid out once per width; only lines on screen are drawn.
use super::{BODY_ID, Body, html, style::BODY};
use crate::{
    panels::{Panel, rect},
    settings_ui::{Action, App, Control},
    text::Text,
};
use std::sync::Arc;

impl App {
    /// The open library's text as `AnnotatedString.fromHtml` gives it, laid out at `width`.
    fn licence_body(&mut self, text: &mut Text, library: usize, width: f32) -> Option<Arc<str>> {
        let data = self.licences.data.clone()?;
        let lib = data.libraries.get(library)?;
        let size = self.sp(16);
        let stale = self
            .licences
            .body
            .as_ref()
            .is_none_or(|b| b.library != library || b.width != width || b.size != size);
        if stale {
            let html_text = data.html(lib);
            // `takeIf { it.isNotEmpty() }`: no license content, no Text at all.
            let body = if html_text.is_empty() {
                String::new()
            } else {
                html::from_html(&html_text)
            };
            let layout = text.android(
                &body,
                size,
                BODY.weight,
                self.style_metrics(BODY),
                Some(width),
            );
            self.licences.body = Some(Body {
                library,
                width,
                size,
                text: body,
                layout,
            });
        }
        self.licences
            .body
            .as_ref()
            .map(|b| Arc::from(b.text.as_str()))
    }
    pub(in crate::settings_ui) fn licence_dialog(&mut self, p: &mut Panel, text: &mut Text) {
        let Some(m) = self.modal.clone() else { return };
        let library = m.selected.parse::<usize>().unwrap_or(usize::MAX);
        let [w, h, _, _] = self.config;
        let outer = w.min(self.px(560.0));
        let width = (outer - 2.0 * self.px(16.0)).max(1.0);
        let left = ((w - outer) / 2.0).round() + self.px(16.0);
        let pad = self.px(8.0);
        let text_width = (width - 2.0 * pad).max(1.0);
        let body = self
            .licence_body(text, library, text_width)
            .unwrap_or_default();
        let layout = self.licences.body.as_ref().map(|b| b.layout.clone());
        let body_height = match &layout {
            Some(l) if !body.is_empty() => l.height.ceil() + 2.0 * pad,
            _ => 0.0,
        };
        // TextButton: a 40 dp button (labelLarge in 12 dp by 8 dp padding, 58 dp wide at least)
        // in a 48 dp touch target, inside the 8 dp by 4 dp padding the dialog gives it.
        let ok = self.label("action__ok");
        let spacing = 0.1 * self.config[2] * self.config[3];
        let label = text.android(
            &ok,
            self.sp(14),
            500,
            [self.line_height(self.sp(14)), spacing],
            None,
        );
        let button_width = (label.width.ceil() + 2.0 * self.px(12.0)).max(self.px(58.0));
        let button = self.px(40.0).max(label.height.ceil() + 2.0 * self.px(8.0));
        let touch = self.px(48.0).max(button);
        let slot = touch + 2.0 * self.px(4.0);
        let room = (h - 2.0 * pad).max(slot);
        let visible = body_height.min(room - slot).max(0.0);
        let height = visible + slot;
        let top = ((h - height - 2.0 * pad) / 2.0).round() + pad;
        let surface = rect(left, top, width, height);
        let clip = rect(left, top, width, visible.max(1.0));
        self.paint_box(p, rect(0.0, 0.0, w, h), 0x99000000, 0.0);
        self.paint_box(p, surface, self.colors[0], self.px(12.0));
        self.controls.clear();
        let scroll = super::super::shown(m.scroll);
        if let Some(l) = layout.filter(|_| !body.is_empty()) {
            let origin = top + pad - scroll;
            for line in &l.lines {
                let y = origin + line.top;
                if y + line.height <= clip.top || y >= clip.bottom {
                    continue;
                }
                let shown = body[line.range.clone()].trim_end_matches('\n');
                if shown.is_empty() {
                    continue;
                }
                let at = (left + pad, y, text_width, line.height);
                self.paint_style(p, shown, at, BODY, self.colors[15], clip);
            }
        }
        self.max_scroll = (body_height - visible).max(0.0);
        self.stretch.extent = clip.height();
        self.set_scroll(m.scroll);
        self.paint_scrollbar(p, clip);
        let plain = |id: i32, label: String, role: &'static str, action: Action| Control {
            id,
            bounds: clip,
            label,
            summary: String::new(),
            role,
            enabled: true,
            checked: None,
            selected: false,
            action,
            range: None,
        };
        self.controls
            .push(plain(4, String::new(), "scroll", Action::None));
        if !body.is_empty() {
            if self.focus == BODY_ID {
                self.paint_clipped_box(p, clip, self.colors[4], 0.0, clip);
            }
            self.controls
                .push(plain(BODY_ID, body.to_string(), "text", Action::None));
        }
        let x = left + width - self.px(8.0) - button_width;
        let y = top + height - self.px(4.0) - touch;
        self.add(
            p,
            Control {
                bounds: rect(x, y, button_width, touch),
                ..plain(7, ok, "button", Action::Confirm)
            },
            surface,
            text,
        );
    }
}
