//! ProjectLicenseScreen (`scrollable = false`, no preview field): the app's licence as one
//! selectable text that scrolls both ways, drawn by [`crate::settings_ui::project_licence`].
use super::{Kind, Row, Screen, row};
const ROWS: &[Row] = &[row("", "", "", Kind::ProjectLicence)];
pub const SCREEN: Screen = Screen {
    route: crate::settings_ui::project_licence::ROUTE,
    title: "about__project_license__title",
    icon_space: false,
    preview: false,
    rows: ROWS,
};
