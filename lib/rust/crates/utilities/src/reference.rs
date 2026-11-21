//! Uri parsing/encoding itself remains Android's Uri; these are FlorisRef's policies.
use crate::{Text, blank, text};
use wire::flag;
use wire::utilities::{self as ops, reference, reference_facts as facts};
pub fn normalize(value: &[u16]) -> Vec<Text> {
    for (legacy, authority) in [("assets:", "assets"), ("internal:", "internal")] {
        if value.starts_with(&text(legacy)) {
            let path = &value[legacy.len()..];
            let mut normalized = vec![];
            if !path.is_empty() && path[0] != 47 {
                normalized.push(47);
            }
            normalized.extend(path);
            return row(text(authority), normalized);
        }
    }
    row(vec![], value.to_vec())
}
/// A `wire::utilities::reference` row.
fn row(authority: Text, path: Text) -> Vec<Text> {
    let mut row = vec![vec![]; reference::SIZE];
    row[reference::AUTHORITY] = authority;
    row[reference::PATH] = path;
    row
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
    let kind = [
        (ops::APP_UI, "app-ui"),
        (ops::ASSETS, "assets"),
        (ops::CACHE, "cache"),
        (ops::INTERNAL, "internal"),
    ]
    .into_iter()
    .find(|(_, name)| internal && authority == text(name))
    .map_or(ops::OTHER, |(kind, _)| kind);
    let mut row = vec![vec![]; facts::SIZE];
    row[facts::KIND] = text(&kind.to_string());
    row[facts::EXTERNAL] = flag::utf16(!internal);
    row[facts::HAS_AUTHORITY] = flag::utf16(!blank(scheme) && !blank(authority));
    row[facts::RELATIVE_PATH] = path.strip_prefix(&[47]).unwrap_or(path).to_vec();
    row
}
