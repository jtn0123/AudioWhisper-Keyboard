//! Group 2 (stage4-followup4.md, part 3): Typing, User dictionaries, Physical keyboard, About and
//! Dictation, drawn natively. These check the new pieces: the spell checker cards and the facts
//! they read, a confirmation with its own message and buttons, the plans links, intents and the
//! version copy return, AboutScreen's header, ModelStatus, and the vocabulary field that shares the
//! preview field's editor. SettingsGroupTwoMatchTest holds every row against the Compose sources.
use render::{
    settings_ui::{App, SpellCard, catalogue},
    text::Text,
};

fn app(route: &str) -> (App, Text) {
    let mut a = App {
        config: [480.0, 4000.0, 1.0, 1.0],
        ..App::default()
    };
    a.navigate(route);
    let mut text = Text::new(Vec::<Vec<u8>>::new());
    a.scene(&mut text);
    (a, text)
}
fn id(a: &App, key: &str) -> i32 {
    1000 + a.rows().iter().position(|r| r.key == key).unwrap() as i32
}
fn set(map: &mut std::collections::BTreeMap<String, String>, pairs: &[(&str, &str)]) {
    map.extend(pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())));
}
fn shown(a: &App, id: i32) -> bool {
    a.controls.iter().any(|c| c.id == id)
}
fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

#[test]
fn group_two_routes_are_native_and_other_and_theme_are_not_yet() {
    for route in [
        "settings/typing",
        "settings/dictionary",
        "settings/other/physical-keyboard",
        "settings/about",
        "settings/dictation",
    ] {
        assert!(catalogue::screen(route).is_some(), "{route}");
    }
    for route in ["settings/other", "settings/theme"] {
        assert!(catalogue::screen(route).is_none(), "{route} is deferred");
    }
}

#[test]
fn the_spell_checker_card_follows_the_system_settings() {
    let (mut a, mut text) = app("settings/typing");
    let card = a
        .rows()
        .iter()
        .position(|r| r.key == "pref__spelling__title#selector")
        .unwrap();
    let languages = id(&a, "spelling__language_mode");
    let enabled = |a: &App| {
        a.controls
            .iter()
            .find(|c| c.id == languages)
            .unwrap()
            .enabled
    };
    assert_eq!(a.spell_card(), SpellCard::Disabled);
    set(&mut a.facts, &[("spell_enabled", "1"), ("package", "me")]);
    assert_eq!(a.spell_card(), SpellCard::None);
    set(
        &mut a.facts,
        &[("spell_id", "other/.S"), ("spell_package", "other")],
    );
    assert_eq!(
        a.spell_card(),
        SpellCard::App(
            "pref__spelling__active_spellchecker__unknown".into(),
            "other".into()
        )
    );
    a.scene(&mut text);
    assert!(
        !enabled(&a),
        "another app's spell checker leaves Languages disabled"
    );
    set(
        &mut a.facts,
        &[("spell_label", "Spell App"), ("spell_package", "me")],
    );
    assert_eq!(
        a.spell_card(),
        SpellCard::App("Spell App".into(), "me".into())
    );
    a.scene(&mut text);
    assert!(enabled(&a));
    let c = a
        .controls
        .iter()
        .find(|c| c.id == 1000 + card as i32)
        .unwrap();
    assert_eq!(
        (c.role, c.label.as_str(), c.summary.as_str()),
        ("card", "Spell App", "me")
    );
    assert!(a.activate(1000 + card as i32));
    assert_eq!(a.effects, [["intent", "spellCheckerSettings"]]);
}

#[test]
fn forgetting_learned_words_asks_with_its_own_message_and_buttons() {
    let (mut a, mut text) = app("settings/typing");
    set(
        &mut a.strings,
        &[(
            "pref__suggestion__forget_learned__done",
            "Learned words erased",
        )],
    );
    let forget = id(&a, "pref__suggestion__forget_learned__label");
    for confirm in [false, true] {
        assert!(a.activate(forget));
        let panel = a.scene(&mut text).panel.unwrap();
        assert!(
            panel
                .items
                .iter()
                .any(|i| i.label == "pref__suggestion__forget_learned__confirm")
        );
        let label = |id: i32| {
            a.controls
                .iter()
                .find(|c| c.id == id)
                .map(|c| c.label.clone())
        };
        assert_eq!(label(5), None, "no neutral button");
        assert_eq!(label(6).as_deref(), Some("action__cancel"));
        assert_eq!(label(7).as_deref(), Some("action__delete"));
        assert!(a.activate(if confirm { 7 } else { 6 }));
        a.scene(&mut text);
        assert!(a.modal.is_none());
        let expected: &[&[&str]] = if confirm {
            &[&["forgetLearnedWords", "Learned words erased"]]
        } else {
            &[]
        };
        assert_eq!(a.effects, expected);
        a.effects.clear();
    }
}

