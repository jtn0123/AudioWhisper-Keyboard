//! The code editor's painting and the rule editor's code chips. The editor shows the key data
//! preview, the code field with *record a key* at its end (Compose's Pageview button, red while
//! recording) and its message under it, and the help with its two links, which Compose hid behind
//! a help button in the dialog's title and a screen has room for. The rule editor shows each code
//! as a chip that opens the editor on it, and Add.
use super::{
    Tap,
    codes::{CodeTap, LINKS},
    codes_preview::{key_data, shown_code},
    ui::Chip,
};
use crate::settings_ui::{Action, App, Control, cards::plain, catalogue::theme_editor::CodeRow};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

pub const PREVIEW_ID: i32 = 400_300;
pub const RECORD_ID: i32 = 400_301;
/// The help's two paragraphs, and its link `k` at `LINK_ID + k`.
pub const HELP_ID: i32 = 400_302;
pub const LINK_ID: i32 = 400_304;
/// The rule editor's codes: the note that there are none, Add, and code `j` at `CODE_ID + j`.
pub const NO_CODES_ID: i32 = 400_309;
pub const CODE_ADD_ID: i32 = 400_310;
pub const CODE_ID: i32 = 401_000;
/// The help's paragraphs.
const HELP: [&str; 2] = [
    "settings__theme_editor__code_recording_help_text",
    "settings__theme_editor__code_help_text",
];

