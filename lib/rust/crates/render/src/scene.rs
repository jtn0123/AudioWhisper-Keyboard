//! Scene snapshots contain the evaluated keys and resolved Snygg values, never another layout.
use keys::{Bounds, Rect};

#[derive(Clone, Debug, PartialEq)]
pub struct Key {
    pub code: i32,
    pub bounds: Bounds,
    pub label: String,
    pub hint: String,
    pub icon: String,
    pub size: f32,
    pub radius: f32,
    pub shadow: f32,
    pub hint_size: f32,
    pub hint_colors: [u32; 2],
    pub colors: [u32; 4], // normal background/foreground, pressed background/foreground
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Popup {
    pub key: usize,
    pub labels: Vec<String>,
    pub selected: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scene {
    pub width: u16,
    pub height: u16,
    pub strip: f32,
    pub background: u32,
    pub candidate_color: u32,
    pub candidate_size: f32,
    pub popup_colors: [u32; 3],
    pub keys: Vec<Key>,
    pub candidates: Vec<String>,
    pub pressed: Vec<usize>,
    pub popup: Option<Popup>,
}

impl Scene {
    pub fn candidate_at(&self, x: f32, y: f32) -> Option<usize> {
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
        if y < self.strip {
            return None;
        }
        let rects: Vec<_> = self.keys.iter().map(|k| k.bounds.touch).collect();
        keys::key_at(&[&rects], x, y - self.strip).filter(|&i| self.keys[i].enabled)
    }

    pub fn popup_bounds(&self) -> Option<Rect> {
        let popup = self.popup.as_ref()?;
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
        if self == next {
            return None;
        }
        let mut a = self.clone();
        let mut b = next.clone();
        a.pressed.clear();
        b.pressed.clear();
        a.popup = None;
        b.popup = None;
        a.candidates.clear();
        b.candidates.clear();
        if a != b {
            return Some(Rect {
                left: 0.0,
                top: 0.0,
                right: next.width as f32,
                bottom: next.height as f32,
            });
        }
        let mut rects = Vec::new();
        if self.candidates != next.candidates {
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
                let margin = key.shadow * 4.0;
                rects.push(Rect {
                    left: r.left - margin,
                    right: r.right + margin,
                    top: r.top + next.strip - margin,
                    bottom: r.bottom + next.strip + margin,
                });
            }
        }
        if self.popup != next.popup {
            rects.extend(self.popup_bounds());
            rects.extend(next.popup_bounds());
        }
        rects.into_iter().reduce(union)
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
