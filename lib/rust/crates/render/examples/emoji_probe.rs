//! Run with an output directory and system font paths; no fonts are checked into the repository.
use parley::PositionedLayoutItem;
use read_fonts::{FontRef, TableProvider};
use render::text::Text;
use vello_cpu::{Glyph, Pixmap, RenderContext, Resources};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let output =
        std::path::PathBuf::from(args.next().ok_or("usage: emoji_probe OUTPUT_DIR FONT...")?);
    std::fs::create_dir_all(&output)?;
    let paths: Vec<_> = args.collect();
    let mut fonts = Vec::new();
    for path in paths {
        let bytes = std::fs::read(&path)?;
        let font = FontRef::new(&bytes)?;
        println!(
            "{path}: CBDT={} CBLC={} COLR={:?} sbix={} SVG={}",
            font.cbdt().is_ok(),
            font.cblc().is_ok(),
            font.colr().ok().map(|colr| colr.version()),
            font.sbix().is_ok(),
            font.svg().is_ok()
        );
        fonts.push(bytes);
    }
    let mut text = Text::new(fonts);
    let mut resources = Resources::new();
    for (name, sample) in [
        ("family", "👨‍👩‍👧‍👦"),
        ("flag", "🇺🇸"),
        ("tone", "👍🏽"),
        ("keycap", "1️⃣"),
        ("VS16", "❤️"),
        ("VS15", "❤︎"),
    ] {
        let layout = text.layout(sample, 64.0);
        let mut context = RenderContext::new(256, 128);
        let mut glyph_ids = Vec::new();
        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(run) = item {
                    let font = run.run().font();
                    let glyphs: Vec<_> = run
                        .positioned_glyphs()
                        .map(|g| {
                            glyph_ids.push(g.id);
                            Glyph {
                                id: g.id,
                                x: g.x + 8.0,
                                y: g.y + 8.0,
                            }
                        })
                        .collect();
                    context
                        .glyph_run(&mut resources, font)
                        .font_size(run.run().font_size())
                        .fill_glyphs(glyphs.into_iter())?;
                }
            }
        }
        let mut pixmap = Pixmap::new(256, 128);
        context.render(&mut pixmap, &mut resources);
        let colored = pixmap
            .data_as_u8_slice()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > 0 && (p[0] != p[1] || p[1] != p[2]))
            .count();
        let covered = pixmap
            .data_as_u8_slice()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] != 0)
            .count();
        println!("{name}: glyphs={glyph_ids:?} covered_pixels={covered} colored_pixels={colored}");
        if glyph_ids.len() != 1 || glyph_ids.contains(&0) || covered == 0 {
            return Err(format!("{name}: sequence did not render as one nonempty glyph").into());
        }
        if name != "family" && name != "VS15" && colored == 0 {
            return Err(format!("{name}: sequence did not render as one colour glyph").into());
        }
        std::fs::write(output.join(format!("{name}.png")), pixmap.into_png()?)?;
    }
    Ok(())
}
