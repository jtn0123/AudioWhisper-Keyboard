//! Shared drawing interface: the same keyboard scene feeds both measured Vello backends.
use parley::GlyphRun;
use read_fonts::TableProvider;
use vello_cpu::{
    Pixmap, RenderContext, Resources,
    kurbo::{Affine, BezPath, Rect, Shape},
    peniko::color::{AlphaColor, Srgb},
};

pub(crate) struct Cpu {
    context: RenderContext,
    resources: Resources,
    bitmap_atlas: bool,
}
pub(crate) struct Hybrid {
    pub context: vello_hybrid::Scene,
    pub resources: vello_hybrid::Resources,
}
pub(crate) enum Backend {
    Cpu(Box<Cpu>),
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
            bitmap_atlas: true,
        }))
    }
    pub fn hybrid(width: u16, height: u16, resources: vello_hybrid::Resources) -> Self {
        Self::Hybrid(Box::new(Hybrid {
            context: vello_hybrid::Scene::new(width, height),
            resources,
        }))
    }
    pub fn settings_scene(&mut self, settings: bool) {
        if let Self::Cpu(cpu) = self {
            cpu.bitmap_atlas = !settings;
        }
    }
    forward!(reset());
    forward!(set_paint(paint: AlphaColor<Srgb>));
    forward!(fill_path(path: &BezPath));
    forward!(fill_rect(rect: &Rect));
    forward!(set_transform(transform: Affine));
    forward!(reset_transform());

    pub fn push_clip_rect(&mut self, rect: &Rect) {
        match self {
            Self::Cpu(c) => c.context.push_clip_rect(rect),
            Self::Hybrid(c) => c.context.push_clip_path(&rect.to_path(0.1)),
        }
    }
    pub fn pop_clip(&mut self) {
        match self {
            Self::Cpu(c) => c.context.pop_clip(),
            Self::Hybrid(c) => c.context.pop_clip_path(),
        }
    }
    pub fn fill_blurred_rounded_rect(&mut self, rect: &Rect, radius: f32, sigma: f32) {
        match self {
            Self::Cpu(c) => c.context.fill_blurred_rounded_rect(rect, radius, sigma),
            Self::Hybrid(c) => c
                .context
                .fill_blurred_rounded_rect(rect, radius, sigma, false),
        }
    }
    pub fn glyphs(&mut self, run: &GlyphRun<'_, ()>, coords: &[i16]) -> Result<(), String> {
        let glyphs = run.positioned_glyphs().map(|g| (g.id, g.x, g.y));
        self.glyph_list(run.run().font(), run.run().font_size(), coords, glyphs)
    }
    /// Draw positioned glyphs (id, x, y) of one font at the current transform.
    pub fn glyph_list(
        &mut self,
        font: &parley::FontData,
        size: f32,
        coords: &[i16],
        glyphs: impl Iterator<Item = (u32, f32, f32)> + Clone,
    ) -> Result<(), String> {
        match self {
            Self::Cpu(c) => c
                .context
                .glyph_run(&mut c.resources, font)
                .font_size(size)
                .normalized_coords(coords)
                .hint(true)
                // Settings draws directly to avoid the 4096² RGBA atlas (64 MiB).
                // Other scenes retain their existing bitmap cache policy.
                .atlas_cache(c.bitmap_atlas)
                .fill_glyphs(glyphs.map(|(id, x, y)| vello_cpu::Glyph { id, x, y }))
                .map_err(|_| "glyph rendering failed".to_owned()),
            Self::Hybrid(c) => {
                // Hybrid requires bitmap glyphs to use its image atlas. Mixing outline
                // glyphs into that atlas loses labels on the tested software Vulkan path;
                // draw ordinary outlines directly and reserve the atlas for colour fonts.
                let color_font = read_fonts::FontRef::from_index(font.data.as_ref(), font.index)
                    .is_ok_and(|font| {
                        font.colr().is_ok() || font.cbdt().is_ok() || font.sbix().is_ok()
                    });
                c.context
                    .glyph_run(&mut c.resources, font)
                    .font_size(size)
                    .normalized_coords(coords)
                    .hint(true)
                    .atlas_cache(color_font)
                    .fill_glyphs(glyphs.map(|(id, x, y)| glifo_hybrid::Glyph { id, x, y }));
                Ok(())
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
            Self::Hybrid(c) => c.context.set_paint(vello_common_hybrid::paint::Image {
                image: vello_common_hybrid::paint::ImageSource::Pixmap(frame.hybrid.clone()),
                sampler: Default::default(),
            }),
        }
        self.fill_rect(&Rect::new(0.0, 0.0, w, h));
        self.reset_transform();
    }
    pub fn render(&mut self, pixmap: &mut Pixmap) {
        if let Self::Cpu(c) = self {
            c.context.render_with(
                pixmap,
                &mut c.resources,
                vello_cpu::RasterizerSettings {
                    target_init: vello_cpu::TargetInit::SrcOver,
                    ..Default::default()
                },
            );
        }
    }
    pub fn hybrid_parts(&mut self) -> Option<(&vello_hybrid::Scene, &mut vello_hybrid::Resources)> {
        match self {
            Self::Hybrid(c) => Some((&c.context, &mut c.resources)),
            Self::Cpu(_) => None,
        }
    }
}
