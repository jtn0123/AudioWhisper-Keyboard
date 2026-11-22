//! What the colour picker adds for a theme colour, as JetPrefColorPicker draws it with
//! `withAlpha = true` and its default `saturationValueAspectRatio` of 1.5: the plane 1.5 times as
//! wide as it is tall, and an alpha bar under the hue bar (a 12 dp pill of the colour fading to
//! clear over the checkered backing, in 12 dp padding) whose thumb shows the colour over white.
//! The accent colour rows' picker keeps its square plane and no alpha bar.
use super::{
    App, color,
    picker::{ColorPicker, Surface},
};
use crate::panels::{Panel, rect};
use keys::Rect;

/// The alpha bar's drag control; the plane's is 7130 and the hue bar's 7131.
pub const ALPHA_ID: i32 = 7132;

impl App {
    /// The Custom page's plane height and the height of the row holding the preview and the
    /// bars, for picker `p` on a page `w` wide. Compose's `aspectRatio` rounds the height; the
    /// row is the 64 dp preview or the column of 36 dp bars, whichever is taller.
    pub(super) fn picker_rows(&self, p: &ColorPicker, w: f32) -> (f32, f32) {
        let (plane, bars) = match p.alpha {
            Some(_) => ((w / 1.5).round(), 2.0),
            None => (w, 1.0),
        };
        (plane, self.px(64.0).max(bars * 3.0 * self.px(12.0)))
    }
    /// The alpha bar on `track` with, outside the editor, its thumb and drag control; nothing
    /// for a picker without one.
    pub(super) fn alpha_bar(&mut self, p: &mut Panel, s: &ColorPicker, track: Rect, clip: Rect) {
        let Some(alpha) = s.alpha else {
            return;
        };
        let (tw, th) = (
            track.width().round().max(1.0) as usize,
            track.height().round().max(1.0) as usize,
        );
        // `checkeredBackground()`'s default grid: half the 12 dp track.
        let even = 0x2900_0000 | self.colors[15] & 0x00ff_ffff;
        let cell = 6.0 * self.config[2];
        let backing = format!("@pill:{tw}x{th}:{even:08x}");
        self.cached(&backing, tw, th, "@pill:", |w, h| {
            color::checkered_box(w, h, h as f32 / 2.0, cell, even)
        });
        self.paint_image(p, &backing, track, clip);
        let rgb = s.color() & 0x00ff_ffff;
        let bar = format!("@alpha:{tw}x{th}:{rgb:06x}");
        self.cached(&bar, tw, th, "@alpha:", |w, h| color::alpha_bar(w, h, rgb));
        self.paint_image(p, &bar, track, clip);
        if s.editing {
            return;
        }
        let centre = track.top + track.height() / 2.0;
        let fill = color::over(s.color(), 0xffff_ffff);
        self.thumb(p, (track.left + track.width() * alpha, centre), fill, clip);
        let touch = rect(
            track.left,
            centre - self.px(24.0),
            track.width(),
            self.px(48.0),
        );
        self.drag_control(ALPHA_ID, touch, Surface::Alpha);
    }
}
