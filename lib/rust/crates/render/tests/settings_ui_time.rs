//! The time picker's maths (stage4-other-theme.md): Jetpref's stored form and summary, the dial's
//! angles and rings as Material 3 1.4.0 rounds them, and TimeInput's rules for typed numbers.
use render::settings_ui::{
    editor::Editor,
    time::{self, TimePicker},
};
use std::f32::consts::TAU;

fn typed(text: &str, at: usize) -> Editor {
    Editor {
        text: text.encode_utf16().collect(),
        anchor: at,
        focus: at,
        ..Editor::default()
    }
}
fn shown(e: &Editor) -> String {
    String::from_utf16_lossy(&e.text)
}
/// The point at `twelfths` of a turn clockwise from 12, `r` from the centre.
fn at(twelfths: f32, r: f32) -> (f32, f32) {
    let a = twelfths / 12.0 * TAU;
    (r * a.sin(), -r * a.cos())
}

#[test]
fn every_time_stores_and_reads_back() {
    for hour in 0..24 {
        for minute in 0..60 {
            let stored = time::store(hour, minute);
            assert_eq!(stored.len(), 12, "{stored}");
            assert_eq!(time::parse(&stored), Some((hour, minute)), "{stored}");
        }
    }
    assert_eq!(time::store(7, 5), "07:05:00.000");
    // Jetpref's pattern takes seconds and milliseconds it then drops.
    assert_eq!(time::parse("18:30:59.999"), Some((18, 30)));
}

