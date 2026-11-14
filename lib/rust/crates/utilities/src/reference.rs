//! Uri parsing/encoding itself remains Android's Uri; these are FlorisRef's policies.
use crate::{Text, blank, flag, text};
pub fn normalize(value: &[u16]) -> Vec<Text> {
    for (legacy, authority) in [("assets:", "assets"), ("internal:", "internal")] {
        if value.starts_with(&text(legacy)) {
            let path = &value[legacy.len()..];
            let mut normalized = vec![];
            if !path.is_empty() && path[0] != 47 {
                normalized.push(47);
            }
            normalized.extend(path);
            return vec![text(authority), normalized];
        }
    }
    vec![vec![], value.to_vec()]
}
pub fn url(value: &[u16]) -> Text {
    if ["http://", "https://", "mailto:"]
        .iter()
        .any(|s| value.starts_with(&text(s)))
    {
        return value.to_vec();
    }
    let mut out = text("https://");
    out.extend(value);
    out
}
pub fn facts(scheme: &[u16], authority: &[u16], path: &[u16]) -> Vec<Text> {
    let internal = scheme == text("florisboard");
    let kind = if internal {
        ["app-ui", "assets", "cache", "internal"]
            .iter()
            .position(|s| authority == text(s))
            .map_or(0, |i| i + 1)
    } else {
        0
    };
    vec![
        text(&kind.to_string()),
        flag(!internal),
        flag(!blank(scheme) && !blank(authority)),
        path.strip_prefix(&[47]).unwrap_or(path).to_vec(),
    ]
}
