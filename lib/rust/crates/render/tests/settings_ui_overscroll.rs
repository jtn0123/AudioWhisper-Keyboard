//! Where content rests after a drag past an end (Compose's stretch overscroll), one-line
//! switch rows' height, the disabled switch's colours, the dialog radio text's whole-pixel
//! width and the switches' whole-pixel placement: the five differences the group 1 gallery
//! found against Compose pictures (stage4-followup4.md, part 1).
use render::{
    settings_ui::{App, catalogue::Kind},
    text::Text,
    touch::Motion,
};

const FRAME: i64 = 16_000_000;

fn app(route: &str) -> (App, Text) {
    let mut a = App {
        config: [480.0, 640.0, 1.0, 1.0],
        ..App::default()
    };
    a.navigate(route);
    let mut text = Text::new(Vec::<Vec<u8>>::new());
    a.scene(&mut text);
    (a, text)
}
fn event(a: &mut App, text: &mut Text, action: i32, t: i64, y: f32) {
    a.motion(&Motion {
        action,
        index: 0,
        ids: vec![0],
        times: vec![0, t],
        coords: vec![240.0, y],
    });
    a.scene(text);
}
/// Down at `ys[0]`, a move every frame through the rest, then a lift where the last one was.
fn drag(a: &mut App, text: &mut Text, ys: &[f32]) -> i64 {
    let mut t = 1_000 * FRAME;
    event(a, text, 0, t, ys[0]);
    for &y in &ys[1..] {
        t += FRAME;
        event(a, text, 2, t, y);
    }
    event(a, text, 1, t, ys[ys.len() - 1]);
    t
}
fn settle(a: &mut App, text: &mut Text, mut now: i64) {
    while a.fling.is_some() {
        now += FRAME;
        a.tick(now);
        a.scene(text);
    }
}
/// Up the screen by `forward` pixels in 40 px moves, then 30 px back down over six moves.
fn forward_then_back(start: f32, forward: f32) -> Vec<f32> {
    let steps = (forward / 40.0).ceil() as usize;
    let mut ys: Vec<f32> = (0..=steps).map(|k| start - k as f32 * 40.0).collect();
    let end = ys[steps];
    ys.extend((1..=6).map(|k| end + k as f32 * 5.0));
    ys
}

#[test]
fn a_backward_lift_after_pulling_past_the_end_leaves_content_at_the_end() {
    // The control: the same ending in the middle of the content flings it backward.
    let (mut a, mut text) = app("settings/clipboard");
    let max = a.max_scroll;
    assert!(
        max > 600.0,
        "the clipboard screen scrolls well past 600 px at 480x640"
    );
    let t = drag(&mut a, &mut text, &forward_then_back(600.0, 400.0));
    let lifted = a.scroll();
    assert!(a.fling.as_ref().is_some_and(|f| f.distance < 0.0));
    settle(&mut a, &mut text, t);
    assert!(
        a.scroll() < lifted - 1.0,
        "the backward fling moved the content"
    );

    // Past the end the stretch takes the reverse moves and the backward fling.
    let (mut a, mut text) = app("settings/clipboard");
    let t = drag(&mut a, &mut text, &forward_then_back(600.0, max + 400.0));
    assert_eq!(a.scroll(), max, "the reverse moves relaxed the stretch");
    assert!(a.stretch.edges[1] > 300.0);
    assert!(a.fling.as_ref().is_some_and(|f| f.distance < 0.0));
    settle(&mut a, &mut text, t);
    assert_eq!(a.scroll(), max);
    assert_eq!(a.can_scroll(), [true, false]);
    assert_eq!(a.stretch.edges, [0.0; 2], "released when the fling ended");
}

