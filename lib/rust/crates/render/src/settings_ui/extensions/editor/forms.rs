//! What the editor's sheets type and how they check it: ManageMetaDataScreen's fields, Create
//! component's choice and empty component, and a file's name. Every check is
//! `ExtensionValidation`'s rule from the `extensions` crate, its message shown under the field
//! once Apply or Create was tried, as `rememberValidationResult` showed it. The keywords and the
//! maintainers are one comma-separated line where Compose took one per line, as the settings
//! renderer's fields are single-line; a line left as it opened keeps its list exactly.
use super::{COMPONENT_ADD, FILE, FILE_ADD, FOLDERS, META, MIME, Source, draft};
use crate::settings_ui::{App, catalogue::ext_editor::Edit};
use draft::{Comp, Meta, maintainer_text};
use extensions::validation::validate;
use wire::extensions::{self as rules, INVALID, validation};
use wire::settings_ui::files::{job, work};

/// ManageMetaDataScreen's fields, as typed, in its order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetaForm {
    pub fields: [String; 9],
    /// The keywords' and the maintainers' lines as they opened.
    pub opened: [String; 2],
    /// Apply was tried: the fields' messages show.
    pub checked: bool,
}

/// What each meta field is checked by: the id, version, title, maintainers and license.
const META_RULES: [Option<&str>; 9] = [
    Some(rules::ID),
    Some(rules::VERSION),
    Some(rules::TITLE),
    None,
    None,
    None,
    None,
    Some(rules::MAINTAINERS),
    Some(rules::LICENSE),
];

/// A comma-separated line's items: trimmed, the blank ones dropped.
fn items(line: &str) -> Vec<String> {
    let all = line.split(',').map(str::trim);
    all.filter(|s| !s.is_empty()).map(Into::into).collect()
}
/// `s` trimmed, None when blank, as Apply kept an optional field.
fn optional(s: &str) -> Option<String> {
    Some(s.trim().to_owned()).filter(|s| !s.is_empty())
}

impl MetaForm {
    pub fn of(m: &Meta) -> Self {
        let keywords = m.keywords.as_deref().unwrap_or_default().join(", ");
        let maintainers = m.maintainers.join(", ");
        let o = |s: &Option<String>| s.clone().unwrap_or_default();
        Self {
            fields: [
                m.id.clone(),
                m.version.clone(),
                m.title.clone(),
                o(&m.description),
                keywords.clone(),
                o(&m.homepage),
                o(&m.issue_tracker),
                maintainers.clone(),
                m.license.clone(),
            ],
            opened: [keywords, maintainers],
            checked: false,
        }
    }
    /// The text field `k`'s rule checks: the maintainers one per line, as Compose took them.
    fn checked_text(&self, k: usize) -> String {
        match k {
            7 => items(&self.fields[7]).join("\n"),
            _ => self.fields[k].clone(),
        }
    }
    /// The meta Apply gives, ManageMetaDataScreen's `handleApply`: each field trimmed, the blank
    /// optional ones dropped, each maintainer as `fromOrTakeRaw` reads it. A list left as it
    /// opened is kept as it was.
    pub fn meta(&self, was: &Meta) -> Meta {
        let f = &self.fields;
        let keywords = match f[4] == self.opened[0] {
            true => was.keywords.clone(),
            false => Some(items(&f[4])).filter(|k| !k.is_empty()),
        };
        let maintainers = match f[7] == self.opened[1] {
            true => was.maintainers.clone(),
            false => items(&f[7]).iter().map(|m| maintainer_text(m)).collect(),
        };
        Meta {
            id: f[0].trim().into(),
            version: f[1].trim().into(),
            title: f[2].trim().into(),
            description: optional(&f[3]),
            keywords,
            homepage: optional(&f[5]),
            issue_tracker: optional(&f[6]),
            maintainers,
            license: f[8].trim().into(),
        }
    }
}

/// Whether `ExtensionMeta.validate()` passes: its id, version, title, maintainers and license.
pub fn meta_valid(m: &Meta) -> bool {
    let t = |s: &str| snygg::text(s);
    let input = [
        t(&m.id),
        t(&m.version),
        t(&m.title),
        t(&m.maintainers.join("\n")),
        t(&m.license),
    ];
    let out = extensions::call(rules::METADATA_VALID, &input).unwrap_or_default();
    out.first().is_some_and(|f| wire::flag::decode(f))
}

