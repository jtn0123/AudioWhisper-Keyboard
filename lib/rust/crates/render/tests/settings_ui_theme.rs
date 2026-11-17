//! Theme (stage4-other-theme.md): the rows the theme mode enables, the day and night themes named
//! from the extension index Kotlin reports, the links into the Compose theme manager and the
//! extension list, and the sunrise and sunset dialog driven by finger and by keyboard.
use render::{
    settings_ui::{App, time},
    text::Text,
    touch::Motion,
};

const DAY: &str = "org.florisboard.themes:floris_day";
const NIGHT: &str = "org.florisboard.themes:floris_night";

fn app(prefs: &[(&str, &str)], facts: &[(&str, &str)]) -> (App, Text) {
    let mut a = App {
        config: [1080.0, 2400.0, 2.625, 1.0],
        ..App::default()
    };
    for (k, v) in prefs {
        a.prefs.insert((*k).into(), (*v).into());
    }
    for (k, v) in facts {
        a.facts.insert((*k).into(), (*v).into());
    }
    a.navigate("settings/theme");
    let mut text = Text::new(vec![
        include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec(),
    ]);
    a.scene(&mut text);
    (a, text)
}
fn index(a: &App, key: &str) -> usize {
    a.rows().iter().position(|r| r.key == key).unwrap()
}
fn id(a: &App, key: &str) -> i32 {
    1000 + index(a, key) as i32
}
fn control(a: &App, id: i32) -> render::settings_ui::Control {
    a.controls.iter().find(|c| c.id == id).cloned().unwrap()
}
fn touch(a: &mut App, action: i32, (x, y): (f32, f32), t: i64) {
    a.motion(&Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, t],
        coords: vec![x, y],
    });
}
/// Paints, then taps control `id` as it now stands.
fn act(a: &mut App, text: &mut Text, id: i32) -> bool {
    a.scene(text);
    a.activate(id)
}
fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}
/// The point on the dial at `twelfths` of a turn clockwise from 12, `ring` 256ths of the dial's
/// width from its centre (101 the outer ring, 69 the inner).
fn on_dial(a: &App, twelfths: f32, ring: f32) -> (f32, f32) {
    let b = control(a, 7132).bounds;
    let half = (b.width() / 2.0).floor();
    let r = b.width() * ring / 256.0;
    let angle = twelfths / 12.0 * std::f32::consts::TAU;
    (
        b.left + half + r * angle.sin(),
        b.top + half - r * angle.cos(),
    )
}
fn tap_dial(a: &mut App, text: &mut Text, twelfths: f32, ring: f32) {
    a.scene(text);
    let at = on_dial(a, twelfths, ring);
    touch(a, 0, at, 1);
    touch(a, 1, at, 2);
}
/// Selects all of the field holding the keyboard and types `s` over it.
fn type_over(a: &mut App, s: &str) {
    a.editor.anchor = 0;
    a.editor.focus = a.editor.text.len();
    assert!(a.editor.replace(&utf16(s), 1, false));
    a.sync_field();
}

#[test]
fn the_theme_mode_enables_the_rows_it_uses() {
    let keys = [
        "theme__mode",
        "theme__day_theme_id",
        "theme__night_theme_id",
        "theme__sunrise_time",
        "theme__sunset_time",
        "theme__accent_color",
        "ext_theme",
    ];
    let cases = [
        (None, [true, true, true, false, false, true, true]),
        (
            Some("ALWAYS_DAY"),
            [true, true, false, false, false, true, true],
        ),
        (
            Some("ALWAYS_NIGHT"),
            [true, false, true, false, false, true, true],
        ),
        (
            Some("FOLLOW_SYSTEM"),
            [true, true, true, false, false, true, true],
        ),
        (Some("FOLLOW_TIME"), [true; 7]),
    ];
    for (mode, enabled) in cases {
        let prefs: Vec<_> = mode.map(|m| ("theme__mode", m)).into_iter().collect();
        let (a, _) = app(&prefs, &[]);
        assert_eq!(
            keys.map(|k| a.rows().iter().position(|r| r.key == k).is_some()),
            [true; 7]
        );
        assert_eq!(keys.map(|k| a.enabled(index(&a, k))), enabled, "{mode:?}");
        assert!((0..keys.len()).all(|i| a.visible(i)), "{mode:?}");
    }
    // A disabled time row opens nothing.
    let (mut a, mut text) = app(&[], &[]);
    let row = id(&a, "theme__sunrise_time");
    assert!(!act(&mut a, &mut text, row));
    assert!(a.modal.is_none() && a.effects.is_empty());
}

