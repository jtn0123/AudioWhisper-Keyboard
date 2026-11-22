//! The importer and Export, which ExtensionImportScreen and ExtensionExportScreen were, through
//! the file picker (`files.rs`). Kotlin reads the picked files into an import workspace in the
//! cache, describes what each holds (`EXTENSION_FILES`), and imports or exports on Rust's word;
//! Rust decides which files can be imported and says why the others cannot.
//!
//! The workspace's id is in the importer's route, so the selection outlives the activity:
//! `uuid=` for a file another app shared, as Compose's route had it, which hides Select files as
//! Compose did; `files=` for files picked here; `document=` for a shared file not yet read.
use super::{KEYBOARD, LANGUAGE_PACK, THEME};
use crate::settings_ui::{App, files::Picked};
use extensions::manifest::{Kind, accepts};
use wire::settings_ui::extensions::{effect, file, row};
use wire::settings_ui::files::{done, job, mode, result, work};

pub const ANY: &str = "ext/import/EXT_ANY";
pub const KEYBOARDS: &str = "ext/import/EXT_KEYBOARD";
pub const THEMES: &str = "ext/import/EXT_THEME";
pub const PACKS: &str = "ext/import/EXT_LANGUAGEPACK";
/// The picker purposes: Select files', and Export's, which the extension's id follows.
pub const PICK: &str = "extImport";
pub const EXPORT: &str = "extExport:";
/// FileRegistry.FlexExtension's media type, and the name `createFlexName` gave an export.
pub const FLEX_MIME: &str = "application/vnd.florisboard.extension+zip";
pub fn flex_name(id: &str) -> String {
    format!("{id}.flex")
}

/// What an archive in the workspace holds, as FileInfoView showed it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Archive {
    pub id: String,
    pub serial: String,
    pub title: String,
    pub version: String,
    /// The installed extension with its id ships with the app.
    pub core: bool,
    pub description: Option<String>,
    pub maintainers: Vec<String>,
    pub components: Vec<String>,
}

/// A file in the workspace: its name, media type (empty when unknown), size as Android words it,
/// whether it is named and typed as an extension archive, and the archive read from it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct File {
    pub name: String,
    pub media: String,
    pub size: String,
    pub flex: bool,
    pub archive: Option<Archive>,
}

/// The importer's own state, none of it saved: the route keeps the workspace.
#[derive(Debug, Default)]
pub struct Import {
    /// The workspace Kotlin last described, and its files.
    pub files: Option<(String, Vec<File>)>,
    /// The picked files could not be read.
    pub failed: bool,
}

/// Why the importer skips a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    Unsupported,
    Corrupted,
    Core,
    /// The archive holds a type of extension the importer on screen does not take. Compose
    /// listed it as importable, skipped it and said the import worked.
    OtherType,
}
impl Skip {
    pub fn key(self) -> &'static str {
        match self {
            Skip::Unsupported => "ext__import__file_skip_unsupported",
            Skip::Corrupted => "ext__import__file_skip_ext_corrupted",
            Skip::Core => "ext__import__file_skip_ext_core",
            Skip::OtherType => "settings__ext__skip_other_type",
        }
    }
}

/// The type `ExtensionImportScreenType` an importer route names: its id and its serial name,
/// none for any.
fn requested(base: &str) -> (&'static str, Option<&'static str>) {
    match base {
        KEYBOARDS => ("ext-keyboard", Some(KEYBOARD)),
        THEMES => ("ext-theme", Some(THEME)),
        PACKS => ("ext-languagepack", Some(LANGUAGE_PACK)),
        _ => ("ext-any", None),
    }
}

/// Why the importer at `base` skips `f`, in ExtensionImportScreen's order: not typed as an
/// archive, no manifest read from it, replacing an extension the app ships, then of a type the
/// importer does not take. None for a file it imports.
pub fn skip(base: &str, f: &File) -> Option<Skip> {
    let Some(a) = f.archive.as_ref().filter(|_| f.flex) else {
        return Some(if f.flex {
            Skip::Corrupted
        } else {
            Skip::Unsupported
        });
    };
    if a.core {
        return Some(Skip::Core);
    }
    let serial: Vec<u16> = a.serial.encode_utf16().collect();
    let id: Vec<u16> = requested(base).0.encode_utf16().collect();
    let takes = Kind::of(&serial).is_some_and(|k| accepts(&id, k));
    (!takes).then_some(Skip::OtherType)
}

/// The importer route `route` is on: its base and its query's key and value.
pub fn parts(route: &str) -> Option<(&'static str, Option<(&str, &str)>)> {
    let (base, query) = route
        .split_once('?')
        .map_or((route, None), |(b, q)| (b, Some(q)));
    let base = [ANY, KEYBOARDS, THEMES, PACKS]
        .into_iter()
        .find(|r| *r == base)?;
    Some((base, query.and_then(|q| q.split_once('='))))
}

/// The catalogue route of the importer at `route`: its route without the query.
pub fn screen_route(route: &str) -> Option<&'static str> {
    parts(route).map(|(base, _)| base)
}

