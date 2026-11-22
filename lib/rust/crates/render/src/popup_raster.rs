//! A key's long-press popup, drawn over the keys and the strip. It is the one thing drawn above
//! the scene's top, into the headroom the drawing buffer keeps there
//! (`popup_scene::headroom`), so that a top-row key's popup shows whole over the app.
use crate::{raster::Raster, scene::Scene};

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
