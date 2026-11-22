//! The theme editor's part of the saved state: the draft and what it started from, so turning
//! the screen keeps unsaved edits, and an open rule, meta or property editor with what it holds, as
//! Compose's `rememberSaveable` kept its dialogs. Kotlin's report is asked for again, and the
//! draft replaces the stylesheet and meta it brings. The deleted rule or property is not kept:
//! Undo goes with the activity, as the dictionary's does.
use super::{
    Load, META, Meta, PROPERTY_ADD, PROPERTY_EDIT, RULE_ADD, RULE_EDIT, ThemeEditor,
    form::{EXISTS, INVALID_CODE, INVALID_FONT, RuleForm},
    meta::MetaForm,
    property::{self, PropertyForm},
    rules::ordered,
    target,
};
use crate::settings_ui::App;
use snygg::{editor::Editor, grammar, json::parse, text, value::Encoder};

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
    pub prop: Option<PropertyForm>,
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
/// The id, the label, the night flag, the path and the Material You flags: what a meta and a
/// meta form share, in that order.
fn meta_fields(id: &str, label: &str, night: bool, path: &str, flags: &[String; 3]) -> Vec<String> {
    let mut fields = vec![clean(id), clean(label), night.to_string(), clean(path)];
    fields.extend(flags.iter().map(|f| clean(f)));
    fields
}
/// Those fields read back, and the ones after them.
fn read_meta_fields<'a>(
    fields: &'a [&'a str],
    error: &'static str,
) -> Result<(Meta, &'a [&'a str]), &'static str> {
    let [id, label, night, path, palette, contrast, spec, rest @ ..] = fields else {
        return Err(error);
    };
    let meta = Meta {
        id: (*id).into(),
        label: (*label).into(),
        authors: vec![],
        night: flag(night)?,
        path: (*path).into(),
        flags: [(*palette).into(), (*contrast).into(), (*spec).into()],
    };
    Ok((meta, rest))
}
fn meta_part(m: &Meta) -> String {
    let mut fields = meta_fields(&m.id, &m.label, m.night, &m.path, &m.flags);
    fields.push(clean(&m.authors.join("\n")));
    fields.join(&FIELD.to_string())
}
fn read_meta(s: &str) -> Result<Meta, &'static str> {
    let (error, fields) = (
        "settings saved theme editor meta",
        s.split(FIELD).collect::<Vec<_>>(),
    );
    let (mut meta, [authors]) = read_meta_fields(&fields, error)? else {
        return Err(error);
    };
    meta.authors = authors.lines().map(Into::into).collect();
    Ok(meta)
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
    let mut fields = meta_fields(&f.id, &f.label, f.night, &f.path, &f.flags);
    fields.extend([clean(&f.authors), f.checked.to_string()]);
    fields.join(&FIELD.to_string())
}
fn read_meta_form(s: &str) -> Result<Option<MetaForm>, &'static str> {
    if s.is_empty() {
        return Ok(None);
    }
    let (error, fields) = (
        "settings saved theme editor meta form",
        s.split(FIELD).collect::<Vec<_>>(),
    );
    let (m, [authors, checked]) = read_meta_fields(&fields, error)? else {
        return Err(error);
    };
    Ok(Some(MetaForm {
        authors: (*authors).into(),
        checked: flag(checked)?,
        ..MetaForm::of(&m)
    }))
}

fn prop_part(f: &PropertyForm) -> String {
    let mut fields = vec![
        rule_text(&Some(f.rule.clone())),
        f.set.to_string(),
        f.original.as_deref().map_or(String::new(), clean),
        clean(&f.name),
        f.encoder.map_or(String::new(), |e| e.id().to_string()),
        clean(&f.choice),
    ];
    fields.extend(f.texts.iter().map(|t| clean(t)));
    fields.extend([f.last.to_string(), f.error.into(), f.checked.to_string()]);
    fields.join(&FIELD.to_string())
}
fn read_prop(s: &str) -> Result<Option<PropertyForm>, &'static str> {
    const ERROR: &str = "settings saved theme editor property form";
    if s.is_empty() {
        return Ok(None);
    }
    let fields: Vec<&str> = s.split(FIELD).collect();
    let [
        rule,
        set,
        original,
        name,
        encoder,
        choice,
        t0,
        t1,
        t2,
        t3,
        last,
        error,
        checked,
    ] = fields[..]
    else {
        return Err(ERROR);
    };
    let encoder = match encoder {
        "" => None,
        id => Some(id.parse().ok().and_then(Encoder::from_id).ok_or(ERROR)?),
    };
    let error = [property::EXISTS, ""].into_iter().find(|k| *k == error);
    let (Some(rule), Ok(set), Ok(last), Some(error)) =
        (read_rule(rule)?, set.parse(), last.parse::<usize>(), error)
    else {
        return Err(ERROR);
    };
    Ok(Some(PropertyForm {
        rule,
        set,
        original: (!original.is_empty()).then(|| original.into()),
        name: name.into(),
        encoder,
        choice: choice.into(),
        texts: [t0, t1, t2, t3].map(Into::into),
        last: last.min(3),
        error,
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
            e.prop.as_ref().map_or(String::new(), prop_part),
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
            prop,
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
            prop: read_prop(prop)?,
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
                Some(PROPERTY_ADD | PROPERTY_EDIT) => d.prop.is_some(),
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
        e.prop = d.prop;
        e.keyboard_was = d.keyboard_was;
        e.load = Load::Ready;
        self.theme_preview();
    }
}
