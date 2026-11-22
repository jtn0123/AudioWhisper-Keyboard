//! Shared drawing interface: the same keyboard scene feeds both measured Vello backends (Hybrid
//! only with the `hybrid` feature).
use parley::GlyphRun;
use read_fonts::TableProvider;
use vello_cpu::{
    PixmapMut, RenderContext, Resources,
    kurbo::{Affine, BezPath, Rect},
    peniko::color::{AlphaColor, Srgb},
};

pub(crate) struct Cpu {
    context: RenderContext,
    resources: Resources,
    color: crate::color_glyphs::ColorGlyphs,
    /// Rows down the target the scene's top is drawn ([`Backend::set_origin`]).
    origin: u16,
}
#[cfg(feature = "hybrid")]
pub(crate) struct Hybrid {
    pub context: vello_hybrid::Scene,
    pub resources: vello_hybrid::Resources,
    origin: u16,
}
pub(crate) enum Backend {
    Cpu(Box<Cpu>),
    #[cfg(feature = "hybrid")]
    Hybrid(Box<Hybrid>),
}

impl std::fmt::Debug for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Backend")
    }
}

macro_rules! forward {
    ($name:ident($($arg:ident: $ty:ty),*)) => {
        pub fn $name(&mut self, $($arg: $ty),*) {
            match self {
                Self::Cpu(c) => c.context.$name($($arg),*),
                #[cfg(feature = "hybrid")]
                Self::Hybrid(c) => c.context.$name($($arg),*),
            }
        }
    };
}

impl Backend {
    pub fn cpu(width: u16, height: u16) -> Self {
        Self::Cpu(Box::new(Cpu {
            context: RenderContext::new(width, height),
            resources: Resources::new(),
            color: Default::default(),
            origin: 0,
        }))
    }
    #[cfg(feature = "hybrid")]
    pub fn hybrid(width: u16, height: u16, resources: vello_hybrid::Resources) -> Self {
        Self::Hybrid(Box::new(Hybrid {
            context: vello_hybrid::Scene::new(width, height),
            resources,
            origin: 0,
        }))
    }
    forward!(set_paint(paint: AlphaColor<Srgb>));
    forward!(fill_path(path: &BezPath));
    forward!(fill_rect(rect: &Rect));
    forward!(push_clip_path(path: &BezPath));

    /// Draws the scene `rows` down the target, which has that many rows above the scene's top
    /// (the scene's headroom). Every transform is taken from there, so whatever is drawn, colour
    /// glyph tiles included, lands whole pixels lower and otherwise unchanged.
    pub fn set_origin(&mut self, rows: u16) {
        match self {
            Self::Cpu(c) => c.origin = rows,
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c.origin = rows,
        }
        self.reset_transform();
    }
    /// Clears what was drawn and the state, keeping the origin.
    pub fn reset(&mut self) {
        match self {
            Self::Cpu(c) => c.context.reset(),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c.context.reset(),
        }
        self.reset_transform();
    }
    /// `transform`, taken from the origin.
    pub fn set_transform(&mut self, transform: Affine) {
        let origin = match self {
            Self::Cpu(c) => c.origin,
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c.origin,
        };
        let transform = Affine::translate((0.0, f64::from(origin))) * transform;
        match self {
            Self::Cpu(c) => c.context.set_transform(transform),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c.context.set_transform(transform),
        }
    }
    pub fn reset_transform(&mut self) {
        self.set_transform(Affine::IDENTITY);
    }

