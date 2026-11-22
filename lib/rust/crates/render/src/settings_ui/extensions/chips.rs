//! The page's rows of chips, as ExtensionMetaRowScrollableChips held them: the maintainers, and
//! the keywords. Compose scrolled each row sideways; here the chips wrap onto more lines, each
//! line ending where the single line did. Keywords were disabled chips in faded ink; here they
//! are in full ink and read as text. A maintainer chip tells TalkBack what its dialog offers, and
//! one with nothing to offer reads as text rather than a disabled button.
use super::{
    Extension, Tap,
    page::{KEYWORD_ID, LABEL_ID, MAINTAINER_ID},
};
use crate::settings_ui::{
    Action, App,
    cards::{BODY_LARGE, LABEL_LARGE, plain},
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// A chip in a row of chips: its label, what TalkBack hears after it, its trailing icons, and
/// its tap if it has one.
pub(super) struct Chip {
    id: i32,
    label: String,
    summary: String,
    icons: Vec<&'static str>,
    tap: Option<Tap>,
}

impl App {
    /// The maintainers' chips: the name, and a mail and a link icon for what it offers.
    pub(super) fn maintainer_chips(&self, ext: &Extension) -> Vec<Chip> {
        let mut out = vec![];
        for (m, maintainer) in ext.maintainers.iter().enumerate() {
            let mut icons = vec![];
            let mut offers = vec![];
            if maintainer.email.is_some() {
                icons.push("mail");
                offers.push(self.label("ext__meta__email"));
            }
            if maintainer.url.is_some() {
                icons.push("link");
                offers.push(self.label("ext__meta__website"));
            }
            out.push(Chip {
                id: MAINTAINER_ID + m as i32,
                label: maintainer.name.clone(),
                summary: offers.join(", "),
                tap: (!icons.is_empty()).then_some(Tap::Maintainer(m)),
                icons,
            });
        }
        out
    }
    /// Row `k`: its label, and `chips` flowing from the end in lines 48 dp tall, 8 dp apart, in
    /// the room 24 dp after the label; its height.
    pub(super) fn chip_row(
        &mut self,
        p: &mut Panel,
        k: usize,
        chips: Vec<Chip>,
        ((left, width), y): ((f32, f32), f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let (label, lw, lh) = self.meta_label(text, k, width);
        let room = (width - lw - self.px(24.0)).max(1.0);
        let widths: Vec<f32> = chips
            .iter()
            .map(|c| self.chip_size(text, c).min(room))
            .collect();
        let lines = flow(&widths, room, self.px(8.0));
        let h = lh.max(self.px(48.0) * lines.len() as f32);
        let at = rect(left, y + ((h - lh) / 2.0).round(), lw, lh);
        self.paint_styled(p, &label, at, BODY_LARGE, self.colors[15], clip);
        self.push_control(plain(LABEL_ID + k as i32, at, label, "text"), clip);
        let top = y + ((h - self.px(48.0) * lines.len() as f32) / 2.0).round();
        let mut chips = chips.into_iter();
        for (n, (count, line_width)) in lines.into_iter().enumerate() {
            let mut cx = left + width - line_width;
            for c in chips.by_ref().take(count) {
                let w = self.chip_size(text, &c).min(room);
                self.ext_chip(p, c, (cx, top + self.px(48.0) * n as f32, w), clip, text);
                cx += w + self.px(8.0);
            }
        }
        h
    }
    /// How wide FlorisChip is: 8 dp padding and an 8 dp spacer each side of the labelLarge label,
    /// and 8 dp and 16 dp per trailing icon.
    fn chip_size(&self, text: &mut Text, c: &Chip) -> f32 {
        let lw = self.styled(text, &c.label, LABEL_LARGE, None).width.ceil();
        lw + self.px(32.0) + self.px(24.0) * c.icons.len() as f32
    }
    /// FlorisChip with 4 dp corners, 32 dp tall in its 48 dp line at `(x, y)` and `w` wide: a
    /// 1 dp outline, the label in onSurfaceVariant, then its 16 dp icons.
    fn ext_chip(
        &mut self,
        p: &mut Panel,
        c: Chip,
        (x, y, w): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) {
        let b = rect(x, y + self.px(8.0), w, self.px(32.0));
        let (radius, line) = (self.px(4.0), self.px(1.0));
        let inside = rect(
            b.left + line,
            b.top + line,
            w - 2.0 * line,
            b.height() - 2.0 * line,
        );
        let fill = if self.focus == c.id {
            self.colors[4]
        } else {
            self.colors[0]
        };
        self.paint_clipped_box(p, b, self.colors[7], radius, clip);
        self.paint_clipped_box(p, inside, fill, radius - line, clip);
        let (icon, gap) = (self.px(16.0), self.px(8.0));
        let room = (w - self.px(32.0) - self.px(24.0) * c.icons.len() as f32).max(1.0);
        let label = self.single_line(text, &c.label, (LABEL_LARGE, 14), room);
        let layout = self.styled(text, &label, LABEL_LARGE, None);
        let (lw, lh) = (layout.width.ceil().min(room), layout.height.ceil());
        let at = rect(
            b.left + 2.0 * gap,
            b.top + ((b.height() - lh) / 2.0).round(),
            lw + 1.0,
            lh,
        );
        self.paint_styled(p, &label, at, LABEL_LARGE, self.colors[2], clip);
        let mut ix = at.left + lw + gap;
        for name in &c.icons {
            let glyph = rect(
                ix + gap,
                b.top + ((b.height() - icon) / 2.0).round(),
                icon,
                icon,
            );
            self.paint_icon(p, name, glyph, self.colors[2], clip);
            ix += gap + icon;
        }
        let role = if c.tap.is_some() { "button" } else { "text" };
        let mut control = plain(c.id, rect(x, y, w, self.px(48.0)), c.label, role);
        control.summary = c.summary;
        control.action = c.tap.map_or(Action::None, Action::Ext);
        self.push_control(control, clip);
    }
}

/// The keywords as chips: plain text, with nothing to tap.
pub(super) fn keyword_chips(ext: &Extension) -> Vec<Chip> {
    let chips = ext.keywords.iter().enumerate();
    chips
        .map(|(k, keyword)| Chip {
            id: KEYWORD_ID + k as i32,
            label: keyword.clone(),
            summary: String::new(),
            icons: vec![],
            tap: None,
        })
        .collect()
}

/// Chips `widths` wide, in lines `room` wide with `gap` between them: each line's count and
/// width. A chip wider than the room has a line of its own.
pub fn flow(widths: &[f32], room: f32, gap: f32) -> Vec<(usize, f32)> {
    let mut lines: Vec<(usize, f32)> = vec![];
    for &w in widths {
        match lines.last_mut() {
            Some((n, at)) if *at + gap + w <= room => {
                *n += 1;
                *at += gap + w;
            }
            _ => lines.push((1, w)),
        }
    }
    lines
}
