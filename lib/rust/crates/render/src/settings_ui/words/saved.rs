//! The user dictionary screens' state in the activity's saved state: the editor's draft, the
//! search, the delete to undo and an import or export on its way, so a recreated activity keeps
//! the user's edits and finishes the transfer. The report is not kept: it is asked for again.
use super::{Word, editor::Draft, encode, list::deleted_word, transfer::Transfer};
use crate::settings_ui::{
    App,
    items::{Deleted, Source},
};
use serde_json::{Value as Json, json};

const WRONG: &str = "settings saved words";

fn word_json(w: &Word) -> Json {
    Json::from(encode(w))
}

fn word_from(v: &Json) -> Result<Word, &'static str> {
    super::decode(&serde_json::from_value::<Vec<String>>(v.clone()).map_err(|_| WRONG)?)
        .ok_or(WRONG)
}

impl App {
    /// The draft, search, undo and transfer, as JSON.
    pub fn words_saved(&self) -> String {
        let draft = self
            .words
            .draft
            .as_ref()
            .map(|d| json!([d.fields, d.stored.as_ref().map(word_json), d.checked]));
        let deleted = self
            .list
            .deleted
            .as_ref()
            .filter(|g| g.source == Source::Words)
            .map(|g| json!([g.index, g.value, g.title]));
        let transfer = self
            .words
            .transfer
            .as_ref()
            .map(|t| json!([t.dictionary, t.import]));
        json!({
            "draft": draft,
            "search": self.words.search,
            "deleted": deleted,
            "transfer": transfer,
        })
        .to_string()
    }
    /// Takes back what [`App::words_saved`] wrote; anything else fails and changes nothing.
    pub fn restore_words(&mut self, saved: &str) -> Result<(), &'static str> {
        let v: Json = serde_json::from_str(saved).map_err(|_| WRONG)?;
        let text = |v: &Json| v.as_str().map(String::from).ok_or(WRONG);
        let flag = |v: &Json| v.as_bool().ok_or(WRONG);
        let draft = match &v["draft"] {
            Json::Null => None,
            Json::Array(a) if a.len() == 3 => Some(Draft {
                fields: serde_json::from_value(a[0].clone()).map_err(|_| WRONG)?,
                stored: match &a[1] {
                    Json::Null => None,
                    w => Some(word_from(w)?),
                },
                checked: flag(&a[2])?,
            }),
            _ => return Err(WRONG),
        };
        let deleted = match &v["deleted"] {
            Json::Null => None,
            Json::Array(a) if a.len() == 3 => {
                let value = text(&a[1])?;
                deleted_word(&value).ok_or(WRONG)?;
                Some(Deleted {
                    source: Source::Words,
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
        let transfer = match &v["transfer"] {
            Json::Null => None,
            Json::Array(a) if a.len() == 2 => Some(Transfer {
                dictionary: text(&a[0])?,
                import: flag(&a[1])?,
            }),
            _ => return Err(WRONG),
        };
        let search = text(&v["search"])?;
        self.words.draft = draft;
        self.words.search = search;
        self.words.transfer = transfer;
        if deleted.is_some() {
            self.list.deleted = deleted;
        }
        Ok(())
    }
}
