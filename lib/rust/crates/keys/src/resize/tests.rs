use super::*;

/// Two rows of three keys ten wide and ten high; the second row starts five to the right.
fn bounds(key: usize) -> Option<Rect> {
    let (row, column) = (key / 3, key % 3);
    let left = column as f32 * 10.0 + row as f32 * 5.0;
    let top = row as f32 * 10.0;
    (key < 6).then_some(Rect {
        left,
        top,
        right: left + 10.0,
        bottom: top + 10.0,
    })
}

fn key_at(x: f32, y: f32) -> Option<usize> {
    (0..6).find(|&k| bounds(k).is_some_and(|r| r.contains(x, y)))
}

/// Bayes' factor for a spread of a quarter key across and a fifth down, moving at most 0.15.
const TUNING: Resize = Resize {
    across: 0.0625,
    down: 0.04,
    cap: 0.15,
};

/// Key `likely` is ten times likelier than every other key.
fn favouring(likely: usize) -> impl Fn(usize) -> Option<f32> {
    move |key| Some(if key == likely { 10f32.ln() } else { 0.0 })
}

fn resize(hit: usize, at: (f32, f32), prior: impl Fn(usize) -> Option<f32>) -> usize {
    retarget(hit, at, &TUNING, prior, bounds, key_at)
}

#[test]
fn a_likelier_neighbour_takes_a_tap_near_the_border() {
    // ln 10 × 0.0625 = 0.144 key: a tap 0.1 key from the border moves, one 0.2 key away doesn't.
    assert_eq!(resize(0, (9.0, 5.0), favouring(1)), 1);
    assert_eq!(resize(0, (8.0, 5.0), favouring(1)), 0);
    assert_eq!(resize(2, (21.0, 5.0), favouring(1)), 1);
    // The likelier key keeps its own taps, and equal odds move nothing.
    assert_eq!(resize(1, (10.5, 5.0), favouring(1)), 1);
    assert_eq!(resize(0, (9.9, 5.0), |_| Some(0.0)), 0);
    // Between rows, with the factor down: key 4's top border, under key 1 and key 2.
    assert_eq!(resize(4, (17.0, 10.5), favouring(1)), 1);
    assert_eq!(resize(4, (23.0, 10.5), favouring(2)), 2);
    assert_eq!(resize(4, (17.0, 10.5), favouring(2)), 4);
    // 0.04 × ln 10 = 0.09 key down: a tap 0.1 key below the border stays.
    assert_eq!(resize(4, (17.0, 11.0), favouring(1)), 4);
}

#[test]
fn a_border_moves_no_further_than_the_cap() {
    let overwhelming = |key: usize| Some(if key == 1 { 1000f32.ln() } else { 0.0 });
    assert_eq!(resize(0, (8.6, 5.0), overwhelming), 1);
    assert_eq!(resize(0, (8.4, 5.0), overwhelming), 0);
    assert_eq!(resize(0, (5.0, 5.0), overwhelming), 0);
    let off = |hit, at| retarget(hit, at, &Resize::OFF, overwhelming, bounds, key_at);
    assert_eq!(off(0, (9.9, 5.0)), 0);
}

#[test]
fn keys_without_odds_never_move() {
    let no_neighbour = |key: usize| (key != 1).then_some(if key == 0 { 0.0 } else { 5.0 });
    assert_eq!(resize(0, (9.9, 5.0), no_neighbour), 0);
    let no_hit = |key: usize| (key != 0).then_some(5.0);
    assert_eq!(resize(0, (9.9, 5.0), no_hit), 0);
    assert_eq!(resize(0, (9.9, 5.0), |_| None), 0);
}

#[test]
fn the_nearer_border_wins_between_two_likelier_neighbours() {
    // Key 1's tap near its right border and its bottom border, both neighbours as likely.
    let both = |key: usize| {
        Some(if key == 2 || key == 4 {
            10f32.ln()
        } else {
            0.0
        })
    };
    assert_eq!(resize(1, (19.5, 9.0), both), 2);
    assert_eq!(resize(1, (19.0, 9.8), both), 4);
}

fn combined(at: (f32, f32), shift: impl Fn(usize) -> Option<(f32, f32)>, likely: usize) -> usize {
    let hit = key_at(at.0, at.1).unwrap();
    key_for(hit, at, &TUNING, shift, favouring(likely), bounds, key_at)
}

#[test]
fn the_learned_aim_goes_first_and_the_odds_never_undo_it() {
    // This user's taps land two short of what they mean on every key: the aim moves the tap on
    // key 0 at 9 to key 1, even though key 0 is the likelier.
    let short = |_: usize| Some((-2.0, 0.0));
    assert_eq!(combined((9.0, 5.0), short, 0), 1);
    // Where the aim keeps the tap, the odds measure from where it was meant: 1.5 short of the
    // border at 8.5 is 0.15 key from it, and from 7 + 2 = 9 it is 0.1, so the likelier key 1
    // takes it.
    assert_eq!(combined((7.0, 5.0), short, 1), 1);
    assert_eq!(combined((7.0, 5.0), |_| None, 1), 0);
    // The aim declines when the neighbour's own aim disagrees: the meant point stops at the
    // border, so only odds that would move a tap on the border move it.
    let disagree = |key: usize| Some(if key == 0 { (-3.0, 0.0) } else { (3.0, 0.0) });
    assert_eq!(combined((8.0, 5.0), disagree, 0), 0);
    assert_eq!(combined((8.0, 5.0), disagree, 1), 1);
    assert_eq!(combined((8.0, 5.0), |_| Some((0.0, 0.0)), 1), 0);
    // However far past the border the declined aim would put it, it is measured from the border.
    let far = |key: usize| Some(if key == 0 { (-5.0, 0.0) } else { (5.0, 0.0) });
    assert_eq!(combined((8.0, 5.0), far, 1), 1);
    assert_eq!(combined((8.0, 5.0), far, 0), 0);
}
