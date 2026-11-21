//! What each picker answer and piece of file work leads to on Back up and Restore.
use super::{
    App,
    backup_screens::{ALL, DEFAULT_MASK, Loaded, OPEN, SAVE},
    catalogue::{BACKUP_ROUTE, RESTORE_ROUTE},
    files::Picked,
};
use wire::backup::{self as policy, layout};
use wire::settings_ui::files::{done, job, mode, result, work};

impl App {
    pub(super) fn backup_picked(&mut self, purpose: &str, answer: Picked) {
        match (purpose, answer) {
            (SAVE, Picked::Documents(d)) => {
                self.work(job::BACKUP_WRITE, &[(work::DOCUMENT, d[0].handle.clone())]);
                self.progress(BACKUP_ROUTE, "settings__backup__saving");
            }
            (SAVE, answer) => {
                self.work(job::DISCARD, &[]);
                let key = if answer == Picked::NoPicker {
                    "settings__backup__no_picker"
                } else {
                    "settings__backup__cancelled"
                };
                self.tell(BACKUP_ROUTE, key, answer == Picked::NoPicker);
            }
            (OPEN, Picked::Documents(d)) => {
                self.backups.file = None;
                self.backups.reading = Some(d[0].name.clone());
                self.work(job::RESTORE_READ, &[(work::DOCUMENT, d[0].handle.clone())]);
                self.progress(RESTORE_ROUTE, "settings__restore__reading");
            }
            (OPEN, Picked::Cancelled) => {
                self.tell(RESTORE_ROUTE, "settings__restore__no_file", false)
            }
            (OPEN, Picked::NoPicker) => {
                self.tell(RESTORE_ROUTE, "settings__restore__no_picker", true)
            }
            _ => {}
        }
    }
    pub(super) fn backup_done(&mut self, p: &[String]) {
        let ok = p[done::RESULT] == result::OK;
        let gone = p[done::RESULT] == result::GONE;
        match p[done::JOB].as_str() {
            // A backup made after its screen closed is thrown away, not offered for saving.
            job::BACKUP_PREPARE if ok && !self.on(BACKUP_ROUTE) => self.work(job::DISCARD, &[]),
            job::BACKUP_PREPARE if ok && self.backups.share => {
                self.work(job::BACKUP_SHARE, &[]);
            }
            job::BACKUP_PREPARE if ok => {
                self.pick(SAVE, mode::CREATE, &["application/zip"], &p[done::NAME]);
                self.progress(BACKUP_ROUTE, "settings__backup__choose_place");
            }
            job::BACKUP_PREPARE => {
                self.tell(BACKUP_ROUTE, "settings__backup__prepare_failed", true)
            }
            job::BACKUP_WRITE if ok => {
                self.finished(BACKUP_ROUTE, "backup_and_restore__back_up__success")
            }
            job::BACKUP_WRITE if gone => self.tell(BACKUP_ROUTE, "settings__backup__gone", true),
            job::BACKUP_WRITE => self.tell(BACKUP_ROUTE, "settings__backup__write_failed", true),
            job::BACKUP_SHARE if ok => self.progress(BACKUP_ROUTE, "settings__backup__share_ready"),
            job::BACKUP_SHARE => self.tell(BACKUP_ROUTE, "settings__backup__no_app", true),
            job::RESTORE_READ if ok && !self.on(RESTORE_ROUTE) => self.work(job::DISCARD, &[]),
            job::RESTORE_READ if ok => self.read(p),
            job::RESTORE_READ => {
                self.backups.reading = None;
                let key = match p[done::RESULT].as_str() {
                    result::NOT_BACKUP => "settings__restore__not_backup",
                    result::BAD_METADATA => "settings__restore__bad_metadata",
                    _ => "settings__restore__unreadable",
                };
                self.tell(RESTORE_ROUTE, key, true);
            }
            job::RESTORE_APPLY if ok => {
                self.backups.applying = None;
                self.backups.file = None;
                self.finished(RESTORE_ROUTE, "backup_and_restore__restore__success");
            }
            job::RESTORE_APPLY if gone => {
                self.backups.applying = None;
                self.backups.file = None;
                self.tell(RESTORE_ROUTE, "settings__restore__gone", true);
            }
            job::RESTORE_APPLY => {
                let steps = p[done::STEPS].parse().unwrap_or(0);
                let text = self.stopped(steps);
                self.backups.applying = None;
                self.say(RESTORE_ROUTE, text, true);
            }
            _ => {}
        }
    }
    /// Success: a toast, as Compose gave, and the screen closes if it is still open.
    fn finished(&mut self, route: &str, key: &str) {
        self.backups.note = None;
        let text = self.label(key);
        self.toast(text);
        if self.on(route) {
            self.back();
        }
    }
    fn read(&mut self, p: &[String]) {
        let number = |at: usize| p[at].parse::<i32>().unwrap_or(0);
        let file = Loaded {
            name: self.backups.reading.take().unwrap_or_default(),
            package: p[done::PACKAGE].clone(),
            version_code: number(done::VERSION_CODE),
            version_name: p[done::VERSION_NAME].clone(),
            date: p[done::DATE].clone(),
            present: number(done::PRESENT) & ALL,
            damaged: number(done::DAMAGED) & ALL,
        };
        self.backups.file = Some(file);
        self.backups.restore_mask = DEFAULT_MASK & self.restorable();
        self.backups.note = None;
    }
    /// What a restore that stopped after `steps` of its plan's steps changed, from the same plan
    /// BackupIO ran: which groups came back, which did not, and whether erasing had begun.
    fn stopped(&self, steps: usize) -> String {
        let Some((mask, erase)) = self.backups.applying else {
            return self.label("settings__restore__nothing_changed");
        };
        if steps == 0 {
            return self.label("settings__restore__nothing_changed");
        }
        let present = self.backups.file.as_ref().map_or(0, |f| f.present);
        let flags: Vec<Vec<u16>> = (0..layout::SIZE)
            .map(|g| wire::flag::utf16(present & (1 << g) != 0))
            .collect();
        let paths = ::backup::layout(&[]);
        let plan = ::backup::restore(mask, erase, &paths, &flags);
        let (mut back, mut cleared) = (0, false);
        for step in plan.chunks(policy::effect::SIZE).take(steps) {
            let kind = String::from_utf16_lossy(&step[policy::effect::EFFECT]);
            let group = match kind.as_str() {
                policy::PREFERENCES_MERGE | policy::PREFERENCES_ERASE => layout::PREFERENCES,
                policy::RESTORE_TEXT => layout::CLIPBOARD_TEXT,
                policy::RESTORE_IMAGES => layout::CLIPBOARD_IMAGES,
                policy::RESTORE_VIDEO => layout::CLIPBOARD_VIDEO,
                policy::COPY_DIRECTORY | policy::CLEAR_DIRECTORY => {
                    let path = &step[policy::effect::PATH];
                    let group = (0..layout::SIZE).find(|g| paths[*g] == *path).unwrap_or(0);
                    if kind == policy::CLEAR_DIRECTORY {
                        cleared = true;
                        continue;
                    }
                    group
                }
                _ => {
                    cleared = true;
                    continue;
                }
            };
            back |= 1 << group;
        }
        let wanted = mask & present;
        let mut text = self
            .label("settings__restore__part_way")
            .replace("{done}", &self.group_names(back))
            .replace("{rest}", &self.group_names(wanted & !back));
        if cleared {
            text = format!("{text} {}", self.label("settings__restore__cleared"));
        }
        text
    }
    /// `IDLE` while work was running: Android closed the app and the work was lost with it.
    pub(super) fn backup_lost(&mut self, lost: &str) {
        let route = if lost.starts_with("restore") {
            self.backups.applying = None;
            self.backups.reading = None;
            self.backups.file = None;
            RESTORE_ROUTE
        } else {
            BACKUP_ROUTE
        };
        self.tell(route, "settings__files__interrupted", true);
    }
}