#[test]
fn the_day_and_night_rows_name_the_indexed_theme_or_show_its_id() {
    let labels = [
        (format!("theme_label:{DAY}"), "Floris Day"),
        (format!("theme_label:{NIGHT}"), "Floris Night"),
        ("theme_label:my.ext:sunny".into(), "Sunny"),
    ];
    let facts: Vec<_> = labels.iter().map(|(k, v)| (k.as_str(), *v)).collect();
    let (mut a, _) = app(&[], &facts);
    let (day, night) = (
        index(&a, "theme__day_theme_id"),
        index(&a, "theme__night_theme_id"),
    );
    assert_eq!(
        (a.summary(day), a.summary(night)),
        ("Floris Day".into(), "Floris Night".into())
    );
    a.prefs
        .insert("theme__day_theme_id".into(), "my.ext:sunny".into());
    assert_eq!(a.summary(day), "Sunny");
    // Not in the index: the id as Kotlin's `toString` writes it.
    a.prefs
        .insert("theme__day_theme_id".into(), "my.ext:gone".into());
    assert_eq!(a.summary(day), "my.ext:gone");
    // Not an id at all: the preference reads back as its default.
    for bad in ["", "no-colon", "a:b:c"] {
        a.prefs.insert("theme__night_theme_id".into(), bad.into());
        assert_eq!(a.summary(night), "Floris Night", "{bad:?}");
    }
    // Before the index arrives, the ids themselves.
    a.facts.clear();
    a.prefs.clear();
    assert_eq!(
        (a.summary(day), a.summary(night)),
        (DAY.into(), NIGHT.into())
    );
}

#[test]
fn the_theme_rows_and_the_addon_box_open_their_compose_pages() {
    let (mut a, mut text) = app(&[], &[]);
    for (k, v) in [
        (
            "ext__addon_management_box__managing_placeholder",
            "Managing {extensions}",
        ),
        (
            "ext__addon_management_box__go_to_page",
            "Go to {ext_home_title}",
        ),
        ("ext__list__ext_theme", "Theme extensions"),
    ] {
        a.strings.insert(k.into(), v.into());
    }
    let routes = [
        (
            id(&a, "theme__day_theme_id"),
            "settings/theme/manage/SELECT_DAY",
        ),
        (
            id(&a, "theme__night_theme_id"),
            "settings/theme/manage/SELECT_NIGHT",
        ),
        (
            6000 + index(&a, "ext_theme") as i32,
            "ext/list/EXT_THEME?showUpdate=true",
        ),
    ];
    for (control, route) in routes {
        a.effects.clear();
        assert!(act(&mut a, &mut text, control), "{route}");
        assert_eq!(a.effects.len(), 1, "{route}");
        assert_eq!(a.effects[0][..2], ["compose", route]);
        assert!(a.effects[0][2].split('|').any(|r| r == "settings/theme"));
    }
    let i = index(&a, "ext_theme");
    a.scene(&mut text);
    assert_eq!(
        control(&a, 5000 + i as i32).label,
        "Managing theme extensions"
    );
    let button = control(&a, 6000 + i as i32);
    assert_eq!(
        (button.label.as_str(), button.role),
        ("Go to Theme extensions", "button")
    );
    assert_eq!(control(&a, 1000 + i as i32).role, "text");
    // The box sits below the accent colour row and inside the screen's width.
    let accent = control(&a, id(&a, "theme__accent_color")).bounds;
    assert!(button.bounds.top > accent.bottom && button.bounds.right < 1080.0);
}

