//! The language screens, drawn by Rust at Compose's routes: Languages & Layouts (`list.rs`),
//! Select language (`picker.rs`), the language pack manager (`packs.rs`) and the subtype editor
//! (`editor.rs`, painted by `form.rs`), with their dialogs (`dialog.rs`). Kotlin reports the keyboard's language
//! resources (`wire::settings_ui::languages`); Rust edits the subtype list itself, through
//! `settings::subtype::plan`, and writes `localization__subtypes` as the keyboard stored it.
pub mod dialog;
pub mod editor;
pub mod form;
pub mod list;
pub mod packs;
pub mod parts;
pub mod picker;
pub mod saved;
use super::{App, items::Source};
use settings::{Value, codec};
use wire::settings_ui::languages::{effect, kind, row};

pub const LIST_ROUTE: &str = "settings/localization";
pub const PICKER_ROUTE: &str = "settings/localization/select-locale";
pub const PACKS_ROUTE: &str = "settings/localization/language-pack-manage";
pub const ADD_ROUTE: &str = "settings/localization/subtype/add";
/// The editor of an existing subtype: its id follows as one more path segment.
pub const EDIT_ROUTE: &str = "settings/localization/subtype/edit";
pub const SUBTYPES_KEY: &str = "localization__subtypes";
pub const NAMES_KEY: &str = "localization__display_language_names_in";
const ACTIVE_KEY: &str = "localization__active_subtype_id";
/// The routes whose screens need the report.
const ROUTES: [&str; 5] = [LIST_ROUTE, PICKER_ROUTE, PACKS_ROUTE, ADD_ROUTE, EDIT_ROUTE];
/// The strings these screens show beyond their rows'.
pub const KEYS: [&str; 9] = [
    "settings__localization__subtype_summary",
    "settings__localization__subtype_error_already_exists",
    "settings__localization__subtype_error_fields_no_value",
    "settings__localization__subtype_select_placeholder",
    "settings__localization__suggested_languages",
    "settings__localization__all_languages",
    "action__delete_confirm_message",
    "action__delete",
    "action__ok",
];

/// A locale's names as Java gives them, and its tags.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Names {
    /// The language tag, as `FlorisLocale.languageTag()` writes it.
    pub tag: String,
    /// The name in the app's locale (`displayName()`).
    pub label: String,
    /// The name in the locale itself (`displayName(locale)`).
    pub native: String,
    pub english: String,
    /// The locale tag (`localeTag()`), which the search matches too.
    pub locale_tag: String,
    /// Java's `Locale.toString()` and language code: what a preset is matched by.
    pub base: String,
    pub language: String,
}

/// A component the keyboard offers: its component name and label.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Component {
    pub id: String,
    pub label: String,
}

/// A language pack extension and its items' labels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pack {
    pub id: String,
    pub title: String,
    pub deletable: bool,
    pub items: Vec<String>,
}

/// A subtype preset: its locale's tags and the subtype it makes.
#[derive(Clone, Debug, PartialEq)]
pub struct Preset {
    pub tag: String,
    pub base: String,
    pub language: String,
    pub value: Value,
}

/// What Kotlin reported, in its order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Resources {
    /// The raw `localization__subtypes` Kotlin read, and the list it gave back normalised.
    pub basis: String,
    pub subtypes: String,
    pub locales: Vec<Names>,
    pub system: Vec<Names>,
    pub names: Vec<Names>,
    /// Each layout with its layout type's id (`characters`, `symbols`, ...).
    pub layouts: Vec<(String, Component)>,
    pub composers: Vec<Component>,
    pub currencies: Vec<Component>,
    pub popups: Vec<Component>,
    pub presets: Vec<Preset>,
    pub packs: Vec<Pack>,
}

/// The language screens' own state.
#[derive(Debug, Default)]
pub struct Languages {
    pub resources: Option<Resources>,
    /// A report was asked for and has not come yet.
    pub requested: bool,
    /// Select language's search text.
    pub search: String,
}

/// A tap on a language screen's own controls.
#[derive(Clone, Debug, PartialEq)]
pub enum Tap {
    /// Select language's locale with this language tag.
    Locale(String),
    /// A suggested preset, by its index in the report's presets.
    Preset(usize),
    /// Add subtype's Show all.
    ShowAll,
    /// Import a language pack, in Compose's extension import screen.
    Import,
    /// Open the extension's page, in Compose.
    View(String),
    /// Ask to delete the extension.
    DeletePack(String),
    /// The editor's Delete action.
    DeleteSubtype,
}

/// The wall clock in milliseconds since the epoch.
pub fn wall_clock() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}

