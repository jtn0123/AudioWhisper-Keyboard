//! Frozen Floris clipboard policy. Android owns storage and executes these index plans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clip {
    pub timestamp: i64,
    pub kind: i32,
    pub pinned: bool,
    pub sensitive: bool,
    pub text: Option<Vec<u16>>,
}
/// Stable descending timestamps, including equal timestamps in Room's input order.
pub fn order(clips: &[Clip]) -> Vec<usize> {
    let mut indices: Vec<_> = (0..clips.len()).collect();
    indices.sort_by_key(|&i| std::cmp::Reverse(clips[i].timestamp));
    indices
}
pub fn unpinned(clips: &[Clip]) -> Vec<usize> {
    clips
        .iter()
        .enumerate()
        .filter_map(|(i, c)| (!c.pinned).then_some(i))
        .collect()
}
pub fn group(clip: &Clip, now: i64) -> usize {
    if clip.pinned {
        0
    } else if now.wrapping_sub(clip.timestamp) < 300_000 {
        1
    } else {
        2
    }
}
pub fn groups(clips: &[Clip], now: i64) -> [Vec<usize>; 3] {
    let mut out: [Vec<usize>; 3] = Default::default();
    for (i, clip) in clips.iter().enumerate() {
        out[group(clip, now)].push(i);
    }
    out
}
/// Preserve Compose's TEXT-only comparison, including nullable text equality.
pub fn duplicate(
    clips: &[Clip],
    text: Option<&[u16]>,
    enabled: bool,
    new_pinned: bool,
) -> Option<(usize, bool)> {
    if !enabled {
        return None;
    }
    clips
        .iter()
        .enumerate()
        .find(|(_, c)| c.kind == 1 && c.text.as_deref() == text)
        .map(|(i, c)| (i, c.pinned || new_pinned))
}
pub fn limit(clips: &[Clip], now: i64, enabled: bool, maximum: i32) -> Vec<usize> {
    if !enabled {
        return vec![];
    }
    let g = groups(clips, now);
    let unpinned: Vec<_> = g[1].iter().chain(&g[2]).copied().collect();
    let count = (unpinned.len() as i32).wrapping_sub(maximum);
    unpinned
        .into_iter()
        .rev()
        .take(count.max(0) as usize)
        .collect()
}
/// Kotlin computes the preference duration as Int before widening to Long. Sensitive pins expire too.
pub fn expiry(
    clips: &[Clip],
    now: i64,
    old: bool,
    minutes: i32,
    sensitive: bool,
    seconds: i32,
) -> Vec<usize> {
    let old_time = now.wrapping_sub(minutes.wrapping_mul(60).wrapping_mul(1000) as i64);
    let sensitive_time = now.wrapping_sub(seconds.wrapping_mul(1000) as i64);
    let g = groups(clips, now);
    let mut out = vec![];
    if old {
        out.extend(
            g[1].iter()
                .chain(&g[2])
                .copied()
                .filter(|&i| clips[i].timestamp < old_time),
        );
    }
    if sensitive {
        for (i, c) in clips.iter().enumerate() {
            if c.sensitive && c.timestamp < sensitive_time && !out.contains(&i) {
                out.push(i);
            }
        }
    }
    out
}
pub fn mime_matches(concrete: &[u16], desired: &[u16]) -> bool {
    if desired == [42, 47, 42] {
        return true;
    }
    match desired.iter().position(|&c| c == 47) {
        Some(slash) if slash > 0 => {
            if desired.len() == slash + 2 && desired[slash + 1] == 42 {
                concrete.starts_with(&desired[..slash + 1])
            } else {
                concrete == desired
            }
        }
        _ => false,
    }
}
pub fn eligible(types: &[Vec<u16>], editor: &[Vec<u16>]) -> bool {
    let plain: Vec<_> = "text/plain".encode_utf16().collect();
    types.contains(&plain)
        || editor
            .iter()
            .any(|e| types.iter().any(|c| mime_matches(c, e)))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_and_sensitive_pins() {
        let c = Clip {
            timestamp: 0,
            kind: 1,
            pinned: false,
            sensitive: true,
            text: None,
        };
        assert_eq!(group(&c, 299_999), 1);
        assert_eq!(group(&c, 300_000), 2);
        let p = Clip {
            pinned: true,
            ..c.clone()
        };
        assert!(expiry(std::slice::from_ref(&p), 60_001, true, 1, false, 1).is_empty());
        assert_eq!(expiry(&[p], 1001, false, 1, true, 1), vec![0]);
        assert_eq!(duplicate(&[c], None, true, false), Some((0, false)));
    }
}
