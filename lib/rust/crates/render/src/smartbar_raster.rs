//! Paint the same smartbar layout that touch dispatch consumes.
use crate::{
    raster::{Raster, color},
    scene::Scene,
    smartbar::inset,
    style::Style,
};
use keys::Rect;
use vello_cpu::kurbo::Affine;
impl Raster {
    pub(crate) fn smartbar(&mut self, scene: &Scene) -> Result<(), String> {
        let Some(bar) = &scene.smartbar else {
            return Ok(());
        };
        for slot in &bar.layout.actions {
            let action = &bar.actions[slot.id];
            let style = &action.styles[usize::from(bar.pressed.contains(&slot.id))];
            let r = inset(slot.face(), style.margin);
            if action.lit == 0 {
                self.shadowed_box(r, style);
            } else {
                self.shadow(r, style);
                self.lit_action(r, style, action, bar.phase, bar.config.density);
            }
            let r = inset(r, style.padding);
            if action.icon.is_empty() {
                self.label(
                    &action.label,
                    style.size,
                    style.foreground,
                    r,
                    style.mono,
                    style.weight,
                )?;
            } else {
                self.icon(&action.icon, style, r)?;
            }
        }
        let area = bar.layout.center;
        self.context.push_clip_rect(&vello_cpu::kurbo::Rect::new(
            area.left as f64,
            area.top as f64,
            area.right as f64,
            area.bottom as f64,
        ));
        for slot in &bar.layout.candidates {
            let i = slot.id;
            let style =
                &scene.candidate_styles[i][usize::from(scene.candidate_pressed.contains(&i))];
            let r = inset(slot.bounds, style.container.margin);
            if i > 0 && slot.bounds.width() > 0.0 {
                let gap = bar.config.density.round();
                self.box_at(
                    Rect {
                        left: slot.bounds.left - gap,
                        right: slot.bounds.left,
                        top: area.top + area.height() * 0.2,
                        bottom: area.bottom - area.height() * 0.2,
                    },
                    &Style {
                        background: bar.spacer.foreground,
                        ..Style::default()
                    },
                );
            }
            self.box_at(r, &style.container);
            let mut r = inset(r, style.container.padding);
            let (icon, second) = bar
                .aux
                .get(i)
                .map_or(("", ""), |(a, b)| (a.as_str(), b.as_str()));
            if !icon.is_empty() {
                let icon_style = &bar.icons[i];
                let width = icon_style.size.round() + icon_style.margin[0] + icon_style.margin[2];
                self.icon(
                    icon,
                    icon_style,
                    inset(
                        Rect {
                            right: r.left + width,
                            ..r
                        },
                        icon_style.margin,
                    ),
                )?;
                r.left += width;
            }
            // The word squeezes, then shrinks, before it ellipsises, as Compose draws it.
            let text = &scene.candidates[i];
            let t = &style.text;
            let fit = crate::smartbar_fit::fit_text(r.width(), |scale| {
                self.text.measure(text, t.size * scale, t.mono, t.weight)
            });
            let size = t.size * fit.scale;
            if !second.is_empty() {
                let secondary = &bar.secondary[i];
                let primary_height = self.text.height(text, size, t.mono, t.weight);
                let second_height =
                    self.text
                        .height(second, secondary.size, secondary.mono, secondary.weight);
                let total =
                    primary_height + second_height + secondary.margin[1] + secondary.margin[3];
                let top = r.top + (r.height() - total) / 2.0;
                self.fitted_label(
                    second,
                    secondary,
                    (secondary.size, 1.0),
                    Rect {
                        top: top + primary_height + secondary.margin[1],
                        bottom: top + total - secondary.margin[3],
                        ..r
                    },
                )?;
                r.top = top;
                r.bottom = top + primary_height;
            }
            self.fitted_label(text, t, (size, fit.squeeze), r)?;
        }
        self.context.pop_clip();
        Ok(())
    }
    fn lit_action(
        &mut self,
        r: Rect,
        style: &Style,
        action: &crate::smartbar_scene::Action,
        phase: f32,
        density: f32,
    ) {
        let mut disc = style.clone();
        if action.lit == 1 {
            let alpha = (255.0 * 0.35 * action.level) as u32;
            disc.background = (alpha << 24) | (style.background & 0xffffff);
            self.box_at(r, &disc);
        }
        let scale = if action.lit == 1 {
            0.78 + 0.14 * action.level
        } else {
            0.85
        };
        disc.background = if action.finishing && action.lit == 1 {
            (0x80 << 24) | (style.background & 0xffffff)
        } else {
            style.background
        };
        self.box_at(inset(r, [(r.width() * (1.0 - scale)) / 2.0; 4]), &disc);
        if action.warming {
            let cx = (r.left + r.right) / 2.0;
            let cy = (r.top + r.bottom) / 2.0;
            let radius = r.width() / 2.0;
            let mut path = vello_cpu::kurbo::BezPath::new();
            for step in 0..=40 {
                let (part, rad) = if step <= 20 {
                    (step as f32 / 20.0, radius)
                } else {
                    ((40 - step) as f32 / 20.0, radius - 2.0 * density)
                };
                let angle = (phase * 360.0 + part * 100.0).to_radians();
                let p = (
                    (cx + rad * angle.cos()) as f64,
                    (cy + rad * angle.sin()) as f64,
                );
                if step == 0 {
                    path.move_to(p)
                } else {
                    path.line_to(p)
                }
            }
            path.close_path();
            self.context.set_paint(color(style.background));
            self.context.fill_path(&path);
        }
    }
    pub(crate) fn icon(&mut self, svg: &str, style: &Style, r: Rect) -> Result<(), String> {
        let path =
            vello_cpu::kurbo::BezPath::from_svg(svg).map_err(|_| "invalid icon path".to_owned())?;
        self.context.set_transform(
            Affine::translate((
                (r.left + (r.width() - style.size) / 2.0) as f64,
                (r.top + (r.height() - style.size) / 2.0) as f64,
            )) * Affine::scale(style.size as f64 / 24.0),
        );
        self.context.set_paint(color(style.foreground));
        self.context.fill_path(&path);
        self.context.reset_transform();
        Ok(())
    }
    /// `fit` is (size, squeeze): a word too wide at both ellipsises from its start.
    fn fitted_label(
        &mut self,
        text: &str,
        style: &Style,
        fit: (f32, f32),
        r: Rect,
    ) -> Result<(), String> {
        let (size, squeeze) = fit;
        let font = (size, style.mono, style.weight);
        let measured = self.text.measure(text, size, style.mono, style.weight) * squeeze;
        if measured <= r.width() {
            return self.squeezed_label(text, font, squeeze, style.foreground, r);
        }
        let label = crate::smartbar_fit::ellipsised(text, r.width(), |label| {
            self.text.measure(label, size, style.mono, style.weight) * squeeze
        });
        // Compose's overflowing Text occupies the constrained column and draws from its start.
        // Center only text which fit intrinsically (the early return above).
        let fitted = self.text.measure(&label, size, style.mono, style.weight) * squeeze;
        let r = Rect {
            right: r.left + fitted,
            ..r
        };
        self.squeezed_label(&label, font, squeeze, style.foreground, r)
    }
}