/// `rule`'s check of `value`: its message's string key with the `{name}` it fills in and the
/// pattern, and whether it refuses.
fn check(rule: &str, value: &str) -> (String, [String; 2], bool) {
    let row = validate(rule, &snygg::text(value));
    let field = |f: usize| String::from_utf16_lossy(&row[f]);
    let name = [field(validation::ARGUMENT), field(validation::PATTERN)];
    let refused = field(validation::STATUS) == INVALID;
    (field(validation::RESOURCE), name, refused)
}

/// Create component's state: From existing or Empty, the component chosen, and the empty one's
/// id, label and authors (comma-separated).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompForm {
    pub empty: bool,
    pub selected: Option<usize>,
    pub fields: [String; 3],
    pub checked: bool,
    /// The copy whose stylesheet is being copied, added once that is done.
    pub pending: Option<Comp>,
}

/// A file's name, on its properties or on adding a picked file to `folder`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FileForm {
    pub folder: usize,
    pub name: String,
    /// The file's name, for its properties; None when adding.
    pub original: Option<String>,
    /// The picked document's handle, when adding.
    pub document: String,
    pub checked: bool,
}

impl FileForm {
    /// The name as it is kept: trimmed when adding, as the import dialog trimmed it.
    pub fn kept(&self) -> String {
        match self.original {
            Some(_) => self.name.clone(),
            None => self.name.trim().into(),
        }
    }
}

/// Whether `name` stays in its folder, as Compose's `parentFile` checks asked.
fn file_name_ok(name: &str) -> bool {
    !name.is_empty() && !name.contains(['/', '\0']) && name != "." && name != ".."
}

