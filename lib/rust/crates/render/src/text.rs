//! System font registration, fallback and shaping using Parley.
use parley::fontique::{FontInfoOverride, GenericFamily};
use parley::{FontContext, FontFamily, FontFamilyName, Layout, LayoutContext, StyleProperty};
use std::{collections::HashMap, sync::Arc};

pub type FontBlob = parley::fontique::Blob<u8>;

/// The family text is drawn in: the platform's sans-serif or monospace chain, or a theme's
/// `@font` ([`crate::theme_fonts`]) tried before the sans-serif chain. `false` and `true` are
/// sans-serif and monospace, as the `mono` flag this replaced was.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Font {
    #[default]
    Sans,
    Mono,
    Theme(u32),
}

impl From<bool> for Font {
    fn from(mono: bool) -> Self {
        if mono { Self::Mono } else { Self::Sans }
    }
}

/// Reused shaping resources, with fonts supplied by the platform host.
pub struct Text {
    fonts: FontContext,
    widths: HashMap<(String, u32, Font, i32), (f32, f32)>,
    android: HashMap<(String, [u32; 4], i32), Arc<crate::android_text::Layout>>,
    hinting: crate::android_text::Hinting,
    context: LayoutContext<()>,
    /// Each theme font this context has registered, by id, under its alias; `None` when none of
    /// its faces registered.
    themed: HashMap<u32, Option<Arc<str>>>,
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
            themed: HashMap::new(),
        }
    }

    pub fn measure(&mut self, text: &str, size: f32, font: impl Into<Font>, weight: i32) -> f32 {
        self.metrics(text, size, font, weight).0
    }
    pub fn height(&mut self, text: &str, size: f32, font: impl Into<Font>, weight: i32) -> f32 {
        self.metrics(text, size, font, weight).1
    }
    fn metrics(&mut self, text: &str, size: f32, font: impl Into<Font>, weight: i32) -> (f32, f32) {
        let font = font.into();
        let key = (text.to_owned(), size.to_bits(), font, weight);
        if let Some(width) = self.widths.get(&key) {
            return *width;
        }
        if self.widths.len() >= 256 {
            self.widths.clear();
        }
        let layout = self.layout_weight(text, size, font, weight);
        let metrics = (layout.width().ceil(), layout.height().ceil());
        self.widths.insert(key, metrics);
        metrics
    }
    pub fn layout(&mut self, text: &str, size: f32) -> Layout<()> {
        self.layout_style(text, size, Font::Sans)
    }

    pub fn layout_style(&mut self, text: &str, size: f32, font: impl Into<Font>) -> Layout<()> {
        self.layout_weight(text, size, font, 400)
    }

    pub fn layout_weight(
        &mut self,
        text: &str,
        size: f32,
        font: impl Into<Font>,
        weight: i32,
    ) -> Layout<()> {
        self.layout_font(text, size, font, weight, false)
    }
    pub fn layout_font(
        &mut self,
        text: &str,
        size: f32,
        font: impl Into<Font>,
        weight: i32,
        italic: bool,
    ) -> Layout<()> {
        self.layout_line(text, size, font, weight, italic, 0.0)
    }
    pub fn layout_line(
        &mut self,
        text: &str,
        size: f32,
        font: impl Into<Font>,
        weight: i32,
        italic: bool,
        line_height: f32,
    ) -> Layout<()> {
        self.layout_typography(text, size, font, weight, italic, [line_height, 0.0])
    }
    pub fn layout_typography(
        &mut self,
        text: &str,
        size: f32,
        font: impl Into<Font>,
        weight: i32,
        italic: bool,
        metrics: [f32; 2],
    ) -> Layout<()> {
        let mut layout = self.shape(text, size, (font.into(), italic), weight, metrics);
        layout.break_all_lines(None);
        layout
    }

    /// The lines `text` breaks into `width` wide, as byte ranges without the white space and
    /// line break that end them, and whether its paragraph runs right to left.
    pub fn wrap(
        &mut self,
        text: &str,
        size: f32,
        font: impl Into<Font>,
        weight: i32,
        width: f32,
    ) -> (Vec<std::ops::Range<usize>>, bool) {
        let mut layout = self.shape(text, size, (font.into(), false), weight, [0.0; 2]);
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
        let mut shaped = self.shape(text, size, (Font::Sans, false), weight, [line_height, 0.0]);
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
        (font, italic): (Font, bool),
        weight: i32,
        [line_height, letter_spacing]: [f32; 2],
    ) -> Layout<()> {
        let family = self.family(font);
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
        builder.push_default(StyleProperty::FontFamily(family));
        builder.build(text)
    }

    /// `font`'s family list. Generic families contain Android-supplied real faces and variable
    /// fonts; a theme font goes before the sans-serif one, so a glyph it lacks falls through to
    /// exactly the chain a theme without fonts uses, and one that did not register is that chain.
    fn family(&mut self, font: Font) -> FontFamily<'static> {
        let sans = || FontFamily::Source("sans-serif".into());
        match font {
            Font::Sans => sans(),
            Font::Mono => FontFamily::Source("monospace".into()),
            Font::Theme(id) => match self.theme_alias(id) {
                Some(alias) => FontFamily::List(
                    vec![
                        FontFamilyName::Named(alias.to_string().into()),
                        FontFamilyName::Generic(GenericFamily::SansSerif),
                    ]
                    .into(),
                ),
                None => sans(),
            },
        }
    }

    /// The alias theme font `id` is registered under in this context, registering its faces
    /// with their `@font` weight and style the first time.
    fn theme_alias(&mut self, id: u32) -> Option<Arc<str>> {
        if let Some(alias) = self.themed.get(&id) {
            return alias.clone();
        }
        let alias: Arc<str> = format!("audiowhisper theme font {id}").into();
        let mut registered = false;
        for face in crate::theme_fonts::family(id).iter().flat_map(|f| f.iter()) {
            let info = FontInfoOverride {
                family_name: Some(&alias),
                weight: Some(parley::FontWeight::new(face.weight as f32)),
                style: Some(if face.italic {
                    parley::FontStyle::Italic
                } else {
                    parley::FontStyle::Normal
                }),
                ..FontInfoOverride::default()
            };
            let families = self
                .fonts
                .collection
                .register_fonts(face.blob.clone(), Some(info));
            registered |= !families.is_empty();
        }
        let alias = registered.then_some(alias);
        self.themed.insert(id, alias.clone());
        alias
    }
}
