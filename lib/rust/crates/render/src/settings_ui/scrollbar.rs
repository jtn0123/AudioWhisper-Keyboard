//! Frozen Floris scrollbar timing/geometry; Android only supplies frame timestamps.
use super::App;
use crate::panels::{Panel, rect};
use keys::Rect;
impl App {
    pub fn scrollbar_alpha(&self, slot: usize) -> f32 {
        if self.scrollbar_start[slot] < 0 || self.scrollbar_last[slot] < 0 {
            return 1.0;
        }
        let fade_start = (self.scrollbar_start[slot].saturating_add(1_850_000_000))
            .max(self.scrollbar_last[slot]);
        if self.clock <= fade_start {
            return 1.0;
        }
        let x = (self.clock.saturating_sub(fade_start) as f32 / 950_000_000.0).clamp(0.0, 1.0);
        // Compose CubicBezierEasing(1, 0, .82, -.13): solve x, then evaluate y.
        let (mut low, mut high) = (0.0, 1.0);
        for _ in 0..24 {
            let t = (low + high) * 0.5;
            let bx = 3.0 * (1.0 - t) * (1.0 - t) * t + 3.0 * (1.0 - t) * t * t * 0.82 + t * t * t;
            if bx < x {
                low = t;
            } else {
                high = t;
            }
        }
        let t = (low + high) * 0.5;
        (1.0 - (3.0 * (1.0 - t) * t * t * (-0.13) + t * t * t)).clamp(0.0, 1.0)
    }
    pub fn animating(&self) -> bool {
        self.fling.is_some()
            || self.project_animating()
            || (0..3).any(|i| {
                self.scrollbar_exists[i]
                    && (self.scrollbar_start[i] < 0
                        || self.scrollbar_last[i] < 0
                        || self.clock
                            < self.scrollbar_start[i]
                                .saturating_add(1_850_000_000)
                                .max(self.scrollbar_last[i])
                                .saturating_add(950_000_000))
            })
    }
    pub(super) fn paint_scrollbar(&mut self, panel: &mut Panel, clip: Rect) {
        let slot = usize::from(self.modal.is_some());
        // The libraries list is a LazyColumn: its bar follows the visible rows, not pixels.
        let bar = if slot == 0 && self.on_licences() {
            self.licence_scrollbar(clip)
        } else if self.max_scroll > 0.0 {
            let height = clip.height() * clip.height() / (clip.height() + self.max_scroll);
            let top = clip.top + (clip.height() - height) * (self.shown_scroll() / self.max_scroll);
            Some(rect(
                clip.right - 4.0 * self.config[2],
                top,
                4.0 * self.config[2],
                height,
            ))
        } else {
            None
        };
        self.scrollbar_exists[slot] = bar.is_some();
        let Some(bar) = bar else { return };
        let alpha = (self.scrollbar_alpha(slot) * 0.28 * 255.0).round() as u32;
        self.paint_box(
            panel,
            bar,
            (self.colors[1] & 0x00ffffff) | (alpha << 24),
            0.0,
        );
    }
}
