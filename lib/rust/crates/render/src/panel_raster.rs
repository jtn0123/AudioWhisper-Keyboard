//! Paint panel items through the same colour-font path as keys and candidates.
use crate::{raster::Raster, scene::Scene, smartbar::inset};
use parley::{Alignment, AlignmentOptions, PositionedLayoutItem};
use vello_cpu::kurbo::{Affine, Rect};
impl Raster {
    pub(crate) fn panel(&mut self, scene: &Scene) -> Result<(), String> {
        for panel in scene
            .panel
            .iter()
            .chain(scene.search.iter())
            .chain(scene.preview.iter())
        {
            for item in &panel.items {
                let clip = crate::panel_layout::intersection(item.bounds, item.clip);
                if clip.is_empty() {
                    continue;
                }
                self.context.push_clip_rect(&Rect::new(
                    clip.left as f64,
                    clip.top as f64,
                    clip.right as f64,
                    clip.bottom as f64,
                ));
                let style = &item.styles[usize::from(panel.pressed == Some(item.id))];
                let r = inset(item.bounds, style.margin);
                self.shadowed_box(r, style);
                let mut r = inset(r, style.padding);
                if item.element.ends_with("-tile") && !item.label.is_empty() {
                    let label = self.panel_text_layout(&item.label, &item.text, r.width(), true);
                    let label_height = label.height().min(item.text.size * 2.4);
                    let total = item.icon_style.size + item.icon_style.margin[3] + label_height;
                    let top = r.top + (r.height() - total) / 2.0;
                    let label_bounds = keys::Rect {
                        top: top + item.icon_style.size + item.icon_style.margin[3],
                        bottom: top + total,
                        ..r
                    };
                    self.panel_text(&item.label, &item.text, label_bounds, true)?;
                    r.top = top;
                    r.bottom = top + item.icon_style.size;
                }
                if item.id == -14
                    || item.id == -15
                    || item.element == "clipboard-scrim"
                    || item.element == "subtype-scrim"
                    || matches!(item.command, crate::panels::Command::Gif(11, _))
                {
                    self.context.pop_clip();
                    continue;
                }
                if item.id == -11 {
                    r.left += 48.0 * panel.density;
                    r.right -= 48.0 * panel.density;
                    r.top += (r.height()
                        - self.text.height(
                            &item.label,
                            item.text.size,
                            item.text.font,
                            item.text.weight,
                        ))
                        / 2.0;
                    self.panel_text(
                        if item.glyph.is_empty() {
                            &item.label
                        } else {
                            &item.glyph
                        },
                        &item.text,
                        r,
                        ["clipboard-content", "gif-message"].contains(&item.element.as_str()),
                    )?;
                    self.context.pop_clip();
                    continue;
                }
                if let Some(frame) = item.media.and_then(|id| panel.media.frame(id)) {
                    self.context.image(frame, r);
                    if let crate::panels::Command::Gif(3, index) = item.command
                        && let Some(g) = &panel.gif
                        && g.sending == Some(index)
                    {
                        self.context.set_paint(crate::raster::color(0x80000000));
                        self.context.fill_rect(&Rect::new(
                            r.left as f64,
                            r.top as f64,
                            r.right as f64,
                            r.bottom as f64,
                        ));
                        let size = 24.0 * panel.density;
                        self.media_spinner(
                            crate::panels::rect(
                                r.left + (r.width() - size) / 2.0,
                                r.top + (r.height() - size) / 2.0,
                                size,
                                size,
                            ),
                            g.phase,
                            0xffffffff,
                        );
                    }
                    self.context.pop_clip();
                    continue;
                }
                if item.element == "gif-spinner" {
                    self.media_spinner(
                        r,
                        panel.gif.as_ref().map_or(0.0, |g| g.phase),
                        style.foreground,
                    );
                    self.context.pop_clip();
                    continue;
                }
                if item.element == "gif-tile" {
                    self.context.pop_clip();
                    continue;
                }
                if item.element == "subtype-panel-list-item" {
                    let size = item.icon_style.size;
                    self.icon(
                        &item.icon,
                        &item.icon_style,
                        crate::panels::rect(r.left, r.top + (r.height() - size) / 2.0, size, size),
                    )?;
                    r.left += size + item.icon_style.padding[2];
                    r.top += (r.height()
                        - self.text.height(
                            &item.label,
                            item.text.size,
                            item.text.font,
                            item.text.weight,
                        ))
                        / 2.0;
                    self.panel_text(&item.label, &item.text, r, false)?;
                    self.context.pop_clip();
                    continue;
                }
                if item.element == "clipboard-item-action" {
                    let icon = item.icon_style.size;
                    self.icon(
                        &item.icon,
                        &item.icon_style,
                        crate::panels::rect(r.left, r.top + (r.height() - icon) / 2.0, icon, icon),
                    )?;
                    r.left += icon + 4.0 * panel.density;
                    r.top += (r.height()
                        - self.text.height(
                            &item.label,
                            item.text.size,
                            item.text.font,
                            item.text.weight,
                        ))
                        / 2.0;
                }
                if item.element == "clipboard-header-text" {
                    r.top += (r.height()
                        - self.text.height(
                            &item.label,
                            item.text.size,
                            item.text.font,
                            item.text.weight,
                        ))
                        / 2.0;
                    self.panel_text(&item.label, &item.text, r, false)?;
                    self.context.pop_clip();
                    continue;
                }
                if item.multiline {
                    r.top -= item.text_offset;
                    if !item.secondary.is_empty() {
                        let h = self.text.height(
                            &item.secondary,
                            item.secondary_style.size,
                            item.secondary_style.font,
                            item.secondary_style.weight,
                        );
                        self.panel_text(
                            &item.secondary,
                            &item.secondary_style,
                            keys::Rect {
                                bottom: r.top + h,
                                ..r
                            },
                            false,
                        )?;
                        r.top += h;
                    }
                    self.panel_text(
                        if item.glyph.is_empty() {
                            &item.label
                        } else {
                            &item.glyph
                        },
                        &item.text,
                        r,
                        ["clipboard-content", "gif-message"].contains(&item.element.as_str()),
                    )?;
                    self.context.pop_clip();
                    continue;
                }
                if item.element.ends_with("subheader") || item.id == -36 || item.id == -42 {
                    if panel.kind == 9
                        && item.text.font == crate::text::Font::Sans
                        && !item.text.italic
                    {
                        self.android_text(&item.label, &item.text, r)?;
                    } else {
                        self.panel_text(&item.label, &item.text, r, false)?;
                    }
                    self.context.pop_clip();
                    continue;
                }
                if !item.icon.is_empty() {
                    self.icon(&item.icon, &item.icon_style, r)?;
                } else {
                    self.label(
                        if item.glyph.is_empty() {
                            &item.label
                        } else {
                            &item.glyph
                        },
                        style.size,
                        style.foreground,
                        r,
                        style.font,
                        style.weight,
                    )?;
                }
                if item.element == "media-emoji-tab" && item.selected {
                    let w = 24.0 * panel.density;
                    let h = 4.0 * panel.density;
                    self.box_at(
                        keys::Rect {
                            left: item.bounds.left + (item.bounds.width() - w) / 2.0,
                            right: item.bounds.left + (item.bounds.width() + w) / 2.0,
                            top: item.bounds.bottom - h,
                            bottom: item.bounds.bottom,
                        },
                        &crate::style::Style {
                            background: item.icon_style.foreground,
                            ..Default::default()
                        },
                    );
                }
                if let crate::panels::Command::Emoji(index) = item.command
                    && item.id < 400_000
                    && panel
                        .emoji
                        .as_ref()
                        .is_some_and(|e| e.category == 0 || !e.variants(index).is_empty())
                {
                    let d = 4.0 * panel.density;
                    let x = item.bounds.right - d;
                    let y = item.bounds.bottom - d;
                    let mut triangle = vello_cpu::kurbo::BezPath::new();
                    triangle.move_to((x as f64, (y - d) as f64));
                    triangle.line_to((x as f64, y as f64));
                    triangle.line_to(((x - d) as f64, y as f64));
                    triangle.close_path();
                    self.context
                        .set_paint(crate::raster::color(panel.indicator.foreground));
                    self.context.fill_path(&triangle);
                }
                self.context.pop_clip();
            }
        }
        Ok(())
    }
    fn media_spinner(&mut self, r: keys::Rect, phase: f32, color: u32) {
        let mut path = vello_cpu::kurbo::BezPath::new();
        let center = (
            (r.left + r.right) as f64 / 2.0,
            (r.top + r.bottom) as f64 / 2.0,
        );
        let radius = r.width() as f64 / 2.0;
        let inner = radius * 0.85;
        let start = phase as f64 * std::f64::consts::TAU;
        for n in 0..=32 {
            let a = start + n as f64 / 32.0 * std::f64::consts::TAU * 0.75;
            let p = (center.0 + radius * a.cos(), center.1 + radius * a.sin());
            if n == 0 {
                path.move_to(p);
            } else {
                path.line_to(p);
            }
        }
        for n in (0..=32).rev() {
            let a = start + n as f64 / 32.0 * std::f64::consts::TAU * 0.75;
            path.line_to((center.0 + inner * a.cos(), center.1 + inner * a.sin()));
        }
        path.close_path();
        self.context.set_paint(crate::raster::color(color));
        self.context.fill_path(&path);
    }
    fn panel_text(
        &mut self,
        text: &str,
        style: &crate::style::Style,
        r: keys::Rect,
        center: bool,
    ) -> Result<(), String> {
        let mut layout = self.panel_text_layout(text, style, r.width(), center);
        layout.set_alignment(
            if center {
                Alignment::Center
            } else {
                Alignment::Start
            },
            AlignmentOptions::default(),
        );
        self.context
            .set_transform(Affine::translate((r.left as f64, r.top as f64)));
        self.context
            .set_paint(crate::raster::color(style.foreground));
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    let coords: Vec<_> = run
                        .run()
                        .normalized_coords()
                        .iter()
                        .map(|c| c.to_bits())
                        .collect();
                    self.context.glyphs(&run, &coords)?;
                }
            }
        }
        self.context.reset_transform();
        Ok(())
    }
    /// Settings text, laid out and drawn the way Android draws hinted Compose text.
    fn android_text(
        &mut self,
        text: &str,
        style: &crate::style::Style,
        r: keys::Rect,
    ) -> Result<(), String> {
        let layout = self.text.android(
            text,
            style.size,
            style.weight,
            [style.line_height, style.letter_spacing],
            Some(r.width()),
        );
        self.context
            .set_paint(crate::raster::color(style.foreground));
        for line in &layout.lines {
            let baseline = r.top + line.baseline;
            for run in &line.runs {
                // No subpixel positioning: every glyph starts on a whole device pixel.
                let glyphs = run
                    .glyphs
                    .iter()
                    .map(|&(id, x, y)| (id, (r.left + x).round(), (baseline + y).round()));
                self.context
                    .glyph_list(&run.font, run.size, &run.coords, glyphs)?;
            }
        }
        Ok(())
    }
    fn panel_text_layout(
        &mut self,
        text: &str,
        style: &crate::style::Style,
        width: f32,
        limit: bool,
    ) -> parley::Layout<()> {
        let mut layout = self.text.layout_typography(
            text,
            style.size,
            style.font,
            style.weight,
            style.italic,
            [style.line_height, style.letter_spacing],
        );
        layout.break_all_lines(Some(width));
        if limit && layout.lines().count() > 2 {
            let end = layout.lines().nth(1).map_or(0, |l| l.text_range().end);
            let mut fitted = text[..end].trim_end().to_owned();
            loop {
                let candidate = format!("{fitted}…");
                layout = self.text.layout_font(
                    &candidate,
                    style.size,
                    style.font,
                    style.weight,
                    style.italic,
                );
                layout.break_all_lines(Some(width));
                if layout.lines().count() <= 2 || fitted.is_empty() {
                    break;
                }
                fitted.pop();
            }
        }
        layout
    }
}