/// Whether `route` is one of the language screens, the editor's with or without its id.
pub fn language_route(route: &str) -> bool {
    ROUTES.contains(&route) || edit_id(route).is_some()
}

/// Whether `route` is the subtype editor's, adding or editing.
pub fn editor_route(route: &str) -> bool {
    route == ADD_ROUTE || route == EDIT_ROUTE || edit_id(route).is_some()
}

/// The subtype id an editor route names.
pub fn edit_id(route: &str) -> Option<i64> {
    route
        .strip_prefix(EDIT_ROUTE)?
        .strip_prefix('/')?
        .parse()
        .ok()
}

/// A report's rows as [`Resources`].
pub fn parse(p: &[String]) -> Result<Resources, &'static str> {
    if !p.len().is_multiple_of(row::SIZE) {
        return Err("settings languages");
    }
    let mut r = Resources::default();
    for f in p.chunks(row::SIZE) {
        let names = || Names {
            tag: f[row::ID].clone(),
            label: f[row::LABEL].clone(),
            native: f[row::NATIVE].clone(),
            english: f[row::ENGLISH].clone(),
            locale_tag: f[row::DETAIL].clone(),
            base: f[row::BASE].clone(),
            language: f[row::LANGUAGE].clone(),
        };
        let component = || Component {
            id: f[row::ID].clone(),
            label: f[row::LABEL].clone(),
        };
        match f[row::KIND].as_str() {
            kind::SUBTYPES => {
                r.basis = f[row::ID].clone();
                r.subtypes = f[row::DETAIL].clone();
            }
            kind::LOCALE => r.locales.push(names()),
            kind::SYSTEM => r.system.push(names()),
            kind::NAME => r.names.push(names()),
            kind::LAYOUT => r.layouts.push((f[row::DETAIL].clone(), component())),
            kind::COMPOSER => r.composers.push(component()),
            kind::CURRENCY => r.currencies.push(component()),
            kind::POPUP => r.popups.push(component()),
            kind::PRESET => r.presets.push(Preset {
                tag: f[row::ID].clone(),
                base: f[row::BASE].clone(),
                language: f[row::LANGUAGE].clone(),
                value: codec::decode("subtype", &settings::text(&f[row::DETAIL]), false)
                    .map_err(|_| "settings languages preset")?,
            }),
            kind::PACK => r.packs.push(Pack {
                id: f[row::ID].clone(),
                title: f[row::LABEL].clone(),
                deletable: f[row::DETAIL] == "true",
                items: vec![],
            }),
            kind::PACK_ITEM => {
                let pack = r.packs.iter_mut().find(|p| p.id == f[row::ID]);
                pack.ok_or("settings languages item")?
                    .items
                    .push(f[row::LABEL].clone());
            }
            _ => return Err("settings languages kind"),
        }
    }
    Ok(r)
}

/// A string field of `value` at `path`, or empty.
pub fn field_text(value: &Value, path: &[&str]) -> String {
    let mut v = value;
    for key in path {
        match v.get(key) {
            Ok(next) => v = next,
            Err(_) => return String::new(),
        }
    }
    v.string_value()
        .map(String::from_utf16_lossy)
        .unwrap_or_default()
}

