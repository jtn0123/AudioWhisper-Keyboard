//! AudioWhisper's roomy Smartbar: while words show, the main row's pinned controls give way.
use keys::Rect;
use render::smartbar::{self, Config, Layout};

/// The phone's defaults: shared layout, flipped toggles, speaker and sticky mic shown.
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

#[test]
fn words_take_the_whole_main_row_in_every_shared_and_extended_arrangement() {
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
                    let mut ids: Vec<_> = before.actions.iter().map(|s| s.id).collect();
                    ids.sort_unstable();
                    let mut today = vec![2, 4];
                    today.extend((speaker != 0.0).then_some(1));
                    today.extend((sticky != 0.0).then_some(0));
                    today.sort_unstable();
                    assert_eq!(ids, today, "today's row: {case}");
                    assert!(before.center.width() < 300.0, "{case}");
                    assert!(after.actions.is_empty(), "{case}");
                    assert_eq!((after.center.left, after.center.right), (0.0, 400.0));
                    let spans: Vec<_> = after
                        .candidates
                        .iter()
                        .map(|s| (s.bounds.left, s.bounds.right))
                        .collect();
                    assert_eq!(
                        spans,
                        [(0.0, 132.0), (133.0, 266.0), (267.0, 400.0)],
                        "{case}"
                    );
                    // Touch and TalkBack follow the layout: where a control was, a word is.
                    for slot in &before.actions {
                        let x = mid(slot.bounds);
                        assert_eq!(before.hit(x, 20.0), Some((true, slot.id)), "{case}");
                        assert_eq!(after.hit(x, 20.0).map(|h| h.0), Some(false), "{case}");
                    }
                    assert_eq!(after.hit(1.0, 20.0), Some((false, 0)), "{case}");
                    assert_eq!(after.hit(399.0, 20.0), Some((false, 2)), "{case}");
                    let packed = after.packed(&words);
                    assert!(
                        packed[8..].chunks(6).all(|s| s[0] == 0.0),
                        "no action packets: {case}"
                    );
                }
            }
        }
    }
}

#[test]
fn an_empty_row_keeps_todays_controls_and_auto_expansion() {
    let mut a = facts();
    for layout in [2.0, 3.0] {
        a[1] = layout;
        let idle = decode(&a);
        assert!(!idle.roomy());
        let empty = smartbar::layout(&idle, 400.0, &[], 0.0);
        assert_eq!(empty.actions.len(), 4);
    }
    // Shared and expanded with no words: the actions toolbar, between the same pinned controls.
    (a[1], a[3]) = (2.0, 1.0);
    let expanded = smartbar::layout(&decode(&a), 400.0, &[], 0.0);
    assert!(expanded.actions.len() > 4);
}

#[test]
fn other_layouts_inline_autofill_and_expanded_actions_are_unchanged() {
    // (layout, shared, inline)
    for (layout, shared, inline) in [
        (0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0),
        (0.0, 0.0, 1.0),
        (1.0, 0.0, 1.0),
        (2.0, 1.0, 0.0),
        (2.0, 1.0, 1.0),
        (2.0, 0.0, 1.0),
        (3.0, 0.0, 1.0),
    ] {
        let mut a = facts();
        (a[1], a[3], a[12]) = (layout, shared, inline);
        let idle = decode(&a);
        a[14] = 1.0;
        let words = decode(&a);
        assert!(
            !words.roomy(),
            "layout={layout} shared={shared} inline={inline}"
        );
        assert_eq!(
            row(&idle),
            row(&words),
            "layout={layout} shared={shared} inline={inline}"
        );
    }
}

#[test]
fn dictation_and_reading_with_the_speaker_keep_todays_row() {
    for layout in [2.0, 3.0] {
        for flip in [0.0, 1.0] {
            let mut a = facts();
            (a[1], a[2], a[14]) = (layout, flip, 1.0);
            let today = |a: &[f32; 17]| {
                let mut idle = *a;
                idle[14] = 0.0;
                row(&decode(&idle))
            };
            for (at, speaker, roomy) in [
                (15, 1.0, false), // dictating: the mic shows its level and stop
                (15, 0.0, false),
                (16, 1.0, false), // reading: the speaker is the stop
                (16, 0.0, true),  // reading without a speaker: nothing to keep
                (13, 1.0, false), // dictation preview shows in the row
            ] {
                let mut b = a;
                (b[at], b[6]) = (1.0, speaker);
                let c = decode(&b);
                let case = format!("layout={layout} flip={flip} fact={at} speaker={speaker}");
                assert_eq!(c.roomy(), roomy, "{case}");
                if !roomy && at != 13 {
                    assert_eq!(row(&c), today(&b), "{case}");
                    assert!(
                        row(&c)
                            .actions
                            .iter()
                            .any(|s| s.id == if at == 15 { 0 } else { 1 })
                    );
                }
            }
        }
    }
}

#[test]
fn an_expanded_extended_row_keeps_its_actions_while_words_fill_the_main_row() {
    for placement in [0.0, 1.0, 2.0] {
        for flip in [0.0, 1.0] {
            let mut a = facts();
            (a[1], a[2], a[4], a[5]) = (3.0, flip, 1.0, placement);
            let before = row(&decode(&a));
            a[14] = 1.0;
            let words = decode(&a);
            let after = row(&words);
            let top = before.center.top;
            let extended: Vec<_> = before
                .actions
                .iter()
                .filter(|s| s.bounds.top != top)
                .cloned()
                .collect();
            assert!(extended.len() > 4);
            assert_eq!(after.actions, extended, "placement={placement} flip={flip}");
            assert_eq!(after.visible_dynamic, before.visible_dynamic);
            assert_eq!(
                (after.center.left, after.center.top, after.center.right),
                (0.0, top, 400.0)
            );
            assert_eq!(words.height(), 80.0);
        }
    }
}

#[test]
fn decode_takes_the_new_facts_only_as_a_full_seventeen() {
    let mut a = facts().to_vec();
    a.push(0.0);
    for len in 11..=18 {
        assert_eq!(
            Config::decode(&a[..len]).is_some(),
            [12, 14, 17].contains(&len),
            "{len}"
        );
    }
    let mut b = facts();
    (b[14], b[15], b[16]) = (1.0, 1.0, 1.0);
    let c = decode(&b);
    assert!(c.words && c.dictating && c.reading);
    let short = decode(&facts()[..14]);
    assert!(!short.words && !short.dictating && !short.reading && !short.roomy());
}
