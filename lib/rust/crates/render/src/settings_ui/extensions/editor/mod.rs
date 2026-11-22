//! The extension editor, ExtensionEditScreen, ExtensionEditFilesScreen and ExtensionComponentView
//! drawn by Rust at Compose's `ext/edit/<id>` route and the theme list's Create. Rust holds the
//! draft (the meta, the dependencies and the themes), decides every screen, check and message,
//! and writes `extension.json` (`draft.rs`). Kotlin keeps the editor's workspace, the files the
//! extension is made of, with a copy of the draft for the theme editor, which opens over this one
//! as it did over Compose's (`wire::settings_ui::ext_editor`); saving and changing files are file
//! work (`files.rs`). The sheets Compose switched between are screens: `forms.rs` holds what
//! they type, `paint.rs` draws them, and `saved.rs` keeps it all over recreation.
pub mod draft;
mod forms;
mod paint;
mod saved;
mod sheets;
mod work;

use super::import::flex_name;
use crate::settings_ui::{App, catalogue::languages::Lang};
use draft::{Comp, Draft};
pub use forms::{CompForm, FileForm, MetaForm};
pub use paint::{ADD_ID, COMP_ID, HEADER_ID, MANAGE_ID, NONE_ID, STATUS_ID};
pub use sheets::{
    DEPENDENCY_ID, EDITOR_ICONS, FILE_DELETE_ID, FIXED_ID, FOLDER_ID, FROM_ID, NO_DEPENDENCIES_ID,
    NOTE_ID, SOURCE_ID,
};
use wire::settings_ui::ext_editor::{effect, folder, kind, open, outcome, row};

pub const EDIT: &str = "ext/edit";
pub const META: &str = "ext/editor/meta";
pub const DEPENDENCIES: &str = "ext/editor/dependencies";
pub const FILES: &str = "ext/editor/files";
pub const FILE: &str = "ext/editor/file";
pub const FILE_ADD: &str = "ext/editor/file/add";
pub const COMPONENT_ADD: &str = "ext/editor/component/add";
/// The folders ExtensionEditFilesScreen lists, in its order.
pub const FOLDERS: [&str; 2] = [folder::FONTS, folder::IMAGES];
/// The media types each folder takes, ExtensionEditFilesScreen's `MIME_TYPES`.
pub const MIME: [&[&str]; 2] = [
    &[
        "font/*",
        "application/font-*",
        "application/x-font-*",
        "application/vnd.ms-fontobject",
    ],
    &["image/*"],
];
/// The picker's purpose for a folder's Add: this, then the folder's index.
pub const PICK: &str = "extEditor:";
/// The strings the editor shows beyond its rows' labels.
pub const KEYS: [&str; 24] = [
    "ext__editor__title_create_theme",
    "ext__editor__title_edit_theme",
    "ext__editor__metadata__title",
    "ext__editor__dependencies__title",
    "ext__editor__files__title",
    "ext__editor__create_component__from_existing",
    "ext__editor__create_component__from_empty",
    "ext__editor__metadata__message_invalid",
    "action__discard",
    "action__save",
    "settings__theme_editor__component_id_exists",
    "settings__ext_editor__opening",
    "settings__ext_editor__unpack_failed",
    "settings__ext_editor__gone",
    "settings__ext_editor__no_dependencies",
    "settings__ext_editor__file_name_invalid",
    "settings__ext_editor__file_exists",
    "settings__ext_editor__wrong_type",
    "settings__ext_editor__file_failed",
    "settings__ext_editor__copy_failed",
    "settings__ext_editor__save_failed",
    "settings__ext_editor__save_interrupted",
    "settings__ext_editor__id_taken",
    "settings__ext_editor__component_invalid",
];

/// The extension a route edits: an installed one, a new theme extension, or a new one of a type
/// the editor cannot make, which ExtensionEditScreen also showed as not found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    Edit(String),
    Create,
    Unknown,
}

