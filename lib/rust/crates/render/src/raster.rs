//! Vello CPU keeps a backing pixmap and only paints invalidated rectangles.
use crate::profile::{Profile, elapsed};
use crate::{scene::Scene, style::Style, text::Text};
use keys::Rect;
use parley::{Layout, PositionedLayoutItem};
use read_fonts::TableProvider;
use std::collections::HashMap;
use std::time::Instant;
use vello_cpu::{
    Pixmap,
    kurbo::{Affine, RoundedRect, Shape},
    peniko::color::AlphaColor,
};

pub struct Raster {
    pub(crate) text: Text,
    layouts: HashMap<(String, u32, bool, i32), Layout<()>>,
    pub(crate) context: crate::backend::Backend,
    pub(crate) pixmap: Pixmap,
    /// The damaged rectangle's pixels while it renders, when it is not whole rows.
    scratch: Vec<u8>,
    /// The rectangle the frame being drawn repaints; nothing outside it shows.
    pub(crate) damage: Rect,
    profiling: bool,
    pub(crate) profile: Profile,
    /// The pixels under the press bubble the buffer shows, kept to put back when it goes.
    pub(crate) under: Option<crate::bubble_frame::Under>,
    /// The press bubbles drawn once each, to lay over the keys.
    pub(crate) images: crate::bubble_cache::Images,
    /// The labels too big for their boxes, fitted once each (`label_fit`).
    pub(crate) fits: crate::label_fit::Fits,
}

impl std::fmt::Debug for Raster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Raster").finish_non_exhaustive()
    }
}

impl Raster {
    pub fn new(width: u16, height: u16, fonts: Vec<Vec<u8>>) -> Self {
        Self::shared(
            width,
            height,
            fonts.into_iter().map(crate::text::FontBlob::from).collect(),
        )
    }
    pub fn shared(width: u16, height: u16, fonts: Vec<crate::text::FontBlob>) -> Self {
        Self {
            text: Text::shared(fonts),
            layouts: HashMap::new(),
            context: crate::backend::Backend::cpu(width, height),
            pixmap: Pixmap::new(width, height),
            scratch: Vec::new(),
            damage: Rect::default(),
            profiling: false,
            profile: Profile::default(),
            under: None,
            images: Default::default(),
            fits: Default::default(),
        }
    }

    pub fn pixels(&self) -> &[u8] {
        self.pixmap.data_as_u8_slice()
    }

    pub fn set_profiling(&mut self, enabled: bool) {
        self.profiling = enabled;
    }

    pub fn profile(&self) -> Profile {
        self.profile
    }

    #[cfg(feature = "hybrid")]
    pub(crate) fn hybrid(
        width: u16,
        height: u16,
        fonts: Vec<parley::fontique::Blob<u8>>,
        resources: vello_hybrid::Resources,
    ) -> Self {
        let mut raster = Self::shared(width, height, fonts);
        raster.context = crate::backend::Backend::hybrid(width, height, resources);
        raster
    }

    #[cfg(feature = "hybrid")]
    pub(crate) fn hybrid_parts(
        &mut self,
    ) -> Option<(&vello_hybrid::Scene, &mut vello_hybrid::Resources)> {
        self.context.hybrid_parts()
    }

    /// Retain the existing rectangular clip semantics for exact full/dirty pixel parity.
    pub fn draw_regions(&mut self, scene: &Scene, regions: &[Rect]) -> Result<(), String> {
        let mut total = Profile::default();
        for &damage in regions {
            self.draw(scene, damage)?;
            total.accumulate(self.profile);
        }
        self.profile = total;
        Ok(())
    }

    pub fn draw(&mut self, scene: &Scene, damage: Rect) -> Result<(), String> {
        self.under = None;
        self.draw_scene(scene, damage)?;
        if self.context.is_cpu() {
            let mut total = self.profile;
            self.blend_bubble(scene, damage, &mut total)?;
            self.profile = total;
        }
        Ok(())
    }

