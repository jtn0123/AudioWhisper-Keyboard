//! SetupScreen: the first-run steps (enable the keyboard, select it, answer the notification
//! permission, finish), as FlorisStepLayout showed them. Rust picks the first screen, works out
//! each step's state from the facts Kotlin reports, decides when to ask for the permission and what
//! an answer means, and finishes setup. Kotlin opens the system's screens, asks Android for the
//! permission and brings the app back to the front (`wire::settings_ui::setup`).
pub mod paint;
use super::{App, catalogue};
use wire::settings_ui::setup::{effect, fact};

pub const ROUTE: &str = "setup";
/// Whether setup was finished: `prefs.internal.isImeSetUp`.
pub const SET_UP: &str = "internal__is_ime_set_up";
/// The notification question's answer: `NOT_SET`, `GRANTED` or `DENIED`.
pub const NOTIFICATIONS: &str = "internal__notification_permission_state";
/// Each step's control ids start here; see `paint.rs`.
pub const SETUP_ID: i32 = 7600;
/// The strings only this screen draws: Compose's, then the step and permission states.
pub const KEYS: [&str; 26] = [
    "setup__intro_message",
    "setup__footer__privacy_policy",
    "setup__footer__repository",
    "setup__enable_ime__title",
    "setup__enable_ime__description",
    "setup__enable_ime__open_settings_btn",
    "setup__select_ime__title",
    "setup__select_ime__description",
    "setup__select_ime__switch_keyboard_btn",
    "setup__grant_notification_permission__title",
    "setup__grant_notification_permission__description",
    "setup__grant_notification_permission__btn",
    "setup__finish_up__title",
    "setup__finish_up__description_p1",
    "setup__finish_up__description_p2",
    "setup__finish_up__finish_btn",
    "florisboard__privacy_policy_url",
    "florisboard__repo_url",
    "setup__step__label",
    "setup__step__done",
    "setup__step__current",
    "setup__step__next",
    "setup__step__later",
    "setup__notifications__allowed",
    "setup__notifications__off",
    "setup__notifications__settings_btn",
];

/// FlorisStepLayout's steps, in order; a later step opens only once the earlier ones are done.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Step {
    Enable,
    Select,
    Notifications,
    Finish,
}
pub const STEPS: [Step; 4] = [
    Step::Enable,
    Step::Select,
    Step::Notifications,
    Step::Finish,
];
impl Step {
    /// The number its header shows, from 1.
    pub fn number(self) -> usize {
        self as usize + 1
    }
    pub fn title(self) -> &'static str {
        match self {
            Step::Enable => "setup__enable_ime__title",
            Step::Select => "setup__select_ime__title",
            Step::Notifications => "setup__grant_notification_permission__title",
            Step::Finish => "setup__finish_up__title",
        }
    }
}

/// What the notification step knows, from the stored answer and the permission's state now.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Notice {
    /// Never answered: the step is still to do.
    Unasked,
    Granted,
    /// Denied, and Android would ask again.
    Denied,
    /// Denied for good: Android shows no dialog, so the step opens the notification settings.
    Blocked,
}

/// A tap on the setup screen's own controls.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tap {
    /// A done step's header (or the first undone one's, while an earlier step is open): open it.
    Header(Step),
    /// The open step's button.
    Act(Step),
    Privacy,
    Repository,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    /// The step the user opened by its header (FlorisStepState's `currentManual`); it shows while
    /// it is not past the first undone step.
    pub manual: Option<Step>,
    /// The step whose system screen the user was sent to, and whether it was the first undone
    /// step then. Cleared by the next tap, or when the user comes back with it done.
    pub away: Option<(Step, bool)>,
}