#[test]
fn the_auto_space_card_shows_with_its_switch_and_opens_its_thread() {
    let (mut a, _) = app("settings/typing");
    // A real font: a link that measures zero pixels wide is no control at all.
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    let url = "https://example.org/discussions/1935";
    set(
        &mut a.strings,
        &[("florisboard__auto_space_feedback_url", url)],
    );
    let card = id(&a, "pref__correction__auto_space_punctuation__feedback");
    let link = card + 5000;
    a.prefs
        .insert("correction__auto_space_punctuation".into(), "false".into());
    a.scene(&mut text);
    assert!(!shown(&a, card) && !shown(&a, link));
    a.prefs
        .insert("correction__auto_space_punctuation".into(), "true".into());
    a.scene(&mut text);
    assert!(shown(&a, card));
    assert_eq!(
        a.controls.iter().find(|c| c.id == link).unwrap().role,
        "link"
    );
    assert!(a.activate(link));
    assert_eq!(a.effects, [["url", url]]);
}

#[test]
fn about_copies_the_version_fills_links_and_draws_its_header() {
    let (mut a, mut text) = app("settings/about");
    set(
        &mut a.facts,
        &[("version_name", "0.5.0-alpha01"), ("version_code", "101")],
    );
    set(
        &mut a.strings,
        &[
            (
                "florisboard__changelog_url",
                "https://example.org/releases/v{version}",
            ),
            (
                "about__project_license__summary",
                "Licensed under {license_name}",
            ),
        ],
    );
    a.scene(&mut text);
    let version = (id(&a, "about__version__title") - 1000) as usize;
    let license = (id(&a, "about__project_license__title") - 1000) as usize;
    assert_eq!(a.summary(version), "0.5.0-alpha01 (101)");
    assert_eq!(a.summary(license), "Licensed under Apache 2.0");
    assert!(a.activate(1000 + version as i32));
    assert!(a.activate(id(&a, "about__changelog__title")));
    assert_eq!(
        a.effects,
        [
            vec![
                "copy",
                "0.5.0-alpha01 (101)",
                "about__version_copied__title",
                "about__version_copied__error"
            ],
            vec!["url", "https://example.org/releases/v0.5.0-alpha01"],
        ]
    );
    // AboutScreen's 64 dp icon, centred, 24 dp below the app bar; no preview field.
    let icon = a.controls.iter().find(|c| c.id == 6000).unwrap();
    assert_eq!(icon.role, "image");
    assert_eq!(
        (icon.bounds.width(), icon.bounds.height(), icon.bounds.left),
        (64.0, 64.0, 208.0)
    );
    assert!(!shown(&a, 2));
}

#[test]
fn physical_keyboard_rows_follow_the_attached_keyboard() {
    let (mut a, mut text) = app("settings/other/physical-keyboard");
    let open = id(&a, "physical_keyboard__system_settings__title");
    let info = id(&a, "physical_keyboard__system_settings__not_attached");
    for attached in [true, false] {
        set(
            &mut a.facts,
            &[("physical_keyboard", if attached { "true" } else { "false" })],
        );
        a.scene(&mut text);
        assert_eq!((shown(&a, open), shown(&a, info)), (attached, !attached));
    }
    assert!(!a.activate(info), "the not-attached row has no onClick");
    set(&mut a.facts, &[("physical_keyboard", "true")]);
    a.scene(&mut text);
    assert!(a.activate(open));
    assert_eq!(a.effects, [["intent", "hardKeyboardSettings"]]);
}

#[test]
fn model_status_asks_for_the_files_and_reads_their_facts() {
    let (mut a, _) = app("settings/typing");
    assert!(a.effects.is_empty());
    a.navigate("settings/dictation");
    assert_eq!(a.effects, [["checkModels"]]);
    let speech = (id(&a, "dictation__models__speech") - 1000) as usize;
    let voice = (id(&a, "dictation__models__voice") - 1000) as usize;
    assert_eq!(a.summary(speech), "", "nothing before Kotlin reports");
    set(
        &mut a.strings,
        &[("dictation__models__voice_damaged", "Damaged (%1$s).")],
    );
    set(
        &mut a.facts,
        &[
            ("speech_state", "ready"),
            ("voice_state", "damaged"),
            ("voice_files", "a.onnx, b.txt"),
        ],
    );
    assert_eq!(a.summary(speech), "dictation__models__checking");
    assert_eq!(a.summary(voice), "Damaged (a.onnx, b.txt).");
    set(
        &mut a.facts,
        &[("models_checked", "true"), ("speech_state", "missing")],
    );
    assert_eq!(a.summary(speech), "dictation__models__speech_missing");
    set(&mut a.facts, &[("speech_state", "ready")]);
    assert_eq!(a.summary(speech), "dictation__models__speech_ready");
}