    pub fn push_clip_rect(&mut self, rect: &Rect) {
        match self {
            Self::Cpu(c) => c.context.push_clip_rect(rect),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c
                .context
                .push_clip_path(&vello_cpu::kurbo::Shape::to_path(rect, 0.1)),
        }
    }
    pub fn pop_clip(&mut self) {
        match self {
            Self::Cpu(c) => c.context.pop_clip(),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c.context.pop_clip_path(),
        }
    }
    pub fn fill_blurred_rounded_rect(&mut self, rect: &Rect, radius: f32, sigma: f32) {
        match self {
            Self::Cpu(c) => c.context.fill_blurred_rounded_rect(rect, radius, sigma),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c
                .context
                .fill_blurred_rounded_rect(rect, radius, sigma, false),
        }
    }
    /// `fill_blurred_rounded_rect`'s blur painted only into `path`. Hybrid paints the whole
    /// blurred rectangle: its callers cover what `path` leaves out.
    pub fn fill_blurred_rounded_rect_in(
        &mut self,
        path: &BezPath,
        rect: &Rect,
        radius: f32,
        sigma: f32,
    ) {
        match self {
            Self::Cpu(c) => c
                .context
                .fill_blurred_rounded_rect_in(path, rect, radius, sigma, false),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(_) => self.fill_blurred_rounded_rect(rect, radius, sigma),
        }
    }
    /// Draws a shaped run; `Ok(true)` when it went through the glyph atlas (Hybrid only).
    pub fn glyphs(&mut self, run: &GlyphRun<'_, ()>, coords: &[i16]) -> Result<bool, String> {
        let glyphs = run.positioned_glyphs().map(|g| (g.id, g.x, g.y));
        self.glyph_list(run.run().font(), run.run().font_size(), coords, glyphs)
    }
    /// Draw positioned glyphs (id, x, y) of one font at the current transform; `Ok(true)` when
    /// they went through the glyph atlas, which only the Hybrid backend still uses.
    pub fn glyph_list(
        &mut self,
        font: &parley::FontData,
        size: f32,
        coords: &[i16],
        glyphs: impl Iterator<Item = (u32, f32, f32)> + Clone,
    ) -> Result<bool, String> {
        let color_font = read_fonts::FontRef::from_index(font.data.as_ref(), font.index)
            .is_ok_and(|font| font.colr().is_ok() || font.cbdt().is_ok() || font.sbix().is_ok());
        match self {
            // vello_cpu's atlas page is a fixed 4096² RGBA pixmap, filled at creation, so one
            // cached glyph would make 64 MiB resident. Outlines draw directly; colour glyphs,
            // which drawn directly decode their bitmaps every frame, use tiles sized to what
            // is shown (`color_glyphs`). Neither reaches the atlas.
            Self::Cpu(c) if color_font => c
                .color
                .draw(&mut c.context, &mut c.resources, font, size, coords, glyphs)
                .map(|()| false),
            Self::Cpu(c) => crate::color_glyphs::direct(
                &mut c.context,
                &mut c.resources,
                font,
                size,
                coords,
                glyphs,
            )
            .map(|()| false),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => {
                // Hybrid requires bitmap glyphs to use its image atlas. Mixing outline
                // glyphs into that atlas loses labels on the tested software Vulkan path;
                // draw ordinary outlines directly and reserve the atlas for colour fonts.
                c.context
                    .glyph_run(&mut c.resources, font)
                    .font_size(size)
                    .normalized_coords(coords)
                    .hint(true)
                    .atlas_cache(color_font)
                    .fill_glyphs(glyphs.map(|(id, x, y)| glifo_hybrid::Glyph { id, x, y }));
                Ok(color_font)
            }
        }
    }
    pub fn image(&mut self, frame: &crate::media_cache::Frame, bounds: keys::Rect) {
        let w = frame.cpu.width() as f64;
        let h = frame.cpu.height() as f64;
        let scale = (bounds.width() as f64 / w).max(bounds.height() as f64 / h);
        self.set_transform(
            Affine::translate((
                bounds.left as f64 + (bounds.width() as f64 - w * scale) / 2.0,
                bounds.top as f64 + (bounds.height() as f64 - h * scale) / 2.0,
            )) * Affine::scale(scale),
        );
        match self {
            Self::Cpu(c) => c.context.set_paint(vello_cpu::Image {
                image: vello_cpu::ImageSource::Pixmap(frame.cpu.clone()),
                sampler: Default::default(),
            }),
            #[cfg(feature = "hybrid")]
            Self::Hybrid(c) => c.context.set_paint(vello_common_hybrid::paint::Image {
                image: vello_common_hybrid::paint::ImageSource::Pixmap(frame.hybrid.clone()),
                sampler: Default::default(),
            }),
        }
        self.fill_rect(&Rect::new(0.0, 0.0, w, h));
        self.reset_transform();
    }
    /// Composite the scene over `target`, the scene's origin at `offset` in it. Vello rasterizes
    /// only the part of the scene the target covers.
    pub fn render<'a>(&mut self, target: impl Into<PixmapMut<'a>>, offset: (i32, i32)) {
        match self {
            Self::Cpu(c) => {
                c.context.render_with(
                    target,
                    &mut c.resources,
                    vello_cpu::RasterizerSettings {
                        target_init: vello_cpu::TargetInit::SrcOver,
                        offset,
                        ..Default::default()
                    },
                );
                c.color.end_frame();
            }
            // The GPU renders the scene itself (`crate::hybrid`).
            #[cfg(feature = "hybrid")]
            Self::Hybrid(_) => {}
        }
    }
    #[cfg(feature = "hybrid")]
    pub fn hybrid_parts(&mut self) -> Option<(&vello_hybrid::Scene, &mut vello_hybrid::Resources)> {
        match self {
            Self::Hybrid(c) => Some((&c.context, &mut c.resources)),
            Self::Cpu(_) => None,
        }
    }
}
