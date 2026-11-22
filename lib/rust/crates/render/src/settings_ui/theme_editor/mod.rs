//! The theme editor, drawn by Rust where the extension editor's Manage component opened
//! ThemeEditorScreen: the component box (`paint.rs`), the rules in `compare_order` with their
//! properties as text (`rules.rs`), the rule editor (`form.rs`), the component meta editor
//! (`meta.rs`) and fine-tune, a screen of three preferences. Kotlin reads the extension editor's
//! workspace (`wire::settings_ui::theme_editor`), writes the stylesheet Rust serializes back into
//! it on Save and shows the draft on the keyboard; Rust parses, edits and serializes the
//! stylesheet. Edits are a draft until Save: Back with changes asks to discard them, where
//! Compose wrote every edit through to the workspace.
pub mod boxes;
pub mod form;
pub mod form_paint;
pub mod meta;
pub mod names;
pub mod paint;
pub mod property;
pub mod property_paint;
pub mod property_values;
pub mod rules;
pub mod saved;
pub mod sets;
pub mod ui;
use super::{
    App,
    catalogue::{Kind, languages::Lang, theme_editor::Part},
};
use snygg::{
    Text,
    grammar::Rule,
    json::parse,
    json_write::to_pretty_json,
    resolve::{PropertySets, Stylesheet},
    value::Value,
};
use wire::settings_ui::theme_editor::{effect, outcome, report};

/// A component's editor: `/` and the workspace's uuid, then `/` and the component's index in the
/// extension's themes follow.
pub const EDIT: &str = "settings/theme/edit";
pub const RULE_ADD: &str = "settings/theme/editor/rule/add";
pub const RULE_EDIT: &str = "settings/theme/editor/rule";
pub const PROPERTY_ADD: &str = "settings/theme/editor/property/add";
pub const PROPERTY_EDIT: &str = "settings/theme/editor/property";
pub const META: &str = "settings/theme/editor/meta";
pub const TUNE: &str = "settings/theme/editor/tune";
/// The strings the editor shows beyond its rows' and the element, property, selector and value
/// names.
pub const KEYS: [&str; 28] = [
    "settings__theme_editor__stylesheet_error_title",
    "settings__theme_editor__font_name_invalid",
    "settings__theme_editor__stylesheet_error_description",
    "settings__theme_editor__rule_already_exists",
    "settings__theme_editor__code_invalid",
    "settings__theme_editor__no_codes_defined",
    "settings__theme_editor__duplicate_rule",
    "settings__theme_editor__missing_workspace",
    "settings__theme_editor__loading",
    "settings__theme_editor__add_rule",
    "snygg__rule_annotation__defines",
    "snygg__rule_annotation__defines_description",
    "snygg__rule_annotation__font",
    "action__yes",
    "action__no",
    "action__save",
    "action__edit",
    "action__delete",
    "action__discard",
    "general__select_dropdown_value_placeholder",
    "ext__validation__enter_component_label",
    "ext__validation__hint_component_label_to_long",
    "ext__validation__error_author",
    "ext__validation__enter_component_id",
    "ext__validation__error_component_id",
    "ext__validation__error_stylesheet_path",
    "ext__validation__error_stylesheet_path_blank",
    meta::ID_TAKEN,
];

/// Every string key the editor shows beyond its rows': [`KEYS`], the elements', properties',
/// selectors' and attribute values' names.
pub fn string_keys() -> Vec<&'static str> {
    let mut out = KEYS.to_vec();
    out.extend(names::ELEMENT_KEYS);
    out.extend(names::PROPERTIES.iter().map(|p| p.1));
    out.extend(rules::SELECTOR_KEYS);
    out.extend(form::MODES.iter().chain(&form::SHIFT_STATES).map(|v| v.1));
    out.extend(property::string_keys());
    out
}

/// The workspace and component index a component editor's route names.
pub fn target(route: &str) -> Option<(&str, usize)> {
    let rest = route.strip_prefix(EDIT)?.strip_prefix('/')?;
    let (workspace, index) = rest.split_once('/')?;
    if workspace.is_empty() || index.contains('/') {
        return None;
    }
    Some((workspace, index.parse().ok()?))
}
/// The catalogue route of a component editor's route, which carries its target.
pub fn screen_route(route: &str) -> Option<&'static str> {
    target(route).map(|_| EDIT)
}

