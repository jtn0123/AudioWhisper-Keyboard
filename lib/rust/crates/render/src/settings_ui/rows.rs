//! A screen's row model: JetPref's enabledIf/visibleIf, list entries and value labels.
use super::{
    App,
    catalogue::{Choices, Cond, Do, Kind, Row, Screen, Slider, Unit},
};
impl App {
    /// The native screen on top of the back stack; None on home.
    pub fn screen(&self) -> Option<&'static Screen> {
        self.stack.last().and_then(|r| super::catalogue::screen(r))
    }
    pub fn is_home(&self) -> bool {
        self.screen().is_none()
    }
    pub fn rows(&self) -> &'static [Row] {
        self.screen().map_or(&[], |s| s.rows)
    }
    pub fn cond(&self, c: &Cond) -> bool {
        match c {
            Cond::Always => true,
            Cond::Never => false,
            Cond::Yes(key) => self.yes(key),
            // JetPref's `isEqualTo false`: a stored boolean, not "anything but true".
            Cond::No(key) => self.value(key) == "false",
            Cond::Is(key, value) => self.value(key) == *value,
            Cond::Not(key, value) => self.value(key) != *value,
            Cond::Any(all) => all.iter().any(|c| self.cond(c)),
            Cond::All(all) => all.iter().all(|c| self.cond(c)),
            Cond::Vibrator => self.vibrator[0],
            Cond::Amplitude => self.vibrator[1],
            Cond::Fact(key, value) => self.fact(key) == *value,
            Cond::FlorisSpellChecker => {
                self.fact("spell_enabled") == "1"
                    && self.facts.contains_key("package")
                    && self.fact("spell_package") == self.fact("package")
            }
        }
    }
    /// `"${VERSION_NAME} (${VERSION_CODE})"`, as AboutScreen shows and copies it.
    pub fn version(&self) -> String {
        format!(
            "{} ({})",
            self.fact("version_name"),
            self.fact("version_code")
        )
    }
    pub fn visible(&self, i: usize) -> bool {
        self.rows().get(i).is_some_and(|r| self.cond(&r.visible))
    }
    pub fn enabled(&self, i: usize) -> bool {
        self.rows().get(i).is_some_and(|r| self.cond(&r.enabled))
    }
    fn list(&self, i: usize) -> Option<&'static Choices> {
        match self.rows().get(i)?.kind {
            Kind::Choice(c) => Some(c),
            _ => None,
        }
    }
    pub fn slider_spec(&self, i: usize) -> Option<&'static Slider> {
        match self.rows().get(i)?.kind {
            Kind::Slider(s) => Some(s),
            _ => None,
        }
    }
    /// The list's stored values, in entry order.
    pub fn choices(&self, i: usize) -> Vec<String> {
        self.list(i).map_or(vec![], |c| {
            c.entries.iter().map(|e| e.0.to_owned()).collect()
        })
    }
    /// An entry's label; JetPref shows "!! invalid !!" for a stored value with no entry.
    pub fn choice_label(&self, i: usize, choice: &str) -> String {
        let Some(c) = self.list(i) else {
            return String::new();
        };
        match c.entries.iter().position(|e| e.0 == choice) {
            Some(at) => {
                let label = self.label(c.entries[at].1);
                match c.emoji.get(at) {
                    Some(emoji) => label.replace("{emoji}", emoji),
                    None => label,
                }
            }
            None => "!! invalid !!".into(),
        }
    }
    /// The radio row's description: shown always, or only while selected.
    pub fn choice_description(&self, i: usize, choice: &str, selected: bool) -> String {
        let Some(c) = self.list(i) else {
            return String::new();
        };
        c.entries
            .iter()
            .find(|e| e.0 == choice && !e.2.is_empty() && (selected || !c.selected_only))
            .map_or(String::new(), |e| self.label(e.2))
    }
    /// The slider's `valueLabel` for a raw value. Plurals arrive resolved per value as
    /// `key#n`; a value outside the slider's range shows bare.
    pub fn value_label(&self, unit: &Unit, value: &str) -> String {
        if !unit.zero.is_empty() && value.parse::<f32>() == Ok(0.0) {
            self.label(unit.zero)
        } else if unit.key.is_empty() {
            value.into()
        } else if unit.plural {
            self.strings
                .get(&format!("{}#{value}", unit.key))
                .cloned()
                .unwrap_or_else(|| value.into())
        } else {
            self.label(unit.key).replace("{v}", value)
        }
    }
    /// The row's summary line, as its Preference composable words it.
    pub fn summary(&self, i: usize) -> String {
        let r = &self.rows()[i];
        match r.kind {
            Kind::Choice(_) if !r.switch.is_empty() && !self.yes(r.switch) => self.label(r.off),
            Kind::Choice(_) => self.choice_label(i, &self.value(r.key)),
            Kind::Slider(s) if s.hardware > 0 && !self.vibrator[0] => {
                self.label("pref__input_feedback__haptic_vibration_strength__summary_no_vibrator")
            }
            Kind::Slider(s) if s.hardware > 1 && !self.vibrator[1] => self.label(
                "pref__input_feedback__haptic_vibration_strength__summary_no_amplitude_ctrl",
            ),
            Kind::Slider(s) => {
                let first = self.value_label(&s.unit, &self.value(r.key));
                if s.second.is_empty() {
                    first
                } else {
                    format!(
                        "{first} / {}",
                        self.value_label(&s.unit, &self.value(s.second))
                    )
                }
            }
            Kind::Effect(Do::Copy) => self.version(),
            Kind::Status(model) => self.model_summary(model),
            Kind::Time => {
                let (hour, minute) = self.time_of(r.key);
                super::time::shown(hour, minute, self.is24())
            }
            _ if r.summary.is_empty() => String::new(),
            _ if r.arg.0.is_empty() => self.label(r.summary),
            _ => self
                .label(r.summary)
                .replace(&format!("{{{}}}", r.arg.0), r.arg.1),
        }
    }
    /// Every string resource key the native screens can show, for Kotlin to resolve; plurals
    /// come from [`App::plural_requests`] instead.
    pub fn string_keys() -> Vec<&'static str> {
        let mut out: Vec<&'static str> = CODE_KEYS.to_vec();
        out.extend(super::color_dialog::KEYS);
        out.extend(super::time_dialog::KEYS);
        out.extend(super::catalogue::HOME.iter().map(|h| h.1));
        for screen in super::catalogue::SCREENS {
            out.push(screen.title);
            for r in screen.rows {
                out.extend([r.label, r.summary]);
                if !r.switch.is_empty() {
                    out.push(r.off);
                }
                match r.kind {
                    Kind::Choice(c) => out.extend(c.entries.iter().flat_map(|e| [e.1, e.2])),
                    Kind::Slider(s) => {
                        if !s.unit.plural {
                            out.push(s.unit.key);
                        }
                        out.push(s.unit.zero);
                        out.extend(s.labels);
                    }
                    Kind::Confirm(ask) => out.extend([
                        ask.title,
                        ask.message,
                        ask.name,
                        ask.dismiss,
                        ask.confirm,
                        ask.toast,
                    ]),
                    Kind::Effect(Do::Url(key)) => out.push(key),
                    Kind::Card(card) => out.extend([card.text, card.link, card.url]),
                    Kind::Status(_) => out.extend(super::fields::MODEL_KEYS),
                    _ => {}
                }
            }
        }
        out.retain(|k| !k.is_empty());
        out.sort_unstable();
        out.dedup();
        out
    }
    /// Every plural a slider can show, as `key|min|max`, for Kotlin to resolve per value.
    pub fn plural_requests() -> Vec<String> {
        let mut out: Vec<String> = vec![];
        for screen in super::catalogue::SCREENS {
            for r in screen.rows {
                let Kind::Slider(s) = r.kind else { continue };
                if !s.unit.plural {
                    continue;
                }
                for key in [r.key, s.second].into_iter().filter(|k| !k.is_empty()) {
                    if let Ok(p) = settings::catalogue::get(&settings::text(key)) {
                        let request = format!("{}|{}|{}", s.unit.key, p.min, p.max);
                        if !out.contains(&request) {
                            out.push(request);
                        }
                    }
                }
            }
        }
        out
    }
}
/// Keys the settings code names directly rather than through a catalogue row.
pub const CODE_KEYS: &[&str] = &[
    "about__app_icon__description",
    "about__version_copied__error",
    "about__version_copied__title",
    "action__cancel",
    "action__default",
    "action__no",
    "action__ok",
    "action__reset_confirm_message",
    "action__reset_confirm_title",
    "action__yes",
    "pref__input_feedback__haptic_vibration_strength__summary_no_amplitude_ctrl",
    "pref__input_feedback__haptic_vibration_strength__summary_no_vibrator",
    "pref__spelling__active_spellchecker__summary_disabled",
    "pref__spelling__active_spellchecker__summary_none",
    "pref__spelling__active_spellchecker__unknown",
    "settings__home__ime_not_enabled",
    "settings__home__ime_not_selected",
    "settings__home__title",
    "settings__navigate_back",
    "settings__preview_keyboard",
    "settings__selection_end",
    "settings__selection_start",
    "setup__select_ime__switch_keyboard_btn",
    "state__disabled",
];
