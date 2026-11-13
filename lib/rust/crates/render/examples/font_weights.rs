//! Inspect the supplied system face without bundling fonts or synthesizing emboldening.
use parley::PositionedLayoutItem;
fn main() {
    let fonts = std::env::args()
        .skip(1)
        .map(|p| std::fs::read(p).expect("font read"))
        .collect::<Vec<_>>();
    assert!(
        !fonts.is_empty(),
        "supply Android SystemFonts paths copied for offline inspection"
    );
    let mut text = render::text::Text::new(fonts);
    let mut instances = Vec::new();
    for weight in (100..=900).step_by(100) {
        let layout = text.layout_weight("Weight", 36.0, false, weight);
        let run = layout
            .lines()
            .flat_map(|line| line.items())
            .find_map(|item| match item {
                PositionedLayoutItem::GlyphRun(run) => Some(run),
                _ => None,
            })
            .expect("glyph run");
        let coords = run
            .run()
            .normalized_coords()
            .iter()
            .map(|c| c.to_bits())
            .collect::<Vec<_>>();
        println!(
            "weight={weight} font_index={} coordinates={coords:?} width={}",
            run.run().font().index,
            layout.width()
        );
        instances.push(coords);
    }
    assert_ne!(
        instances[3], instances[6],
        "400 and 700 must select different real font instances"
    );
}
