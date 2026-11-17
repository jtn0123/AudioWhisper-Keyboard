//! Settings screen descriptions, as data: each FlorisScreen's rows in source order with their
//! enabledIf/visibleIf conditions. Defaults, ranges and enum membership remain in settings.
mod about;
mod clipboard;
mod dictation;
mod dictionary;
mod gestures;
mod input_feedback;
mod keyboard;
mod licences;
mod media;
mod physical_keyboard;
mod smartbar;
mod typing;
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
    /// A device fact Kotlin reports (settings op 20) has this value.
    Fact(&'static str, &'static str),
    /// The Floris spell checker is the system's enabled one (`florisSpellCheckerEnabled`).
    FlorisSpellChecker,
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
    /// A plain Preference whose onClick asks to confirm an effect.
    Confirm(&'static Ask),
    /// A disabled Preference with a disabled, checked trailing switch.
    Fixed,
    /// A Preference with no onClick: enabled styling, not clickable.
    Info,
    /// A Preference whose onClick is a platform effect.
    Effect(Do),
    /// An M3 Card of body text with a hyperlink under it.
    Card(&'static Card),
    /// SpellCheckerServiceSelector's error, warning or app card.
    Spell,
    /// AboutScreen's centred app icon and name.
    Header,
    /// ModelStatus's Preference for the "speech" or "voice" model: no onClick, its summary
    /// from the model facts.
    Status(&'static str),
    /// A bodyMedium Text in 16 dp by 8 dp padding.
    Note,
    /// An OutlinedTextField over the row's string preference, its label the placeholder.
    Field,
    /// AboutLibraries' LibrariesContainer: every library row, then its license dialog.
    Libraries,
    /// Jetpref's ColorPickerPreference with the advanced layout: a swatch at the end, and a
    /// dialog of presets or a hue, saturation and value picker (`picker.rs`).
    Color,
}
/// A JetPrefAlertDialog asking to confirm an effect.
#[derive(Clone, Copy, Debug)]
pub struct Ask {
    pub title: &'static str,
    pub message: &'static str,
    /// The message's `{name}` argument string key, or empty.
    pub name: &'static str,
    pub dismiss: &'static str,
    pub confirm: &'static str,
    /// The effect plan confirming returns; the toast's string key is appended when not empty.
    pub effect: &'static [&'static str],
    pub toast: &'static str,
}
/// A platform effect a row's onClick returns as a plan.
#[derive(Clone, Copy, Debug)]
pub enum Do {
    /// `launchUrl` of this string resource, `{version}` filled from the version fact.
    Url(&'static str),
    /// Copy the row's summary to the clipboard and toast the result.
    Copy,
    /// Start a system activity Kotlin knows by this name.
    Intent(&'static str),
}
/// Body text and a FlorisHyperlinkText to `url`, all string keys.
#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub text: &'static str,
    pub link: &'static str,
    pub url: &'static str,
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
    /// The summary's one template argument: its name and a literal value, or empty.
    pub arg: (&'static str, &'static str),
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
        arg: ("", ""),
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
    pub(super) const fn arg(mut self, name: &'static str, value: &'static str) -> Self {
        self.arg = (name, value);
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
/// A FlorisScreen: its route, title, `iconSpaceReserved`, `previewFieldVisible` and rows.
#[derive(Debug)]
pub struct Screen {
    pub route: &'static str,
    pub title: &'static str,
    pub icon_space: bool,
    pub preview: bool,
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
    typing::SCREEN,
    dictionary::SCREEN,
    physical_keyboard::SCREEN,
    about::SCREEN,
    dictation::SCREEN,
    licences::SCREEN,
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
