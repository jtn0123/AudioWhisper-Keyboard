//! Colour glyphs on the CPU backend, without vello_cpu's glyph atlas.
//!
//! vello_cpu 0.4's atlas page is a fixed 4096×4096 RGBA pixmap, filled when it is made, so the
//! first glyph to use it makes 64 MiB resident. Colour glyphs (COLR, CBDT, sbix) still want a
//! cache: drawn directly, a CBDT glyph decodes its PNG every frame and a COLR glyph repaints
//! every layer. This keeps one trimmed tile per glyph, size, variation and colour instead, so
//! the memory follows what is on screen, and draws it as an image, as the atlas did.
//!
//! Like the atlas, which keys colour glyphs without a subpixel position and samples them with
//! nearest-neighbour, a tile lands on whole device pixels: the glyph origin rounds to the
//! nearest pixel, so a scroll never re-renders. Where the origin is already whole (settings
//! text, which rounds every glyph) the tile is the direct drawing, to within the rounding of
//! compositing through an 8-bit tile.
use std::{collections::HashMap, sync::Arc};
use vello_cpu::{
    Glyph, Image, ImageSource, Pixmap, RasterizerSettings, RenderContext, Resources, TargetInit,
    kurbo::{Affine, Rect},
    peniko::{
        Brush, Extend, ImageAlphaType, ImageQuality, ImageSampler,
        color::{AlphaColor, Srgb},
    },
};

/// Tiles not drawn in the frame just finished are dropped, oldest first, above this.
pub(crate) const BUDGET_BYTES: usize = 4 << 20;
/// Bookkeeping per entry, so empty and direct entries count too.
const ENTRY_BYTES: usize = 64;
/// A glyph whose tile would be wider or taller than this draws directly. Text glyphs are far
/// smaller; this bounds the scratch canvas, not ordinary use.
const MAX_TILE: f32 = 1024.0;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Key {
    font: u64,
    index: u32,
    glyph: u32,
    size: u32,
    color: [u32; 4],
    coords: u64,
}

struct Entry {
    /// `None` for a glyph with no ink (a space or joiner).
    tile: Option<Arc<Pixmap>>,
    /// The tile's top-left corner relative to the glyph origin, in whole pixels.
    at: (f64, f64),
    /// The glyph reached the edge of its scratch canvas: draw it directly, every time.
    direct: bool,
    coords: Box<[i16]>,
    used: u64,
}

impl Entry {
    fn bytes(&self) -> usize {
        ENTRY_BYTES
            + self
                .tile
                .as_ref()
                .map_or(0, |t| usize::from(t.width()) * usize::from(t.height()) * 4)
    }
}

#[derive(Default)]
pub(crate) struct ColorGlyphs {
    entries: HashMap<Key, Entry>,
    bytes: usize,
    frame: u64,
    scratch: Option<(RenderContext, Resources)>,
    /// Tiles rendered since creation (cache misses).
    pub(crate) rendered: u64,
}

fn hash_coords(coords: &[i16]) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    coords.hash(&mut h);
    h.finish()
}

/// Draw glyphs through vello_cpu without its atlas, at the context's current transform.
pub(crate) fn direct(
    ctx: &mut RenderContext,
    resources: &mut Resources,
    font: &parley::FontData,
    size: f32,
    coords: &[i16],
    glyphs: impl Iterator<Item = (u32, f32, f32)> + Clone,
) -> Result<(), String> {
    ctx.glyph_run(resources, font)
        .font_size(size)
        .normalized_coords(coords)
        .hint(true)
        .atlas_cache(false)
        .fill_glyphs(glyphs.map(|(id, x, y)| Glyph { id, x, y }))
        .map_err(|_| "glyph rendering failed".to_owned())
}

impl ColorGlyphs {
    /// Draw a colour font's glyphs (id, x, y) at the context's current transform.
    pub(crate) fn draw(
        &mut self,
        ctx: &mut RenderContext,
        resources: &mut Resources,
        font: &parley::FontData,
        size: f32,
        coords: &[i16],
        glyphs: impl Iterator<Item = (u32, f32, f32)> + Clone,
    ) -> Result<(), String> {
        let [a, b, c, d, tx, ty] = ctx.transform().as_coeffs();
        let color = match ctx.paint() {
            Brush::Solid(color) => Some(*color),
            _ => None,
        };
        let Some(color) = color.filter(|_| (a, b, c, d) == (1.0, 0.0, 0.0, 1.0)) else {
            // Scaled, rotated or image-painted text keeps vello's own (uncached) path.
            return direct(ctx, resources, font, size, coords, glyphs);
        };
        for (id, x, y) in glyphs {
            let key = Key {
                font: font.data.id(),
                index: font.index,
                glyph: id,
                size: size.to_bits(),
                color: color.components.map(f32::to_bits),
                coords: hash_coords(coords),
            };
            if self.entries.get(&key).is_none_or(|e| *e.coords != *coords) {
                let entry = self.render(font, size, coords, id, color)?;
                self.bytes += entry.bytes();
                if let Some(old) = self.entries.insert(key, entry) {
                    self.bytes -= old.bytes();
                }
            }
            let entry = self.entries.get_mut(&key).expect("inserted above");
            entry.used = self.frame;
            if entry.direct {
                direct(
                    ctx,
                    resources,
                    font,
                    size,
                    coords,
                    std::iter::once((id, x, y)),
                )?;
            } else if let Some(tile) = &entry.tile {
                let origin = ((tx + f64::from(x)).round(), (ty + f64::from(y)).round());
                paint_tile(ctx, tile, (origin.0 + entry.at.0, origin.1 + entry.at.1));
            }
        }
        Ok(())
    }

