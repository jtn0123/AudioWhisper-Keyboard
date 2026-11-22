//! Which theme the settings make active: `ThemeManager.evaluateActiveThemeName`.

use crate::Text;

/// When the day theme shows and when the night theme does: `ThemeMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    AlwaysDay,
    AlwaysNight,
    FollowSystem,
    FollowTime,
}

impl Mode {
    /// The mode a `ThemeMode` entry's name names.
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "ALWAYS_DAY" => Self::AlwaysDay,
            "ALWAYS_NIGHT" => Self::AlwaysNight,
            "FOLLOW_SYSTEM" => Self::FollowSystem,
            "FOLLOW_TIME" => Self::FollowTime,
            _ => return None,
        })
    }
}

/// A theme's name: its extension's id and its component's id, `ExtensionComponentName`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Name {
    pub extension: Text,
    pub component: Text,
}

/// What the active theme is chosen from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub mode: Mode,
    pub day: Name,
    pub night: Name,
    /// Sunrise, sunset and now, in nanoseconds of the day.
    pub sunrise: i64,
    pub sunset: i64,
    pub now: i64,
    /// The system is in night mode.
    pub system_night: bool,
    /// The theme the theme manager previews, which wins over the settings.
    pub preview: Option<Name>,
}

impl Facts {
    /// The theme to show: the previewed one, else the day or night one as the mode chooses. In
    /// `FollowTime`, day is from sunrise to sunset, both included; a sunset before sunrise makes
    /// it always night.
    pub fn active(&self) -> &Name {
        if let Some(preview) = &self.preview {
            return preview;
        }
        let day = match self.mode {
            Mode::AlwaysDay => true,
            Mode::AlwaysNight => false,
            Mode::FollowSystem => !self.system_night,
            Mode::FollowTime => self.sunrise <= self.now && self.now <= self.sunset,
        };
        if day { &self.day } else { &self.night }
    }
}
