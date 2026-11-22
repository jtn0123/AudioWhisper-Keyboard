//! The editor's part of the saved state: its route and workspace, the draft as it stands, whether
//! it changed, and an open sheet with what it holds, so turning the screen or Android ending the
//! app mid-edit loses nothing; Compose kept only the workspace, in memory, and lost it with the
//! process. Kotlin's report is asked for again with the draft (`ext_editor_restored`). A state
//! whose workspace had not opened keeps nothing: the editor asks for a new one.
use super::{CompForm, ExtEditor, FileForm, Load, MetaForm, draft::Comp, target};
use crate::settings_ui::App;
use serde_json::{Value, json};

const ERROR: &str = "settings saved extension editor";

fn comp_value(c: &Comp) -> Value {
    json!({"id": c.id, "label": c.label, "authors": c.authors, "night": c.night,
        "flags": c.flags, "path": c.path})
}
fn text(v: &Value, key: &str) -> Result<String, &'static str> {
    v.get(key)
        .and_then(Value::as_str)
        .map(Into::into)
        .ok_or(ERROR)
}
fn flag(v: &Value, key: &str) -> Result<bool, &'static str> {
    v.get(key).and_then(Value::as_bool).ok_or(ERROR)
}
fn texts<const N: usize>(v: &Value, key: &str) -> Result<[String; N], &'static str> {
    let all = v.get(key).and_then(Value::as_array).ok_or(ERROR)?;
    let all: Vec<String> = all
        .iter()
        .map(|s| s.as_str().map(Into::into).ok_or(ERROR))
        .collect::<Result<_, _>>()?;
    all.try_into().map_err(|_| ERROR)
}
fn index(v: &Value, key: &str) -> Result<Option<usize>, &'static str> {
    match v.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(n) => n.as_u64().map(|n| Some(n as usize)).ok_or(ERROR),
    }
}
fn read_comp(v: &Value) -> Result<Comp, &'static str> {
    let authors = v.get("authors").and_then(Value::as_array).ok_or(ERROR)?;
    Ok(Comp {
        id: text(v, "id")?,
        label: text(v, "label")?,
        authors: authors
            .iter()
            .map(|a| a.as_str().map(Into::into).ok_or(ERROR))
            .collect::<Result<_, _>>()?,
        night: flag(v, "night")?,
        flags: texts(v, "flags")?,
        path: text(v, "path")?,
    })
}
fn read_meta_form(v: &Value) -> Result<MetaForm, &'static str> {
    Ok(MetaForm {
        fields: texts(v, "fields")?,
        opened: texts(v, "opened")?,
        checked: flag(v, "checked")?,
    })
}
fn read_comp_form(v: &Value) -> Result<CompForm, &'static str> {
    let pending = v
        .get("pending")
        .filter(|p| !p.is_null())
        .map(read_comp)
        .transpose()?;
    Ok(CompForm {
        empty: flag(v, "empty")?,
        selected: index(v, "selected")?,
        fields: texts(v, "fields")?,
        checked: flag(v, "checked")?,
        pending,
    })
}
fn read_file_form(v: &Value) -> Result<FileForm, &'static str> {
    let original = v.get("original").and_then(Value::as_str).map(Into::into);
    Ok(FileForm {
        folder: index(v, "folder")?.ok_or(ERROR)?.min(1),
        name: text(v, "name")?,
        original,
        document: text(v, "document")?,
        checked: flag(v, "checked")?,
    })
}
/// `v`'s `key`, read by `read` unless it is null or missing.
fn form<T>(
    v: &Value,
    key: &str,
    read: fn(&Value) -> Result<T, &'static str>,
) -> Result<Option<T>, &'static str> {
    v.get(key).filter(|f| !f.is_null()).map(read).transpose()
}

impl App {
    /// The editor's saved part: empty unless a workspace is open.
    pub fn ext_editor_saved(&self) -> String {
        let e = &self.extensions.editor;
        if e.workspace.is_empty() || e.load != Load::Ready {
            return String::new();
        }
        let meta = e
            .meta_form
            .as_ref()
            .map(|f| json!({"fields": f.fields, "opened": f.opened, "checked": f.checked}));
        let comp = e.comp_form.as_ref().map(|f| {
            json!({"empty": f.empty, "selected": f.selected, "fields": f.fields, "checked": f.checked,
                "pending": f.pending.as_ref().map(comp_value)})
        });
        let file = e.file_form.as_ref().map(|f| {
            json!({"folder": f.folder, "name": f.name, "original": f.original, "document": f.document,
                "checked": f.checked})
        });
        let all = json!({"route": e.route, "workspace": e.workspace, "draft": e.draft.raw_json(),
            "changed": e.changed, "meta": meta, "comp": comp, "file": file});
        all.to_string()
    }
    /// Reads a saved part into the editor's state, its workspace to be asked for again; nothing
    /// changes on an error.
    pub fn restore_ext_editor(&mut self, s: &str) -> Result<(), &'static str> {
        if s.is_empty() {
            self.extensions.editor = ExtEditor::default();
            return Ok(());
        }
        let v: Value = serde_json::from_str(s).map_err(|_| ERROR)?;
        let route = text(&v, "route")?;
        let workspace = text(&v, "workspace")?;
        if target(&route).is_none() || workspace.is_empty() {
            return Err(ERROR);
        }
        let editor = ExtEditor {
            route,
            workspace,
            draft: super::Draft::parse(&text(&v, "draft")?)?,
            changed: flag(&v, "changed")?,
            meta_form: form(&v, "meta", read_meta_form)?,
            comp_form: form(&v, "comp", read_comp_form)?,
            file_form: form(&v, "file", read_file_form)?,
            ..ExtEditor::default()
        };
        self.extensions.editor = editor;
        Ok(())
    }
}
