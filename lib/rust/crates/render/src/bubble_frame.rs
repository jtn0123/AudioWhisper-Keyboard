//! Frames that open or close a press bubble, drawn without painting the keys under it again.
//!
//! A bubble covers some 50 000 pixels of keys, labels and shadows. Painting them took about 4 ms
//! of the emulator's CPU when a bubble opened and again when it went, on every tap. So the pixels
//! a bubble opens over are kept, and put back when it goes; only what the scene changed under it
//! meanwhile is painted again. A rectangle that lies inside another one being painted is not
//! painted twice. Every pixel ends as `Raster::draw_regions` paints it.
use crate::{profile::Profile, raster::Raster, scene::Scene};
use keys::Rect;

/// The pixels under an open bubble, from before it drew, and the scene they show.
pub(crate) struct Under {
    /// The scene the pixels show, without its bubble.
    scene: Scene,
    /// The bubble's damage, in the scene's coordinates.
    rect: Rect,
    /// The same rectangle as the buffer's whole pixels.
    pixels: Rect,
    /// Its rows, top to bottom, four bytes a pixel.
    bytes: Vec<u8>,
}

impl Raster {
    /// Repaints `regions`, the damage from `previous` to `scene`, as `draw_regions` does.
    /// `previous` must be the scene the pixels show.
    pub fn draw_frame(
        &mut self,
        previous: &Scene,
        scene: &Scene,
        regions: &[Rect],
    ) -> Result<(), String> {
        let mut total = Profile::default();
        let opened = self.keep_under(previous, scene);
        let shown = if previous.bubble_view() == scene.bubble_view() {
            None
        } else {
            self.put_back(scene, &mut total)?
        };
        if opened.is_some() {
            self.under = opened;
        }
        for (i, &r) in regions.iter().enumerate() {
            let twice = regions
                .iter()
                .enumerate()
                .any(|(j, &o)| j != i && within(r, o) && (r != o || j < i));
            if !twice && !shown.is_some_and(|s| within(r, s)) {
                self.paint(scene, r, &mut total)?;
            }
        }
        self.profile = total;
        Ok(())
    }

    /// The pixels a bubble opening in this frame will cover, while they still show `previous`
    /// without a bubble. None when no bubble opens, when the old bubble's pixels reach them, or
    /// when the frame repaints everything.
    fn keep_under(&self, previous: &Scene, scene: &Scene) -> Option<Under> {
        let rect = scene.bubble_damage()?;
        if previous.bubble_damage().is_some_and(|d| meets(d, rect)) || previous.restructured(scene)
        {
            return None;
        }
        let pixels = previous.buffer_pixels(rect);
        let mut bytes = Vec::with_capacity(span(pixels).1 * rows(pixels).len());
        for row in rows(pixels) {
            bytes.extend_from_slice(&self.pixmap.data_as_u8_slice()[self.at(row, pixels)]);
        }
        let mut scene = previous.clone();
        scene.bubble = None;
        Some(Under {
            scene,
            rect,
            pixels,
            bytes,
        })
    }

    /// Puts back the pixels kept under a bubble that has moved or gone, then paints what the
    /// scene has changed there since: the rectangle returned now shows `scene`. None when
    /// nothing was kept or the scene has been laid out anew.
    fn put_back(&mut self, scene: &Scene, total: &mut Profile) -> Result<Option<Rect>, String> {
        let Some(under) = self.under.take() else {
            return Ok(None);
        };
        if under.scene.restructured(scene) {
            return Ok(None);
        }
        let width = span(under.pixels).1;
        total.restored_px += (under.pixels.width() * under.pixels.height()) as u64;
        for (row, kept) in rows(under.pixels).zip(under.bytes.chunks_exact(width.max(1))) {
            let at = self.at(row, under.pixels);
            self.pixmap.data_as_u8_slice_mut()[at].copy_from_slice(kept);
        }
        for changed in under.scene.damage_regions(scene) {
            if let Some(r) = overlap(changed, under.rect) {
                self.paint(scene, r, total)?;
            }
        }
        Ok(Some(under.rect))
    }

    /// Paints `r` as `draw` does, but where the bubble's box is solid paints the bubble alone.
    fn paint(&mut self, scene: &Scene, r: Rect, total: &mut Profile) -> Result<(), String> {
        let hole = scene.bubble_cover().and_then(|c| overlap(r, c));
        self.draw_over(scene, r, hole)?;
        total.accumulate(self.profile);
        Ok(())
    }

    /// The bytes of `row` that `pixels` spans.
    fn at(&self, row: usize, pixels: Rect) -> std::ops::Range<usize> {
        let (start, width) = span(pixels);
        let line = usize::from(self.pixmap.width()) * 4;
        row * line + start..row * line + start + width
    }
}

/// Where a row of `pixels` starts, in bytes, and how many bytes it takes.
fn span(pixels: Rect) -> (usize, usize) {
    (pixels.left as usize * 4, pixels.width() as usize * 4)
}

fn rows(pixels: Rect) -> std::ops::Range<usize> {
    pixels.top as usize..pixels.bottom as usize
}

/// Whether `inner` lies inside `outer`, once both are rounded out to whole pixels as each is
/// painted.
fn within(inner: Rect, outer: Rect) -> bool {
    inner.left.floor() >= outer.left.floor()
        && inner.top.floor() >= outer.top.floor()
        && inner.right.ceil() <= outer.right.ceil()
        && inner.bottom.ceil() <= outer.bottom.ceil()
}

fn meets(a: Rect, b: Rect) -> bool {
    overlap(a, b).is_some()
}

/// Where `a` and `b` overlap, rounded out to whole pixels; None if they do not.
fn overlap(a: Rect, b: Rect) -> Option<Rect> {
    let r = Rect {
        left: a.left.max(b.left).floor(),
        top: a.top.max(b.top).floor(),
        right: a.right.min(b.right).ceil(),
        bottom: a.bottom.min(b.bottom).ceil(),
    };
    (r.left < r.right && r.top < r.bottom).then_some(r)
}
