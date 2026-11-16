//! Fast typing on the Rust touch path: keys pressed together, or one after another a millisecond
//! apart, are typed in the order they went down, none dropped, none twice, none cancelled, with
//! glide typing on or off. A key is typed when it goes up (`KEY_UP`).

use keys::codes::{DELETE, SHIFT, SPACE};
use keys::{Bounds, Rect};
use render::gesture_config::GestureConfig;
use render::protocol::{
    CURSOR_RIGHT, GLIDE_CANCEL, GLIDE_COMPLETE, GLIDE_POINT, KEY_CANCEL, KEY_DOWN, KEY_UP,
    PREF_SPACE_RIGHT,
};
use render::scene::{Key, Scene};
use render::touch::{Action, Motion, Touch};

const A: usize = 0;
const B: usize = 1;
const C: usize = 2;
const SPACE_KEY: usize = 3;
const DELETE_KEY: usize = 4;
const SHIFT_KEY: usize = 5;

/// a, b and c in a row, and space, delete and shift under them: keys 50 by 60 under a 20-high
/// strip.
fn scene() -> Scene {
    let codes = [97, 98, 99, SPACE, DELETE, SHIFT];
    let keys = codes
        .iter()
        .enumerate()
        .map(|(i, &code)| {
            let (column, row) = ((i % 3) as f32, (i / 3) as f32);
            let r = Rect {
                left: column * 50.0,
                right: (column + 1.0) * 50.0,
                top: row * 60.0,
                bottom: (row + 1.0) * 60.0,
            };
            Key {
                code,
                bounds: Bounds {
                    touch: r,
                    visible: r,
                },
                label: String::new(),
                hint: String::new(),
                icon: String::new(),
                styles: Default::default(),
                hints: Default::default(),
                enabled: true,
            }
        })
        .collect();
    Scene {
        width: 150,
        height: 140,
        strip: 20.0,
        keys,
        ..Scene::default()
    }
}

/// Fingers on the keyboard, as Android reports them.
struct Hand {
    scene: Scene,
    touch: Touch,
    /// Each finger down, by pointer id: its id and where it is.
    fingers: Vec<(i32, f32, f32)>,
    down_at: i64,
    now: i64,
    /// Every key and glide action, in order.
    actions: Vec<Action>,
}

impl Hand {
    fn new(glide: bool) -> Hand {
        Hand::with(glide, vec!["NO_ACTION".to_string(); 9])
    }

    /// With the gesture `actions` (`PREF_*`).
    fn with(glide: bool, actions: Vec<String>) -> Hand {
        let values = [
            1.0,
            32.0,
            f32::from(u8::from(glide)),
            0.0,
            0.0,
            1.0,
            200.0,
            32.0,
            1900.0,
            0.0,
            0.0,
        ];
        let config = GestureConfig::decode(&values, actions).unwrap();
        let mut touch = Touch::default();
        touch.configure(config);
        Hand {
            scene: scene(),
            touch,
            fingers: Vec::new(),
            down_at: 0,
            now: 0,
            actions: Vec::new(),
        }
    }

    /// One event a millisecond after the last, for the finger with `id`; its key actions.
    fn event(&mut self, action: i32, id: i32) -> Vec<Action> {
        self.now += 1_000_000;
        let motion = Motion {
            action,
            index: self.fingers.iter().position(|f| f.0 == id).unwrap_or(0),
            ids: self.fingers.iter().map(|f| f.0).collect(),
            times: vec![self.down_at, self.now],
            coords: self.fingers.iter().flat_map(|f| [f.1, f.2]).collect(),
        };
        let kinds = [
            CURSOR_RIGHT,
            KEY_DOWN,
            KEY_UP,
            KEY_CANCEL,
            GLIDE_POINT,
            GLIDE_COMPLETE,
            GLIDE_CANCEL,
        ];
        let out: Vec<Action> = self
            .touch
            .event(&mut self.scene, &motion)
            .into_iter()
            .filter(|a| kinds.contains(&a.0))
            .collect();
        self.actions.extend(&out);
        out
    }

