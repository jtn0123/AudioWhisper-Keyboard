//! The subtype editor's state and what its controls do: SubtypeEditorScreen and
//! SubtypeEditorState. Its rows are painted by `form.rs`. A draft starts blank when adding, as
//! the subtype with the route's id when editing, and saves through SubtypeManager's `add` or
//! `modify` plan, with the same checks and messages.
use super::{ADD_ROUTE, PICKER_ROUTE, edit_id, field_text};
use crate::settings_ui::{
    App, Modal,
    catalogue::languages::{Field, Lang},
};
use settings::{Value, codec};

/// SubtypeEditorState's "‒ select ‒": a locale and a component no one chose yet.
pub const LOCALE_UNSET: &str = "00-00";
pub const UNSET: &str = "00:00";
/// The one suggestion engine SubtypeEditorScreen offers, as it named it.
pub const PROVIDERS: [(&str, &str); 1] = [("org.florisboard.nlp.providers.latin", "Latin")];
/// `extCorePopupMapping("default")`: a picked locale's popup mapping when no preset has one.
pub const DEFAULT_POPUP: &str = "org.florisboard.localization:default";

/// The subtype being edited.
#[derive(Clone, Debug, PartialEq)]
pub struct Draft {
    pub value: Value,
    /// Adding a subtype, or editing the one with `id`.
    pub adding: bool,
    pub id: i64,
    /// A save found a field without a value: those fields show as errors from now on.
    pub checked: bool,
}

/// SubtypeEditorState with no subtype: every field unset but the suggestion engine and the
/// punctuation rule, which keep `Subtype.DEFAULT`'s.
pub fn blank() -> Value {
    let mut v = codec::default_subtype();
    let _ = v.set("primaryLocale", Value::string(LOCALE_UNSET));
    for key in ["composer", "currencySet", "popupMapping"] {
        let _ = v.set(key, Value::string(UNSET));
    }
    let layouts = codec::LAYOUTS
        .iter()
        .map(|(k, _)| (k.to_string(), Value::string(UNSET)))
        .collect();
    let _ = v.set("layoutMap", Value::Object(layouts));
    v
}

/// Where `f` sits in a subtype; the suggestion engine is read from `suggestion`.
fn path(f: Field) -> [&'static str; 2] {
    match f {
        Field::Locale => ["primaryLocale", ""],
        Field::Popup => ["popupMapping", ""],
        Field::Layout(k) => ["layoutMap", codec::LAYOUTS[k].0],
        Field::Provider => ["nlpProviders", "suggestion"],
        Field::Composer => ["composer", ""],
        Field::Currency => ["currencySet", ""],
    }
}

/// Sets the string at `path` (one or two keys deep) in `v`.
fn put(v: &mut Value, path: [&str; 2], value: &str) {
    if path[1].is_empty() {
        let _ = v.set(path[0], Value::string(value));
    } else if let Ok(inner) = v.get(path[0]) {
        let mut inner = inner.clone();
        let _ = inner.set(path[1], Value::string(value));
        let _ = v.set(path[0], inner);
    }
}

impl Draft {
    pub fn get(&self, f: Field) -> String {
        let [a, b] = path(f);
        if b.is_empty() {
            field_text(&self.value, &[a])
        } else {
            field_text(&self.value, &[a, b])
        }
    }
    /// Sets `f`; the suggestion engine sets spelling and suggestion together.
    pub fn set(&mut self, f: Field, value: &str) {
        put(&mut self.value, path(f), value);
        if f == Field::Provider {
            put(&mut self.value, ["nlpProviders", "spelling"], value);
        }
    }
    /// Whether `f` is still "‒ select ‒".
    pub fn unset(&self, f: Field) -> bool {
        self.get(f)
            == if f == Field::Locale {
                LOCALE_UNSET
            } else {
                UNSET
            }
    }
    /// SubtypeEditorState.toSubtype's check, `SettingsRules.editorValid` over its 15 fields.
    pub fn valid(&self) -> bool {
        let v = &self.value;
        let mut fields = vec![field_text(v, &["primaryLocale"])];
        fields.push(field_text(v, &["nlpProviders", "spelling"]));
        fields.push(field_text(v, &["nlpProviders", "suggestion"]));
        for key in ["composer", "currencySet", "punctuationRule", "popupMapping"] {
            fields.push(field_text(v, &[key]));
        }
        for (key, _) in codec::LAYOUTS {
            fields.push(field_text(v, &["layoutMap", key]));
        }
        let fields: Vec<_> = fields.iter().map(|f| settings::text(f)).collect();
        settings::call(wire::settings::EDITOR_VALID, &fields)
            .is_ok_and(|out| out.first().is_some_and(|f| wire::flag::decode(f)))
    }
}

