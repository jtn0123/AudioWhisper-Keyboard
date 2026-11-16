//! While words show, the main row is Gboard's, mirrored for a left hand: the mic, the words and
//! the arrow. Its ends are Gboard's sizes, a row-high mic and an arrow three quarters as wide.
use keys::{Rect, codes::*};
use render::{
    accessibility::Semantics,
    scene::Scene,
    smartbar::{self, Config, GBOARD_TOGGLE, Layout},
    smartbar_scene::{Action, Smartbar},
};

/// The phone's defaults at density 1: shared layout, flipped toggles, speaker and sticky mic.
fn facts() -> [f32; 17] {
    [
        1.0, 2.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 19.0, 0.0, 1.0, 40.0, 0.0, 0.0, 0.0, 0.0, 0.0,
    ]
}
fn decode(a: &[f32]) -> Config {
    Config::decode(a).unwrap()
}
fn row(c: &Config) -> Layout {
    smartbar::layout(c, 400.0, &[50.0; 3], 0.0)
}
fn mid(r: Rect) -> f32 {
    (r.left + r.right) / 2.0
}
fn spans(slots: &[smartbar::Slot]) -> Vec<(usize, f32, f32)> {
    slots
        .iter()
        .map(|s| (s.id, s.bounds.left, s.bounds.right))
        .collect()
}

#[test]
fn words_show_between_the_mic_and_the_arrow_mirrored_by_flip() {
    for layout in [2.0, 3.0] {
        for flip in [0.0, 1.0] {
            for speaker in [0.0, 1.0] {
                for sticky in [0.0, 1.0] {
                    let mut a = facts();
                    (a[1], a[2], a[6], a[7]) = (layout, flip, speaker, sticky);
                    let idle = decode(&a);
                    a[14] = 1.0;
                    let words = decode(&a);
                    let case =
                        format!("layout={layout} flip={flip} speaker={speaker} sticky={sticky}");
                    assert!(!idle.roomy() && words.roomy(), "{case}");
                    let (before, after) = (row(&idle), row(&words));
                    // A row-high mic (absent without a sticky action) and a 30 of 40 arrow.
                    let mic = 40.0 * sticky;
                    let (expected, center) = if flip != 0.0 {
                        let mut s = vec![(0, 0.0, 40.0)];
                        s.truncate(sticky as usize);
                        s.push((4, 370.0, 400.0));
                        (s, (mic, 370.0))
                    } else {
                        let mut s = vec![(4, 0.0, 30.0)];
                        s.extend((sticky != 0.0).then_some((0, 360.0, 400.0)));
                        (s, (30.0, 400.0 - mic))
                    };
                    assert_eq!(spans(&after.actions), expected, "{case}");
                    assert!(after.actions.iter().all(|s| s.bounds.height() == 40.0));
                    assert_eq!((after.center.left, after.center.right), center, "{case}");
                    // Three words share the rest, a pixel apart, end to end.
                    let words_at = spans(&after.candidates);
                    assert_eq!(words_at.len(), 3, "{case}");
                    assert_eq!((words_at[0].1, words_at[2].2), center, "{case}");
                    for pair in words_at.windows(2) {
                        assert_eq!(pair[1].1 - pair[0].2, 1.0, "{case}");
                    }
                    let widths: Vec<_> = words_at.iter().map(|s| s.2 - s.1).collect();
                    assert!(
                        widths.iter().all(|w| (w - widths[2]).abs() <= 1.0),
                        "{case}"
                    );
                    // Touch follows: the ends are the mic and the arrow, the speaker's place a word.
                    for slot in &after.actions {
                        assert_eq!(after.hit(mid(slot.bounds), 20.0), Some((true, slot.id)));
                    }
                    assert_eq!(after.hit(center.0 + 0.5, 20.0), Some((false, 0)), "{case}");
                    assert_eq!(after.hit(center.1 - 0.5, 20.0), Some((false, 2)), "{case}");
                    for slot in before.actions.iter().filter(|s| s.id == 1) {
                        let x = mid(slot.bounds);
                        assert_eq!(after.hit(x, 20.0).map(|h| h.0), Some(false), "{case}");
                    }
                    let packed = after.packed(&words);
                    let ids: Vec<_> = packed[8..]
                        .chunks(6)
                        .filter(|s| s[0] == 1.0)
                        .map(|s| s[1] as usize)
                        .collect();
                    let want: Vec<_> = expected.iter().map(|s| s.0).collect();
                    assert_eq!(ids, want, "only the mic and the arrow are packed: {case}");
                }
            }
        }
    }
}

