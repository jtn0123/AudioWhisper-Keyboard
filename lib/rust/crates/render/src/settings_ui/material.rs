//! Material 3 switch tokens; platform supplies the same ColorScheme as Compose.
use super::App;
use crate::panels::{Panel, rect};
use keys::Rect;
impl App {
    pub fn header_color(&self) -> u32 {
        self.colors[if self.screen_scroll() > 0.0 { 12 } else { 0 }]
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

    /// `SwitchDefaults.colors()` (Material 3 1.4.0): the outline (or a checked switch's whole
    /// track), the unchecked inner track and the thumb. A disabled switch uses the disabled
    /// tokens, each `copy(alpha).compositeOver(surface)`; the disabled row then fades it all,
    /// as it fades the row's text, so callers pass these through `enabled_color`.
    pub(super) fn switch_colors(&self, checked: bool, enabled: bool) -> [u32; 3] {
        let [on_surface, primary, highest, on_primary, outline, surface] =
            [1, 3, 5, 6, 7, 8].map(|i| self.colors[i]);
        match (enabled, checked) {
            (true, true) => [primary, primary, on_primary],
            (true, false) => [outline, highest, outline],
            // DisabledSelectedHandleColor is surface at full opacity, the border transparent.
            (false, true) => {
                let track = composite(on_surface, 0.12, surface);
                [track, track, surface]
            }
            (false, false) => [
                composite(on_surface, 0.12, surface),
                composite(highest, 0.12, surface),
                composite(on_surface, 0.38, surface),
            ],
        }
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
        let color = |c| self.enabled_color(c, enabled);
        if linked {
            // Jetpref's `drawLine` with a 2 px stroke along its box's left edge: centred on it.
            self.paint_clipped_box(
                p,
                rect(
                    b.left - 1.0,
                    b.top + b.height() * 0.1,
                    2.0,
                    b.height() * 0.8,
                ),
                color(self.colors[10]),
                0.0,
                clip,
            );
        }
        let [x, y, w, h] = self.switch_track(b, linked);
        // [outline or whole track, inner track (unchecked only), thumb].
        let [outer, inner, thumb] = self.switch_colors(checked, enabled).map(color);
        self.paint_clipped_box(p, rect(x, y, w, h), outer, h / 2.0, clip);
        if !checked {
            // `border(2.dp)` strokes inside the shape, in fractional pixels.
            let line = 2.0 * d;
            let inside = rect(x + line, y + line, w - 2.0 * line, h - 2.0 * line);
            self.paint_clipped_box(p, inside, inner, h / 2.0 - line, clip);
        }
        // M3's ThumbNode: the size and its offset along the track are `toPx().toInt()`, and
        // `Alignment.CenterStart` rounds it to the middle of the track.
        let (size, along) = if checked { (24.0, 24.0) } else { (16.0, 8.0) };
        let size = (size * d).trunc();
        let top = y + ((h - size) / 2.0).round();
        let thumb_box = rect(x + (along * d).trunc(), top, size, size);
        self.paint_clipped_box(p, thumb_box, thumb, size / 2.0, clip);
    }
    /// The Switch's 52 by 32 dp track, `[left, top, width, height]`, where Compose lays it out.
    ///
    /// SwitchPreference sizes the Switch to the 48 dp touch target, 16 dp from the row's end;
    /// ListPreference's linked switch has 8 dp of start padding in its 56 by 48 dp box `b`.
    /// Either way the track (`requiredSize`, each side `roundToPx`) is wider than the space
    /// left, so `Placeable` centres the overflow by integer division, and the rest is centred
    /// with `Alignment.Center`'s rounding. Whole pixels, as Compose's layout always is. The
    /// dialog's switch keeps the earlier fractional placement.
    pub(super) fn switch_track(&self, b: Rect, linked: bool) -> [f32; 4] {
        let [tw, th, target] = [self.px(52.0), self.px(32.0), self.px(48.0)];
        let centre = |space: f32, size: f32| ((space - size) / 2.0).round();
        // `(width - measuredWidth) / 2` in Int arithmetic truncates toward zero.
        let overflow = |space: f32| (space.min(tw) - tw) / 2.0;
        if self.modal.is_some() && !linked {
            let d = self.config[2];
            let y = (b.top + b.bottom) / 2.0 - 16.0 * d;
            return [b.right - 68.0 * d, y, 52.0 * d, 32.0 * d];
        }
        let (left, top, x) = if linked {
            let [pad, width] = [self.px(8.0), self.px(56.0)];
            let fitted = (width - pad).min(tw);
            let x = centre(width, pad + fitted) + pad + overflow(width - pad).trunc();
            (b.left, b.top + centre(b.height(), target), x)
        } else {
            let left = b.right - self.px(16.0) - target;
            let x = centre(target, target.min(tw)) + overflow(target).trunc();
            (left, b.top + centre(b.height(), target), x)
        };
        [left + x, top + centre(target, th), tw, th]
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
            self.paint_clipped_box(
                p,
                rect(left, y - 8.0 * d, x - gap - left, 16.0 * d),
                self.colors[3],
                8.0 * d,
                clip,
            );
            let item = p.items.last_mut().unwrap();
            for style in &mut item.styles {
                style.corners[1] = 2.0 * d;
                style.corners[2] = 2.0 * d;
            }
        }
        if right - x - gap > 8.0 * d {
            self.paint_clipped_box(
                p,
                rect(x + gap, y - 8.0 * d, right - x - gap, 16.0 * d),
                self.colors[4],
                8.0 * d,
                clip,
            );
            let item = p.items.last_mut().unwrap();
            for style in &mut item.styles {
                style.corners[0] = 2.0 * d;
                style.corners[3] = 2.0 * d;
            }
            self.paint_clipped_box(
                p,
                rect(right - 10.0 * d, y - 2.0 * d, 4.0 * d, 4.0 * d),
                self.colors[3],
                2.0 * d,
                clip,
            );
        }
        self.paint_clipped_box(
            p,
            rect(x - 2.0 * d, y - 22.0 * d, 4.0 * d, 44.0 * d),
            self.colors[3],
            2.0 * d,
            clip,
        );
    }
}

/// Compose's `color.copy(alpha).compositeOver(background)` for opaque sRGB colours. An sRGB
/// `Color` packs each component, the copied alpha included, into 8 bits.
pub(super) fn composite(color: u32, alpha: f32, background: u32) -> u32 {
    let alpha = (alpha * 255.0 + 0.5).floor() / 255.0;
    let channel = |shift: u32| {
        let [c, b] = [color, background].map(|v| ((v >> shift) & 255) as f32 / 255.0);
        ((c * alpha + b * (1.0 - alpha)) * 255.0 + 0.5).floor() as u32
    };
    0xff000000 | channel(16) << 16 | channel(8) << 8 | channel(0)
}
