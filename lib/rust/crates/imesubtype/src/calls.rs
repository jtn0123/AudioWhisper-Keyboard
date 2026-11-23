//! The subtype session's calls as they cross from Android: `wire::ime_subtype`'s rows in, rows out.

use settings::Value;
use wire::ime_subtype::{evaluate, load, loaded, select, subtype, switched};

use crate::Text;
use crate::session::{Load, Session, Switch};

/// Why a call was refused: a row cut short, a number or stored list that doesn't parse, or an
/// unknown operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error(pub &'static str);

impl From<&'static str> for Error {
    fn from(message: &'static str) -> Self {
        Self(message)
    }
}

type Result<T> = std::result::Result<T, Error>;
type Rows = Vec<Text>;

/// Each layout a subtype names, by its `settings::codec::LAYOUTS` key, and its field in a
/// [`subtype`] row.
const LAYOUTS: [(&str, usize); 8] = [
    ("characters", subtype::CHARACTERS),
    ("symbols", subtype::SYMBOLS),
    ("symbols2", subtype::SYMBOLS2),
    ("numeric", subtype::NUMERIC),
    ("numericAdvanced", subtype::NUMERIC_ADVANCED),
    ("numericRow", subtype::NUMERIC_ROW),
    ("phone", subtype::PHONE),
    ("phone2", subtype::PHONE2),
];

/// `operation`'s answer to `fields`, as `wire::ime_subtype::OPS` describes each.
pub fn call(session: &mut Session, operation: &[u16], fields: &[Text]) -> Result<Rows> {
    match utf8(operation)?.as_str() {
        wire::ime_subtype::LOAD => load_list(session, field(fields, load::RAW)?),
        wire::ime_subtype::LOCALES => {
            session.name(fields)?;
            Ok(vec![])
        }
        wire::ime_subtype::EVALUATE => {
            let stored = number(field(fields, evaluate::STORED)?)?;
            let switch = session.evaluate(stored)?;
            switched_rows(session, Some(switch))
        }
        wire::ime_subtype::NEXT => {
            let switch = session.cycle(false)?;
            switched_rows(session, Some(switch))
        }
        wire::ime_subtype::PREVIOUS => {
            let switch = session.cycle(true)?;
            switched_rows(session, Some(switch))
        }
        wire::ime_subtype::SELECT => {
            let switch = session.select(number(field(fields, select::ID)?)?)?;
            switched_rows(session, switch)
        }
        wire::ime_subtype::SETS => {
            session.report(fields.to_vec());
            Ok(vec![])
        }
        wire::ime_subtype::CURRENCY => {
            let index = session.currency(field(fields, 0)?);
            Ok(vec![number_text(index.map_or(-1, |i| i as i64))])
        }
        _ => Err(Error("not an operation")),
    }
}

fn load_list(session: &mut Session, raw: &[u16]) -> Result<Rows> {
    let mut head = vec![Text::new(); loaded::SIZE];
    match session.load(raw)? {
        Load::Ask(tags) => {
            head[loaded::ASKS] = number_text(tags.len());
            head.extend(tags);
            Ok(head)
        }
        Load::Loaded { changed } => {
            head[loaded::ASKS] = number_text(0);
            head[loaded::CHANGED] = flag(changed);
            if changed {
                let list = Value::List(session.list().to_vec());
                head[loaded::RAW] = settings::codec::encode("subtypes", &list)?;
            }
            head[loaded::COUNT] = number_text(session.list().len());
            for value in session.list().to_vec() {
                head.extend(row(session, &value)?);
            }
            Ok(head)
        }
    }
}

fn switched_rows(session: &mut Session, switch: Option<Switch>) -> Result<Rows> {
    let mut out = vec![Text::new(); switched::SIZE];
    out[switched::WRITE] = flag(switch.is_some_and(|s| s.write));
    out[switched::ACTIVE] = flag(switch.is_some());
    if let Some(switch) = switch {
        out[switched::ID] = number_text(switch.id);
        let active = session.active().clone();
        out.extend(row(session, &active)?);
    }
    Ok(out)
}

/// A [`subtype`] row for `value`, then its secondary locale tags.
fn row(session: &mut Session, value: &Value) -> Result<Rows> {
    let mut out = vec![Text::new(); subtype::SIZE];
    let text = |name: &str| -> Result<Text> { Ok(value.get(name)?.string_value()?.to_vec()) };
    out[subtype::ID] = number_text(value.get("id")?.number()?);
    out[subtype::KEY] = number_text(session.key(value));
    out[subtype::LOCALE] = text("primaryLocale")?;
    let providers = value.get("nlpProviders")?;
    out[subtype::SPELLING] = providers.get("spelling")?.string_value()?.to_vec();
    out[subtype::SUGGESTION] = providers.get("suggestion")?.string_value()?.to_vec();
    out[subtype::COMPOSER] = text("composer")?;
    out[subtype::CURRENCY] = text("currencySet")?;
    out[subtype::PUNCTUATION] = text("punctuationRule")?;
    out[subtype::POPUP] = text("popupMapping")?;
    let layouts = value.get("layoutMap")?;
    for (name, at) in LAYOUTS {
        out[at] = layouts.get(name)?.string_value()?.to_vec();
    }
    let secondary = value.get("secondaryLocales")?.list()?;
    out[subtype::SECONDARY] = number_text(secondary.len());
    for tag in secondary {
        out.push(tag.string_value()?.to_vec());
    }
    Ok(out)
}

fn field(fields: &[Text], at: usize) -> Result<&Text> {
    fields.get(at).ok_or(Error("a row is cut short"))
}

fn utf8(units: &[u16]) -> Result<String> {
    String::from_utf16(units).map_err(|_| Error("not UTF-16"))
}

fn number(units: &[u16]) -> Result<i64> {
    utf8(units)?.parse().map_err(|_| Error("not a number"))
}

fn flag(value: bool) -> Text {
    settings::text(wire::flag::encode(value))
}

fn number_text(value: impl ToString) -> Text {
    settings::text(&value.to_string())
}
