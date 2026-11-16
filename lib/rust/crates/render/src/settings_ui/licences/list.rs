//! The LazyColumn of TraditionalRows: heights come from line metrics and badge widths alone, so
//! the whole list is measured without shaping a name; only rows on screen are laid out.
//!
//! A row is a Column (fillMaxWidth, clickable, 20 dp by 14 dp padding, 4 dp apart) of the name
//! with its version, the author, and a FlowRow of license badges (6 dp apart both ways).
use super::{
    Geometry, ROW_ID,
    data::{Libraries, Library},
    style::{AUTHOR, BADGE, NAME, VERSION},
};
use crate::{
    panels::{Panel, rect},
    settings_ui::{Action, App, Control},
    text::Text,
};
use keys::Rect;

/// Kotlin's `isBlank`: empty or only whitespace.
fn blank(s: &str) -> bool {
    s.chars().all(char::is_whitespace)
}
pub(super) fn version(library: &Library) -> Option<&str> {
    library.version.as_deref().filter(|v| !blank(v))
}
pub(super) fn author(library: &Library) -> Option<&str> {
    Some(library.author.as_str()).filter(|a| !blank(a))
}
impl App {
    fn content_width(&self) -> f32 {
        self.config[0] - 2.0 * self.px(20.0)
    }
    /// A badge's pill: its text extent padded 10 dp by 3 dp, at most the row's width.
    fn pill(&self, text: &mut Text, name: &str) -> [f32; 2] {
        let [w, h] = self.extent(text, name, BADGE);
        [
            (w + 2.0 * self.px(10.0)).min(self.content_width()),
            h + 2.0 * self.px(3.0),
        ]
    }
    /// FlowRow placement of a row's badges: each pill's offset from the flow's top-left, and
    /// the flow's height.
    fn badges(&self, text: &mut Text, data: &Libraries, library: &Library) -> (Vec<Rect>, f32) {
        let (gap, max) = (self.px(6.0), self.content_width());
        let (mut x, mut y, mut line) = (0.0f32, 0.0f32, 0.0f32);
        let mut out = vec![];
        for &l in &library.licences {
            let [w, h] = self.pill(text, &data.licences[l].name);
            if x > 0.0 && x + w > max {
                y += line + gap;
                x = 0.0;
                line = 0.0;
            }
            out.push(rect(x, y, w, h));
            x += w + gap;
            line = line.max(h);
        }
        (out, y + line)
    }
    fn row_height(&self, text: &mut Text, data: &Libraries, library: &Library) -> f32 {
        let gap = self.px(4.0);
        let first = self.extent(text, "Ag", NAME)[1].max(if version(library).is_some() {
            self.extent(text, "0", VERSION)[1]
        } else {
            0.0
        });
        let mut h = 2.0 * self.px(14.0) + first;
        if author(library).is_some() {
            h += gap + self.extent(text, "Ag", AUTHOR)[1];
        }
        if !library.licences.is_empty() {
            h += gap + self.badges(text, data, library).1;
        }
        h
    }
    /// Every row's height and top at the current configuration, kept until it changes. A
    /// change keeps the first visible row and its offset, as LazyListState does.
    pub(super) fn licence_geometry(&mut self, text: &mut Text, data: &Libraries) -> Geometry {
        let key = [
            self.config[0].to_bits(),
            self.config[2].to_bits(),
            self.sp(14).to_bits(),
            data.libraries.len() as u32,
        ];
        if self.licences.geometry.key == key && !self.licences.geometry.tops.is_empty() {
            return self.licences.geometry.clone();
        }
        let old = std::mem::take(&mut self.licences.geometry);
        let mut g = Geometry {
            key,
            ..Default::default()
        };
        let mut top = 0.0;
        for library in &data.libraries {
            let h = self.row_height(text, data, library);
            g.tops.push(top);
            g.heights.push(h);
            top += h;
        }
        if !old.tops.is_empty() && self.on_licences() {
            let (index, offset) = old.at(self.screen_scroll());
            if let (Some(t), Some(s)) = (g.tops.get(index), self.scrolls.last_mut()) {
                *s = t + offset;
            }
        }
        self.licences.geometry = g.clone();
        g
    }
    /// Lays out and paints the rows inside `clip`; returns the list's whole height.
    pub(in crate::settings_ui) fn libraries(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(data) = self.licences.data.clone() else {
            if !self.licences.requested {
                self.licences.requested = true;
                self.effects
                    .push(vec!["load".into(), super::LIBRARIES.into()]);
            }
            self.licences.first = None;
            return 0.0;
        };
        let g = self.licence_geometry(text, &data);
        let (start, _) = g.at((clip.top - y).max(0.0));
        let mut shown = 0;
        self.licences.first = None;
        for j in start..data.libraries.len() {
            let top = y + g.tops[j];
            if top >= clip.bottom {
                break;
            }
            if top + g.heights[j] <= clip.top {
                continue;
            }
            if self.licences.first.is_none() {
                self.licences.first = Some((j, top - clip.top, g.heights[j]));
            }
            shown += 1;
            self.library_row(
                p,
                &data,
                j,
                rect(x, top, self.config[0], g.heights[j]),
                clip,
                text,
            );
        }
        if shown > 0 && self.licences.visible_items.is_none() {
            self.licences.visible_items = Some(shown);
        }
        if self.reveal_focus
            && let Some(j) = self.focus.checked_sub(ROW_ID).map(|j| j as usize)
            && j < data.libraries.len()
        {
            let top = y + g.tops[j];
            let bottom = top + g.heights[j];
            if top < clip.top {
                self.reveal_target = Some(self.shown_scroll() + top - clip.top);
            } else if bottom > clip.bottom {
                self.reveal_target = Some(self.shown_scroll() + bottom - clip.bottom);
            }
        }
        g.total()
    }
    fn library_row(
        &mut self,
        p: &mut Panel,
        data: &Libraries,
        j: usize,
        b: Rect,
        clip: Rect,
        text: &mut Text,
    ) {
        let library = &data.libraries[j];
        let id = ROW_ID + j as i32;
        if id == self.focus {
            self.paint_clipped_box(p, b, self.colors[4], 8.0 * self.config[2], clip);
        }
        let (left, width, gap) = (b.left + self.px(20.0), self.content_width(), self.px(4.0));
        let mut top = b.top + self.px(14.0);
        let mut line = self.extent(text, "Ag", NAME)[1];
        let mut name_width = width;
        let mut said = vec![];
        if let Some(v) = version(library) {
            let [vw, vh] = self.extent(text, v, VERSION);
            name_width = (width - self.px(8.0) - vw).max(0.0);
            let at = (left + width - vw, top, vw + 1.0, vh);
            self.paint_style(p, v, at, VERSION, self.colors[2], clip);
            line = line.max(vh);
            said.push(v.to_owned());
        }
        let name = self.one_line(text, &library.name, NAME, name_width);
        let at = (left, top, name_width.max(1.0), line);
        self.paint_style(p, &name, at, NAME, self.colors[15], clip);
        top += line;
        if let Some(a) = author(library) {
            let h = self.extent(text, "Ag", AUTHOR)[1];
            let shown = self.one_line(text, a, AUTHOR, width);
            let at = (left, top + gap, width, h);
            self.paint_style(p, &shown, at, AUTHOR, self.colors[2], clip);
            top += gap + h;
            said.push(a.to_owned());
        }
        if !library.licences.is_empty() {
            let (pills, _) = self.badges(text, data, library);
            for (&l, pill) in library.licences.iter().zip(pills) {
                let pill = rect(
                    left + pill.left,
                    top + gap + pill.top,
                    pill.width(),
                    pill.height(),
                );
                self.paint_clipped_box(p, pill, self.colors[4], pill.height() / 2.0, clip);
                let licence = &data.licences[l].name;
                let inner = pill.width() - 2.0 * self.px(10.0);
                let shown = self.one_line(text, licence, BADGE, inner);
                let at = (
                    pill.left + self.px(10.0),
                    pill.top + self.px(3.0),
                    inner.max(1.0) + 1.0,
                    pill.height(),
                );
                self.paint_style(p, &shown, at, BADGE, self.colors[14], clip);
                said.push(licence.clone());
            }
        }
        self.push_control(
            Control {
                id,
                bounds: b,
                label: library.name.clone(),
                summary: said.join(", "),
                role: "card",
                enabled: true,
                checked: None,
                selected: false,
                action: Action::Library(j),
                range: None,
            },
            clip,
        );
    }
    /// florisScrollbar over a LazyListState: `visibleItems` rows tall, each row a
    /// `viewport / totalItems` share, placed at the first row's index plus its scrolled part.
    pub(in crate::settings_ui) fn licence_scrollbar(&self, clip: Rect) -> Option<Rect> {
        let total = self.licences.data.as_ref()?.libraries.len();
        let (index, offset, size) = self.licences.first?;
        let visible = self.licences.visible_items? as f32;
        let element = clip.height() / total as f32;
        let width = 4.0 * self.config[2];
        let fraction = if size > 0.0 { offset / size } else { 0.0 };
        Some(rect(
            clip.right - width,
            clip.top + (index as f32 - fraction) * element,
            width,
            visible * element,
        ))
    }
}
