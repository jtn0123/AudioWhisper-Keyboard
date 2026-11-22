//! The theme editor's part of the saved state: the draft and what it started from, so turning
//! the screen keeps unsaved edits, and an open rule or meta editor with what it holds, as
//! Compose's `rememberSaveable` kept its dialogs. Kotlin's report is asked for again, and the
//! draft replaces the stylesheet and meta it brings. The deleted rule or property is not kept:
//! Undo goes with the activity, as the dictionary's does.
use super::{
    Load, META, Meta, RULE_ADD, RULE_EDIT, ThemeEditor,
    form::{EXISTS, INVALID_CODE, INVALID_FONT, RuleForm},
    meta::MetaForm,
    rules::ordered,
    target,
};
use crate::settings_ui::App;
use snygg::{editor::Editor, grammar, json::parse, text};

/// What separates a part's fields, and the parts. The stylesheet JSON escapes both.
const FIELD: char = '\u{1f}';
const PART: char = '\u{1e}';

/// A draft a saved state brought back.
#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub target: (String, usize),
    /// The draft and the stylesheet as loaded or last saved, as JSON.
    pub rules: String,
    pub base: String,
    pub meta: Meta,
    pub base_meta: Meta,
    pub form: Option<RuleForm>,
    pub meta_form: Option<MetaForm>,
    pub keyboard_was: bool,
}

fn clean(s: &str) -> String {
    s.replace([FIELD, PART], "")
}
fn lossy(t: &[u16]) -> String {
    String::from_utf16_lossy(t)
}
fn flag(s: &str) -> Result<bool, &'static str> {
    s.parse().map_err(|_| "settings saved theme editor flag")
}
fn meta_part(m: &Meta) -> String {
    let fields = [
        clean(&m.label),
        clean(&m.authors.join("\n")),
        m.night.to_string(),
    ];
    fields.join(&FIELD.to_string())
}
fn read_meta(s: &str) -> Result<Meta, &'static str> {
    let [label, authors, night] = s.split(FIELD).collect::<Vec<_>>()[..] else {
        return Err("settings saved theme editor meta");
    };
    let authors = authors.lines().map(Into::into).collect();
    Ok(Meta {
        label: label.into(),
        authors,
        night: flag(night)?,
    })
}
fn rule_text(rule: &Option<grammar::Rule>) -> String {
    rule.as_ref()
        .map_or(String::new(), |r| clean(&lossy(&r.format())))
}
fn read_rule(s: &str) -> Result<Option<grammar::Rule>, &'static str> {
    if s.is_empty() {
        return Ok(None);
    }
    let rule = grammar::parse(&text(s)).map_err(|_| "settings saved theme editor rule")?;
    rule.map(Some).ok_or("settings saved theme editor rule")
}
fn form_part(f: &RuleForm) -> String {
    let fields = [
        rule_text(&f.original),
        rule_text(&f.source),
        clean(&f.element),
        clean(&f.font),
        f.selector.to_string(),
        clean(&lossy(&f.attributes.format())),
        clean(&f.codes),
        f.error.into(),
        f.checked.to_string(),
    ];
    fields.join(&FIELD.to_string())
}
fn read_form(s: &str) -> Result<Option<RuleForm>, &'static str> {
    if s.is_empty() {
        return Ok(None);
    }
    let fields: Vec<&str> = s.split(FIELD).collect();
    let [
        original,
        source,
        element,
        font,
        selector,
        attributes,
        codes,
        error,
        checked,
    ] = fields[..]
    else {
        return Err("settings saved theme editor form");
    };
    let selector = selector
        .parse::<usize>()
        .ok()
        .filter(|s| *s < grammar::SELECTORS.len());
    let attributes = grammar::parse_attributes(&text(attributes));
    let error = [EXISTS, INVALID_CODE, INVALID_FONT, ""]
        .into_iter()
        .find(|k| *k == error);
    let (Some(selector), Ok(attributes), Some(error)) = (selector, attributes, error) else {
        return Err("settings saved theme editor form");
    };
    Ok(Some(RuleForm {
        original: read_rule(original)?,
        source: read_rule(source)?,
        element: element.into(),
        font: font.into(),
        selector,
        attributes,
        codes: codes.into(),
        error,
        checked: flag(checked)?,
    }))
}
fn meta_form_part(f: &MetaForm) -> String {
    let fields = [
        clean(&f.label),
        clean(&f.authors),
        f.night.to_string(),
        f.checked.to_string(),
    ];
    fields.join(&FIELD.to_string())
}
fn read_meta_form(s: &str) -> Result<Option<MetaForm>, &'static str> {
    if s.is_empty() {
        return Ok(None);
    }
    let [label, authors, night, checked] = s.split(FIELD).collect::<Vec<_>>()[..] else {
        return Err("settings saved theme editor meta form");
    };
    Ok(Some(MetaForm {
        label: label.into(),
        authors: authors.into(),
        night: flag(night)?,
        checked: flag(checked)?,
    }))
}

