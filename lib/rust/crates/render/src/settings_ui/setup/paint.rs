//! FlorisStepLayout drawn in the screen's viewport, 16 dp in from each side: the intro text, then
//! each step's 56 dp header (a 40 dp numbered circle and a 32 dp title pill), the open step's
//! text and button beside a dashed line, then the privacy policy and repository buttons. The open
//! step fills what the viewport leaves, as its `weight(1f)` did; a taller one scrolls the screen.
use super::{Notice, SETUP_ID, STEPS, Step, Tap};
use crate::settings_ui::{
    Action, App,
    cards::{BODY_LARGE, plain},
    languages::form::Button,
    material::composite,
};
use crate::{
    panels::{Panel, rect},
    text::Text,
};
use keys::Rect;
use std::cmp::Ordering;

/// The intro text, a header (`HEADER_ID + number`), the open step's paragraphs
/// (`TEXT_ID + index`), its permission state, its button and the two footer buttons.
pub const INTRO_ID: i32 = SETUP_ID;
pub const HEADER_ID: i32 = SETUP_ID;
pub const TEXT_ID: i32 = SETUP_ID + 10;
pub const STATUS_ID: i32 = SETUP_ID + 15;
pub const ACTION_ID: i32 = SETUP_ID + 20;
pub const PRIVACY_ID: i32 = SETUP_ID + 21;
pub const REPOSITORY_ID: i32 = SETUP_ID + 22;

/// What the open step shows under its header.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub paragraphs: Vec<String>,
    /// The notification step's answer, once there is one.
    pub status: Option<String>,
    pub button: Option<String>,
}

