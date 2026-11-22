//! SetupScreen (`scrollable = false`, no back arrow, no preview field): the first-run steps,
//! drawn by [`crate::settings_ui::setup`].
use super::{Kind, Row, Screen, row};
const ROWS: &[Row] = &[row("", "", "", Kind::Setup)];
pub const SCREEN: Screen = Screen {
    route: crate::settings_ui::setup::ROUTE,
    title: "setup__title",
    icon_space: false,
    preview: false,
    rows: ROWS,
};
