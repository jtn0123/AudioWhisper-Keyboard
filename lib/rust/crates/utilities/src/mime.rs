//! MIME policy. The host executes regex predicates using its exact platform grammar.
use crate::{Text, text};

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
        return vec![error];
    }
    let mut out = vec![vec![]];
    for part in pieces {
        let mut pattern = vec![];
        for &unit in part {
            if unit == 42 {
                pattern.extend(text(r"[^\s]+"));
            } else {
                pattern.push(unit);
            }
        }
        out.push(pattern);
    }
    out
}

/// Each admitted value becomes an original index plus its two nonempty components.
pub fn inputs(values: &[Text]) -> Vec<Text> {
    values
        .iter()
        .enumerate()
        .filter_map(|(i, value)| {
            parts(value).map(|(a, b)| [text(&i.to_string()), a.to_vec(), b.to_vec()])
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
        .checked_mul(2)
        .and_then(|n| n.checked_add(1))
        .ok_or("invalid MIME width")?;
    let mut rows = facts.chunks_exact(width);
    let mut seen = std::collections::HashSet::new();
    let mut matched = 0;
    for row in &mut rows {
        let index = usize::try_from(crate::num(&row[0])?).map_err(|_| "invalid MIME index")?;
        if index >= count || !seen.insert(index) {
            return Err("invalid MIME index");
        }
        if row[1..]
            .as_chunks::<2>()
            .0
            .iter()
            .any(|pair| pair[0] == text("1") && pair[1] == text("1"))
        {
            matched += 1;
        }
    }
    if !rows.remainder().is_empty() {
        return Err("invalid MIME facts");
    }
    Ok(match mode {
        0 | 2 => matched > 0,
        1 => count > 0 && matched == count,
        3 => matched == 1,
        _ => return Err("invalid MIME mode"),
    })
}
