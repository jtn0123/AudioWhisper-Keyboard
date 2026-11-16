//! Group 1 screens: InputFeedback, Smartbar, Gestures, Clipboard and Media, drawn natively.
use render::{
    settings_ui::{
        App,
        catalogue::{self, Kind},
    },
    text::Text,
};
use std::collections::BTreeSet;

const GROUP_ONE: [&str; 5] = [
    "settings/keyboard/input-feedback",
    "settings/smartbar",
    "settings/gestures",
    "settings/clipboard",
    "settings/media",
];

fn app(route: &str) -> (App, Text) {
    let mut a = App {
        config: [480.0, 640.0, 1.0, 1.0],
        ..App::default()
    };
    a.navigate(route);
    (a, Text::new(Vec::<Vec<u8>>::new()))
}
fn at(a: &App, key: &str) -> usize {
    a.rows().iter().position(|r| r.key == key).unwrap()
}
/// Opens `key`'s dialog the way a user reaches it: scrolled into view, then activated.
fn open(a: &mut App, text: &mut Text, key: &str) {
    let i = at(a, key);
    open_row(a, text, i);
}
fn open_row(a: &mut App, text: &mut Text, i: usize) {
    a.focus = 1000 + i as i32;
    a.reveal_focus = true;
    a.scene(text);
    assert!(a.activate(1000 + i as i32), "row {i} not activatable");
    a.scene(text);
}
/// `<string name="…">` and `<plurals name="…">` names in the app's only resource locale,
/// with the build types' `resValue` strings.
fn resources() -> (BTreeSet<String>, BTreeSet<String>) {
    let dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../app/src/main/res/values"
    );
    let mut xml = String::new();
    for file in std::fs::read_dir(dir).unwrap() {
        let path = file.unwrap().path();
        if path.extension().is_some_and(|e| e == "xml") {
            xml += &std::fs::read_to_string(path).unwrap();
        }
    }
    let names = |tag: &str| {
        xml.split(&format!("<{tag} name=\""))
            .skip(1)
            .map(|s| s.split('"').next().unwrap().to_owned())
            .collect::<BTreeSet<_>>()
    };
    // Every build type also defines `resValue` strings (the app's name among them).
    let gradle = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../app/build.gradle.kts"
    );
    let gradle = std::fs::read_to_string(gradle).unwrap();
    let mut strings = names("string");
    strings.extend(
        gradle
            .split("resValue(\"string\", \"")
            .skip(1)
            .map(|s| s.split('"').next().unwrap().to_owned()),
    );
    (strings, names("plurals"))
}

#[test]
fn every_string_key_rust_shows_is_an_android_resource() {
    let (strings, plurals) = resources();
    let keys = App::string_keys();
    for route in GROUP_ONE {
        assert!(keys.contains(&catalogue::screen(route).unwrap().title));
    }
    let mut missing: Vec<String> = keys
        .iter()
        .filter(|k| !strings.contains(**k))
        .map(|k| k.to_string())
        .collect();
    for request in App::plural_requests() {
        let key = request.split('|').next().unwrap().to_owned();
        if !plurals.contains(&key) {
            missing.push(key);
        }
    }
    assert!(missing.is_empty(), "missing resources: {missing:?}");
}

#[test]
fn every_resource_key_the_settings_code_names_is_in_the_key_list() {
    let keys = App::string_keys();
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/settings_ui");
    let prefixes = [
        "action", "settings", "setup", "state", "pref", "unit", "general", "enum",
    ];
    let mut unlisted = vec![];
    for file in std::fs::read_dir(dir).unwrap() {
        let path = file.unwrap().path();
        if path.extension().is_none_or(|e| e != "rs") {
            continue;
        }
        let source = std::fs::read_to_string(&path).unwrap();
        for literal in source.split('"').skip(1).step_by(2) {
            let resource = prefixes
                .iter()
                .any(|p| literal.starts_with(&format!("{p}__")))
                && literal
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
            if resource && !keys.contains(&literal) {
                unlisted.push(literal.to_owned());
            }
        }
    }
    assert!(unlisted.is_empty(), "add to CODE_KEYS: {unlisted:?}");
}

