//! System font registration, fallback and shaping using Parley.
use parley::fontique::GenericFamily;
use parley::{FontContext, Layout, LayoutContext, StyleProperty};
use std::{collections::HashMap, sync::Arc};

pub type FontBlob = parley::fontique::Blob<u8>;

/// Reused shaping resources, with fonts supplied by the platform host.
pub struct Text {
    fonts: FontContext,
    widths: HashMap<(String, u32, bool, i32), (f32, f32)>,
    android: HashMap<(String, [u32; 4], i32), Arc<crate::android_text::Layout>>,
    hinting: crate::android_text::Hinting,
    context: LayoutContext<()>,
}

impl std::fmt::Debug for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Text").finish_non_exhaustive()
    }
}

impl Text {
    pub fn new(fonts: impl IntoIterator<Item = Vec<u8>>) -> Self {
        Self::shared(fonts.into_iter().map(FontBlob::from))
    }
    pub fn shared(fonts: impl IntoIterator<Item = FontBlob>) -> Self {
        let mut context = FontContext::new();
        let mut families = Vec::new();
        for bytes in fonts {
            families.extend(
                context
                    .collection
                    .register_fonts(bytes, None)
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
            android: Default::default(),
            hinting: Default::default(),
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
        let mut layout = self.shape(text, size, [mono, italic], weight, metrics);
        layout.break_all_lines(None);
        layout
    }

    /// The lines `text` breaks into `width` wide, as byte ranges without the white space and
    /// line break that end them, and whether its paragraph runs right to left.
    pub fn wrap(
        &mut self,
        text: &str,
        size: f32,
        mono: bool,
        weight: i32,
        width: f32,
    ) -> (Vec<std::ops::Range<usize>>, bool) {
        let mut layout = self.shape(text, size, [mono, false], weight, [0.0; 2]);
        layout.break_all_lines(Some(width));
        let lines = layout
            .lines()
            .map(|line| {
                let range = line.text_range();
                let kept = text[range.clone()].trim_end().len();
                range.start..range.start + kept
            })
            .collect();
        (lines, layout.is_rtl())
    }

    /// Sans-serif UI text laid out as Android's hinted `TextMotion.Static` text, wrapped at
    /// `width`; see [`crate::android_text`]. `metrics` is line height and letter spacing in px.
    pub fn android(
        &mut self,
        text: &str,
        size: f32,
        weight: i32,
        metrics: [f32; 2],
        width: Option<f32>,
    ) -> Arc<crate::android_text::Layout> {
        let [line_height, letter_spacing] = metrics;
        let bits = [
            size,
            line_height,
            letter_spacing,
            width.unwrap_or(f32::INFINITY),
        ];
        let key = (text.to_owned(), bits.map(f32::to_bits), weight);
        if let Some(layout) = self.android.get(&key) {
            return layout.clone();
        }
        if self.android.len() >= 512 {
            self.android.clear();
        }
        // Shape at the real size without spacing; Minikin adds spacing per cluster itself and,
        // as the device fixture shows, keeps optional ligatures when spacing is set.
        let mut shaped = self.shape(text, size, [false; 2], weight, [line_height, 0.0]);
        shaped.break_all_lines(None);
        let layout = Arc::new(crate::android_text::build(
            &shaped,
            text,
            line_height,
            letter_spacing,
            width,
            &mut self.hinting,
        ));
        self.android.insert(key, layout.clone());
        layout
    }

    fn shape(
        &mut self,
        text: &str,
        size: f32,
        [mono, italic]: [bool; 2],
        weight: i32,
        [line_height, letter_spacing]: [f32; 2],
    ) -> Layout<()> {
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
        builder.build(text)
    }
}