impl App {
    /// A fresh draft for the editor at `route`.
    pub(super) fn open_draft(&mut self, route: &str) {
        let id = edit_id(route);
        let stored = id.and_then(|id| {
            self.subtypes()
                .into_iter()
                .find(|s| s.get("id").and_then(Value::number) == Ok(id))
        });
        self.draft = Some(Draft {
            value: stored.unwrap_or_else(blank),
            adding: route == ADD_ROUTE,
            id: id.unwrap_or(-1),
            checked: false,
        });
        // SubtypeEditorScreen opens the presets dialog when adding.
        if route == ADD_ROUTE {
            self.open_lang_dialog(Lang::AllPresets, "");
        }
    }
    /// What a `Pick` field offers: component names and labels, in the keyboard's order.
    pub fn pick_options(&self, f: Field) -> Vec<(String, String)> {
        let Some(r) = self.languages.resources.as_ref() else {
            return vec![];
        };
        let pairs = |list: &[super::Component]| {
            list.iter()
                .map(|c| (c.id.clone(), c.label.clone()))
                .collect::<Vec<_>>()
        };
        match f {
            Field::Locale => vec![],
            Field::Popup => pairs(&r.popups),
            Field::Composer => pairs(&r.composers),
            Field::Currency => pairs(&r.currencies),
            Field::Provider => PROVIDERS
                .iter()
                .map(|(id, label)| (id.to_string(), label.to_string()))
                .collect(),
            Field::Layout(k) => r
                .layouts
                .iter()
                .filter(|(t, _)| t == codec::LAYOUTS[k].0)
                .map(|(_, c)| (c.id.clone(), c.label.clone()))
                .collect(),
        }
    }
    /// The field's value as it shows: "‒ select ‒" when unset, the locale's or component's
    /// name, or the component name the keyboard does not know.
    pub fn pick_text(&self, f: Field) -> String {
        let Some(d) = &self.draft else {
            return String::new();
        };
        let value = d.get(f);
        if d.unset(f) {
            return self.label("settings__localization__subtype_select_placeholder");
        }
        if f == Field::Locale {
            return self.locale_name(&value);
        }
        self.pick_options(f)
            .into_iter()
            .find(|(id, _)| *id == value)
            .map_or(value, |(_, label)| label)
    }
    /// A tap on field row `i`: the language opens Select language, the others a list.
    pub(in crate::settings_ui) fn open_pick(&mut self, i: usize, f: Field) {
        let Some(d) = &self.draft else { return };
        if f == Field::Locale {
            return self.navigate(PICKER_ROUTE);
        }
        let selected = d.get(f);
        self.open_modal(i, selected);
    }
    /// Opens row `row`'s dialog with `selected`.
    pub(super) fn open_modal(&mut self, row: usize, selected: String) {
        if self.editor.focused {
            self.editor.focused = false;
            self.editor.composing = None;
            self.effects.push(vec!["hideEditor".into()]);
        }
        self.scrollbar_start[1] = -1;
        self.scrollbar_last[1] = -1;
        self.modal = Some(Modal {
            row,
            selected,
            values: vec![],
            scroll: 0.0,
            enabled: true,
        });
        self.focus = -1;
    }
    /// Select language answered with `tag`: the locale, and the popup mapping of its preset as
    /// SubtypeEditorScreen set them.
    pub(super) fn locale_picked(&mut self, tag: &str) {
        let names = self.names(tag);
        let preset = self.languages.resources.as_ref().and_then(|r| {
            let pairs: Vec<_> = r
                .presets
                .iter()
                .flat_map(|p| [settings::text(&p.base), settings::text(&p.language)])
                .collect();
            let at = settings::subtype::preset(
                &settings::text(&names.base),
                &settings::text(&names.language),
                &pairs,
            );
            usize::try_from(at)
                .ok()
                .and_then(|i| r.presets.get(i))
                .map(|p| p.value.clone())
        });
        if let Some(d) = &mut self.draft {
            d.set(Field::Locale, tag);
            let popup = preset.as_ref().map(|p| field_text(p, &["popupMapping"]));
            d.set(Field::Popup, popup.as_deref().unwrap_or(DEFAULT_POPUP));
        }
        self.back();
    }
    /// A preset chosen from the card or the dialog: SubtypeEditorState.applySubtype.
    pub(super) fn apply_preset(&mut self, i: usize) {
        let preset = self
            .languages
            .resources
            .as_ref()
            .and_then(|r| r.presets.get(i));
        if let (Some(p), Some(d)) = (preset.map(|p| p.value.clone()), &mut self.draft) {
            d.value = p;
        }
    }
    /// The bottom bar's Cancel and Save.
    pub(in crate::settings_ui) fn lang_row(&mut self, l: Lang) {
        match l {
            Lang::Cancel => self.back(),
            Lang::Save => self.save_draft(),
            _ => {}
        }
    }
    /// Save: an unset field marks the fields and explains; adding a subtype that exists
    /// already explains that; otherwise the list is written and the editor closes.
    fn save_draft(&mut self) {
        let Some(d) = self.draft.clone() else { return };
        if !d.valid() {
            if let Some(d) = &mut self.draft {
                d.checked = true;
            }
            return self.open_lang_dialog(
                Lang::Message,
                "settings__localization__subtype_error_fields_no_value",
            );
        }
        let op = if d.adding { "add" } else { "modify" };
        match self.subtype_plan(op, d.value, &[]) {
            Some(list) => self.write_subtypes(list),
            None if d.adding => {
                return self.open_lang_dialog(
                    Lang::Message,
                    "settings__localization__subtype_error_already_exists",
                );
            }
            None => {}
        }
        self.back();
    }
    /// The editor's Delete: removes the stored subtype, keeping it to undo, and closes.
    pub(super) fn delete_draft(&mut self) {
        let Some(id) = self.draft.as_ref().filter(|d| !d.adding).map(|d| d.id) else {
            return;
        };
        let list = self.subtypes();
        let Some(index) = list
            .iter()
            .position(|s| s.get("id").and_then(Value::number) == Ok(id))
        else {
            return;
        };
        self.remove_subtype(index, list[index].clone());
        self.back();
    }
}

/// Every field a preset can fill, in the editor's order.
pub fn fields() -> Vec<Field> {
    let mut out = vec![
        Field::Popup,
        Field::Provider,
        Field::Composer,
        Field::Currency,
    ];
    out.extend((0..codec::LAYOUTS.len()).map(Field::Layout));
    out
}