/// The extension the editor route `route` edits, None for any other route. The theme list asks
/// `ext/edit/null?create=ime.extension.theme`; an extension's page `ext/edit/<id>`.
pub fn target(route: &str) -> Option<Target> {
    let (base, query) = route
        .split_once('?')
        .map_or((route, None), |(b, q)| (b, Some(q)));
    let id = base.strip_prefix("ext/edit/").filter(|id| !id.is_empty())?;
    let create = query.and_then(|q| q.split('&').find_map(|kv| kv.strip_prefix("create=")));
    Some(match create {
        None => Target::Edit(id.into()),
        Some(draft::THEME) => Target::Create,
        Some(_) => Target::Unknown,
    })
}

/// Whether the workspace is open, and if not, why.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Load {
    /// Kotlin was asked and has not answered.
    #[default]
    Waiting,
    Ready,
    /// No installed theme extension has the id, ExtensionNotFoundScreen's case.
    Missing,
    /// The archive could not be unpacked; Compose showed the stack trace.
    Failed,
    /// The workspace a saved state named is gone, or it closed while saving.
    Gone,
}

/// A theme component another installed extension has, which a new one can copy.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Source {
    pub extension: String,
    /// Its meta; its stylesheet path is empty, as the copy's is.
    pub comp: Comp,
    /// Where its archive keeps its stylesheet.
    pub path: String,
}

/// The extension editor's state.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ExtEditor {
    /// The editor route this state belongs to.
    pub route: String,
    /// Kotlin's workspace, empty until it opens one.
    pub workspace: String,
    pub load: Load,
    pub requested: bool,
    pub draft: Draft,
    /// Something changed since the extension opened: Back asks before leaving.
    pub changed: bool,
    /// The workspace's files: each folder's index and the name, in name order.
    pub files: Vec<(usize, String)>,
    pub sources: Vec<Source>,
    /// Every installed extension's id, which a new one cannot take.
    pub installed: Vec<String>,
    /// The Material You flags a new component has.
    pub defaults: [String; 3],
    pub meta_form: Option<MetaForm>,
    pub comp_form: Option<CompForm>,
    pub file_form: Option<FileForm>,
}

/// A tap on the editor's own controls.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tap {
    /// Metadata, Dependencies or Files.
    Manage(usize),
    AddComponent,
    DeleteComponent(usize),
    EditComponent(usize),
    /// A folder's Add, and a file's row: its folder and its place in the folder.
    AddFile(usize),
    OpenFile(usize, usize),
    /// The file properties' Delete.
    DeleteFile,
    /// Create component's From existing (true) or Empty.
    From(bool),
    Source(usize),
}