/// A tap on the theme editor's own controls.
#[derive(Clone, Debug, PartialEq)]
pub enum Tap {
    /// The stylesheet error's Yes (load it leniently) and No (start empty).
    Lenient(bool),
    /// The component box's Edit, Fine-tune and Add rule.
    Meta,
    Tune,
    Add,
    /// Rule `r`'s Edit and Duplicate.
    Edit(usize),
    Duplicate(usize),
    /// Property `k` of set `s` of rule `r` deleted.
    DeleteProperty(usize, usize, usize),
    Undo,
    /// The top bar's Save, and in the rule editor its Delete.
    Save,
    AskDelete,
    /// Row `i`'s list: the elements, or the values an attribute lacks.
    Open(usize),
    /// Selector `s` toggled; value `v` of attribute `k` removed.
    Selector(usize),
    Remove(usize, String),
    /// The meta editor's night switch.
    Night,
    /// The property editor's and the source sets' controls.
    Prop(property::PropTap),
}

/// What the component box shows beyond the meta: `extension:id` and the stylesheet's path.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Component {
    pub extension: String,
    pub fonts: Vec<String>,
    pub images: Vec<String>,
    /// The names the Material You palette style, contrast level and spec version can take.
    pub choices: [Vec<String>; 3],
    /// The extension's other components' ids.
    pub other_ids: Vec<String>,
}
/// The meta the meta editor changes, ComponentMetaEditorDialog's fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Meta {
    pub id: String,
    pub label: String,
    pub authors: Vec<String>,
    pub night: bool,
    /// The stylesheet path as the component keeps it: empty for the one its id gives.
    pub path: String,
    /// The Material You palette style, contrast level and spec version, by name.
    pub flags: [String; 3],
}

impl Meta {
    /// Where the stylesheet is kept, `stylesheetPath()`.
    pub fn stylesheet_path(&self) -> String {
        let (id, path) = (snygg::text(&self.id), snygg::text(&self.path));
        String::from_utf16_lossy(&extensions::stylesheet_path(&id, &path))
    }
}
/// Where loading the component stands.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Load {
    /// Asked Kotlin, and waiting.
    #[default]
    Waiting,
    /// The workspace or the component is gone.
    Missing,
    /// The stylesheet did not load strictly: the error, and the question whether to fix it.
    Failed(String),
    Ready,
}
pub type Rules = Vec<(Rule, PropertySets<Value>)>;
/// What Undo puts back.
#[derive(Clone, Debug, PartialEq)]
pub enum Gone {
    Rule(Rule, PropertySets<Value>),
    /// A property: its rule, its set (0 for a single set), its place, its name and value.
    Property(Rule, usize, usize, Text, Value),
    /// A source set: its rule, its place and its properties.
    Set(Rule, usize, Vec<(Text, Value)>),
}

/// The theme editor's own state.
#[derive(Debug, Default)]
pub struct ThemeEditor {
    /// The workspace and component index being edited.
    pub target: Option<(String, usize)>,
    pub requested: bool,
    pub load: Load,
    /// The report's stylesheet, for the error page's choices.
    pub raw: String,
    pub schema: Text,
    pub rules: Rules,
    pub meta: Meta,
    /// The rules and meta as loaded or last saved: anything else is unsaved.
    pub base: Option<(Rules, Meta)>,
    pub component: Component,
    pub deleted: Option<Gone>,
    /// Why Save failed, shown over the rules.
    pub error: Option<String>,
    /// Whether the keyboard showed when an editor or fine-tune opened over the rules.
    pub keyboard_was: bool,
    pub form: Option<form::RuleForm>,
    pub meta_form: Option<meta::MetaForm>,
    pub prop: Option<property::PropertyForm>,
    /// A draft a saved state brought back: it replaces the report's stylesheet and meta.
    pub restored: Option<saved::Draft>,
}

impl ThemeEditor {
    /// Whether the draft differs from what was loaded or last saved.
    pub fn changed(&self) -> bool {
        self.base
            .as_ref()
            .is_some_and(|(rules, meta)| *rules != self.rules || *meta != self.meta)
    }
    /// The draft as Compose's `PrettyPrintConfig` writes it.
    pub fn json(&self) -> Result<String, String> {
        let sheet = Stylesheet {
            rules: self.rules.clone(),
        };
        to_pretty_json(&self.schema, &sheet).map_err(|e| e.0)
    }
}

