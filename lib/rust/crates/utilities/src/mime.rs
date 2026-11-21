//! MIME policy. The host executes regex predicates using its exact platform grammar.
use crate::{Text, text};
use wire::flag;
use wire::utilities::{self as ops, mime_facts, mime_filter, mime_input, mime_match};

fn parts(value: &[u16]) -> Option<(&[u16], &[u16])> {
    let mut parts = value.split(|u| *u == 47);
    let a = parts.next()?;
    let b = parts.next()?;
    (!a.is_empty() && !b.is_empty() && parts.next().is_none()).then_some((a, b))
}

/// Error followed by pairs of platform regex patterns. Process filters in input order,
/// so a host grammar error in an earlier filter still precedes a later shape error.
pub fn compile(filter: &[u16]) -> Vec<Text> {
    let pieces = filter.split(|u| *u == 47).collect::<Vec<_>>();
    let reason = if pieces.len() != 2 {
        Some("Must be type/subtype.")
    } else if pieces.iter().any(|s| s.is_empty()) {
        Some("Type or subtype empty.")
    } else {
        None
    };
    if let Some(reason) = reason {
        let mut error = text("Ill-formatted MIME type filter '");
        error.extend(filter);
        error.extend(text(&format!("'. {reason}")));
        let mut row = vec![vec![]; mime_filter::SIZE];
        row[mime_filter::ERROR] = error;
        return row;
    }
    let mut out = vec![vec![]; mime_filter::SIZE];
    for (field, part) in [mime_filter::TYPE, mime_filter::SUBTYPE]
        .into_iter()
        .zip(pieces)
    {
        let mut pattern = vec![];
        for &unit in part {
            if unit == 42 {
                pattern.extend(text(r"[^\s]+"));
            } else {
                pattern.push(unit);
            }
        }
        out[field] = pattern;
    }
    out
}

/// Each admitted value becomes an original index plus its two nonempty components.
pub fn inputs(values: &[Text]) -> Vec<Text> {
    values
        .iter()
        .enumerate()
        .filter_map(|(i, value)| {
            parts(value).map(|(a, b)| {
                let mut row = vec![vec![]; mime_input::SIZE];
                row[mime_input::INDEX] = text(&i.to_string());
                row[mime_input::TYPE] = a.to_vec();
                row[mime_input::SUBTYPE] = b.to_vec();
                row
            })
        })
        .flatten()
        .collect()
}

/// Host facts are rows: original index, then (type match, subtype match) per filter.
/// Invalid MIME values never get regex tasks, and remain false in the collection.
pub fn reduce(
    mode: i32,
    count: usize,
    filters: usize,
    facts: &[Text],
) -> Result<bool, &'static str> {
    let width = filters
        .checked_mul(mime_match::SIZE)
        .and_then(|n| n.checked_add(mime_facts::SIZE))
        .ok_or("invalid MIME width")?;
    let mut rows = facts.chunks_exact(width);
    let mut seen = std::collections::HashSet::new();
    let mut matched = 0;
    for row in &mut rows {
        let index = usize::try_from(crate::num(&row[mime_facts::INDEX])?)
            .map_err(|_| "invalid MIME index")?;
        if index >= count || !seen.insert(index) {
            return Err("invalid MIME index");
        }
        if row[mime_facts::SIZE..]
            .as_chunks::<{ mime_match::SIZE }>()
            .0
            .iter()
            .any(|pair| {
                flag::decode(&pair[mime_match::TYPE]) && flag::decode(&pair[mime_match::SUBTYPE])
            })
        {
            matched += 1;
        }
    }
    if !rows.remainder().is_empty() {
        return Err("invalid MIME facts");
    }
    Ok(match mode {
        ops::SINGLE | ops::ANY => matched > 0,
        ops::ALL => count > 0 && matched == count,
        ops::ONE => matched == 1,
        _ => return Err("invalid MIME mode"),
    })
}
