//! Vello CPU keeps a backing pixmap and only paints invalidated rectangles.
use crate::profile::{Profile, elapsed};
use crate::{scene::Scene, text::Text};
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
    text: Text,
    layouts: HashMap<(String, u32, bool), Layout<()>>,
    context: crate::backend::Backend,
    pixmap: Pixmap,
    profiling: bool,
    profile: Profile,
}

impl std::fmt::Debug for Raster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Raster").finish_non_exhaustive()
    }
}

impl Raster {
    pub fn new(width: u16, height: u16, fonts: Vec<Vec<u8>>) -> Self {
        Self {
            text: Text::new(fonts),
            layouts: HashMap::new(),
            context: crate::backend::Backend::cpu(width, height),
            pixmap: Pixmap::new(width, height),
            profiling: false,
            profile: Profile::default(),
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

    pub(crate) fn hybrid(
        width: u16,
        height: u16,
        fonts: Vec<Vec<u8>>,
        resources: vello_hybrid::Resources,
    ) -> Self {
        let mut raster = Self::new(width, height, fonts);
        raster.context = crate::backend::Backend::hybrid(width, height, resources);
        raster
    }

    pub(crate) fn hybrid_parts(
        &mut self,
    ) -> Option<(&vello_hybrid::Scene, &mut vello_hybrid::Resources)> {
        self.context.hybrid_parts()
    }

    pub fn draw(&mut self, scene: &Scene, damage: Rect) -> Result<(), String> {
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
        self.context.reset();
        let rect = vello_cpu::kurbo::Rect::new(
            damage.left as f64,
            damage.top as f64,
            damage.right as f64,
            damage.bottom as f64,
        );
        self.context.push_clip_rect(&rect);
        self.context.set_paint(color(scene.background));
        self.context.fill_rect(&vello_cpu::kurbo::Rect::new(
            0.0,
            0.0,
            scene.width as f64,
            scene.height as f64,
        ));
        for (i, key) in scene.keys.iter().enumerate() {
            let r = key.bounds.visible;
            if r.is_empty() {
                continue;
            }
            let r = Rect {
                top: r.top + scene.strip,
                bottom: r.bottom + scene.strip,
                ..r
            };
            let margin = key.shadow * 4.0;
            if r.right + margin < damage.left
                || r.left - margin > damage.right
                || r.bottom + margin < damage.top
                || r.top - margin > damage.bottom
            {
                continue;
            }
            let pressed = usize::from(scene.pressed.contains(&i)) * 2;
            if key.shadow > 0.0 {
                self.context.set_paint(color(0x33000000));
                self.context.fill_blurred_rounded_rect(
                    &vello_cpu::kurbo::Rect::new(
                        r.left as f64,
                        (r.top + key.shadow * 0.5) as f64,
                        r.right as f64,
                        (r.bottom + key.shadow * 0.5) as f64,
                    ),
                    key.radius,
                    key.shadow * 0.75,
                );
            }
            self.box_at(r, key.radius, key.colors[pressed]);
            self.label(&key.label, key.size, key.colors[pressed + 1], r, false)?;
            if !key.icon.is_empty() {
                let path = vello_cpu::kurbo::BezPath::from_svg(&key.icon)
                    .map_err(|_| "invalid icon path".to_owned())?;
                self.context.set_transform(
                    Affine::translate((
                        (r.left + (r.width() - key.size) / 2.0) as f64,
                        (r.top + (r.height() - key.size) / 2.0) as f64,
                    )) * Affine::scale(key.size as f64 / 24.0),
                );
                self.context.set_paint(color(key.colors[pressed + 1]));
                self.context.fill_path(&path);
                self.context.reset_transform();
            }
            if !key.hint.is_empty() {
                self.label(
                    &key.hint,
                    key.hint_size,
                    key.hint_colors[pressed / 2],
                    Rect {
                        left: r.right - key.size * 0.6,
                        bottom: r.top + key.size * 0.7,
                        ..r
                    },
                    true,
                )?;
            }
        }
        // Colour glyph preparation is substantial work even when all of its pixels are clipped.
        // A key-only update must not rebuild the unchanged COLR/CBDT candidate strip.
        for (i, candidate) in scene
            .candidates
            .iter()
            .enumerate()
            .filter(|_| damage.top < scene.strip)
        {
            let width = scene.width as f32 / scene.candidates.len() as f32;
            self.context.push_clip_rect(&vello_cpu::kurbo::Rect::new(
                (i as f32 * width) as f64,
                0.0,
                ((i + 1) as f32 * width) as f64,
                scene.strip as f64,
            ));
            self.label(
                candidate,
                scene.candidate_size,
                scene.candidate_color,
                Rect {
                    left: i as f32 * width,
                    right: (i + 1) as f32 * width,
                    top: 0.0,
                    bottom: scene.strip,
                },
                false,
            )?;
            self.context.pop_clip();
        }
        if let (Some(popup), Some(bounds)) = (&scene.popup, scene.popup_bounds()) {
            self.box_at(bounds, 12.0, scene.popup_colors[0]);
            for (i, label) in popup.labels.iter().enumerate() {
                let width = bounds.width() / popup.labels.len() as f32;
                let r = Rect {
                    left: bounds.left + i as f32 * width,
                    right: bounds.left + (i + 1) as f32 * width,
                    ..bounds
                };
                if i == popup.selected {
                    self.box_at(r, 8.0, scene.popup_colors[2]);
                }
                self.label(
                    label,
                    scene.keys[popup.key].size,
                    scene.popup_colors[1],
                    r,
                    false,
                )?;
            }
        }
        self.context.pop_clip();
        self.profile.encode_ns = elapsed(encode);
        let raster = self.profiling.then(Instant::now);
        self.context.render(&mut self.pixmap);
        self.profile.raster_ns = elapsed(raster);
        Ok(())
    }

    fn box_at(&mut self, r: Rect, radius: f32, argb: u32) {
        self.context.set_paint(color(argb));
        let shape = RoundedRect::new(
            r.left as f64,
            r.top as f64,
            r.right as f64,
            r.bottom as f64,
            radius as f64,
        );
        let start = self.profiling.then(Instant::now);
        let path = shape.to_path(0.1);
        self.profile.alloc_ns += elapsed(start);
        self.profile.alloc_sites += u32::from(self.profiling);
        self.context.fill_path(&path);
    }

    fn label(
        &mut self,
        text: &str,
        size: f32,
        argb: u32,
        r: Rect,
        mono: bool,
    ) -> Result<(), String> {
        if text.is_empty() {
            return Ok(());
        }
        // Bound retained candidate text; no typed/candidate text is ever logged.
        if self.layouts.len() > 256 {
            self.layouts.clear();
        }
        let allocation = self.profiling.then(Instant::now);
        let key = (text.to_owned(), size.to_bits(), mono);
        self.profile.alloc_ns += elapsed(allocation);
        self.profile.alloc_sites += u32::from(self.profiling);
        let shape = self.profiling.then(Instant::now);
        let mut missed = false;
        let layout = self.layouts.entry(key).or_insert_with(|| {
            missed = true;
            self.text.layout_style(text, size, mono)
        });
        self.profile.shape_ns += elapsed(shape);
        self.profile.layout_misses += u32::from(missed && self.profiling);
        self.profile.labels += u32::from(self.profiling);
        let x = (r.left + (r.width() - layout.width()) / 2.0) as f64;
        let y = (r.top + (r.height() - layout.height()) / 2.0) as f64;
        self.context.set_transform(Affine::translate((x, y)));
        self.context.set_paint(color(argb));
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
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
                    self.context.glyphs(&run, &coords)?;
                    let glyph_ns = elapsed(glyph);
                    self.profile.glyph_ns += glyph_ns;
                    if self.profiling {
                        self.profile.glyph_runs += 1;
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
        Ok(())
    }
}

fn color(argb: u32) -> AlphaColor<vello_cpu::peniko::color::Srgb> {
    AlphaColor::from_rgba8(
        (argb >> 16) as u8,
        (argb >> 8) as u8,
        argb as u8,
        (argb >> 24) as u8,
    )
}
