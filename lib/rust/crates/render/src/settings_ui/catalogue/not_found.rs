//! The page a link to a route Rust does not draw opens (compose-removal step 12). Until then such
//! a link opened the Compose fallback activity, blank since step 10. The page does not repeat the
//! link's route: another app chose that text. Back returns to the screen under it.
use super::{HOME_ROUTE, Kind, Row, Screen, row};
pub const ROUTE: &str = "settings/not-found";
const ROWS: &[Row] = &[
    row(
        "settings__not_found__message",
        "settings__not_found__message",
        "",
        Kind::Note,
    ),
    row(
        "settings__not_found__home",
        "settings__not_found__home",
        "",
        Kind::Route(HOME_ROUTE),
    ),
];
pub const SCREEN: Screen = Screen {
    route: ROUTE,
    title: "settings__not_found__title",
    icon_space: false,
    preview: false,
    rows: ROWS,
};
