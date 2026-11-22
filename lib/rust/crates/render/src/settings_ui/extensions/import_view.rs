//! The importer's painting, as ExtensionImportScreen laid it out: Select files as an outlined
//! button centred in 16 dp above and below, then FileInfoView's box for each file read, "No files
//! selected" in italics, or that the files could not be read. A file's box is titled with its
//! name over its media type; in bodyMedium 16 dp in it has the size, id and version in grey, the
//! title, the description in italics, who made it and its components, then, for a file it skips,
//! a thin error bar and why, in the error colour.
use super::{
    Tap,
    import::{File, Skip},
};
use crate::settings_ui::{
    Action, App,
    cards::{BODY_LARGE, plain},
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;

pub const SELECT_ID: i32 = 320_000;
pub const NONE_ID: i32 = 320_001;
pub const FAILED_ID: i32 = 320_002;
/// File `f`'s box: `FILE_ID + 1000 * f`, then 1 its media type and its lines from 2.
pub const FILE_ID: i32 = 321_000;

/// A paragraph: its control, text, ink and whether it is in italics.
type Line<'a> = (i32, &'a str, u32, bool);

/// `ink` at 56%, as Compose's `copy(alpha = 0.56f)`.
fn faded(ink: u32) -> u32 {
    (ink & 0x00ff_ffff) | 0x8f00_0000
}

impl App {
    /// The importer's content; its height.
    pub(super) fn import_part(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let mut at = y;
        if self.shows_select() {
            let select = (SELECT_ID, "action__select_files", "", self.colors[3]);
            let centre = x + (self.config[0] - x) / 2.0;
            let top = at + self.px(16.0) - self.px(4.0);
            self.outlined_button(p, (select, Tap::Select), (centre, top, 0.5), clip, text);
            at += self.px(16.0) + self.px(40.0) + self.px(16.0);
        }
        let left = x + self.px(16.0);
        let width = (self.config[0] - x - 2.0 * self.px(16.0)).max(1.0);
        if self.extensions.import.failed {
            let failed = self.label("settings__ext__read_failed");
            let line = (FAILED_ID, failed.as_str(), self.colors[18], false);
            return at + self.line(p, line, (left, at, width), clip, text) - y;
        }
        if self.import_workspace().is_some() {
            for (f, (file, skip)) in self.import_files().into_iter().enumerate() {
                at += self.file_box(p, (f, &file, skip), (x, at), clip, text);
            }
            return at - y;
        }
        if self.files.busy.is_some() {
            return at - y;
        }
        let none = self.label("state__no_files_selected");
        let w = self
            .styled(text, &none, BODY_LARGE, None)
            .width
            .ceil()
            .min(width);
        let h = self.styled(text, &none, BODY_LARGE, Some(w)).height.ceil();
        let b = rect(left + ((width - w) / 2.0).round(), at, w, h);
        self.paint_styled(p, &none, b, BODY_LARGE, self.colors[15], clip);
        p.items
            .last_mut()
            .expect("paint_text pushed an item")
            .text
            .italic = true;
        self.push_control(plain(NONE_ID, b, none, "text"), clip);
        at + h - y
    }
    /// `line` in bodyMedium across `width`; its height.
    fn line(
        &mut self,
        p: &mut Panel,
        (id, s, ink, italic): Line,
        (left, y, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let h = self.measure(text, s, self.sp(14), width).ceil();
        let b = rect(left, y, width, h);
        self.paint_text(p, s, b, self.sp(14), ink, clip);
        p.items
            .last_mut()
            .expect("paint_text pushed an item")
            .text
            .italic = italic;
        self.push_control(plain(id, b, s.into(), "text"), clip);
        h
    }
    /// What a file's box says under its title, line by line; an empty line is 8 dp of space.
    fn file_lines(&self, file: &File, skip: Option<Skip>) -> Vec<(String, u32, bool)> {
        let (ink, error) = (self.colors[15], self.colors[18]);
        let mut head = file.size.clone();
        let mut out = vec![];
        if let Some(a) = &file.archive {
            head = format!("{head} • {} • {}", a.id, a.version);
            out.push((String::new(), ink, false));
            out.push((a.title.clone(), ink, false));
            if let Some(d) = &a.description {
                out.push((d.clone(), ink, true));
            }
            out.push((String::new(), ink, false));
            let by = self.label("ext__meta__maintainers_by");
            out.push((
                by.replace("{maintainers}", &a.maintainers.join(", ")),
                ink,
                false,
            ));
            out.push((String::new(), ink, false));
            out.extend(a.components.iter().map(|c| (c.clone(), ink, false)));
        }
        out.insert(0, (head, faded(self.colors[1]), false));
        if let Some(s) = skip {
            out.push((self.label("ext__import__file_skip"), error, false));
            out.push((self.label(s.key()), error, true));
        }
        out
    }
    /// FileInfoView: file `f`'s box; its height with the 8 dp round it. Nothing in it is
    /// tapped, so TalkBack reads it all as text.
    fn file_box(
        &mut self,
        p: &mut Panel,
        (f, file, skip): (usize, &File, Option<Skip>),
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let id = FILE_ID + 1000 * f as i32;
        let left = x + self.px(16.0);
        let width = self.config[0] - x - 2.0 * self.px(16.0);
        let border_top = y + self.px(8.0) + self.px(11.0);
        let outline = p.items.len();
        self.outlined_box(p, rect(left, border_top, width, 0.0), clip);
        let media = if file.media.is_empty() {
            "application/unknown"
        } else {
            &file.media
        };
        let typed = (id + 1, media, Action::None);
        let mut at = self.box_id(p, typed, (left, border_top + self.px(11.0)), clip, text);
        self.as_text(id + 1);
        at += self.px(4.0);
        let inner = (width - 2.0 * self.px(16.0)).max(1.0);
        let lines = self.file_lines(file, skip);
        let reason = skip.map(|_| lines.len() - 2);
        for (k, (s, ink, italic)) in lines.iter().enumerate() {
            if reason == Some(k) {
                let bar = rect(
                    left + self.px(16.0),
                    at + self.px(10.0),
                    inner,
                    self.px(1.0),
                );
                self.paint_clipped_box(p, bar, faded(self.colors[18]), 0.0, clip);
                at += self.px(19.0);
            }
            if s.is_empty() {
                at += self.px(8.0);
                continue;
            }
            let line = (id + 2 + k.min(997) as i32, s.as_str(), *ink, *italic);
            at += self.line(p, line, (left + self.px(16.0), at, inner), clip, text);
        }
        at += self.px(8.0);
        self.fit_box(p, outline, at - border_top);
        let border = rect(left, border_top, width, 0.0);
        self.box_title(p, (id, &file.name), Action::None, border, clip, text);
        self.as_text(id);
        at + self.px(8.0) - y
    }
    /// The importer's focus order: Select files, when it shows. Cancel and Import are the
    /// bottom bar's.
    pub(super) fn import_focus(&self) -> Vec<i32> {
        if self.shows_select() {
            vec![SELECT_ID]
        } else {
            vec![]
        }
    }
}
