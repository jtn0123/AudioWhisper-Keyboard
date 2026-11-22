//! Languages & Layouts' subtype list, as the editable list (`items.rs`) shows it: each subtype
//! titled with its language and summarised by its layouts and currency set, as LocalizationScreen
//! did, and changed through SubtypeManager's plans.
use super::{EDIT_ROUTE, field_text};
use crate::settings_ui::{
    App,
    items::{Deleted, Item, Op, Source, Words},
};
use settings::{Value, codec};

pub const WORDS: Words = Words {
    empty: "settings__localization__subtype_no_subtypes_configured_warning",
    add: "settings__localization__subtype_add_title",
    reorder: true,
};

impl App {
    /// Each subtype's title and summary. A component the keyboard does not know shows its name,
    /// where LocalizationScreen showed "null".
    pub fn subtype_items(&self) -> Vec<Item> {
        let r = self.languages.resources.as_ref();
        let currency = |id: &str| {
            r.and_then(|r| r.currencies.iter().find(|c| c.id == id))
                .map_or(id.to_owned(), |c| c.label.clone())
        };
        self.subtypes()
            .iter()
            .map(|s| {
                let characters = field_text(s, &["layoutMap", "characters"]);
                let symbols = field_text(s, &["layoutMap", "symbols"]);
                let set = field_text(s, &["currencySet"]);
                let summary = self
                    .label("settings__localization__subtype_summary")
                    .replace(
                        "{characters_name}",
                        &self
                            .layout_label("characters", &characters)
                            .unwrap_or(characters),
                    )
                    .replace(
                        "{symbols_name}",
                        &self.layout_label("symbols", &symbols).unwrap_or(symbols),
                    )
                    .replace("{currency_set_name}", &currency(&set));
                Item {
                    key: s
                        .get("id")
                        .and_then(Value::number)
                        .unwrap_or(-1)
                        .to_string(),
                    title: self.locale_name(&field_text(s, &["primaryLocale"])),
                    summary,
                }
            })
            .collect()
    }
    /// Carries out a tap on the subtype list.
    pub(in crate::settings_ui) fn subtype_op(&mut self, op: Op) {
        let list = self.subtypes();
        match op {
            Op::Open(i) => {
                if let Some(id) = list.get(i).and_then(|s| s.get("id").ok()?.number().ok()) {
                    self.navigate(&format!("{EDIT_ROUTE}/{id}"));
                }
            }
            Op::Up(i) | Op::Down(i) => {
                let Some(order) = crate::settings_ui::items::moved(list.len(), i, op == Op::Up(i))
                else {
                    return;
                };
                if let Some(new) = self.subtype_plan("reorder", Value::Null, &order) {
                    self.list.deleted = None;
                    self.write_subtypes(new);
                }
            }
            Op::Delete(i) => {
                if let Some(item) = list.get(i).cloned() {
                    self.remove_subtype(i, item);
                }
            }
            Op::Add => self.navigate(super::ADD_ROUTE),
            Op::Undo => self.undo_subtype(),
        }
    }
    /// Removes `item`, at `index`, keeping it to undo.
    pub(super) fn remove_subtype(&mut self, index: usize, item: Value) {
        let title = self.locale_name(&field_text(&item, &["primaryLocale"]));
        let value = String::from_utf16_lossy(&item.json());
        if let Some(new) = self.subtype_plan("remove", item, &[]) {
            self.write_subtypes(new);
            self.list.deleted = Some(Deleted {
                source: Source::Subtypes,
                index,
                value,
                title,
            });
            self.focus = crate::settings_ui::items::UNDO_ID;
        }
    }
    /// Puts the deleted subtype back where it stood, unless one with its id came back since.
    fn undo_subtype(&mut self) {
        let Some(gone) = self.list.deleted.take_if(|d| d.source == Source::Subtypes) else {
            return;
        };
        let Ok(item) = codec::decode("subtype", &settings::text(&gone.value), false) else {
            return;
        };
        let mut list = self.subtypes();
        if list.iter().any(|s| s.get("id").ok() == item.get("id").ok()) {
            return;
        }
        list.insert(gone.index.min(list.len()), item);
        self.write_subtypes(list);
        self.focus = -1;
    }
}
