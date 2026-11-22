//! The press bubble drawn once into an image, then laid over the keys on every press.
//!
//! Drawing a bubble, its shadow above all, was most of a 4 ms press frame on the emulator. So
//! each bubble is drawn alone, once, over transparent pixels, into a premultiplied image of the
//! buffer's pixels its damage covers, and a frame lays that image over the keys with a plain
//! source-over. An image is found again only by all that changes its pixels (`Look`): the label
//! as shift and caps lock show it, the key's height, the bubble's bounds, its styles and the
//! buffer's pixels it lands on. Images are kept for the bubbles shown last, `KEPT` of them, and
//! all dropped when the layout or the theme changes (`relaid`). A change of size makes a new
//! surface, and with it a new `Raster` and no images.
use crate::{profile::Profile, raster::Raster, scene::Scene, style::Style};
use keys::Rect;

/// How many bubbles' images are kept: about a letter layout's keys. Each takes four bytes for
/// every pixel of its damage, some 200 KB on a phone 1080 pixels wide.
const KEPT: usize = 32;

/// Everything a bubble's pixels depend on.
#[derive(Clone, Debug, PartialEq)]
struct Look {
    label: String,
    /// The key's visible height, which the label is centred in.
    key_height: f32,
    bounds: Rect,
    style: Style,
    indicator: Option<Style>,
    /// The buffer's whole pixels the bubble's damage covers: where the headroom places it and
    /// the buffer's edges clip it.
    pixels: Rect,
}

/// A bubble drawn alone over transparent pixels.
struct Image {
    look: Look,
    /// Its rows, top to bottom, four premultiplied bytes a pixel.
    bytes: Vec<u8>,
    /// Each row's runs of pixels that show, by whether they hide what lies under them.
    runs: Vec<Vec<Run>>,
}

/// Pixels `start..end` of a row, all opaque (`solid`) or all partly see-through.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Run {
    start: usize,
    end: usize,
    solid: bool,
}

/// The images kept, the one used last at the end.
#[derive(Default)]
pub(crate) struct Images(Vec<Image>);

impl Images {
    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }
}

impl Raster {
    /// How many bubble images are kept.
    pub fn kept_bubbles(&self) -> usize {
        self.images.0.len()
    }

    /// Lays the bubble `scene` shows over the buffer's pixels in `damage`, which must show the
    /// scene without it. Draws its image first if none is kept.
    pub(crate) fn blend_bubble(
        &mut self,
        scene: &Scene,
        damage: Rect,
        total: &mut Profile,
    ) -> Result<(), String> {
        let Some(look) = look(scene) else {
            return Ok(());
        };
        let Some(area) = meet(scene.buffer_pixels(damage), look.pixels) else {
            return Ok(());
        };
        self.bring(look, scene, total)?;
        let line = usize::from(self.pixmap.width()) * 4;
        let image = self.images.0.last().ok_or("no bubble image")?;
        total.bubble_px += lay(self.pixmap.data_as_u8_slice_mut(), line, image, area);
        Ok(())
    }

    /// Moves the kept image for `look` to the end, or draws and keeps it there if there is none.
    fn bring(&mut self, look: Look, scene: &Scene, total: &mut Profile) -> Result<(), String> {
        let kept = &mut self.images.0;
        if let Some(i) = kept.iter().position(|image| image.look == look) {
            kept[i..].rotate_left(1);
        } else {
            let bytes = self.draw_alone(scene, look.pixels)?;
            total.bubble_draws += 1;
            let runs = bytes
                .chunks_exact(look.pixels.width() as usize * 4)
                .map(runs)
                .collect();
            let kept = &mut self.images.0;
            if kept.len() >= KEPT {
                kept.remove(0);
            }
            kept.push(Image { look, bytes, runs });
        }
        Ok(())
    }

    /// The bubble `scene` shows, drawn alone over transparent pixels: the buffer's `pixels`.
    fn draw_alone(&mut self, scene: &Scene, pixels: Rect) -> Result<Vec<u8>, String> {
        let (width, height) = (pixels.width() as u16, pixels.height() as u16);
        let room = f32::from(scene.headroom);
        self.damage = Rect {
            top: pixels.top - room,
            bottom: pixels.bottom - room,
            ..pixels
        };
        self.context.set_origin(scene.headroom);
        self.context.reset();
        self.bubble(scene)?;
        let mut bytes = vec![0; usize::from(width) * usize::from(height) * 4];
        let target = vello_cpu::PixmapMut::new(width, height, &mut bytes).ok_or("bubble size")?;
        self.context
            .render(target, (-(pixels.left as i32), -(pixels.top as i32)));
        Ok(bytes)
    }
}

