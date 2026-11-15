//! DictationScreen (`iconSpaceReserved = true`, `previewFieldVisible = true`): ModelStatus's two
//! rows, the filler switch, and the custom vocabulary note and OutlinedTextField.
use super::{Kind, Row, Screen, heading, row, toggle};
const ROWS: &[Row] = &[
    heading("dictation__models__title"),
    // ModelStatus's Preferences have no onClick; their summaries come from the model facts.
    row(
        "dictation__models__speech",
        "dictation__models__speech",
        "",
        Kind::Status("speech"),
    ),
    row(
        "dictation__models__voice",
        "dictation__models__voice",
        "",
        Kind::Status("voice"),
    ),
    toggle(
        "dictation__remove_fillers",
        "pref__dictation__remove_fillers__label",
        "pref__dictation__remove_fillers__summary",
    ),
    heading("pref__dictation__vocabulary__label"),
    row(
        "pref__dictation__vocabulary__summary",
        "pref__dictation__vocabulary__summary",
        "",
        Kind::Note,
    ),
    // Saved on every change, as the screen's onValueChange does.
    row(
        "dictation__vocabulary",
        "pref__dictation__vocabulary__placeholder",
        "",
        Kind::Field,
    ),
];
pub const SCREEN: Screen = Screen {
    route: "settings/dictation",
    title: "settings__dictation__title",
    icon_space: true,
    preview: true,
    rows: ROWS,
};
