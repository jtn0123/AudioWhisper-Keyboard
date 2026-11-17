//! Damage no wider than what a change can paint. The theme sets a popup's styles as the popup
//! opens; they paint only inside the popup, so they repaint the popup, not the whole keyboard.
//! A press that only recolours a key's box repaints the box, not its shadow.
use keys::{Bounds, Rect};
use render::{
    raster::Raster,
    scene::{Key, Popup, Scene},
    style::Style,
};

const FULL: Rect = Rect {
    left: 0.0,
    top: 0.0,
    right: 150.0,
    bottom: 100.0,
};

fn scene() -> Scene {
    let keys = (0..3)
        .map(|i| {
            let r = Rect {
                left: i as f32 * 50.0,
                right: (i + 1) as f32 * 50.0,
                top: 10.0,
                bottom: 70.0,
            };
            let style = |background| Style {
                size: 20.0,
                background,
                foreground: 0xff000000,
                shadow: 1.5,
                corners: [4.0; 4],
                ..Style::default()
            };
            Key {
                code: 97 + i,
                bounds: Bounds {
                    touch: r,
                    visible: r,
                },
                label: char::from(b'a' + i as u8).to_string(),
                hint: String::new(),
                icon: String::new(),
                styles: [style(0xfff0f0f0), style(0xffc0c0c0)],
                hints: std::array::from_fn(|_| Style::default()),
                enabled: true,
            }
        })
        .collect();
    Scene {
        width: 150,
        height: 100,
        strip: 30.0,
        keys,
        background: 0xffffffff,
        ..Scene::default()
    }
}

fn styled_popup(closed: &Scene) -> Scene {
    let choice = |background| Style {
        size: 18.0,
        background,
        foreground: 0xff202124,
        ..Style::default()
    };
    Scene {
        popup_style: Style {
            background: 0xff445566,
            corners: [6.0; 4],
            ..Style::default()
        },
        popup_styles: vec![[choice(0xffaabbcc), choice(0xff8899aa)]; 2],
        pressed: vec![1],
        popup: Some(Popup {
            key: 1,
            labels: vec!["b".into(), "B".into()],
            selected: 0,
        }),
        ..closed.clone()
    }
}

#[test]
fn opening_a_styled_popup_repaints_the_popup_and_its_key() {
    let closed = scene();
    let open = styled_popup(&closed);
    let regions = closed.damage_regions(&open);
    assert!(!regions.contains(&FULL), "{regions:?}");
    assert!(regions.contains(&open.popup_bounds().expect("popup")));
    let restyled = Scene {
        popup_style: open.popup_style.clone(),
        popup_styles: open.popup_styles.clone(),
        ..closed.clone()
    };
    assert!(
        closed.damage_regions(&restyled).is_empty(),
        "no popup shows"
    );
    let font = include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec();
    let mut raster = Raster::new(150, 100, vec![font.clone()]);
    let mut reference = Raster::new(150, 100, vec![font]);
    raster.draw(&closed, FULL).expect("raster");
    for (from, to) in [(&closed, &open), (&open, &closed)] {
        raster
            .draw_regions(to, &from.damage_regions(to))
            .expect("raster");
        reference.draw(to, FULL).expect("raster");
        assert!(
            raster.pixels() == reference.pixels(),
            "dirty popup differs from full repaint"
        );
    }
}

/// Repaint `frames` in turn through their damage and check each against a full repaint.
fn assert_dirty_matches_full(frames: &[Scene]) {
    let font = include_bytes!("../../../../../fastlane/staging/fonts/Roboto-Light.ttf").to_vec();
    let mut raster = Raster::new(150, 100, vec![font.clone()]);
    let mut reference = Raster::new(150, 100, vec![font]);
    raster.draw(&frames[0], FULL).expect("raster");
    for pair in frames.windows(2) {
        raster
            .draw_regions(&pair[1], &pair[0].damage_regions(&pair[1]))
            .expect("raster");
        reference.draw(&pair[1], FULL).expect("raster");
        let first = (raster.pixels().as_chunks::<4>().0.iter())
            .zip(reference.pixels().as_chunks::<4>().0)
            .position(|(a, b)| a != b);
        assert_eq!(first, None, "pressed {:?}", pair[1].pressed);
    }
}

#[test]
fn a_press_that_only_recolours_a_key_repaints_only_its_box() {
    let mut scene = scene();
    for key in &mut scene.keys {
        key.bounds.visible.left += 0.25;
        key.bounds.visible.right -= 0.5;
        key.bounds.visible.top += 0.75;
        key.hint = "1".into();
        key.hints = std::array::from_fn(|_| Style {
            size: 9.0,
            foreground: 0xff5f5f5f,
            ..Style::default()
        });
        for style in &mut key.styles {
            style.margin = [3.0, 5.0, 3.0, 5.0];
            style.shadow = 2.625;
        }
    }
    let pressed = Scene {
        pressed: vec![1],
        ..scene.clone()
    };
    // Key 1's cell is x 50..99 and y 40..99 once rounded; its box sits inside the margins.
    let regions = scene.damage_regions(&pressed);
    let r#box = Rect {
        left: 53.0,
        top: 45.0,
        right: 96.0,
        bottom: 94.0,
    };
    assert_eq!(regions, [r#box]);
    let both = Scene {
        pressed: vec![0, 2],
        ..scene.clone()
    };
    assert_dirty_matches_full(&[scene.clone(), pressed, both, scene.clone()]);
    // Anything else a press changes still repaints six elevations around the key.
    let restyle: [fn(&mut Style); 3] = [
        |s| s.foreground = 0xff0000ff,
        |s| s.shadow = 4.0,
        |s| s.corners = [9.0; 4],
    ];
    for change in restyle {
        let mut changed = scene.clone();
        change(&mut changed.keys[1].styles[1]);
        let pressed = Scene {
            pressed: vec![1],
            ..changed.clone()
        };
        let wide = changed.damage_regions(&pressed);
        assert_eq!(wide.len(), 1);
        assert!(wide[0].left < 50.0 && wide[0].right > 99.0, "{wide:?}");
        assert_dirty_matches_full(&[changed.clone(), pressed, changed]);
    }
}