impl App {
    /// The components Create component offers, in its order: the extension's own, then every
    /// other installed extension's.
    pub fn sources(&self) -> Vec<Source> {
        let e = &self.extensions.editor;
        let id = &e.draft.meta.id;
        let own = e.draft.themes.iter().map(|c| Source {
            extension: id.clone(),
            comp: c.clone(),
            path: c.stylesheet(),
        });
        let others = e.sources.iter().filter(|s| &s.extension != id).cloned();
        own.chain(others).collect()
    }
    /// Whether Create component makes an empty one.
    pub fn from_empty(&self) -> bool {
        self.extensions
            .editor
            .comp_form
            .as_ref()
            .is_some_and(|c| c.empty)
    }
    /// Create's `hasSufficientInfoForCreating`, while no file work runs.
    pub fn can_create_component(&self) -> bool {
        let Some(c) = self.extensions.editor.comp_form.as_ref() else {
            return false;
        };
        let typed = c.fields.iter().all(|f| !utilities::blank(&snygg::text(f)));
        let chosen = c.selected.is_some_and(|k| k < self.sources().len());
        self.files.busy.is_none() && if c.empty { typed } else { chosen }
    }
    /// The text of the editor's field `e`; None for one that is not a field here, as the id of
    /// an installed extension, which ManageMetaDataScreen showed disabled.
    pub(in crate::settings_ui) fn ext_line(&self, e: Edit) -> Option<String> {
        let x = &self.extensions.editor;
        match e {
            Edit::Meta(0) if self.editor_target() != Some(super::Target::Create) => None,
            Edit::Meta(k) => x.meta_form.as_ref().map(|f| f.fields[k.min(8)].clone()),
            Edit::New(k) => x.comp_form.as_ref().map(|f| f.fields[k.min(2)].clone()),
            Edit::FileName => x.file_form.as_ref().map(|f| f.name.clone()),
            _ => None,
        }
    }
    pub(in crate::settings_ui) fn ext_typed(&mut self, e: Edit, value: String) {
        if self.ext_line(e).is_none() {
            return;
        }
        let x = &mut self.extensions.editor;
        match e {
            Edit::Meta(k) => x.meta_form.as_mut().map(|f| f.fields[k.min(8)] = value),
            Edit::New(k) => x.comp_form.as_mut().map(|f| f.fields[k.min(2)] = value),
            Edit::FileName => x.file_form.as_mut().map(|f| f.name = value),
            _ => None,
        };
    }
    /// Field `e`'s message once its sheet was tried, translated, its pattern filled in.
    pub fn ext_error(&self, e: Edit) -> String {
        let x = &self.extensions.editor;
        let found = match e {
            Edit::Meta(k) => x
                .meta_form
                .as_ref()
                .filter(|f| f.checked)
                .and_then(|f| META_RULES[k.min(8)].map(|r| check(r, &f.checked_text(k)))),
            Edit::New(k) => x
                .comp_form
                .as_ref()
                .filter(|f| f.checked)
                .map(|f| self.new_check(f, k)),
            Edit::FileName => {
                let f = x.file_form.as_ref().filter(|f| f.checked);
                return f.map_or(String::new(), |f| {
                    self.label(self.file_problem(f).unwrap_or(""))
                });
            }
            _ => None,
        };
        let Some((key, [name, pattern], _)) = found.filter(|(k, ..)| !k.is_empty()) else {
            return String::new();
        };
        let text = self.label(&key);
        if name.is_empty() {
            return text;
        }
        text.replace(&format!("{{{name}}}"), &pattern)
    }
    /// The empty component's field `k`, checked; an id the extension has is refused under it,
    /// where Compose said it in an English toast.
    fn new_check(&self, f: &CompForm, k: usize) -> (String, [String; 2], bool) {
        let rule = [
            rules::COMPONENT_ID,
            rules::COMPONENT_LABEL,
            rules::COMPONENT_AUTHORS,
        ][k.min(2)];
        let value = if k == 2 {
            items(&f.fields[2]).join("\n")
        } else {
            f.fields[k].clone()
        };
        let checked = check(rule, &value);
        let id = f.fields[0].trim();
        let taken = self
            .extensions
            .editor
            .draft
            .themes
            .iter()
            .any(|t| t.id == id);
        if k == 0 && !checked.2 && taken {
            let key = super::super::super::theme_editor::meta::ID_TAKEN;
            return (key.into(), Default::default(), true);
        }
        checked
    }
    /// Why a file's name cannot be kept, as a string key: it leaves the folder, or another file
    /// has it. None for a name that can.
    fn file_problem(&self, f: &FileForm) -> Option<&'static str> {
        let name = f.kept();
        if !file_name_ok(&name) {
            return Some("settings__ext_editor__file_name_invalid");
        }
        let files = &self.extensions.editor.files;
        let other = f.original.as_deref() != Some(name.as_str());
        let taken = files.iter().any(|(d, n)| *d == f.folder && *n == name);
        (other && taken).then_some("settings__ext_editor__file_exists")
    }
    /// Whether the screen on top is the editor's or one of its sheets.
    pub(in crate::settings_ui) fn ext_sheet(&self) -> bool {
        let sheets = [META, COMPONENT_ADD, FILE, FILE_ADD];
        let top = self.stack.last().map_or("", String::as_str);
        sheets.contains(&top) || super::target(top).is_some()
    }
    /// The bottom bar's Save, Apply, Create or Add.
    pub(in crate::settings_ui) fn ext_confirm(&mut self) {
        match self.stack.last().map_or("", String::as_str) {
            META => self.apply_meta(),
            COMPONENT_ADD => self.create_component(),
            FILE | FILE_ADD => self.apply_file(),
            _ => self.save_extension(),
        }
    }
    /// Apply: the meta changes unless a field refuses, and the sheet closes.
    fn apply_meta(&mut self) {
        let creating = self.editor_target() == Some(super::Target::Create);
        let x = &mut self.extensions.editor;
        let Some(f) = x.meta_form.as_mut() else {
            return;
        };
        let refused = (0..9).any(|k| META_RULES[k].is_some_and(|r| check(r, &f.checked_text(k)).2));
        if refused {
            f.checked = true;
            return;
        }
        let mut meta = f.meta(&x.draft.meta);
        if !creating {
            meta.id = x.draft.meta.id.clone();
        }
        if meta != x.draft.meta {
            x.draft.meta = meta;
            self.ext_changed();
        }
        self.close_editor();
    }
    /// Create: an empty component is added at once; a copy once its stylesheet is copied, from
    /// the workspace or from the other extension's archive.
    fn create_component(&mut self) {
        if !self.can_create_component() {
            return;
        }
        let Some(f) = self.extensions.editor.comp_form.clone() else {
            return;
        };
        if f.empty {
            if (0..3).any(|k| self.new_check(&f, k).2) {
                if let Some(f) = self.extensions.editor.comp_form.as_mut() {
                    f.checked = true;
                }
                return;
            }
            let comp = Comp {
                id: f.fields[0].trim().into(),
                label: f.fields[1].trim().into(),
                authors: items(&f.fields[2]),
                night: true,
                flags: self.extensions.editor.defaults.clone(),
                path: String::new(),
            };
            self.extensions.editor.draft.themes.push(comp);
            self.ext_changed();
            return self.close_editor();
        }
        let Some(src) = f.selected.and_then(|k| self.sources().into_iter().nth(k)) else {
            return;
        };
        let e = &self.extensions.editor;
        let comp = Comp {
            id: unique_id(&e.draft.themes, &src.comp.id),
            path: String::new(),
            ..src.comp.clone()
        };
        let same = src.extension == e.draft.meta.id;
        let fields = [
            (work::EXTENSION, e.workspace.clone()),
            (work::TEXT, src.path.clone()),
            (
                work::TARGET,
                if same { String::new() } else { src.extension },
            ),
            (work::NAME, comp.stylesheet()),
        ];
        if let Some(f) = self.extensions.editor.comp_form.as_mut() {
            f.pending = Some(comp);
        }
        self.work(job::EXT_COMPONENT_COPY, &fields);
    }
    /// Apply on a file's properties renames it; Add copies the picked file in.
    fn apply_file(&mut self) {
        let Some(f) = self.extensions.editor.file_form.clone() else {
            return;
        };
        if self.files.busy.is_some() {
            return;
        }
        if self.file_problem(&f).is_some() {
            if let Some(f) = self.extensions.editor.file_form.as_mut() {
                f.checked = true;
            }
            return;
        }
        let ws = self.extensions.editor.workspace.clone();
        let path = |name: &str| format!("{}/{name}", FOLDERS[f.folder.min(1)]);
        let name = f.kept();
        match &f.original {
            Some(old) if *old == name => self.close_editor(),
            Some(old) => {
                let fields = [
                    (work::EXTENSION, ws),
                    (work::TEXT, path(old)),
                    (work::NAME, path(&name)),
                ];
                self.work(job::EXT_FILE_RENAME, &fields);
            }
            None => {
                let fields = [
                    (work::EXTENSION, ws),
                    (work::DOCUMENT, f.document.clone()),
                    (work::NAME, path(&name)),
                    (work::TEXT, MIME[f.folder.min(1)].join("|")),
                ];
                self.work(job::EXT_FILE_ADD, &fields);
            }
        }
    }
    /// A file's row: its properties, its name ready to change.
    pub(super) fn open_file(&mut self, folder: usize, k: usize) {
        let x = &self.extensions.editor;
        let Some(name) = x.files.iter().filter(|(d, _)| *d == folder).nth(k) else {
            return;
        };
        let original = Some(name.1.clone());
        let form = FileForm {
            folder,
            name: name.1.clone(),
            original,
            ..FileForm::default()
        };
        self.extensions.editor.file_form = Some(form);
        self.navigate(FILE);
    }
    /// The properties' Delete.
    pub(super) fn delete_file(&mut self) {
        let x = &self.extensions.editor;
        let Some((f, Some(name))) = x.file_form.as_ref().map(|f| (f.folder, f.original.clone()))
        else {
            return;
        };
        if self.files.busy.is_some() {
            return;
        }
        let fields = [
            (work::EXTENSION, x.workspace.clone()),
            (work::NAME, format!("{}/{name}", FOLDERS[f.min(1)])),
        ];
        self.work(job::EXT_FILE_DELETE, &fields);
    }
}

/// `id`, or with the first `_1`, `_2`, … suffix no theme has, as Create's copy named it.
pub fn unique_id(themes: &[Comp], id: &str) -> String {
    if !themes.iter().any(|t| t.id == id) {
        return id.into();
    }
    let mut n = 1;
    loop {
        let next = format!("{id}_{n}");
        if !themes.iter().any(|t| t.id == next) {
            return next;
        }
        n += 1;
    }
}
