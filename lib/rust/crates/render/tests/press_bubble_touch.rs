//! The press bubble on the Rust touch path: a finger on a character key opens its bubble, a slide
//! carries it to the next key, release takes it away, a long-press popup takes its place, and
//! fast typing with fingers rolling over one another never leaves one behind. Every frame's
//! narrow repaint draws what a whole repaint does, and the room above the keys is clear again
//! once the bubble has gone.
use keys::codes::{DELETE, ENTER, SHIFT, SPACE};
use keys::{Bounds, Rect};
use render::{
    accessibility::Semantics,
    gesture_config::GestureConfig,
    popup_scene::{configure, headroom},
    press_bubble::mark,
    protocol::*,
    raster::Raster,
    scene::{Key, Popup, Scene},
    style::Style,
    theme::{Request, Theme},
    touch::{Action, Motion, Touch},
};

const Q: usize = 0;
const W: usize = 1;
const E: usize = 2;
const R: usize = 3;
const SHIFT_KEY: usize = 4;
const SPACE_KEY: usize = 5;
const ENTER_KEY: usize = 6;
const DELETE_KEY: usize = 7;
const SIZE: f32 = 60.0;

/// q, w, e and r, and shift, space, enter and delete under them: keys 60 square under a strip
/// 20 tall, with the room above that their popups take.
fn scene() -> Scene {
    let codes = [113, 119, 101, 114, SHIFT, SPACE, ENTER, DELETE];
    let keys = codes
        .iter()
        .enumerate()
        .map(|(i, &code)| {
            let (column, row) = ((i % 4) as f32, (i / 4) as f32);
            let touch = Rect {
                left: column * SIZE,
                right: (column + 1.0) * SIZE,
                top: row * SIZE,
                bottom: (row + 1.0) * SIZE,
            };
            let visible = Rect {
                left: touch.left + 2.0,
                right: touch.right - 2.0,
                top: touch.top + 2.0,
                bottom: touch.bottom - 2.0,
            };
            Key {
                code,
                bounds: Bounds { touch, visible },
                label: char::from_u32(code as u32)
                    .filter(|_| code > SPACE)
                    .map_or(String::new(), String::from),
                hint: String::new(),
                icon: String::new(),
                styles: Default::default(),
                hints: Default::default(),
                enabled: true,
            }
        })
        .collect();
    let mut scene = Scene {
        width: 240,
        height: 140,
        strip: 20.0,
        keyboard_height: 2.0 * SIZE,
        desired_key: [SIZE - 4.0, SIZE - 4.0],
        keys,
        background: 0xffffffff,
        ..Scene::default()
    };
    mark(&mut scene, "characters", &vec!["character".into(); 8]);
    scene.headroom = headroom(&scene, false);
    scene
}

fn config(glide: bool, bubble: bool) -> GestureConfig {
    let values = [
        1.,
        32.,
        f32::from(u8::from(glide)),
        0.,
        0.,
        1.,
        200.,
        32.,
        1900.,
        0.,
        0.,
    ];
    let mut values = values.to_vec();
    values.push(f32::from(u8::from(bubble)));
    GestureConfig::decode(&values, vec!["NO_ACTION".into(); 9]).unwrap()
}

fn theme() -> Theme {
    let json = r##"{"$schema":"https://schemas.florisboard.org/snygg/v2/stylesheet",
        "key-popup-box":{"background":"#445566","foreground":"#ffffff","font-size":"20sp",
            "shadow-elevation":"2dp","shape":"rounded-corner(6dp,6dp,6dp,6dp)"},
        "key-popup-element":{"foreground":"#ffffff","font-size":"18sp"}}"##;
    Theme::new(json, vec![], vec![], 1.0, 1.0, 1.0).unwrap()
}

/// Fingers on the keyboard, as Android reports them, and the native session's part: the bubble
/// a touch opens is styled before the frame that draws it.
struct Hand {
    scene: Scene,
    touch: Touch,
    theme: Theme,
    fingers: Vec<(i32, f32, f32)>,
    down_at: i64,
    now: i64,
    actions: Vec<Action>,
}

impl Hand {
    fn new(bubble: bool) -> Hand {
        Hand::with(false, bubble)
    }