    /// Render one glyph on a scratch canvas with its origin on a whole pixel, then trim it.
    fn render(
        &mut self,
        font: &parley::FontData,
        size: f32,
        coords: &[i16],
        id: u32,
        color: AlphaColor<Srgb>,
    ) -> Result<Entry, String> {
        self.rendered += 1;
        let mut entry = Entry {
            tile: None,
            at: (0.0, 0.0),
            direct: true,
            coords: coords.into(),
            used: self.frame,
        };
        // Room for a glyph from half an em left of its origin to two ems right, and from one
        // and a half ems above its baseline to one em below.
        let em = size.max(1.0);
        let side = (em * 2.5).ceil() + 4.0;
        if !side.is_finite() || side > MAX_TILE {
            return Ok(entry);
        }
        let (w, h) = (side as u16, side as u16);
        let origin = ((em * 0.5).ceil() + 2.0, (em * 1.5).ceil() + 2.0);
        let (ctx, resources) = self
            .scratch
            .get_or_insert_with(|| (RenderContext::new(w, h), Resources::new()));
        if (ctx.width(), ctx.height()) == (w, h) {
            ctx.reset();
        } else {
            ctx.reset_and_resize(w, h);
        }
        ctx.set_transform(Affine::translate((
            f64::from(origin.0),
            f64::from(origin.1),
        )));
        ctx.set_paint(color);
        direct(
            ctx,
            resources,
            font,
            size,
            coords,
            std::iter::once((id, 0.0, 0.0)),
        )?;
        let mut canvas = Pixmap::new(w, h);
        ctx.render_with(
            &mut canvas,
            resources,
            RasterizerSettings {
                target_init: TargetInit::SrcOver,
                ..Default::default()
            },
        );
        match ink(&canvas) {
            Ink::None => entry.direct = false,
            Ink::Clipped => {}
            Ink::Bounds(x0, y0, x1, y1) => {
                entry.tile = Some(Arc::new(crop(&canvas, x0, y0, x1, y1)));
                entry.at = (
                    f64::from(x0) - f64::from(origin.0),
                    f64::from(y0) - f64::from(origin.1),
                );
                entry.direct = false;
            }
        }
        Ok(entry)
    }

    /// Call after each rendered frame: drops tiles the frame did not draw while over budget.
    pub(crate) fn end_frame(&mut self) {
        if self.bytes > BUDGET_BYTES {
            let mut stale: Vec<_> = self
                .entries
                .iter()
                .filter(|(_, e)| e.used < self.frame)
                .map(|(k, e)| (e.used, *k))
                .collect();
            stale.sort_unstable_by_key(|(used, _)| *used);
            for (_, key) in stale {
                if self.bytes <= BUDGET_BYTES {
                    break;
                }
                if let Some(old) = self.entries.remove(&key) {
                    self.bytes -= old.bytes();
                }
            }
        }
        self.frame += 1;
    }

    #[cfg(test)]
    pub(crate) fn bytes(&self) -> usize {
        self.bytes
    }
}

enum Ink {
    None,
    /// Ink touches the canvas edge, so the glyph may be cut off there.
    Clipped,
    Bounds(u16, u16, u16, u16),
}

fn ink(canvas: &Pixmap) -> Ink {
    let (w, h) = (canvas.width(), canvas.height());
    let mut bounds: Option<(u16, u16, u16, u16)> = None;
    for (i, pixel) in canvas.data().iter().enumerate() {
        if pixel.a == 0 {
            continue;
        }
        let (x, y) = ((i % usize::from(w)) as u16, (i / usize::from(w)) as u16);
        if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
            return Ink::Clipped;
        }
        bounds = Some(bounds.map_or((x, y, x + 1, y + 1), |(x0, y0, x1, y1)| {
            (x0.min(x), y0.min(y), x1.max(x + 1), y1.max(y + 1))
        }));
    }
    bounds.map_or(Ink::None, |(x0, y0, x1, y1)| Ink::Bounds(x0, y0, x1, y1))
}

fn crop(canvas: &Pixmap, x0: u16, y0: u16, x1: u16, y1: u16) -> Pixmap {
    let stride = usize::from(canvas.width()) * 4;
    let bytes = canvas.data_as_u8_slice();
    let mut data = Vec::with_capacity(usize::from(x1 - x0) * usize::from(y1 - y0) * 4);
    for y in y0..y1 {
        let row = usize::from(y) * stride;
        data.extend_from_slice(&bytes[row + usize::from(x0) * 4..row + usize::from(x1) * 4]);
    }
    Pixmap::from_parts(
        data,
        x1 - x0,
        y1 - y0,
        vello_cpu::PixelMetadata {
            may_have_transparency: true,
            alpha_type: ImageAlphaType::AlphaPremultiplied,
        },
    )
}

/// Composite a tile with its top-left corner at a whole-pixel position.
fn paint_tile(ctx: &mut RenderContext, tile: &Arc<Pixmap>, at: (f64, f64)) {
    let state = ctx.save_current_state();
    ctx.set_transform(Affine::translate(at));
    ctx.reset_paint_transform();
    ctx.set_paint(Image {
        image: ImageSource::Pixmap(tile.clone()),
        sampler: ImageSampler {
            x_extend: Extend::Pad,
            y_extend: Extend::Pad,
            quality: ImageQuality::Low,
            alpha: 1.0,
        },
    });
    ctx.fill_rect(&Rect::new(
        0.0,
        0.0,
        f64::from(tile.width()),
        f64::from(tile.height()),
    ));
    ctx.restore_state(state);
}

#[cfg(test)]
#[path = "color_glyphs_tests.rs"]
mod tests;