impl App {
    /// The editor route on the stack, nearest the top.
    pub fn editor_route(&self) -> Option<&str> {
        let mut all = self.stack.iter().rev();
        all.find(|r| target(r).is_some()).map(String::as_str)
    }
    /// The extension the editor on the stack edits.
    pub fn editor_target(&self) -> Option<Target> {
        target(self.editor_route()?)
    }
    /// Whether the editor's screen is on top with its extension open.
    pub fn ext_editor_ready(&self) -> bool {
        let top = self.stack.last().is_some_and(|r| target(r).is_some());
        top && self.extensions.editor.load == Load::Ready
    }
    /// Whether the editor's state is for the editor on the stack, with its extension open.
    pub fn editing(&self) -> bool {
        let e = &self.extensions.editor;
        e.load == Load::Ready && self.editor_route() == Some(e.route.as_str())
    }
    /// The editor's title: Create or Edit theme extension, as EditScreen's was for a theme.
    pub(in crate::settings_ui) fn ext_editor_title(&self) -> Option<String> {
        let top = self.stack.last()?;
        Some(self.label(match target(top)? {
            Target::Create => "ext__editor__title_create_theme",
            _ => "ext__editor__title_edit_theme",
        }))
    }
    /// The open edit's place on the stack, while Kotlin holds its workspace.
    fn open_edit(&self) -> Option<usize> {
        let e = &self.extensions.editor;
        let at = self.stack.iter().position(|r| *r == e.route)?;
        (!e.workspace.is_empty()).then_some(at)
    }
    /// The open edit's route when `route` is another editor's: it is saved or discarded first.
    pub(in crate::settings_ui) fn ext_editor_redirect(&self, route: &str) -> Option<String> {
        let e = &self.extensions.editor;
        let other = target(route).is_some() && route != e.route;
        (other && self.open_edit().is_some()).then(|| e.route.clone())
    }
    /// Whether going to `route`, a root or not, would take the open edit off the stack, as a
    /// settings link opened mid-edit would: it is pushed over the editor instead, where Compose's
    /// NavController replaced the stack and the draft was lost with its workspace left open.
    pub(in crate::settings_ui) fn ext_editor_kept(&self, route: &str, root: bool) -> bool {
        let Some(at) = self.open_edit() else {
            return false;
        };
        root || self
            .stack
            .iter()
            .position(|r| r == route)
            .is_some_and(|p| p < at)
    }
    /// The editor opened at `route`: a new one forgets the last and asks Kotlin for a workspace.
    pub(in crate::settings_ui) fn ext_editor_opened(&mut self, route: &str) {
        if target(route).is_none() || self.extensions.editor.route == route {
            return;
        }
        self.extensions.editor = ExtEditor {
            route: route.into(),
            ..ExtEditor::default()
        };
        self.request_workspace();
    }
    /// After a saved state came back: Kotlin's side is new, so the workspace is asked for again,
    /// to be taken up as it was.
    pub(in crate::settings_ui) fn ext_editor_restored(&mut self) {
        let Some(route) = self.editor_route().map(str::to_owned) else {
            self.extensions.editor = ExtEditor::default();
            return;
        };
        if self.extensions.editor.route != route {
            self.extensions.editor = ExtEditor::default();
            return self.ext_editor_opened(&route);
        }
        let e = &mut self.extensions.editor;
        (e.requested, e.load) = (false, Load::Waiting);
        self.request_workspace();
    }
    /// Asks Kotlin to open the workspace, or take up the one the state has.
    fn request_workspace(&mut self) {
        let Some(t) = self.editor_target() else {
            return;
        };
        let e = &mut self.extensions.editor;
        if e.requested {
            return;
        }
        e.requested = true;
        let id = match t {
            Target::Unknown => {
                e.load = Load::Missing;
                return;
            }
            Target::Create => String::new(),
            Target::Edit(id) => id,
        };
        let mut packet = vec![String::new(); open::SIZE];
        packet[open::EFFECT] = effect::OPEN.into();
        packet[open::EXTENSION] = id;
        if !e.workspace.is_empty() {
            packet[open::WORKSPACE] = e.workspace.clone();
            packet[open::MANIFEST] = e.draft.raw_json();
        }
        self.effects.push(packet);
    }
    /// An `EXT_EDITOR` report. One the editor did not ask for closes the workspace it opened.
    pub fn set_ext_editor(&mut self, p: &[String]) -> Result<(), &'static str> {
        if p.len() < row::SIZE || !p.len().is_multiple_of(row::SIZE) {
            return Err("settings extension editor");
        }
        let head = &p[..row::SIZE];
        if head[row::KIND] != kind::WORKSPACE {
            return Err("settings extension editor workspace");
        }
        let asked = match self.editor_target() {
            Some(Target::Edit(id)) => Some(id),
            Some(Target::Create) => Some(String::new()),
            _ => None,
        };
        let ws = head[row::WORKSPACE].clone();
        let e = &self.extensions.editor;
        let mine = asked.as_deref() == Some(head[row::EXTENSION].as_str())
            && self.editor_route() == Some(e.route.as_str())
            && (e.workspace.is_empty() || e.workspace == ws);
        if !mine {
            if head[row::ID] == outcome::READY && !ws.is_empty() && ws != e.workspace {
                self.effects.push(vec![effect::CLOSE.into(), ws]);
            }
            return Ok(());
        }
        let load = match head[row::ID].as_str() {
            outcome::READY => return self.workspace_ready(head, &p[row::SIZE..]),
            outcome::MISSING => Load::Missing,
            outcome::FAILED => Load::Failed,
            outcome::GONE => Load::Gone,
            _ => return Err("settings extension editor outcome"),
        };
        let e = &mut self.extensions.editor;
        (e.load, e.workspace) = (load, String::new());
        Ok(())
    }
    /// The workspace is open: a new one's draft is the extension, or a new theme extension; one
    /// taken up again keeps the draft unless the theme editor changed Kotlin's copy.
    fn workspace_ready(&mut self, head: &[String], rows: &[String]) -> Result<(), &'static str> {
        let [ws, text] = [&head[row::WORKSPACE], &head[row::TEXT]];
        let flags: Vec<String> = head[row::DETAIL].split('/').map(Into::into).collect();
        let defaults: [String; 3] = flags
            .try_into()
            .map_err(|_| "settings extension editor flags")?;
        let (files, sources, installed) = report_rows(rows)?;
        let e = &mut self.extensions.editor;
        let fresh = e.workspace.is_empty();
        if fresh {
            e.draft = match text.is_empty() {
                true => Draft::fresh(&head[row::LABEL]),
                false => Draft::parse(text)?,
            };
            e.changed = false;
        } else if head[row::FLAG] == wire::flag::TRUE {
            e.draft = Draft::parse(text)?;
            e.changed = true;
        }
        e.workspace = ws.clone();
        e.load = Load::Ready;
        (e.defaults, e.files, e.sources, e.installed) = (defaults, files, sources, installed);
        if fresh {
            self.ext_sync();
        }
        Ok(())
    }
    /// Kotlin's copy of the draft becomes the draft as it is now.
    pub(in crate::settings_ui) fn ext_sync(&mut self) {
        let e = &self.extensions.editor;
        if !e.workspace.is_empty() {
            let packet = vec![effect::SYNC.into(), e.workspace.clone(), e.draft.raw_json()];
            self.effects.push(packet);
        }
    }
    /// The draft or the files changed.
    pub(in crate::settings_ui) fn ext_changed(&mut self) {
        self.extensions.editor.changed = true;
        self.ext_sync();
    }
    /// Back on the editor with changes asks first, as EditScreen's `handleBackPress` did. While
    /// Save runs, Back waits for it: leaving would close the workspace under it.
    pub(in crate::settings_ui) fn ext_editor_back(&mut self) -> bool {
        let saving = self.files.busy.as_deref() == Some(wire::settings_ui::files::job::EXT_SAVE);
        if self.ext_editor_ready() && saving {
            return true;
        }
        let ask = self.ext_editor_ready() && self.extensions.editor.changed;
        if ask {
            self.open_lang_dialog(Lang::Unsaved, "");
        }
        ask
    }
    /// Whether the unsaved changes question is open.
    pub fn unsaved_open(&self) -> bool {
        use crate::settings_ui::catalogue::Kind;
        let m = self.modal.as_ref();
        m.is_some_and(|m| matches!(self.rows()[m.row].kind, Kind::Lang(Lang::Unsaved)))
    }
    /// The question's Discard: the editor closes without saving, and its workspace with it.
    pub(in crate::settings_ui) fn discard_extension(&mut self) {
        (self.modal, self.focus) = (None, -1);
        self.extensions.editor.changed = false;
        self.back();
    }
    /// A screen was closed: the editor's workspace closes with it, however it was left; a
    /// sheet's form goes with the sheet, as Compose's `currentAction = null` dropped it.
    pub(in crate::settings_ui) fn ext_editor_left(&mut self, left: &str) {
        let e = &mut self.extensions.editor;
        match left {
            META => e.meta_form = None,
            COMPONENT_ADD => e.comp_form = None,
            FILE | FILE_ADD => e.file_form = None,
            _ if target(left).is_some() && e.route == left => {
                if !e.workspace.is_empty() {
                    let ws = std::mem::take(&mut e.workspace);
                    self.effects.push(vec![effect::CLOSE.into(), ws]);
                }
                self.extensions.editor = ExtEditor::default();
            }
            _ => {}
        }
    }
    /// A tap on the editor's own control.
    pub(in crate::settings_ui) fn editor_tap(&mut self, tap: Tap) {
        if !self.editing() {
            return;
        }
        match tap {
            Tap::Manage(0) => {
                let f = MetaForm::of(&self.extensions.editor.draft.meta);
                self.extensions.editor.meta_form = Some(f);
                self.navigate(META);
            }
            Tap::Manage(1) => self.navigate(DEPENDENCIES),
            Tap::Manage(_) => self.navigate(FILES),
            Tap::AddComponent => {
                self.extensions.editor.comp_form = Some(CompForm::default());
                self.navigate(COMPONENT_ADD);
            }
            Tap::DeleteComponent(i) => {
                if i < self.extensions.editor.draft.themes.len() {
                    self.extensions.editor.draft.themes.remove(i);
                    self.ext_changed();
                }
            }
            Tap::EditComponent(i) => {
                let ws = self.extensions.editor.workspace.clone();
                self.effects
                    .push(vec![effect::THEME.into(), ws, i.to_string()]);
            }
            Tap::AddFile(f) => {
                let mime = MIME[f.min(1)];
                let how = wire::settings_ui::files::mode::OPEN;
                self.pick(&format!("{PICK}{}", f.min(1)), how, mime, "");
            }
            Tap::OpenFile(f, k) => self.open_file(f, k),
            Tap::DeleteFile => self.delete_file(),
            Tap::From(existing) => {
                if let Some(c) = self.extensions.editor.comp_form.as_mut() {
                    c.empty = !existing;
                }
            }
            Tap::Source(k) => {
                if let Some(c) = self.extensions.editor.comp_form.as_mut() {
                    c.selected = Some(k);
                }
            }
        }
    }
    /// Save: as EditScreen's `handleSave`, invalid metadata explains first; a theme that cannot
    /// be built, which crashed Compose, says which; a new extension may not take an installed
    /// one's id. Then Kotlin writes the archive, off the main thread and into place at once.
    /// Save while saving, or with nothing open, does nothing.
    pub(in crate::settings_ui) fn save_extension(&mut self) {
        if self.files.busy.is_some() || !self.editing() {
            return;
        }
        let e = &self.extensions.editor;
        if !forms::meta_valid(&e.draft.meta) {
            let key = "ext__editor__metadata__message_invalid";
            return self.open_lang_dialog(Lang::Message, key);
        }
        let json = match e.draft.json() {
            Ok(json) => json,
            Err((i, _)) => {
                let c = &e.draft.themes[i];
                let blank = c.id.trim().is_empty();
                let name = if blank { c.label.clone() } else { c.id.clone() };
                let text = self.label("settings__ext_editor__component_invalid");
                return self.toast(text.replace("{name}", &name));
            }
        };
        let id = e.draft.meta.id.clone();
        let original = match self.editor_target() {
            Some(Target::Edit(id)) => id,
            _ if e.installed.contains(&id) => {
                return self.toast(self.label("settings__ext_editor__id_taken"));
            }
            _ => String::new(),
        };
        use wire::settings_ui::files::{job, work};
        let fields = [
            (work::EXTENSION, e.workspace.clone()),
            (work::TEXT, json),
            (work::NAME, flex_name(&id)),
            (work::TARGET, original),
        ];
        self.work(job::EXT_SAVE, &fields);
    }
}

