//! Settings screen descriptions, as data: each FlorisScreen's rows in source order with their
//! enabledIf/visibleIf conditions. Defaults, ranges and enum membership remain in settings.
mod clipboard;
mod gestures;
mod input_feedback;
mod keyboard;
mod media;
mod smartbar;
pub use keyboard::KEYBOARD;

/// A JetPref `enabledIf` / `visibleIf` lambda over stored preferences and device facts.
#[derive(Clone, Copy, Debug)]
pub enum Cond {
    Always,
    Never,
    Yes(&'static str),
    No(&'static str),
    Is(&'static str, &'static str),
    Not(&'static str, &'static str),
    Any(&'static [Cond]),
    All(&'static [Cond]),
    /// `vibrator != null && vibrator.hasVibrator()`.
    Vibrator,
    /// `vibrator.hasAmplitudeControl()`.
    Amplitude,
}
/// One `listPrefEntries` entry: stored value, label and description string keys.
pub type Entry = (&'static str, &'static str, &'static str);
#[derive(Clone, Copy, Debug)]
pub struct Choices {
    pub entries: &'static [Entry],
    /// `showDescriptionOnlyIfSelected`.
    pub selected_only: bool,
    /// A label template's `{emoji}` argument per entry (EmojiSkinTone), or empty.
    pub emoji: &'static [&'static str],
}
/// How a DialogSliderPreference's `valueLabel` shows a value.
#[derive(Clone, Copy, Debug)]
pub struct Unit {
    /// A `{v}` string key, a plurals key when `plural`, or empty for the bare number.
    pub key: &'static str,
    pub plural: bool,
    /// The label shown instead at value 0 (`general__auto`, `general__unlimited`), or empty.
    pub zero: &'static str,
}
#[derive(Clone, Copy, Debug)]
pub struct Slider {
    /// The secondary preference of a dual slider, or empty.
    pub second: &'static str,
    pub unit: Unit,
    /// `primaryLabel` / `secondaryLabel` string keys of a dual slider.
    pub labels: [&'static str; 2],
    /// The summary replaces the value when the vibrator facts are missing: 0 never, 1 without
    /// a vibrator, 2 also without amplitude control.
    pub hardware: u8,
    /// `onPreviewSelectedValue` vibrates with the selected value.
    pub preview: bool,
}
#[derive(Clone, Copy, Debug)]
pub enum Kind {
    Heading,
    Toggle,
    Choice(&'static Choices),
    Slider(&'static Slider),
    Route(&'static str),
    /// A plain Preference whose onClick asks to confirm an effect: (effect argument, message
    /// `{name}` argument string key).
    Confirm(&'static str, &'static str),
    /// A disabled Preference with a disabled, checked trailing switch.
    Fixed,
}
#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub key: &'static str,
    pub label: &'static str,
    pub summary: &'static str,
    pub kind: Kind,
    /// A ListPreference's `switchPref`, or empty.
    pub switch: &'static str,
    /// `summarySwitchDisabled`'s string key.
    pub off: &'static str,
    /// A leading icon name, or empty.
    pub icon: &'static str,
    pub enabled: Cond,
    pub visible: Cond,
}
pub(super) const fn row(
    key: &'static str,
    label: &'static str,
    summary: &'static str,
    kind: Kind,
) -> Row {
    Row {
        key,
        label,
        summary,
        kind,
        switch: "",
        off: "state__disabled",
        icon: "",
        enabled: Cond::Always,
        visible: Cond::Always,
    }
}
impl Row {
    pub(super) const fn on(mut self, enabled: Cond) -> Self {
        self.enabled = enabled;
        self
    }
    pub(super) const fn shown(mut self, visible: Cond) -> Self {
        self.visible = visible;
        self
    }
    pub(super) const fn linked(mut self, switch: &'static str, off: &'static str) -> Self {
        self.switch = switch;
        self.off = off;
        self
    }
    pub(super) const fn icon(mut self, icon: &'static str) -> Self {
        self.icon = icon;
        self
    }
}
pub(super) const fn toggle(key: &'static str, label: &'static str, summary: &'static str) -> Row {
    row(key, label, summary, Kind::Toggle)
}
pub(super) const fn heading(label: &'static str) -> Row {
    row(label, label, "", Kind::Heading)
}
pub(super) const fn list(key: &'static str, label: &'static str, choices: &'static Choices) -> Row {
    row(key, label, "", Kind::Choice(choices))
}
pub(super) const fn slider(key: &'static str, label: &'static str, s: &'static Slider) -> Row {
    row(key, label, "", Kind::Slider(s))
}
pub(super) const fn entries(entries: &'static [Entry]) -> Choices {
    Choices {
        entries,
        selected_only: false,
        emoji: &[],
    }
}
pub(super) const fn unit(key: &'static str) -> Unit {
    Unit {
        key,
        plural: false,
        zero: "",
    }
}
pub(super) const fn plural(key: &'static str, zero: &'static str) -> Unit {
    Unit {
        key,
        plural: true,
        zero,
    }
}
pub(super) const fn single(unit: Unit) -> Slider {
    Slider {
        second: "",
        unit,
        labels: ["", ""],
        hardware: 0,
        preview: false,
    }
}
pub(super) const fn dual(second: &'static str, unit: Unit, labels: [&'static str; 2]) -> Slider {
    Slider {
        second,
        unit,
        labels,
        hardware: 0,
        preview: false,
    }
}
/// A FlorisScreen: its route, title, `iconSpaceReserved` and content rows.
#[derive(Debug)]
pub struct Screen {
    pub route: &'static str,
    pub title: &'static str,
    pub icon_space: bool,
    pub rows: &'static [Row],
}
pub const HOME: &[(&str, &str)] = &[
    ("settings/localization", "settings__localization__title"),
    ("settings/theme", "settings__theme__title"),
    ("settings/keyboard", "settings__keyboard__title"),
    ("settings/smartbar", "settings__smartbar__title"),
    ("settings/dictation", "settings__dictation__title"),
    ("settings/typing", "settings__typing__title"),
    ("settings/gestures", "settings__gestures__title"),
    ("settings/clipboard", "settings__clipboard__title"),
    ("settings/media", "settings__media__title"),
    ("ext", "ext__home__title"),
    ("settings/other", "settings__other__title"),
    ("settings/about", "about__title"),
];
pub const HOME_ROUTE: &str = "settings/home";
pub const SCREENS: &[Screen] = &[
    keyboard::SCREEN,
    input_feedback::SCREEN,
    smartbar::SCREEN,
    gestures::SCREEN,
    clipboard::SCREEN,
    media::SCREEN,
];
/// The native screen at `route`, if Rust draws it.
pub fn screen(route: &str) -> Option<&'static Screen> {
    SCREENS.iter().find(|s| s.route == route)
}
/// Every route Rust draws: home, then [`SCREENS`].
pub fn native_routes() -> Vec<&'static str> {
    std::iter::once(HOME_ROUTE)
        .chain(SCREENS.iter().map(|s| s.route))
        .collect()
}
/// Whether `route` is drawn natively: home or one of [`SCREENS`].
pub fn native(route: &str) -> bool {
    route == HOME_ROUTE || screen(route).is_some()
}
