//! AudioWhisper's roomy Smartbar: while words show, the main row is Gboard's (smartbar_gboard.rs
//! has its ends, smartbar_fit.rs how a word fits); here, what stays as it was.
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

#[test]
fn an_empty_row_keeps_todays_controls_and_auto_expansion() {
    let mut a = facts();
    for layout in [2.0, 3.0] {
        a[1] = layout;
        let idle = decode(&a);
        assert!(!idle.roomy());
        let empty = smartbar::layout(&idle, 400.0, &[], 0.0);
        let ids: Vec<_> = empty.actions.iter().map(|s| s.id).collect();
        assert_eq!(ids, [0, 1, 4], "mic, speaker and arrow; no pinned emoji");
    }
    // Shared and expanded with no words: the actions toolbar, between the same pinned controls.
    (a[1], a[3]) = (2.0, 1.0);
    let expanded = smartbar::layout(&decode(&a), 400.0, &[], 0.0);
    assert!(expanded.actions.len() > 3);
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
fn an_expanded_extended_row_keeps_its_actions_while_words_show_in_the_main_row() {
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
            let (main, second): (Vec<_>, Vec<_>) = after
                .actions
                .iter()
                .cloned()
                .partition(|s| s.bounds.top == top);
            assert_eq!(second, extended, "placement={placement} flip={flip}");
            let ids: Vec<_> = main.iter().map(|s| s.id).collect();
            assert_eq!(ids, if flip == 1.0 { [0, 4] } else { [4, 0] });
            assert_eq!(after.visible_dynamic, before.visible_dynamic);
            let center = if flip == 1.0 {
                (40.0, 370.0)
            } else {
                (30.0, 360.0)
            };
            assert_eq!(
                (after.center.left, after.center.top, after.center.right),
                (center.0, top, center.1)
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