#[test]
fn group_one_routes_are_native_and_other_routes_fall_back_with_the_native_list() {
    for route in GROUP_ONE {
        let (a, _) = app(route);
        assert_eq!(a.stack.last().map(String::as_str), Some(route));
        assert!(a.effects.is_empty(), "{route} fell back");
        assert!(!a.rows().is_empty());
    }
    let (mut a, _) = app("settings/other");
    assert_eq!(a.stack, ["settings/home"]);
    let native = "settings/home|settings/keyboard|settings/keyboard/input-feedback|settings/smartbar|\
                  settings/gestures|settings/clipboard|settings/media|settings/typing|\
                  settings/dictionary|settings/other/physical-keyboard|settings/about|\
                  settings/dictation|settings/about/third-party-licenses";
    assert_eq!(
        a.effects.pop().unwrap(),
        ["compose", "settings/other", native]
    );
}

#[test]
fn each_back_stack_entry_keeps_its_own_scroll() {
    let (mut a, _) = app("settings/keyboard");
    a.max_scroll = 1000.0;
    a.set_scroll(200.0);
    a.navigate("settings/keyboard/input-feedback");
    assert_eq!(a.scroll(), 0.0);
    a.set_scroll(50.0);
    assert_eq!(a.scrolls, [0.0, 200.0, 50.0]);
    a.back();
    assert_eq!(a.scroll(), 200.0);
    a.navigate("settings/home");
    assert_eq!(a.scrolls, [0.0]);
}

#[test]
fn scrolling_stops_being_offered_at_the_whole_pixel_end_the_screen_shows() {
    let (mut a, mut text) = app("settings/clipboard");
    a.scene(&mut text);
    let max = a.max_scroll;
    assert!(max > 1.0, "the clipboard screen scrolls at 480x640");
    assert_eq!(a.can_scroll(), [false, true]);
    // A fling that decays a fraction short of the end shows the end, as ScrollState's rounded
    // value does.
    a.set_scroll((max + 0.5).floor() - 0.4);
    a.scene(&mut text);
    assert_eq!(a.shown_scroll(), (a.max_scroll + 0.5).floor());
    assert_eq!(a.can_scroll(), [true, false]);
    a.set_scroll(max - 1.0);
    assert_eq!(a.can_scroll(), [true, true]);
}
#[test]
fn haptic_sliders_follow_the_vibrator_facts_like_input_feedback_screen() {
    let (mut a, _) = app("settings/keyboard/input-feedback");
    a.strings
        .insert("unit__milliseconds__symbol".into(), "{v} ms".into());
    a.strings
        .insert("unit__percent__symbol".into(), "{v}%".into());
    let duration = at(&a, "input_feedback__haptic_vibration_duration");
    let strength = at(&a, "input_feedback__haptic_vibration_strength");
    a.prefs
        .insert("input_feedback__haptic_enabled".into(), "true".into());
    a.prefs.insert(
        "input_feedback__haptic_vibration_mode".into(),
        "USE_VIBRATOR_DIRECTLY".into(),
    );
    let no_vibrator = "pref__input_feedback__haptic_vibration_strength__summary_no_vibrator";
    let no_amplitude = "pref__input_feedback__haptic_vibration_strength__summary_no_amplitude_ctrl";
    a.vibrator = [false, false];
    assert!(!a.enabled(duration) && !a.enabled(strength));
    assert_eq!(
        [a.summary(duration), a.summary(strength)],
        [no_vibrator, no_vibrator]
    );
    a.vibrator = [true, false];
    assert!(a.enabled(duration) && !a.enabled(strength));
    let ms = format!(
        "{} ms",
        a.value("input_feedback__haptic_vibration_duration")
    );
    assert_eq!(
        [a.summary(duration), a.summary(strength)],
        [ms.as_str(), no_amplitude]
    );
    a.vibrator = [true, true];
    assert!(a.enabled(duration) && a.enabled(strength));
    a.prefs.insert(
        "input_feedback__haptic_vibration_mode".into(),
        "USE_HAPTIC_FEEDBACK_INTERFACE".into(),
    );
    assert!(!a.enabled(duration) && !a.enabled(strength));
    a.prefs.insert(
        "input_feedback__haptic_vibration_mode".into(),
        "USE_VIBRATOR_DIRECTLY".into(),
    );
    a.prefs
        .insert("input_feedback__haptic_enabled".into(), "false".into());
    assert!(!a.enabled(duration) && !a.enabled(strength));
    // The linked list summarises the switch, not the mode, while the switch is off.
    let mode = at(&a, "input_feedback__haptic_activation_mode");
    assert_eq!(
        a.summary(mode),
        "pref__input_feedback__haptic_enabled__summary_disabled"
    );
}

