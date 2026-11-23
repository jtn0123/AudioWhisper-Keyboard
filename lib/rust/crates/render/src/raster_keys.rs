//! What a frame draws under the long-press popup and the press bubble: the background, a
//! modal's box, the watermark, the keys and the strip.
use crate::{
    label_fit::Place,
    raster::{Raster, color},
    scene::{Key, Scene},
    smartbar::inset,
    style::Style,
};
use keys::Rect;
use vello_cpu::kurbo::Affine;

impl Raster {
    /// Everything in `damage` under the popup and the bubble. `content` is the scene's modal
    /// content where `modal` says it has one, else the scene.
    pub(crate) fn underneath(
        &mut self,
        scene: &Scene,
        content: &Scene,
        modal: bool,
        damage: Rect,
    ) -> Result<(), String> {
        self.context.set_paint(color(
            if scene.panel.as_ref().is_some_and(|p| p.kind == 3) {
                0x52000000
            } else if scene.panel.as_ref().is_some_and(|p| p.kind == 7) {
                0
            } else {
                scene.background
            },
        ));
        self.context.fill_rect(&vello_cpu::kurbo::Rect::new(
            0.0,
            0.0,
            scene.width as f64,
            scene.height as f64,
        ));
        let content_top = if modal {
            crate::modal_scene::offset(scene.height as f32, scene.strip, scene.keyboard_height)
        } else {
            0.0
        };
        if modal {
            self.box_at(
                Rect {
                    left: 0.0,
                    top: content_top,
                    right: scene.width as f32,
                    bottom: scene.height as f32,
                },
                &Style {
                    background: scene.background,
                    ..Default::default()
                },
            );
        }
        if let Some(w) = &content.watermark {
            self.icon(&w.icon, &w.style, w.bounds)?;
        }
        for (i, key) in content
            .keys
            .iter()
            .enumerate()
            .filter(|_| content.panel.is_none())
        {
            let r = key.bounds.visible;
            if r.is_empty() {
                continue;
            }
            let r = crate::scene::key_cell(r, content.strip);
            let margin = key.styles.iter().map(|s| s.shadow).fold(0.0, f32::max) * 6.0;
            if r.right + margin < damage.left
                || r.left - margin > damage.right
                || r.bottom + margin < damage.top
                || r.top - margin > damage.bottom
            {
                continue;
            }
            let pressed = usize::from(content.pressed.contains(&i));
            let style = &key.styles[pressed];
            let hint = &key.hints[pressed];
            let r = crate::smartbar::inset(r, style.margin);
            if crate::shadow_raster::reaches(r, style.shadow, damage) {
                self.shadowed_box(r, style);
            }
            let telpad = content.telpad.get(i) == Some(&true);
            let bias = if telpad { 0.25 } else { 0.5 };
            let at = self.text_at(&key.label, style, r, telpad.then_some(false));
            let place = Place {
                bias: [bias, 0.5],
                outer: r,
            };
            self.contained_label(&key.label, style, at, inset(r, style.padding), place)?;
            if !key.icon.is_empty() {
                let path = vello_cpu::kurbo::BezPath::from_svg(&key.icon)
                    .map_err(|_| "invalid icon path".to_owned())?;
                self.context.set_transform(
                    Affine::translate((
                        (r.left + (r.width() - style.size) / 2.0) as f64,
                        (r.top + (r.height() - style.size) / 2.0) as f64,
                    )) * Affine::scale(style.size as f64 / 24.0),
                );
                self.context.set_paint(color(style.foreground));
                self.context.fill_path(&path);
                self.context.reset_transform();
            }
            self.hint(key, [style, hint], r, telpad)?;
        }
        if damage.top < content.strip && content.panel.as_ref().is_none_or(|p| p.kind == 2) {
            self.box_at(
                Rect {
                    left: 0.0,
                    top: content_top,
                    right: content.width as f32,
                    bottom: content.strip,
                },
                &content.strip_style,
            );
        }
        if damage.top < content.strip && content.panel.as_ref().is_none_or(|p| p.kind == 2) {
            self.smartbar(content)?;
        }
        // Colour glyph preparation is substantial work even when all of its pixels are clipped.
        // A key-only update must not rebuild the unchanged COLR/CBDT candidate strip.
        for (i, candidate) in content
            .candidates
            .iter()
            .enumerate()
            .filter(|_| damage.top < content.strip && content.smartbar.is_none())
        {
            let width = content.width as f32 / content.candidates.len() as f32;
            self.context.push_clip_rect(&vello_cpu::kurbo::Rect::new(
                (i as f32 * width) as f64,
                0.0,
                ((i + 1) as f32 * width) as f64,
                content.strip as f64,
            ));
            let default = crate::scene::CandidateStyle::default();
            let style = content
                .candidate_styles
                .get(i)
                .map(|s| &s[usize::from(content.candidate_pressed.contains(&i))])
                .unwrap_or(&default);
            let r = Rect {
                left: i as f32 * width,
                right: (i + 1) as f32 * width,
                top: 0.0,
                bottom: content.strip,
            };
            self.box_at(r, &style.container);
            self.label(
                candidate,
                style.text.size,
                style.text.foreground,
                r,
                style.text.font,
                style.text.weight,
            )?;
            self.context.pop_clip();
        }
        Ok(())
    }

    /// Where a key's text centres in its box `r` while it fits: the box, or on a phone pad's key,
    /// `Some(hint)`, its side of the key.
    fn text_at(&mut self, text: &str, s: &Style, r: Rect, phone: Option<bool>) -> Rect {
        let Some(hint) = phone else {
            return r;
        };
        let metrics = [
            self.text.measure(text, s.size, s.font, s.weight),
            self.text.height(text, s.size, s.font, s.weight),
        ];
        crate::key_content::phone_label(r, metrics, hint)
    }

    /// The key's hint in the top end of its box `r` (`styles` is the key's and the hint's), or
    /// on a phone pad's key, on its right; a hint that would reach out of the key fits inside it,
    /// at its top end as Compose aligns it.
    fn hint(
        &mut self,
        key: &Key,
        styles: [&Style; 2],
        r: Rect,
        telpad: bool,
    ) -> Result<(), String> {
        let [style, hint] = styles;
        if key.hint.is_empty() {
            return Ok(());
        }
        let at = if telpad {
            self.text_at(&key.hint, hint, r, Some(true))
        } else {
            Rect {
                left: r.right - style.size * 0.6,
                bottom: r.top + style.size * 0.7,
                ..r
            }
        };
        let bias = if telpad { [0.75, 0.5] } else { [1.0, 0.0] };
        let place = Place { bias, outer: r };
        self.contained_label(&key.hint, hint, at, r, place)
    }
}
