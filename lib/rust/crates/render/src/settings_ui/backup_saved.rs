//! Back up's and Restore's state in the activity's saved state, so a picker answer or a report
//! that arrives after Android recreated the activity still finds what it belongs to.
use super::{
    App,
    backup_screens::{ALL, Backups, Loaded, Note},
    catalogue::{BACKUP_ROUTE, RESTORE_ROUTE},
    files::Files,
};
use serde_json::{Value, json};

const WRONG: &str = "settings saved backups";

impl App {
    /// The picker answer and work Rust waits for, and both screens' state, as JSON.
    pub fn backups_saved(&self) -> String {
        let b = &self.backups;
        let file = b.file.as_ref().map(|f| {
            json!([
                f.name,
                f.package,
                f.version_code,
                f.version_name,
                f.date,
                f.present,
                f.damaged
            ])
        });
        let note = b.note.as_ref().map(|n| json!([n.route, n.text, n.error]));
        json!({
            "waiting": self.files.waiting,
            "busy": self.files.busy,
            "choices": [b.share, b.backup_mask, b.erase, b.restore_mask],
            "file": file,
            "reading": b.reading,
            "applying": b.applying.map(|(mask, erase)| json!([mask, erase])),
            "note": note,
        })
        .to_string()
    }
}

/// What [`App::backups_saved`] wrote; anything else fails.
pub fn parse_backups(saved: &str) -> Result<(Files, Backups), &'static str> {
    let v: Value = serde_json::from_str(saved).map_err(|_| WRONG)?;
    let text = |v: &Value| v.as_str().map(String::from).ok_or(WRONG);
    let maybe = |v: &Value| match v {
        Value::Null => Ok(None),
        v => text(v).map(Some),
    };
    let flag = |v: &Value| v.as_bool().ok_or(WRONG);
    let mask = |v: &Value| match v.as_i64() {
        Some(m) if (0..=i64::from(ALL)).contains(&m) => Ok(m as i32),
        _ => Err(WRONG),
    };
    let number = |v: &Value| v.as_i64().and_then(|n| i32::try_from(n).ok()).ok_or(WRONG);
    let list = |v: &Value, n: usize| match v.as_array() {
        Some(a) if a.len() == n => Ok(a.clone()),
        _ => Err(WRONG),
    };
    let files = Files {
        waiting: maybe(&v["waiting"])?,
        busy: maybe(&v["busy"])?,
    };
    let choices = list(&v["choices"], 4)?;
    let file = match &v["file"] {
        Value::Null => None,
        f => {
            let f = list(f, 7)?;
            Some(Loaded {
                name: text(&f[0])?,
                package: text(&f[1])?,
                version_code: number(&f[2])?,
                version_name: text(&f[3])?,
                date: text(&f[4])?,
                present: mask(&f[5])?,
                damaged: mask(&f[6])?,
            })
        }
    };
    let applying = match &v["applying"] {
        Value::Null => None,
        a => {
            let a = list(a, 2)?;
            Some((mask(&a[0])?, flag(&a[1])?))
        }
    };
    let note = match &v["note"] {
        Value::Null => None,
        n => {
            let n = list(n, 3)?;
            let route = text(&n[0])?;
            if route != BACKUP_ROUTE && route != RESTORE_ROUTE {
                return Err(WRONG);
            }
            Some(Note {
                route,
                text: text(&n[1])?,
                error: flag(&n[2])?,
            })
        }
    };
    let backups = Backups {
        share: flag(&choices[0])?,
        backup_mask: mask(&choices[1])?,
        erase: flag(&choices[2])?,
        restore_mask: mask(&choices[3])?,
        file,
        reading: maybe(&v["reading"])?,
        applying,
        note,
    };
    Ok((files, backups))
}
