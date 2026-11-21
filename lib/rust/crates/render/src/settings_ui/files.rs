//! The file picker boundary. A screen asks for the system picker with a purpose; Kotlin opens
//! it, keeps what was picked under numbered handles and reports how it closed. Rust then decides
//! what happens next, usually file work, which Kotlin runs off the main thread and reports when
//! it ends. A document's address never reaches Rust, and one piece of work runs at a time.
//!
//! Both the picker's answer and the work's report reach whichever screen is attached when they
//! arrive, so they survive the activity being recreated; [`Files`] is in the saved state.
use super::App;
use wire::settings_ui::files::{document, done, effect, job, outcome, pick, picked, report, work};

/// A picked document: Kotlin's handle for it and its display name, which may be empty.
#[derive(Clone, Debug, PartialEq)]
pub struct Document {
    pub handle: String,
    pub name: String,
}

/// How the picker closed.
#[derive(Clone, Debug, PartialEq)]
pub enum Picked {
    Documents(Vec<Document>),
    Cancelled,
    /// No app on the device can show the picker.
    NoPicker,
}

/// What Rust waits for from Kotlin.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Files {
    /// The purpose the open picker's answer goes to.
    pub waiting: Option<String>,
    /// The work running.
    pub busy: Option<String>,
}

impl App {
    /// Asks for the picker: `how` is a `SettingsUiPickMode`, `name` a new document's suggested
    /// name. `purpose` names the screen and step its answer goes to.
    pub fn pick(&mut self, purpose: &str, how: &str, mime: &[&str], name: &str) {
        self.files.waiting = Some(purpose.into());
        let mut packet = vec![String::new(); pick::SIZE];
        packet[pick::EFFECT] = effect::PICK.into();
        packet[pick::MODE] = how.into();
        packet[pick::MIME] = mime.join("|");
        packet[pick::NAME] = name.into();
        self.effects.push(packet);
    }
    /// Asks for file work, its `SettingsUiWork` fields given by position. Every job but
    /// `DISCARD` makes the screens busy until it reports.
    pub fn work(&mut self, kind: &str, fields: &[(usize, String)]) {
        let mut packet = vec![String::new(); work::SIZE];
        packet[work::EFFECT] = effect::WORK.into();
        packet[work::JOB] = kind.into();
        for (at, value) in fields {
            packet[*at] = value.clone();
        }
        if kind != job::DISCARD {
            self.files.busy = Some(kind.into());
        }
        self.effects.push(packet);
    }
    /// Shows `text` as a long toast.
    pub fn toast(&mut self, text: String) {
        self.effects.push(vec![effect::TOAST.into(), text]);
    }
    /// A `FILES` report: how the picker closed, how work ended, or that work was lost.
    pub fn file_report(&mut self, p: &[String]) -> Result<(), &'static str> {
        match p.first().map(String::as_str) {
            Some(report::PICKED) => {
                let rows = p.get(picked::SIZE..).ok_or("files picked")?;
                let (documents, rest) = rows.as_chunks::<{ document::SIZE }>();
                if !rest.is_empty() {
                    return Err("files documents");
                }
                let answer = match p[picked::OUTCOME].as_str() {
                    outcome::PICKED if !documents.is_empty() => Picked::Documents(
                        documents
                            .iter()
                            .map(|d| Document {
                                handle: d[document::HANDLE].clone(),
                                name: d[document::NAME].clone(),
                            })
                            .collect(),
                    ),
                    outcome::PICKED | outcome::CANCELLED => Picked::Cancelled,
                    outcome::NO_PICKER => Picked::NoPicker,
                    _ => return Err("files outcome"),
                };
                // An answer nothing waits for changes nothing.
                if let Some(purpose) = self.files.waiting.take() {
                    self.backup_picked(&purpose, answer);
                }
            }
            Some(report::DONE) => {
                if p.len() != done::SIZE {
                    return Err("files done");
                }
                if self.files.busy.as_deref() == Some(p[done::JOB].as_str()) {
                    self.files.busy = None;
                }
                self.backup_done(p);
            }
            Some(report::IDLE) => {
                if let Some(lost) = self.files.busy.take() {
                    self.backup_lost(&lost);
                }
            }
            _ => return Err("files report"),
        }
        Ok(())
    }
}
