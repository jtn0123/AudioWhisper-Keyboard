//! TalkBack on the licence: for read-only text, the selection actions and the text traversal
//! the editor field exposes. Kotlin sends them as settings op 26 and sends the traversal's
//! `TYPE_VIEW_TEXT_TRAVERSED_AT_MOVEMENT_GRANULARITY` event from what the node reports.
use super::super::App;
use super::{SCROLL_ID, TEXT_ID, bounds};
use crate::android_text::Layout;
use std::ops::Range;
use wire::settings_ui::text::selection;

/// `AccessibilityNodeInfo`'s movement granularities.
pub const CHARACTER: i32 = 1;
pub const WORD: i32 = 2;
pub const LINE: i32 = 4;
pub const PARAGRAPH: i32 = 8;
pub const PAGE: i32 = 16;
/// `ACTION_NEXT_AT_MOVEMENT_GRANULARITY` and `ACTION_PREVIOUS_AT_MOVEMENT_GRANULARITY`.
pub const NEXT: i32 = 256;
pub const PREVIOUS: i32 = 512;

/// The line after the one `from` is in, or that line when `from` starts it: newline included.
fn next_line(layout: &Layout, len: usize, from: usize) -> Option<Range<usize>> {
    let (i, _) = layout.caret(from);
    let next = if layout.lines.get(i)?.range.start == from {
        i
    } else {
        i + 1
    };
    let line = layout.lines.get(next)?;
    (line.range.start < len).then(|| line.range.clone())
}
/// The line before the one `from` starts, or the line `from` is inside.
fn prev_line(layout: &Layout, from: usize) -> Option<Range<usize>> {
    if from == 0 {
        return None;
    }
    let (i, _) = layout.caret(from);
    let prev = if layout.lines.get(i)?.range.start == from {
        i.checked_sub(1)?
    } else {
        i
    };
    Some(layout.lines.get(prev)?.range.start..from)
}
/// A page from `from`: as many whole lines as the viewport holds, forward or back.
fn page(
    layout: &Layout,
    len: usize,
    from: usize,
    forward: bool,
    height: f32,
) -> Option<Range<usize>> {
    let line = layout.lines.first()?.height.max(1.0);
    let lines = ((height / line).floor() as usize).max(1);
    let (i, _) = layout.caret(from);
    if forward {
        let last = (i + lines).min(layout.lines.len()) - 1;
        (from < len).then(|| from..layout.lines[last].range.end.max(from))
    } else {
        let first = i.saturating_sub(lines);
        (from > 0).then(|| layout.lines[first].range.start..from)
    }
}
impl App {
    /// A `SELECTION` operation (settings op 26). One that does not apply, as when the text is
    /// not on screen, does nothing.
    pub fn text_selection(&mut self, p: &[String]) -> Result<(), &'static str> {
        let number = |i: usize| {
            p.get(i)
                .and_then(|v| v.parse::<i32>().ok())
                .ok_or("settings selection field")
        };
        match p.first().map(String::as_str) {
            Some(selection::COPY) => {
                self.project_copy();
            }
            Some(selection::SELECT_ALL) => self.project_select_all(),
            Some(selection::SET) => self.set_project_selection(number(1)?, number(2)?),
            Some(selection::TRAVERSE) => {
                let granularity = number(1)?;
                if ![CHARACTER, WORD, LINE, PARAGRAPH, PAGE].contains(&granularity) {
                    return Err("settings selection granularity");
                }
                self.traverse(granularity, number(2)? == 1, number(3)? == 1);
            }
            _ => return Err("settings selection kind"),
        }
        Ok(())
    }
    /// `ACTION_SET_SELECTION` in UTF-16 units; a negative end clears the selection, as
    /// TextView's `Selection.removeSelection`, and one past the text does nothing.
    fn set_project_selection(&mut self, start: i32, end: i32) {
        let Some(text) = self.project_licence.text.clone() else {
            return;
        };
        let units = bounds::utf16(&text, text.len()) as i32;
        if start < 0 || end < 0 {
            self.project_licence.selection = None;
        } else if start <= units && end <= units {
            let (a, b) = (
                bounds::byte(&text, start as usize),
                bounds::byte(&text, end as usize),
            );
            self.select_project(a, b, false);
        }
    }
    /// Moves TalkBack's cursor, the selection's focus, over the next or previous segment, or
    /// extends the selection over it, as Compose's `traverseAtGranularity` does; records the
    /// segment for the event, or None at the text's end so TalkBack moves on.
    fn traverse(&mut self, granularity: i32, forward: bool, extend: bool) {
        self.project_licence.traversed = None;
        let Some(text) = self.project_licence.text.clone() else {
            return;
        };
        let from = self
            .project_licence
            .selection
            .map_or(if forward { 0 } else { text.len() }, |(_, focus)| focus);
        let Some(segment) = self.segment(&text, granularity, forward, from) else {
            return;
        };
        let (start, end) = (segment.start, segment.end);
        let (to, far) = if forward { (end, start) } else { (start, end) };
        let anchor = match self.project_licence.selection {
            Some((anchor, _)) if extend => anchor,
            None if extend => far,
            _ => to,
        };
        self.project_licence.selection = Some((anchor, to));
        let unit = |at: usize| bounds::utf16(&text, at) as i32;
        let action = if forward { NEXT } else { PREVIOUS };
        self.project_licence.traversed = Some([unit(start), unit(end), granularity, action]);
    }
    /// The segment of `granularity` after or before `from`, or None at that end of the text.
    fn segment(
        &self,
        text: &str,
        granularity: i32,
        forward: bool,
        from: usize,
    ) -> Option<Range<usize>> {
        let layout = &self.project_licence.laid.as_ref()?.layout;
        let (len, words) = (text.len(), &self.project_licence.words);
        match (granularity, forward) {
            (CHARACTER, true) => bounds::next_char(text, from).map(|end| from..end),
            (CHARACTER, false) => bounds::prev_char(text, from).map(|start| start..from),
            (WORD, true) => words.following(from),
            (WORD, false) => words.preceding(from),
            (LINE, true) => next_line(layout, len, from),
            (LINE, false) => prev_line(layout, from),
            (PARAGRAPH, true) => bounds::next_paragraph(text, from),
            (PARAGRAPH, false) => bounds::prev_paragraph(text, from),
            _ => {
                let height = self.project_licence.viewport.height();
                page(layout, len, from, forward, height)
            }
        }
    }
    /// A node's selection in UTF-16 units: the text's for its node, -1 when it has none, and
    /// the editor's for every other node.
    pub fn node_selection(&self, id: i32) -> (i64, i64) {
        if id != TEXT_ID {
            return (self.editor.anchor as i64, self.editor.focus as i64);
        }
        match (self.project_licence.selection, &self.project_licence.text) {
            (Some((a, f)), Some(t)) => (bounds::utf16(t, a) as i64, bounds::utf16(t, f) as i64),
            _ => (-1, -1),
        }
    }
    /// The text node's traversal facts: the granularities it moves by and the last traversal,
    /// as from, to, granularity and action. None for every other node.
    pub fn text_node(&self, id: i32) -> Option<serde_json::Value> {
        let granularities = CHARACTER | WORD | LINE | PARAGRAPH | PAGE;
        let traversed = self.project_licence.traversed;
        (id == TEXT_ID)
            .then(|| serde_json::json!({"granularities": granularities, "traversed": traversed}))
    }
    /// Which ways a scroll node can scroll, back then forward: the horizontal scroller's own,
    /// any other the screen's.
    pub fn node_can_scroll(&self, id: i32) -> [bool; 2] {
        if id != SCROLL_ID {
            return self.can_scroll();
        }
        let at = super::super::shown(self.project_licence.scroll_x);
        [
            at > 0.0,
            at < super::super::shown(self.project_licence.max_x),
        ]
    }
    /// TalkBack's scroll on the horizontal scroller: a page of its width.
    pub(in crate::settings_ui) fn project_page(&mut self, forward: bool) {
        let page = self.project_licence.viewport.width();
        let direction = if forward { 1.0 } else { -1.0 };
        self.set_project_x(self.project_licence.scroll_x + direction * page);
    }
}