/// What the bubble `scene` shows looks like; None when it shows none, or none on the buffer.
fn look(scene: &Scene) -> Option<Look> {
    let (bubble, style, bounds) = scene.bubble_view()?;
    let key = &scene.keys[bubble.key];
    let pixels = scene.buffer_pixels(scene.bubble_damage()?);
    (pixels.width() > 0.0 && pixels.height() > 0.0).then(|| Look {
        label: key.label.clone(),
        key_height: key.bounds.visible.height(),
        bounds,
        style: style.clone(),
        indicator: bubble.indicator.clone(),
        pixels,
    })
}

/// Whether the layout or the theme changed from `a` to `b`, which leaves every kept image stale:
/// the keys' places or the colour under them. A label alone changing, as shift does, keeps them.
pub(crate) fn relaid(a: &Scene, b: &Scene) -> bool {
    a.background != b.background
        || a.keys.len() != b.keys.len()
        || a.keys
            .iter()
            .zip(&b.keys)
            .any(|(x, y)| x.bounds != y.bounds)
}

/// A row's runs of pixels that show.
fn runs(row: &[u8]) -> Vec<Run> {
    let mut runs: Vec<Run> = Vec::new();
    for (i, pixel) in row.as_chunks::<4>().0.iter().enumerate() {
        if pixel[3] == 0 {
            continue;
        }
        let solid = pixel[3] == 255;
        match runs.last_mut() {
            Some(run) if run.end == i && run.solid == solid => run.end += 1,
            _ => runs.push(Run {
                start: i,
                end: i + 1,
                solid,
            }),
        }
    }
    runs
}

/// Lays `image` over the buffer's `pixels`, `line` bytes a row, in `area`, which lies inside it.
/// Returns how many pixels it laid.
fn lay(pixels: &mut [u8], line: usize, image: &Image, area: Rect) -> u64 {
    let frame = image.look.pixels;
    let width = frame.width() as usize * 4;
    let left = (area.left - frame.left) as usize;
    let right = (area.right - frame.left) as usize;
    let mut laid = 0;
    for y in area.top as usize..area.bottom as usize {
        let row = y - frame.top as usize;
        let from = &image.bytes[row * width..][..width];
        let to = &mut pixels[y * line + frame.left as usize * 4..][..width];
        for run in &image.runs[row] {
            let (start, end) = (run.start.max(left), run.end.min(right));
            if start >= end {
                continue;
            }
            laid += (end - start) as u64;
            let (to, from) = (&mut to[start * 4..end * 4], &from[start * 4..end * 4]);
            if run.solid {
                to.copy_from_slice(from);
            } else {
                over(to, from);
            }
        }
    }
    laid
}

/// Lays premultiplied `from` over `to`: each byte becomes from + to × (255 − from's alpha) / 255,
/// rounded to the nearest.
fn over(to: &mut [u8], from: &[u8]) {
    for (to, from) in to
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(from.as_chunks::<4>().0)
    {
        let keep = 255 - u16::from(from[3]);
        for (to, &from) in to.iter_mut().zip(from) {
            let x = u16::from(*to) * keep + 128;
            *to = from.saturating_add(((x + (x >> 8)) >> 8) as u8);
        }
    }
}

/// Where `a` and `b`, both whole pixels, overlap; None if they do not.
pub(crate) fn meet(a: Rect, b: Rect) -> Option<Rect> {
    let r = Rect {
        left: a.left.max(b.left),
        top: a.top.max(b.top),
        right: a.right.min(b.right),
        bottom: a.bottom.min(b.bottom),
    };
    (r.left < r.right && r.top < r.bottom).then_some(r)
}

#[cfg(test)]
mod tests {
    use super::{Run, over, runs};

    #[test]
    fn runs_split_a_row_by_what_shows() {
        let row = [
            0, 0, 0, 0, 9, 9, 9, 9, 5, 5, 5, 255, 5, 5, 5, 255, 1, 1, 1, 1,
        ];
        let run = |start, end, solid| Run { start, end, solid };
        assert_eq!(
            runs(&row),
            [run(1, 2, false), run(2, 4, true), run(4, 5, false)]
        );
    }

    #[test]
    fn over_keeps_what_shows_through_and_rounds_to_the_nearest() {
        let mut to = [200, 100, 0, 255];
        over(&mut to, &[64, 0, 32, 128]);
        // 200 × 127 / 255 = 99.6, 100 × 127 / 255 = 49.8, 255 × 127 / 255 = 127.
        assert_eq!(to, [64 + 100, 50, 32, 128 + 127]);
        let mut clear = [7, 7, 7, 7];
        over(&mut clear, &[0, 0, 0, 0]);
        assert_eq!(clear, [7, 7, 7, 7]);
    }
}
