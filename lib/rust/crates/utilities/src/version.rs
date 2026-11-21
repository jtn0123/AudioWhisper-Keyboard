//! Exact legacy VersionName behavior, including its broken dash-suffix parsing.
use crate::{Text, num, text};
use wire::flag;
use wire::utilities::version::{EXTRA_NAME, EXTRA_VALUE, MAJOR, MINOR, PATCH, SIZE};
use wire::utilities::{NUMBER_ERROR, OK, version_plan as plan};

/// A `wire::utilities::version_plan` row that says `part` is not a number.
fn number_error(part: &[u16]) -> Vec<Text> {
    let mut row = vec![vec![]; plan::SIZE];
    row[plan::STATUS] = text(NUMBER_ERROR);
    row[plan::BAD_NUMBER] = part.to_vec();
    row
}
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
        let mut out = vec![vec![]; plan::SIZE];
        out[plan::STATUS] = text(OK);
        for (field, part) in [plan::MAJOR, plan::MINOR, plan::PATCH]
            .into_iter()
            .zip(parts)
        {
            let Some(n) = chars::text::to_int_or_null(part) else {
                return number_error(part);
            };
            out[field] = text(&n.to_string());
        }
        return out;
    }
    // Numeric suffix runs toInt on the whole third component before checking list size.
    if let Some(at) = patch.iter().position(|u| *u == 45)
        && digits(&patch[..at])
        && digits(&patch[at + 1..])
    {
        for part in parts {
            if chars::text::to_int_or_null(part).is_none() {
                return number_error(part);
            }
        }
    }
    // Named suffixes split on '.' then test size == 4; no matching input can satisfy it.
    vec![]
}
/// Two `wire::utilities::version` rows, one after the other.
pub fn compare(values: &[Text]) -> Result<i32, &'static str> {
    let (a, b) = values.split_at(SIZE);
    for field in [MAJOR, MINOR, PATCH] {
        let order = num(&a[field])?.cmp(&num(&b[field])?);
        if !order.is_eq() {
            return Ok(order as i32);
        }
    }
    if !a[EXTRA_VALUE].is_empty() && !b[EXTRA_VALUE].is_empty() {
        return Ok(num(&a[EXTRA_VALUE])?.cmp(&num(&b[EXTRA_VALUE])?) as i32);
    }
    Ok(0)
}
/// A `wire::utilities::version` row, its extra name after a flag of whether it has one.
pub fn format(version: &[Text]) -> Text {
    let (name, value) = (&version[EXTRA_NAME], &version[EXTRA_VALUE]);
    let mut out = version[MAJOR].clone();
    out.push(46);
    out.extend(&version[MINOR]);
    out.push(46);
    out.extend(&version[PATCH]);
    if name.starts_with(&text(flag::TRUE)) || !value.is_empty() {
        out.push(46);
        // The name, after its flag.
        out.extend(&name[1..]);
        out.extend(value);
    }
    out
}
pub fn changelog(install: &[u16], last: &[u16], current: &[u16]) -> Result<bool, &'static str> {
    let parse_or_default = |s| {
        let p = parse(s);
        let mut version = vec![vec![]; SIZE];
        if p.is_empty() {
            for field in [MAJOR, MINOR, PATCH] {
                version[field] = text("0");
            }
        } else if p[plan::STATUS] == text(NUMBER_ERROR) {
            return Err("invalid version number");
        } else {
            for (field, parsed) in [
                (MAJOR, plan::MAJOR),
                (MINOR, plan::MINOR),
                (PATCH, plan::PATCH),
                (EXTRA_NAME, plan::EXTRA_NAME),
                (EXTRA_VALUE, plan::EXTRA_VALUE),
            ] {
                version[field] = p[parsed].clone();
            }
        }
        Ok(version)
    };
    let install = parse_or_default(install)?;
    let last = parse_or_default(last)?;
    let current = parse_or_default(current)?;
    let mut pair = last;
    pair.extend(current.clone());
    Ok(compare(&pair)? < 0 && install != current)
}