    fn with(glide: bool, bubble: bool) -> Hand {
        let mut touch = Touch::default();
        touch.configure(config(glide, bubble));
        Hand {
            scene: scene(),
            touch,
            theme: theme(),
            fingers: Vec::new(),
            down_at: 0,
            now: 0,
            actions: Vec::new(),
        }
    }

    /// One event 8 ms after the last, for the finger with `id`.
    fn event(&mut self, action: i32, id: i32) -> Vec<Action> {
        self.now += 8_000_000;
        let motion = Motion {
            action,
            index: self.fingers.iter().position(|f| f.0 == id).unwrap_or(0),
            ids: self.fingers.iter().map(|f| f.0).collect(),
            times: vec![self.down_at, self.now],
            coords: self.fingers.iter().flat_map(|f| [f.1, f.2]).collect(),
        };
        let out = self.touch.event(&mut self.scene, &motion);
        self.style();
        self.actions.extend(&out);
        out
    }

    /// Styles the bubble a touch just opened, as `style_bubble` in the native session does.
    fn style(&mut self) {
        let unstyled = self.scene.bubble.as_ref().filter(|b| b.style.is_none());
        let Some(key) = unstyled.map(|b| b.key) else {
            return;
        };
        let request = Request {
            element: String::new(),
            mode: "characters".into(),
            shift: "unshifted".into(),
            code: self.scene.keys[key].code,
            key_type: "character".into(),
            selector: 0,
            auto_commit: false,
            popup_source: None,
        };
        self.theme.bubble(&mut self.scene, &request, key == E);
    }

    fn centre(&self, key: usize) -> (f32, f32) {
        let r = self.scene.keys[key].bounds.touch;
        let x = (r.left + r.right) / 2.0;
        (x, (r.top + r.bottom) / 2.0 + self.scene.strip)
    }

    /// A finger goes down on `key`'s centre with the lowest free pointer id; its id.
    fn down(&mut self, key: usize) -> i32 {
        let id = (0..).find(|id| self.fingers.iter().all(|f| f.0 != *id));
        let id = id.unwrap();
        let (x, y) = self.centre(key);
        let first = self.fingers.is_empty();
        if first {
            self.down_at = self.now + 8_000_000;
        }
        self.fingers.push((id, x, y));
        self.fingers.sort_by_key(|f| f.0);
        self.event(if first { 0 } else { 5 }, id);
        id
    }

    fn up(&mut self, id: i32) -> Vec<Action> {
        let out = self.event(if self.fingers.len() == 1 { 1 } else { 6 }, id);
        self.fingers.retain(|f| f.0 != id);
        out
    }

    /// Finger `id` moves to `key`'s centre slowly, 10 px an event, as a slide does.
    fn slide(&mut self, id: i32, key: usize) {
        let (x, y) = self.centre(key);
        loop {
            let finger = self.fingers.iter_mut().find(|f| f.0 == id).unwrap();
            let (dx, dy) = (x - finger.1, y - finger.2);
            if dx == 0.0 && dy == 0.0 {
                break;
            }
            finger.1 += dx.clamp(-10.0, 10.0);
            finger.2 += dy.clamp(-10.0, 10.0);
            self.event(2, id);
        }
    }

    /// The key whose bubble the frame draws.
    fn shown(&self) -> Option<usize> {
        self.scene.bubble_view().map(|(b, ..)| b.key)
    }

    fn typed(&self) -> Vec<usize> {
        let ups = self.actions.iter().filter(|a| a.0 == KEY_UP);
        ups.map(|a| a.1 as usize).collect()
    }
}

/// Opens key `key`'s long-press popup as the native session does on `OPEN_POPUP`.
fn open_popup(scene: &mut Scene, key: usize) {
    let choice = |background| Style {
        size: 18.0,
        background,
        ..Style::default()
    };
    scene.popup = Some(Popup {
        key,
        labels: vec!["3".into(), "é".into()],
        selected: 0,
    });
    scene.popup_style = Style {
        background: 0xff223344,
        ..Style::default()
    };
    scene.popup_styles = vec![[choice(0xff112233), choice(0xff8899aa)]; 2];
    configure(scene, 0, false).expect("popup grid");
}