#[test]
fn text_jetpref_turns_down_reads_as_none() {
    for bad in [
        "",
        "24:00:00.000",
        "12:60:00.000",
        "12:00:60.000",
        "1:00:00.000",
        "12:00:00.00",
        "12:00:00.0000",
        "12:00:00:000",
        "12-00-00.000",
        "ab:cd:ef.ghi",
        "+1:00:00.000",
        "12:00",
        "１２:00:00.000",
    ] {
        assert_eq!(time::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn the_summary_follows_the_system_clock_at_midnight_noon_and_the_day_s_edges() {
    let cases = [
        ((0, 0), "0:00", "12:00 AM"),
        ((0, 1), "0:01", "12:01 AM"),
        ((1, 5), "1:05", "1:05 AM"),
        ((11, 59), "11:59", "11:59 AM"),
        ((12, 0), "12:00", "12:00 PM"),
        ((12, 30), "12:30", "12:30 PM"),
        ((13, 7), "13:07", "1:07 PM"),
        ((23, 59), "23:59", "11:59 PM"),
    ];
    for ((hour, minute), h24, h12) in cases {
        assert_eq!(time::shown(hour, minute, true), h24);
        assert_eq!(time::shown(hour, minute, false), h12);
    }
    assert_eq!(
        (0..24)
            .map(|h| time::display_hour(h, false))
            .collect::<Vec<_>>(),
        [
            12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11
        ]
    );
    assert!((0..24).all(|h| time::display_hour(h, true) == h));
}

#[test]
fn angles_run_clockwise_from_twelve_and_stay_in_one_turn() {
    let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
    assert!(close(time::turn(0.0, -1.0), 0.0));
    assert!(close(time::turn(1.0, 0.0), TAU / 4.0));
    assert!(close(time::turn(0.0, 1.0), TAU / 2.0));
    assert!(close(time::turn(-1.0, 0.0), 3.0 * TAU / 4.0));
    for k in 0..360 {
        let a = time::turn(
            (k as f32).to_radians().sin(),
            -(k as f32).to_radians().cos(),
        );
        assert!((0.0..=TAU).contains(&a), "{k}: {a}");
    }
    // Just left of 12 is the turn's end, which both roundings take back to 0.
    let end = time::turn(-1e-7, -1.0);
    assert_eq!((time::nearest(end, 12), time::snapped(end, false)), (0, 0));
}

#[test]
fn a_drag_rounds_half_marks_up_and_a_tap_rounds_them_to_even() {
    let twelfths = |n: f32| n / 12.0 * TAU;
    assert_eq!(time::nearest(twelfths(2.5), 12), 3);
    assert_eq!(time::nearest(twelfths(3.5), 12), 4);
    assert_eq!(time::nearest(twelfths(2.49), 12), 2);
    assert_eq!(time::nearest(twelfths(11.6), 12), 0);
    assert_eq!(time::nearest(29.5 / 60.0 * TAU, 60), 30);
    assert_eq!(time::nearest(59.7 / 60.0 * TAU, 60), 0);
    assert_eq!(time::snapped(twelfths(2.5), false), 2);
    assert_eq!(time::snapped(twelfths(3.5), false), 4);
    assert_eq!(time::snapped(twelfths(2.5), true), 10);
    assert_eq!(time::snapped(twelfths(11.7), true), 0);
    assert_eq!(time::snapped(twelfths(7.2), true), 35);
}

#[test]
fn a_twenty_four_hour_dial_sets_the_afternoon_on_its_inner_ring() {
    let mut p = TimePicker::new(9, 30, false, true);
    let (x, y) = at(3.0, 100.0);
    p.tap_dial(x, y, true, true);
    assert_eq!(p.hour, 15);
    assert!(p.inner(true) && !p.inner(false));
    p.tap_dial(x, y, false, true);
    assert_eq!(p.hour, 3);
    let (x, y) = at(0.0, 100.0);
    p.drag_dial(x, y, true, true);
    assert_eq!((p.hour, p.hand), (12, Some(0.0)));
    p.drag_dial(x, y, false, true);
    assert_eq!(p.hour, 0);
    // The minute dial has one ring.
    p.minutes = true;
    assert!(!p.inner(true));
    let (x, y) = at(9.0, 40.0);
    p.drag_dial(x, y, true, true);
    assert_eq!((p.hour, p.minute), (0, 45));
}

#[test]
fn a_twelve_hour_dial_keeps_the_period() {
    let mut p = TimePicker::new(14, 0, false, false);
    let (x, y) = at(3.0, 100.0);
    p.tap_dial(x, y, true, false);
    assert_eq!(p.hour, 15);
    p.set_pm(false);
    assert_eq!(p.hour, 3);
    p.set_pm(false);
    assert_eq!(p.hour, 3);
    let (x, y) = at(0.0, 100.0);
    p.tap_dial(x, y, false, false);
    assert_eq!(p.hour, 0);
    p.set_pm(true);
    assert_eq!(p.hour, 12);
    // The numbers' own actions: 12 is midnight or noon by the period.
    p.pick(12, false);
    assert_eq!(p.hour, 12);
    p.set_pm(false);
    p.pick(12, false);
    assert_eq!(p.hour, 0);
    p.pick(7, false);
    assert_eq!(p.hour, 7);
    p.minutes = true;
    p.pick(55, false);
    assert_eq!((p.hour, p.minute), (7, 55));
    assert!((p.angle() - 55.0 / 60.0 * TAU).abs() < 1e-5);
}

#[test]
fn typing_an_hour_follows_time_input_on_a_twelve_hour_clock() {
    let mut p = TimePicker::new(9, 30, true, false);
    assert_eq!(p.fields.each_ref().map(shown), ["09", "30"]);
    // A single 1 could still become 10, 11 or 12: the hour stays selected.
    assert_eq!(
        p.typed(0, &typed("1", 1), false).map(|e| shown(&e)),
        Some("1".into())
    );
    assert_eq!((p.hour, p.minutes), (1, false));
    // Above 1 nothing longer fits, so the minute is next.
    assert!(p.typed(0, &typed("5", 1), false).is_some());
    assert_eq!((p.hour, p.minutes), (5, true));
    assert_eq!(p.typed(0, &typed("13", 2), false), None);
    assert_eq!(p.hour, 5);
    assert!(p.typed(0, &typed("12", 2), false).is_some());
    assert_eq!(p.hour, 0);
    p.set_pm(true);
    assert!(p.typed(0, &typed("12", 2), false).is_some());
    assert_eq!(p.hour, 12);
    assert!(p.typed(0, &typed("3", 1), false).is_some());
    assert_eq!(p.hour, 15);
    assert!(p.typed(0, &typed("", 0), false).is_some());
    assert_eq!(p.hour, 12);
}

#[test]
fn typing_on_a_twenty_four_hour_clock_and_in_the_minute() {
    let mut p = TimePicker::new(9, 30, true, true);
    assert!(p.typed(0, &typed("23", 2), true).is_some());
    assert_eq!((p.hour, p.minutes), (23, false));
    assert_eq!(p.typed(0, &typed("24", 2), true), None);
    assert!(p.typed(0, &typed("", 0), true).is_some());
    assert_eq!(p.hour, 0);
    assert!(p.typed(1, &typed("59", 2), true).is_some());
    assert_eq!(p.minute, 59);
    assert_eq!(p.typed(1, &typed("60", 2), true), None);
    assert!(p.typed(1, &typed("", 0), true).is_some());
    assert_eq!(p.minute, 0);
}

#[test]
fn a_third_digit_typed_after_the_first_keeps_only_that_first() {
    let mut p = TimePicker::new(9, 30, true, true);
    // The caret after the first of three characters: the first is the number.
    let kept = p.typed(0, &typed("109", 1), true).unwrap();
    assert_eq!(
        (shown(&kept), kept.anchor, kept.focus, kept.composing),
        ("1".into(), 1, 1, None)
    );
    assert_eq!(p.hour, 1);
    // Anywhere else the three are read whole, and 91 is no hour.
    assert_eq!(p.typed(0, &typed("091", 3), true), None);
    // Kotlin's toInt: a sign is allowed, but a negative hour is turned down.
    assert!(p.typed(0, &typed("+5", 2), true).is_some());
    assert_eq!(p.hour, 5);
    assert!(p.typed(0, &typed("-0", 2), true).is_some());
    assert_eq!(p.hour, 0);
    for bad in ["-5", "a", "5a", "٣"] {
        assert_eq!(p.typed(0, &typed(bad, 1), true), None, "{bad}");
    }
    // The same text with the caret moved is kept as it is.
    let moved = typed("09", 0);
    assert_eq!(p.typed(0, &moved, true), Some(moved));
}

#[test]
fn the_fields_reset_after_a_change_they_did_not_type() {
    let mut p = TimePicker::new(9, 30, true, false);
    p.typed(0, &typed("1", 1), false);
    // The fields typed it: they keep "1".
    assert!(!p.settle(false));
    assert_eq!(shown(&p.fields[0]), "09");
    p.fields[0] = typed("1", 1);
    p.set_pm(true);
    assert!(p.settle(false));
    assert_eq!(p.fields.each_ref().map(shown), ["01", "30"]);
    assert!(!p.settle(false));
    // The dial never resets fields it does not show.
    let mut dial = TimePicker::new(9, 30, false, false);
    dial.set_pm(true);
    assert!(!dial.settle(false));
}

#[test]
fn the_hand_is_a_two_pixel_line_with_flat_ends() {
    let color = 0xff6750a4;
    let px = time::segment(10, 10, (1.0, 5.0), (9.0, 5.0), 1.0, color);
    let alpha = |x: usize, y: usize| (px[y * 10 + x] as u32) >> 24;
    assert!((1..9).all(|x| alpha(x, 4) == 255 && alpha(x, 5) == 255));
    assert!((0..10).all(|x| alpha(x, 3) == 0 && alpha(x, 6) == 0));
    assert_eq!((alpha(0, 5), alpha(9, 5)), (0, 0));
    assert!(
        px.iter()
            .all(|&p| p as u32 & 0x00ff_ffff == color & 0x00ff_ffff)
    );
    // A slanted line shades the pixels it half covers.
    let slant = time::segment(10, 10, (1.0, 1.0), (9.0, 9.0), 1.0, color);
    let partial = slant
        .iter()
        .filter(|&&p| (1..255).contains(&((p as u32) >> 24)));
    assert!(partial.count() > 0);
}
