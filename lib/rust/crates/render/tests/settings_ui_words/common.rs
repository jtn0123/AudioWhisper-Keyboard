//! What the user dictionary tests share: words as Kotlin reports them, an app on any user
//! dictionary screen with its report in, taps by control id, and typing into a field.
use render::{
    settings_ui::{
        App, Control,
        catalogue::{Kind, languages::Lang, words::Part},
        words::{self, Word},
    },
    text::Text,
};
use wire::settings_ui::words::{effect, kind, outcome, row};

pub const PACKAGE: &str = "com.jtn0123.audiowhisper.keyboard";
/// The time these tests run at, in milliseconds.
pub const NOW: i64 = 1_800_000_000_000;
fn now() -> i64 {
    NOW
}

pub fn word(id: i64, text: &str, freq: i32, locale: Option<&str>, shortcut: Option<&str>) -> Word {
    Word {
        id,
        word: text.into(),
        freq,
        locale: locale.map(String::from),
        shortcut: shortcut.map(String::from),
    }
}

/// Five made-up words: two in American English spelt two ways, one for all languages, one in
/// German, and one under `all`, which the Floris dictionary reads as all languages.
pub fn sample() -> Vec<Word> {
    vec![
        word(1, "zorblat", 200, Some("en_US"), None),
        word(2, "Quexor", 120, Some("en-us"), Some("qx")),
        word(3, "blivet", 255, None, None),
        word(4, "frobnic", 90, Some("de_DE"), Some("fb")),
        word(5, "snarf", 10, Some("all"), None),
    ]
}

/// How Kotlin names each stored locale: `displayName()` and `displayLanguage()`.
const NAMES: [(&str, &str, &str); 4] = [
    ("en_US", "English (United States)", "English"),
    ("en-us", "English (United States)", "English"),
    ("de_DE", "German (Germany)", "German"),
    ("all", "all", "all"),
];

/// The report Kotlin sends for dictionary `d` holding `list`, answering an effect that did `how`.
pub fn report(d: &str, on: bool, how: &str, list: &[Word]) -> Vec<String> {
    let mut first = vec![String::new(); row::SIZE];
    first[row::KIND] = kind::DICTIONARY.into();
    first[row::ID] = d.into();
    first[row::WORD] = wire::flag::encode(on).into();
    first[row::LABEL] = how.into();
    let mut out = first;
    for w in list {
        let mut r = words::encode(w);
        r[row::KIND] = kind::WORD.into();
        out.extend(r);
    }
    let mut seen: Vec<&str> = vec![];
    for stored in list.iter().filter_map(|w| w.locale.as_deref()) {
        if seen.contains(&stored) {
            continue;
        }
        seen.push(stored);
        let (_, label, language) = NAMES.iter().find(|n| n.0 == stored).unwrap();
        let mut r = vec![String::new(); row::SIZE];
        r[row::KIND] = kind::LOCALE.into();
        r[row::LOCALE] = stored.into();
        r[row::LABEL] = (*label).into();
        r[row::LANGUAGE] = (*language).into();
        out.extend(r);
    }
    out
}

/// The strings whose text a test reads; the rest show as their keys.
const STRINGS: [(&str, &str); 7] = [
    ("settings__udm__all_languages", "For all languages"),
    ("settings__udm__word_summary_freq", "Frequency: {freq}"),
    (
        "settings__udm__word_summary_freq_shortcut",
        "Frequency: {freq} | Shortcut: {shortcut}",
    ),
    (
        "settings__udm__search_not_found",
        "No word found for \u{201c}{search_term}\u{201d}",
    ),
    (
        "settings__udm__import_line_error",
        "Line {line} of that file is not a word. Nothing was imported.",
    ),
    (
        "settings__udm__dialog__word_error_invalid",
        "Please enter a word matching {regex}",
    ),
    (
        "settings__udm__dialog__freq_label",
        "Frequency (between {f_min} and {f_max})",
    ),
];

pub fn text() -> Text {
    Text::new(vec![
        include_bytes!("../../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ])
}
/// An app that opened each route of `stack` in turn, the first a dictionary's languages, with
/// Kotlin's report of `list` in; painted, with no effects left.
pub fn app(stack: &[&str], list: &[Word]) -> (App, Text) {
    let mut a = App {
        config: [1080.0, 6000.0, 2.625, 1.0],
        now,
        ..App::default()
    };
    a.facts.insert("package".into(), PACKAGE.into());
    for (key, value) in STRINGS {
        a.strings.insert(key.into(), value.into());
    }
    let (d, _) = words::place(stack[0]).unwrap();
    for route in stack {
        a.navigate(route);
    }
    assert!(
        a.effects
            .contains(&vec![effect::LOAD.to_string(), d.to_string()]),
        "{} asked for no report",
        stack[0]
    );
    a.set_words(&report(d, true, outcome::NONE, list)).unwrap();
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
/// The control id of the first row whose kind matches.
pub fn row_id(a: &App, pick: impl Fn(&Kind) -> bool) -> i32 {
    1000 + a.rows().iter().position(|r| pick(&r.kind)).unwrap() as i32
}
pub fn part_id(a: &App, part: Part) -> i32 {
    row_id(a, |k| matches!(k, Kind::Words(q) if *q == part))
}
pub fn lang_id(a: &App, part: Lang) -> i32 {
    row_id(a, |k| matches!(k, Kind::Lang(q) if *q == part))
}
/// Focuses editor field `k`, types `value` over what it held, and paints.
pub fn type_into(a: &mut App, text: &mut Text, k: usize, value: &str) {
    let id = part_id(a, Part::Field(k));
    assert!(tap(a, text, id));
    a.editor.text = value.encode_utf16().collect();
    a.editor.anchor = a.editor.text.len();
    a.editor.focus = a.editor.text.len();
    a.sync_field();
    a.scene(text);
}
/// The effect packet `kind` for dictionary `d` with `list`'s rows, as Rust sends it.
pub fn packet(kind: &str, d: &str, list: &[Word]) -> Vec<String> {
    let mut p = vec![kind.to_string(), d.to_string()];
    p.extend(list.iter().flat_map(words::encode));
    p
}
/// The effects that are not toasts or the keyboard's.
pub fn word_effects(a: &App) -> Vec<Vec<String>> {
    a.effects
        .iter()
        .filter(|e| e[0].starts_with("word"))
        .cloned()
        .collect()
}
/// The toasts shown, in order.
pub fn toasts(a: &App) -> Vec<String> {
    a.effects
        .iter()
        .filter(|e| e[0] == wire::settings_ui::files::effect::TOAST)
        .map(|e| e[1].clone())
        .collect()
}
/// The control ids focus visits, from Back round to Back.
pub fn focus_order(a: &mut App) -> Vec<i32> {
    a.focus = -1;
    a.focus_next(false);
    let first = a.focus;
    let mut out = vec![first];
    loop {
        a.focus_next(false);
        if a.focus == first {
            return out;
        }
        out.push(a.focus);
    }
}
