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
        (0, []) => Ok(catalogue::all()
            .iter()
            .flat_map(|p| [p.key, p.kind, p.default, p.min, p.max, p.step, p.choices].map(text))
            .collect()),
        (1, [key, value]) => Ok(vec![text(&catalogue::enum_index(key, value)?.to_string())]),
        (2, [key, value]) => Ok(vec![catalogue::clamp(key, value)?]),
        (3, [key, raw]) => scalar::decode(key, raw),
        (10, [kind, json]) => {
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
        (11, [kind, rest @ ..]) => {
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
        (12, [kind, key, value]) => migration::migrate(kind, key, value),
        (20, []) => {
            let mut out = vec![];
            codec::default_subtype().pack(&mut out);
            Ok(out)
        }
        (21, rest) => {
            let mut iter = rest.iter();
            let request = Value::unpack(&mut iter, 0)?;
            if iter.len() != 0 {
                return Err("trailing packet");
            }
            let mut out = vec![];
            subtype::plan(&request)?.pack(&mut out);
            Ok(out)
        }
        (22, [locale, language, presets @ ..]) => Ok(vec![text(
            &subtype::preset(locale, language, presets).to_string(),
        )]),
        (23, fields) => Ok(vec![text(
            if fields.len() == 15
                && fields.first().is_some_and(|v| v != &text("00-00"))
                && fields[1..].iter().all(|v| v != &text("00:00"))
            {
                "1"
            } else {
                "0"
            },
        )]),
        (24, [value]) => {
            let s = String::from_utf16(value).map_err(|_| "invalid color")?;
            let parsed = u64::from_str_radix(&s, 16).map_err(|_| "invalid color")?;
            if s.is_empty() || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
                return Err("invalid color");
            }
            Ok(vec![text(&format!("{parsed:016x}"))])
        }
        (26, [raw]) => Ok(time::decode(raw)),
        (27, values) => Ok(vec![time::encode(values)?]),
        (28, values) if values.len() == 8 => {
            let mut out = vec![];
            for (n, index) in [0, 1, 2, 5, 3, 4, 6, 7].iter().enumerate() {
                if n > 0 {
                    out.push(44);
                }
                out.extend(text(codec::LAYOUTS[*index].0));
                out.push(61);
                out.extend(&values[*index]);
            }
            Ok(vec![out])
        }
        (31, [extension, component]) => {
            let mut out = extension.clone();
            out.push(58);
            out.extend(component);
            Ok(vec![out])
        }
        (32, [raw]) => {
            let result = extensions::call(3, std::slice::from_ref(raw))
                .map_err(|_| "invalid component packet")?;
            Ok(if result[0].is_empty() {
                result[1..].to_vec()
            } else {
                vec![]
            })
        }
        (29, [number_enabled, number, symbol_enabled, symbol]) => Ok(vec![
            if number_enabled == &text("true") {
                number.clone()
            } else {
                text("DISABLED")
            },
            if symbol_enabled == &text("true") {
                symbol.clone()
            } else {
                text("DISABLED")
            },
        ]),
        (30, [target, options @ ..]) => Ok(vec![text(
            &options
                .iter()
                .position(|s| s == target)
                .unwrap_or(0)
                .to_string(),
        )]),
        (25, [id, ids @ ..]) => Ok(vec![text(
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
