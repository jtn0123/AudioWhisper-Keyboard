//! The extension editor's file work and what it says when it ends: Save, the folders' Add,
//! Rename and Delete, and a copied component's stylesheet. Compose ran each on the main thread
//! and said nothing, or an English toast, when it failed; each runs off it here (`files.rs`) and
//! says what happened in the user's language.
use super::{FILE, FILE_ADD, FOLDERS, FileForm, PICK, target};
use crate::settings_ui::{
    App,
    files::{Document, Picked},
};
use wire::settings_ui::files::{done, job, result};

/// The editor's jobs.
const JOBS: [&str; 5] = [
    job::EXT_SAVE,
    job::EXT_FILE_ADD,
    job::EXT_FILE_RENAME,
    job::EXT_FILE_DELETE,
    job::EXT_COMPONENT_COPY,
];

/// A workspace path's folder and name: `fonts/a.ttf` is the fonts' `a.ttf`.
fn split(path: &str) -> Option<(usize, String)> {
    let (dir, name) = path.split_once('/')?;
    Some((FOLDERS.iter().position(|d| *d == dir)?, name.into()))
}

impl App {
    /// The picker closed on a folder's Add: the picked file's name, ready to change, on the
    /// import sheet. False when the answer is not the editor's.
    pub(in crate::settings_ui) fn ext_editor_picked(
        &mut self,
        purpose: &str,
        answer: &Picked,
    ) -> bool {
        let Some(folder) = purpose.strip_prefix(PICK) else {
            return false;
        };
        match answer {
            Picked::Documents(d) if self.editing() => {
                let Document { handle, name } = d[0].clone();
                let folder = folder.parse().unwrap_or(0usize).min(1);
                let form = FileForm {
                    folder,
                    name,
                    document: handle,
                    ..FileForm::default()
                };
                self.extensions.editor.file_form = Some(form);
                self.navigate(FILE_ADD);
            }
            Picked::NoPicker => self.toast(self.label("settings__udm__no_picker")),
            _ => {}
        }
        true
    }
    /// File work ended; false when it is not the editor's.
    pub(in crate::settings_ui) fn ext_editor_done(&mut self, p: &[String]) -> bool {
        let kind = p[done::JOB].as_str();
        if !JOBS.contains(&kind) {
            return false;
        }
        let ok = p[done::RESULT] == result::OK;
        // Not only once it is open: a Save that ended while the activity was recreated reports
        // before Kotlin answers the restored editor, which would find the workspace closed.
        let e = &self.extensions.editor;
        let on = self.editor_route() == Some(e.route.as_str());
        let mine = on && !e.workspace.is_empty() && e.workspace == p[done::NAME];
        match kind {
            job::EXT_SAVE => self.saved(&p[done::RESULT], mine),
            job::EXT_COMPONENT_COPY => self.copied(ok),
            _ if ok => self.file_changed(kind, &p[done::NAME], &p[done::TEXT]),
            _ => {
                let key = match p[done::RESULT].as_str() {
                    result::TAKEN => "settings__ext_editor__file_exists",
                    result::WRONG_TYPE => "settings__ext_editor__wrong_type",
                    _ => "settings__ext_editor__file_failed",
                };
                self.toast(self.label(key));
            }
        }
        true
    }
    /// Save ended: the editor closes on the saved extension, as `handleSave` popped it, or says
    /// why it could not be saved and stays open with the draft.
    fn saved(&mut self, outcome: &str, mine: bool) {
        if outcome != result::OK {
            let key = match outcome {
                result::TAKEN => "settings__ext_editor__id_taken",
                result::GONE => "settings__ext_editor__gone",
                _ => "settings__ext_editor__save_failed",
            };
            if outcome == result::GONE && mine {
                let e = &mut self.extensions.editor;
                (e.load, e.workspace) = (super::Load::Gone, String::new());
            }
            return self.toast(self.label(key));
        }
        if !mine {
            return;
        }
        // Kotlin closed the workspace once the archive was in place.
        let e = &mut self.extensions.editor;
        (e.workspace, e.changed) = (String::new(), false);
        while self.stack.last().is_some_and(|r| target(r).is_none()) && self.stack.len() > 1 {
            self.back();
        }
        if self.stack.last().is_some_and(|r| target(r).is_some()) {
            self.back();
        }
    }
    /// A copied component's stylesheet is in: the component is added and the sheet closes.
    fn copied(&mut self, ok: bool) {
        let pending = self
            .extensions
            .editor
            .comp_form
            .as_mut()
            .and_then(|f| f.pending.take());
        if !ok {
            return self.toast(self.label("settings__ext_editor__copy_failed"));
        }
        let Some(comp) = pending.filter(|_| self.editing()) else {
            return;
        };
        self.extensions.editor.draft.themes.push(comp);
        self.ext_changed();
        if self.on(super::COMPONENT_ADD) {
            self.close_editor();
        }
    }
    /// A file was added, renamed or deleted: the list says so and its sheet closes.
    fn file_changed(&mut self, kind: &str, name: &str, from: &str) {
        if !self.editing() {
            return;
        }
        let files = &mut self.extensions.editor.files;
        let gone = if kind == job::EXT_FILE_ADD {
            None
        } else {
            split(if from.is_empty() { name } else { from })
        };
        if let Some(gone) = gone {
            files.retain(|f| *f != gone);
        }
        if kind != job::EXT_FILE_DELETE
            && let Some(new) = split(name)
        {
            files.push(new);
            files.sort_by(|a, b| a.1.cmp(&b.1));
        }
        self.ext_changed();
        if self.on(FILE) || self.on(FILE_ADD) {
            self.close_editor();
        }
    }
    /// Work Rust waited for was lost with the activity; false when it was not the editor's. A
    /// lost Save may have finished: the editor says it cannot tell.
    pub(in crate::settings_ui) fn ext_editor_lost(&mut self, lost: &str) -> bool {
        if !JOBS.contains(&lost) {
            return false;
        }
        if let Some(f) = self.extensions.editor.comp_form.as_mut() {
            f.pending = None;
        }
        let key = match lost {
            job::EXT_SAVE => "settings__ext_editor__save_interrupted",
            _ => "settings__files__interrupted",
        };
        self.toast(self.label(key));
        true
    }
}
