//! ThirdPartyLicensesScreen timing on the host: opening the screen (parse, measure every row,
//! lay out and rasterise the first page on the CPU), one scroll frame (scene and raster), and
//! opening a licence dialog. Prints numbers only.
//!
//! cargo run --release -p render --example settings_licences_timing -- <aboutlibraries.json> <font.ttf>...
use keys::Rect;
use render::{
    raster::Raster,
    settings_ui::{
        App,
        licences::{LIBRARIES, ROUTE, ROW_ID},
    },
    text::Text,
};
use std::time::Instant;

fn ms(since: Instant) -> f64 {
    since.elapsed().as_secs_f64() * 1000.0
}
/// The scene for the current state, rasterised whole on the CPU as the settings surface is.
fn frame(a: &mut App, text: &mut Text, raster: &mut Raster) {
    let scene = a.scene(text);
    let whole = Rect {
        left: 0.0,
        top: 0.0,
        right: scene.width as f32,
        bottom: scene.height as f32,
    };
    raster.draw(&scene, whole).expect("raster");
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let json = std::fs::read_to_string(&args[0]).expect("aboutlibraries.json");
    let fonts: Vec<_> = args[1..]
        .iter()
        .map(|p| std::fs::read(p).expect("font"))
        .collect();
    // A 1080 by 2400 phone at 2.625 density and the default font scale.
    let mut open = vec![];
    let mut frames = vec![];
    let mut dialogs = vec![];
    for _ in 0..20 {
        let mut text = Text::new(fonts.clone());
        let mut raster = Raster::new(1080, 2400, fonts.clone());
        let mut a = App {
            config: [1080.0, 2400.0, 2.625, 1.0],
            ..App::default()
        };
        a.navigate("settings/about");
        a.navigate(ROUTE);
        frame(&mut a, &mut text, &mut raster);
        let start = Instant::now();
        a.set_document(LIBRARIES, &json);
        frame(&mut a, &mut text, &mut raster);
        open.push(ms(start));
        // A fling's worth of frames, 40 px apart, through the whole list.
        while a.scroll() < a.max_scroll {
            let next = a.scroll() + 40.0;
            a.set_scroll(next);
            let start = Instant::now();
            frame(&mut a, &mut text, &mut raster);
            frames.push(ms(start));
        }
        a.set_scroll(0.0);
        frame(&mut a, &mut text, &mut raster);
        let start = Instant::now();
        a.activate(ROW_ID);
        frame(&mut a, &mut text, &mut raster);
        dialogs.push(ms(start));
    }
    for (name, mut v) in [
        ("open", open),
        ("scroll frame", frames),
        ("dialog", dialogs),
    ] {
        v.sort_by(f64::total_cmp);
        let median = v[v.len() / 2];
        let p95 = v[v.len() * 95 / 100];
        println!(
            "{name}: median {median:.3} ms, p95 {p95:.3} ms, max {:.3} ms over {}",
            v[v.len() - 1],
            v.len()
        );
    }
}