    /// A finger goes down on `key`'s centre, with the lowest pointer id not in use, as Android
    /// gives; its id.
    fn down(&mut self, key: usize) -> i32 {
        let id = (0..)
            .find(|id| self.fingers.iter().all(|f| f.0 != *id))
            .unwrap();
        let r = self.scene.keys[key].bounds.touch;
        let at = (
            (r.left + r.right) / 2.0,
            (r.top + r.bottom) / 2.0 + self.scene.strip,
        );
        let first = self.fingers.is_empty();
        if first {
            self.down_at = self.now + 1_000_000;
        }
        self.fingers.push((id, at.0, at.1));
        self.fingers.sort_by_key(|f| f.0);
        self.event(if first { 0 } else { 5 }, id);
        id
    }

    fn up(&mut self, id: i32) -> Vec<Action> {
        let action = if self.fingers.len() == 1 { 1 } else { 6 };
        let out = self.event(action, id);
        self.fingers.retain(|f| f.0 != id);
        out
    }

    /// Every finger down moves by its `(dx, dy)`, in one move event.
    fn drift(&mut self, by: impl Fn(i32) -> (f32, f32)) {
        for finger in &mut self.fingers {
            let (dx, dy) = by(finger.0);
            finger.1 += dx;
            finger.2 += dy;
        }
        if let Some(&(id, ..)) = self.fingers.first() {
            self.event(2, id);
        }
    }

    /// The codes typed, in order.
    fn typed(&self) -> Vec<i32> {
        self.actions
            .iter()
            .filter(|a| a.0 == KEY_UP)
            .map(|a| self.scene.keys[a.1 as usize].code)
            .collect()
    }

    /// Nothing was cancelled and nothing was taken for a glide.
    fn all_taps(&self) -> bool {
        self.actions
            .iter()
            .all(|a| [KEY_DOWN, KEY_UP].contains(&a.0))
    }
}

fn code(key: usize) -> i32 {
    scene().keys[key].code
}

#[test]
fn a_rolled_key_goes_up_before_the_next_goes_down() {
    for glide in [false, true] {
        let mut hand = Hand::new(glide);
        let a = hand.down(A);
        let b = hand.down(B);
        assert_eq!(
            hand.actions,
            [
                Action(KEY_DOWN, A as i32, a),
                Action(KEY_UP, A as i32, a),
                Action(KEY_DOWN, B as i32, b),
            ],
            "glide {glide}"
        );
        // The rolled finger lifting types nothing more.
        assert_eq!(hand.up(a), []);
        assert_eq!(hand.up(b), [Action(KEY_UP, B as i32, b)]);
        assert_eq!(hand.typed(), [code(A), code(B)]);
    }
}

#[test]
fn rolled_keys_type_in_the_order_they_went_down_whichever_lifts_first() {
    let lifts = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for glide in [false, true] {
        for order in lifts {
            let mut hand = Hand::new(glide);
            let ids = [A, B, C].map(|key| hand.down(key));
            for finger in order {
                hand.up(ids[finger]);
            }
            assert_eq!(hand.typed(), [code(A), code(B), code(C)], "{order:?}");
            assert!(hand.all_taps(), "{:?}", hand.actions);
            assert_eq!(hand.actions.len(), 6, "{:?}", hand.actions);
        }
    }
}

#[test]
fn a_second_finger_landing_mid_press_types_both_in_order() {
    for glide in [false, true] {
        for first_up in [true, false] {
            let mut hand = Hand::new(glide);
            let a = hand.down(A);
            // The first finger rocks a little as the second comes down.
            hand.drift(|_| (3.0, 2.0));
            hand.drift(|_| (-1.0, 3.0));
            let b = hand.down(B);
            hand.drift(|id| if id == a { (2.0, -2.0) } else { (-3.0, 1.0) });
            let (first, second) = if first_up { (a, b) } else { (b, a) };
            hand.up(first);
            hand.drift(|_| (1.0, 1.0));
            hand.up(second);
            assert_eq!(hand.typed(), [code(A), code(B)], "glide {glide}");
            assert!(hand.all_taps(), "{:?}", hand.actions);
        }
    }
}

