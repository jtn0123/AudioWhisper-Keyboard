//! Rows that are not ListItems: AboutScreen's app icon header, the auto-space feedback Card and
//! SpellCheckerServiceSelector's cards (DictationScreen's note and field are in `fields.rs`).
//! Compose lays these out in whole pixels: each dp padding rounds on its own, a Text is `ceil`'d
//! wide and tall, and centring offsets round.
use super::{
    Action, App, Control,
    catalogue::{Card, Kind},
};
use crate::{
    panels::{Command, Item, Panel, rect},
    text::Text,
};
use keys::Rect;

/// A Material text style: size and line height in sp (0: font metrics), tracking in sp, weight.
#[derive(Clone, Copy)]
pub(super) struct Type {
    size: usize,
    line: f32,
    tracking: f32,
    weight: i32,
}
/// The app's bodyLarge override: 16 sp, no line height, no tracking.
const BODY_LARGE: Type = Type {
    size: 16,
    line: 0.0,
    tracking: 0.0,
    weight: 400,
};
/// Material 3's titleMedium and titleSmall, which FlorisSimpleCard uses, as the resolved library
/// has them (`TypeScaleTokens`: 0.2 sp titleMedium tracking, not the published 0.15 sp).
const TITLE_MEDIUM: Type = Type {
    size: 16,
    line: 24.0,
    tracking: 0.2,
    weight: 500,
};
pub(super) const TITLE_SMALL: Type = Type {
    size: 14,
    line: 20.0,
    tracking: 0.1,
    weight: 500,
};
/// bodySmall and labelLarge: AddonManagementReferenceBox's note and its TextButton.
pub(super) const BODY_SMALL: Type = Type {
    size: 12,
    line: 16.0,
    tracking: 0.4,
    weight: 400,
};
pub(super) const LABEL_LARGE: Type = Type {
    size: 14,
    line: 20.0,
    tracking: 0.1,
    weight: 500,
};
/// AboutScreen's app name: bodyLarge at 24 sp, SemiBold.
const APP_NAME: Type = Type {
    size: 24,
    line: 0.0,
    tracking: 0.0,
    weight: 600,
};
/// FlorisErrorCard and FlorisWarningCard paint fixed colours, not the theme's.
const ERROR: [u32; 2] = [0xffff0000, 0xffffffff];
const WARNING: [u32; 2] = [0xffffff00, 0xff000000];

/// Which of SpellCheckerServiceSelector's cards shows, from the system's settings.
#[derive(Clone, Debug, PartialEq)]
pub enum SpellCard {
    /// `spell_checker_enabled` is not "1": FlorisErrorCard.
    Disabled,
    /// No `selected_spell_checker`: FlorisWarningCard.
    None,
    /// The selected service's app: its label ("Unknown" when the lookup failed) and package.
    App(String, String),
}

