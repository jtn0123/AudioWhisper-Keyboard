//! Material 3 switch tokens; platform supplies the same ColorScheme as Compose.
use super::App;
use crate::panels::{Panel, rect};
use keys::Rect;
impl App {
    pub fn header_color(&self) -> u32 {
        self.colors[if self.scrolls[self.screen_index()] > 0.0 {
            12
        } else {
            0
        }]
    }
    pub fn system_bar_color(&self) -> u32 {
        if self.modal.is_none() {
            return self.header_color();
        }
        let channel = |shift: u32| {
            ((((self.header_color() >> shift) & 255) as f32 * 0.4).round() as u32) << shift
        };
        0xff000000 | channel(16) | channel(8) | channel(0)
    }
    pub fn light_system_bars(&self) -> bool {
        let color = self.system_bar_color();
        ((color >> 16) & 255) * 299 + ((color >> 8) & 255) * 587 + (color & 255) * 114 > 128000
    }

    pub(super) fn enabled_color(&self, color: u32, enabled: bool) -> u32 {
        if enabled {
            return color;
        }
        let background = self.colors[if self.modal.is_some() { 11 } else { 0 }];
        let channel = |shift: u32| {
            (((color >> shift) & 255u32) as f32 * 0.38
                + ((background >> shift) & 255u32) as f32 * 0.62)
                .round() as u32
        };
        0xff000000 | channel(16) << 16 | channel(8) << 8 | channel(0)
    }

    pub(super) fn paint_switch(
        &self,
        p: &mut Panel,
        b: Rect,
        checked: bool,
        enabled: bool,
        linked: bool,
        clip: Rect,
    ) {
        let d = self.config[2];
        let x = b.right
            - if linked {
                50.0
            } else if self.modal.is_some() {
                68.0
            } else {
                66.0
            } * d;
        let y = (b.top + b.bottom) / 2.0;
        let color = |c| self.enabled_color(c, enabled);
        if linked {
            // Jetpref draws a 2 physical-pixel divider through 80% of its switch box.
            self.paint_box(
                p,
                rect(b.left, b.top + b.height() * 0.1, 2.0, b.height() * 0.8),
                color(self.colors[10]),
                0.0,
            );
        }
        self.paint_box(
            p,
            rect(x, y - 16.0 * d, 52.0 * d, 32.0 * d),
            color(if checked {
                self.colors[3]
            } else {
                self.colors[7]
            }),
            16.0 * d,
        );
        if !checked {
            self.paint_box(
                p,
                rect(x + 2.0 * d, y - 14.0 * d, 48.0 * d, 28.0 * d),
                color(self.colors[5]),
                14.0 * d,
            );
        }
        let size = if checked { 24.0 } else { 16.0 } * d;
        let cx = x + if checked { 36.0 } else { 16.0 } * d;
        self.paint_box(
            p,
            rect(cx - size / 2.0, y - size / 2.0, size, size),
            color(if checked {
                self.colors[6]
            } else {
                self.colors[7]
            }),
            size / 2.0,
        );
        for item in p.items.iter_mut().rev().take(if checked { 2 } else { 3 }) {
            item.clip = clip;
        }
    }
}

impl App {
    pub(super) fn paint_slider(&self, p: &mut Panel, c: &super::Control, clip: Rect) {
        let d = self.config[2];
        let [min, max, value] = c.range.unwrap();
        let fraction = ((value - min) / (max - min)).clamp(0.0, 1.0);
        let b = c.bounds;
        let y = (b.top + b.bottom) / 2.0;
        let left = b.left + 2.0 * d;
        let right = b.right - 2.0 * d;
        let width = right - left;
        // All prototype sliders have steps. Interior stops account for the 8dp end caps.
        let at = if fraction > 0.0 && fraction < 1.0 {
            8.0 * d + (width - 16.0 * d) * fraction
        } else {
            width * fraction
        };
        let x = left + at.round();
        let gap = 8.0 * d; // 4dp thumb / 2 + 6dp track gap.
        if x - gap - left > 8.0 * d {
            self.paint_box(
                p,
                rect(left, y - 8.0 * d, x - gap - left, 16.0 * d),
                self.colors[3],
                8.0 * d,
            );
            let item = p.items.last_mut().unwrap();
            for style in &mut item.styles {
                style.corners[1] = 2.0 * d;
                style.corners[2] = 2.0 * d;
            }
        }
        if right - x - gap > 8.0 * d {
            self.paint_box(
                p,
                rect(x + gap, y - 8.0 * d, right - x - gap, 16.0 * d),
                self.colors[4],
                8.0 * d,
            );
            let item = p.items.last_mut().unwrap();
            for style in &mut item.styles {
                style.corners[0] = 2.0 * d;
                style.corners[3] = 2.0 * d;
            }
            self.paint_box(
                p,
                rect(right - 10.0 * d, y - 2.0 * d, 4.0 * d, 4.0 * d),
                self.colors[3],
                2.0 * d,
            );
        }
        self.paint_box(
            p,
            rect(x - 2.0 * d, y - 22.0 * d, 4.0 * d, 44.0 * d),
            self.colors[3],
            2.0 * d,
        );
        for item in p.items.iter_mut().rev().take(5) {
            item.clip = crate::panel_layout::intersection(item.clip, clip);
        }
    }
}
