//! What each smartbar action shows (`render::smartbar_actions`): the arrow, the toggle's icon,
//! each action's flags, icon and label, and the packed call Android makes.

use keys::codes::{READ_ALOUD, VOICE_INPUT};
use render::protocol::*;
use render::smartbar_actions::{Action, Bar, KeyAnswer, plan, text_label};
use wire::smartbar::{action, bar, icon, kind, plan as packed, shown};

const SHARED_LAYOUT: i32 = 2;
const EXTENDED_LAYOUT: i32 = 3;
const REPLACES_TOGGLE: i32 = 0;
const BEHIND_KEYS: i32 = 1;

fn empty() -> Action<'static> {
    Action {
        kind: kind::NONE,
        code: 0,
        enabled: false,
        key_icon: false,
        text: &[],
    }
}

fn key(code: i32) -> Action<'static> {
    Action {
        kind: kind::KEY,
        code,
        enabled: true,
        key_icon: true,
        text: &[],
    }
}

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

fn listening() -> Bar {
    Bar {
        layout: SHARED_LAYOUT,
        listening: true,
        ..Bar::default()
    }
}

#[test]
fn the_arrow_points_left_when_flipped_or_expanded_behind_the_keys_but_not_both() {
    for flip in [false, true] {
        for shared in [false, true] {
            for display in [REPLACES_TOGGLE, BEHIND_KEYS] {
                let bar = Bar {
                    flip,
                    shared,
                    incognito_display: display,
                    ..listening()
                };
                let left = flip ^ (shared && display == BEHIND_KEYS);
                let want = if left {
                    icon::ARROW_LEFT
                } else {
                    icon::ARROW_RIGHT
                };
                assert_eq!(bar.arrow(), want, "flip {flip} shared {shared} {display}");
                assert_eq!(bar.toggle(), want);
            }
        }
    }
}

#[test]
fn the_extended_layout_reads_its_own_expansion_and_the_toggle_unfolds_it() {
    let folded = Bar {
        layout: EXTENDED_LAYOUT,
        shared: true,
        incognito: true,
        ..listening()
    };
    assert!(!folded.expanded());
    assert_eq!(folded.toggle(), icon::UNFOLD_MORE);
    let unfolded = Bar {
        extended: true,
        ..folded
    };
    assert!(unfolded.expanded());
    assert_eq!(unfolded.toggle(), icon::UNFOLD_LESS);
    let answer = plan(&unfolded, &[]);
    assert!(answer.expanded && answer.toggles_extended);
    let shared = plan(
        &Bar {
            extended: true,
            ..listening()
        },
        &[],
    );
    assert!(!shared.expanded && !shared.toggles_extended);
}

#[test]
fn incognito_takes_the_arrows_place_only_when_set_to() {
    let replaces = Bar {
        incognito: true,
        ..listening()
    };
    assert_eq!(replaces.toggle(), icon::INCOGNITO);
    let behind = Bar {
        incognito_display: BEHIND_KEYS,
        ..replaces
    };
    assert_eq!(behind.toggle(), icon::ARROW_RIGHT);
}

#[test]
fn an_empty_slot_shows_the_toggle_reports_no_code_and_is_always_enabled() {
    let bar = Bar {
        incognito: true,
        ..listening()
    };
    let slot = bar.shown(&Action {
        code: VOICE_INPUT,
        ..empty()
    });
    assert_eq!(
        (slot.code, slot.flags, slot.icon),
        (0, BAR_ENABLED, icon::INCOGNITO)
    );
    assert!(slot.label.is_empty() && !slot.key_label);
}

#[test]
fn a_key_is_enabled_as_its_evaluator_says() {
    let bar = listening();
    let off = bar.shown(&Action {
        enabled: false,
        ..key(-10)
    });
    assert_eq!((off.code, off.flags), (-10, 0));
    assert_eq!(bar.shown(&key(-10)).flags, BAR_ENABLED);
}

#[test]
fn the_microphone_lights_while_dictating_with_its_level_and_whether_it_warms_or_finishes() {
    let mic = key(VOICE_INPUT);
    let idle = Bar {
        listening: false,
        ..listening()
    };
    assert_eq!(idle.shown(&mic).flags, BAR_ENABLED | BAR_FINISHING);
    let warming = Bar {
        dictating: true,
        warming: true,
        level: 0.5,
        ..listening()
    };
    assert_eq!(
        warming.shown(&mic).flags,
        BAR_ENABLED | BAR_DICTATING | BAR_WARMING | (127 << BAR_LEVEL_SHIFT)
    );
    for (level, packed) in [
        (-1.0, 0),
        (0.0, 0),
        (1.0, 255),
        (7.0, 255),
        (f32::NAN, 0),
        (f32::INFINITY, 255),
    ] {
        let bar = Bar {
            level,
            ..listening()
        };
        assert_eq!(bar.shown(&mic).flags >> BAR_LEVEL_SHIFT, packed, "{level}");
    }
    let other = Bar {
        dictating: true,
        level: 1.0,
        ..listening()
    };
    assert_eq!(other.shown(&key(-10)).flags, BAR_ENABLED);
}

