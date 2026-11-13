//! Scene snapshots contain the evaluated keys and resolved Snygg values, never another layout.
use crate::style::Style;
use keys::{Bounds, Rect};

#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub code: i32,
    pub bounds: Bounds,
    pub label: String,
    pub hint: String,
    pub icon: String,
    pub styles: [Style; 2],
    pub hints: [Style; 2],
    pub enabled: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CandidateStyle {
    pub container: Style,
    pub text: Style,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Popup {
    pub key: usize,
    pub labels: Vec<String>,
    pub selected: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub keyboard_height: f32,
    pub watermark: Option<crate::ime_decorations::Watermark>,
    pub preview: Option<crate::panels::Panel>,
    pub search: Option<crate::panels::Panel>,
    pub panel: Option<crate::panels::Panel>,
    pub width: u16,
    pub height: u16,
    pub strip: f32,
    pub background: u32,
    pub strip_style: Style,
    pub smartbar: Option<crate::smartbar_scene::Smartbar>,
    pub candidate_styles: Vec<[CandidateStyle; 2]>,
    pub candidate_pressed: Vec<usize>,
    pub popup_style: Style,
    pub popup_styles: Vec<[Style; 2]>,
    pub keys: Vec<Key>,
    pub telpad: Vec<bool>,
    pub candidates: Vec<String>,
    pub candidate_tokens: Vec<i32>,
    pub pressed: Vec<usize>,
    pub popup: Option<Popup>,
    pub popup_geometry: Option<crate::popup_scene::Geometry>,
    pub desired_key: [f32; 2],
    pub trail: Vec<crate::trail::Circle>,
    pub trail_color: u32,
}

impl Scene {
    pub fn candidate_at(&self, x: f32, y: f32) -> Option<usize> {
        if let Some(bar) = &self.smartbar {
            return bar
                .layout
                .hit(x, y)
                .and_then(|(action, id)| (!action).then_some(id));
        }
        if self.candidates.is_empty()
            || x < 0.0
            || x >= self.width as f32
            || y < 0.0
            || y >= self.strip
        {
            return None;
        }
        Some((x / self.width as f32 * self.candidates.len() as f32).floor() as usize)
    }

    pub fn key_at(&self, x: f32, y: f32) -> Option<usize> {
        if self.panel.is_some() || y < self.strip {
            return None;
        }
        let rects: Vec<_> = self.keys.iter().map(|k| k.bounds.touch).collect();
        keys::key_at(&[&rects], x, y - self.strip).filter(|&i| self.keys[i].enabled)
    }

    pub fn popup_bounds(&self) -> Option<Rect> {
        let popup = self.popup.as_ref()?;
        if let Some(grid) = &self.popup_geometry {
            let r = grid.layout.bounds;
            return Some(Rect {
                left: r.left,
                top: r.top + self.strip,
                right: r.right,
                bottom: r.bottom + self.strip,
            });
        }
        let key = self.keys.get(popup.key)?.bounds.visible;
        let width = (key.width() * popup.labels.len() as f32).min(self.width as f32);
        let left =
            (key.left - width / 2.0 + key.width() / 2.0).clamp(0.0, self.width as f32 - width);
        Some(Rect {
            left,
            right: left + width,
            top: (key.top + self.strip - key.height()).max(0.0),
            bottom: key.top + self.strip,
        })
    }

    /// Union of the regions whose pixels can change. A structural update invalidates everything.
    pub fn damage(&self, next: &Self) -> Option<Rect> {
        self.damage_regions(next).into_iter().reduce(union)
    }

    /// Separate strip/key/popup damage so a distant key does not repaint intervening rows.
    pub fn damage_regions(&self, next: &Self) -> Vec<Rect> {
        if self == next {
            return Vec::new();
        }
        // Compare stable fields directly, without cloning all labels and candidate strings.
        if self.watermark != next.watermark
            || self.panel != next.panel
            || self.width != next.width
            || self.height != next.height
            || self.keyboard_height != next.keyboard_height
            || self.strip != next.strip
            || self.background != next.background
            || self.strip_style != next.strip_style
            || self.popup_style != next.popup_style
            || self.popup_styles != next.popup_styles
            || self.keys != next.keys
        {
            return vec![Rect {
                left: 0.0,
                top: 0.0,
                right: next.width as f32,
                bottom: next.height as f32,
            }];
        }
        let mut rects = Vec::new();
        if self.trail != next.trail || self.trail_color != next.trail_color {
            rects.extend(crate::trail::bounds(&self.trail));
            rects.extend(crate::trail::bounds(&next.trail));
        }
        if self.preview != next.preview
            || self.search != next.search
            || self.candidates != next.candidates
            || self.candidate_pressed != next.candidate_pressed
            || self.smartbar != next.smartbar
            || self.candidate_styles != next.candidate_styles
        {
            rects.push(Rect {
                left: 0.0,
                top: 0.0,
                right: next.width as f32,
                bottom: next.strip,
            });
        }
        for &i in self.pressed.iter().chain(&next.pressed) {
            if self.pressed.contains(&i) == next.pressed.contains(&i) {
                continue;
            }
            if let Some(key) = next.keys.get(i) {
                let r = key.bounds.visible;
                let margin = key.styles.iter().map(|s| s.shadow).fold(0.0, f32::max) * 6.0;
                rects.push(Rect {
                    left: r.left - margin,
                    right: r.right + margin,
                    top: r.top + next.strip - margin,
                    bottom: r.bottom + next.strip + margin,
                });
            }
        }
        if self.popup != next.popup || self.popup_geometry != next.popup_geometry {
            rects.extend(self.popup_bounds());
            rects.extend(next.popup_bounds());
        }
        rects
    }
}

pub fn union(a: Rect, b: Rect) -> Rect {
    Rect {
        left: a.left.min(b.left),
        top: a.top.min(b.top),
        right: a.right.max(b.right),
        bottom: a.bottom.max(b.bottom),
    }
}