fn font() -> Vec<u8> {
    include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec()
}

/// Draws each frame's damage onto the last and checks it against a whole repaint, and that the
/// room above the keys is clear whenever no bubble or popup is up.
fn repaint(frames: &[Scene]) {
    let (height, full) = (frames[0].buffer_height(), frames[0].buffer_rect());
    let mut raster = Raster::new(frames[0].width, height, vec![font()]);
    let mut reference = Raster::new(frames[0].width, height, vec![font()]);
    raster.draw(&frames[0], full).expect("raster");
    let room = usize::from(frames[0].headroom) * usize::from(frames[0].width) * 4;
    for (i, pair) in frames.windows(2).enumerate() {
        let regions = pair[0].damage_regions(&pair[1]);
        // Keys that change (a label, under a held bubble too) repaint everything.
        let narrow = pair[0].keys == pair[1].keys;
        assert_eq!(
            regions.contains(&full),
            !narrow,
            "frame {i}: a bubble repaints itself only"
        );
        let shown = raster.pixels().to_vec();
        raster
            .draw_frame(&pair[0], &pair[1], &regions)
            .expect("raster");
        posted(&shown, raster.pixels(), &pair[1], &regions);
        reference.draw(&pair[1], full).expect("raster");
        assert!(
            raster.pixels() == reference.pixels(),
            "frame {i}: dirty differs from full"
        );
        let up = pair[1].bubble_view().is_some() || pair[1].popup.is_some();
        let clear = raster.pixels()[..room].iter().all(|&v| v == 0);
        assert_eq!(
            clear, !up,
            "frame {i}: the room above shows only a bubble or popup"
        );
    }
}

/// Checks that every pixel that changed from `before` to `after` lies inside the damage the
/// frame posts to the screen.
fn posted(before: &[u8], after: &[u8], scene: &Scene, regions: &[Rect]) {
    let width = usize::from(scene.width);
    let posted: Vec<Rect> = regions.iter().map(|&r| scene.buffer_pixels(r)).collect();
    let pixels = before
        .as_chunks::<4>()
        .0
        .iter()
        .zip(after.as_chunks::<4>().0);
    for (i, _) in pixels.enumerate().filter(|(_, (a, b))| a != b) {
        let (x, y) = ((i % width) as f32, (i / width) as f32);
        let inside = |r: &Rect| x >= r.left && x < r.right && y >= r.top && y < r.bottom;
        assert!(
            posted.iter().any(inside),
            "({x}, {y}) changed outside the damage"
        );
    }
}

#[test]
fn a_press_shows_the_keys_bubble_until_it_goes_up() {
    let mut hand = Hand::new(true);
    let id = hand.down(Q);
    assert_eq!(hand.shown(), Some(Q));
    assert!(
        hand.scene.bubble_bounds(Q).expect("bounds").top < 0.0,
        "over the app"
    );
    hand.up(id);
    assert_eq!(hand.scene.bubble, None);
    assert_eq!(hand.typed(), [Q]);
}

/// A tap paints its key as it did with no bubble, lays the bubble's image over it and puts the
/// keys back when it goes up. The next tap of the key draws no bubble.
#[test]
fn a_tap_paints_its_key_as_without_a_bubble_and_lays_the_bubble_over_it() {
    let mut hand = Hand::new(true);
    let before = hand.scene.clone();
    let id = hand.down(W);
    let pressed = hand.scene.clone();
    hand.up(id);
    let after = hand.scene.clone();
    let plain = Scene {
        bubble: None,
        ..pressed.clone()
    };
    let raster = |scene: &Scene| {
        let mut raster = Raster::new(scene.width, scene.buffer_height(), vec![font()]);
        raster.set_profiling(true);
        raster.draw(scene, scene.buffer_rect()).expect("raster");
        raster
    };
    let frame = |raster: &mut Raster, a: &Scene, b: &Scene| {
        raster
            .draw_frame(a, b, &a.damage_regions(b))
            .expect("raster");
        raster.profile()
    };
    let (mut frames, mut without) = (raster(&before), raster(&before));
    let (p, q) = (
        frame(&mut frames, &before, &pressed),
        frame(&mut without, &before, &plain),
    );
    assert_eq!(p.labels, q.labels, "the key paints as with no bubble");
    let kept = pressed.buffer_pixels(pressed.bubble_damage().expect("a bubble"));
    let area = (kept.width() * kept.height()) as u64;
    assert!(p.bubble_px > area / 2 && p.bubble_px <= area, "laid once");
    assert_eq!(p.bubble_draws, 1);
    let p = frame(&mut frames, &pressed, &after);
    assert_eq!((p.bubble_px, p.restored_px), (0, area), "put back");
    assert!(frames.pixels() == raster(&after).pixels(), "as they were");
    assert_eq!(frame(&mut frames, &after, &pressed).bubble_draws, 0, "kept");
}

