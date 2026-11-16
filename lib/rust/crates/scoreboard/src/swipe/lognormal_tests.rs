use super::*;
use crate::geometry::Keyboard;
use crate::swipe::Board;
use std::path::Path;

fn board() -> Board {
    let assets = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../app/src/main/assets");
    Board::new(&Keyboard::phone(&assets).unwrap())
}

fn key(board: &Board, c: char) -> usize {
    board.keys.iter().position(|k| k.code == c as i32).unwrap()
}

fn speeds(samples: &[Sample]) -> Vec<f32> {
    samples
        .windows(2)
        .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y) / (w[1].t - w[0].t))
        .collect()
}

#[test]
fn the_error_function_is_close_enough() {
    assert!(erf(0.0).abs() < 1e-8);
    assert!((erf(0.5) - 0.520_499_877_8).abs() < 2e-7);
    assert!((erf(-1.0) + 0.842_700_792_9).abs() < 2e-7);
    assert!((erf(3.0) - 0.999_977_909_5).abs() < 2e-7);
}

#[test]
fn a_stroke_ends_on_its_target_along_an_arc() {
    for turn in [0.0, 0.6, -1.2] {
        let chord: f64 = 300.0;
        let length = if turn == 0.0 {
            chord
        } else {
            chord * (turn / 2.0) / (turn / 2.0f64).sin()
        };
        let stroke = Stroke {
            t0: 0.0,
            mu: 150f64.ln(),
            sigma: 0.3,
            length,
            theta: 0.4 - turn / 2.0,
            turn,
        };
        let (x, y) = stroke.moved(1e6);
        assert!((x - chord * 0.4f64.cos()).abs() < 1e-6, "{turn}: {x}");
        assert!((y - chord * 0.4f64.sin()).abs() < 1e-6, "{turn}: {y}");
        assert_eq!(stroke.moved(0.0), (0.0, 0.0));
        // Half done at the median.
        assert!((stroke.done(150.0) - 0.5).abs() < 1e-6);
    }
}

#[test]
fn a_swipe_runs_from_the_first_key_to_the_last_in_time_order() {
    let board = board();
    let path = [
        key(&board, 'h'),
        key(&board, 'e'),
        key(&board, 'l'),
        key(&board, 'o'),
    ];
    for seed in 0..50 {
        let samples = draw(&board, &path, Hand::Neutral, &mut Rng::new(seed));
        let (first, last) = (samples[0], samples[samples.len() - 1]);
        let (h, o) = (&board.keys[path[0]], &board.keys[path[3]]);
        assert!((first.x - h.center_x).hypot(first.y - h.center_y) < 4.0 * 33.0);
        assert!((last.x - o.center_x).hypot(last.y - o.center_y) < 4.0 * 33.0);
        assert!(samples.windows(2).all(|w| w[1].t > w[0].t));
        // Three strokes of 100 to 300 ms each, overlapping.
        assert!((150.0..1500.0).contains(&last.t), "{}", last.t);
    }
}

#[test]
fn the_finger_slows_at_a_letter_it_passes_straight_through() {
    let board = board();
    // Q, T and P are in a line.
    let path = [key(&board, 'q'), key(&board, 't'), key(&board, 'p')];
    let mut slower = 0;
    for seed in 0..200 {
        let samples = draw(&board, &path, Hand::Neutral, &mut Rng::new(seed));
        let speed = speeds(&samples);
        // The slowest moment between the two speed peaks, and near T.
        let t = &board.keys[path[1]];
        let near_t: Vec<usize> = (0..speed.len())
            .filter(|&i| (samples[i].x - t.center_x).abs() < t.width)
            .collect();
        let peak = speed.iter().copied().fold(0.0, f32::max);
        let low = near_t.iter().map(|&i| speed[i]).fold(f32::MAX, f32::min);
        slower += usize::from(low < 0.8 * peak);
    }
    assert!(slower > 150, "{slower}");
}

#[test]
fn corners_are_cut_and_the_left_thumb_falls_short() {
    let board = board();
    // A sharp turn at E between Z and M... and back: Z, E, M.
    let path = [key(&board, 'z'), key(&board, 'e'), key(&board, 'm')];
    let e = &board.keys[path[1]];
    let (mut cut, mut short) = (0.0, 0.0);
    let n = 200;
    for seed in 0..n {
        let nearest = |hand| {
            let samples = draw(&board, &path, hand, &mut Rng::new(seed));
            samples
                .iter()
                .map(|s| (s.x - e.center_x).hypot(s.y - e.center_y))
                .fold(f32::MAX, f32::min)
        };
        cut += nearest(Hand::Neutral) / n as f32;
        short += nearest(Hand::LeftThumb) / n as f32;
    }
    // On average the finger turns a fifth of a key or more short of E's centre, but on the key.
    assert!(cut > 0.2 * e.width && cut < e.height / 2.0, "{cut}");
    assert!(short > cut, "{short} {cut}");
}

#[test]
fn the_left_thumb_bows_its_strokes() {
    let pivot = (0.0, 800.0);
    // Along the top row to the right, the thumb turns clockwise on screen (y down): positive.
    assert!(swept(pivot, (100.0, 100.0), (1300.0, 100.0)) > 0.5);
    assert!(swept(pivot, (1300.0, 100.0), (100.0, 100.0)) < -0.5);
    assert!(swept(pivot, (-10.0, 801.0), (-10.0, 799.0)).abs() < 0.3);
}