#[test]
fn the_arrow_is_three_quarters_of_the_row_and_its_button_keeps_its_size() {
    assert_eq!(GBOARD_TOGGLE, 0.75);
    for (row, slot) in [(40.0, 30.0), (150.0, 113.0), (113.0, 85.0), (105.0, 79.0)] {
        let mut a = facts();
        a[11] = row;
        assert_eq!(decode(&a).toggle_width(), row, "without words, a square");
        a[14] = 1.0;
        let c = decode(&a);
        assert_eq!(c.toggle_width(), slot, "row={row}");
        let arrow = smartbar::layout(&c, 1440.0, &[50.0; 3], 0.0).actions[1].clone();
        assert_eq!(arrow.bounds.width(), slot);
        // The button is still a row-high square, centred: only its margins overhang the slot.
        let face = arrow.face();
        assert_eq!((face.width(), face.height()), (row, row));
        assert_eq!(mid(face), mid(arrow.bounds));
        // Floris day's toggle margin is 6dp: its circle still fits the slot at either density.
        let margin = 6.0 * row / 40.0;
        assert!(face.left + margin >= arrow.bounds.left, "row={row}");
        assert!(face.right - margin <= arrow.bounds.right, "row={row}");
    }
}

fn action(code: i32) -> Action {
    Action {
        code,
        enabled: true,
        ..Default::default()
    }
}
fn talkback(words: bool, flip: bool) -> Vec<render::accessibility::Node> {
    let mut a = facts();
    (a[2], a[14]) = (f32::from(u8::from(flip)), f32::from(u8::from(words)));
    let mut bar = Smartbar::new(
        decode(&a),
        [
            VOICE_INPUT,
            READ_ALOUD,
            0,
            TOGGLE_ACTIONS_OVERFLOW,
            0,
            SETTINGS,
        ]
        .map(action)
        .to_vec(),
    );
    bar.intrinsic = vec![50.0, 60.0, 70.0];
    bar.relayout(400.0);
    let scene = Scene {
        width: 400,
        height: 300,
        strip: 40.0,
        candidates: vec!["one".into(), "two".into(), "three".into()],
        candidate_tokens: vec![10, 20, 30],
        smartbar: Some(bar),
        ..Default::default()
    };
    Semantics::default().snapshot(&scene, "unshifted")
}

#[test]
fn talkback_reads_the_mic_the_words_and_the_arrow_and_nothing_hidden() {
    for flip in [false, true] {
        let idle = talkback(false, flip);
        let words = talkback(true, flip);
        let bar = |nodes: &[render::accessibility::Node]| -> Vec<(i32, String)> {
            nodes
                .iter()
                .filter(|n| (2000..3000).contains(&n.id))
                .map(|n| (n.id, n.label.clone()))
                .collect()
        };
        let label = |id| bar(&idle).into_iter().find(|n| n.0 == id).unwrap().1;
        assert!(
            bar(&idle).iter().any(|n| n.0 == 2001),
            "today the speaker is read"
        );
        let mut kept = vec![(2000, label(2000)), (2004, label(2004))];
        if !flip {
            kept.reverse();
        }
        assert_eq!(bar(&words), kept, "flip={flip}");
        assert!(kept.iter().all(|n| !n.1.is_empty()));
        assert_eq!(label(2004), "Expand actions");
        let node = |id| words.iter().find(|n| n.id == id).unwrap().bounds;
        let (mic, arrow) = if flip {
            ((0.0, 40.0), (370.0, 400.0))
        } else {
            ((360.0, 400.0), (0.0, 30.0))
        };
        assert_eq!((node(2000).left, node(2000).right), mic, "flip={flip}");
        assert_eq!((node(2004).left, node(2004).right), arrow, "flip={flip}");
        let read: Vec<_> = words
            .iter()
            .filter(|n| n.label == "one" || n.label == "three")
            .map(|n| (n.bounds.left, n.bounds.right))
            .collect();
        let center = if flip { (40.0, 370.0) } else { (30.0, 360.0) };
        assert_eq!(
            (read[0].0, read[1].1),
            center,
            "words span the middle: flip={flip}"
        );
    }
}