#[test]
fn space_enter_shift_and_delete_show_none() {
    for key in [SHIFT_KEY, SPACE_KEY, ENTER_KEY, DELETE_KEY] {
        let mut hand = Hand::new(true);
        let id = hand.down(key);
        assert_eq!(hand.scene.bubble, None, "key {key}");
        hand.up(id);
    }
}

#[test]
fn the_setting_off_shows_none_and_each_touch_takes_its_current_value() {
    let mut hand = Hand::new(false);
    for key in [Q, W, E, R] {
        let id = hand.down(key);
        assert_eq!(hand.scene.bubble, None, "key {key}");
        hand.up(id);
    }
    // Kotlin sends the setting before every touch, so a change takes at the next event.
    hand.touch.configure(config(false, true));
    let id = hand.down(W);
    assert_eq!(hand.shown(), Some(W));
    hand.touch.configure(config(false, false));
    hand.slide(id, E);
    assert_eq!(hand.scene.bubble, None);
    hand.up(id);
    assert_eq!(hand.typed(), [Q, W, E, R, E]);
}

#[test]
fn a_slide_carries_the_bubble_to_each_key_the_finger_reaches() {
    let mut hand = Hand::new(true);
    let id = hand.down(Q);
    hand.slide(id, W);
    assert_eq!(hand.shown(), Some(W));
    hand.slide(id, SPACE_KEY);
    assert_eq!(hand.scene.bubble, None, "onto space: none");
    hand.slide(id, R);
    assert_eq!(hand.shown(), Some(R));
    hand.up(id);
    assert_eq!(hand.scene.bubble, None);
    assert_eq!(hand.typed(), [R]);
}

#[test]
fn a_long_press_popup_takes_the_bubbles_place_and_leaves_none_behind() {
    let mut hand = Hand::new(true);
    let mut frames = vec![hand.scene.clone()];
    let id = hand.down(E);
    frames.push(hand.scene.clone());
    let held = hand.touch.long_press(&hand.scene, id, true, true);
    assert!(held.iter().any(|a| a.0 == OPEN_POPUP), "{held:?}");
    open_popup(&mut hand.scene, E);
    assert_eq!(hand.shown(), None, "one box at a time");
    frames.push(hand.scene.clone());
    let out = hand.up(id);
    assert!(out.iter().any(|a| a.0 == POPUP_SELECT), "{out:?}");
    assert_eq!((&hand.scene.bubble, &hand.scene.popup), (&None, &None));
    frames.push(hand.scene.clone());
    repaint(&frames);
}

#[test]
fn a_hold_with_no_choices_keeps_the_bubble_and_leaving_a_popup_moves_it() {
    let mut hand = Hand::new(true);
    let id = hand.down(W);
    let held = hand.touch.long_press(&hand.scene, id, false, true);
    assert!(held.iter().all(|a| a.0 != OPEN_POPUP));
    assert_eq!(hand.shown(), Some(W), "no popup, so the bubble stays");
    hand.up(id);
    // Off the popup, a choice's width past its end, onto another key: the popup closes and
    // that key's bubble opens, as Compose cancels the popup and presses the key.
    let id = hand.down(Q);
    hand.touch.long_press(&hand.scene, id, true, true);
    open_popup(&mut hand.scene, Q);
    hand.slide(id, R);
    assert_eq!(hand.scene.popup, None);
    assert_eq!(hand.shown(), Some(R));
    hand.up(id);
    assert_eq!(hand.scene.bubble, None);
}