/// An `EXTENSION_FILES` report: the workspace's id and whether it is open, then its files.
pub fn parse(p: &[String]) -> Result<(String, bool, Vec<File>), &'static str> {
    if p.len() < row::SIZE || !p.len().is_multiple_of(row::SIZE) {
        return Err("settings import files");
    }
    let head = &p[..row::SIZE];
    if head[row::KIND] != file::WORKSPACE {
        return Err("settings import workspace");
    }
    let mut files: Vec<File> = vec![];
    for f in p[row::SIZE..].chunks(row::SIZE) {
        let at = |i: usize| f[i].clone();
        if f[row::KIND] == file::FILE {
            let flex = f[row::FLAG] == wire::flag::TRUE;
            let (name, media, size) = (at(row::LABEL), at(row::DETAIL), at(row::TEXT));
            files.push(File {
                name,
                media,
                size,
                flex,
                archive: None,
            });
            continue;
        }
        let owner = files.last_mut().ok_or("settings import file")?;
        if f[row::KIND] == file::ARCHIVE {
            owner.archive = Some(Archive {
                id: at(row::EXTENSION),
                serial: at(row::ID),
                title: at(row::LABEL),
                version: at(row::DETAIL),
                core: f[row::FLAG] == wire::flag::TRUE,
                ..Default::default()
            });
            continue;
        }
        let a = owner.archive.as_mut().ok_or("settings import archive")?;
        match f[row::KIND].as_str() {
            file::DESCRIPTION => a.description = Some(at(row::TEXT)),
            file::MAINTAINER => a.maintainers.push(at(row::LABEL)),
            file::COMPONENT => a.components.push(at(row::ID)),
            _ => return Err("settings import kind"),
        }
    }
    let open = head[row::FLAG] == wire::flag::TRUE;
    Ok((head[row::EXTENSION].clone(), open, files))
}

