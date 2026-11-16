//! Compose Smartbar/MainRow, QuickActionsRow and CandidatesRow measurement rules.
//! Android supplies action identities and font advances; policy and hit regions live here.
use keys::Rect;

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub enabled: bool,
    pub layout: i32,
    pub flip: bool,
    pub shared: bool,
    pub extended: bool,
    pub placement: i32,
    pub speaker: bool,
    pub sticky: bool,
    pub dynamic: usize,
    pub mode: i32,
    pub density: f32,
    pub row: f32,
    pub inline: bool,
    pub preview: bool,
    /// The main row shows words, or holds their place across a keystroke's refill (Rust NLP).
    pub words: bool,
    /// Dictation is listening or finishing.
    pub dictating: bool,
    /// Read-aloud is speaking.
    pub reading: bool,
}
impl Config {
    pub fn decode(a: &[f32]) -> Option<Self> {
        if ![12, 14, 17].contains(&a.len())
            || a.iter().any(|v| !v.is_finite())
            || a[10] <= 0.0
            || a[11] <= 0.0
            || !(0.0..=3.0).contains(&a[1])
            || !(0.0..=2.0).contains(&a[5])
            || !(0.0..=2.0).contains(&a[9])
            || !(0.0..=512.0).contains(&a[8])
        {
            return None;
        }
        Some(Self {
            enabled: a[0] != 0.0,
            layout: a[1] as i32,
            flip: a[2] != 0.0,
            shared: a[3] != 0.0 && !a.get(13).is_some_and(|v| *v != 0.0),
            extended: a[4] != 0.0,
            placement: a[5] as i32,
            speaker: a[6] != 0.0,
            sticky: a[7] != 0.0,
            dynamic: a[8] as usize,
            mode: a[9] as i32,
            density: a[10],
            row: a[11].round(),
            inline: a.get(12).is_some_and(|v| *v != 0.0),
            preview: a.get(13).is_some_and(|v| *v != 0.0),
            words: a.get(14).is_some_and(|v| *v != 0.0),
            dictating: a.get(15).is_some_and(|v| *v != 0.0),
            reading: a.get(16).is_some_and(|v| *v != 0.0),
        })
    }
    pub fn extra(&self) -> bool {
        self.enabled && self.layout == 3 && self.extended
    }
    /// Compose's inline views replace candidates, or the actions-only main row.
    /// An explicitly expanded shared row still takes precedence over inline suggestions.
    pub fn inline_visible(&self) -> bool {
        self.enabled && self.inline && !self.preview && !(self.layout == 2 && self.shared)
    }
    /// Words fill the main row: while suggestions show in a shared or extended layout, the
    /// pinned toggle, sticky action, speaker and emoji give way so long words fit. The row stays
    /// as it was while dictating, and while reading aloud with the speaker as the way to stop.
    pub fn roomy(&self) -> bool {
        self.enabled
            && self.layout >= 2
            && self.words
            && !(self.layout == 2 && self.shared)
            && !self.inline_visible()
            && !self.preview
            && !self.dictating
            && !(self.reading && self.speaker)
    }
    pub fn height(&self) -> f32 {
        if self.enabled {
            self.row * if self.extra() { 2.0 } else { 1.0 }
        } else {
            0.0
        }
    }
    pub fn inset(&self) -> f32 {
        self.height()
            - if self.extra() && self.placement == 2 {
                self.row
            } else {
                0.0
            }
    }
}

