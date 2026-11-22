//! What the language screens' tests share: subtypes as the keyboard stores them, a report as
//! Kotlin sends it, an app on any language screen, and taps by control id.
use render::{
    settings_ui::{App, Control, languages},
    text::Text,
};
use wire::settings_ui::languages::{kind, row};

/// The time a subtype added in these tests is stamped with.
pub const NOW: i64 = 1_800_000_000_000;
fn now() -> i64 {
    NOW
}

/// A subtype exactly as SubtypeManager stores one: kotlinx's field order, defaults written.
pub fn stored(id: i64, locale: &str, popup: &str, characters: &str, currency: &str) -> String {
    format!(
        "{{\"id\":{id},\"primaryLocale\":\"{locale}\",\"secondaryLocales\":[],\"nlpProviders\":\
         {{\"spelling\":\"org.florisboard.nlp.providers.latin\",\"suggestion\":\
         \"org.florisboard.nlp.providers.latin\"}},\"composer\":\"org.florisboard.composers:appender\",\
         \"currencySet\":\"org.florisboard.currencysets:{currency}\",\"punctuationRule\":\
         \"org.florisboard.localization:default\",\"popupMapping\":\"org.florisboard.localization:{popup}\",\
         \"layoutMap\":{{\"characters\":\"org.florisboard.layouts:{characters}\",\"symbols\":\
         \"org.florisboard.layouts:western\",\"symbols2\":\"org.florisboard.layouts:western\",\
         \"numeric\":\"org.florisboard.layouts:western_arabic\",\"numericAdvanced\":\
         \"org.florisboard.layouts:western_arabic\",\"numericRow\":\"org.florisboard.layouts:western_arabic\",\
         \"phone\":\"org.florisboard.layouts:telpad\",\"phone2\":\"org.florisboard.layouts:telpad\"}}}}"
    )
}
pub fn english() -> String {
    stored(1, "en-US", "en", "qwerty", "dollar")
}
pub fn german() -> String {
    stored(2, "de-DE", "de", "qwertz", "euro")
}
pub fn french() -> String {
    stored(3, "fr-FR", "fr", "azerty", "euro")
}
/// A stored list of `items`, as `persistNewSubtypeList` writes it.
pub fn list(items: &[String]) -> String {
    format!("[{}]", items.join(","))
}