#[test]
fn reading_aloud_lights_the_speaker_and_shows_stop_in_place_of_its_icon_and_label() {
    let reading = Bar {
        reading: true,
        ..listening()
    };
    let speaker = Action {
        key_icon: false,
        ..key(READ_ALOUD)
    };
    let shown = reading.shown(&speaker);
    assert_eq!(
        (shown.flags, shown.icon),
        (BAR_ENABLED | BAR_READING, icon::STOP)
    );
    assert!(!shown.key_label);
    let quiet = listening().shown(&speaker);
    assert_eq!((quiet.flags, quiet.icon), (BAR_ENABLED, icon::NONE));
    assert!(quiet.key_label);
}

#[test]
fn a_key_shows_its_own_icon_or_else_its_own_label() {
    let bar = listening();
    let drawn = bar.shown(&key(-10));
    assert_eq!(drawn.icon, icon::KEY);
    assert!(!drawn.key_label && drawn.label.is_empty());
    let named = bar.shown(&Action {
        key_icon: false,
        ..key(-10)
    });
    assert_eq!(named.icon, icon::NONE);
    assert!(named.key_label && named.label.is_empty());
}

#[test]
fn a_text_action_shows_its_first_character_or_a_question_mark() {
    for (text, label) in [
        ("abc", "a"),
        ("é!", "é"),
        ("😀x", "😀"),
        ("", "?"),
        (" x", "?"),
        ("\t", "?"),
        ("\u{a0}", "?"),
        ("\u{1c}", "?"),
        ("\u{2028}", "?"),
        ("\u{85}", "\u{85}"),
    ] {
        assert_eq!(text_label(&units(text)), units(label), "{text:?}");
    }
    // A surrogate without its pair stays as it is: there is nothing whole to show.
    assert_eq!(text_label(&[0xD83D, 0x78]), [0xD83D]);
    assert_eq!(text_label(&[0xDE00, 0xD83D]), [0xDE00]);
    let bar = listening();
    let text = units("hi");
    let shown = bar.shown(&Action {
        kind: kind::TEXT,
        code: 0,
        enabled: true,
        key_icon: true,
        text: &text,
    });
    assert_eq!((shown.icon, shown.label), (icon::NONE, units("h")));
    assert!(!shown.key_label);
}

/// A packed smartbar config, as Kotlin's `smartbarConfig` packs it, for `layout`, the shared
/// actions' setting and whether it is a preview, with dictation running.
fn config(layout: f32, shared: bool, preview: bool) -> [f32; 17] {
    let bit = |on: bool| if on { 1.0 } else { 0.0 };
    [
        1.0,
        layout,
        0.0,
        bit(shared),
        0.0,
        0.0,
        1.0,
        1.0,
        3.0,
        0.0,
        2.75,
        132.0,
        0.0,
        bit(preview),
        1.0,
        1.0,
        0.0,
    ]
}

fn bar_row(level: &str) -> Vec<Vec<u16>> {
    let mut row = vec![units("0"); bar::SIZE];
    row[bar::LEVEL] = units(level);
    row[bar::LISTENING] = units("1");
    row
}

fn action_row(kind: i32, code: i32, text: &str) -> Vec<Vec<u16>> {
    let mut row = vec![units("0"); action::SIZE];
    row[action::KIND] = units(&kind.to_string());
    row[action::CODE] = units(&code.to_string());
    row[action::LABEL] = units("key");
    row[action::TEXT] = units(text);
    row
}

/// The packed call, with every key enabled and labelled by its label, and drawing an icon only
/// for code 1.
fn call(config: &[f32], fields: &[Vec<u16>]) -> Option<Vec<Vec<u16>>> {
    let mut key = |_kind, code, label: &[u16]| KeyAnswer {
        enabled: true,
        icon: if code == 1 { 1003 } else { 0 },
        label: Some(label.to_vec()),
    };
    render::smartbar_actions::call(config, fields, &mut key)
}

fn flags(answer: &[Vec<u16>], action: usize) -> i32 {
    let flags = &answer[packed::SIZE + action * shown::SIZE + shown::FLAGS];
    String::from_utf16(flags).unwrap().parse().unwrap()
}