impl App {
    /// Whether setup is the screen on top.
    pub fn on_setup(&self) -> bool {
        self.stack.last().is_some_and(|r| r == ROUTE)
    }
    /// Settings op 40: the activity opened fresh, with the route a settings link named, if any.
    /// Setup shows until it is finished, unless a link asks for a screen, as FlorisAppActivity
    /// decided. A notification question never answered sends a finished user through setup again.
    pub fn start(&mut self, link: Option<&str>) {
        if self.value(NOTIFICATIONS) == "NOT_SET" && self.yes(SET_UP) {
            self.write(SET_UP, "false".into());
        }
        match link {
            Some(route) => self.navigate(route),
            None if !self.yes(SET_UP) => self.navigate(ROUTE),
            None => {}
        }
        // Facts reported before the first screen was chosen count as a later report would.
        self.setup_facts();
    }
    /// The notification step's state. A permission granted now counts whatever was stored; one
    /// revoked since it was granted reads as denied.
    pub fn notice(&self) -> Notice {
        let granted = match self.fact(fact::NOTIFICATIONS) {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        };
        match (granted, self.value(NOTIFICATIONS).as_str()) {
            (Some(true), _) | (None, "GRANTED") => Notice::Granted,
            (_, "NOT_SET") => Notice::Unasked,
            (_, "DENIED") if self.fact(fact::RATIONALE) != "true" => Notice::Blocked,
            _ => Notice::Denied,
        }
    }
    /// The first step not done: FlorisStepState's `currentAuto`.
    pub fn auto_step(&self) -> Step {
        if !self.ime_enabled {
            Step::Enable
        } else if !self.ime_selected {
            Step::Select
        } else if self.notice() == Notice::Unasked {
            Step::Notifications
        } else {
            Step::Finish
        }
    }
    /// The open step: the one the user chose while it is not past the first undone one.
    pub fn setup_step(&self) -> Step {
        let auto = self.auto_step();
        self.setup.manual.filter(|m| *m <= auto).unwrap_or(auto)
    }
    /// Whether the step is what the user went to the system's screens for: the keyboard enabled,
    /// selected, or notifications allowed.
    fn satisfied(&self, step: Step) -> bool {
        match step {
            Step::Enable => self.ime_enabled,
            Step::Select => self.ime_selected,
            Step::Notifications => self.notice() == Notice::Granted,
            Step::Finish => false,
        }
    }
    /// Back from the system's screens with the step done: the next undone step opens, even when
    /// the user had reopened this one.
    fn settle_setup(&mut self) {
        if let Some((step, was_first)) = self.setup.away
            && (self.satisfied(step) || (was_first && step < self.auto_step()))
        {
            self.setup.away = None;
            self.setup.manual = None;
        }
    }
    /// The input method facts in CONFIG. The keyboard enabled while the user is in the system's
    /// settings for it brings the app back, as Compose's setup did.
    pub fn set_ime(&mut self, enabled: bool, selected: bool) {
        let returning = self.on_setup()
            && enabled
            && !self.ime_enabled
            && matches!(self.setup.away, Some((Step::Enable, _)));
        self.ime_enabled = enabled;
        self.ime_selected = selected;
        if returning {
            self.effects.push(vec![effect::RETURN.into()]);
        }
        self.settle_setup();
    }
    /// After a FACTS report: a permission granted outside setup answers its question.
    pub fn setup_facts(&mut self) {
        if self.on_setup()
            && self.fact(fact::NOTIFICATIONS) == "true"
            && self.value(NOTIFICATIONS) != "GRANTED"
        {
            self.write(NOTIFICATIONS, "GRANTED".into());
        }
        self.settle_setup();
    }
    /// Settings op 41: how the permission request ended. The answer is stored as Compose stored
    /// it; whether Android would ask again decides what the step's button does next.
    pub fn permission_answer(&mut self, p: &[String]) -> Result<(), &'static str> {
        let [granted, rationale] = p else {
            return Err("settings permission fields");
        };
        let flag = |s: &str| match s {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err("settings permission answer"),
        };
        let (granted, rationale) = (flag(granted)?, flag(rationale)?);
        self.facts
            .insert(fact::NOTIFICATIONS.into(), granted.to_string());
        self.facts
            .insert(fact::RATIONALE.into(), rationale.to_string());
        let answer = if granted { "GRANTED" } else { "DENIED" };
        self.write(NOTIFICATIONS, answer.into());
        self.settle_setup();
        Ok(())
    }
    pub(super) fn setup_tap(&mut self, tap: Tap) {
        self.setup.away = None;
        match tap {
            // FlorisStepState.setCurrentManual: the first undone step is the automatic one.
            Tap::Header(step) => {
                self.setup.manual = (step != self.auto_step()).then_some(step);
            }
            Tap::Act(step) => self.setup_act(step),
            Tap::Privacy => self.open_url("florisboard__privacy_policy_url"),
            Tap::Repository => self.open_url("florisboard__repo_url"),
        }
    }
    fn open_url(&mut self, key: &str) {
        let url = self.label(key);
        self.effects.push(vec!["url".into(), url]);
    }
    fn setup_act(&mut self, step: Step) {
        let plan = match step {
            Step::Enable => "enable",
            Step::Select => "picker",
            Step::Notifications if self.notice() == Notice::Blocked => {
                effect::NOTIFICATION_SETTINGS
            }
            Step::Notifications => effect::ASK,
            Step::Finish => {
                self.write(SET_UP, "true".into());
                self.navigate(catalogue::HOME_ROUTE);
                return;
            }
        };
        self.setup.away = Some((step, step == self.auto_step()));
        self.effects.push(vec![plan.into()]);
    }
    /// Opening setup starts with no step chosen, as a new composition of the screen does.
    pub(super) fn setup_opened(&mut self, route: &str) {
        if route == ROUTE {
            self.setup = State::default();
        }
    }
    /// The saved state's part: the chosen step and the step the user went away for, as numbers
    /// (0 for none) and whether that was the first undone step.
    pub fn setup_saved(&self) -> String {
        let number = |s: Option<Step>| s.map_or(0, Step::number);
        let (away, first) = self.setup.away.unzip();
        format!(
            "{}|{}|{}",
            number(self.setup.manual),
            number(away),
            first.unwrap_or(false)
        )
    }
    /// Parses [`App::setup_saved`]'s text.
    pub fn parse_setup(text: &str) -> Result<State, &'static str> {
        let step = |s: &str| match s.parse::<usize>() {
            Ok(0) => Ok(None),
            Ok(n) if n <= STEPS.len() => Ok(Some(STEPS[n - 1])),
            _ => Err("settings saved setup step"),
        };
        let [manual, away, first] = text.split('|').collect::<Vec<_>>()[..] else {
            return Err("settings saved setup");
        };
        let first = first.parse().map_err(|_| "settings saved setup flag")?;
        Ok(State {
            manual: step(manual)?,
            away: step(away)?.map(|s| (s, first)),
        })
    }
}
