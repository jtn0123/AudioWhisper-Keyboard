//! BackupScreen and RestoreScreen, which Compose drew in outlined boxes over a bottom bar of
//! Cancel and Back up or Restore. Here the boxes' titles are headings and the bar stays.
use super::{Cond, Kind, Part, Row, Screen, heading, row};

pub const BACKUP_ROUTE: &str = "settings/other/backup";
pub const RESTORE_ROUTE: &str = "settings/other/restore";

const fn part(label: &'static str, part: Part) -> Row {
    row(label, label, "", Kind::Backup(part))
}
/// Restore's rows about the backup show once one is read; its groups only if it can be restored.
const LOADED: Cond = Cond::Restore(false);
const USABLE: Cond = Cond::Restore(true);
const PREFERENCES: &str = "backup_and_restore__back_up__files_jetpref_datastore";
const KEYBOARDS: &str = "backup_and_restore__back_up__files_ime_keyboard";
const THEMES: &str = "backup_and_restore__back_up__files_ime_theme";
const CLIPBOARD: &str = "backup_and_restore__back_up__files_clipboard_history";
const TEXT: &str = "backup_and_restore__back_up__files_clipboard_history__clipboard_text_items";
const IMAGES: &str = "backup_and_restore__back_up__files_clipboard_history__clipboard_image_items";
const VIDEO: &str = "backup_and_restore__back_up__files_clipboard_history__clipboard_video_items";
/// Each group's name, in layout order, as the confirmation lists them.
pub const GROUP_NAMES: [&str; 6] = [PREFERENCES, KEYBOARDS, THEMES, TEXT, IMAGES, VIDEO];

const BACKUP_ROWS: &[Row] = &[
    heading("backup_and_restore__back_up__destination"),
    part(
        "backup_and_restore__back_up__destination_file_sys",
        Part::Destination(0),
    ),
    part(
        "backup_and_restore__back_up__destination_share_intent",
        Part::Destination(1),
    ),
    heading("backup_and_restore__back_up__files"),
    part(PREFERENCES, Part::Group(0)),
    part(KEYBOARDS, Part::Group(1)),
    part(THEMES, Part::Group(2)),
    part(CLIPBOARD, Part::Clipboard),
    part(TEXT, Part::Group(3)),
    part(IMAGES, Part::Group(4)),
    part(VIDEO, Part::Group(5)),
    part("action__cancel", Part::Cancel),
    part("action__back_up", Part::Start),
];
pub const BACKUP: Screen = Screen {
    route: BACKUP_ROUTE,
    title: "backup_and_restore__back_up__title",
    icon_space: true,
    preview: false,
    rows: BACKUP_ROWS,
};

const RESTORE_ROWS: &[Row] = &[
    heading("backup_and_restore__restore__mode"),
    part("backup_and_restore__restore__mode_merge", Part::Mode(false)),
    part(
        "backup_and_restore__restore__mode_erase_and_overwrite",
        Part::Mode(true),
    ),
    part("action__select_file", Part::Choose).icon("description"),
    heading("backup_and_restore__restore__metadata").shown(LOADED),
    part("", Part::About(0)).icon("schedule").shown(LOADED),
    part("", Part::About(1)).icon("info").shown(LOADED),
    part("", Part::About(2)).icon("code").shown(LOADED),
    part("", Part::Warning).shown(LOADED),
    heading("backup_and_restore__restore__files").shown(USABLE),
    part(PREFERENCES, Part::Group(0)).shown(USABLE),
    part(KEYBOARDS, Part::Group(1)).shown(USABLE),
    part(THEMES, Part::Group(2)).shown(USABLE),
    part(CLIPBOARD, Part::Clipboard).shown(USABLE),
    part(TEXT, Part::Group(3)).shown(USABLE),
    part(IMAGES, Part::Group(4)).shown(USABLE),
    part(VIDEO, Part::Group(5)).shown(USABLE),
    part("settings__restore__confirm_title", Part::Confirm).shown(Cond::Never),
    part("action__cancel", Part::Cancel),
    part("action__restore", Part::Start),
];
pub const RESTORE: Screen = Screen {
    route: RESTORE_ROUTE,
    title: "backup_and_restore__restore__title",
    icon_space: true,
    preview: false,
    rows: RESTORE_ROWS,
};
