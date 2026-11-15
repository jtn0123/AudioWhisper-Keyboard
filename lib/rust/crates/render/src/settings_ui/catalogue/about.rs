//! AboutScreen (`iconSpaceReserved = true`, no preview field): the app icon header, the version
//! (copied on click), three links and the two licence screens.
use super::{Do, Kind, Row, Screen, row};
const ROWS: &[Row] = &[
    row("floris_app_name", "floris_app_name", "", Kind::Header),
    row(
        "about__version__title",
        "about__version__title",
        "",
        Kind::Effect(Do::Copy),
    )
    .icon("info"),
    row(
        "about__changelog__title",
        "about__changelog__title",
        "about__changelog__summary",
        Kind::Effect(Do::Url("florisboard__changelog_url")),
    )
    .icon("history"),
    row(
        "about__repository__title",
        "about__repository__title",
        "about__repository__summary",
        Kind::Effect(Do::Url("florisboard__repo_url")),
    )
    .icon("code"),
    row(
        "about__privacy_policy__title",
        "about__privacy_policy__title",
        "about__privacy_policy__summary",
        Kind::Effect(Do::Url("florisboard__privacy_policy_url")),
    )
    .icon("policy"),
    row(
        "about__project_license__title",
        "about__project_license__title",
        "about__project_license__summary",
        Kind::Route("settings/about/project-license"),
    )
    .icon("description")
    .arg("license_name", "Apache 2.0"),
    row(
        "about__third_party_licenses__title",
        "about__third_party_licenses__title",
        "about__third_party_licenses__summary",
        Kind::Route("settings/about/third-party-licenses"),
    )
    .icon("description"),
];
pub const SCREEN: Screen = Screen {
    route: "settings/about",
    title: "about__title",
    icon_space: true,
    preview: false,
    rows: ROWS,
};