/// Host action ids: sticky=0, speaker=1, emoji=2, overflow=3, toggle=4, dynamic=5+.
#[derive(Clone, Debug, PartialEq)]
pub struct Slot {
    pub id: usize,
    pub bounds: Rect,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Layout {
    pub actions: Vec<Slot>,
    pub candidates: Vec<Slot>,
    pub center: Rect,
    pub visible_dynamic: usize,
    pub content_width: f32,
}
pub fn inset(r: Rect, a: [f32; 4]) -> Rect {
    Rect {
        left: (r.left + a[0]).min(r.right),
        top: (r.top + a[1]).min(r.bottom),
        right: (r.right - a[2]).max(r.left + a[0]).min(r.right),
        bottom: (r.bottom - a[3]).max(r.top + a[1]).min(r.bottom),
    }
}
impl Layout {
    pub fn hit(&self, x: f32, y: f32) -> Option<(bool, usize)> {
        let inside = |r: Rect| x >= r.left && x < r.right && y >= r.top && y < r.bottom;
        self.actions
            .iter()
            .find(|s| inside(s.bounds))
            .map(|s| (true, s.id))
            .or_else(|| {
                self.candidates
                    .iter()
                    .find(|s| inside(s.bounds) && inside(self.center))
                    .map(|s| (false, s.id))
            })
    }
    pub fn packed(&self, c: &Config) -> Vec<f32> {
        let mut a = vec![
            c.height(),
            c.inset(),
            self.visible_dynamic as f32,
            self.center.left,
            self.center.top,
            self.center.right,
            self.center.bottom,
            self.content_width,
        ];
        for (kind, slots) in [(1.0, &self.actions), (0.0, &self.candidates)] {
            for s in slots {
                a.extend([
                    kind,
                    s.id as f32,
                    s.bounds.left,
                    s.bounds.top,
                    s.bounds.right,
                    s.bounds.bottom,
                ]);
            }
        }
        if c.inline_visible() {
            a.extend([
                2.0,
                0.0,
                self.center.left,
                self.center.top,
                self.center.right,
                self.center.bottom,
            ]);
        }
        a
    }
}
pub fn layout(c: &Config, width: f32, intrinsic: &[f32], scroll: f32) -> Layout {
    let mut out = Layout::default();
    if !c.enabled {
        return out;
    }
    let h = c.row;
    let y = if c.extra() && c.placement != 1 {
        h
    } else {
        0.0
    };
    out.center = Rect {
        left: 0.0,
        top: y,
        right: width,
        bottom: y + h,
    };
    if c.layout >= 2 && !c.roomy() {
        let edge = (4.0 * c.density).round();
        let pinned = h + 2.0 * edge;
        let speaker = if c.speaker { pinned } else { 0.0 };
        let mut add = |id, left, w| {
            out.actions.push(Slot {
                id,
                bounds: Rect {
                    left,
                    top: y,
                    right: left + w,
                    bottom: y + h,
                },
            })
        };
        if c.flip {
            if c.sticky || (c.layout == 2 && c.shared) {
                add(if c.sticky { 0 } else { 3 }, edge, h);
            }
            if c.speaker {
                add(1, pinned + edge, h);
            }
            add(2, width - h - pinned + edge, h);
            add(4, width - h, h);
            out.center.left = pinned + speaker;
            out.center.right = (width - h - pinned).max(out.center.left);
        } else {
            add(4, 0.0, h);
            let right = width - pinned - speaker;
            add(2, right - pinned + edge, h);
            if c.speaker {
                add(1, right + edge, h);
            }
            if c.sticky || (c.layout == 2 && c.shared) {
                add(if c.sticky { 0 } else { 3 }, width - pinned + edge, h);
            }
            out.center.left = h;
            out.center.right = (right - pinned).max(h);
        }
    }
    if c.inline_visible() {
        // Native drawing, hit testing and semantics leave the platform region empty.
    } else if c.layout == 1 || (c.layout == 2 && c.shared) {
        actions(c, out.center, &mut out);
    } else {
        let area = out.center;
        let count = if c.mode == 0 {
            intrinsic.len().min(3)
        } else {
            intrinsic.len()
        };
        let gap = c.density.round();
        let available = (area.width() - gap * count.saturating_sub(1) as f32).max(0.0);
        let unit = if count == 0 {
            0.0
        } else {
            (available / count as f32).round()
        };
        let mut remainder = available.round() as i32 - unit as i32 * count as i32;
        let mut row_remaining = area.width();
        let widths: Vec<_> = intrinsic
            .iter()
            .take(count)
            .map(|w| {
                if count == 1 {
                    if c.mode == 0 {
                        available
                    } else {
                        w.min(available)
                    }
                } else if c.mode == 0 {
                    {
                        let correction = remainder.signum();
                        remainder -= correction;
                        unit + correction as f32
                    }
                } else {
                    let width = w.min(160.0 * c.density).min(if c.mode == 1 {
                        row_remaining
                    } else {
                        f32::INFINITY
                    });
                    row_remaining = (row_remaining - width - gap).max(0.0);
                    width
                }
            })
            .collect();
        out.content_width = widths.iter().sum::<f32>() + gap * count.saturating_sub(1) as f32;
        if c.mode == 1 && count > 1 {
            out.content_width = out.content_width.min(area.width());
        }
        let scroll = if c.mode == 2 {
            scroll.clamp(0.0, (out.content_width - area.width()).max(0.0))
        } else {
            0.0
        };
        let mut left = area.left
            + if count == 1 {
                (available - widths[0]) / 2.0
            } else {
                -scroll
            };
        for (id, w) in widths.into_iter().enumerate() {
            out.candidates.push(Slot {
                id,
                bounds: Rect {
                    left,
                    right: left + w,
                    ..area
                },
            });
            left += w + gap;
            if c.mode == 1 {
                left = left.min(area.right);
            }
        }
    }
    if c.extra() {
        let top = if c.placement == 1 { h } else { 0.0 };
        actions(
            c,
            Rect {
                left: 0.0,
                top,
                right: width,
                bottom: top + h,
            },
            &mut out,
        );
    }
    out
}
fn actions(c: &Config, r: Rect, out: &mut Layout) {
    let overflow = c.sticky || c.layout != 2 || !c.shared;
    let capacity = ((r.width() / c.row) as usize).saturating_sub(usize::from(overflow));
    let sticky = c.layout == 1 && c.sticky;
    out.visible_dynamic = capacity.saturating_sub(usize::from(sticky));
    let mut ids: Vec<_> = (5..5 + c.dynamic).collect();
    if sticky {
        ids.insert(0, 0);
    }
    ids.truncate(capacity);
    if overflow {
        if c.flip {
            ids.insert(0, 3)
        } else {
            ids.push(3)
        }
    }
    let gap = (r.width() - c.row * ids.len() as f32).max(0.0) / (ids.len() + 1) as f32;
    for (i, id) in ids.into_iter().enumerate() {
        let left = (r.left + gap + (gap + c.row) * i as f32).round();
        out.actions.push(Slot {
            id,
            bounds: Rect {
                left,
                right: left + c.row,
                ..r
            },
        });
    }
}

/// Largest complete prefix which fits with the ellipsis. Advances include the ellipsis.
pub fn ellipsis(advances: &[f32], available: f32) -> usize {
    fit_prefix(advances.len(), available, |i| advances[i])
}
pub fn fit_prefix(count: usize, available: f32, mut measure: impl FnMut(usize) -> f32) -> usize {
    let mut low = 0;
    let mut high = count;
    while low < high {
        let mid = (low + high) / 2;
        if measure(mid) <= available {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    low
}