impl App {
    /// Paints the code editor's row `row`; its height.
    pub(super) fn code_part(
        &mut self,
        p: &mut Panel,
        (i, row): (usize, CodeRow),
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        if self.theme_editor.code.is_none() {
            return 0.0;
        }
        match row {
            CodeRow::Preview => self.code_preview(p, at, clip, text),
            CodeRow::Field => self.code_field(p, i, at, clip, text),
            CodeRow::Help => self.code_help(p, at, clip, text),
        }
    }
    /// TextKeyDataPreviewBox: a 36 dp box with the key's icon or label, then its name over its
    /// kind.
    fn code_preview(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let Some(f) = self.theme_editor.code.as_ref() else {
            return 0.0;
        };
        let shown = self.key_preview(&key_data(shown_code(&f.text)), f.said.as_ref());
        let (left, pad, size) = (x + self.px(16.0), self.px(8.0), self.px(36.0));
        let label = shown.label.clone().unwrap_or_default();
        let lw = self.android(text, &label, self.sp(16), None).width.ceil();
        let b = rect(left, y + pad, size.max(lw), size);
        let fill = (self.colors[1] & 0x00ff_ffff) | 0x1f00_0000;
        self.paint_clipped_box(p, b, fill, self.px(12.0), clip);
        if shown.icon.is_empty() {
            let lh = self
                .measure(text, &label, self.sp(16), b.width() + 1.0)
                .ceil();
            let at = rect(
                b.left,
                b.top + ((size - lh) / 2.0).round(),
                b.width() + 1.0,
                lh,
            );
            self.paint_text(p, &label, at, self.sp(16), self.colors[1], clip);
        } else {
            let icon = self.px(24.0);
            let at = rect(
                b.left + ((b.width() - icon) / 2.0).round(),
                b.top + self.px(6.0),
                icon,
                icon,
            );
            self.paint_path(p, &shown.icon, at, self.colors[1], clip);
        }
        let column = b.right + self.px(16.0);
        let width = (self.config[0] - column - self.px(16.0)).max(1.0);
        let nh = self.measure(text, &shown.name, self.sp(14), width).ceil();
        let kh = self.measure(text, shown.kind, self.sp(14), width).ceil();
        let top = b.top + ((size - nh - kh) / 2.0).max(0.0).round();
        self.paint_text(
            p,
            &shown.name,
            rect(column, top, width, nh),
            self.sp(14),
            self.colors[1],
            clip,
        );
        let kind = rect(column, top + nh, width, kh);
        self.paint_text(p, shown.kind, kind, self.sp(14), self.colors[2], clip);
        let bottom = b.bottom.max(kind.bottom) + pad;
        let spoken: Vec<&str> = [label.as_str(), &shown.name]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();
        let mut c = plain(
            PREVIEW_ID,
            rect(left, y, b.width(), bottom - y),
            spoken.join(", "),
            "text",
        );
        c.summary = shown.kind.into();
        self.push_control(c, clip);
        bottom - y
    }
    /// The code field, *record a key* at its end, and the message under it; its height.
    fn code_field(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let recording = self.theme_editor.code.as_ref().is_some_and(|f| f.recording);
        let message = self.code_message();
        let (left, width) = (
            x + self.px(16.0),
            (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0),
        );
        let b = rect(left, y, width, self.px(56.0));
        let id = 1000 + i as i32;
        let focused = self.focus == id || (self.field_editor(id).1 && self.editor.focused);
        self.outlined_field(p, b, (focused, !recording && !message.is_empty()), clip);
        let button = self.px(48.0);
        // The placeholder is both what an empty field shows and TalkBack's hint, as Compose's was.
        let mut field = plain(
            id,
            rect(b.left, b.top, (width - button).max(1.0), b.height()),
            self.label("settings__theme_editor__code_placeholder"),
            "edit",
        );
        (field.summary, field.action) = (message.clone(), Action::Field(i));
        self.paint_editor(p, &field, text, clip);
        self.push_control(field, clip);
        let icon = self.px(24.0);
        let at = rect(
            b.right - self.px(12.0) - icon,
            b.top + self.px(16.0),
            icon,
            icon,
        );
        self.paint_icon(
            p,
            "pageview",
            at,
            self.colors[if recording { 18 } else { 2 }],
            clip,
        );
        let touch = rect(b.right - button, b.top + self.px(4.0), button, button);
        let record = Control {
            checked: Some(recording),
            action: Action::ThemeEdit(Tap::Code(CodeTap::Record)),
            ..plain(
                RECORD_ID,
                touch,
                self.label("settings__theme_editor__code_record"),
                "button",
            )
        };
        self.push_control(record, clip);
        let mut bottom = b.bottom;
        if !message.is_empty() {
            let mh = self.measure(text, &message, self.sp(14), width).ceil();
            let at = rect(left, b.bottom + self.px(4.0), width, mh);
            let ink = self.colors[if recording { 3 } else { 18 }];
            self.paint_text(p, &message, at, self.sp(14), ink, clip);
            bottom = at.bottom;
        }
        bottom + self.px(8.0) - y
    }
    /// How to record a key, where to look a code up, and the two links; its height.
    fn code_help(&mut self, p: &mut Panel, (x, y): (f32, f32), clip: Rect, text: &mut Text) -> f32 {
        let mut h = 0.0;
        for (k, key) in HELP.iter().enumerate() {
            let note = (
                HELP_ID + k as i32,
                self.label(key),
                self.colors[2],
                self.px(4.0),
            );
            h += self.theme_note(p, note, (x, y + h), clip, text);
        }
        for (k, (name, _)) in LINKS.iter().enumerate() {
            h += self.code_link(p, k, name, (x + self.px(16.0), y + h), clip, text);
        }
        h + self.px(8.0)
    }
    /// FlorisHyperlinkText: `name` underlined in the primary colour, in a 48 dp row; its height.
    fn code_link(
        &mut self,
        p: &mut Panel,
        k: usize,
        name: &str,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let room = (self.config[0] - x - self.px(16.0)).max(1.0);
        let layout = self.android(text, name, self.sp(14), Some(room));
        let (w, th) = (layout.width.ceil().min(room), layout.height.ceil());
        let row = self.px(48.0).max(th);
        let at = rect(x, y + ((row - th) / 2.0).round(), w + 1.0, th);
        let ink = self.colors[3];
        self.paint_text(p, name, at, self.sp(14), ink, clip);
        let line = rect(at.left, at.bottom - self.px(1.0), w, self.px(1.0));
        self.paint_clipped_box(p, line, ink, 0.0, clip);
        let mut c = plain(LINK_ID + k as i32, rect(x, y, w, row), name.into(), "link");
        c.action = Action::ThemeEdit(Tap::Code(CodeTap::Link(k)));
        self.push_control(c, clip);
        row
    }
    /// The rule's codes as chips that open the code editor, or the note that there are none,
    /// and Add; where they end.
    pub(super) fn code_chips(
        &mut self,
        p: &mut Panel,
        (x, mut y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let codes = self
            .theme_editor
            .form
            .as_ref()
            .map_or(vec![], |f| f.values(0));
        if codes.is_empty() {
            let (note, ink) = (
                self.label("settings__theme_editor__no_codes_defined"),
                self.colors[2],
            );
            y += self.theme_note(p, (NO_CODES_ID, note, ink, 0.0), (x, y), clip, text);
        }
        let mut chips: Vec<Chip> = codes
            .into_iter()
            .enumerate()
            .map(|(j, code)| Chip {
                id: code_chip_id(j),
                label: code,
                selected: true,
                action: Action::ThemeEdit(Tap::Code(CodeTap::Open(Some(j)))),
            })
            .collect();
        chips.push(Chip {
            id: CODE_ADD_ID,
            label: self.label("action__add"),
            selected: false,
            action: Action::ThemeEdit(Tap::Code(CodeTap::Open(None))),
        });
        self.chips(p, chips, (x, y), clip, text)
    }
    /// The code chips' focus order: each code, then Add.
    pub(super) fn code_chips_focus(&self) -> Vec<i32> {
        let n = self
            .theme_editor
            .form
            .as_ref()
            .map_or(0, |f| f.values(0).len());
        (0..n).map(code_chip_id).chain([CODE_ADD_ID]).collect()
    }
    /// The code editor's focus order, row by row: Delete in the top bar first when editing.
    pub(super) fn code_focus(&self, i: usize, row: CodeRow) -> Vec<i32> {
        match row {
            CodeRow::Preview if self.code_header().is_some() => vec![super::paint::DELETE_ID],
            CodeRow::Preview => vec![],
            CodeRow::Field => vec![1000 + i as i32, RECORD_ID],
            CodeRow::Help => vec![LINK_ID, LINK_ID + 1],
        }
    }
}

/// Code chip `j`'s control id.
fn code_chip_id(j: usize) -> i32 {
    CODE_ID + j.min(90_000) as i32
}
