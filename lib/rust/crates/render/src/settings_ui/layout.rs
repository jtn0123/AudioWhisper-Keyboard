use super::{
    Action, App, Control,
    catalogue::{HOME, Kind},
};
use crate::{
    panels::{Panel, rect},
    scene::Scene,
    text::Text,
};
impl App {
    pub fn sp(&self, value: usize) -> f32 {
        let pixels = self.sp_pixels[value];
        if pixels > 0.0 {
            pixels
        } else {
            value as f32 * self.config[2] * self.config[3]
        }
    }
    /// Compose's `resolveLineHeightInPx`: under nonlinear font scaling a line height keeps its
    /// ratio to the font size (20/14 of the scaled 14 sp), not the separately scaled 20 sp.
    pub(super) fn line_height(&self, size: f32) -> f32 {
        for (font, line) in [(14, 20), (22, 28), (24, 32)] {
            if (size - self.sp(font)).abs() < 0.01 {
                return size * line as f32 / font as f32;
            }
        }
        0.0 // The app overrides bodyLarge with an unspecified lineHeight: use real font metrics.
    }
    /// Material 3 ListItem's `isSupportingMultilineHeuristic`: supporting text taller than
    /// `30.sp.roundToPx()` makes a three-line item (12 dp padding, 88 dp minimum).
    pub(super) fn multiline(&self, supporting: f32) -> bool {
        supporting.ceil() > self.sp(30).round()
    }
    /// Material 3 1.4.0's bodyMedium tracking (`TypeScaleTokens.BodyMediumTracking`, 0.2 sp, not
    /// the 0.25 sp of the published type scale) for 14 sp body text; the app's bodyLarge override
    /// has none. SettingsTypographyTokensTest pins both against the resolved library.
    pub(super) fn letter_spacing(&self, size: f32) -> f32 {
        if (size - self.sp(14)).abs() < 0.01 {
            0.2 * self.config[2] * self.config[3]
        } else {
            0.0
        }
    }
    /// Body text as Android lays it out, wrapped at `width`.
    pub(super) fn android(
        &self,
        text: &mut Text,
        label: &str,
        size: f32,
        width: Option<f32>,
    ) -> std::sync::Arc<crate::android_text::Layout> {
        text.android(
            label,
            size,
            400,
            [self.line_height(size), self.letter_spacing(size)],
            width.map(|w| w.max(1.0)),
        )
    }
    pub(super) fn measure(&self, text: &mut Text, label: &str, size: f32, width: f32) -> f32 {
        if label.is_empty() {
            return 0.0;
        }
        self.android(text, label, size, Some(width)).height
    }
    pub(super) fn row_text(
        &self,
        text: &mut Text,
        label: &str,
        size: f32,
        width: f32,
        max: usize,
    ) -> String {
        if self.config[3] > 1.0 || label.is_empty() {
            return label.into();
        }
        let layout = self.android(text, label, size, Some(width));
        if layout.lines.len() <= max {
            return label.into();
        }
        // StaticLayout's END ellipsis: the last line holds all the remaining text and keeps whole
        // clusters, spaces included, while their advances plus the ellipsis's fit the width.
        let start = layout.lines[max - 1].range.start;
        let spacing = self.letter_spacing(size) * crate::android_text::layout_size(size) / size;
        let ellipsis = self.android(text, "…", size, None).width + spacing;
        let rest = self.android(text, &label[start..], size, None);
        let carets = &rest.lines[0].carets;
        // Carets drop the first cluster's leading half spacing; Android's widths keep it.
        let cut = carets
            .windows(2)
            .find(|pair| pair[1].1 + spacing / 2.0 > width - ellipsis)
            .map_or(label.len(), |pair| start + pair[0].0);
        let mut fitted = label[..cut].to_owned();
        // Kerning against the ellipsis can tip the line over; drop clusters until it fits.
        loop {
            let candidate = format!("{fitted}…");
            if self
                .android(text, &candidate, size, Some(width))
                .lines
                .len()
                <= max
                || fitted.is_empty()
            {
                return candidate;
            }
            fitted.pop();
        }
    }
    /// A screen row's text insets, left and right: the icon space (56 dp, or 16 dp without
    /// `iconSpaceReserved`), and the trailing switch or linked-list switch with its divider.
    pub(super) fn row_insets(&self, i: usize) -> [f32; 2] {
        let d = self.config[2];
        let left = if self.screen().is_none_or(|s| s.icon_space) {
            56.0
        } else {
            16.0
        };
        let r = &self.rows()[i];
        let right = match r.kind {
            Kind::Choice(_) if !r.switch.is_empty() => 88.0,
            Kind::Toggle | Kind::Fixed => 80.0,
            _ => 16.0,
        };
        [left * d, right * d]
    }
    /// Rows whose trailing content is a Switch: a toggle, a linked list switch, the fixed row.
    pub(super) fn has_switch(&self, i: usize) -> bool {
        let r = &self.rows()[i];
        matches!(r.kind, Kind::Toggle | Kind::Fixed) || !r.switch.is_empty()
    }
    pub fn scene(&mut self, text: &mut Text) -> Scene {
        self.scene_pass(text, true)
    }
    fn scene_pass(&mut self, text: &mut Text, replay: bool) -> Scene {
        self.controls.clear();
        self.scrollbar_exists = [false; 2];
        self.reveal_target = None;
        let [w, h, d, _] = self.config;
        let old_scroll = self.scroll();
        // Paint the underlying screen from its own scroll and styling state.
        let modal = self.modal.take();
        let title = self.label(self.screen().map_or("settings__home__title", |s| s.title));
        let title_left = if self.is_home() { 16.0 * d } else { 56.0 * d };
        let header = (64.0 * d)
            .max(self.measure(text, &title, self.sp(22), w - title_left - 16.0 * d) + 16.0 * d);
        let footer = 56.0 * d;
        let mut p = Panel {
            kind: 9,
            density: d,
            ..Default::default()
        };
        let offset = self.back_progress * 32.0 * d;
        let viewport = rect(offset, header, w, h - header - footer);
        self.controls.push(Control {
            id: 4,
            bounds: viewport,
            label: title.clone(),
            summary: String::new(),
            role: "scroll",
            enabled: true,
            checked: None,
            selected: false,
            action: Action::None,
            range: None,
        });
        let mut y = header - self.shown_scroll();
        if self.is_home() {
            if !self.ime_enabled || !self.ime_selected {
                let label = self.label(if self.ime_enabled {
                    "settings__home__ime_not_selected"
                } else {
                    "settings__home__ime_not_enabled"
                });
                let rh = self.measure(text, &label, self.sp(14), w - 32.0 * d) + 32.0 * d;
                self.add(
                    &mut p,
                    Control {
                        id: 40,
                        bounds: rect(offset + 8.0 * d, y, w - 16.0 * d, rh),
                        label,
                        summary: String::new(),
                        role: "button",
                        enabled: true,
                        checked: None,
                        selected: false,
                        action: if self.ime_enabled {
                            Action::Picker
                        } else {
                            Action::Route("@enable".into())
                        },
                        range: None,
                    },
                    viewport,
                    text,
                );
                y += rh;
            }
            for (i, (route, label)) in HOME.iter().enumerate() {
                let label = self.label(label);
                // A one-line M3 ListItem: 8 dp vertical padding, 56 dp minimum.
                let rh = (self.measure(text, &label, self.sp(16), w - 72.0 * d) + 16.0 * d)
                    .max(56.0 * d);
                self.add(
                    &mut p,
                    Control {
                        id: 100 + i as i32,
                        bounds: rect(offset, y, w, rh),
                        label,
                        summary: String::new(),
                        role: "button",
                        enabled: true,
                        checked: None,
                        selected: false,
                        action: Action::Route((*route).into()),
                        range: None,
                    },
                    viewport,
                    text,
                );
                y += rh;
            }
        } else {
            for (i, r) in self.rows().iter().enumerate() {
                if !self.visible(i) {
                    continue;
                }
                let summary = self.summary(i);
                let label = self.label(r.label);
                // Match widgets::add's icon/text/switch bounds when measuring wrapped rows.
                let width = w - self.row_insets(i).iter().sum::<f32>();
                let drawn_label = self.row_text(text, &label, self.sp(16), width, 1);
                let drawn_summary = self.row_text(text, &summary, self.sp(14), width, 2);
                let rh = if matches!(r.kind, Kind::Heading) {
                    (self.measure(text, &drawn_label, self.sp(16), width) + 16.0 * d).max(56.0 * d)
                } else {
                    (self.measure(text, &drawn_label, self.sp(16), width)
                        + self.measure(text, &drawn_summary, self.sp(14), width)
                        + if self.multiline(self.measure(text, &drawn_summary, self.sp(14), width))
                        {
                            24.0
                        } else {
                            16.0
                        } * d)
                        .max(
                            if summary.is_empty() && self.has_switch(i) {
                                // ListItem's 8 dp padding around the Switch's 48 dp minimum
                                // interactive size, taller than a one-line item's 56 dp.
                                64.0
                            } else if summary.is_empty() {
                                56.0
                            } else if self.multiline(self.measure(
                                text,
                                &drawn_summary,
                                self.sp(14),
                                width,
                            )) {
                                88.0
                            } else {
                                72.0
                            } * d,
                        )
                };
                let role = match r.kind {
                    Kind::Heading => "heading",
                    Kind::Toggle => "toggle",
                    _ => "button",
                };
                let action = match r.kind {
                    Kind::Heading | Kind::Fixed => Action::None,
                    Kind::Route(route) => Action::Route(route.into()),
                    _ => Action::Row(i),
                };
                self.add(
                    &mut p,
                    Control {
                        id: 1000 + i as i32,
                        bounds: rect(offset, y, w, rh),
                        label,
                        summary,
                        role,
                        enabled: self.enabled(i),
                        checked: matches!(r.kind, Kind::Toggle).then(|| self.yes(r.key)),
                        selected: false,
                        action,
                        range: None,
                    },
                    viewport,
                    text,
                );
                if !r.switch.is_empty() {
                    self.add(
                        &mut p,
                        Control {
                            id: 2000 + i as i32,
                            bounds: rect(
                                offset + w - 72.0 * d,
                                y + if self.multiline(self.measure(
                                    text,
                                    &drawn_summary,
                                    self.sp(14),
                                    width,
                                )) {
                                    12.0 * d
                                } else {
                                    (rh - 48.0 * d) / 2.0
                                },
                                56.0 * d,
                                48.0 * d,
                            ),
                            label: self.label(r.label),
                            summary: String::new(),
                            role: "toggle",
                            enabled: self.enabled(i),
                            checked: Some(self.yes(r.switch)),
                            selected: false,
                            action: Action::Switch(i),
                            range: None,
                        },
                        viewport,
                        text,
                    );
                }
                y += rh;
            }
        }
        let content = y + self.shown_scroll() - header;
        self.max_scroll = (content - viewport.height()).max(0.0);
        self.stretch.extent = viewport.height();
        if self.modal.is_none() {
            self.set_scroll(self.scroll());
        }
        self.paint_scrollbar(&mut p, viewport);
        self.paint_box(&mut p, rect(0.0, 0.0, w, header), self.header_color(), 0.0);
        if !self.is_home() {
            self.add(
                &mut p,
                Control {
                    id: 1,
                    bounds: rect(0.0, 0.0, 56.0 * d, header),
                    label: self.label("settings__navigate_back"),
                    summary: String::new(),
                    role: "button",
                    enabled: true,
                    checked: None,
                    selected: false,
                    action: Action::Back,
                    range: None,
                },
                rect(0.0, 0.0, w, h),
                text,
            );
        }
        let left = if self.is_home() { 16.0 } else { 56.0 } * d;
        self.paint_text(
            &mut p,
            &title,
            rect(left, 18.0 * d, w - left - 16.0 * d, header - 18.0 * d),
            self.sp(22),
            self.colors[1],
            rect(0.0, 0.0, w, h),
        );
        self.paint_box(
            &mut p,
            rect(0.0, h - footer, w, footer),
            self.colors[5],
            0.0,
        );
        self.add(
            &mut p,
            Control {
                id: 2,
                bounds: rect(0.0, h - footer, w - 48.0 * d, footer),
                label: self.label("settings__preview_keyboard"),
                summary: String::new(),
                role: "edit",
                enabled: true,
                checked: None,
                selected: false,
                action: Action::Editor,
                range: None,
            },
            rect(0.0, 0.0, w, h),
            text,
        );
        self.add(
            &mut p,
            Control {
                id: 3,
                bounds: rect(w - 48.0 * d, h - footer, 48.0 * d, footer),
                label: self.label("setup__select_ime__switch_keyboard_btn"),
                summary: String::new(),
                role: "button",
                enabled: true,
                checked: None,
                selected: false,
                action: Action::Picker,
                range: None,
            },
            rect(0.0, 0.0, w, h),
            text,
        );
        self.modal = modal;
        if self.modal.is_some() {
            self.dialog(&mut p, text);
        }
        if let Some(value) = self.reveal_target.take() {
            self.set_scroll(value);
        }
        self.reveal_focus = false;
        if replay && self.scroll() != old_scroll {
            return self.scene_pass(text, false);
        }
        self.controls
            .retain(|c| c.bounds.width() > 0.0 && c.bounds.height() > 0.0);
        self.controls
            .sort_by_key(|c| matches!(c.action, Action::SelectionHandle(_)));
        Scene {
            width: w as u16,
            height: h as u16,
            background: self.colors[0],
            panel: Some(p),
            ..Default::default()
        }
    }
}
