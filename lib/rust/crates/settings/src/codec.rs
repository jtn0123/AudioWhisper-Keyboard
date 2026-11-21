//! kotlinx-compatible stored codecs. Unknown values use the existing streaming lexer's
//! bracket-only skip. Known values are decoded by their schema; field order is declaration order.
use crate::{
    Result, Text, text,
    value::{Value, object},
};
use gif::json::Lexer;

type Fields = Vec<(String, Text)>;
fn fields(raw: &[u16]) -> Result<Fields> {
    let mut lex = Lexer::new(raw);
    let mut values = Vec::new();
    lex.object(|lex, key| {
        let key = String::from_utf16_lossy(key);
        let value = lex.raw_value()?;
        values.push((key, value));
        Ok(true)
    })
    .map_err(|_| "invalid object")?;
    lex.expect_eof().map_err(|_| "trailing JSON")?;
    Ok(values)
}
// Streaming class decoding validates every occurrence; a bad earlier field cannot
// be hidden by a later valid value. Valid repetitions retain their last value.
fn field(
    values: &Fields,
    name: &str,
    default: Option<Value>,
    mut decode: impl FnMut(&[u16]) -> Result<Value>,
) -> Result<Value> {
    let mut result = default;
    for (_, raw) in values.iter().filter(|(key, _)| key == name) {
        result = Some(decode(raw)?);
    }
    result.ok_or("missing required field")
}
fn member(
    values: &Fields,
    name: &str,
    kind: Kind,
    default: Option<Value>,
    nullable: bool,
    compact: bool,
    depth: usize,
) -> Result<Value> {
    field(values, name, default, |raw| {
        if nullable && raw == text("null") {
            Ok(Value::Null)
        } else {
            read(kind, raw, compact, depth)
        }
    })
}
fn list(input: &[u16], mut decode: impl FnMut(&[u16]) -> Result<Value>) -> Result<Value> {
    let mut lexer = Lexer::new(input);
    let mut values = Vec::new();
    let mut failure = None;
    lexer
        .list(|lex| {
            let raw = lex.raw_value()?;
            match decode(&raw) {
                Ok(value) => values.push(value),
                Err(e) => {
                    failure = Some(e);
                    return Err(gif::json::Unreadable);
                }
            }
            Ok(())
        })
        .map_err(|_| failure.unwrap_or("invalid array"))?;
    lexer.expect_eof().map_err(|_| "trailing JSON")?;
    Ok(Value::List(values))
}
fn string(input: &[u16]) -> Result<Value> {
    let mut lex = Lexer::new(input);
    let value = lex.string().map_err(|_| "expected string")?;
    lex.expect_eof().map_err(|_| "trailing string")?;
    Ok(Value::String(value))
}
fn number(input: &[u16]) -> Result<Value> {
    Ok(Value::Number(crate::numeric::decode(input)?))
}