#[test]
fn a_fling_toward_a_stretched_edge_is_absorbed_and_releases_it() {
    let (mut a, mut text) = app("settings/clipboard");
    let max = a.max_scroll;
    let ys: Vec<f32> = (0..=40).map(|k| 600.0 - k as f32 * 60.0).collect();
    assert!(ys[0] - ys[40] > max + 200.0);
    drag(&mut a, &mut text, &ys);
    assert!(a.fling.is_none(), "the forward fling went into the stretch");
    assert_eq!(a.scroll(), max);
    assert_eq!(a.stretch.edges, [0.0; 2]);
    // At the top the same holds the other way.
    a.set_scroll(0.0);
    a.scene(&mut text);
    let ys: Vec<f32> = (0..=10).map(|k| 100.0 + k as f32 * 60.0).collect();
    drag(&mut a, &mut text, &ys);
    assert!(a.fling.is_none());
    assert_eq!(a.scroll(), 0.0);
}

#[test]
fn a_reverse_drag_relaxes_the_pull_before_it_moves_content() {
    let (mut a, mut text) = app("settings/clipboard");
    let max = a.max_scroll;
    a.set_scroll(max);
    a.scene(&mut text);
    let t = 1_000 * FRAME;
    event(&mut a, &mut text, 0, t, 500.0);
    // 8 px of touch slop, then 100 px past the end.
    event(&mut a, &mut text, 2, t + FRAME, 392.0);
    assert_eq!(a.stretch.edges, [0.0, 100.0]);
    event(&mut a, &mut text, 2, t + 2 * FRAME, 452.0);
    assert_eq!((a.scroll(), a.stretch.edges), (max, [0.0, 40.0]));
    event(&mut a, &mut text, 2, t + 3 * FRAME, 512.0);
    assert_eq!((a.scroll(), a.stretch.edges), (max - 20.0, [0.0, 0.0]));
    // Half a pixel past an end is rounding noise and pulls nothing.
    a.set_scroll(0.0);
    a.drag_by(-0.5);
    assert_eq!(a.stretch.edges, [0.0; 2]);
    a.drag_by(-3.0);
    assert_eq!(a.stretch.edges, [3.0, 0.0]);
}

#[test]
fn one_line_switch_rows_leave_room_for_the_switch_touch_target() {
    let (mut a, mut text) = app("settings/clipboard");
    a.config = [480.0, 4000.0, 1.0, 1.0];
    a.scene(&mut text);
    let height = |a: &App, i: usize| {
        let c = a.controls.iter().find(|c| c.id == 1000 + i as i32).unwrap();
        c.bounds.height()
    };
    let mut one_line = 0;
    for (i, r) in a.rows().iter().enumerate() {
        if a.visible(i) && a.summary(i).is_empty() {
            let expected = match r.kind {
                Kind::Toggle => 64.0,
                _ => 56.0,
            };
            assert_eq!(height(&a, i), expected, "{}", r.key);
            one_line += usize::from(matches!(r.kind, Kind::Toggle));
        }
    }
    assert!(one_line >= 3, "clipboard has one-line toggles");
}

#[test]
fn disabled_switches_use_the_disabled_tokens_under_the_row_fade() {
    let palette = |a: &mut App| {
        a.colors[0] = 0xfffafafa;
        a.colors[1] = 0xff1d1b20;
        a.colors[5] = 0xffe6e0e9;
        a.colors[7] = 0xff79747e;
        a.colors[8] = 0xfffef7ff;
    };
    let boxes = |a: &mut App, text: &mut Text| -> Vec<(f32, f32, u32)> {
        let panel = a.scene(text).panel.unwrap();
        panel
            .items
            .iter()
            .filter(|i| i.element == "settings-box")
            .map(|i| (i.bounds.width(), i.bounds.height(), i.styles[0].background))
            .collect()
    };
    // Clipboard history is off, so its toggles are disabled and unchecked.
    let (mut a, mut text) = app("settings/clipboard");
    a.config = [480.0, 4000.0, 1.0, 1.0];
    palette(&mut a);
    let b = boxes(&mut a, &mut text);
    assert!(b.contains(&(52.0, 32.0, 0xfff1eff2)), "outline");
    assert!(b.contains(&(48.0, 28.0, 0xfffaf8fb)), "track");
    assert!(b.contains(&(16.0, 16.0, 0xffdbd9dc)), "thumb");
    assert!(
        !b.iter().any(|x| x.2 == 0xffc9c7cb),
        "the faded enabled outline"
    );
    // The Smartbar's fixed row: a checked, disabled switch.
    let (mut a, mut text) = app("settings/smartbar");
    a.config = [480.0, 4000.0, 1.0, 1.0];
    palette(&mut a);
    a.prefs.insert("smartbar__enabled".into(), "true".into());
    a.prefs.insert(
        "smartbar__layout".into(),
        "SUGGESTIONS_ACTIONS_SHARED".into(),
    );
    let b = boxes(&mut a, &mut text);
    assert!(b.contains(&(52.0, 32.0, 0xfff1eff2)), "track");
    assert!(b.contains(&(24.0, 24.0, 0xfffcf9fc)), "thumb");
}

