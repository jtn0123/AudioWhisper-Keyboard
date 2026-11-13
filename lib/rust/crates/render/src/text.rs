//! System font registration, fallback and shaping using Parley.
use parley::fontique::GenericFamily;
use parley::{FontContext, Layout, LayoutContext, StyleProperty};

/// Reused shaping resources, with fonts supplied by the platform host.
pub struct Text {
    fonts: FontContext,
    context: LayoutContext<()>,
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text").finish_non_exhaustive()
    }
}

impl Text {
    pub fn new(fonts: impl IntoIterator<Item = Vec<u8>>) -> Self {
        let mut context = FontContext::new();
        let mut families = Vec::new();
        for bytes in fonts {
            families.extend(
                context
                    .collection
                    .register_fonts(bytes.into(), None)
                    .into_iter()
                    .map(|(id, _)| id),
            );
        }
        let mut emoji = Vec::new();
        let mut mono = Vec::new();
        let mut symbols = Vec::new();
        let mut sans = Vec::new();
        for id in families {
            let name = context.collection.family_name(id).unwrap_or("");
            if name.contains("Emoji") {
                emoji.push(id);
            } else if name.contains("Mono") {
                mono.push(id);
            } else if name.contains("Symbols") {
                symbols.push(id);
            } else {
                sans.push(id);
            }
        }
        // Parley appends the emoji generic for emoji clusters before script fallbacks. Keep
        // Symbols out of the explicit sans family, or flags and VS16 choose monochrome symbols.
        context
            .collection
            .set_generic_families(GenericFamily::Emoji, emoji.into_iter());
        context
            .collection
            .set_generic_families(GenericFamily::SansSerif, sans.iter().copied());
        mono.extend(sans.iter().copied());
        context
            .collection
            .set_generic_families(GenericFamily::Monospace, mono.into_iter());
        for script in [*b"Latn", *b"Zyyy", *b"Zinh"] {
            context.collection.set_fallbacks(
                parley::fontique::Script::from_bytes(script),
                symbols.iter().copied(),
            );
        }
        Self {
            fonts: context,
            context: LayoutContext::new(),
        }
    }

    pub fn layout(&mut self, text: &str, size: f32) -> Layout<()> {
        self.layout_style(text, size, false)
    }

    pub fn layout_style(&mut self, text: &str, size: f32, mono: bool) -> Layout<()> {
        let mut builder = self
            .context
            .ranged_builder(&mut self.fonts, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(size));
        // Unsupported theme families/weights have already selected the regular system
        // fallback in Style; always request its public generic explicitly.
        builder.push_default(StyleProperty::FontFamily(parley::FontFamily::Source(
            if mono { "monospace" } else { "sans-serif" }.into(),
        )));
        let mut layout = builder.build(text);
        layout.break_all_lines(None);
        layout
    }
}
