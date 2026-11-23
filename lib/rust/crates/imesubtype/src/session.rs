//! The subtype session: the list and the active subtype, kept as `SubtypeManager` kept them, and
//! what each subtype names.
//!
//! Main's manager passed every subtype through Kotlin's `Subtype`, whose locales are
//! `FlorisLocale`s: a stored tag became `FlorisLocale.fromTag(tag)` and was written back as its
//! `languageTag()`. That naming is `java.util.Locale`'s, so the session asks Android for it once
//! per tag and keeps every subtype with its locales as Android named them. Two subtypes are the
//! same subtype when those values are equal, as two `Subtype`s were.

use std::collections::HashMap;

use settings::{Result, Value, codec, subtype};

use crate::Text;

/// The subtypes and the active one.
#[derive(Debug)]
pub struct Session {
    list: Vec<Value>,
    active: Value,
    /// Each locale tag Android has named, and its name.
    names: HashMap<Text, Text>,
    /// The tags the last load asked Android to name.
    asked: Vec<Text>,
    /// The names of the currency sets Android last reported loading.
    sets: Vec<Text>,
    /// Every distinct subtype seen, by its cache key.
    keys: Vec<Value>,
}

/// What a load needs or did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Load {
    /// Android must name these tags first, then load again.
    Ask(Vec<Text>),
    /// The list is loaded; when `changed`, some subtype was repaired and the list is written back.
    Loaded { changed: bool },
}

/// What a switch did: the active subtype's id, and whether Android stores it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Switch {
    pub write: bool,
    pub id: i64,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    /// No subtypes, and the default subtype active, as a new `SubtypeManager` started.
    pub fn new() -> Self {
        Self {
            list: vec![],
            active: codec::default_subtype(),
            names: HashMap::new(),
            asked: vec![],
            sets: vec![],
            keys: vec![],
        }
    }

    /// The list.
    pub fn list(&self) -> &[Value] {
        &self.list
    }

    /// The active subtype.
    pub fn active(&self) -> &Value {
        &self.active
    }

    /// Loads the stored list `raw`: decodes it (blank is empty), and repairs each subtype the
    /// keyboard can no longer use, as main's `load` plan did. The active subtype is chosen by
    /// [`Session::evaluate`] after it, as main launched it after the list was published.
    pub fn load(&mut self, raw: &[u16]) -> Result<Load> {
        let decoded = subtype::decode_list(raw)?;
        let list = decoded.list()?;
        let mut asked: Vec<Text> = vec![];
        for value in list.iter().chain([&codec::default_subtype()]) {
            for tag in tags(value)? {
                if !self.names.contains_key(tag) && !asked.iter().any(|t| t == tag) {
                    asked.push(tag.to_vec());
                }
            }
        }
        if !asked.is_empty() {
            self.asked.clone_from(&asked);
            return Ok(Load::Ask(asked));
        }
        let mut changed = false;
        let mut repaired = Vec::with_capacity(list.len());
        for value in list {
            let named = self.named(value)?;
            let new = self.named(&subtype::supported(&named)?)?;
            changed |= named != new;
            repaired.push(new);
        }
        self.list = repaired;
        Ok(Load::Loaded { changed })
    }

    /// Android's names for the tags the last load asked about, in order.
    pub fn name(&mut self, names: &[Text]) -> Result<()> {
        if names.len() != self.asked.len() {
            return Err("an answer for each tag asked");
        }
        for (tag, name) in std::mem::take(&mut self.asked).into_iter().zip(names) {
            self.names.insert(tag, name.clone());
        }
        Ok(())
    }

    /// The active subtype once the list changed: the one whose id is `stored`, else the first,
    /// else the default; Android stores its id if that is not `stored`.
    pub fn evaluate(&mut self, stored: i64) -> Result<Switch> {
        let ids = self.ids()?;
        let (index, _) = subtype::active(&ids, stored, -1);
        self.active = self.at(index)?;
        let id = self.active.get("id")?.number()?;
        Ok(Switch {
            write: id != stored,
            id,
        })
    }

    /// The language key: the subtype after the active one (before it when `reverse`), wrapping,
    /// or the default when the active subtype is not in the list. Android always stores its id.
    pub fn cycle(&mut self, reverse: bool) -> Result<Switch> {
        let flags: Vec<i64> = self
            .list
            .iter()
            .map(|v| i64::from(*v == self.active))
            .collect();
        self.active = self.at(subtype::cycle(&flags, reverse))?;
        Ok(Switch {
            write: true,
            id: self.active.get("id")?.number()?,
        })
    }

    /// The subtype with `id`, made active, or nothing when the list has none.
    pub fn select(&mut self, id: i64) -> Result<Option<Switch>> {
        let Some(index) = self.ids()?.iter().position(|&i| i == id) else {
            return Ok(None);
        };
        self.active = self.list[index].clone();
        Ok(Some(Switch { write: true, id }))
    }

    /// The key graph cache key of `value`: the same number for equal subtypes, and a new one for
    /// each subtype not seen before.
    pub fn key(&mut self, value: &Value) -> usize {
        if let Some(key) = self.keys.iter().position(|v| v == value) {
            return key;
        }
        self.keys.push(value.clone());
        self.keys.len() - 1
    }

    /// Android loaded the currency sets named `sets`.
    pub fn report(&mut self, sets: Vec<Text>) {
        self.sets = sets;
    }

    /// The reported currency set a subtype's currency set name names, by its index, if any.
    pub fn currency(&self, name: &[u16]) -> Option<usize> {
        self.sets.iter().position(|s| s == name)
    }

    fn ids(&self) -> Result<Vec<i64>> {
        self.list.iter().map(|v| v.get("id")?.number()).collect()
    }

    /// The subtype at `index`, or the default subtype for none.
    fn at(&self, index: i64) -> Result<Value> {
        match usize::try_from(index).ok().and_then(|i| self.list.get(i)) {
            Some(value) => Ok(value.clone()),
            None => self.named(&codec::default_subtype()),
        }
    }

    /// `value` with its locale tags as Android named them; a tag never named stays as it is.
    fn named(&self, value: &Value) -> Result<Value> {
        let name = |tag: &Value| -> Result<Value> {
            let tag = tag.string_value()?;
            Ok(Value::String(
                self.names.get(tag).cloned().unwrap_or_else(|| tag.to_vec()),
            ))
        };
        let mut out = value.clone();
        out.set("primaryLocale", name(value.get("primaryLocale")?)?)?;
        let secondary = value.get("secondaryLocales")?.list()?;
        out.set(
            "secondaryLocales",
            Value::List(secondary.iter().map(name).collect::<Result<_>>()?),
        )?;
        Ok(out)
    }
}

/// A subtype's locale tags: the primary, then the secondary ones.
fn tags(value: &Value) -> Result<Vec<&[u16]>> {
    let mut out = vec![value.get("primaryLocale")?.string_value()?];
    for tag in value.get("secondaryLocales")?.list()? {
        out.push(tag.string_value()?);
    }
    Ok(out)
}
