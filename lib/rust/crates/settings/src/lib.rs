//! Settings are data and whole edit/migration plans; no Android, JVM, callbacks or file I/O.
#![forbid(unsafe_code)]
pub mod catalogue;
pub mod codec;
pub mod migration;
pub mod numeric;
pub mod scalar;
pub mod subtype;
pub mod time;
pub mod value;
pub use value::Value;
use wire::settings::{self as ops, catalogue as row, key_hint_modes, key_hints, layouts};
pub type Text = Vec<u16>;
pub type Result<T> = std::result::Result<T, &'static str>;
pub fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
pub fn integer(v: &[u16]) -> Result<i64> {
    let s = String::from_utf16(v).map_err(|_| "invalid integer")?;
    s.parse().map_err(|_| "invalid integer")
}
pub fn call(op: i32, input: &[Text]) -> Result<Vec<Text>> {
    match (op, input) {
        (ops::CATALOGUE, []) => Ok(catalogue::all()
            .iter()
            .flat_map(|p| {
                let mut out = vec![vec![]; row::SIZE];
                out[row::KEY] = text(p.key);
                out[row::KIND] = text(p.kind);
                out[row::DEFAULT] = text(p.default);
                out[row::MIN] = text(p.min);
                out[row::MAX] = text(p.max);
                out[row::STEP] = text(p.step);
                out[row::CHOICES] = text(p.choices);
                out
            })
            .collect()),
        (ops::ENUM_INDEX, [key, value]) => {
            Ok(vec![text(&catalogue::enum_index(key, value)?.to_string())])
        }
        (ops::CLAMP, [key, value]) => Ok(vec![catalogue::clamp(key, value)?]),
        (ops::SCALAR, [key, raw]) => scalar::decode(key, raw),
        (ops::DECODE, [kind, json]) => {
            let empty_subtypes;
            let json = if kind == &text("subtypes")
                && json
                    .iter()
                    .all(|&u| chars::is_whitespace(u) || matches!(u, 0xa0 | 0x2007 | 0x202f))
            {
                empty_subtypes = text("[]");
                &empty_subtypes
            } else {
                json
            };
            let kind = String::from_utf16_lossy(kind);
            let decoded = codec::decode(&kind, json, false);
            let value = if kind == "history" {
                decoded.unwrap_or_else(|_| {
                    value::object([
                        ("pinned", Value::List(vec![])),
                        ("recent", Value::List(vec![])),
                    ])
                })
            } else {
                decoded?
            };
            let mut out = vec![];
            value.pack(&mut out);
            Ok(out)
        }
        (ops::ENCODE, [kind, rest @ ..]) => {
            let mut iter = rest.iter();
            let value = Value::unpack(&mut iter, 0)?;
            if iter.len() != 0 {
                return Err("trailing packet");
            }
            Ok(vec![codec::encode(
                &String::from_utf16_lossy(kind),
                &value,
            )?])
        }
        (ops::MIGRATE, [kind, key, value]) => migration::migrate(kind, key, value),
        (ops::DEFAULT_SUBTYPE, []) => {
            let mut out = vec![];
            codec::default_subtype().pack(&mut out);
            Ok(out)
        }
        (ops::SUBTYPE, rest) => {
            let mut iter = rest.iter();
            let request = Value::unpack(&mut iter, 0)?;
            if iter.len() != 0 {
                return Err("trailing packet");
            }
            let mut out = vec![];
            subtype::plan(&request)?.pack(&mut out);
            Ok(out)
        }
        (ops::PRESET, [locale, language, presets @ ..]) => Ok(vec![text(
            &subtype::preset(locale, language, presets).to_string(),
        )]),
        (ops::EDITOR_VALID, fields) => Ok(vec![wire::flag::utf16(
            fields.len() == 15
                && fields.first().is_some_and(|v| v != &text("00-00"))
                && fields[1..].iter().all(|v| v != &text("00:00")),
        )]),
        (ops::COLOR, [value]) => {
            let s = String::from_utf16(value).map_err(|_| "invalid color")?;
            let parsed = u64::from_str_radix(&s, 16).map_err(|_| "invalid color")?;
            if s.is_empty() || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err("invalid color");
            }
            Ok(vec![text(&format!("{parsed:016x}"))])
        }
        (ops::TIME_DECODE, [raw]) => Ok(time::decode(raw)),
        (ops::TIME_ENCODE, values) => Ok(vec![time::encode(values)?]),
        (ops::LAYOUT_MAP, values) if values.len() == layouts::SIZE => {
            use layouts::*;
            let order = [
                CHARACTERS,
                SYMBOLS,
                SYMBOLS2,
                NUMERIC_ROW,
                NUMERIC,
                NUMERIC_ADVANCED,
                PHONE,
                PHONE2,
            ];
            let mut out = vec![];
            for (n, index) in order.iter().enumerate() {
                if n > 0 {
                    out.push(44);
                }
                out.extend(text(codec::LAYOUTS[*index].0));
                out.push(61);
                out.extend(&values[*index]);
            }
            Ok(vec![out])
        }
        (ops::COMPONENT_ENCODE, [extension, component]) => {
            let mut out = extension.clone();
            out.push(58);
            out.extend(component);
            Ok(vec![out])
        }
        (ops::COMPONENT_DECODE, [raw]) => {
            use wire::extensions::{COMPONENT_NAME, component as name};
            use wire::settings::component as out;
            let parts = extensions::call(COMPONENT_NAME, std::slice::from_ref(raw))
                .map_err(|_| "invalid component packet")?;
            Ok(if parts[name::ERROR].is_empty() {
                let mut row = vec![vec![]; out::SIZE];
                row[out::EXTENSION] = parts[name::EXTENSION].clone();
                row[out::COMPONENT] = parts[name::COMPONENT].clone();
                row
            } else {
                vec![]
            })
        }
        (ops::KEY_HINTS, hints) if hints.len() == key_hints::SIZE => {
            let mode = |enabled: usize, mode: usize| {
                if wire::flag::decode(&hints[enabled]) {
                    hints[mode].clone()
                } else {
                    text("DISABLED")
                }
            };
            let mut out = vec![vec![]; key_hint_modes::SIZE];
            out[key_hint_modes::NUMBER] = mode(key_hints::NUMBER_ENABLED, key_hints::NUMBER);
            out[key_hint_modes::SYMBOLS] = mode(key_hints::SYMBOLS_ENABLED, key_hints::SYMBOLS);
            Ok(out)
        }
        (ops::CHOICE_INDEX, [target, options @ ..]) => Ok(vec![text(
            &options
                .iter()
                .position(|s| s == target)
                .unwrap_or(0)
                .to_string(),
        )]),
        (ops::SUBTYPE_INDEX, [id, ids @ ..]) => Ok(vec![text(
            &ids.iter()
                .position(|v| v == id)
                .map_or(-1, |i| i as i64)
                .to_string(),
        )]),
        _ => Err("invalid settings packet"),
    }
}
#[cfg(test)]
mod tests;
