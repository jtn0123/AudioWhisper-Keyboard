//! Exact legacy VersionName behavior, including its broken dash-suffix parsing.
use crate::{Text, num, text};
pub fn parse(raw: &[u16]) -> Vec<Text> {
    let parts = raw.split(|u| *u == 46).collect::<Vec<_>>();
    if parts.len() != 3 {
        return vec![];
    }
    let digits = |s: &[u16]| !s.is_empty() && s.iter().all(|u| (48..=57).contains(u));
    if !digits(parts[0]) || !digits(parts[1]) {
        return vec![];
    }
    let patch = parts[2];
    if digits(patch) {
        let mut out = vec![text("ok")];
        for part in &parts {
            let Some(n) = chars::text::to_int_or_null(part) else {
                return vec![text("number-error"), part.to_vec()];
            };
            out.push(text(&n.to_string()));
        }
        out.extend([vec![], vec![]]);
        return out;
    }
    // Numeric suffix runs toInt on the whole third component before checking list size.
    if let Some(at) = patch.iter().position(|u| *u == 45)
        && digits(&patch[..at])
        && digits(&patch[at + 1..])
    {
        for part in parts {
            if chars::text::to_int_or_null(part).is_none() {
                return vec![text("number-error"), part.to_vec()];
            }
        }
    }
    // Named suffixes split on '.' then test size == 4; no matching input can satisfy it.
    vec![]
}
pub fn compare(values: &[Text]) -> Result<i32, &'static str> {
    for i in 0..3 {
        let order = num(&values[i])?.cmp(&num(&values[i + 5])?);
        if !order.is_eq() {
            return Ok(order as i32);
        }
    }
    if !values[4].is_empty() && !values[9].is_empty() {
        return Ok(num(&values[4])?.cmp(&num(&values[9])?) as i32);
    }
    Ok(0)
}
pub fn format(major: &[u16], minor: &[u16], patch: &[u16], name: &[u16], value: &[u16]) -> Text {
    let mut out = major.to_vec();
    out.push(46);
    out.extend(minor);
    out.push(46);
    out.extend(patch);
    if name.first() == Some(&49) || !value.is_empty() {
        out.push(46);
        out.extend(&name[1..]);
        out.extend(value);
    }
    out
}
pub fn changelog(install: &[u16], last: &[u16], current: &[u16]) -> Result<bool, &'static str> {
    let default = || vec![text("0"), text("0"), text("0"), vec![], vec![]];
    let parse_or_default = |s| {
        let p = parse(s);
        if p.first() == Some(&text("number-error")) {
            Err("invalid version number")
        } else if p.is_empty() {
            Ok(default())
        } else {
            Ok(p[1..].to_vec())
        }
    };
    let install = parse_or_default(install)?;
    let last = parse_or_default(last)?;
    let current = parse_or_default(current)?;
    let mut pair = last;
    pair.extend(current.clone());
    Ok(compare(&pair)? < 0 && install != current)
}