/// The rows after a report's workspace row: the files, the other extensions' themes and the
/// installed ids.
type Rows = (Vec<(usize, String)>, Vec<Source>, Vec<String>);
fn report_rows(rows: &[String]) -> Result<Rows, &'static str> {
    let (mut files, mut sources, mut installed) = (vec![], vec![], vec![]);
    for f in rows.chunks(row::SIZE) {
        let at = |i: usize| f[i].clone();
        match f[row::KIND].as_str() {
            kind::FILE => {
                let folder = FOLDERS.iter().position(|d| *d == f[row::ID]);
                files.push((
                    folder.ok_or("settings extension editor folder")?,
                    at(row::LABEL),
                ));
            }
            kind::THEME => {
                let flags: Vec<String> = f[row::DETAIL].split('/').map(Into::into).collect();
                let comp = Comp {
                    id: at(row::ID),
                    label: at(row::LABEL),
                    authors: f[row::TEXT].lines().map(Into::into).collect(),
                    night: f[row::FLAG] == wire::flag::TRUE,
                    flags: flags
                        .try_into()
                        .map_err(|_| "settings extension editor flags")?,
                    path: String::new(),
                };
                let (extension, path) = (at(row::EXTENSION), at(row::PATH));
                sources.push(Source {
                    extension,
                    comp,
                    path,
                });
            }
            kind::INSTALLED => installed.push(at(row::EXTENSION)),
            _ => return Err("settings extension editor kind"),
        }
    }
    Ok((files, sources, installed))
}
