//! ThirdPartyLicensesScreen (`iconSpaceReserved = false`, no preview field): one LazyColumn of
//! the generated libraries, drawn by [`crate::settings_ui::licences`].
use super::{Kind, Row, Screen, row};
const ROWS: &[Row] = &[row("", "", "", Kind::Libraries)];
pub const SCREEN: Screen = Screen {
    route: crate::settings_ui::licences::ROUTE,
    title: "about__third_party_licenses__title",
    icon_space: false,
    preview: false,
    rows: ROWS,
};