impl App {
    /// The importer on top: its base and its query's key and value.
    pub fn importer(&self) -> Option<(&'static str, Option<(&str, &str)>)> {
        parts(self.stack.last()?)
    }
    /// The workspace the importer on top shows.
    pub fn import_workspace(&self) -> Option<&str> {
        match self.importer()?.1? {
            ("uuid" | "files", id) => Some(id),
            _ => None,
        }
    }
    /// Select files shows unless the importer was opened for a shared file, as Compose hid it
    /// when its route carried a workspace.
    pub fn shows_select(&self) -> bool {
        self.importer()
            .is_some_and(|(_, q)| !q.is_some_and(|(k, _)| k == "uuid" || k == "document"))
    }
    /// The files of the workspace on screen, each with why it is skipped.
    pub fn import_files(&self) -> Vec<(File, Option<Skip>)> {
        let (Some(base), Some(id)) = (self.importer().map(|i| i.0), self.import_workspace()) else {
            return vec![];
        };
        match &self.extensions.import.files {
            Some((uuid, files)) if uuid == id => {
                files.iter().map(|f| (f.clone(), skip(base, f))).collect()
            }
            _ => vec![],
        }
    }
    /// Whether Import does something: a file can be imported and no file work runs.
    pub fn import_ready(&self) -> bool {
        self.files.busy.is_none() && self.import_files().iter().any(|(_, s)| s.is_none())
    }
    /// Gives the importer on top the query `query`, or none.
    fn set_import_query(&mut self, query: Option<(&str, &str)>) {
        let Some(base) = self.importer().map(|i| i.0) else {
            return;
        };
        let route = query.map_or(base.to_owned(), |(k, v)| format!("{base}?{k}={v}"));
        if let Some(top) = self.stack.last_mut() {
            *top = route;
        }
    }
    /// The importer opened, or came back with the activity: a shared file is read, and a
    /// workspace described.
    pub(super) fn import_opened(&mut self) {
        self.extensions.import = Import::default();
        match self.importer().and_then(|i| i.1) {
            Some(("document", handle)) if self.files.busy.as_deref() != Some(job::EXT_READ) => {
                let handle = handle.to_owned();
                self.work(job::EXT_READ, &[(work::DOCUMENT, handle)]);
            }
            Some(("uuid" | "files", id)) => {
                let id = id.to_owned();
                self.effects.push(vec![effect::FILES.into(), id]);
            }
            _ => {}
        }
    }
    /// An `EXTENSION_FILES` report. One for a workspace no longer on screen changes nothing; a
    /// closed one, as after Android ended the app, leaves the importer with nothing selected.
    pub fn set_import_files(&mut self, p: &[String]) -> Result<(), &'static str> {
        let (id, open, files) = parse(p)?;
        if self.import_workspace() != Some(id.as_str()) {
            return Ok(());
        }
        if !open {
            self.set_import_query(None);
            self.extensions.import.files = None;
            return Ok(());
        }
        self.extensions.import.files = Some((id, files));
        Ok(())
    }
    /// Select files: GetMultipleContents.
    pub(super) fn select_files(&mut self) {
        self.pick(PICK, mode::OPEN_MANY, &["*/*"], "");
    }
    /// Import: the files it can import, by their place in the workspace.
    pub(in crate::settings_ui) fn import_selected(&mut self) {
        if !self.import_ready() {
            return;
        }
        let Some(id) = self.import_workspace().map(str::to_owned) else {
            return;
        };
        let files = self.import_files();
        let ok = files.iter().enumerate().filter(|(_, (_, s))| s.is_none());
        let at: Vec<String> = ok.map(|(i, _)| i.to_string()).collect();
        self.work(
            job::EXT_IMPORT,
            &[(work::EXTENSION, id), (work::TEXT, at.join(","))],
        );
    }
    /// Export: CreateDocument for a `.flex` named after the extension, from its page.
    pub(super) fn export(&mut self, id: &str) {
        self.pick(
            &format!("{EXPORT}{id}"),
            mode::CREATE,
            &[FLEX_MIME],
            &flex_name(id),
        );
    }
    /// Closes the workspace `id`, deleting what was read into it.
    fn close_workspace(&mut self, id: &str) {
        self.effects.push(vec![effect::CLOSE.into(), id.into()]);
    }
    /// The picker closed for Select files or Export; false when `purpose` is neither's.
    pub(in crate::settings_ui) fn import_picked(&mut self, purpose: &str, answer: &Picked) -> bool {
        if let Some(id) = purpose.strip_prefix(EXPORT) {
            if let Picked::Documents(d) = answer {
                let fields = [
                    (work::DOCUMENT, d[0].handle.clone()),
                    (work::EXTENSION, id.into()),
                ];
                self.work(job::EXT_EXPORT, &fields);
            }
        } else if purpose != PICK {
            return false;
        } else if let Picked::Documents(d) = answer
            && self.importer().is_some()
        {
            if let Some(old) = self.import_workspace().map(str::to_owned) {
                self.close_workspace(&old);
            }
            self.set_import_query(None);
            self.extensions.import = Import::default();
            let handles: Vec<&str> = d.iter().map(|d| d.handle.as_str()).collect();
            self.work(job::EXT_READ, &[(work::DOCUMENT, handles.join("|"))]);
        }
        // Compose said nothing when the picker was dismissed.
        if *answer == Picked::NoPicker {
            self.toast(self.label("settings__udm__no_picker"));
        }
        true
    }
    /// File work ended; false when it is not the importer's or Export's.
    pub(in crate::settings_ui) fn import_done(&mut self, p: &[String]) -> bool {
        let ok = p[done::RESULT] == result::OK;
        match p[done::JOB].as_str() {
            job::EXT_READ => self.read_done(ok, &p[done::NAME]),
            job::EXT_IMPORT if ok => {
                self.toast(self.label("ext__import__success"));
                if self.importer().is_some() {
                    self.back();
                }
            }
            job::EXT_IMPORT => {
                if p[done::RESULT] == result::GONE && self.importer().is_some() {
                    self.set_import_query(None);
                    self.extensions.import.files = None;
                }
                self.failure_toast("ext__import__failure", &p[done::RESULT], true);
            }
            job::EXT_EXPORT if ok => self.toast(self.label("ext__export__success")),
            job::EXT_EXPORT => self.failure_toast("ext__export__failure", &p[done::RESULT], false),
            _ => return false,
        }
        true
    }
    /// The picked or shared files were read into workspace `id`, or could not be.
    fn read_done(&mut self, ok: bool, id: &str) {
        let Some((_, query)) = self.importer() else {
            if ok {
                self.close_workspace(id);
            }
            return;
        };
        let shared = query.is_some_and(|(k, _)| k == "document");
        if !ok {
            self.set_import_query(None);
            self.extensions.import.failed = true;
            return;
        }
        self.set_import_query(Some((if shared { "uuid" } else { "files" }, id)));
        self.extensions.import.failed = false;
        self.effects.push(vec![effect::FILES.into(), id.into()]);
    }
    /// `key`'s toast, its `{error_message}` what went wrong in plain words; Compose put the
    /// exception's message there, which can hold a file's address.
    fn failure_toast(&mut self, key: &str, outcome: &str, import: bool) {
        let reason = match outcome {
            result::UNWRITABLE => "settings__ext__unwritable",
            result::GONE if import => "settings__ext__files_gone",
            _ => "error__snackbar_message",
        };
        let text = self
            .label(key)
            .replace("{error_message}", &self.label(reason));
        self.toast(text);
    }
    /// Work Rust waited for was lost with the activity; false when it was not the importer's or
    /// Export's.
    pub(in crate::settings_ui) fn import_lost(&mut self, lost: &str) -> bool {
        if ![job::EXT_READ, job::EXT_IMPORT, job::EXT_EXPORT].contains(&lost) {
            return false;
        }
        if lost == job::EXT_READ && self.importer().is_some() {
            self.set_import_query(None);
        }
        self.toast(self.label("settings__files__interrupted"));
        true
    }
    /// A screen was closed: an importer's workspace closes with it, however it was left.
    pub(in crate::settings_ui) fn extensions_left(&mut self, left: &str) {
        self.ext_editor_left(left);
        if let Some((_, Some(("uuid" | "files", id)))) = parts(left) {
            let id = id.to_owned();
            self.close_workspace(&id);
        }
        if parts(left).is_some() {
            self.extensions.import = Import::default();
        }
    }
}