    /// `draw`, but on the CPU without the press bubble, whose image is laid over the pixels after
    /// (`Raster::blend_bubble`). The GPU draws the bubble here, under the panels and the trail.
    pub(crate) fn draw_scene(&mut self, scene: &Scene, damage: Rect) -> Result<(), String> {
        self.profile = Profile::default();
        let encode = self.profiling.then(Instant::now);
        // Reset entire edge pixels before reapplying antialiasing, rather than accumulating
        // fractional clip coverage over an earlier key state.
        let damage = Rect {
            left: damage.left.floor(),
            top: damage.top.floor(),
            right: damage.right.ceil(),
            bottom: damage.bottom.ceil(),
        };
        self.damage = damage;
        self.context.set_origin(scene.headroom);
        self.context.reset();
        // Nothing but a popup or a press bubble draws above the scene's top, into the headroom.
        let inside = clip(Rect {
            top: damage.top.max(0.0),
            bottom: damage.bottom.max(0.0),
            ..damage
        });
        let modal = crate::modal_scene::content(scene);
        let content = modal.as_ref().unwrap_or(scene);
        self.context.push_clip_rect(&inside);
        self.underneath(scene, content, modal.is_some(), damage)?;
        self.context.pop_clip();
        self.context.push_clip_rect(&clip(damage));
        if !self.context.is_cpu() {
            self.bubble(scene)?;
        }
        self.popup(content)?;
        self.context.pop_clip();
        self.context.push_clip_rect(&inside);
        self.panel(scene)?;
        self.context.set_paint(color(scene.trail_color));
        for circle in &scene.trail {
            self.context.fill_path(
                &vello_cpu::kurbo::Circle::new(
                    (circle.x as f64, circle.y as f64),
                    circle.radius as f64,
                )
                .to_path(0.1),
            );
        }
        self.context.pop_clip();
        self.profile.encode_ns = elapsed(encode);
        let raster = self.profiling.then(Instant::now);
        self.rasterize(scene.buffer_pixels(damage))?;
        self.profile.raster_ns = elapsed(raster);
        Ok(())
    }

    /// Composite the damaged rectangle, in the buffer's pixels, over transparent and store it. Vello unpacks and repacks
    /// every pixel of its target, drawn on or not, so the target is only the damage: a band of
    /// whole rows in place, or else a scratch copied back. The scene keeps surface coordinates
    /// and the target is placed by a negative offset, so each pixel comes out exactly as a full
    /// redraw computes it.
    pub(crate) fn rasterize(&mut self, damage: Rect) -> Result<(), String> {
        let (width, height) = (self.pixmap.width(), self.pixmap.height());
        let clamp = |v: f32, limit: u16| v.clamp(0.0, f32::from(limit)) as u16;
        let (left, right) = (clamp(damage.left, width), clamp(damage.right, width));
        let (top, bottom) = (clamp(damage.top, height), clamp(damage.bottom, height));
        let (w, h) = (right.saturating_sub(left), bottom.saturating_sub(top));
        if w == 0 || h == 0 {
            return Ok(());
        }
        let row = usize::from(width) * 4;
        let rows = usize::from(top) * row..usize::from(top + h) * row;
        let offset = (-i32::from(left), -i32::from(top));
        let pixels = self.pixmap.data_as_u8_slice_mut();
        if w == width {
            let band = &mut pixels[rows];
            band.fill(0);
            let target = vello_cpu::PixmapMut::new(w, h, band).ok_or("damage band size")?;
            self.context.render(target, offset);
            return Ok(());
        }
        self.scratch.clear();
        self.scratch.resize(usize::from(w) * usize::from(h) * 4, 0);
        let target = vello_cpu::PixmapMut::new(w, h, &mut self.scratch).ok_or("damage size")?;
        self.context.render(target, offset);
        let (start, span) = (usize::from(left) * 4, usize::from(w) * 4);
        for (dst, src) in pixels[rows]
            .chunks_exact_mut(row)
            .zip(self.scratch.chunks_exact(span))
        {
            dst[start..start + span].copy_from_slice(src);
        }
        Ok(())
    }

    pub(crate) fn box_at(&mut self, r: Rect, style: &Style) {
        self.context.set_paint(color(style.background));
        let c = style.radii(r.width(), r.height());
        let shape = RoundedRect::new(
            r.left as f64,
            r.top as f64,
            r.right as f64,
            r.bottom as f64,
            vello_cpu::kurbo::RoundedRectRadii::new(
                c[0] as f64,
                c[1] as f64,
                c[2] as f64,
                c[3] as f64,
            ),
        );
        let start = self.profiling.then(Instant::now);
        let path = if style.shape == snygg::value::ShapeKind::Cut {
            let mut p = vello_cpu::kurbo::BezPath::new();
            for (i, (x, y)) in [
                (r.left + c[0], r.top),
                (r.right - c[1], r.top),
                (r.right, r.top + c[1]),
                (r.right, r.bottom - c[2]),
                (r.right - c[2], r.bottom),
                (r.left + c[3], r.bottom),
                (r.left, r.bottom - c[3]),
                (r.left, r.top + c[0]),
            ]
            .into_iter()
            .enumerate()
            {
                if i == 0 {
                    p.move_to((x as f64, y as f64));
                } else {
                    p.line_to((x as f64, y as f64));
                }
            }
            p.close_path();
            p
        } else {
            shape.to_path(0.1)
        };
        self.profile.alloc_ns += elapsed(start);
        self.profile.alloc_sites += u32::from(self.profiling);
        self.context.fill_path(&path);
    }

