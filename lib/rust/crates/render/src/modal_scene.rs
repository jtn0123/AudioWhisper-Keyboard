//! A subtype sheet occludes input/semantics but retains a dimmed, bottom-anchored keyboard.
use crate::scene::Scene;
pub fn offset(height: f32, strip: f32, keyboard: f32) -> f32 {
    (height - strip - keyboard).max(0.0)
}
pub fn content(scene: &Scene) -> Option<Scene> {
    if !scene.panel.as_ref().is_some_and(|p| p.kind == 7) {
        return None;
    }
    let mut content = scene.clone();
    let y = offset(scene.height as f32, scene.strip, scene.keyboard_height);
    content.panel = None;
    content.strip += y;
    content.popup = None;
    content.popup_geometry = None;
    if let Some(bar) = &mut content.smartbar {
        for slot in bar
            .layout
            .actions
            .iter_mut()
            .chain(bar.layout.candidates.iter_mut())
        {
            slot.bounds.top += y;
            slot.bounds.bottom += y;
        }
        bar.layout.center.top += y;
        bar.layout.center.bottom += y;
    }
    Some(content)
}
