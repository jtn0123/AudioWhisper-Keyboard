//! ProjectLicenseScreen: the app's licence, `license/project_license.txt`, as one unwrapped
//! 10 sp monospace Text in a SelectionContainer that scrolls both ways, left to right whatever
//! the locale. Kotlin reads the asset off the main thread and answers with settings op 25; Rust
//! lays the text out, scrolls it, selects, draws the highlight and handles, places Android's
//! floating toolbar and answers TalkBack's selection and traversal. Only lines on screen are
//! drawn.
mod access;
pub mod bounds;
mod layout;
mod paint;
mod selection;
mod touch;
use super::App;
use crate::{compose_velocity::ComposeVelocity, panel_scroll::Fling};
use keys::Rect;
use std::sync::Arc;
use wire::settings_ui::text::{effect, node, outcome};

pub const ROUTE: &str = "settings/about/project-license";
/// The asset ProjectLicenseScreen reads with `FlorisRef.assets(...).loadTextAsset`.
pub const ASSET: &str = "license/project_license.txt";
/// The selectable text's node id.
pub const TEXT_ID: i32 = node::TEXT;
/// The horizontal scroller's node id; the vertical one is the screen's.
pub const SCROLL_ID: i32 = node::SCROLL;
/// What the screen shows when the asset cannot be read, `{error_message}` filled in.
pub const ERROR_KEY: &str = "about__project_license__error_license_text_failed";
/// The strings only this screen draws.
pub const KEYS: [&str; 1] = [ERROR_KEY];

#[derive(Debug, Default)]
pub struct State {
    /// The licence, or the error text when the asset could not be read; None until Kotlin
    /// answers, which leaves the screen empty for that moment.
    pub text: Option<Arc<str>>,
    /// An asset plan went out and has not been answered.
    pub requested: bool,
    /// The text's word segments, found once per text.
    pub(super) words: bounds::Words,
    /// The text laid out at one size.
    pub(super) laid: Option<layout::Laid>,
    /// The horizontal ScrollState's position and its end, in pixels.
    pub scroll_x: f32,
    pub max_x: f32,
    /// The scroll a saved state restored, vertical then horizontal, held until the text is laid
    /// out: before that the screen is empty and would clamp it to the top.
    pub(super) restored: Option<[f32; 2]>,
    /// The selection's anchor and focus as byte offsets into `text`. Equal ones are TalkBack's
    /// cursor and draw nothing.
    pub selection: Option<(usize, usize)>,
    pub(super) touch: touch::Touch,
    /// The horizontal fling and the velocity tracker it starts from.
    pub(super) fling: Option<Fling>,
    pub(super) velocity: ComposeVelocity,
    /// The text's viewport at the last frame.
    pub(super) viewport: Rect,
    /// The last traversal TalkBack asked for, as its event reports it: from and to in UTF-16
    /// units, the granularity and the action.
    pub traversed: Option<[i32; 4]>,
}
impl App {
    /// Whether the project licence is the screen on top, with no dialog over it.
    pub fn on_project_licence(&self) -> bool {
        self.modal.is_none() && self.stack.last().is_some_and(|r| r == ROUTE)
    }
    /// An asset Kotlin read for an asset plan (settings op 25): its path, the outcome, then the
    /// text or the exception's message.
    pub fn set_asset(&mut self, p: &[String]) -> Result<(), &'static str> {
        let [path, kind, rest @ ..] = p else {
            return Err("settings asset fields");
        };
        if path != ASSET || rest.len() > 1 {
            return Err("settings asset path");
        }
        let value = rest.first().map_or("", String::as_str);
        let shown = match kind.as_str() {
            outcome::OK => value.to_owned(),
            // stringRes(..., "error_message" to (it.message ?: "")).
            outcome::FAILED => self.label(ERROR_KEY).replace("{error_message}", value),
            _ => return Err("settings asset outcome"),
        };
        self.project_licence.words = bounds::Words::new(&shown);
        self.project_licence.text = Some(shown.into());
        self.project_licence.requested = false;
        self.project_licence.laid = None;
        self.project_licence.selection = None;
        self.project_licence.traversed = None;
        Ok(())
    }
    /// Opening the screen: a fresh horizontal ScrollState and no selection, as Compose composes
    /// the screen anew; a load lost with an earlier view is asked for again.
    pub(super) fn project_opened(&mut self, route: &str) {
        if route != ROUTE {
            return;
        }
        let words = std::mem::take(&mut self.project_licence.words);
        let text = self.project_licence.text.take();
        self.project_licence = State {
            words,
            text,
            ..State::default()
        };
        self.scrollbar_start[2] = -1;
        self.scrollbar_last[2] = -1;
    }
    /// The plan that asks Kotlin for the asset, once until it answers.
    pub(super) fn request_project(&mut self) {
        if self.project_licence.text.is_none() && !self.project_licence.requested {
            self.project_licence.requested = true;
            self.effects.push(vec![effect::ASSET.into(), ASSET.into()]);
        }
    }
    /// The saved state's part: the horizontal scroll, which `rememberScrollState` saves.
    pub fn project_saved(&self) -> String {
        self.project_licence.scroll_x.to_string()
    }
    /// Restores the horizontal scroll, and holds both scrolls until the text is laid out.
    pub fn project_restored(&mut self, horizontal: f32) {
        self.project_licence = State::default();
        if self.stack.last().is_some_and(|r| r == ROUTE) {
            self.project_licence.restored = Some([self.screen_scroll(), horizontal]);
        }
    }
    /// Whether the pending long press, a fling or an edge scroll needs frames.
    pub(super) fn project_animating(&self) -> bool {
        self.on_project_licence()
            && (self.project_licence.fling.is_some()
                || self.project_licence.touch.gesture == touch::Gesture::Pressed
                || self.edge_speed() != [0.0; 2])
    }
    /// Where the text's first line starts on screen: the viewport's corner, scrolled.
    pub(super) fn project_origin(&self) -> [f32; 2] {
        let v = self.project_licence.viewport;
        [
            v.left - super::shown(self.project_licence.scroll_x),
            v.top - super::shown(self.screen_scroll()),
        ]
    }
}
