//! Utility decisions independent of Android. Text packets preserve UTF-16 exactly.
#![forbid(unsafe_code)]
pub mod locale;
pub mod mime;
pub mod reference;
pub mod validation;
pub mod version;
pub type Text = Vec<u16>;
pub fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
pub fn blank(s: &[u16]) -> bool {
    s.iter().all(|&u| chars::is_whitespace(u))
}
fn flag(b: bool) -> Text {
    text(if b { "1" } else { "0" })
}
fn num(s: &[u16]) -> Result<i32, &'static str> {
    chars::text::to_int_or_null(s).ok_or("invalid utility integer")
}
pub fn call(op: i32, v: &[Text]) -> Result<Vec<Text>, &'static str> {
    Ok(match (op, v) {
        (0, [filter]) => mime::compile(filter),
        (1, values) => mime::inputs(values),
        (2, [value]) => reference::normalize(value),
        (3, [value]) => vec![reference::url(value)],
        (4, [scheme, authority, path]) => reference::facts(scheme, authority, path),
        (5, [tag]) => locale::parse(tag),
        (6, [delimiter, language, country, variant]) => vec![locale::tag(
            u16::try_from(num(delimiter)?).map_err(|_| "invalid locale delimiter")?,
            language,
            country,
            variant,
        )],
        (
            7,
            [
                language,
                country,
                variant,
                raw_language,
                raw_country,
                raw_variant,
            ],
        ) => vec![locale::display(
            language,
            country,
            variant,
            raw_language,
            raw_country,
            raw_variant,
        )],
        (8, [language]) => vec![flag(
            !["zh", "ko", "th", "bn", "hi"]
                .iter()
                .any(|s| language == &text(s)),
        )],
        (9, [raw]) => version::parse(raw),
        (10, values) if values.len() == 10 => vec![text(&version::compare(values)?.to_string())],
        (11, [major, minor, patch, name, value])
            if name == &text("0") || name.first() == Some(&49) =>
        {
            vec![version::format(major, minor, patch, name, value)]
        }
        (12, [install, last, current]) => vec![flag(version::changelog(install, last, current)?)],
        (13, [raw]) => vec![flag(raw == &text("0.0.0"))],
        (14, [rule, value, min, max]) => vec![text(validation::validate(
            rule,
            value,
            num(min)?,
            num(max)?,
        ))],
        (15, [mode, count, filters, facts @ ..]) => vec![flag(mime::reduce(
            num(mode)?,
            usize::try_from(num(count)?).map_err(|_| "invalid MIME count")?,
            usize::try_from(num(filters)?).map_err(|_| "invalid MIME filters")?,
            facts,
        )?)],
        _ => return Err("invalid utility transfer"),
    })
}

#[cfg(test)]
mod tests;
