//! The clipboard history's policy for `ClipboardRules`, with lossless UTF-16 text equality. Its
//! MIME rules are stores::clipboard_policy's own, which the generated entry points call directly.

use stores::clipboard_policy::{self as policy, Clip};

use crate::jni::{Result, illegal};

/// The history's order, its three groups, the clips over the limit and expired, the duplicate of
/// the last text and whether it is pinned, and the unpinned clips: five longs a clip in
/// `records`, one text a clip in `texts` and the new text last, with ten settings in `config`.
pub fn plan(records: &[i64], texts: &[Vec<u16>], config: &[i64]) -> Result<Vec<Vec<i32>>> {
    let c = config;
    if !records.len().is_multiple_of(5) || texts.len() != records.len() / 5 + 1 || c.len() != 10 {
        return Err(illegal("clipboard payload"));
    }
    let clips: Vec<_> = records
        .as_chunks::<5>()
        .0
        .iter()
        .enumerate()
        .map(|(i, d)| Clip {
            timestamp: d[0],
            kind: d[1] as i32,
            pinned: d[2] != 0,
            sensitive: d[3] != 0,
            text: (d[4] != 0).then(|| texts[i].clone()),
        })
        .collect();
    let [first, second, third] = policy::groups(&clips, c[0]);
    let dup = policy::duplicate(
        &clips,
        (c[8] != 0).then(|| texts.last().unwrap().as_slice()),
        c[1] != 0,
        c[9] != 0,
    );
    let ints = |v: Vec<usize>| v.into_iter().map(|i| i as i32).collect();
    Ok(vec![
        ints(policy::order(&clips)),
        ints(first),
        ints(second),
        ints(third),
        ints(policy::limit(&clips, c[0], c[2] != 0, c[3] as i32)),
        ints(policy::expiry(
            &clips,
            c[0],
            c[4] != 0,
            c[5] as i32,
            c[6] != 0,
            c[7] as i32,
        )),
        dup.map_or(vec![-1, 0], |(i, p)| vec![i as i32, i32::from(p)]),
        ints(policy::unpinned(&clips)),
    ])
}
