//! Utility decisions independent of Android. Text packets preserve UTF-16 exactly.
#![forbid(unsafe_code)]
pub mod locale;
pub mod mime;
pub mod reference;
pub mod validation;
pub mod version;

use wire::flag;
use wire::utilities::{
    self as ops, locale_display, locale_tag, mime_request, validation as checked, version as fields,
};
pub type Text = Vec<u16>;
pub fn text(s: &str) -> Text {
    s.encode_utf16().collect()
}
pub fn blank(s: &[u16]) -> bool {
    s.iter().all(|&u| chars::is_whitespace(u))
}
fn num(s: &[u16]) -> Result<i32, &'static str> {
    chars::text::to_int_or_null(s).ok_or("invalid utility integer")
}
pub fn call(op: i32, v: &[Text]) -> Result<Vec<Text>, &'static str> {
    Ok(match (op, v) {
        (ops::MIME_FILTER, [filter]) => mime::compile(filter),
        (ops::MIME_INPUTS, values) => mime::inputs(values),
        (ops::NORMALIZE_REFERENCE, [value]) => reference::normalize(value),
        (ops::URL, [value]) => vec![reference::url(value)],
        (ops::REFERENCE_FACTS, [scheme, authority, path]) => {
            reference::facts(scheme, authority, path)
        }
        (ops::PARSE_LOCALE, [tag]) => locale::parse(tag),
        (ops::LOCALE_TAG, v) if v.len() == locale_tag::SIZE => vec![locale::tag(
            u16::try_from(num(&v[locale_tag::DELIMITER])?)
                .map_err(|_| "invalid locale delimiter")?,
            &v[locale_tag::LANGUAGE],
            &v[locale_tag::COUNTRY],
            &v[locale_tag::VARIANT],
        )],
        (ops::LOCALE_DISPLAY_NAME, v) if v.len() == locale_display::SIZE => {
            use locale_display::*;
            vec![locale::display(
                &v[LANGUAGE],
                &v[COUNTRY],
                &v[VARIANT],
                &v[RAW_LANGUAGE],
                &v[RAW_COUNTRY],
                &v[RAW_VARIANT],
            )]
        }
        (ops::CAPITALIZES, [language]) => vec![flag::utf16(
            !["zh", "ko", "th", "bn", "hi"]
                .iter()
                .any(|s| language == &text(s)),
        )],
        (ops::PARSE_VERSION, [raw]) => version::parse(raw),
        (ops::COMPARE_VERSIONS, values) if values.len() == 2 * fields::SIZE => {
            vec![text(&version::compare(values)?.to_string())]
        }
        (ops::FORMAT_VERSION, v)
            if v.len() == fields::SIZE && {
                let name = &v[fields::EXTRA_NAME];
                name == &text(flag::FALSE) || name.starts_with(&text(flag::TRUE))
            } =>
        {
            vec![version::format(v)]
        }
        (ops::SHOWS_CHANGELOG, [install, last, current]) => {
            vec![flag::utf16(version::changelog(install, last, current)?)]
        }
        (ops::UNSET_VERSION, [raw]) => vec![flag::utf16(raw == &text("0.0.0"))],
        (ops::VALIDATE, v) if v.len() == checked::SIZE => vec![text(validation::validate(
            &v[checked::RULE],
            &v[checked::VALUE],
            num(&v[checked::MIN])?,
            num(&v[checked::MAX])?,
        ))],
        (ops::MIME_MATCH, v) if v.len() >= mime_request::SIZE => {
            use mime_request::{COUNT, FILTERS, MODE, SIZE};
            vec![flag::utf16(mime::reduce(
                num(&v[MODE])?,
                usize::try_from(num(&v[COUNT])?).map_err(|_| "invalid MIME count")?,
                usize::try_from(num(&v[FILTERS])?).map_err(|_| "invalid MIME filters")?,
                &v[SIZE..],
            )?)]
        }
        _ => return Err("invalid utility transfer"),
    })
}

#[cfg(test)]
mod tests;
