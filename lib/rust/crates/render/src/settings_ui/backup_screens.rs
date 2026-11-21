//! Back up and Restore: what their rows hold and do, and what each picker answer and piece of file
//! work leads to. Kotlin's BackupIO does the archive and file work; nothing here touches a file.
//! The rules for which groups a restore touches stay in the backup crate, which BackupIO runs.
use super::{
    App, Modal,
    catalogue::{BACKUP_ROUTE, GROUP_NAMES, Kind, Part, RESTORE_ROUTE},
};
use wire::backup::{self as policy, layout};
use wire::settings_ui::files::{job, mode, work};

/// The picker purposes: where to save a backup, and which backup to restore.
pub(super) const SAVE: &str = "backupSave";
pub(super) const OPEN: &str = "restoreOpen";
/// Preferences, keyboard extensions and themes, which Compose selected first.
pub(super) const DEFAULT_MASK: i32 = 0b111;
pub(super) const ALL: i32 = (1 << layout::SIZE) - 1;

/// String keys the code below names.
pub const KEYS: &[&str] = &[
    "backup_and_restore__back_up__success",
    "backup_and_restore__restore__success",
    "backup_and_restore__restore__metadata_warn_different_version",
    "backup_and_restore__restore__metadata_warn_different_vendor",
    "backup_and_restore__restore__metadata_error_invalid_metadata",
    "state__no_file_selected",
    "settings__backup__preparing",
    "settings__backup__choose_place",
    "settings__backup__saving",
    "settings__backup__cancelled",
    "settings__backup__no_picker",
    "settings__backup__prepare_failed",
    "settings__backup__write_failed",
    "settings__backup__share_ready",
    "settings__backup__no_app",
    "settings__backup__gone",
    "settings__restore__no_file",
    "settings__restore__no_picker",
    "settings__restore__reading",
    "settings__restore__unreadable",
    "settings__restore__not_backup",
    "settings__restore__bad_metadata",
    "settings__restore__newer",
    "settings__restore__made",
    "settings__restore__version",
    "settings__restore__package",
    "settings__restore__missing",
    "settings__restore__damaged",
    "settings__restore__confirm_merge",
    "settings__restore__confirm_erase",
    "settings__restore__restoring",
    "settings__restore__nothing_changed",
    "settings__restore__part_way",
    "settings__restore__none",
    "settings__restore__cleared",
    "settings__restore__gone",
    "settings__files__interrupted",
];

/// A backup Restore read: what it is, and which groups it holds and could not read.
#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    pub name: String,
    pub package: String,
    pub version_code: i32,
    pub version_name: String,
    /// When it was made, as the device words a date.
    pub date: String,
    pub present: i32,
    pub damaged: i32,
}

/// What is happening on a screen, or what went wrong there, in plain words.
#[derive(Clone, Debug, PartialEq)]
pub struct Note {
    pub route: String,
    pub text: String,
    pub error: bool,
}

/// Both screens' choices. Compose forgot them when the activity was recreated; these are saved.
#[derive(Clone, Debug, PartialEq)]
pub struct Backups {
    pub share: bool,
    pub backup_mask: i32,
    pub erase: bool,
    pub restore_mask: i32,
    pub file: Option<Loaded>,
    /// The name of the document being read.
    pub reading: Option<String>,
    /// The mask and mode of the restore running, so a stop part-way can say what came back.
    pub applying: Option<(i32, bool)>,
    pub note: Option<Note>,
}
impl Default for Backups {
    fn default() -> Self {
        Self {
            share: false,
            backup_mask: DEFAULT_MASK,
            erase: false,
            restore_mask: DEFAULT_MASK,
            file: None,
            reading: None,
            applying: None,
            note: None,
        }
    }
}