impl App {
    fn metrics(&self, t: Type) -> [f32; 2] {
        let size = self.sp(t.size);
        let line = if t.line > 0.0 {
            // `resolveLineHeightInPx`: an sp line height keeps its ratio to the scaled size.
            size * t.line / t.size as f32
        } else {
            0.0
        };
        [line, t.tracking * self.config[2] * self.config[3]]
    }
    pub(super) fn styled(
        &self,
        text: &mut Text,
        label: &str,
        t: Type,
        width: Option<f32>,
    ) -> std::sync::Arc<crate::android_text::Layout> {
        text.android(
            label,
            self.sp(t.size),
            t.weight,
            self.metrics(t),
            width.map(|w| w.max(1.0)),
        )
    }
    pub(super) fn paint_styled(
        &self,
        p: &mut Panel,
        label: &str,
        b: Rect,
        t: Type,
        color: u32,
        clip: Rect,
    ) {
        self.paint_text(p, label, b, self.sp(t.size), color, clip);
        let [line, tracking] = self.metrics(t);
        let item = p.items.last_mut().expect("paint_text pushed an item");
        item.text.line_height = line;
        item.text.letter_spacing = tracking;
        item.text.weight = t.weight;
    }
    /// A picture Kotlin reported, scaled into `b` as `Image` does with `ContentScale.Fit`.
    pub(super) fn paint_image(&self, p: &mut Panel, name: &str, b: Rect, clip: Rect) -> bool {
        let Some(frame) = self.images.get(name) else {
            return false;
        };
        let id = p.media.entries.len() as i64 + 1;
        if !p.media.upload(id, 0, frame.clone(), true) {
            return false;
        }
        let mut item = Item::new(
            -10000 - p.items.len() as i32,
            b,
            String::new(),
            "settings-image",
            Command::None,
        );
        item.media = Some(id);
        item.clip = clip;
        p.items.push(item);
        true
    }
    pub fn spell_card(&self) -> SpellCard {
        if self.fact("spell_enabled") != "1" {
            SpellCard::Disabled
        } else if !self.facts.contains_key("spell_id") {
            SpellCard::None
        } else {
            let label = self
                .facts
                .get("spell_label")
                .cloned()
                .unwrap_or_else(|| self.label("pref__spelling__active_spellchecker__unknown"));
            SpellCard::App(label, self.fact("spell_package").to_owned())
        }
    }
    /// Lays out and paints row `i` when it is not a ListItem; returns its height.
    pub(super) fn card_row(
        &mut self,
        p: &mut Panel,
        i: usize,
        at: (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> Option<f32> {
        match self.rows()[i].kind {
            Kind::Header => Some(self.header(p, i, at, clip, text)),
            Kind::Card(card) => Some(self.feedback(p, i, card, at, clip, text)),
            Kind::Spell => Some(self.spell(p, i, at, clip, text)),
            Kind::Note => Some(self.note(p, i, at, clip, text)),
            Kind::Field => Some(self.text_field(p, i, at, clip, text)),
            Kind::Libraries => Some(self.libraries(p, at, clip, text)),
            Kind::Addons(a) => Some(self.addons(p, i, a, at, clip, text)),
            _ => None,
        }
    }
    /// Records a control these rows painted themselves, revealing it when focus moved to it.
    pub(super) fn push_control(&mut self, mut c: Control, clip: Rect) {
        let b = c.bounds;
        if self.reveal_focus && c.id == self.focus {
            if b.top < clip.top {
                self.reveal_target = Some(self.shown_scroll() + b.top - clip.top);
            } else if b.bottom > clip.bottom {
                self.reveal_target = Some(self.shown_scroll() + b.bottom - clip.bottom);
            }
        }
        c.bounds = crate::panel_layout::intersection(b, clip);
        self.controls.push(c);
    }
    /// The keyboard-focus highlight `add` paints, under a clickable card or link.
    fn focus_box(&self, p: &mut Panel, id: i32, b: Rect, clip: Rect) {
        if id == self.focus {
            self.paint_clipped_box(p, b, self.colors[4], 8.0 * self.config[2], clip);
        }
    }
    fn header(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0];
        let icon = self.px(64.0);
        let name = self.label(self.rows()[i].label);
        let layout = self.styled(text, &name, APP_NAME, Some(w));
        let (tw, th) = (layout.width.ceil().min(w), layout.height.ceil());
        let top = y + self.px(24.0);
        let image = rect(x + ((w - icon) / 2.0).round(), top, icon, icon);
        self.paint_image(p, "app_icon", image, clip);
        let name_box = rect(
            x + ((w - tw) / 2.0).round(),
            top + icon + self.px(16.0),
            tw,
            th,
        );
        self.paint_styled(p, &name, name_box, APP_NAME, self.colors[1], clip);
        let description = self.label("about__app_icon__description");
        self.push_control(plain(6000 + i as i32, image, description, "image"), clip);
        self.push_control(plain(1000 + i as i32, name_box, name, "text"), clip);
        self.px(24.0) + icon + self.px(16.0) + th + self.px(32.0)
    }
    fn feedback(
        &mut self,
        p: &mut Panel,
        i: usize,
        card: &Card,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0];
        let pad = self.px(8.0);
        let inner = w - 4.0 * pad;
        let body = self.label(card.text);
        let bh = self
            .styled(text, &body, BODY_LARGE, Some(inner))
            .height
            .ceil();
        let link = self.row_text(text, &self.label(card.link), self.sp(16), inner, 1);
        let layout = self.styled(text, &link, BODY_LARGE, Some(inner));
        let (lw, lh) = (layout.width.ceil().min(inner), layout.height.ceil());
        let b = rect(x + pad, y + pad, w - 2.0 * pad, 2.0 * pad + bh + lh);
        // M3 Card: surfaceContainerHighest and the medium (12 dp) shape. Its content colour is
        // `contentColorFor(surfaceContainerHighest)`, which is onSurfaceVariant whenever the
        // scheme's surfaceVariant is the same colour, as materialkolor's 2025 spec makes it.
        self.paint_clipped_box(p, b, self.colors[5], 12.0 * self.config[2], clip);
        let body_box = rect(b.left + pad, b.top + pad, inner, bh);
        self.paint_styled(p, &body, body_box, BODY_LARGE, self.colors[13], clip);
        let link_box = rect(b.left + pad, b.top + pad + bh, lw, lh);
        self.focus_box(p, 6000 + i as i32, link_box, clip);
        self.paint_styled(p, &link, link_box, BODY_LARGE, self.colors[3], clip);
        self.push_control(plain(1000 + i as i32, body_box, body, "text"), clip);
        let mut open = plain(6000 + i as i32, link_box, link, "link");
        open.action = Action::Row(i);
        self.push_control(open, clip);
        b.height() + 2.0 * pad
    }
    fn spell(
        &mut self,
        p: &mut Panel,
        i: usize,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let w = self.config[0];
        let d = self.config[2];
        let card = rect(x + self.px(8.0), y, w - 2.0 * self.px(8.0), 0.0);
        let (label, secondary, colors, icon) = match self.spell_card() {
            SpellCard::Disabled => (
                self.label("pref__spelling__active_spellchecker__summary_disabled"),
                String::new(),
                ERROR,
                "error_outline",
            ),
            SpellCard::None => (
                self.label("pref__spelling__active_spellchecker__summary_none"),
                String::new(),
                WARNING,
                "warning",
            ),
            SpellCard::App(label, package) => {
                (label, package, [self.colors[12], self.colors[1]], "")
            }
        };
        let simple = matches!(self.spell_card(), SpellCard::App(..));
        // FlorisSimpleCard's Row: padding (start 0, end 16, top and bottom 8) around the error
        // and warning cards' icon, a 24 dp icon padded 12 dp all round; 8 dp all round and a
        // 32 dp icon padded 8 dp at the end otherwise. Compose rounds each dp to whole pixels,
        // so the padded icon is 32 + 63 + 32 = 127 px at 2.625, not 48 dp's 126.
        let [start, end, vertical] = if simple {
            [self.px(8.0); 3]
        } else {
            [0.0, self.px(16.0), self.px(8.0)]
        };
        let padded = 2.0 * self.px(12.0) + self.px(24.0);
        let icon_box = if simple {
            self.px(32.0) + self.px(8.0)
        } else {
            padded
        };
        let column = card.width() - start - end - icon_box;
        let th = self
            .styled(text, &label, TITLE_MEDIUM, Some(column))
            .height
            .ceil();
        let sh = if secondary.is_empty() {
            0.0
        } else {
            self.styled(text, &secondary, TITLE_SMALL, Some(column))
                .height
                .ceil()
        };
        let icon_height = if simple { self.px(32.0) } else { padded };
        let row = icon_height.max(th + sh);
        let b = rect(
            card.left,
            card.top,
            card.width(),
            (2.0 * vertical + row).max(self.px(48.0)),
        );
        self.paint_clipped_box(p, b, colors[0], 12.0 * d, clip);
        self.focus_box(p, 1000 + i as i32, b, clip);
        let row_top = b.top + vertical;
        let icon_top = row_top + ((row - icon_height) / 2.0).round();
        if simple {
            let image = rect(b.left + start, icon_top, self.px(32.0), self.px(32.0));
            if !self.paint_image(p, "spell_icon", image, clip) {
                self.paint_icon(p, "help_outline", image, colors[1], clip);
            }
        } else {
            let at = rect(
                b.left + self.px(12.0),
                icon_top + self.px(12.0),
                self.px(24.0),
                self.px(24.0),
            );
            self.paint_icon(p, icon, at, colors[1], clip);
        }
        let left = b.left + start + icon_box;
        let text_top = row_top + ((row - th - sh) / 2.0).round();
        self.paint_styled(
            p,
            &label,
            rect(left, text_top, column, th),
            TITLE_MEDIUM,
            colors[1],
            clip,
        );
        if !secondary.is_empty() {
            let at = rect(left, text_top + th, column, sh);
            self.paint_styled(p, &secondary, at, TITLE_SMALL, colors[1], clip);
        }
        let mut c = plain(1000 + i as i32, b, label, "card");
        c.summary = secondary;
        c.action = Action::Row(i);
        self.push_control(c, clip);
        b.height()
    }
}

/// An enabled control with no state: text, an image, a link or a card.
pub(super) fn plain(id: i32, bounds: Rect, label: String, role: &'static str) -> Control {
    Control {
        id,
        bounds,
        label,
        summary: String::new(),
        role,
        enabled: true,
        checked: None,
        selected: false,
        action: Action::None,
        range: None,
    }
}
