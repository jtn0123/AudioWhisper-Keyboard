//! System font registration, fallback and shaping using Parley.
use parley::fontique::GenericFamily;
use parley::{FontContext, Layout, LayoutContext, StyleProperty};

/// Reused shaping resources, with fonts supplied by the platform host.
pub struct Text {
    fonts: FontContext,
    widths: std::collections::HashMap<(String, u32, bool, i32), (f32, f32)>,
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
            widths: Default::default(),
            context: LayoutContext::new(),
        }
    }

    pub fn measure(&mut self, text: &str, size: f32, mono: bool, weight: i32) -> f32 {
        self.metrics(text, size, mono, weight).0
    }
    pub fn height(&mut self, text: &str, size: f32, mono: bool, weight: i32) -> f32 {
        self.metrics(text, size, mono, weight).1
    }
    fn metrics(&mut self, text: &str, size: f32, mono: bool, weight: i32) -> (f32, f32) {
        let key = (text.to_owned(), size.to_bits(), mono, weight);
        if let Some(width) = self.widths.get(&key) {
            return *width;
        }
        if self.widths.len() >= 256 {
            self.widths.clear();
        }
        let layout = self.layout_weight(text, size, mono, weight);
        let metrics = (layout.width().ceil(), layout.height().ceil());
        self.widths.insert(key, metrics);
        metrics
    }
    pub fn layout(&mut self, text: &str, size: f32) -> Layout<()> {
        self.layout_style(text, size, false)
    }

    pub fn layout_style(&mut self, text: &str, size: f32, mono: bool) -> Layout<()> {
        self.layout_weight(text, size, mono, 400)
    }

    pub fn layout_weight(&mut self, text: &str, size: f32, mono: bool, weight: i32) -> Layout<()> {
        self.layout_font(text, size, mono, weight, false)
    }
    pub fn layout_font(
        &mut self,
        text: &str,
        size: f32,
        mono: bool,
        weight: i32,
        italic: bool,
    ) -> Layout<()> {
        self.layout_line(text, size, mono, weight, italic, 0.0)
    }
    pub fn layout_line(
        &mut self,
        text: &str,
        size: f32,
        mono: bool,
        weight: i32,
        italic: bool,
        line_height: f32,
    ) -> Layout<()> {
        self.layout_typography(text, size, mono, weight, italic, [line_height, 0.0])
    }
    pub fn layout_typography(
        &mut self,
        text: &str,
        size: f32,
        mono: bool,
        weight: i32,
        italic: bool,
        metrics: [f32; 2],
    ) -> Layout<()> {
        let [line_height, letter_spacing] = metrics;
        let mut builder = self
            .context
            .ranged_builder(&mut self.fonts, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::LetterSpacing(letter_spacing));
        if line_height > 0.0 {
            builder.push_default(StyleProperty::LineHeight(parley::LineHeight::Absolute(
                line_height,
            )));
        }
        builder.push_default(StyleProperty::FontStyle(if italic {
            parley::FontStyle::Italic
        } else {
            parley::FontStyle::Normal
        }));
        builder.push_default(StyleProperty::OverflowWrap(parley::OverflowWrap::Anywhere));
        builder.push_default(StyleProperty::FontWeight(parley::FontWeight::new(
            weight as f32,
        )));
        // Generic families contain Android-supplied real faces and variable fonts.
        builder.push_default(StyleProperty::FontFamily(parley::FontFamily::Source(
            if mono { "monospace" } else { "sans-serif" }.into(),
        )));
        let mut layout = builder.build(text);
        layout.break_all_lines(None);
        layout
    }
}
