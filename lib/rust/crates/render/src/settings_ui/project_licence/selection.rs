//! The read-only selection, as Compose's SelectionContainer keeps it over one Text: a long
//! press selects a word and a drag after it extends by whole words; a handle drag moves its end
//! by characters and may cross the other end; a tap clears. Copy puts the selected text on the
//! clipboard and clears, as the toolbar's Copy does.
use super::super::App;
use crate::android_text::Line;
use crate::panel_layout::intersection;
use keys::Rect;
use std::ops::Range;
use wire::settings_ui::text::effect;

impl App {
    /// The selected bytes, start first, when something is selected.
    pub(super) fn project_range(&self) -> Option<Range<usize>> {
        let (a, f) = self.project_licence.selection?;
        (a != f).then(|| a.min(f)..a.max(f))
    }
    /// Sets the selection. A change made under the finger ticks the handle-move haptic, as
    /// Compose's `onSelectionChange` does.
    pub(super) fn select_project(&mut self, anchor: usize, focus: usize, haptic: bool) {
        let next = Some((anchor, focus));
        if next != self.project_licence.selection {
            self.project_licence.selection = next;
            if haptic {
                self.effects.push(vec![effect::HAPTIC.into()]);
            }
        }
    }
    /// The toolbar's Select all.
    pub fn project_select_all(&mut self) {
        if let Some(len) = self.project_licence.text.as_ref().map(|t| t.len()) {
            self.select_project(0, len, false);
        }
    }
    /// The toolbar's or TalkBack's Copy: the selected text to the clipboard, then no selection.
    /// Nothing selected copies nothing.
    pub fn project_copy(&mut self) -> bool {
        let (Some(range), Some(text)) = (self.project_range(), self.project_licence.text.clone())
        else {
            return false;
        };
        self.effects
            .push(vec![effect::CLIP.into(), text[range].to_owned()]);
        self.project_licence.selection = None;
        true
    }
    /// The offset nearest the screen point (`x`, `y`).
    pub(super) fn project_offset(&self, x: f32, y: f32) -> Option<usize> {
        let laid = self.project_licence.laid.as_ref()?;
        let [ox, oy] = self.project_origin();
        Some(laid.layout.index_at(x - ox, y - oy))
    }
    /// Where a handle at `offset` points on screen: its caret, a pixel above its line's bottom
    /// so the point stays inside the line, as Compose's `getAdjustedCoordinates` puts it.
    pub fn project_point(&self, offset: usize) -> Option<(f32, f32)> {
        let layout = &self.project_licence.laid.as_ref()?.layout;
        let (line, x) = layout.caret(offset);
        let line = layout.lines.get(line)?;
        let [ox, oy] = self.project_origin();
        Some((ox + x, oy + line.top + line.height - 1.0))
    }
    /// A long press at (`x`, `y`): selects the word there and returns it.
    pub(super) fn project_long_press(&mut self, x: f32, y: f32) -> Option<Range<usize>> {
        let offset = self.project_offset(x, y)?;
        let text = self.project_licence.text.clone()?;
        let word = self.project_licence.words.at(&text, offset);
        self.select_project(word.start, word.end, true);
        Some(word)
    }
    /// A drag after a long press on `pressed`: the selection runs from that word to the word
    /// under the finger, whole words either way.
    pub(super) fn project_extend(&mut self, pressed: Range<usize>, x: f32, y: f32) {
        let (Some(offset), Some(text)) =
            (self.project_offset(x, y), self.project_licence.text.clone())
        else {
            return;
        };
        let word = self.project_licence.words.at(&text, offset);
        if word.start >= pressed.start {
            self.select_project(pressed.start, word.end.max(pressed.end), true);
        } else {
            self.select_project(pressed.end, word.start, true);
        }
    }
    /// A handle drag: the dragged end moves to the offset at (`x`, `y`) while `other` stays.
    /// It never collapses onto the other end, as `ensureAtLeastOneChar` keeps one character.
    pub(super) fn project_drag_handle(&mut self, other: usize, x: f32, y: f32) {
        if let Some(offset) = self.project_offset(x, y).filter(|o| *o != other) {
            self.select_project(other, offset, true);
        }
    }
    /// The selection's highlight boxes on screen, one a line, as Android's `Layout.getSelection`
    /// draws them for Compose: every line but the last runs on to the text's right edge and every
    /// line but the first starts at its left, so the lines between, blank ones too, are filled.
    pub fn project_boxes(&self) -> Vec<Rect> {
        let (Some(range), Some(laid)) = (self.project_range(), self.project_licence.laid.as_ref())
        else {
            return vec![];
        };
        let [ox, oy] = self.project_origin();
        let lines = &laid.layout.lines;
        // Compose lays the text out at least as wide as the screen (fillMaxSize).
        let width = laid.layout.width.max(self.project_licence.viewport.width());
        let line_of = |offset: usize| {
            lines
                .iter()
                .position(|l| offset < l.range.end)
                .unwrap_or(lines.len().saturating_sub(1))
        };
        let (first, last) = (line_of(range.start), line_of(range.end));
        let x = |line: &Line, at: usize| {
            line.carets
                .iter()
                .rev()
                .find(|(c, _)| *c <= at)
                .map_or(0.0, |c| c.1)
        };
        (first..=last)
            .filter_map(|i| lines.get(i).map(|line| (i, line)))
            .map(|(i, line)| Rect {
                left: ox
                    + if i == first {
                        x(line, range.start)
                    } else {
                        0.0
                    },
                top: oy + line.top,
                right: ox + if i == last { x(line, range.end) } else { width },
                bottom: oy + line.top + line.height,
            })
            .filter(|b| b.right > b.left)
            .collect()
    }
    /// Where Android's floating toolbar goes and whether it offers Select all: around the
    /// selection's visible part with the handles under it. None while nothing is selected,
    /// while a selection drag runs, or when the selection is scrolled out of view.
    pub fn text_toolbar(&self) -> Option<[f32; 5]> {
        if !self.on_project_licence() || self.project_licence.touch.selecting() {
            return None;
        }
        let range = self.project_range()?;
        let viewport = self.project_licence.viewport;
        let visible: Vec<Rect> = self
            .project_boxes()
            .into_iter()
            .map(|b| intersection(viewport, b))
            .filter(|b| b.width() > 0.0 && b.height() > 0.0)
            .collect();
        let first = visible.first()?;
        let mut area = *first;
        for b in &visible {
            area.left = area.left.min(b.left);
            area.right = area.right.max(b.right);
            area.bottom = area.bottom.max(b.bottom);
        }
        area.bottom = (area.bottom + 25.0 * self.config[2]).min(viewport.bottom);
        let all = self
            .project_licence
            .text
            .as_ref()
            .is_some_and(|t| range == (0..t.len()));
        Some([
            area.left,
            area.top,
            area.right,
            area.bottom,
            f32::from(u8::from(!all)),
        ])
    }
}
