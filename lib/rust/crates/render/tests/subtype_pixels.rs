use keys::Bounds;
use render::{
    panels::{Command, Item, Panel, rect},
    raster::Raster,
    scene::{Key, Scene},
    style::Style,
};
#[test]
fn modal_keeps_bottom_keyboard_and_transparent_scrim_repaints_do_not_accumulate() {
    let mut p = Panel {
        kind: 7,
        ..Default::default()
    };
    let mut scrim = Item::new(
        -70,
        rect(0.0, 0.0, 100.0, 100.0),
        "Dismiss".into(),
        "subtype-scrim",
        Command::Subtype(None),
    );
    scrim.styles = [
        Style {
            background: 0x52000000,
            ..Default::default()
        },
        Style {
            background: 0x52000000,
            ..Default::default()
        },
    ];
    p.items.push(scrim);
    let bounds = rect(0.0, 0.0, 100.0, 40.0);
    let style = Style {
        background: 0xffff0000,
        ..Default::default()
    };
    let scene = Scene {
        width: 100,
        height: 100,
        strip: 0.0,
        keyboard_height: 40.0,
        background: 0xffffffff,
        panel: Some(p),
        keys: vec![Key {
            code: 97,
            bounds: Bounds {
                touch: bounds,
                visible: bounds,
            },
            label: String::new(),
            hint: String::new(),
            icon: String::new(),
            styles: [style.clone(), style],
            hints: Default::default(),
            enabled: true,
        }],
        ..Default::default()
    };
    let mut raster = Raster::new(100, 100, vec![]);
    let full = rect(0.0, 0.0, 100.0, 100.0);
    raster.draw(&scene, full).unwrap();
    let first = raster.pixels().to_vec();
    assert_eq!(&first[0..4], &[0, 0, 0, 82]);
    let pixel = (80 * 100 + 50) * 4;
    assert_eq!(&first[pixel..pixel + 4], &[173, 0, 0, 255]);
    raster.draw(&scene, full).unwrap();
    assert_eq!(first, raster.pixels());
}
