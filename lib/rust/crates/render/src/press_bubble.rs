//! The bubble over a pressed key, which shows the character it types: Compose's preview popup
//! (`PopupUiController.show`, drawn by `PopupBaseBox`). It is drawn in the drawing buffer's
//! headroom, as the long-press popup is (`popup_scene::headroom`), so that a top-row key's bubble
//! shows whole over the app. A long-press popup takes its place.
use crate::{scene::Scene, style::Style};
use keys::{
    Rect,
    codes::{CJK_SPACE, SPACE},
};

/// A pressed key's bubble. The touch opens it with the key alone; the native session styles it
/// as that key's long-press popup box before the frame that draws it.
#[derive(Clone, Debug, PartialEq)]
pub struct Bubble {
    pub key: usize,
    /// `key-popup-box` for the key. Nothing draws until it is styled.
    pub style: Option<Style>,
    /// `key-popup-extended-indicator`, where the key has long-press choices: the dots Compose's
    /// bubble shows at its end.
    pub indicator: Option<Style>,
}

impl Bubble {
    pub fn new(key: usize) -> Self {
        Self {
            key,
            style: None,
            indicator: None,
        }
    }
}

/// Whether a key shows a bubble, as Compose's `isSuitableForBasicPopup` decides: a character
/// (code above space, not the CJK space), on no number or phone pad. `mode` and `kind` are the
/// keyboard mode's and the key type's lower-case names.
pub fn suitable(code: i32, mode: &str, kind: &str) -> bool {
    let numeric = matches!(mode, "numeric" | "phone" | "phone2")
        || (mode == "numeric_advanced" && kind == "numeric");
    code > SPACE && code != CJK_SPACE && !numeric
}

/// Marks which of the scene's keys show a bubble, in keyboard `mode`, each key of the type in
/// `types` at its index.
pub fn mark(scene: &mut Scene, mode: &str, types: &[String]) {
    scene.bubbles = scene
        .keys
        .iter()
        .enumerate()
        .map(|(i, key)| suitable(key.code, mode, types.get(i).map_or("", String::as_str)))
        .collect();
}

impl Scene {
    /// Where key `key`'s bubble draws, in the scene's coordinates: the preview box Compose grew
    /// the long-press popup from (`popup_scene::preview`), moved sideways no further than it
    /// takes to stay inside the scene, its corner truncated and its size rounded as Compose
    /// places a box. None before the desired key has a size.
    pub fn bubble_bounds(&self, key: usize) -> Option<Rect> {
        let visible = self.keys.get(key)?.bounds.visible;
        let preview = crate::popup_scene::preview(visible, self.desired_key, self.landscape)?;
        let room = (f32::from(self.width) - preview.width()).max(0.0);
        let left = preview.left.clamp(0.0, room);
        let placed = Rect {
            left,
            right: left + preview.width(),
            ..preview
        };
        Some(crate::scene::key_cell(placed, self.strip))
    }

    /// The bubble the frame draws, its style and its bounds. None while a long-press popup is
    /// open, which takes its place, and none for a key that shows no bubble.
    pub fn bubble_view(&self) -> Option<(&Bubble, &Style, Rect)> {
        let bubble = self.bubble.as_ref().filter(|_| self.popup.is_none())?;
        let style = bubble.style.as_ref()?;
        if self.bubbles.get(bubble.key) != Some(&true) {
            return None;
        }
        Some((bubble, style, self.bubble_bounds(bubble.key)?))
    }

    /// What drawing or taking away the bubble repaints: its box, and as far as its shadow paints.
    pub fn bubble_damage(&self) -> Option<Rect> {
        let (_, style, r) = self.bubble_view()?;
        Some(crate::shadow_raster::extent(r, style.shadow))
    }
}