#[test]
fn a_twelve_hour_dial_sets_the_sunrise_and_ok_stores_it_as_jetpref_does() {
    let (mut a, mut text) = app(&[("theme__mode", "FOLLOW_TIME")], &[]);
    let row = id(&a, "theme__sunrise_time");
    assert_eq!(a.summary(index(&a, "theme__sunrise_time")), "6:00 AM");
    assert!(act(&mut a, &mut text, row));
    a.scene(&mut text);
    let p = a.time_picker().unwrap();
    assert_eq!((p.hour, p.minute, p.minutes, p.input), (6, 0, false, false));
    // The twelve hours, 12 first, with 6 under the hand.
    assert_eq!(
        control(&a, 7160).label,
        "settings__time_picker__hour_suffix"
    );
    assert!(control(&a, 7166).selected && !control(&a, 7163).selected);
    assert!(a.controls.iter().all(|c| c.id != 7172));
    // A tap on 3 sets the hour and moves on to the minute.
    tap_dial(&mut a, &mut text, 3.0, 101.0);
    let p = a.time_picker().unwrap();
    assert_eq!((p.hour, p.minutes), (3, true));
    // A drag outward along 9 o'clock: past the slop the hand follows to 45.
    a.scene(&mut text);
    let (from, to) = (on_dial(&a, 9.0, 20.0), on_dial(&a, 9.0, 101.0));
    touch(&mut a, 0, from, 3);
    touch(&mut a, 2, to, 4);
    assert_eq!(a.time_picker().unwrap().minute, 45);
    touch(&mut a, 1, to, 5);
    let p = a.time_picker().unwrap();
    assert_eq!((p.hour, p.minute, p.hand), (3, 45, None));
    // PM keeps the hour on the dial and adds the afternoon.
    assert!(act(&mut a, &mut text, 7153));
    assert_eq!(a.time_picker().unwrap().hour, 15);
    assert!(act(&mut a, &mut text, 7));
    assert!(a.modal.is_none());
    assert_eq!(
        a.effects,
        [["write", "theme__sunrise_time", "15:45:00.000"]]
    );
    a.prefs
        .insert("theme__sunrise_time".into(), "15:45:00.000".into());
    assert_eq!(a.summary(index(&a, "theme__sunrise_time")), "3:45 PM");
}

#[test]
fn a_twenty_four_hour_dial_puts_the_afternoon_inside_and_accessibility_stays_on_the_hour() {
    let facts = [("is24hour", "true"), ("accessibility_touch", "true")];
    let (mut a, mut text) = app(&[("theme__mode", "FOLLOW_TIME")], &facts);
    let i = index(&a, "theme__sunset_time");
    assert_eq!(a.summary(i), "18:00");
    assert!(act(&mut a, &mut text, 1000 + i as i32));
    a.scene(&mut text);
    // 24 numbers, 0..11 outside and 12..23 inside, and no period toggle.
    assert!(control(&a, 7178).selected);
    assert!(a.controls.iter().all(|c| c.id != 7152 && c.id != 7153));
    tap_dial(&mut a, &mut text, 3.0, 69.0);
    let p = a.time_picker().unwrap();
    assert_eq!((p.hour, p.minutes), (15, false), "no move to the minute");
    tap_dial(&mut a, &mut text, 3.0, 101.0);
    assert_eq!(a.time_picker().unwrap().hour, 3);
    // The minute selector, then a minute's own button.
    assert!(act(&mut a, &mut text, 7151));
    a.scene(&mut text);
    assert_eq!(
        control(&a, 7163).label,
        "settings__time_picker__minute_suffix"
    );
    assert!(act(&mut a, &mut text, 7163));
    let p = a.time_picker().unwrap();
    assert_eq!((p.hour, p.minute, p.minutes), (3, 15, true));
    assert!(act(&mut a, &mut text, 7));
    assert_eq!(a.effects, [["write", "theme__sunset_time", "03:15:00.000"]]);
}

#[test]
fn the_neutral_button_resets_and_cancel_and_back_store_nothing() {
    let (mut a, mut text) = app(&[("theme__mode", "FOLLOW_TIME")], &[]);
    let row = id(&a, "theme__sunrise_time");
    assert!(act(&mut a, &mut text, row));
    tap_dial(&mut a, &mut text, 3.0, 101.0);
    assert!(act(&mut a, &mut text, 5));
    assert!(a.modal.is_none());
    assert_eq!(a.effects, [["reset", "theme__sunrise_time"]]);
    assert_eq!(a.value("theme__sunrise_time"), "06:00:00.000");
    for close in [true, false] {
        a.effects.clear();
        assert!(act(&mut a, &mut text, row));
        tap_dial(&mut a, &mut text, 3.0, 101.0);
        if close {
            assert!(act(&mut a, &mut text, 6));
        } else {
            a.back();
        }
        assert!(a.modal.is_none() && a.effects.is_empty(), "{close}");
        assert!(a.time_picker().is_none());
    }
}