impl App {
    pub(super) fn on(&self, route: &str) -> bool {
        self.stack.last().is_some_and(|r| r == route)
    }
    /// The backup's metadata warning and error, as the backup crate decides them.
    fn status(&self, file: &Loaded) -> [&'static str; 2] {
        let current = self.fact("version_code").parse().unwrap_or(0);
        let package: Vec<u16> = file.package.encode_utf16().collect();
        ::backup::metadata(&package, file.version_code, current)
    }
    /// `Cond::Restore`: a backup was read, and with `usable` it can be restored.
    pub(super) fn restore_shown(&self, usable: bool) -> bool {
        self.backups
            .file
            .as_ref()
            .is_some_and(|f| !usable || self.status(f)[1].is_empty())
    }
    /// The groups the read backup can restore: those it holds that were read whole.
    pub fn restorable(&self) -> i32 {
        match &self.backups.file {
            Some(f) if self.status(f)[1].is_empty() => f.present & !f.damaged,
            _ => 0,
        }
    }
    /// The groups a screen offers, and its mask.
    fn offered(&self) -> (i32, i32) {
        if self.on(RESTORE_ROUTE) {
            (self.restorable(), self.backups.restore_mask)
        } else {
            (ALL, self.backups.backup_mask)
        }
    }
    pub(super) fn part_enabled(&self, part: Part) -> bool {
        if self.files.busy.is_some() && part != Part::Cancel {
            return false;
        }
        let (offered, mask) = self.offered();
        match part {
            Part::Group(g) => offered & (1 << g) != 0,
            Part::Clipboard => offered & ::backup::CLIPBOARD_MASK != 0,
            Part::Start => mask & offered != 0,
            _ => true,
        }
    }
    /// A checkbox's state: on, off, or None when only some of the clipboard is on.
    pub(super) fn part_checked(&self, part: Part) -> Option<bool> {
        let (offered, mask) = self.offered();
        match part {
            Part::Group(g) => Some(mask & offered & (1 << g) != 0),
            _ => match ::backup::clipboard_state(mask, offered) {
                policy::MIXED => None,
                state => Some(state == policy::ON),
            },
        }
    }
    pub(super) fn part_selected(&self, part: Part) -> bool {
        match part {
            Part::Destination(n) => self.backups.share == (n == 1),
            Part::Mode(erase) => self.backups.erase == erase,
            _ => false,
        }
    }
    /// A row's label and summary where they depend on the screen's state.
    pub(super) fn part_text(&self, part: Part, label: String) -> (String, String) {
        let file = self.backups.file.as_ref();
        match part {
            Part::Choose => (
                label,
                file.map_or(self.label("state__no_file_selected"), |f| f.name.clone()),
            ),
            Part::About(n) => {
                let f = file.expect("shown only with a backup");
                match n {
                    0 => (f.date.clone(), self.label("settings__restore__made")),
                    1 => (
                        format!("{} ({})", f.version_name, f.version_code),
                        self.label("settings__restore__version"),
                    ),
                    _ => (f.package.clone(), self.label("settings__restore__package")),
                }
            }
            Part::Group(g) if self.on(RESTORE_ROUTE) => {
                let summary = match file {
                    Some(f) if f.damaged & (1 << g) != 0 => "settings__restore__damaged",
                    Some(f) if f.present & (1 << g) == 0 => "settings__restore__missing",
                    _ => return (label, String::new()),
                };
                (label, self.label(summary))
            }
            _ => (label, String::new()),
        }
    }
    /// The backup's metadata warning or error, and whether it is an error.
    pub(super) fn backup_warning(&self) -> (String, bool) {
        let Some(f) = &self.backups.file else {
            return (String::new(), false);
        };
        let current: i32 = self.fact("version_code").parse().unwrap_or(0);
        match self.status(f) {
            [_, policy::INVALID_METADATA] => (
                self.label("backup_and_restore__restore__metadata_error_invalid_metadata"),
                true,
            ),
            [policy::DIFFERENT_VERSION, _] if current > 0 && f.version_code > current => {
                (self.label("settings__restore__newer"), false)
            }
            [policy::DIFFERENT_VERSION, _] => (
                self.label("backup_and_restore__restore__metadata_warn_different_version"),
                false,
            ),
            // The vendor check knows only FlorisBoard's package: this app's own backups pass.
            [policy::DIFFERENT_VENDOR, _] if f.package == self.fact("package") => {
                (String::new(), false)
            }
            [policy::DIFFERENT_VENDOR, _] => (
                self.label("backup_and_restore__restore__metadata_warn_different_vendor"),
                false,
            ),
            _ => (String::new(), false),
        }
    }
    /// The note the bottom bar shows on this screen.
    pub(super) fn backup_note(&self) -> Option<&Note> {
        self.backups.note.as_ref().filter(|n| self.on(&n.route))
    }
    /// A tap on one of the screens' rows or bottom bar.
    pub(super) fn backup_tap(&mut self, part: Part) {
        let restore = self.on(RESTORE_ROUTE);
        let (offered, _) = self.offered();
        let mask = if restore {
            &mut self.backups.restore_mask
        } else {
            &mut self.backups.backup_mask
        };
        match part {
            Part::Destination(n) => self.backups.share = n == 1,
            Part::Mode(erase) => self.backups.erase = erase,
            Part::Group(g) => *mask ^= 1 << g,
            // Compose's TriStateCheckbox: off or mixed turns every offered clip on, on turns off.
            Part::Clipboard => {
                let clips = offered & ::backup::CLIPBOARD_MASK;
                *mask = if *mask & clips == clips {
                    *mask & !clips
                } else {
                    *mask | clips
                };
            }
            Part::Choose => {
                self.backups.note = None;
                self.pick(OPEN, mode::OPEN, &["*/*"], "");
            }
            Part::Cancel => self.back(),
            Part::Start if restore => self.ask_restore(),
            Part::Start => {
                let mask = self.backups.backup_mask.to_string();
                self.work(job::BACKUP_PREPARE, &[(work::MASK, mask)]);
                self.progress(BACKUP_ROUTE, "settings__backup__preparing");
            }
            _ => {}
        }
    }
    fn ask_restore(&mut self) {
        let Some(row) = self
            .rows()
            .iter()
            .position(|r| matches!(r.kind, Kind::Backup(Part::Confirm)))
        else {
            return;
        };
        self.scrollbar_start[1] = -1;
        self.scrollbar_last[1] = -1;
        self.modal = Some(Modal {
            row,
            selected: String::new(),
            values: vec![],
            scroll: 0.0,
            enabled: true,
        });
        self.focus = -1;
    }
    /// The confirmation's message: what the restore replaces, and that erasing clears the
    /// whole clipboard history, which the backup crate's plan does whatever is chosen.
    pub(super) fn restore_question(&self) -> String {
        let mask = self.backups.restore_mask & self.restorable();
        let date = self.backups.file.as_ref().map_or("", |f| f.date.as_str());
        let key = if self.backups.erase {
            "settings__restore__confirm_erase"
        } else {
            "settings__restore__confirm_merge"
        };
        self.label(key)
            .replace("{groups}", &self.group_names(mask))
            .replace("{date}", date)
    }
    pub(super) fn group_names(&self, mask: i32) -> String {
        let names: Vec<String> = (0..layout::SIZE)
            .filter(|g| mask & (1 << g) != 0)
            .map(|g| self.label(GROUP_NAMES[g]))
            .collect();
        if names.is_empty() {
            self.label("settings__restore__none")
        } else {
            names.join(", ")
        }
    }
    /// The confirmation's Restore.
    pub(super) fn restore_confirmed(&mut self) {
        let mask = self.backups.restore_mask & self.restorable();
        let erase = self.backups.erase;
        self.backups.applying = Some((mask, erase));
        let fields = [
            (work::MASK, mask.to_string()),
            (work::ERASE, wire::flag::encode(erase).to_owned()),
        ];
        self.work(job::RESTORE_APPLY, &fields);
        self.progress(RESTORE_ROUTE, "settings__restore__restoring");
    }
    /// Words for a screen: its note while it is open, else a toast.
    pub(super) fn say(&mut self, route: &str, text: String, error: bool) {
        if self.on(route) {
            self.backups.note = Some(Note {
                route: route.into(),
                text,
                error,
            });
        } else {
            self.toast(text);
        }
    }
    pub(super) fn tell(&mut self, route: &str, key: &str, error: bool) {
        let text = self.label(key);
        self.say(route, text, error);
    }
    /// What is under way, shown only on its screen.
    pub(super) fn progress(&mut self, route: &str, key: &str) {
        if self.on(route) {
            self.tell(route, key, false);
        }
    }
    /// After the back stack changed: a screen that closed forgets its choices, as Compose's did,
    /// and its archives are deleted unless work still uses them.
    pub(super) fn backups_left(&mut self, had: [bool; 2]) {
        let has = [BACKUP_ROUTE, RESTORE_ROUTE].map(|r| self.stack.iter().any(|s| s == r));
        let mut left = false;
        if had[0] && !has[0] {
            left = true;
            self.backups.share = false;
            self.backups.backup_mask = DEFAULT_MASK;
        }
        if had[1] && !has[1] && self.backups.applying.is_none() {
            left = true;
            self.backups.erase = false;
            self.backups.restore_mask = DEFAULT_MASK;
            self.backups.file = None;
        }
        if left {
            self.backups.note = None;
            self.work(job::DISCARD, &[]);
        }
    }
    /// Whether Back up and Restore are on the back stack.
    pub(super) fn backups_open(&self) -> [bool; 2] {
        [BACKUP_ROUTE, RESTORE_ROUTE].map(|r| self.stack.iter().any(|s| s == r))
    }
}