fn component(input: &[u16]) -> Result<Value> {
    let value = string(input)?;
    let name = extensions::call(
        wire::extensions::COMPONENT_NAME,
        &[value.string_value()?.to_vec()],
    )
    .map_err(|_| "invalid component")?;
    if !name[wire::extensions::component::ERROR].is_empty() {
        return Err("invalid component");
    }
    Ok(value)
}
#[derive(Clone, Copy)]
enum Kind {
    String,
    Number,
    Int,
    Component,
    Strings,
    Providers,
    Layouts,
    Subtype,
    Subtypes,
    History,
    Action,
    Actions,
    Key,
    AbstractKey,
    Keys,
    Popup,
    Ints,
}
fn read(kind: Kind, input: &[u16], compact: bool, depth: usize) -> Result<Value> {
    if depth > 128 {
        return Err("JSON nesting limit");
    }
    match kind {
        Kind::String => string(input),
        Kind::Number => number(input),
        Kind::Int => {
            let n = number(input)?.number()?;
            i32::try_from(n).map_err(|_| "integer overflow")?;
            Ok(Value::Number(n))
        }
        Kind::Component => component(input),
        Kind::Strings | Kind::Subtypes | Kind::Actions | Kind::Keys | Kind::Ints => {
            let item = match kind {
                Kind::Subtypes => Kind::Subtype,
                Kind::Actions => Kind::Action,
                Kind::Keys => Kind::AbstractKey,
                Kind::Ints => Kind::Int,
                _ => Kind::String,
            };
            list(input, |v| read(item, v, compact, depth + 1))
        }
        _ => record(kind, input, compact, depth + 1),
    }
}
// Every omitted default and required-field check lives here, never in the managed DTO mapping.
fn record(kind: Kind, input: &[u16], compact: bool, depth: usize) -> Result<Value> {
    let values = fields(input)?;
    let mut result = Vec::new();
    let mut add =
        |name: &'static str, kind: Kind, default: Option<Value>, nullable: bool| -> Result<()> {
            let value = member(
                &values,
                name,
                kind,
                default.clone(),
                nullable,
                compact,
                depth,
            )?;
            if !compact || default.as_ref() != Some(&value) {
                result.push((name.to_owned(), value));
            }
            Ok(())
        };
    let s = |v: &str| Some(Value::string(v));
    let null = Some(Value::Null);
    let empty = || Some(Value::List(vec![]));
    match kind {
        Kind::Providers => {
            add(
                "spelling",
                Kind::String,
                s("org.florisboard.nlp.providers.latin"),
                false,
            )?;
            add(
                "suggestion",
                Kind::String,
                s("org.florisboard.nlp.providers.latin"),
                false,
            )?;
        }
        Kind::Layouts => {
            for (name, value) in LAYOUTS {
                add(
                    name,
                    Kind::Component,
                    s(&format!("org.florisboard.layouts:{value}")),
                    false,
                )?;
            }
        }
        Kind::Subtype => {
            add("id", Kind::Number, None, false)?;
            add("primaryLocale", Kind::String, None, false)?;
            add("secondaryLocales", Kind::Strings, None, false)?;
            add("nlpProviders", Kind::Providers, Some(providers()), false)?;
            add("composer", Kind::Component, None, false)?;
            add("currencySet", Kind::Component, None, false)?;
            add(
                "punctuationRule",
                Kind::Component,
                s("org.florisboard.localization:default"),
                false,
            )?;
            add("popupMapping", Kind::Component, None, false)?;
            add("layoutMap", Kind::Layouts, None, false)?;
        }
        Kind::History => {
            if values.iter().any(|(k, _)| k != "pinned" && k != "recent") {
                return Err("unknown history field");
            }
            add("pinned", Kind::Strings, None, false)?;
            add("recent", Kind::Strings, None, false)?;
        }
        Kind::Action => {
            let tag = field(&values, "$", Some(Value::string("insert_key")), string)?;
            let tag = if tag.string_value()? == text("insert_text") {
                "insert_text"
            } else {
                "insert_key"
            };
            result.push(("$".to_owned(), Value::string(tag)));
            result.push((
                "data".to_owned(),
                member(
                    &values,
                    "data",
                    if tag == "insert_text" {
                        Kind::String
                    } else {
                        Kind::Key
                    },
                    None,
                    false,
                    compact,
                    depth,
                )?,
            ));
        }

        Kind::Popup => {
            add("main", Kind::AbstractKey, null, true)?;
            add("relevant", Kind::Keys, empty(), false)?;
        }
        Kind::Key | Kind::AbstractKey => {
            return key(&values, matches!(kind, Kind::AbstractKey), compact, depth);
        }
        _ => return Err("invalid record kind"),
    }
    Ok(Value::Object(result))
}
fn key(values: &Fields, abstract_key: bool, compact: bool, depth: usize) -> Result<Value> {
    let tag = field(values, "$", Some(Value::string("text_key")), string)?;
    let tag = String::from_utf16_lossy(tag.string_value()?);
    let selectors = [
        "case_selector",
        "shift_state_selector",
        "variation_selector",
        "layout_direction_selector",
        "char_width_selector",
    ];
    let tag = if ["auto_text_key", "multi_text_key"].contains(&tag.as_str())
        || (abstract_key && selectors.contains(&tag.as_str()))
    {
        tag.as_str()
    } else {
        "text_key"
    };
    let mut out = vec![("$".to_owned(), Value::string(tag))];
    if selectors.contains(&tag) {
        let (names, nullable, required): (&[&str], bool, bool) = match tag {
            "case_selector" => (&["lower", "upper"], false, true),
            "shift_state_selector" => (
                &[
                    "unshifted",
                    "shifted",
                    "shiftedManual",
                    "shiftedAutomatic",
                    "capsLock",
                ],
                true,
                false,
            ),
            "variation_selector" => (
                &["default", "email", "uri", "normal", "password"],
                true,
                false,
            ),
            "layout_direction_selector" => (&["ltr", "rtl"], false, true),
            _ => (&["full", "half"], true, true),
        };
        for &name in names {
            let value = member(
                values,
                name,
                Kind::AbstractKey,
                if required { None } else { Some(Value::Null) },
                nullable,
                compact,
                depth + 1,
            )?;
            if !compact || required || value != Value::Null {
                out.push((name.to_owned(), value));
            }
        }
    } else {
        let key_type = field(values, "type", Some(Value::string("character")), |raw| {
            let upper = chars::case::uppercase(string(raw)?.string_value()?);
            let names = [
                "character",
                "enter_editing",
                "function",
                "lock",
                "modifier",
                "navigation",
                "system_gui",
                "numeric",
                "placeholder",
                "unspecified",
            ];
            let index = names
                .iter()
                .position(|name| chars::case::uppercase(&text(name)) == upper)
                .ok_or("unknown key type")?;
            Ok(Value::string(names[index]))
        })?;
        let fields = if tag == "multi_text_key" {
            vec![
                ("type", Kind::String, Value::string("character")),
                ("codePoints", Kind::Ints, Value::List(vec![])),
            ]
        } else {
            vec![
                ("type", Kind::String, Value::string("character")),
                ("code", Kind::Int, Value::Number(0)),
            ]
        };
        for (name, kind, default) in fields.into_iter().chain([
            ("label", Kind::String, Value::string("")),
            ("groupId", Kind::Int, Value::Number(0)),
            ("popup", Kind::Popup, Value::Null),
        ]) {
            let value = if name == "type" {
                key_type.clone()
            } else {
                member(
                    values,
                    name,
                    kind,
                    Some(default.clone()),
                    name == "popup",
                    compact,
                    depth + 1,
                )?
            };
            if !compact || value != default {
                out.push((name.to_owned(), value));
            }
        }
    }
    Ok(Value::Object(out))
}
pub const LAYOUTS: &[(&str, &str)] = &[
    ("characters", "qwerty"),
    ("symbols", "western"),
    ("symbols2", "western"),
    ("numeric", "western_arabic"),
    ("numericAdvanced", "western_arabic"),
    ("numericRow", "western_arabic"),
    ("phone", "telpad"),
    ("phone2", "telpad"),
];
pub fn providers() -> Value {
    object([
        (
            "spelling",
            Value::string("org.florisboard.nlp.providers.latin"),
        ),
        (
            "suggestion",
            Value::string("org.florisboard.nlp.providers.latin"),
        ),
    ])
}
pub fn default_subtype() -> Value {
    object([
        ("id", Value::Number(-1)),
        ("primaryLocale", Value::string("en-US")),
        ("secondaryLocales", Value::List(vec![])),
        ("nlpProviders", providers()),
        (
            "composer",
            Value::string("org.florisboard.composers:appender"),
        ),
        (
            "currencySet",
            Value::string("org.florisboard.currencysets:dollar"),
        ),
        (
            "punctuationRule",
            Value::string("org.florisboard.localization:default"),
        ),
        (
            "popupMapping",
            Value::string("org.florisboard.localization:en"),
        ),
        (
            "layoutMap",
            Value::Object(
                LAYOUTS
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.to_string(),
                            Value::string(&format!("org.florisboard.layouts:{v}")),
                        )
                    })
                    .collect(),
            ),
        ),
    ])
}
pub fn decode(kind: &str, input: &[u16], compact: bool) -> Result<Value> {
    match kind {
        "subtypes" => read(Kind::Subtypes, input, false, 0),
        "subtype" => read(Kind::Subtype, input, false, 0),
        "history" => read(Kind::History, input, false, 0),
        "arrangement" => {
            let values = fields(input)?;
            Ok(object([
                (
                    "stickyAction",
                    member(
                        &values,
                        "stickyAction",
                        Kind::Action,
                        None,
                        true,
                        compact,
                        0,
                    )?,
                ),
                (
                    "dynamicActions",
                    member(
                        &values,
                        "dynamicActions",
                        Kind::Actions,
                        None,
                        false,
                        compact,
                        0,
                    )?,
                ),
                (
                    "hiddenActions",
                    member(
                        &values,
                        "hiddenActions",
                        Kind::Actions,
                        None,
                        false,
                        compact,
                        0,
                    )?,
                ),
            ]))
        }

        _ => Err("unknown codec"),
    }
}
pub fn encode(kind: &str, value: &Value) -> Result<Text> {
    match kind {
        "subtypes" | "subtype" | "history" => Ok(value.json()),
        "arrangement" => Ok(decode(kind, &value.json(), true)?.json()),
        _ => Err("unknown codec"),
    }
}
