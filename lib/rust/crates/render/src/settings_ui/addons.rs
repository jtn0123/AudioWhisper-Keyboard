//! AddonManagementReferenceBox, ThemeScreen's last row: a FlorisOutlinedBox titled "Managing
//! theme extensions" round the addon manager's note and a TextButton to the extension list.
//! Compose lays it out in whole pixels: the box 16 dp in and 8 dp down, its 1 dp outlineVariant
//! border 11 dp below that with 11 dp inside it, the title on the background 12 dp up from the
//! border, and the button's 40 dp container in its 48 dp touch height.
use super::{
    Action, App,
    cards::{BODY_SMALL, LABEL_LARGE, TITLE_SMALL, plain},
    catalogue::Addons,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

/// The box's title template, note and button template, the same for every extension type.
pub const KEYS: [&str; 3] = [
    "ext__addon_management_box__managing_placeholder",
    "ext__addon_management_box__addon_manager_info",
    "ext__addon_management_box__go_to_page",
];

impl App {
    /// "Managing {extensions}", the type's title in lower case as Kotlin's `lowercase()` has it.
    pub fn addons_title(&self, a: &Addons) -> String {
        self.label(KEYS[0])
            .replace("{extensions}", &self.label(a.title).to_lowercase())
    }
    /// "Go to {ext_home_title}", the type's title as it is.
    pub fn addons_button(&self, a: &Addons) -> String {
        self.label(KEYS[2])
            .replace("{ext_home_title}", &self.label(a.title))
    }
    /// Paints row `i`'s box from `(x, y)`; its height with the 8 dp round it.
    pub(super) fn addons(
        &mut self,
        p: &mut Panel,
        i: usize,
        a: &Addons,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let (left, width) = (x + self.px(16.0), self.config[0] - 2.0 * self.px(16.0));
        let border_top = y + self.px(8.0) + self.px(11.0);
        let content = border_top + self.px(11.0);
        let note = self.label(KEYS[1]);
        let inner = width - 2.0 * self.px(16.0);
        let nh = self
            .styled(text, &note, BODY_SMALL, Some(inner))
            .height
            .ceil();
        let row = content + self.px(4.0) + nh + self.px(4.0);
        let bottom = row + self.px(48.0);
        let line = self.px(1.0);
        let border = rect(left, border_top, width, bottom - border_top);
        self.paint_clipped_box(p, border, self.colors[10], self.px(8.0), clip);
        let inside = rect(
            left + line,
            border_top + line,
            width - 2.0 * line,
            border.height() - 2.0 * line,
        );
        self.paint_clipped_box(p, inside, self.colors[0], self.px(8.0) - line, clip);
        let note_box = rect(left + self.px(16.0), content + self.px(4.0), inner, nh);
        self.paint_styled(p, &note, note_box, BODY_SMALL, self.colors[15], clip);
        self.addons_link(p, i, a, (left + width - self.px(6.0), row), clip, text);
        let title = self.row_text(
            text,
            &self.addons_title(a),
            self.sp(14),
            width - 2.0 * self.px(6.0),
            1,
        );
        let layout = self.styled(text, &title, TITLE_SMALL, None);
        let (tw, th) = (layout.width.ceil(), layout.height.ceil());
        let tag = rect(
            left + self.px(10.0),
            border_top - self.px(12.0),
            tw + 2.0 * self.px(6.0),
            self.px(23.0),
        );
        self.paint_clipped_box(p, tag, self.colors[0], 0.0, clip);
        let title_box = rect(
            tag.left + self.px(6.0),
            tag.top + ((tag.height() - th) / 2.0).round(),
            tw,
            th,
        );
        self.paint_styled(p, &title, title_box, TITLE_SMALL, self.colors[15], clip);
        self.push_control(plain(5000 + i as i32, title_box, title, "text"), clip);
        self.push_control(plain(1000 + i as i32, note_box, note, "text"), clip);
        bottom + self.px(8.0) - y
    }
    /// FlorisTextButton ending at `right`: the 18 dp Shop icon, 8 dp, the label in labelLarge
    /// and primary, in 12 dp padding at least 58 dp wide, centred in its 48 dp touch height.
    fn addons_link(
        &mut self,
        p: &mut Panel,
        i: usize,
        a: &Addons,
        (right, top): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) {
        let label = self.addons_button(a);
        let layout = self.styled(text, &label, LABEL_LARGE, None);
        let (lw, lh) = (layout.width.ceil(), layout.height.ceil());
        let (pad, icon, gap) = (self.px(12.0), self.px(18.0), self.px(8.0));
        let content = icon + gap + lw;
        let w = (content + 2.0 * pad).max(self.px(58.0));
        let touch = rect(right - w, top, w, self.px(48.0));
        let container = rect(touch.left, top + self.px(4.0), w, self.px(40.0));
        let id = 6000 + i as i32;
        if self.focus == id {
            let r = container.height() / 2.0;
            self.paint_clipped_box(p, container, self.colors[4], r, clip);
        }
        let start = container.left + pad + ((w - 2.0 * pad - content) / 2.0).round();
        let glyph = rect(
            start,
            container.top + ((container.height() - icon) / 2.0).round(),
            icon,
            icon,
        );
        self.paint_icon(p, "shop", glyph, self.colors[3], clip);
        let at = rect(
            start + icon + gap,
            container.top + ((container.height() - lh) / 2.0).round(),
            lw,
            lh,
        );
        self.paint_styled(p, &label, at, LABEL_LARGE, self.colors[3], clip);
        let mut link = plain(id, touch, label, "button");
        link.action = Action::Route(a.route.into());
        self.push_control(link, clip);
    }
}
