//! Locale construction, ICU display names and locale-sensitive casing stay at the platform edge.
use crate::{Text, blank, text};
pub fn parse(tag: &[u16]) -> Vec<Text> {
    let parts = tag
        .split(|u| *u == 45 || *u == 95)
        .take(3)
        .map(<[u16]>::to_vec)
        .collect::<Vec<_>>();
    (0..3)
        .map(|i| parts.get(i).cloned().unwrap_or_default())
        .collect()
}
pub fn tag(delimiter: u16, language: &[u16], country: &[u16], variant: &[u16]) -> Text {
    let mut out = language.to_vec();
    if !blank(language) && !blank(country) {
        out.push(delimiter);
    }
    out.extend(country);
    if !blank(country) && !blank(variant) {
        out.push(delimiter);
    }
    out.extend(variant);
    out
}
pub fn display(
    language: &[u16],
    country: &[u16],
    variant: &[u16],
    raw_language: &[u16],
    raw_country: &[u16],
    raw_variant: &[u16],
) -> Text {
    let language = if blank(language) {
        raw_language
    } else {
        language
    };
    let country = if blank(country) { raw_country } else { country };
    let variant = if blank(variant) { raw_variant } else { variant };
    let mut out = language.to_vec();
    if !blank(country) {
        if !blank(language) {
            out.push(32);
        }
        out.push(40);
        out.extend(country);
        out.push(41);
    }
    if !blank(variant) {
        if !blank(language) || !blank(country) {
            out.push(32);
        }
        out.extend(text("["));
        out.extend(chars::case::uppercase(variant));
        out.push(93);
    }
    out
}
