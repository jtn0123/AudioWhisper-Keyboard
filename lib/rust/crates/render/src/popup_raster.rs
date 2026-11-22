//! A key's long-press popup, and the bubble over a pressed key, drawn over the keys and the strip.
//! They are the only things drawn above the scene's top, into the headroom the drawing buffer
//! keeps there (`popup_scene::headroom`), so that a top-row key's popup or bubble shows whole
//! over the app.
use crate::{raster::Raster, scene::Scene, smartbar::inset};
use keys::Rect;

impl Raster {
    /// The open popup's box, then each choice in its own style, the selected one marked.
    pub(crate) fn popup(&mut self, content: &Scene) -> Result<(), String> {
        let (Some(popup), Some(bounds)) = (&content.popup, content.popup_bounds()) else {
            return Ok(());
        };
        self.box_at(bounds, &content.popup_style);
        for (i, label) in popup.labels.iter().enumerate() {
            let r = content
                .popup_choice_bounds(i)
                .ok_or("missing popup choice bounds")?;
            let style = content
                .popup_styles
                .get(i)
                .map(|s| &s[usize::from(i == popup.selected)])
                .unwrap_or(&content.popup_style);
            self.box_at(r, style);
            self.label(
                label,
                style.size,
                style.foreground,
                r,
                style.mono,
                style.weight,
            )?;
        }
        Ok(())
    }
}

/// Material's "more horizontal" icon on its 24-unit grid: the dots at a bubble's end that say
/// the key has long-press choices, as Compose's `PopupBaseBox` draws `Icons.Default.MoreHoriz`.
const MORE: &str = "M6 10c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm12 0c-1.1 0-2 .9-2 2s.9 2 2 \
                    2 2-.9 2-2-.9-2-2-2zm-6 0c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z";

impl Raster {
    /// The pressed key's bubble over its shadow, laid out as Compose's `PopupBaseBox`: the key's
    /// label centred in a box the key's height at the top, and the dots centred at the end where
    /// the key has long-press choices.
    pub(crate) fn bubble(&mut self, content: &Scene) -> Result<(), String> {
        let Some((bubble, style, bounds)) = content.bubble_view() else {
            return Ok(());
        };
        let key = &content.keys[bubble.key];
        let r = inset(bounds, style.margin);
        self.shadowed_box(r, style);
        let inner = inset(r, style.padding);
        let label = Rect {
            bottom: (inner.top + key.bounds.visible.height().round()).min(inner.bottom),
            ..inner
        };
        let font = (style.size, style.mono, style.weight);
        self.squeezed_label(&key.label, font, 1.0, style.foreground, label)?;
        if let Some(dots) = &bubble.indicator {
            let middle = (inner.top + inner.bottom) / 2.0;
            let half = dots.size / 2.0;
            let at = Rect {
                left: inner.right - dots.size,
                top: middle - half,
                right: inner.right,
                bottom: middle + half,
            };
            self.icon(MORE, dots, at)?;
        }
        Ok(())
    }
}
