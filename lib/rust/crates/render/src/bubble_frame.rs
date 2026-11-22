//! Frames drawn while a press bubble opens, moves, is held or goes, without painting the keys
//! under it again.
//!
//! The bubble is an image laid over the keys (`crate::bubble_cache`). The pixels it lands on are
//! kept, as the scene shows them without it (`Under`). When it goes or moves they are put back;
//! when a frame paints under a held bubble (a shift key, the strip), they take the fresh pixels
//! and the image is laid over those again. So a tap paints its key's highlight, as it did before
//! there was a bubble, and copies and blends the rest. A rectangle that lies inside another one
//! being painted is not painted twice. Every pixel ends as `Raster::draw` paints it.
use crate::{
    bubble_cache::{meet, relaid},
    profile::Profile,
    raster::Raster,
    scene::Scene,
};
use keys::Rect;
use std::ops::Range;

/// The pixels under the bubble the buffer shows, as its scene shows them without it.
pub(crate) struct Under {
    /// The bubble's damage, in the scene's coordinates.
    rect: Rect,
    /// The same rectangle as the buffer's whole pixels.
    pixels: Rect,
    /// Its rows, top to bottom, four bytes a pixel.
    bytes: Vec<u8>,
}

impl Raster {
    /// Repaints `regions`, the damage from `previous` to `scene`, as `draw` does. `previous`
    /// must be the scene the pixels show.
    pub fn draw_frame(
        &mut self,
        previous: &Scene,
        scene: &Scene,
        regions: &[Rect],
    ) -> Result<(), String> {
        if !self.context.is_cpu() {
            return self.draw_regions(scene, regions);
        }
        let mut total = Profile::default();
        let restructured = previous.restructured(scene);
        if restructured && relaid(previous, scene) {
            self.images.clear();
        }
        let same = !restructured && previous.bubble_view() == scene.bubble_view();
        let mut paint = if same || restructured {
            regions.to_vec()
        } else {
            changed_under(previous, scene, regions).to_vec()
        };
        let old = previous.bubble_damage();
        match self
            .under
            .take()
            .filter(|u| !restructured && Some(u.rect) == old)
        {
            Some(under) if same => self.under = Some(under),
            Some(under) => self.put_back(&under, &mut total),
            None => paint.extend(old),
        }
        self.paint_once(scene, &paint, &mut total)?;
        if self.under.is_none()
            && let Some(rect) = scene.bubble_damage()
        {
            self.under = Some(self.keep(scene, rect));
            self.blend_bubble(scene, rect, &mut total)?;
        }
        self.profile = total;
        Ok(())
    }

    /// Paints each of `regions` that lies inside no other one, the first of two the same.
    fn paint_once(
        &mut self,
        scene: &Scene,
        regions: &[Rect],
        total: &mut Profile,
    ) -> Result<(), String> {
        for (i, &r) in regions.iter().enumerate() {
            let twice = regions
                .iter()
                .enumerate()
                .any(|(j, &o)| j != i && within(r, o) && (r != o || j < i));
            if !twice {
                self.paint(scene, r, total)?;
            }
        }
        Ok(())
    }

    /// Paints `r` as `draw` does. Under a held bubble the fresh pixels are kept, and its image
    /// laid over them again.
    fn paint(&mut self, scene: &Scene, r: Rect, total: &mut Profile) -> Result<(), String> {
        self.draw_scene(scene, r)?;
        total.accumulate(self.profile);
        let frame = self.frame();
        let Some(under) = &mut self.under else {
            return Ok(());
        };
        if let Some(fresh) = meet(scene.buffer_pixels(r), under.pixels) {
            let pixels = self.pixmap.data_as_u8_slice();
            copy(&mut under.bytes, under.pixels, pixels, frame, fresh);
        }
        self.blend_bubble(scene, r, total)
    }

    /// The buffer's pixels under the bubble `scene` shows, whose damage is `rect`.
    fn keep(&self, scene: &Scene, rect: Rect) -> Under {
        let pixels = scene.buffer_pixels(rect);
        let mut bytes = vec![0; (pixels.width() * pixels.height()) as usize * 4];
        let buffer = self.pixmap.data_as_u8_slice();
        copy(&mut bytes, pixels, buffer, self.frame(), pixels);
        Under {
            rect,
            pixels,
            bytes,
        }
    }

    /// Puts back the pixels kept under a bubble that has moved or gone.
    fn put_back(&mut self, under: &Under, total: &mut Profile) {
        let frame = self.frame();
        let buffer = self.pixmap.data_as_u8_slice_mut();
        copy(buffer, frame, &under.bytes, under.pixels, under.pixels);
        total.restored_px += (under.pixels.width() * under.pixels.height()) as u64;
    }

    /// The whole buffer, in its pixels.
    fn frame(&self) -> Rect {
        Rect {
            left: 0.0,
            top: 0.0,
            right: f32::from(self.pixmap.width()),
            bottom: f32::from(self.pixmap.height()),
        }
    }
}

/// `regions` without the bubbles' own damage, which `Scene::damage_regions` gives last when the
/// bubble changes: what changed under them. All of `regions` when they don't end so.
fn changed_under<'a>(previous: &Scene, scene: &Scene, regions: &'a [Rect]) -> &'a [Rect] {
    let bubbles: Vec<Rect> = previous
        .bubble_damage()
        .into_iter()
        .chain(scene.bubble_damage())
        .collect();
    regions.strip_suffix(bubbles.as_slice()).unwrap_or(regions)
}

/// Copies `area` from `from`, the rows of `from_frame`, into `to`, the rows of `to_frame`; all
/// three in the buffer's whole pixels, `area` inside both frames.
fn copy(to: &mut [u8], to_frame: Rect, from: &[u8], from_frame: Rect, area: Rect) {
    for (t, f) in spans(to_frame, area).zip(spans(from_frame, area)) {
        to[t].copy_from_slice(&from[f]);
    }
}

/// The bytes each row of `area` takes in the rows of `frame`, four bytes a pixel.
fn spans(frame: Rect, area: Rect) -> impl Iterator<Item = Range<usize>> {
    let line = frame.width() as usize * 4;
    let start = (area.left - frame.left) as usize * 4;
    let width = area.width() as usize * 4;
    let top = frame.top as usize;
    (area.top as usize..area.bottom as usize).map(move |y| {
        let at = (y - top) * line + start;
        at..at + width
    })
}

/// Whether `inner` lies inside `outer`, once both are rounded out to whole pixels as each is
/// painted.
fn within(inner: Rect, outer: Rect) -> bool {
    inner.left.floor() >= outer.left.floor()
        && inner.top.floor() >= outer.top.floor()
        && inner.right.ceil() <= outer.right.ceil()
        && inner.bottom.ceil() <= outer.bottom.ceil()
}