/// One report row.
pub fn report_row(fields: &[(usize, &str)]) -> Vec<String> {
    let mut r = vec![String::new(); row::SIZE];
    for (at, value) in fields {
        r[*at] = (*value).into();
    }
    r
}
/// A locale's row of `k`, with its names.
pub fn locale(k: &str, tag: &str, label: &str, native: &str) -> Vec<String> {
    let base = tag.replace('-', "_");
    let language = tag.split('-').next().unwrap();
    report_row(&[
        (row::KIND, k),
        (row::ID, tag),
        (row::LABEL, label),
        (row::NATIVE, native),
        (row::ENGLISH, label),
        (row::DETAIL, &base),
        (row::BASE, &base),
        (row::LANGUAGE, language),
    ])
}
fn component(k: &str, id: &str, label: &str, detail: &str) -> Vec<String> {
    report_row(&[
        (row::KIND, k),
        (row::ID, id),
        (row::LABEL, label),
        (row::DETAIL, detail),
    ])
}
/// What Kotlin reports with `raw` stored: three presets, a handful of locales, the system set
/// to English (US) then German, and two language packs.
pub fn report(raw: &str) -> Vec<String> {
    let mut rows = vec![report_row(&[
        (row::KIND, kind::SUBTYPES),
        (row::ID, raw),
        (row::DETAIL, raw),
    ])];
    let locales = [
        (
            "en-US",
            "English (United States)",
            "English (United States)",
        ),
        ("de-DE", "German (Germany)", "Deutsch (Deutschland)"),
        ("fr-FR", "French (France)", "français (France)"),
        ("fr-CA", "French (Canada)", "français (Canada)"),
        ("es-ES", "Spanish (Spain)", "español (España)"),
        ("ja-JP", "Japanese (Japan)", "日本語 (日本)"),
    ];
    for (tag, label, native) in locales {
        rows.push(locale(kind::LOCALE, tag, label, native));
        rows.push(locale(kind::NAME, tag, label, native));
    }
    rows.push(locale(kind::SYSTEM, "en-US", locales[0].1, locales[0].2));
    rows.push(locale(kind::SYSTEM, "de-DE", locales[1].1, locales[1].2));
    let l = "org.florisboard.layouts:";
    for (id, label) in [
        ("qwerty", "QWERTY"),
        ("qwertz", "QWERTZ"),
        ("azerty", "AZERTY"),
    ] {
        rows.push(component(
            kind::LAYOUT,
            &format!("{l}{id}"),
            label,
            "characters",
        ));
    }
    for t in ["symbols", "symbols2"] {
        rows.push(component(
            kind::LAYOUT,
            &format!("{l}western"),
            "Western",
            t,
        ));
    }
    for t in ["numeric", "numericAdvanced", "numericRow"] {
        rows.push(component(
            kind::LAYOUT,
            &format!("{l}western_arabic"),
            "Western Arabic",
            t,
        ));
    }
    for t in ["phone", "phone2"] {
        rows.push(component(kind::LAYOUT, &format!("{l}telpad"), "Telpad", t));
    }
    rows.push(component(
        kind::COMPOSER,
        "org.florisboard.composers:appender",
        "Appender",
        "",
    ));
    for (id, label) in [("dollar", "Dollar"), ("euro", "Euro")] {
        rows.push(component(
            kind::CURRENCY,
            &format!("org.florisboard.currencysets:{id}"),
            label,
            "",
        ));
    }
    for id in ["default", "en", "de", "fr"] {
        rows.push(component(
            kind::POPUP,
            &format!("org.florisboard.localization:{id}"),
            id,
            "",
        ));
    }
    for (tag, value) in [
        ("en-US", english()),
        ("de-DE", german()),
        ("fr-FR", french()),
    ] {
        let base = tag.replace('-', "_");
        let preset = value.replacen(&value[6..value.find(',').unwrap()], "-1", 1);
        rows.push(report_row(&[
            (row::KIND, kind::PRESET),
            (row::ID, tag),
            (row::DETAIL, &preset),
            (row::BASE, &base),
            (row::LANGUAGE, tag.split('-').next().unwrap()),
        ]));
    }
    rows.push(component(
        kind::PACK,
        "org.florisboard.han",
        "Han language pack",
        "true",
    ));
    rows.push(component(
        kind::PACK_ITEM,
        "org.florisboard.han",
        "Zhengma",
        "",
    ));
    rows.push(component(
        kind::PACK_ITEM,
        "org.florisboard.han",
        "Cangjie",
        "",
    ));
    rows.push(component(
        kind::PACK,
        "org.example.builtin",
        "Built in",
        "false",
    ));
    rows.concat()
}

pub fn text() -> Text {
    Text::new(vec![
        include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ])
}
/// An app on `route` with `raw` stored and Kotlin's report in, painted, with no effects left.
pub fn app(route: &str, raw: &str) -> (App, Text) {
    let mut a = App {
        config: [1080.0, 6000.0, 2.625, 1.0],
        now,
        ..App::default()
    };
    a.prefs.insert(languages::SUBTYPES_KEY.into(), raw.into());
    a.navigate(languages::LIST_ROUTE);
    if route != languages::LIST_ROUTE {
        a.navigate(route);
    }
    assert!(
        a.effects.contains(&vec!["languages".to_string()]),
        "{route} asked for no report"
    );
    a.set_languages(&report(raw)).unwrap();
    let mut text = text();
    a.scene(&mut text);
    a.effects.clear();
    (a, text)
}
pub fn control(a: &App, id: i32) -> Option<Control> {
    a.controls.iter().find(|c| c.id == id).cloned()
}
/// Paints, taps `id` as it now stands, and paints again.
pub fn tap(a: &mut App, text: &mut Text, id: i32) -> bool {
    a.scene(text);
    let tapped = a.activate(id);
    a.scene(text);
    tapped
}
/// The subtype list written last.
pub fn written(a: &App) -> Option<String> {
    a.effects
        .iter()
        .rev()
        .find(|e| e[0] == "write" && e[1] == languages::SUBTYPES_KEY)
        .map(|e| e[2].clone())
}