    pub(crate) fn label(
        &mut self,
        text: &str,
        size: f32,
        argb: u32,
        r: Rect,
        mono: bool,
        weight: i32,
    ) -> Result<(), String> {
        self.squeezed_label(text, (size, mono, weight), 1.0, argb, r)
    }
    /// A label centred in `r` whose glyphs are `squeeze` as wide at the same height, as Android's
    /// text scale X draws them. `font` is (size, mono, weight).
    pub(crate) fn squeezed_label(
        &mut self,
        text: &str,
        font: (f32, bool, i32),
        squeeze: f32,
        argb: u32,
        r: Rect,
    ) -> Result<(), String> {
        self.label_within(text, font, squeeze, argb, r, None)
            .map(drop)
    }
    /// `squeezed_label`, but where `limit` is a box and a number of lines that the label, centred
    /// in `r`, would not stay inside, nothing is drawn and the answer is false.
    pub(crate) fn label_within(
        &mut self,
        text: &str,
        font: (f32, bool, i32),
        squeeze: f32,
        argb: u32,
        r: Rect,
        limit: Option<(Rect, usize)>,
    ) -> Result<bool, String> {
        let (size, mono, weight) = font;
        if text.is_empty() {
            return Ok(true);
        }
        // Bound retained candidate text; no typed/candidate text is ever logged.
        if self.layouts.len() > 256 {
            self.layouts.clear();
        }
        let allocation = self.profiling.then(Instant::now);
        let key = (text.to_owned(), size.to_bits(), mono, weight);
        self.profile.alloc_ns += elapsed(allocation);
        self.profile.alloc_sites += u32::from(self.profiling);
        let shape = self.profiling.then(Instant::now);
        let mut missed = false;
        let layout = self.layouts.entry(key).or_insert_with(|| {
            missed = true;
            self.text.layout_weight(text, size, mono, weight)
        });
        self.profile.shape_ns += elapsed(shape);
        self.profile.layout_misses += u32::from(missed && self.profiling);
        self.profile.labels += u32::from(self.profiling);
        let (width, height) = (layout.width() * squeeze, layout.height());
        let (x, y) = (
            r.left + (r.width() - width) / 2.0,
            r.top + (r.height() - height) / 2.0,
        );
        if let Some((room, lines)) = limit
            && (x < room.left - SLACK
                || x + width > room.right + SLACK
                || y < room.top - SLACK
                || y + height > room.bottom + SLACK
                || layout.len() > lines)
        {
            return Ok(false);
        }
        let (x, y) = (f64::from(x), f64::from(y));
        self.context.set_transform(
            Affine::translate((x, y)) * Affine::scale_non_uniform(f64::from(squeeze), 1.0),
        );
        self.context.set_paint(color(argb));
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item
                    && crate::ink::may_paint(&run, (x, y), f64::from(squeeze), self.damage)
                {
                    let allocation = self.profiling.then(Instant::now);
                    let coords: Vec<_> = run
                        .run()
                        .normalized_coords()
                        .iter()
                        .map(|c| c.to_bits())
                        .collect();
                    self.profile.alloc_ns += elapsed(allocation);
                    self.profile.alloc_sites += u32::from(self.profiling);
                    let glyph = self.profiling.then(Instant::now);
                    let atlas = self.context.glyphs(&run, &coords)?;
                    let glyph_ns = elapsed(glyph);
                    self.profile.glyph_ns += glyph_ns;
                    if self.profiling {
                        self.profile.glyph_runs += 1;
                        self.profile.atlas_runs += u32::from(atlas);
                        // First use of a shaped layout is a cold-work proxy, not a claim
                        // about hits/misses in Vello's private glyph/atlas caches.
                        if missed {
                            self.profile.cold_runs += 1;
                            self.profile.cold_glyph_ns += glyph_ns;
                        }
                        let font = run.run().font();
                        if read_fonts::FontRef::from_index(font.data.as_ref(), font.index)
                            .is_ok_and(|f| f.colr().is_ok() || f.cbdt().is_ok() || f.sbix().is_ok())
                        {
                            self.profile.color_runs += 1;
                            self.profile.color_glyph_ns += glyph_ns;
                        }
                    }
                }
            }
        }
        self.context.reset_transform();
        Ok(true)
    }
}

/// How far past its box a label may reach and still be said to fit: float noise, not a pixel.
const SLACK: f32 = 0.01;

/// `r` as a clip rectangle.
pub(crate) fn clip(r: Rect) -> vello_cpu::kurbo::Rect {
    vello_cpu::kurbo::Rect::new(r.left as f64, r.top as f64, r.right as f64, r.bottom as f64)
}

pub(crate) fn color(argb: u32) -> AlphaColor<vello_cpu::peniko::color::Srgb> {
    AlphaColor::from_rgba8(
        (argb >> 16) as u8,
        (argb >> 8) as u8,
        argb as u8,
        (argb >> 24) as u8,
    )
}