#[test]
fn haptic_sliders_preview_a_vibration_with_the_other_stored_value() {
    let (mut a, mut text) = app("settings/keyboard/input-feedback");
    a.vibrator = [true, true];
    a.prefs
        .insert("input_feedback__haptic_enabled".into(), "true".into());
    a.prefs.insert(
        "input_feedback__haptic_vibration_mode".into(),
        "USE_VIBRATOR_DIRECTLY".into(),
    );
    a.prefs.insert(
        "input_feedback__haptic_vibration_strength".into(),
        "61".into(),
    );
    a.prefs.insert(
        "input_feedback__haptic_vibration_duration".into(),
        "23".into(),
    );
    open(
        &mut a,
        &mut text,
        "input_feedback__haptic_vibration_duration",
    );
    assert!(a.set_slider(0, 37.0));
    a.slider_finished(0);
    assert_eq!(a.effects.pop().unwrap(), ["vibrate", "37", "61"]);
    a.scene(&mut text);
    assert!(a.adjust(4000, 1.0));
    assert_eq!(a.effects.pop().unwrap(), ["vibrate", "38", "61"]);
    assert!(a.effects.is_empty(), "previewing wrote a preference");
    a.back();
    open(
        &mut a,
        &mut text,
        "input_feedback__haptic_vibration_strength",
    );
    assert!(a.set_slider(0, 80.0));
    a.slider_finished(0);
    assert_eq!(a.effects.pop().unwrap(), ["vibrate", "23", "80"]);
    a.back();
    // Sliders without onPreviewSelectedValue preview nothing.
    a.prefs
        .insert("input_feedback__audio_enabled".into(), "true".into());
    open(&mut a, &mut text, "input_feedback__audio_volume");
    assert!(a.adjust(4000, 1.0));
    assert!(a.effects.is_empty());
}

#[test]
fn emoji_history_resets_ask_yes_or_no_before_deleting() {
    let (mut a, mut text) = app("settings/media");
    for (key, value) in [
        ("action__yes", "Yes"),
        ("action__no", "No"),
        ("action__reset_confirm_title", "Confirm reset"),
        (
            "action__reset_confirm_message",
            "Are you sure you want to reset \"{name}\"?",
        ),
        (
            "prefs__media__emoji_history_pinned_reset__name",
            "pinned emojis",
        ),
    ] {
        a.strings.insert(key.into(), value.into());
    }
    let pinned = a
        .rows()
        .iter()
        .position(|r| matches!(r.kind, Kind::Confirm(ask) if ask.effect == ["deleteEmojiHistory", "pinned"]))
        .unwrap();
    open_row(&mut a, &mut text, pinned);
    let ids = a
        .controls
        .iter()
        .map(|c| (c.id, c.label.clone()))
        .collect::<Vec<_>>();
    assert!(
        ids.contains(&(6, "No".into())) && ids.contains(&(7, "Yes".into())),
        "{ids:?}"
    );
    assert!(ids.iter().all(|(id, _)| [4, 6, 7].contains(id)), "{ids:?}");
    let mut order = vec![];
    for _ in 0..3 {
        a.focus_next(false);
        order.push(a.focus);
    }
    assert_eq!(order, [6, 7, 6]);
    assert!(a.activate(6));
    assert!(a.modal.is_none() && a.effects.is_empty());
    open_row(&mut a, &mut text, pinned);
    assert!(a.activate(7));
    assert_eq!(a.effects, [["deleteEmojiHistory", "pinned"]]);
    let recent = a
        .rows()
        .iter()
        .position(|r| matches!(r.kind, Kind::Confirm(ask) if ask.effect == ["deleteEmojiHistory", "recent"]))
        .unwrap();
    a.effects.clear();
    open_row(&mut a, &mut text, recent);
    assert!(a.activate(7));
    assert_eq!(a.effects, [["deleteEmojiHistory", "recent"]]);
}