impl App {
    /// The editor's saved part: empty unless a loaded component is being edited.
    pub fn theme_editor_saved(&self) -> String {
        let e = &self.theme_editor;
        let (Some((workspace, index)), Load::Ready) = (&e.target, &e.load) else {
            return String::new();
        };
        let base = e.base.as_ref();
        let base_sheet = ThemeEditor {
            rules: base.map_or(vec![], |b| b.0.clone()),
            schema: e.schema.clone(),
            ..ThemeEditor::default()
        };
        let (Ok(rules), Ok(base_json)) = (e.json(), base_sheet.json()) else {
            return String::new();
        };
        let base_meta = base.map_or(Meta::default(), |b| b.1.clone());
        let parts = [
            clean(workspace),
            index.to_string(),
            rules,
            base_json,
            meta_part(&e.meta),
            meta_part(&base_meta),
            e.form.as_ref().map_or(String::new(), form_part),
            e.meta_form.as_ref().map_or(String::new(), meta_form_part),
            e.keyboard_was.to_string(),
        ];
        parts.join(&PART.to_string())
    }
    /// Reads a saved part into `restored`, for the report to come; nothing changes on an error.
    pub fn restore_theme_editor(&mut self, s: &str) -> Result<(), &'static str> {
        if s.is_empty() {
            self.theme_editor.restored = None;
            return Ok(());
        }
        let parts: Vec<&str> = s.split(PART).collect();
        let [
            workspace,
            index,
            rules,
            base,
            meta,
            base_meta,
            form,
            meta_form,
            keyboard,
        ] = parts[..]
        else {
            return Err("settings saved theme editor");
        };
        let index = index
            .parse()
            .map_err(|_| "settings saved theme editor index")?;
        self.theme_editor.restored = Some(Draft {
            target: (workspace.into(), index),
            rules: rules.into(),
            base: base.into(),
            meta: read_meta(meta)?,
            base_meta: read_meta(base_meta)?,
            form: read_form(form)?,
            meta_form: read_meta_form(meta_form)?,
            keyboard_was: flag(keyboard)?,
        });
        Ok(())
    }
    /// After a saved state came back: the component named by the stack is asked for again, and
    /// a draft for it waits for the report.
    pub fn theme_editor_restored(&mut self, draft: Option<Draft>) {
        let at = self.stack.iter().find_map(|r| target(r));
        let Some((workspace, index)) = at.map(|(w, i)| (w.to_owned(), i)) else {
            self.theme_editor = ThemeEditor::default();
            return;
        };
        let restored = draft.filter(|d| d.target == (workspace.clone(), index));
        // An editor over the rules comes back only with what it held.
        let held = restored
            .as_ref()
            .is_some_and(|d| match self.stack.last().map(String::as_str) {
                Some(RULE_ADD | RULE_EDIT) => d.form.is_some(),
                Some(META) => d.meta_form.is_some(),
                _ => true,
            });
        if !held {
            self.stack.truncate(1);
            self.scrolls.truncate(1);
            self.modal = None;
        }
        self.theme_editor = ThemeEditor {
            target: Some((workspace, index)),
            restored,
            ..ThemeEditor::default()
        };
        self.request_component();
    }
    /// The report came after a saved state: the draft replaces its stylesheet and meta.
    pub(super) fn resume_draft(&mut self, d: Draft) {
        let sheet = |json: &str| parse(json, false).or_else(|_| parse(json, true)).ok();
        let (Some((schema, rules)), Some((_, base))) = (sheet(&d.rules), sheet(&d.base)) else {
            return self.load_stylesheet(None);
        };
        let e = &mut self.theme_editor;
        e.schema = schema;
        e.rules = ordered(Editor::new(rules).sheet.rules);
        e.base = Some((ordered(Editor::new(base).sheet.rules), d.base_meta));
        e.meta = d.meta;
        e.form = d.form;
        e.meta_form = d.meta_form;
        e.keyboard_was = d.keyboard_was;
        e.load = Load::Ready;
        self.theme_preview();
    }
}