#[test]
fn rolling_fingers_move_the_bubble_to_the_newest_key_and_repaint_exactly() {
    let mut hand = Hand::new(true);
    let mut frames = vec![hand.scene.clone()];
    let q = hand.down(Q);
    frames.push(hand.scene.clone());
    // w lands while q is held: q goes up first, typed, and w's bubble replaces q's.
    let w = hand.down(W);
    assert_eq!(hand.shown(), Some(W));
    frames.push(hand.scene.clone());
    // Shift turns off now q is typed: w's label changes under the finger, and the whole
    // keyboard, bubble and all, repaints.
    hand.scene.keys[W].label = "W".into();
    frames.push(hand.scene.clone());
    // Shift held with a letter: both are down, the letter's bubble shows.
    let shift = hand.down(SHIFT_KEY);
    assert_eq!(hand.shown(), None, "w rolled up under shift");
    let e = hand.down(E);
    assert_eq!(hand.shown(), Some(E));
    frames.push(hand.scene.clone());
    hand.up(e);
    frames.push(hand.scene.clone());
    // The rolled fingers still on the glass lift last, with shift's.
    for id in [shift, q, w] {
        hand.up(id);
        frames.push(hand.scene.clone());
    }
    assert_eq!(hand.scene.bubble, None);
    assert_eq!(hand.typed(), [Q, W, E, SHIFT_KEY]);
    repaint(&frames);
}

/// A small deterministic generator, so a failure repeats.
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.0 >> 33) % n as u64) as usize
    }
}

#[test]
fn fast_typing_leaves_no_stale_bubble() {
    let mut hand = Hand::new(true);
    let mut random = Lcg(7);
    let mut frames = vec![hand.scene.clone()];
    for _ in 0..240 {
        if hand.fingers.len() < 3 && (hand.fingers.is_empty() || random.below(2) == 0) {
            hand.down(random.below(8));
        } else {
            let id = hand.fingers[random.below(hand.fingers.len())].0;
            hand.up(id);
        }
        let scene = &hand.scene;
        let held: Vec<_> = scene
            .pressed
            .iter()
            .filter(|&&k| scene.bubbles[k])
            .collect();
        assert!(
            held.len() <= 1,
            "one character key held at a time: {held:?}"
        );
        assert_eq!(
            hand.shown(),
            held.first().map(|&&k| k),
            "{:?}",
            scene.pressed
        );
        frames.push(hand.scene.clone());
    }
    while let Some(&(id, ..)) = hand.fingers.first() {
        hand.up(id);
    }
    assert_eq!(hand.scene.bubble, None);
    frames.push(hand.scene.clone());
    repaint(&frames);
}

#[test]
fn a_cancel_or_a_glide_takes_the_bubble_away() {
    let mut hand = Hand::new(true);
    let id = hand.down(Q);
    hand.event(3, id);
    assert_eq!(hand.scene.bubble, None, "cancelled");
    let mut hand = Hand::new(true);
    hand.down(W);
    hand.touch.cancel(&mut hand.scene);
    assert_eq!(hand.scene.bubble, None, "detached");
    let mut hand = Hand::with(true, true);
    let id = hand.down(Q);
    assert_eq!(hand.shown(), Some(Q));
    for x in [55.0, 85.0, 115.0, 145.0, 175.0, 205.0] {
        hand.fingers[0].1 = x;
        hand.event(2, id);
    }
    assert!(hand.actions.iter().any(|a| a.0 == GLIDE_POINT), "a glide");
    assert_eq!(hand.scene.bubble, None, "a glide shows no bubble");
    hand.up(id);
    assert_eq!(hand.scene.bubble, None);
}

#[test]
fn an_accessibility_click_types_without_a_bubble() {
    let mut hand = Hand::new(true);
    let mut semantics = Semantics::default();
    let nodes = semantics.snapshot(&hand.scene, "unshifted");
    assert!(!nodes.is_empty());
    for node in nodes {
        let out = semantics.activate(&mut hand.scene, "unshifted", node.id, ACCESS_CLICK);
        assert!(out.iter().any(|a| a.0 == KEY_DOWN), "{}", node.label);
        assert_eq!(hand.scene.bubble, None, "{}", node.label);
    }
}
