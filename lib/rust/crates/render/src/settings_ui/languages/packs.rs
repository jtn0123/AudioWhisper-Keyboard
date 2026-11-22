//! The language pack manager, as LanguagePackManagerScreen had it: an outlined box with Import,
//! then one FlorisOutlinedBox per language pack extension, titled with its name over its id
//! (both open its page), listing its packs by label, with Delete when it can be deleted. Import
//! and the page are Compose's extension screens; Delete asks first, then Kotlin deletes.
use super::{Pack, Tap};
use crate::settings_ui::{
    Action, App,
    cards::{TITLE_SMALL, plain},
    items::Entry,
    languages::form::Button,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// The Import row's control id.
pub const IMPORT_ID: i32 = 7905;
/// Pack `p`'s controls start at `PACK_ID + 64 * p`: its title, id, delete button, then items.
pub const PACK_ID: i32 = 50000;

/// Pack `p`'s control `part`: 0 the title, 1 the id, 2 Delete, 3 onwards its items.
pub fn pack_id(p: usize, part: usize) -> i32 {
    PACK_ID + 64 * p as i32 + part.min(63) as i32
}

impl App {
    /// The outlined box round Import: 16 dp in, 8 dp down; its height with that padding.
    pub(in crate::settings_ui) fn import_row(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, y + self.px(8.0), width, 0.0), clip);
        let entry = Entry {
            id: IMPORT_ID,
            title: self.label("action__import"),
            summary: String::new(),
            icon: "input",
            ink: [None, None],
            buttons: 0,
            right: self.px(16.0),
            enabled: true,
            action: Action::Lang(Tap::Import),
        };
        let (rh, _) = self.paint_entry(p, entry, (left, y + self.px(8.0)), clip, text);
        self.fit_box(p, outline, rh);
        rh + 2.0 * self.px(8.0)
    }
    /// Every extension's box; their height.
    pub(in crate::settings_ui) fn packs_row(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let packs = self
            .languages
            .resources
            .as_ref()
            .map(|r| r.packs.clone())
            .unwrap_or_default();
        let mut at = y;
        for (k, pack) in packs.iter().enumerate() {
            at += self.pack_box(p, k, pack, (x, at), clip, text);
        }
        at - y
    }
    /// Extension `pack`'s box: the 11 dp title gap above the border and inside it, the id, the
    /// packs, and Delete in a row 6 dp in; its height with the 8 dp round it.
    fn pack_box(
        &mut self,
        p: &mut Panel,
        k: usize,
        pack: &Pack,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let border_top = y + self.px(8.0) + self.px(11.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, border_top, width, 0.0), clip);
        let view = Action::Lang(Tap::View(pack.id.clone()));
        let id = (pack_id(k, 1), pack.id.as_str(), view);
        let mut at = self.box_id(p, id, (left, border_top + self.px(11.0)), clip, text);
        for (j, item) in pack.items.iter().enumerate() {
            let entry = Entry {
                id: pack_id(k, 3 + j),
                title: item.clone(),
                summary: String::new(),
                icon: "",
                ink: [None, None],
                buttons: 0,
                right: self.px(16.0),
                enabled: true,
                action: Action::None,
            };
            at += self.paint_entry(p, entry, (left, at), clip, text).0;
        }
        if pack.deletable {
            let delete = Button {
                id: pack_id(k, 2),
                label: self.label("action__delete"),
                icon: "delete",
                filled: false,
                ink: self.colors[18],
                action: Action::Lang(Tap::DeletePack(pack.id.clone())),
            };
            let bw = self.button_width(text, &delete);
            let touch = rect(left + self.px(6.0), at, bw, self.px(48.0));
            self.material_button(p, delete, touch, clip, text);
            at = touch.bottom;
        }
        self.fit_box(p, outline, at - border_top);
        let title = (pack_id(k, 0), pack.title.as_str());
        let view = Action::Lang(Tap::View(pack.id.clone()));
        let border = rect(left, border_top, width, 0.0);
        self.box_title(p, title, view, border, clip, text);
        at + self.px(8.0) - y
    }
    /// A box's id under its title: 10 sp monospace at 56%, 6 dp in its tappable box 10 dp in
    /// from `left`, 4 dp under each; where the content under it starts.
    pub(in crate::settings_ui) fn box_id(
        &mut self,
        p: &mut Panel,
        (control, id, action): (i32, &str, Action),
        (left, at): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let size = self.sp(10);
        let layout = text.layout_typography(id, size, true, 400, false, [0.0, 0.0]);
        let (iw, ih) = (layout.width().ceil(), layout.height().ceil());
        let id_box = rect(
            left + self.px(10.0),
            at,
            iw + 2.0 * self.px(6.0),
            ih + self.px(4.0),
        );
        let id_at = rect(id_box.left + self.px(6.0), at, iw + 1.0, ih);
        let faded = (self.colors[1] & 0x00ff_ffff) | 0x8f00_0000;
        self.paint_text(p, id, id_at, size, faded, clip);
        p.items
            .last_mut()
            .expect("paint_text pushed an item")
            .text
            .mono = true;
        let mut c = plain(control, id_box, id.into(), "button");
        c.action = action;
        self.push_control(c, clip);
        id_box.bottom + self.px(4.0)
    }
    /// A box's title on the background over the border, tapped as `action`: 23 dp tall, 10 dp in
    /// and 12 dp up, in 6 dp padding, in titleSmall on one line. `b` is the border's top edge.
    pub(in crate::settings_ui) fn box_title(
        &mut self,
        p: &mut Panel,
        (control, name): (i32, &str),
        action: Action,
        b: Rect,
        clip: Rect,
        text: &mut Text,
    ) {
        let (left, border_top) = (b.left, b.top);
        let room = b.width() - 2.0 * self.px(6.0) - self.px(10.0);
        let title = self.row_text(text, name, self.sp(14), room, 1);
        let layout = self.styled(text, &title, TITLE_SMALL, None);
        let (tw, th) = (layout.width.ceil(), layout.height.ceil());
        let tag = rect(
            left + self.px(10.0),
            border_top - self.px(12.0),
            tw + 2.0 * self.px(6.0),
            self.px(23.0),
        );
        if self.focus == control {
            self.paint_clipped_box(p, tag, self.colors[4], self.px(4.0), clip);
        } else {
            self.paint_clipped_box(p, tag, self.colors[0], 0.0, clip);
        }
        let at = rect(
            tag.left + self.px(6.0),
            tag.top + ((tag.height() - th) / 2.0).round(),
            tw,
            th,
        );
        self.paint_styled(p, &title, at, TITLE_SMALL, self.colors[1], clip);
        let mut c = plain(control, tag, name.into(), "button");
        c.action = action;
        self.push_control(c, clip);
        // Painted last to sit over the border, but read first: TalkBack names the box, then
        // what is in it, as Compose's on-screen order did.
        let at = self.box_start.min(self.controls.len() - 1);
        self.controls[at..].rotate_right(1);
    }
    /// FlorisOutlinedBox's border: 1 dp outlineVariant round the background, 8 dp corners. Its
    /// height is set by [`fit_box`](Self::fit_box) once the content is painted, and its title
    /// by [`box_title`](Self::box_title) reads before the controls recorded from here on.
    pub(in crate::settings_ui) fn outlined_box(&mut self, p: &mut Panel, b: Rect, clip: Rect) {
        self.box_start = self.controls.len();
        self.paint_clipped_box(p, b, self.colors[10], self.px(8.0), clip);
        let line = self.px(1.0);
        let inside = rect(b.left + line, b.top + line, b.width() - 2.0 * line, 0.0);
        self.paint_clipped_box(p, inside, self.colors[0], self.px(8.0) - line, clip);
    }
    /// Gives the border painted at item `at` its height.
    pub(in crate::settings_ui) fn fit_box(&self, p: &mut Panel, at: usize, height: f32) {
        let line = self.px(1.0);
        let b = p.items[at].bounds;
        p.items[at].bounds = rect(b.left, b.top, b.width(), height);
        let i = p.items[at + 1].bounds;
        p.items[at + 1].bounds = rect(i.left, i.top, i.width(), (height - 2.0 * line).max(0.0));
    }
    /// The focus order of the boxes: each title, id, pack, then Delete.
    pub(in crate::settings_ui) fn packs_focus(&self) -> Vec<i32> {
        let packs = self
            .languages
            .resources
            .as_ref()
            .map_or(&[][..], |r| &r.packs[..]);
        let mut out = vec![];
        for (k, pack) in packs.iter().enumerate() {
            out.extend([pack_id(k, 0), pack_id(k, 1)]);
            if pack.deletable {
                out.push(pack_id(k, 2));
            }
        }
        out
    }
}
