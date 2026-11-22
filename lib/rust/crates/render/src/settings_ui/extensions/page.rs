//! An extension's page, as ExtensionViewScreen laid it out, 16 dp in: the description in
//! bodyLarge, 16 dp, then rows of a label and its value with a divider between them (12 dp above
//! and below a value; none round chips): Maintainers, ID, Version, Keywords, Homepage, Issue
//! tracker and License; then Delete and Export as outlined buttons. Under that, the bundled themes
//! or language packs, each in ExtensionComponentView's box. Before the first report the page is
//! empty; when no extension has the id it says so, as ExtensionNotFoundScreen did.
//!
//! A link tells TalkBack where it goes, not only its host. The rows of chips are `chips.rs`.
use super::{Component, Extension, Link, Tap, chips::keyword_chips, component_text};
use crate::settings_ui::{
    Action, App,
    cards::{BODY_BOLD, BODY_LARGE, LABEL_LARGE, Type, plain},
    catalogue::extensions::Part,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

pub const DESCRIPTION_ID: i32 = 300_000;
pub const NOT_FOUND_ID: i32 = 300_001;
/// Row `k`'s label, and its value or link: 0 Maintainers, 1 ID, 2 Version, 3 Keywords,
/// 4 Homepage, 5 Issue tracker, 6 License.
pub const LABEL_ID: i32 = 300_010;
pub const VALUE_ID: i32 = 300_020;
pub const DELETE_ID: i32 = 300_030;
pub const EXPORT_ID: i32 = 300_031;
pub const HEADER_ID: i32 = 300_040;
pub const NONE_ID: i32 = 300_041;
pub const MAINTAINER_ID: i32 = 301_000;
pub const KEYWORD_ID: i32 = 302_000;
/// Component `c`'s box: `COMPONENT_ID + 4 * c`, then 1 its name and 2 its text.
pub const COMPONENT_ID: i32 = 310_000;

impl App {
    /// The page down to its buttons; its height.
    pub(super) fn page_about(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        if self.extensions.list.is_none() {
            return 0.0;
        }
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let Some(ext) = self.page_extension().cloned() else {
            let id = self.page_id().unwrap_or_default().to_owned();
            let message = self
                .label("ext__error__not_found_description")
                .replace("{id}", &id);
            return self.body(p, (NOT_FOUND_ID, &message), (left, y, width), clip, text);
        };
        let mut at = y;
        if let Some(d) = ext.description.as_ref().filter(|d| !d.is_empty()) {
            at += self.body(p, (DESCRIPTION_ID, d), (left, at, width), clip, text);
        }
        at += self.px(16.0);
        let frame = (left, width);
        at += self.chip_row(p, 0, self.maintainer_chips(&ext), (frame, at), clip, text);
        at += self.meta_text(p, 1, &ext.id, (frame, at), clip, text);
        at += self.meta_text(p, 2, &ext.version, (frame, at), clip, text);
        if !ext.keywords.is_empty() {
            at += self.divider(p, (frame, at), clip);
            at += self.chip_row(p, 3, keyword_chips(&ext), (frame, at), clip, text);
        }
        for (k, link) in [(4, &ext.homepage), (5, &ext.issue_tracker)] {
            if let Some(link) = link.as_ref().filter(|_| App::shows_link(link)) {
                at += self.meta_link(p, k, link, (frame, at), clip, text);
            }
        }
        at += self.meta_text(p, 6, &ext.license, (frame, at), clip, text);
        at += self.page_buttons(p, &ext, (frame, at), clip, text);
        at - y
    }
    /// Text in bodyLarge across `width`; its height.
    fn body(
        &mut self,
        p: &mut Panel,
        (id, s): (i32, &str),
        (left, y, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let h = self.styled(text, s, BODY_LARGE, Some(width)).height.ceil();
        let b = rect(left, y, width, h);
        self.paint_styled(p, s, b, BODY_LARGE, self.colors[15], clip);
        self.push_control(plain(id, b, s.into(), "text"), clip);
        h
    }
    /// HorizontalDivider: 1 dp of outlineVariant; its height.
    fn divider(&mut self, p: &mut Panel, ((left, width), y): ((f32, f32), f32), clip: Rect) -> f32 {
        let h = self.px(1.0);
        self.paint_clipped_box(p, rect(left, y, width, h), self.colors[10], 0.0, clip);
        h
    }
    /// Row `k`'s label in bodyLarge at the start; its width.
    pub(super) fn meta_label(&self, text: &mut Text, k: usize, width: f32) -> (String, f32, f32) {
        let key = [
            "ext__meta__maintainers",
            "ext__meta__id",
            "ext__meta__version",
            "ext__meta__keywords",
            "ext__meta__homepage",
            "ext__meta__issue_tracker",
            "ext__meta__license",
        ][k];
        let label = self.label(key);
        let layout = self.styled(text, &label, BODY_LARGE, Some(width));
        let (w, h) = (layout.width.ceil(), layout.height.ceil());
        (label, w, h)
    }
    /// A divider, then row `k`: its label, and `value` at the end in the room left 24 dp after
    /// the label, both centred in 12 dp padding. TalkBack hears the row as one: the label, then
    /// the value. Its height.
    fn meta_text(
        &mut self,
        p: &mut Panel,
        k: usize,
        value: &str,
        ((left, width), y): ((f32, f32), f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let line = self.divider(p, ((left, width), y), clip);
        let top = y + line;
        let (label, lw, lh) = self.meta_label(text, k, width);
        let room = (width - lw - self.px(24.0)).max(1.0);
        let layout = self.styled(text, value, BODY_LARGE, Some(room));
        let (vw, vh) = (layout.width.ceil().min(room), layout.height.ceil());
        let h = lh.max(vh) + 2.0 * self.px(12.0);
        let at = rect(left, top + ((h - lh) / 2.0).round(), lw, lh);
        self.paint_styled(p, &label, at, BODY_LARGE, self.colors[15], clip);
        let v = rect(
            left + width - vw,
            top + ((h - vh) / 2.0).round(),
            vw + 1.0,
            vh,
        );
        self.paint_styled(p, value, v, BODY_LARGE, self.colors[15], clip);
        let mut c = plain(
            VALUE_ID + k as i32,
            rect(left, top, width, h),
            label,
            "text",
        );
        c.summary = value.into();
        self.push_control(c, clip);
        line + h
    }
    /// A divider, then row `k` with a link at the end: its host in the primary colour on one
    /// line, as FlorisHyperlinkText drew it. TalkBack hears the host, then the whole link.
    fn meta_link(
        &mut self,
        p: &mut Panel,
        k: usize,
        link: &Link,
        ((left, width), y): ((f32, f32), f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let line = self.divider(p, ((left, width), y), clip);
        let top = y + line;
        let (label, lw, lh) = self.meta_label(text, k, width);
        let room = (width - lw - self.px(24.0)).max(1.0);
        let host = self.single_line(text, &link.host, (BODY_LARGE, 16), room);
        let layout = self.styled(text, &host, BODY_LARGE, None);
        let (vw, vh) = (layout.width.ceil().min(room), layout.height.ceil());
        let h = lh.max(vh) + 2.0 * self.px(12.0);
        let at = rect(left, top + ((h - lh) / 2.0).round(), lw, lh);
        self.paint_styled(p, &label, at, BODY_LARGE, self.colors[15], clip);
        self.push_control(plain(LABEL_ID + k as i32, at, label, "text"), clip);
        let id = VALUE_ID + k as i32;
        let touch = rect(left + width - vw, top, vw, h);
        if self.focus == id {
            self.paint_clipped_box(p, touch, self.colors[4], self.px(8.0), clip);
        }
        let v = rect(touch.left, top + ((h - vh) / 2.0).round(), vw + 1.0, vh);
        self.paint_styled(p, &host, v, BODY_LARGE, self.colors[3], clip);
        let mut c = plain(id, touch, host, "link");
        c.summary = link.link.clone();
        c.action = Action::Ext(Tap::Link(link.link.clone()));
        self.push_control(c, clip);
        line + h
    }
    /// Delete, in the error colour, at the start for an extension made or imported here, and
    /// Export at the end; their 48 dp row.
    fn page_buttons(
        &mut self,
        p: &mut Panel,
        ext: &Extension,
        ((left, width), y): ((f32, f32), f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        if ext.deletable {
            let delete = (DELETE_ID, "action__delete", "delete", self.colors[18]);
            self.outlined_button(p, (delete, Tap::Delete), (left, y, false), clip, text);
        }
        let export = (EXPORT_ID, "action__export", "share", self.colors[3]);
        self.outlined_button(
            p,
            (export, Tap::Export),
            (left + width, y, true),
            clip,
            text,
        );
        self.px(48.0)
    }
    /// FlorisOutlinedButton from `x` (or ending there): 24 dp padding round the 18 dp icon, 8 dp
    /// and the label in `ink`, at least 58 dp, 40 dp tall with 8 dp corners and a 1 dp outline,
    /// in its 48 dp touch height.
    fn outlined_button(
        &mut self,
        p: &mut Panel,
        ((id, key, icon, ink), tap): ((i32, &str, &'static str, u32), Tap),
        (x, y, end): (f32, f32, bool),
        clip: Rect,
        text: &mut Text,
    ) {
        let label = self.label(key);
        let layout = self.styled(text, &label, LABEL_LARGE, None);
        let (lw, lh) = (layout.width.ceil(), layout.height.ceil());
        let (size, gap) = (self.px(18.0), self.px(8.0));
        let w = (lw + size + gap + 2.0 * self.px(24.0)).max(self.px(58.0));
        let left = if end { x - w } else { x };
        let b = rect(left, y + self.px(4.0), w, self.px(40.0));
        let (radius, line) = (self.px(8.0), self.px(1.0));
        let inside = rect(
            b.left + line,
            b.top + line,
            w - 2.0 * line,
            b.height() - 2.0 * line,
        );
        let fill = if self.focus == id {
            self.colors[4]
        } else {
            self.colors[0]
        };
        self.paint_clipped_box(p, b, self.colors[7], radius, clip);
        self.paint_clipped_box(p, inside, fill, radius - line, clip);
        let start = b.left + ((w - size - gap - lw) / 2.0).round();
        let glyph = rect(
            start,
            b.top + ((b.height() - size) / 2.0).round(),
            size,
            size,
        );
        self.paint_icon(p, icon, glyph, ink, clip);
        let at = rect(
            start + size + gap,
            b.top + ((b.height() - lh) / 2.0).round(),
            lw + 1.0,
            lh,
        );
        self.paint_styled(p, &label, at, LABEL_LARGE, ink, clip);
        let mut c = plain(id, rect(left, y, w, self.px(48.0)), label, "button");
        c.action = Action::Ext(tap);
        self.push_control(c, clip);
    }
    /// The bundled components: ExtensionComponentListView's header, then a box per component,
    /// or the note that there are none; nothing for a keyboard extension. Its height.
    pub(super) fn page_components(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(ext) = self.page_extension().cloned() else {
            return 0.0;
        };
        let key = match ext.serial.as_str() {
            super::THEME => "ext__meta__components_theme",
            super::LANGUAGE_PACK => "ext__meta__components_language_pack",
            _ => return 0.0,
        };
        let mut at = y + self.components_header(p, key, (x, y), clip, text);
        if ext.components.is_empty() {
            let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
            let note = self.label("ext__meta__components_none_found");
            let h = self
                .styled(text, &note, BODY_LARGE, Some(width))
                .height
                .ceil();
            let b = rect(x + self.px(16.0), at + self.px(8.0), width, h);
            self.paint_styled(p, &note, b, BODY_LARGE, self.colors[15], clip);
            p.items
                .last_mut()
                .expect("paint_text pushed an item")
                .text
                .italic = true;
            self.push_control(plain(NONE_ID, b, note, "text"), clip);
            return b.bottom + self.px(8.0) - y;
        }
        for (c, component) in ext.components.iter().enumerate() {
            at += self.bundled_box(p, (c, &ext, component), (x, at), clip, text);
        }
        at - y
    }
    /// The header ListItem: the title in bold and the secondary colour, on one line, 16 dp in,
    /// at least 56 dp tall; its height.
    fn components_header(
        &mut self,
        p: &mut Panel,
        key: &str,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        let title = self.single_line(text, &self.label(key), (BODY_BOLD, 16), width);
        let th = self
            .styled(text, &title, BODY_BOLD, Some(width))
            .height
            .ceil();
        let h = (th + 2.0 * self.px(8.0)).max(self.px(56.0));
        let b = rect(x + self.px(16.0), y + ((h - th) / 2.0).round(), width, th);
        self.paint_styled(p, &title, b, BODY_BOLD, self.colors[9], clip);
        self.push_control(plain(HEADER_ID, b, title, "heading"), clip);
        h
    }
    /// ExtensionComponentView: a box titled with the component's label over its full name, and
    /// what it is in bodyMedium 16 dp in and 8 dp above the bottom; its height with the 8 dp
    /// round it. Nothing in it is tapped, so TalkBack reads it all as text.
    fn bundled_box(
        &mut self,
        p: &mut Panel,
        (c, ext, component): (usize, &Extension, &Component),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let id = COMPONENT_ID + 4 * c as i32;
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let border_top = y + self.px(8.0) + self.px(11.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, border_top, width, 0.0), clip);
        let name = format!("{}:{}", ext.id, component.id);
        let named = (id + 1, name.as_str(), Action::None);
        let mut at = self.box_id(p, named, (left, border_top + self.px(11.0)), clip, text);
        self.as_text();
        let body = component_text(&ext.serial, component);
        let inner = (width - 2.0 * self.px(16.0)).max(1.0);
        let h = self.measure(text, &body, self.sp(14), inner).ceil();
        let b = rect(left + self.px(16.0), at, inner, h);
        self.paint_text(p, &body, b, self.sp(14), self.colors[15], clip);
        self.push_control(plain(id + 2, b, body, "text"), clip);
        at = b.bottom + self.px(8.0);
        self.fit_box(p, outline, at - border_top);
        let border = rect(left, border_top, width, 0.0);
        self.box_title(p, (id, &component.label), Action::None, border, clip, text);
        self.as_text();
        at + self.px(8.0) - y
    }
    /// `s` on one line in style `t` of `size` sp: whole when it fits `room` as `t` measures it,
    /// cut with an ellipsis when not.
    pub(super) fn single_line(
        &self,
        text: &mut Text,
        s: &str,
        (t, size): (Type, usize),
        room: f32,
    ) -> String {
        match self.styled(text, s, t, None).width.ceil() <= room {
            true => s.into(),
            false => self.row_text(text, s, self.sp(size), room, 1),
        }
    }
    /// The control just recorded is passive text, not a button.
    fn as_text(&mut self) {
        if let Some(c) = self.controls.last_mut() {
            c.role = "text";
        }
    }
    /// The page's focus order: the maintainers that offer something, the links, Delete and
    /// Export. Nothing in the components is focusable.
    pub(super) fn page_focus(&self, part: Part) -> Vec<i32> {
        let Some(ext) = self.page_extension().filter(|_| part == Part::About) else {
            return vec![];
        };
        let mut out: Vec<i32> = ext
            .maintainers
            .iter()
            .enumerate()
            .filter(|(_, m)| m.email.is_some() || m.url.is_some())
            .map(|(m, _)| MAINTAINER_ID + m as i32)
            .collect();
        for (k, link) in [(4, &ext.homepage), (5, &ext.issue_tracker)] {
            if App::shows_link(link) {
                out.push(VALUE_ID + k);
            }
        }
        if ext.deletable {
            out.push(DELETE_ID);
        }
        out.push(EXPORT_ID);
        out
    }
}