#[test]
fn a_letter_tapped_over_a_held_space_or_delete_comes_after_it() {
    for held in [SPACE_KEY, DELETE_KEY] {
        for glide in [false, true] {
            let mut hand = Hand::new(glide);
            let thumb = hand.down(held);
            let a = hand.down(A);
            // Space and delete act when they go up: they go up as the letter goes down.
            assert_eq!(
                hand.actions[1..],
                [
                    Action(KEY_UP, held as i32, thumb),
                    Action(KEY_DOWN, A as i32, a)
                ]
            );
            hand.up(a);
            hand.up(thumb);
            assert_eq!(hand.typed(), [code(held), code(A)], "glide {glide}");
            assert!(hand.all_taps(), "{:?}", hand.actions);
        }
    }
}

#[test]
fn shift_and_a_long_held_space_stay_down_under_a_second_finger() {
    // Shift held while a letter is tapped capitalises it: it stays down until it lifts.
    let mut hand = Hand::new(true);
    let shift = hand.down(SHIFT_KEY);
    let a = hand.down(A);
    hand.up(a);
    hand.up(shift);
    assert_eq!(
        hand.actions,
        [
            Action(KEY_DOWN, SHIFT_KEY as i32, shift),
            Action(KEY_DOWN, A as i32, a),
            Action(KEY_UP, A as i32, a),
            Action(KEY_UP, SHIFT_KEY as i32, shift),
        ]
    );
    // A space held long has done what holding it does; a letter landing leaves it be.
    let mut hand = Hand::new(true);
    let thumb = hand.down(SPACE_KEY);
    hand.touch.long_press(&hand.scene, thumb, false, true);
    let a = hand.down(A);
    assert_eq!(hand.actions[1..], [Action(KEY_DOWN, A as i32, a)]);
    // So does a space swiping the cursor along: the swipe goes on under the letter.
    let mut actions = vec!["NO_ACTION".to_string(); 9];
    actions[PREF_SPACE_RIGHT as usize] = "MOVE_CURSOR_RIGHT".to_string();
    let mut hand = Hand::with(true, actions);
    let thumb = hand.down(SPACE_KEY);
    hand.drift(|_| (20.0, 0.0));
    hand.drift(|_| (20.0, 0.0));
    assert!(
        hand.actions.iter().any(|a| a.0 == CURSOR_RIGHT),
        "{:?}",
        hand.actions
    );
    let swiped = hand.actions.len();
    let a = hand.down(A);
    assert_eq!(hand.actions[swiped..], [Action(KEY_DOWN, A as i32, a)]);
    hand.up(a);
    hand.up(thumb);
    assert!(!hand.typed().contains(&SPACE), "{:?}", hand.actions);
}

/// A small deterministic generator, so a failure repeats.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 33) % n
    }
}

#[test]
fn quick_taps_overlapping_or_not_keep_their_order() {
    for glide in [false, true] {
        let mut random = Lcg(20261116);
        let mut hand = Hand::new(glide);
        let mut pressed = Vec::new();
        // Each finger down and how many more presses it stays down for.
        let mut held: Vec<(i32, u64)> = Vec::new();
        for _ in 0..600 {
            let key = [A, B, C, SPACE_KEY, DELETE_KEY][random.below(5) as usize];
            pressed.push(code(key));
            let id = hand.down(key);
            held.push((id, random.below(4)));
            let jitter = [random.below(5), random.below(5)].map(|d| d as f32 - 2.0);
            hand.drift(|_| (jitter[0], jitter[1]));
            // Fingers whose time is up lift, in a shuffled order; at most three stay down.
            for finger in &mut held {
                finger.1 = finger.1.saturating_sub(1);
            }
            while held.len() > 2 || held.iter().any(|f| f.1 == 0) {
                let due: Vec<usize> = (0..held.len())
                    .filter(|&i| held[i].1 == 0 || held.len() > 2)
                    .collect();
                let (id, _) = held.remove(due[random.below(due.len() as u64) as usize]);
                hand.up(id);
            }
        }
        for (id, _) in held {
            hand.up(id);
        }
        let typed = hand.typed();
        let differs = typed.iter().zip(&pressed).position(|(t, p)| t != p);
        assert_eq!(typed.len(), pressed.len(), "glide {glide}");
        assert_eq!(
            differs, None,
            "glide {glide}: the first key typed out of order"
        );
        assert!(hand.all_taps(), "glide {glide}");
    }
}