#[test]
fn the_keyboard_mode_types_the_time_and_is_kept_for_the_row() {
    let (mut a, mut text) = app(&[("theme__mode", "FOLLOW_TIME")], &[]);
    let row = id(&a, "theme__sunrise_time");
    assert!(act(&mut a, &mut text, row));
    assert!(act(&mut a, &mut text, 7154));
    let p = a.time_picker().unwrap();
    assert!(p.input && !p.minutes);
    assert_eq!((a.field_id(), a.editor.focused), (7001, true));
    assert_eq!(a.editor.text, utf16("06"));
    assert_eq!(a.number_field(), Some("next"));
    assert!(a.effects.iter().any(|e| e == &["editor"]));
    a.scene(&mut text);
    assert_eq!(control(&a, 7001).role, "edit");
    assert!(a.controls.iter().any(|c| c.id == 7151));
    // 9 can only be 9 o'clock: the minute is next.
    type_over(&mut a, "9");
    let p = a.time_picker().unwrap();
    assert_eq!((p.hour, p.minutes), (9, true));
    assert_eq!((a.field_id(), a.number_field()), (7002, Some("done")));
    // 75 is no minute: the field keeps what it had.
    a.effects.clear();
    type_over(&mut a, "75");
    assert_eq!(
        (a.time_picker().unwrap().minute, a.editor.text.clone()),
        (0, utf16("00"))
    );
    assert!(a.effects.iter().any(|e| e == &["restartInput"]));
    type_over(&mut a, "41");
    assert_eq!(a.time_picker().unwrap().minute, 41);
    // Enter on the minute is Done: the keyboard goes, the dialog stays.
    a.effects.clear();
    assert!(a.field_key(66, 0));
    assert_eq!(a.effects, [["hideEditor"]]);
    assert!(!a.editor.focused && a.modal.is_some());
    assert!(act(&mut a, &mut text, 7153));
    assert_eq!(a.time_picker().unwrap().hour, 21);
    a.effects.clear();
    assert!(act(&mut a, &mut text, 7));
    assert_eq!(
        a.effects.last().unwrap(),
        &["write", "theme__sunrise_time", "21:41:00.000"]
    );
    // Opened again, the row keeps the keyboard; the sunset row still opens on the dial.
    assert!(act(&mut a, &mut text, row));
    assert!(a.time_picker().unwrap().input);
    a.back();
    let row = id(&a, "theme__sunset_time");
    assert!(act(&mut a, &mut text, row));
    assert!(!a.time_picker().unwrap().input);
}

#[test]
fn enter_on_the_hour_and_a_key_at_the_minute_s_start_move_between_the_fields() {
    let (mut a, mut text) = app(&[("theme__mode", "FOLLOW_TIME")], &[("is24hour", "true")]);
    let row = id(&a, "theme__sunset_time");
    assert!(act(&mut a, &mut text, row));
    assert!(act(&mut a, &mut text, 7154));
    assert_eq!(a.editor.text, utf16("18"));
    // On a 24-hour clock 1 could become 1x: the hour stays until Enter.
    type_over(&mut a, "1");
    assert_eq!((a.time_picker().unwrap().hour, a.field_id()), (1, 7001));
    assert!(a.field_key(66, 0));
    assert_eq!(
        (a.time_picker().unwrap().minutes, a.field_id()),
        (true, 7002)
    );
    // A letter is the field's to refuse; Delete at the start goes back to the hour.
    assert!(!a.field_key(29, 'a' as i32));
    a.editor.anchor = 0;
    a.editor.focus = 0;
    assert!(a.field_key(67, 0));
    assert_eq!(
        (a.time_picker().unwrap().minutes, a.field_id()),
        (false, 7001)
    );
    assert!(act(&mut a, &mut text, 7));
    assert_eq!(
        a.effects.last().unwrap(),
        &["write", "theme__sunset_time", "01:00:00.000"]
    );
}

#[test]
fn a_dragged_hand_trails_the_finger_by_the_touch_slop() {
    let (mut a, mut text) = app(&[("theme__mode", "FOLLOW_TIME")], &[("is24hour", "true")]);
    let row = id(&a, "theme__sunrise_time");
    assert!(act(&mut a, &mut text, row));
    assert!(act(&mut a, &mut text, 7151));
    a.scene(&mut text);
    let r = control(&a, 7132).bounds.width() * 101.0 / 256.0;
    let slop = (8.0f32 * 2.625 + 0.5).floor();
    let (x, y) = on_dial(&a, 0.0, 101.0);
    touch(&mut a, 0, (x, y), 1);
    // Sideways past the slop: detectDragGestures reports the finger less the slop, 30 px
    // right of 12, not where the finger is.
    touch(&mut a, 2, (x + slop + 30.0, y), 2);
    let trailed = time::nearest(time::turn(30.0, -r), 60);
    assert_ne!(trailed, time::nearest(time::turn(slop + 30.0, -r), 60));
    assert_eq!(a.time_picker().unwrap().minute, trailed);
    // Later moves keep the same offset.
    touch(&mut a, 2, (x + slop + 30.0, y + r), 3);
    let p = a.time_picker().unwrap();
    assert_eq!(p.minute, time::nearest(time::turn(30.0, 0.0), 60));
}