#[test]
fn the_packed_call_answers_a_row_for_each_action() {
    let mut fields = bar_row("1.0E-5");
    fields.extend(action_row(kind::NONE, 0, ""));
    fields.extend(action_row(kind::KEY, VOICE_INPUT, ""));
    fields.extend(action_row(kind::TEXT, 0, "ok"));
    let answer = call(&config(3.0, true, false), &fields).expect("a whole packet");
    assert_eq!(answer.len(), packed::SIZE + 3 * shown::SIZE);
    assert_eq!(answer[packed::EXPANDED], units("0"));
    assert_eq!(answer[packed::TOGGLES_EXTENDED], units("1"));
    let row = |i: usize, field: usize| &answer[packed::SIZE + i * shown::SIZE + field];
    assert_eq!(row(0, shown::ICON), &units(&icon::UNFOLD_MORE.to_string()));
    assert_eq!(row(1, shown::CODE), &units(&VOICE_INPUT.to_string()));
    // The config says dictation is running; the row, that it is listening.
    assert_eq!(flags(&answer, 1), BAR_ENABLED | BAR_DICTATING);
    // Its key draws no icon, so it shows its key's label.
    assert_eq!(row(1, shown::LABEL), &units("key"));
    assert_eq!(row(1, shown::KEY_ICON), &units("0"));
    assert_eq!(row(2, shown::LABEL), &units("o"));
}

#[test]
fn an_empty_slot_has_no_key_to_ask_about() {
    let mut fields = bar_row("0");
    fields.extend(action_row(kind::NONE, 7, ""));
    fields.extend(action_row(kind::KEY, VOICE_INPUT, ""));
    fields.extend(action_row(kind::TEXT, 0, "ok"));
    let mut asked = Vec::new();
    let mut key = |_kind, code, _label: &[u16]| {
        asked.push(code);
        KeyAnswer::default()
    };
    let config = config(0.0, false, false);
    let answer = render::smartbar_actions::call(&config, &fields, &mut key);
    assert!(answer.is_some());
    // The key action's key, and the text action's, which has none of its own (`UNSPECIFIED`).
    assert_eq!(asked, [VOICE_INPUT, 0]);
}

#[test]
fn a_key_that_draws_an_icon_names_it_and_shows_no_label() {
    let mut fields = bar_row("0");
    fields.extend(action_row(kind::KEY, 1, ""));
    fields.extend(action_row(kind::TEXT, 1, "ok"));
    let answer = call(&config(0.0, false, false), &fields).expect("a whole packet");
    let row = |i: usize, field: usize| &answer[packed::SIZE + i * shown::SIZE + field];
    assert_eq!(row(0, shown::ICON), &units(&icon::KEY.to_string()));
    assert_eq!(row(0, shown::KEY_ICON), &units("1003"));
    assert!(row(0, shown::LABEL).is_empty());
    // A text action never shows its key's icon.
    assert_eq!(row(1, shown::KEY_ICON), &units("0"));
    assert_eq!(row(1, shown::LABEL), &units("o"));
}

#[test]
fn the_arrow_follows_the_shared_setting_even_in_a_preview_that_folds_them() {
    for preview in [false, true] {
        let answer = call(&config(2.0, true, preview), &bar_row("0")).expect("a whole packet");
        assert_eq!(answer[packed::EXPANDED], units("1"), "preview {preview}");
    }
}

#[test]
fn the_packed_call_reads_every_spelling_of_kotlins_floats() {
    for (level, packed) in [
        ("NaN", 0),
        ("Infinity", 255),
        ("-Infinity", 0),
        ("0.5", 127),
    ] {
        let mut fields = bar_row(level);
        fields.extend(action_row(kind::KEY, VOICE_INPUT, ""));
        let answer = call(&config(0.0, false, false), &fields).expect("a whole packet");
        assert_eq!(flags(&answer, 0) >> BAR_LEVEL_SHIFT, packed, "{level}");
    }
}

#[test]
fn the_packed_call_refuses_a_bad_config_a_short_row_or_a_number_it_cannot_read() {
    let config = config(0.0, false, false);
    assert_eq!(call(&config, &bar_row("0")[..bar::SIZE - 1]), None);
    assert!(call(&config, &bar_row("0")).is_some_and(|a| a.len() == packed::SIZE));
    assert_eq!(call(&config[..16], &bar_row("0")), None);
    let mut cut = bar_row("0");
    cut.extend(action_row(kind::KEY, 1, "")[..action::SIZE - 1].to_vec());
    assert_eq!(call(&config, &cut), None);
    assert_eq!(call(&config, &bar_row("loud")), None);
    let mut display = bar_row("0");
    display[bar::INCOGNITO_DISPLAY] = units("behind");
    assert_eq!(call(&config, &display), None);
    let mut bad = bar_row("0");
    bad.extend(action_row(kind::KEY, 1, ""));
    bad[bar::SIZE + action::CODE] = units("one");
    assert_eq!(call(&config, &bad), None);
}
