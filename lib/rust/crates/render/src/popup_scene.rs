//! Reuse the popup crate's Compose grid for rendering, hit testing and flattened choice handles.
use crate::scene::Scene;
use keys::Rect;
use popup::grid::{self, Bounds, Extended};
#[derive(Clone, Debug, PartialEq)]
pub struct Geometry {
    pub key: usize,
    pub key_bounds: Bounds,
    pub base: Bounds,
    pub layout: Extended,
    pub order: Vec<usize>,
}
fn bounds(r: Rect) -> Bounds {
    Bounds {
        left: r.left,
        top: r.top,
        right: r.right,
        bottom: r.bottom,
    }
}
/// The preview box a key's popup grows from, as `TextKeyboardLayout` sized it from the desired
/// key: 1.1 by 2.5 of it in portrait, 1 by 3 in landscape, centred over the key and ending at its
/// bottom. None until the desired key has a size.
fn base(key: Bounds, [width, height]: [f32; 2], landscape: bool) -> Option<Bounds> {
    if width <= 0.0 || height <= 0.0 {
        return None;
    }
    let width = width * if landscape { 1.0 } else { 1.1 };
    let height = height * if landscape { 3.0 } else { 2.5 };
    let left = key.left + (key.width() - width) / 2.0;
    Some(Bounds {
        left,
        top: key.bottom - height,
        right: left + width,
        bottom: key.bottom,
    })
}

/// The preview box (`base`) for a key's `visible` bounds, as a scene rectangle: where a press
/// bubble draws before it is kept on screen (`Scene::bubble_bounds`).
pub(crate) fn preview(visible: Rect, desired: [f32; 2], landscape: bool) -> Option<Rect> {
    let b = base(bounds(visible), desired, landscape)?;
    Some(Rect {
        left: b.left,
        top: b.top,
        right: b.right,
        bottom: b.bottom,
    })
}

/// Six choices are the fewest that take two rows, the most a popup has: the tallest popup.
const TALLEST: i32 = 6;

/// The rows the drawing buffer keeps above the scene so that any key's popup, laid out for
/// `landscape`, is drawn whole: how far the tallest popup of the highest key reaches above the
/// scene's top. A popup is drawn above the key it belongs to, over the app, as Compose drew it.
/// None while a panel fills the screen (kind 3), which shows no keys, and none before the
/// desired key has a size.
pub fn headroom(scene: &Scene, landscape: bool) -> u16 {
    if scene.panel.as_ref().is_some_and(|p| p.kind == 3) {
        return 0;
    }
    let top = scene
        .keys
        .iter()
        .filter(|k| !k.bounds.visible.is_empty())
        .filter_map(|k| {
            let key = bounds(k.bounds.visible);
            let base = base(key, scene.desired_key, landscape)?;
            grid::extend(TALLEST, 0, key, base, scene.width as f32)
        })
        .map(|popup| popup.bounds.top + scene.strip)
        .filter(|top| top.is_finite())
        .fold(0.0, f32::min);
    (-top).ceil().clamp(0.0, 8192.0) as u16
}

pub fn configure(scene: &mut Scene, prioritized: i32, landscape: bool) -> Option<Vec<usize>> {
    let popup = scene.popup.as_mut()?;
    let key = bounds(scene.keys.get(popup.key)?.bounds.visible);
    if !(0..=3).contains(&prioritized) || prioritized as usize > popup.labels.len() {
        return None;
    }
    let base = base(key, scene.desired_key, landscape)?;
    let layout = grid::extend(
        popup.labels.len() as i32,
        prioritized,
        key,
        base,
        scene.width as f32,
    )?;
    let order: Vec<_> = layout
        .popup_indices
        .iter()
        .map(|&i| {
            if i < 0 {
                (-i - 1) as usize
            } else {
                (prioritized + i) as usize
            }
        })
        .collect();
    if order.iter().any(|&i| i >= popup.labels.len()) {
        return None;
    }
    popup.labels = order.iter().map(|&i| popup.labels[i].clone()).collect();
    popup.selected = layout.initial_index as usize;
    if !scene.popup_styles.is_empty() {
        scene.popup_styles = order
            .iter()
            .map(|&i| scene.popup_styles[i].clone())
            .collect();
    }
    scene.popup_geometry = Some(Geometry {
        key: popup.key,
        key_bounds: key,
        base,
        layout,
        order: order.clone(),
    });
    Some(order)
}
impl Scene {
    pub fn popup_choice_index(&self, selected: usize) -> usize {
        self.popup_geometry
            .as_ref()
            .and_then(|g| g.order.get(selected))
            .copied()
            .unwrap_or(selected)
    }
    pub fn popup_choice_bounds(&self, index: usize) -> Option<Rect> {
        let popup = self.popup.as_ref()?;
        let bounds = self.popup_bounds()?;
        if let Some(g) = &self.popup_geometry {
            let row1 = g.layout.grid.row1_count as usize;
            let top = index < row1;
            let column = if top { index } else { index - row1 };
            let pad = if top && !g.layout.grid.anchor_left {
                (g.layout.grid.row0_count - g.layout.grid.row1_count) as f32 * g.base.width()
            } else {
                0.0
            };
            let left = bounds.left + pad + column as f32 * g.base.width();
            let height = g.base.height() * 0.4;
            let y = if top {
                bounds.top
            } else {
                bounds.bottom - height
            };
            Some(Rect {
                left,
                top: y,
                right: left + g.base.width(),
                bottom: y + height,
            })
        } else {
            let width = bounds.width() / popup.labels.len() as f32;
            Some(Rect {
                left: bounds.left + index as f32 * width,
                right: bounds.left + (index + 1) as f32 * width,
                ..bounds
            })
        }
    }
    /// None means the finger left the popup; the controller cancels and hit-tests a new key.
    pub fn popup_pointer(&self, x: f32, y: f32) -> Option<usize> {
        if let Some(g) = &self.popup_geometry {
            grid::pointer(g.key_bounds, g.base, g.layout.grid, x, y - self.strip)
                .map(|i| i as usize)
        } else {
            let p = self.popup.as_ref()?;
            let r = self.popup_bounds()?;
            Some(
                (((x - r.left) / r.width() * p.labels.len() as f32)
                    .floor()
                    .max(0.0) as usize)
                    .min(p.labels.len().saturating_sub(1)),
            )
        }
    }
}