#[test]
fn the_vocabulary_field_owns_the_editor_and_saves_each_change() {
    let (mut a, mut text) = app("settings/home");
    a.prefs
        .insert("dictation__vocabulary".into(), "Kubernetes".into());
    a.editor.text = utf16("preview");
    a.navigate("settings/dictation");
    a.scene(&mut text);
    a.effects.clear();
    let field = id(&a, "dictation__vocabulary");
    let (shown_editor, active) = a.field_editor(field);
    assert_eq!((shown_editor.text, active), (utf16("Kubernetes"), false));
    assert!(a.activate(field));
    assert_eq!(a.effects, [vec!["restartInput"], vec!["editor"]]);
    assert_eq!((a.field_id(), a.editor.focus), (field, 10));
    assert_eq!(a.preview_editor().text, utf16("preview"));
    a.effects.clear();
    assert!(a.editor.replace(&utf16("\nPostgres"), 1, false));
    a.sync_field();
    a.sync_field();
    assert_eq!(
        a.effects,
        [["write", "dictation__vocabulary", "Kubernetes\nPostgres"]]
    );
    // A lone surrogate is saved as U+FFFD (a documented deviation: Kotlin strings keep it).
    a.effects.clear();
    assert!(a.editor.replace(&[0xd83d], 1, false));
    a.sync_field();
    assert_eq!(
        a.prefs["dictation__vocabulary"],
        "Kubernetes\nPostgres\u{fffd}"
    );
    assert!(a.focus_field(2));
    assert_eq!((a.field_id(), a.editor.text.clone()), (2, utf16("preview")));
    assert!(!a.focus_field(1000), "a heading is not a field");
    assert!(a.focus_field(field));
    a.back();
    assert_eq!((a.field, a.editor.text.clone()), (None, utf16("preview")));
    assert!(!a.focus_field(field), "home has no vocabulary field");
}

#[test]
fn the_vocabulary_field_minimum_includes_its_padding() {
    // `heightIn(min = 120.dp)` before `padding(16.dp, 8.dp)`: the field is 120 - 16 dp at least.
    for (density, height, inset) in [(1.0, 104.0, 16.0), (2.625, 273.0, 42.0)] {
        let (mut a, mut text) = app("settings/dictation");
        a.config = [1080.0, 6000.0, density, 1.0];
        a.scene(&mut text);
        let field = id(&a, "dictation__vocabulary");
        let c = a.controls.iter().find(|c| c.id == field).unwrap();
        assert_eq!(
            (c.role, c.bounds.height(), c.bounds.left),
            ("edit", height, inset)
        );
        assert_eq!(c.bounds.width(), 1080.0 - 2.0 * inset);
    }
}

#[test]
fn cards_take_the_content_colour_and_tracking_compose_resolves() {
    // The auto-space Card's text is `contentColorFor(surfaceContainerHighest)`, palette slot 13
    // (onSurfaceVariant under materialkolor's scheme, not onSurface). FlorisErrorCard's text is
    // titleMedium, whose tracking the resolved Material library has at 0.2 sp.
    let (mut a, _) = app("settings/typing");
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.config = [1080.0, 6000.0, 2.625, 1.0];
    a.colors[13] = 0xff474648;
    a.prefs
        .insert("correction__auto_space_punctuation".into(), "true".into());
    let panel = a.scene(&mut text).panel.unwrap();
    let style = |label: &str| {
        let item = panel.items.iter().find(|i| i.label == label).unwrap();
        (item.text.foreground, item.text.letter_spacing)
    };
    let body = style("pref__correction__auto_space_punctuation__feedback");
    assert_eq!(body, (0xff474648, 0.0));
    let (foreground, tracking) = style("pref__spelling__active_spellchecker__summary_disabled");
    assert_eq!(foreground, 0xffffffff);
    assert!((tracking - 0.2 * 2.625).abs() < 1e-4, "{tracking}");
    // The error icon's 12 dp padding and 24 dp size round separately (32 + 63 + 32), so the text
    // starts 127 px into the card, not 48 dp's 126.
    let card = a.controls.iter().find(|c| c.role == "card").unwrap().bounds;
    let label = panel
        .items
        .iter()
        .find(|i| i.label == "pref__spelling__active_spellchecker__summary_disabled")
        .unwrap()
        .bounds;
    assert_eq!(label.left - card.left, 127.0);
}
