//! The theme manager (`settings_ui/themes`): ThemeManagerScreen at its two routes, choosing the
//! day or the night theme. Its FlorisScreen shows the preview field, so the keyboard can be seen
//! in the theme being chosen. Rows whose kind is `Themes` are painted by that module.
use super::{Kind, Row, Screen, row};
use crate::settings_ui::themes as screens;

/// A theme manager's own parts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Part {
    /// The outlined box with Create and Import, as Gboard puts "My themes" and its + first.
    Actions,
    /// One outlined box per theme extension, its themes as radio rows.
    Extensions,
}

const ROWS: &[Row] = &[
    row("", "", "", Kind::Themes(Part::Actions)),
    row("", "", "", Kind::Themes(Part::Extensions)),
];
const fn screen(route: &'static str, title: &'static str) -> Screen {
    Screen {
        route,
        title,
        icon_space: false,
        preview: true,
        rows: ROWS,
    }
}
pub const DAY: Screen = screen(screens::DAY, "settings__theme_manager__title_day");
pub const NIGHT: Screen = screen(screens::NIGHT, "settings__theme_manager__title_night");