impl App {
    /// A `LANGUAGES` report: the resources the screens show.
    pub fn set_languages(&mut self, p: &[String]) -> Result<(), &'static str> {
        self.languages.resources = Some(parse(p)?);
        self.languages.requested = false;
        Ok(())
    }
    /// Asks Kotlin for the report unless one is on its way.
    pub(super) fn request_languages(&mut self) {
        if !self.languages.requested {
            self.languages.requested = true;
            self.effects.push(vec![effect::LOAD.into()]);
        }
    }
    /// A native screen opened at `route`.
    pub(super) fn languages_opened(&mut self, route: &str) {
        if route != LIST_ROUTE {
            self.forget_deleted(Source::Subtypes);
        }
        if !editor_route(route) && route != PICKER_ROUTE {
            self.draft = None;
        }
        if !language_route(route) {
            return;
        }
        // The list and the pack manager show what the keyboard has now; the others reuse it.
        if matches!(route, LIST_ROUTE | PACKS_ROUTE) || self.languages.resources.is_none() {
            self.request_languages();
        }
        if route == PICKER_ROUTE {
            self.languages.search.clear();
        }
        if editor_route(route) {
            self.open_draft(route);
        }
    }
    /// Back went to the screen now on top.
    pub(super) fn languages_left(&mut self) {
        let top = self.stack.last().cloned().unwrap_or_default();
        if top != LIST_ROUTE {
            self.forget_deleted(Source::Subtypes);
        }
        if !editor_route(&top) {
            self.draft = None;
        }
    }
    /// After a saved state came back: the report again, and a draft if the editor lost it.
    pub fn languages_restored(&mut self) {
        let top = self.stack.last().cloned().unwrap_or_default();
        if language_route(&top) {
            self.request_languages();
        }
        if editor_route(&top) && self.draft.is_none() {
            self.open_draft(&top);
        }
    }
    /// The subtypes as SubtypeManager holds them: the stored list, normalised by Kotlin when its
    /// report read the same preference, with each unsupported subtype replaced (its `load`).
    pub fn subtypes(&self) -> Vec<Value> {
        let raw = self.value(SUBTYPES_KEY);
        let normalised = self.languages.resources.as_ref().filter(|r| r.basis == raw);
        let text = normalised.map_or(raw.as_str(), |r| r.subtypes.as_str());
        let list = codec::decode("subtypes", &settings::text(text), false);
        list.ok()
            .and_then(|v| v.list().ok().map(<[Value]>::to_vec))
            .unwrap_or_default()
            .iter()
            .map(|v| settings::subtype::supported(v).unwrap_or_else(|_| v.clone()))
            .collect()
    }
    /// Runs SubtypeManager's plan `op` on the subtypes; the new list when it changed them.
    pub(super) fn subtype_plan(
        &self,
        op: &str,
        item: Value,
        order: &[usize],
    ) -> Option<Vec<Value>> {
        let active = self.value(ACTIVE_KEY).parse::<i64>().unwrap_or(-1);
        let request = settings::value::object([
            ("op", Value::string(op)),
            ("list", Value::List(self.subtypes())),
            ("active", Value::Number(active)),
            ("item", item),
            ("time", Value::Number((self.now)())),
            (
                "order",
                Value::List(order.iter().map(|&i| Value::Number(i as i64)).collect()),
            ),
        ]);
        let plan = settings::subtype::plan(&request).ok()?;
        if plan.get("changed").and_then(Value::number) != Ok(1) {
            return None;
        }
        plan.get("list").ok()?.list().ok().map(<[Value]>::to_vec)
    }
    /// Writes `list` as SubtypeManager's `persistNewSubtypeList` does.
    pub(super) fn write_subtypes(&mut self, list: Vec<Value>) {
        if let Ok(encoded) = codec::encode("subtypes", &Value::List(list)) {
            self.write(SUBTYPES_KEY, String::from_utf16_lossy(&encoded));
        }
    }
    /// The names of `tag`: a subtype's or preset's, else the picker's, else the tag alone.
    pub fn names(&self, tag: &str) -> Names {
        let r = self.languages.resources.as_ref();
        r.and_then(|r| {
            r.names
                .iter()
                .chain(&r.locales)
                .chain(&r.system)
                .find(|n| n.tag == tag)
        })
        .cloned()
        .unwrap_or_else(|| Names {
            tag: tag.into(),
            label: tag.into(),
            native: tag.into(),
            english: tag.into(),
            locale_tag: tag.replace('-', "_"),
            base: tag.replace('-', "_"),
            language: tag.split('-').next().unwrap_or_default().into(),
        })
    }
    /// `tag`'s name as Display language names in asks: in the app's locale, or in its own.
    pub fn locale_name(&self, tag: &str) -> String {
        self.shown_name(&self.names(tag))
    }
    pub(super) fn shown_name(&self, n: &Names) -> String {
        if self.value(NAMES_KEY) == "NATIVE_LOCALE" {
            n.native.clone()
        } else {
            n.label.clone()
        }
    }
    /// A layout's label, by its layout type's id and component name.
    pub fn layout_label(&self, layout_type: &str, id: &str) -> Option<String> {
        let r = self.languages.resources.as_ref()?;
        r.layouts
            .iter()
            .find(|(t, c)| t == layout_type && c.id == id)
            .map(|(_, c)| c.label.clone())
    }
    /// A tap on a language screen's own control.
    pub(super) fn lang_tap(&mut self, tap: Tap) {
        match tap {
            Tap::Locale(tag) => self.locale_picked(&tag),
            Tap::Preset(i) => self.apply_preset(i),
            Tap::ShowAll => {
                self.open_lang_dialog(super::catalogue::languages::Lang::AllPresets, "")
            }
            Tap::Import => self.navigate("ext/import/EXT_LANGUAGEPACK"),
            Tap::View(id) => self.navigate(&format!("ext/view/{id}")),
            Tap::DeletePack(id) => {
                self.open_lang_dialog(super::catalogue::languages::Lang::DeletePack, &id)
            }
            Tap::DeleteSubtype => self.delete_draft(),
        }
    }
}