#[test]
fn smartbar_fixed_row_shows_only_for_the_shared_layout_drawn_disabled_and_checked_as_plain_text() {
    let (mut a, mut text) = app("settings/smartbar");
    let fixed = a
        .rows()
        .iter()
        .position(|r| matches!(r.kind, Kind::Fixed))
        .unwrap();
    a.prefs.insert("smartbar__enabled".into(), "true".into());
    for (layout, shown) in [
        ("SUGGESTIONS_ACTIONS_SHARED", true),
        ("SUGGESTIONS_ACTIONS_EXTENDED", false),
        ("SUGGESTIONS_ONLY", false),
        ("ACTIONS_ONLY", false),
    ] {
        a.prefs.insert("smartbar__layout".into(), layout.into());
        assert_eq!(a.visible(fixed), shown, "{layout}");
        assert!(!a.enabled(fixed));
    }
    a.prefs.insert(
        "smartbar__layout".into(),
        "SUGGESTIONS_ACTIONS_SHARED".into(),
    );
    a.focus = 1000 + fixed as i32;
    a.reveal_focus = true;
    a.scene(&mut text);
    let c = a
        .controls
        .iter()
        .find(|c| c.id == 1000 + fixed as i32)
        .unwrap();
    assert!(!c.enabled);
    // Drawn checked, but a Switch with `onCheckedChange = null` adds no toggleable semantics.
    assert_eq!(c.checked, None);
    // A Preference without onClick: Compose's ListItem node is enabled text with no role.
    assert_eq!(c.role, "text");
    assert!(c.exposed_enabled());
    assert!(a.controls.iter().all(|c| c.id != 2000 + fixed as i32));
    assert!(!a.activate(c.id));
}

#[test]
fn value_labels_show_zero_labels_resolved_plurals_and_the_bare_number_otherwise() {
    let (mut a, _) = app("settings/clipboard");
    a.strings.insert("general__auto".into(), "Auto".into());
    a.strings
        .insert("unit__items__written#7".into(), "7 items".into());
    let columns = a
        .rows()
        .iter()
        .find_map(|r| match r.kind {
            Kind::Slider(s) if !s.unit.zero.is_empty() => Some(s),
            _ => None,
        })
        .unwrap();
    assert_eq!(a.value_label(&columns.unit, "0"), "Auto");
    assert_eq!(a.value_label(&columns.unit, "4"), "4");
    let items = catalogue::Unit {
        key: "unit__items__written",
        plural: true,
        zero: "",
    };
    assert_eq!(a.value_label(&items, "7"), "7 items");
    assert_eq!(a.value_label(&items, "8"), "8");
    let requests = App::plural_requests();
    for unit in [
        "unit__minutes__written",
        "unit__seconds__written",
        "unit__items__written",
        "unit__characters__written",
        "unit__candidates__written",
    ] {
        assert!(
            requests.iter().any(|r| r.starts_with(&format!("{unit}|"))),
            "{unit}"
        );
    }
    assert_eq!(
        requests.len(),
        requests.iter().collect::<BTreeSet<_>>().len()
    );
    assert!(requests.iter().all(|r| r.split('|').count() == 3));
}

#[test]
fn skin_tone_entries_substitute_their_emoji_and_unknown_values_show_invalid() {
    let (mut a, _) = app("settings/media");
    a.strings.insert(
        "enum__emoji_skin_tone__medium_skin_tone".into(),
        "{emoji} Medium".into(),
    );
    let tone = at(&a, "emoji__preferred_skin_tone");
    assert_eq!(
        a.choice_label(tone, "MEDIUM_SKIN_TONE"),
        "👋\u{1f3fd} Medium"
    );
    assert_eq!(a.choice_label(tone, "NO_SUCH_TONE"), "!! invalid !!");
}