impl App {
    /// Whether a component editor's route, or one of its screens, is on screen.
    pub fn in_theme_editor(&self) -> bool {
        self.stack.iter().any(|r| target(r).is_some())
    }
    /// Asks Kotlin for the component the editor route names, unless that report is coming.
    fn request_component(&mut self) {
        let Some((workspace, index)) = self.theme_editor.target.clone() else {
            return;
        };
        if !self.theme_editor.requested {
            self.theme_editor.requested = true;
            self.effects
                .push(vec![effect::LOAD.into(), workspace, index.to_string()]);
        }
    }
    /// A native screen opened at `route`: a component editor starts from Kotlin's report.
    pub(super) fn theme_editor_opened(&mut self, route: &str) {
        if let Some((workspace, index)) = target(route) {
            self.theme_editor = ThemeEditor {
                target: Some((workspace.into(), index)),
                ..ThemeEditor::default()
            };
            self.request_component();
        } else if route == EDIT {
            // A component editor's route names its component; without one there is nothing.
            self.theme_editor = ThemeEditor {
                load: Load::Missing,
                ..ThemeEditor::default()
            };
        }
    }
    /// A `THEME_EDITOR` report: the component, or that it is gone.
    pub fn set_theme_editor(&mut self, p: &[String]) -> Result<(), &'static str> {
        if p.len() != report::SIZE {
            return Err("settings theme editor");
        }
        let e = &mut self.theme_editor;
        let wanted = e.target.as_ref().map(|(w, i)| (w.clone(), i.to_string()));
        if wanted != Some((p[report::WORKSPACE].clone(), p[report::COMPONENT].clone())) {
            return Ok(());
        }
        e.requested = false;
        if p[report::OUTCOME] != outcome::LOADED {
            e.load = Load::Missing;
            return Ok(());
        }
        let list = |s: &str| -> Vec<String> {
            s.split('/')
                .filter(|n| !n.is_empty())
                .map(Into::into)
                .collect()
        };
        let [palettes, contrasts, specs] = [report::PALETTES, report::CONTRASTS, report::SPECS];
        e.component = Component {
            extension: p[report::EXTENSION].clone(),
            fonts: list(&p[report::FONTS]),
            images: list(&p[report::IMAGES]),
            choices: [list(&p[palettes]), list(&p[contrasts]), list(&p[specs])],
            other_ids: list(&p[report::OTHER_IDS]),
        };
        let [palette, contrast, spec] = [report::PALETTE, report::CONTRAST, report::SPEC];
        e.meta = Meta {
            id: p[report::ID].clone(),
            label: p[report::LABEL].clone(),
            authors: p[report::AUTHORS].lines().map(Into::into).collect(),
            night: p[report::NIGHT] == wire::flag::TRUE,
            path: p[report::PATH].clone(),
            flags: [p[palette].clone(), p[contrast].clone(), p[spec].clone()],
        };
        e.raw = p[report::STYLESHEET].clone();
        match e.restored.take() {
            Some(draft) => self.resume_draft(draft),
            None => self.load_stylesheet(None),
        }
        Ok(())
    }
    /// Loads the report's stylesheet as `loadThemeEditorStylesheet` did: none gives an empty
    /// one; `None` loads strictly and asks on an error; `Some(true)` loads leniently and
    /// `Some(false)` starts empty, the error page's Yes and No.
    pub(super) fn load_stylesheet(&mut self, lenient: Option<bool>) {
        let e = &mut self.theme_editor;
        let loaded = match lenient {
            _ if e.raw.is_empty() => None,
            Some(false) => None,
            Some(true) => parse(&e.raw, true).ok(),
            None => match parse(&e.raw, false) {
                Ok(sheet) => Some(sheet),
                Err(error) => {
                    e.load = Load::Failed(error.0);
                    return;
                }
            },
        };
        let (schema, sheet) = loaded.unwrap_or_else(|| {
            (
                snygg::text(snygg::document::SCHEMA_V2),
                Stylesheet { rules: vec![] },
            )
        });
        e.schema = schema;
        e.rules = rules::ordered(snygg::editor::Editor::new(sheet).sheet.rules);
        if !e.rules.iter().any(|(r, _)| *r == Rule::Defines) {
            e.rules
                .insert(0, (Rule::Defines, PropertySets::Single(vec![])));
        }
        e.base = Some((e.rules.clone(), e.meta.clone()));
        e.load = Load::Ready;
        self.theme_preview();
    }
    /// Shows the draft on the keyboard, as ThemeEditorScreen set `previewThemeInfo` on every
    /// change. A draft that cannot be written is not shown.
    pub(super) fn theme_preview(&mut self) {
        let (Some((workspace, index)), Ok(json)) =
            (self.theme_editor.target.clone(), self.theme_editor.json())
        else {
            return;
        };
        self.effects.push(vec![
            effect::PREVIEW.into(),
            workspace,
            index.to_string(),
            json,
        ]);
    }
    /// The draft changed: the keyboard shows it, and a failed save's message goes.
    pub(super) fn theme_changed(&mut self) {
        self.theme_editor.error = None;
        self.theme_preview();
    }
    /// Save: the stylesheet Rust writes and the meta go back to the workspace, and the editor
    /// closes. A stylesheet that cannot be written stays open with the reason.
    pub(super) fn save_theme(&mut self) {
        let e = &mut self.theme_editor;
        let Some((workspace, index)) = e.target.clone() else {
            return;
        };
        let json = match e.json() {
            Ok(json) => json,
            Err(error) => {
                e.error = Some(error);
                return;
            }
        };
        let m = &e.meta;
        let night = wire::flag::encode(m.night).to_owned();
        let meta = [
            m.label.clone(),
            m.authors.join("\n"),
            night,
            m.id.clone(),
            m.path.clone(),
        ];
        let save = [effect::SAVE.into(), workspace, index.to_string(), json];
        let packet = [&save[..], &meta, &m.flags].concat();
        e.base = Some((e.rules.clone(), e.meta.clone()));
        self.effects.push(packet);
        self.close_editor();
    }
    /// Back on the rules with unsaved changes: the question instead of leaving.
    pub fn theme_holds_back(&self) -> bool {
        self.modal.is_none()
            && self.stack.last().is_some_and(|r| target(r).is_some())
            && self.theme_editor.changed()
    }
    /// Back, before it leaves the screen: true when it asked to discard instead.
    pub(super) fn theme_back(&mut self) -> bool {
        if !self.theme_holds_back() {
            return false;
        }
        self.open_lang_dialog(Lang::Discard, "");
        true
    }
    /// The discard question's Discard: the editor closes without saving.
    pub(super) fn discard_theme(&mut self) {
        let e = &mut self.theme_editor;
        e.rules = e.base.as_ref().map_or(vec![], |b| b.0.clone());
        e.meta = e.base.as_ref().map_or(Meta::default(), |b| b.1.clone());
        self.close_editor();
    }
    /// Back went to the screen now on top: on the rules again after an editor or fine-tune,
    /// the keyboard shows as `DisplayKbdAfterDialogs` says.
    pub(super) fn theme_editor_left(&mut self) {
        if self.in_theme_editor() {
            self.close_theme_forms();
            self.keyboard_after_dialog();
        } else if self.theme_editor.target.take().is_some() {
            self.effects.push(vec![effect::PREVIEW.into()]);
            self.theme_editor = ThemeEditor::default();
        }
    }
    /// An editor or fine-tune opens over the rules: whether the keyboard showed is kept.
    pub(super) fn theme_dialog_opened(&mut self) {
        self.theme_editor.keyboard_was = self.editor.focused && self.field.is_none();
    }
    /// `DisplayKbdAfterDialogs`: always show the keyboard again, never, or as it was.
    fn keyboard_after_dialog(&mut self) {
        let show = match self
            .value("theme__editor_display_kbd_after_dialogs")
            .as_str()
        {
            "ALWAYS" => true,
            "NEVER" => false,
            _ => self.theme_editor.keyboard_was,
        };
        if show && self.stack.last().is_some_and(|r| target(r).is_some()) {
            self.own_field(None);
            self.editor.focused = true;
            self.effects.push(vec!["editor".into()]);
        }
    }
    /// A tap on the theme editor's own control.
    pub(super) fn theme_tap(&mut self, tap: Tap) {
        match tap {
            Tap::Lenient(yes) => self.load_stylesheet(Some(yes)),
            Tap::Meta => self.open_meta(),
            Tap::Tune => {
                self.theme_dialog_opened();
                self.navigate(TUNE);
            }
            Tap::Add => self.add_rule(),
            Tap::Edit(r) => self.edit_rule(r),
            Tap::Duplicate(r) => self.duplicate_rule(r),
            Tap::DeleteProperty(r, s, k) => self.delete_property(r, s, k),
            Tap::Undo => self.undo_theme_delete(),
            Tap::Save => self.save_theme(),
            Tap::AskDelete => self.open_lang_dialog(Lang::DeleteRule, ""),
            Tap::Open(i) => self.open_theme_list(i),
            Tap::Night => {
                if let Some(f) = self.theme_editor.meta_form.as_mut() {
                    f.night = !f.night;
                }
            }
            Tap::Selector(s) => {
                if let Some(f) = self.theme_editor.form.as_mut() {
                    f.selector = if f.selector == s { 0 } else { s };
                }
            }
            Tap::Remove(k, v) => {
                if let Some(f) = self.theme_editor.form.as_mut() {
                    f.set_value(k, &v, false);
                }
            }
            Tap::Prop(tap) => self.property_tap(tap),
        }
    }
    /// Opens row `i`'s list with the element chosen, or nothing chosen for a value to add.
    fn open_theme_list(&mut self, i: usize) {
        if let Some(Kind::ThemeEdit(Part::MetaChoice(k))) = self.rows().get(i).map(|r| r.kind) {
            let chosen = self
                .theme_editor
                .meta_form
                .as_ref()
                .map(|f| f.flags[k].clone());
            return self.open_modal(i, chosen.unwrap_or_default());
        }
        if let Some(selected) = self.property_selected(i) {
            return self.open_modal(i, selected);
        }
        let Some(form) = self.theme_editor.form.as_ref() else {
            return;
        };
        let element = matches!(
            self.rows().get(i).map(|r| r.kind),
            Some(Kind::ThemeEdit(Part::Element))
        );
        if element && form.original.is_some() {
            return;
        }
        let selected = if element {
            form.element.clone()
        } else {
            String::new()
        };
        self.open_modal(i, selected);
    }
    /// The bottom bar's Add or Apply on the rule, the meta or the property editor.
    pub(in crate::settings_ui) fn save_theme_form(&mut self) {
        if self.theme_editor.form.is_some() {
            self.save_rule();
        } else if self.theme_editor.meta_form.is_some() {
            self.save_meta();
        } else {
            self.save_property();
        }
    }
    /// Row `i`'s place in the focus order.
    pub(in crate::settings_ui) fn theme_focus(&self, i: usize, part: Part) -> Vec<i32> {
        let ready = self.theme_editor.load == Load::Ready;
        match part {
            Part::Status if ready => vec![paint::SAVE_ID],
            Part::Status if matches!(self.theme_editor.load, Load::Failed(_)) => {
                vec![paint::YES_ID, paint::NO_ID]
            }
            Part::Meta if ready => vec![paint::META_ID, paint::META_ID + 1, paint::META_ID + 3],
            Part::Tune if ready => vec![paint::TUNE_ID],
            Part::Rules if ready => self.rules_focus(),
            Part::MetaField(_) | Part::MetaChoice(_) => vec![1000 + i as i32],
            Part::MaterialYou => vec![],
            Part::Night => vec![meta::NIGHT_ID],
            Part::Status | Part::Meta | Part::Tune | Part::Rules => vec![],
            _ => self.rule_focus(i, part),
        }
    }
    /// Line field `part`'s text, or None when it is not one now.
    pub(in crate::settings_ui) fn theme_line(&self, part: Part) -> Option<String> {
        let e = &self.theme_editor;
        match part {
            Part::Font => e.form.as_ref().map(|f| f.font.clone()),
            Part::Attribute(0) => e.form.as_ref().map(|f| f.codes.clone()),
            Part::MetaField(k) => e.meta_form.as_ref().map(|f| f.field(k).to_owned()),
            _ => self.property_line(part),
        }
    }
    /// Line field `part` changed.
    pub(in crate::settings_ui) fn theme_typed(&mut self, part: Part, value: String) {
        if let Part::MetaField(k) = part {
            return self.meta_typed(k, value);
        }
        let Some(f) = self.theme_editor.form.as_mut() else {
            return self.property_typed(part, value);
        };
        match part {
            Part::Font => f.font = value,
            Part::Attribute(0) => f.codes = value,
            _ => {}
        }
    }
}
