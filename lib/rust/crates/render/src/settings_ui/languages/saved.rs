//! The language screens' state in the activity's saved state: the editor's draft, Select
//! language's search and the delete to undo, so a recreated activity keeps the user's edits.
//! The report is not kept: it is asked for again.
use super::editor::Draft;
use crate::settings_ui::{
    App,
    items::{Deleted, Source},
};
use serde_json::{Value as Json, json};
use settings::codec;

const WRONG: &str = "settings saved languages";

impl App {
    /// The draft, search and undo, as JSON.
    pub fn languages_saved(&self) -> String {
        let draft = self.draft.as_ref().map(|d| {
            json!([
                String::from_utf16_lossy(&d.value.json()),
                d.adding,
                d.id,
                d.checked
            ])
        });
        let deleted = self
            .list
            .deleted
            .as_ref()
            .map(|g| json!([g.index, g.value, g.title]));
        json!({
            "draft": draft,
            "search": self.languages.search,
            "deleted": deleted,
            "slots": self.list.slots,
        })
        .to_string()
    }
    /// Takes back what [`App::languages_saved`] wrote; anything else fails and changes nothing.
    pub fn restore_languages(&mut self, saved: &str) -> Result<(), &'static str> {
        let v: Json = serde_json::from_str(saved).map_err(|_| WRONG)?;
        let text = |v: &Json| v.as_str().map(String::from).ok_or(WRONG);
        let flag = |v: &Json| v.as_bool().ok_or(WRONG);
        let subtype =
            |s: &str| codec::decode("subtype", &settings::text(s), false).map_err(|_| WRONG);
        let draft = match &v["draft"] {
            Json::Null => None,
            Json::Array(a) if a.len() == 4 => Some(Draft {
                value: subtype(&text(&a[0])?)?,
                adding: flag(&a[1])?,
                id: a[2].as_i64().ok_or(WRONG)?,
                checked: flag(&a[3])?,
            }),
            _ => return Err(WRONG),
        };
        let deleted = match &v["deleted"] {
            Json::Null => None,
            Json::Array(a) if a.len() == 3 => {
                let value = text(&a[1])?;
                subtype(&value)?;
                Some(Deleted {
                    source: Source::Subtypes,
                    index: a[0]
                        .as_u64()
                        .and_then(|n| usize::try_from(n).ok())
                        .ok_or(WRONG)?,
                    value,
                    title: text(&a[2])?,
                })
            }
            _ => return Err(WRONG),
        };
        let slots = v["slots"]
            .as_array()
            .ok_or(WRONG)?
            .iter()
            .map(text)
            .collect::<Result<Vec<_>, _>>()?;
        let search = text(&v["search"])?;
        self.draft = draft;
        self.languages.search = search;
        self.list.deleted = deleted;
        self.list.slots = slots;
        Ok(())
    }
}