#[test]
fn dialog_radio_text_gets_the_whole_pixel_width_compose_gives_it() {
    // At 2.625 the 288 dp dialog's row padding is 21 px, the radio 137 and the end 42 (each
    // dp padding rounded on its own), so the text box is 535 px, not 288 - 84 dp = 535.5.
    let (mut a, mut text) = app("settings/keyboard");
    a.config = [1080.0, 2400.0, 2.625, 1.0];
    a.scene(&mut text);
    assert!(a.activate(1006)); // Spacebar label.
    let panel = a.scene(&mut text).panel.unwrap();
    let radio = a.controls.iter().find(|c| c.id == 3000).unwrap().clone();
    assert_eq!((radio.bounds.left, radio.bounds.width()), (183.0, 714.0));
    let label = panel
        .items
        .iter()
        .find(|i| i.element == "settings-subheader" && i.label == radio.label)
        .unwrap();
    assert_eq!((label.bounds.left, label.bounds.width()), (320.0, 535.0));
}

#[test]
fn switches_sit_on_the_whole_pixels_compose_lays_them_out() {
    // At 2.625 the 52 dp track is 137 px (136.5 rounded), 21 px below the top of the 126 px
    // touch target and 5 px outside its start (the overflow split by integer division). A
    // checked thumb is 63 px, 63 px along and 11 px down (10.5 rounded); an unchecked one 42 px,
    // 21 along and 21 down. A linked switch's 2 px divider straddles its box's start.
    let (mut a, mut text) = app("settings/keyboard");
    a.config = [1080.0, 6000.0, 2.625, 1.0];
    let panel = a.scene(&mut text).panel.unwrap();
    let boxes: Vec<_> = panel
        .items
        .iter()
        .filter(|i| i.element == "settings-box")
        .map(|i| i.bounds)
        .collect();
    let (mut rows, mut linked) = (0, 0);
    for c in a.controls.iter().filter(|c| c.role == "toggle") {
        let b = c.bounds;
        let inside = |w: f32| {
            boxes
                .iter()
                .find(|x| x.width() == w && x.top >= b.top && x.bottom <= b.bottom)
                .copied()
        };
        let track = inside(137.0).unwrap_or_else(|| panic!("track of {}", c.label));
        assert_eq!(track.height(), 84.0);
        if c.id >= 2000 {
            linked += 1;
            assert_eq!((b.left, b.width(), b.height()), (891.0, 147.0, 126.0));
            assert_eq!(track.left, b.left + 16.0);
            assert_eq!(track.top, b.top + 21.0);
            assert_eq!(inside(2.0).unwrap().left, b.left - 1.0, "divider");
        } else {
            rows += 1;
            assert_eq!(track.left, 907.0, "{}", c.label);
            let target = ((b.height() - 126.0) / 2.0).round();
            assert!(
                [target, 32.0].contains(&(track.top - 21.0 - b.top)),
                "{}",
                c.label
            );
        }
        let (size, along, down) = if c.checked == Some(true) {
            (63.0, 63.0, 11.0)
        } else {
            (42.0, 21.0, 21.0)
        };
        let thumb = inside(size).unwrap_or_else(|| panic!("thumb of {}", c.label));
        assert_eq!(
            (thumb.left, thumb.top, thumb.height()),
            (track.left + along, track.top + down, size),
            "{}",
            c.label
        );
    }
    assert!(rows > 3 && linked > 1, "{rows} rows, {linked} linked");
}