impl App {
    /// Lays the whole screen out from `(x, y)` in the viewport `clip`; returns its height.
    pub(in crate::settings_ui) fn setup_screen(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let left = x + self.px(16.0);
        let width = self.config[0] - 2.0 * self.px(16.0);
        let intro = self.label("setup__intro_message");
        let ih = self.paragraph(p, INTRO_ID, &intro, (left, y, width), clip, text);
        let open = self.setup_step();
        let mut body = Some(self.step_body(open));
        let natural = body
            .as_ref()
            .map_or(0.0, |b| self.body_height(b, width - self.px(64.0), text));
        let fixed = ih + 2.0 * self.px(16.0) + 4.0 * self.px(56.0) + self.px(48.0);
        let fill = (clip.height() - fixed).max(natural);
        let mut top = y + ih + self.px(16.0);
        for step in STEPS {
            self.step_header(p, step, (left, top), width, clip, text);
            top += self.px(56.0);
            if let Some(b) = body.take_if(|_| step == open) {
                self.paint_body(p, open, b, rect(left, top, width, fill), clip, text);
                top += fill;
            }
        }
        top += self.px(16.0);
        self.setup_footer(p, (left, top), width, clip, text);
        top + self.px(48.0) - y
    }
    /// A justified StepText, drawn as plain bodyLarge text; its height.
    fn paragraph(
        &mut self,
        p: &mut Panel,
        id: i32,
        label: &str,
        (x, y, width): (f32, f32, f32),
        clip: Rect,
        text: &mut Text,
    ) -> f32 {
        let h = self
            .styled(text, label, BODY_LARGE, Some(width))
            .height
            .ceil();
        let at = rect(x, y, width, h);
        self.paint_styled(p, label, at, BODY_LARGE, self.colors[15], clip);
        let role = if id == STATUS_ID { "status" } else { "text" };
        self.push_control(plain(id, at, label.into(), role), clip);
        h
    }
    /// The open step's text, the notification answer and the button that does the step.
    pub fn step_body(&self, step: Step) -> Body {
        let (paragraphs, button): (&[&str], &str) = match step {
            Step::Enable => (
                &["setup__enable_ime__description"],
                "setup__enable_ime__open_settings_btn",
            ),
            Step::Select => (
                &["setup__select_ime__description"],
                "setup__select_ime__switch_keyboard_btn",
            ),
            Step::Notifications => (&["setup__grant_notification_permission__description"], ""),
            Step::Finish => (
                &[
                    "setup__finish_up__description_p1",
                    "setup__finish_up__description_p2",
                ],
                "setup__finish_up__finish_btn",
            ),
        };
        let mut body = Body {
            paragraphs: paragraphs.iter().map(|k| self.label(k)).collect(),
            status: None,
            button: (!button.is_empty()).then(|| self.label(button)),
        };
        if step == Step::Notifications {
            let (status, button) = match self.notice() {
                Notice::Unasked => ("", "setup__grant_notification_permission__btn"),
                Notice::Granted => ("setup__notifications__allowed", ""),
                Notice::Denied => (
                    "setup__notifications__off",
                    "setup__grant_notification_permission__btn",
                ),
                Notice::Blocked => (
                    "setup__notifications__off",
                    "setup__notifications__settings_btn",
                ),
            };
            body.status = (!status.is_empty()).then(|| self.label(status));
            body.button = (!button.is_empty()).then(|| self.label(button));
        }
        body
    }
    /// The body's own height: its text, the answer 8 dp under it, and the button's 16 dp gap
    /// and 48 dp touch height.
    fn body_height(&self, body: &Body, width: f32, text: &mut Text) -> f32 {
        let mut h = 0.0;
        for label in &body.paragraphs {
            h += self
                .styled(text, label, BODY_LARGE, Some(width))
                .height
                .ceil();
        }
        if let Some(status) = &body.status {
            h += self.px(8.0)
                + self
                    .styled(text, status, BODY_LARGE, Some(width))
                    .height
                    .ceil();
        }
        if body.button.is_some() {
            h += self.px(16.0) + self.px(48.0);
        }
        h
    }
    /// The open step's content in `area`: 56 dp in, 8 dp from the end, beside the dashed line
    /// FlorisStepLayout draws down the circle's centre (2 dp dashes, 10 dp gaps, 12 % ink).
    fn paint_body(
        &mut self,
        p: &mut Panel,
        step: Step,
        body: Body,
        area: Rect,
        clip: Rect,
        text: &mut Text,
    ) {
        let dash = composite(self.colors[1], 0.12, self.colors[0]);
        let mut at = area.top;
        while at < area.bottom {
            let b = rect(
                area.left + self.px(20.0),
                at,
                self.px(2.0),
                self.px(2.0).min(area.bottom - at),
            );
            self.paint_clipped_box(p, b, dash, 0.0, clip);
            at += self.px(12.0);
        }
        let x = area.left + self.px(56.0);
        let width = area.width() - self.px(64.0);
        let mut y = area.top;
        for (i, label) in body.paragraphs.iter().enumerate() {
            y += self.paragraph(p, TEXT_ID + i as i32, label, (x, y, width), clip, text);
        }
        if let Some(status) = &body.status {
            y += self.px(8.0);
            y += self.paragraph(p, STATUS_ID, status, (x, y, width), clip, text);
        }
        if let Some(label) = body.button {
            let button = Button {
                id: ACTION_ID,
                label,
                icon: "",
                filled: true,
                ink: self.colors[6],
                action: Action::Setup(Tap::Act(step)),
            };
            let bw = self.button_width(text, &button).min(width);
            let left = x + ((width - bw) / 2.0).round();
            let touch = rect(left, y + self.px(16.0), bw, self.px(48.0));
            self.material_button(p, button, touch, clip, text);
        }
    }
    /// A step's header: FlorisStepLayout's numbered circle and title pill, in its 56 dp row. Done
    /// steps open again by a tap; later ones are dimmed and do nothing.
    fn step_header(
        &mut self,
        p: &mut Panel,
        step: Step,
        (x, y): (f32, f32),
        width: f32,
        clip: Rect,
        text: &mut Text,
    ) {
        let (open, auto) = (self.setup_step(), self.auto_step());
        let id = HEADER_ID + step.number() as i32;
        let row = rect(x, y, width, self.px(56.0));
        if self.focus == id {
            self.paint_clipped_box(p, row, self.colors[4], self.px(8.0), clip);
        }
        let [fill, ink] = self.header_colors(step, open, auto);
        let d = self.px(40.0);
        let circle = rect(x, y + self.px(8.0), d, d);
        self.paint_clipped_box(p, circle, fill, d / 2.0, clip);
        self.centred_line(p, &step.number().to_string(), circle, ink, clip, text);
        let pill_left = x + d + self.px(16.0);
        let pill = rect(
            pill_left,
            y + self.px(12.0),
            x + width - pill_left,
            self.px(32.0),
        );
        self.paint_clipped_box(p, pill, fill, pill.height() / 2.0, clip);
        let inner = pill.width() - 2.0 * self.px(16.0);
        let title = self.row_text(text, &self.label(step.title()), self.sp(16), inner, 1);
        let inside = rect(pill.left + self.px(16.0), pill.top, inner, pill.height());
        self.centred_line(p, &title, inside, ink, clip, text);
        self.push_control(self.header_control(step, row, open, auto), clip);
    }
    /// The pill's fill and ink: primary for the open step, primary at 38 % for the others, and
    /// a later step's whole header at 38 % again.
    fn header_colors(&self, step: Step, open: Step, auto: Step) -> [u32; 2] {
        let [primary, ground, ink] = [self.colors[3], self.colors[0], self.colors[15]];
        if step == open {
            [primary, self.colors[6]]
        } else if step <= auto {
            [composite(primary, 0.38, ground), ink]
        } else {
            [
                composite(primary, 0.38 * 0.38, ground),
                composite(ink, 0.38, ground),
            ]
        }
    }
    /// What TalkBack reads for a header: "Step 2 of 4: …", whether it is done, open, next or
    /// still locked, and a button only where a tap does something.
    fn header_control(
        &self,
        step: Step,
        row: Rect,
        open: Step,
        auto: Step,
    ) -> crate::settings_ui::Control {
        let label = self
            .label("setup__step__label")
            .replace("{n}", &step.number().to_string())
            .replace("{count}", &STEPS.len().to_string())
            .replace("{title}", &self.label(step.title()));
        let mut c = plain(HEADER_ID + step.number() as i32, row, label, "button");
        let state = match step.cmp(&auto) {
            _ if step == open => "setup__step__current",
            Ordering::Less => "setup__step__done",
            Ordering::Equal => "setup__step__next",
            Ordering::Greater => "setup__step__later",
        };
        c.summary = self.label(state);
        if step == open {
            c.role = "heading";
        } else if step <= auto {
            c.action = Action::Setup(Tap::Header(step));
        } else {
            c.role = "text";
            c.enabled = false;
        }
        c
    }
    /// One line of bodyLarge text centred in `b`.
    fn centred_line(
        &self,
        p: &mut Panel,
        label: &str,
        b: Rect,
        ink: u32,
        clip: Rect,
        text: &mut Text,
    ) {
        let layout = self.styled(text, label, BODY_LARGE, None);
        let (w, h) = (layout.width.ceil().min(b.width()), layout.height.ceil());
        let at = rect(
            b.left + ((b.width() - w) / 2.0).round(),
            b.top + ((b.height() - h) / 2.0).round(),
            w + 1.0,
            h,
        );
        self.paint_styled(p, label, at, BODY_LARGE, ink, clip);
    }
    /// SetupScreen's footer: the privacy policy and repository text buttons with a bullet
    /// between them (FlorisBulletSpacer: a 4 dp outline-coloured dot 8 dp from each).
    fn setup_footer(
        &mut self,
        p: &mut Panel,
        (x, y): (f32, f32),
        width: f32,
        clip: Rect,
        text: &mut Text,
    ) {
        let buttons = [
            (PRIVACY_ID, "setup__footer__privacy_policy", Tap::Privacy),
            (REPOSITORY_ID, "setup__footer__repository", Tap::Repository),
        ]
        .map(|(id, key, tap)| Button {
            id,
            label: self.label(key),
            icon: "",
            filled: false,
            ink: self.colors[3],
            action: Action::Setup(tap),
        });
        let widths = buttons.each_ref().map(|b| self.button_width(text, b));
        let gap = self.px(20.0);
        let mut left = x + ((width - widths[0] - widths[1] - gap) / 2.0).round();
        let dot = self.px(4.0);
        for (i, (button, bw)) in buttons.into_iter().zip(widths).enumerate() {
            if i == 1 {
                let mid = rect(
                    left + self.px(8.0),
                    y + ((self.px(48.0) - dot) / 2.0).round(),
                    dot,
                    dot,
                );
                self.paint_clipped_box(p, mid, self.colors[7], dot / 2.0, clip);
                left += gap;
            }
            self.material_button(p, button, rect(left, y, bw, self.px(48.0)), clip, text);
            left += bw;
        }
    }
    /// The screen's focus order for a keyboard: the headers a tap opens, the open step's
    /// button, then the footer.
    pub fn setup_focus(&self) -> Vec<i32> {
        let (open, auto) = (self.setup_step(), self.auto_step());
        let mut ids = Vec::new();
        for step in STEPS {
            if step != open && step <= auto {
                ids.push(HEADER_ID + step.number() as i32);
            }
            if step == open && self.step_body(step).button.is_some() {
                ids.push(ACTION_ID);
            }
        }
        ids.extend([PRIVACY_ID, REPOSITORY_ID]);
        ids
    }
}
