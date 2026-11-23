//! Subtype edit plans; the caller supplies time, normalized Locale facts and resource snapshots.
use crate::{Result, Value, codec, text, value::object};
/// The stored subtype list, as `SubtypeManager` read it: blank text (Java's whitespace and the
/// three no-break spaces) is an empty list.
pub fn decode_list(raw: &[u16]) -> Result<Value> {
    if raw
        .iter()
        .all(|&u| chars::is_whitespace(u) || matches!(u, 0xa0 | 0x2007 | 0x202f))
    {
        return Ok(Value::List(vec![]));
    }
    codec::decode("subtypes", raw, false)
}
pub fn active(ids: &[i64], current: i64, fallback: i64) -> (i64, bool) {
    let index = ids
        .iter()
        .position(|&id| id == current)
        .or(if ids.is_empty() { None } else { Some(0) });
    (
        index.map_or(-1, |i| i as i64),
        index.map_or(fallback, |i| ids[i]) != current,
    )
}
pub fn cycle(flags: &[i64], reverse: bool) -> i64 {
    let mut trigger = false;
    let mut selected = -1;
    let mut order: Vec<_> = (0..flags.len()).collect();
    if reverse {
        order.reverse();
    }
    for &i in &order {
        if trigger {
            trigger = false;
            selected = i as i64;
        } else if flags[i] != 0 {
            trigger = true;
        }
    }
    if trigger {
        selected = order[0] as i64;
    }
    selected
}
fn excluding_id(a: &Value, b: &Value) -> Result<bool> {
    let mut a = a.clone();
    let mut b = b.clone();
    a.set("id", Value::Number(0))?;
    b.set("id", Value::Number(0))?;
    Ok(a == b)
}
pub fn supported(value: &Value) -> Result<Value> {
    let removed_layouts = include_str!("removed-layouts.txt")
        .lines()
        .map(|v| format!("org.florisboard.layouts:{v}"));
    let removed_popups = include_str!("removed-popups.txt")
        .lines()
        .map(|v| format!("org.florisboard.localization:{v}"));
    let composer = value.get("composer")?.string_value()?;
    let providers = value.get("nlpProviders")?;
    let han = Value::string("org.florisboard.nlp.providers.han.shape");
    let deleted = removed_layouts.into_iter().any(|s| {
        value
            .get("layoutMap")
            .and_then(|v| v.get("characters"))
            .ok()
            == Some(&Value::string(&s))
    }) || [
        "org.florisboard.composers:hangul-unicode",
        "org.florisboard.composers:kana-unicode",
    ]
    .iter()
    .any(|s| text(s) == composer)
        || removed_popups
            .into_iter()
            .any(|s| value.get("popupMapping").ok() == Some(&Value::string(&s)))
        || providers.get("spelling")? == &han
        || providers.get("suggestion")? == &han;
    if deleted {
        let mut out = codec::default_subtype();
        out.set("id", value.get("id")?.clone())?;
        Ok(out)
    } else {
        Ok(value.clone())
    }
}
pub fn plan(request: &Value) -> Result<Value> {
    let op = request.get("op")?.string_value()?;
    let mut list = request.get("list")?.list()?.to_vec();
    let current = request.get("active")?.number()?;
    let item = request.get("item")?;
    let mut changed = false;
    match String::from_utf16_lossy(op).as_str() {
        "load" => {
            for value in &mut list {
                let new = supported(value)?;
                changed |= *value != new;
                *value = new;
            }
        }
        "add" => {
            if !list
                .iter()
                .map(|v| excluding_id(v, item))
                .collect::<Result<Vec<_>>>()?
                .contains(&true)
            {
                let mut new = item.clone();
                new.set("id", request.get("time")?.clone())?;
                list.push(new);
                changed = true;
            }
        }
        "modify" => {
            if let Some(index) = list
                .iter()
                .position(|v| v.get("id").ok() == item.get("id").ok())
            {
                list[index] = item.clone();
                changed = true;
            }
        }
        "remove" => {
            if let Some(index) = list.iter().position(|v| v == item) {
                list.remove(index);
                changed = true;
            }
        }
        "reorder" => {
            let order = request.get("order")?.list()?;
            let indices = order
                .iter()
                .map(Value::number)
                .collect::<Result<Vec<_>>>()?;
            let mut sorted = indices.clone();
            sorted.sort_unstable();
            if sorted == (0..list.len() as i64).collect::<Vec<_>>() {
                list = indices.iter().map(|&i| list[i as usize].clone()).collect();
                changed = true;
            }
        }
        "select" | "evaluate" | "next" | "previous" => {}
        _ => return Err("unknown subtype operation"),
    }
    let ids = list
        .iter()
        .map(|v| v.get("id")?.number())
        .collect::<Result<Vec<_>>>()?;
    let index = if op == text("select") {
        ids.iter()
            .position(|&id| {
                id == request
                    .get("target")
                    .and_then(Value::number)
                    .unwrap_or(current)
            })
            .map_or(-1, |i| i as i64)
    } else if op == text("next") || op == text("previous") {
        let selected = request.get("selected")?;
        cycle(
            &list
                .iter()
                .map(|v| i64::from(v == selected))
                .collect::<Vec<_>>(),
            op == text("previous"),
        )
    } else {
        active(&ids, current, -1).0
    };
    let index = if op == text("select") && index < 0 {
        ids.iter()
            .position(|&id| id == current)
            .map_or(-1, |i| i as i64)
    } else {
        index
    };
    let selected = usize::try_from(index)
        .ok()
        .and_then(|i| list.get(i))
        .cloned()
        .unwrap_or_else(codec::default_subtype);
    let active_id = if op == text("select") && index < 0 {
        current
    } else {
        selected.get("id")?.number()?
    };
    Ok(object([
        ("changed", Value::Number(i64::from(changed))),
        ("active", Value::Number(active_id)),
        (
            "writeActive",
            Value::Number(i64::from(active_id != current)),
        ),
        ("selected", selected),
        ("list", Value::List(list)),
    ]))
}
pub fn preset(locale: &[u16], language: &[u16], presets: &[Vec<u16>]) -> i64 {
    presets
        .as_chunks::<2>()
        .0
        .iter()
        .position(|p| p[0] == locale)
        .or_else(|| {
            presets
                .as_chunks::<2>()
                .0
                .iter()
                .position(|p| p[1] == language)
        })
        .map_or(-1, |i| i as i64)
}
