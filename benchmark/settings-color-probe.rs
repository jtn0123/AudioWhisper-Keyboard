//! Supply public Android system-font files. No labels or editable data are printed.
use render::{
    panels::{Command, Item, Panel, rect},
    raster::Raster,
    scene::Scene,
};
fn main() {
    let fonts: Vec<_> = std::env::args()
        .skip(1)
        .map(|p| std::fs::read(p).unwrap())
        .collect();
    assert!(!fonts.is_empty());
    let mut item = Item::new(
        1,
        rect(0.0, 0.0, 800.0, 120.0),
        "🚀 🇺🇸 👨‍👩‍👧‍👦 👍🏽".into(),
        "settings-subheader",
        Command::None,
    );
    item.text.size = 48.0;
    let mut scene = Scene {
        width: 800,
        height: 120,
        background: 0xffffffff,
        panel: Some(Panel {
            kind: 9,
            items: vec![item],
            ..Default::default()
        }),
        ..Default::default()
    };
    let damage = rect(0.0, 0.0, 800.0, 120.0);
    let mut direct = Raster::new(800, 120, fonts.clone());
    direct.draw(&scene, damage).unwrap();
    scene.panel.as_mut().unwrap().kind = 8;
    let mut cached = Raster::new(800, 120, fonts);
    cached.draw(&scene, damage).unwrap();
    let (a, b) = (direct.pixels(), cached.pixels());
    let mut different = 0;
    let mut flat_differences = 0;
    let mut painted = 0;
    for y in 1..119usize {
        for x in 1..799usize {
            let at = (y * 800 + x) * 4;
            painted += usize::from(a[at..at + 3].iter().any(|v| *v < 245));
            let differs = (0..4).any(|c| a[at + c].abs_diff(b[at + c]) > 3);
            different += usize::from(differs);
            // Same 1px glyph-edge mask and 3/255 channel threshold as the picture comparisons.
            let edge = (-1isize..=1).any(|dy| {
                (-1isize..=1).any(|dx| {
                    let n = ((y as isize + dy) as usize * 800 + (x as isize + dx) as usize) * 4;
                    (0..4).any(|c| {
                        a[at + c].abs_diff(a[n + c]) > 3 || b[at + c].abs_diff(b[n + c]) > 3
                    })
                })
            });
            flat_differences += usize::from(differs && !edge);
        }
    }
    assert!(painted > 1000);
    assert_eq!(flat_differences, 0);
    println!(
        "{{\"painted_pixels\":{painted},\"different_pixels\":{different},\"flat_differences_above_3\":{flat_differences}}}"
    );
}
