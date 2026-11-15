//! Compose's stretch overscroll on a `verticalScroll` (foundation 1.9's
//! AndroidEdgeEffectOverscrollEffect over the platform EdgeEffect). Only its effect on where
//! content rests is modelled; the stretch itself is not drawn.
//!
//! * A drag past an end pulls that edge by the pixels the scroll could not take, one for one
//!   (`onPullDistance`), up to the viewport's height. Half a pixel or less is rounding noise.
//! * Scrolling the other way, by drag or by fling, relaxes the stretch before it moves content.
//! * A fling toward a stretched edge is absorbed by it; any other fling runs, relaxing first.
//! * The stretch is released when the scroll cycle ends: at a lift with no fling, or when the
//!   fling stops.
//!
//! So a drag that ends at the bottom and lifts with a small backward velocity (an injected
//! gesture's last resampled move) leaves the content at the end, where without this the fling
//! would carry it back.
use super::App;
use crate::panel_scroll::Fling;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Stretch {
    /// Pixels pulled past the top and the bottom.
    pub edges: [f32; 2],
    /// The running fling's stretch to relax before it moves content.
    pub slack: f32,
    /// The scroller's viewport height, the most a stretch can hold.
    pub extent: f32,
}

impl App {
    /// A drag's scroll delta, positive forward: relax, scroll, then pull with what is left.
    pub fn drag_by(&mut self, delta: f32) {
        let left = self.relax(delta);
        let before = self.scroll();
        self.set_scroll(before + left);
        let over = left - (self.scroll() - before);
        if over.abs() > 0.5 {
            let edge = usize::from(over > 0.0);
            let s = &mut self.stretch;
            s.edges[edge] = (s.edges[edge] + over.abs()).min(s.extent.max(0.0));
        }
    }
    /// Spends `delta` against the edge stretched the other way; returns what is left of it.
    fn relax(&mut self, delta: f32) -> f32 {
        let edge = usize::from(delta < 0.0);
        let used = delta.abs().min(self.stretch.edges[edge]);
        self.stretch.edges[edge] -= used;
        delta - used * delta.signum()
    }
    /// The lift that ends a drag: fling with `velocity` (positive forward) unless a stretched
    /// edge absorbs it, and release the stretch when no fling runs.
    pub(super) fn release_drag(&mut self, t: i64, velocity: f32) {
        let toward = usize::from(velocity > 0.0);
        self.fling = if velocity != 0.0 && self.stretch.edges[toward] > 0.0 {
            None
        } else {
            Fling::new(t, self.scroll(), self.config[2], velocity)
        };
        match &self.fling {
            Some(f) => self.stretch.slack = self.stretch.edges[usize::from(f.distance < 0.0)],
            None => self.end_scroll_cycle(),
        }
    }
    /// Where the fling has content at `now`, after relaxing its slack; updates the stretch.
    pub(super) fn fling_value(&mut self, f: &Fling, now: i64) -> f32 {
        let moved = f.value(now) - f.origin;
        let used = moved.abs().min(self.stretch.slack);
        self.stretch.edges[usize::from(f.distance < 0.0)] = self.stretch.slack - used;
        f.origin + moved - used * moved.signum()
    }
    pub(super) fn end_scroll_cycle(&mut self) {
        self.stretch.edges = [0.0; 2];
        self.stretch.slack = 0.0;
    }
}
