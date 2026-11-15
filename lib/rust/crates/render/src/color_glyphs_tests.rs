//! Colour-glyph tiles against vello_cpu's direct drawing, with Noto Color Emoji subsets
//! (COLRv1 and CBDT; SIL OFL 1.1, see tests/fixtures/NOTO-EMOJI-OFL.txt).
use super::*;
use skrifa::MetadataProvider;

const COLR: &[u8] = include_bytes!("../tests/fixtures/noto-color-emoji-subset.ttf");
const CBDT: &[u8] = include_bytes!("../tests/fixtures/noto-color-emoji-flags-subset.ttf");
const W: u16 = 320;
const H: u16 = 120;

fn font(bytes: &[u8]) -> parley::FontData {
    parley::FontData::new(crate::text::FontBlob::from(bytes.to_vec()), 0)
}

fn glyph(bytes: &[u8], c: char) -> u32 {
    let id = skrifa::FontRef::new(bytes)
        .unwrap()
        .charmap()
        .map(c)
        .unwrap();
    id.to_u32()
}

/// Every case: the COLR face (smile, rocket, wave, thumbs up) and the CBDT face (a regional
/// indicator), at sizes from a candidate label to a large popup.
fn cases() -> Vec<(parley::FontData, f32, Vec<u32>)> {
    let colr: Vec<u32> = ['😀', '🚀', '👋', '👍']
        .iter()
        .map(|&c| glyph(COLR, c))
        .collect();
    let cbdt = vec![glyph(CBDT, '🇺'), glyph(CBDT, '🇸')];
    let mut out = Vec::new();
    for size in [20.0, 47.25, 63.0] {
        out.push((font(COLR), size, colr.clone()));
        out.push((font(CBDT), size, cbdt.clone()));
    }
    out
}

/// Light and dark keyboard backgrounds.
const BACKGROUNDS: [[f32; 4]; 2] = [[1.0, 1.0, 1.0, 1.0], [0.11, 0.11, 0.12, 1.0]];

fn blank(background: [f32; 4]) -> (RenderContext, Resources) {
    let mut ctx = RenderContext::new(W, H);
    ctx.set_paint(AlphaColor::<Srgb>::new(background));
    ctx.fill_rect(&Rect::new(0.0, 0.0, f64::from(W), f64::from(H)));
    (ctx, Resources::new())
}

fn finish(ctx: RenderContext, mut resources: Resources) -> Vec<u8> {
    let mut pixmap = Pixmap::new(W, H);
    ctx.render_with(
        &mut pixmap,
        &mut resources,
        RasterizerSettings {
            target_init: TargetInit::SrcOver,
            ..Default::default()
        },
    );
    pixmap.data_as_u8_slice().to_vec()
}

/// One row of glyphs, 70 px apart from `x`, on the baseline `y`, in the given paint.
fn row(
    cache: Option<&mut ColorGlyphs>,
    case: &(parley::FontData, f32, Vec<u32>),
    (x, y): (f32, f32),
    color: AlphaColor<Srgb>,
    background: [f32; 4],
) -> Vec<u8> {
    let (mut ctx, mut resources) = blank(background);
    ctx.set_transform(Affine::translate((f64::from(x), f64::from(y))));
    ctx.set_paint(color);
    let (font, size, ids) = case;
    let glyphs = ids
        .iter()
        .enumerate()
        .map(|(i, &id)| (id, i as f32 * 70.0, 0.0));
    match cache {
        Some(cache) => cache
            .draw(&mut ctx, &mut resources, font, *size, &[], glyphs)
            .unwrap(),
        None => direct(&mut ctx, &mut resources, font, *size, &[], glyphs).unwrap(),
    }
    finish(ctx, resources)
}

fn max_difference(a: &[u8], b: &[u8]) -> u8 {
    a.iter().zip(b).map(|(x, y)| x.abs_diff(*y)).max().unwrap()
}

fn inked(pixels: &[u8]) -> usize {
    pixels.chunks(4).filter(|p| p[..3] != pixels[..3]).count()
}

const BLACK: AlphaColor<Srgb> = AlphaColor::new([0.0, 0.0, 0.0, 1.0]);

#[test]
fn tiles_match_direct_drawing_on_whole_pixels_and_hit_on_the_next_frame() {
    for case in cases() {
        for (color, bg) in [BLACK, AlphaColor::new([0.2, 0.4, 0.8, 0.6])]
            .into_iter()
            .flat_map(|c| BACKGROUNDS.map(|bg| (c, bg)))
        {
            let mut cache = ColorGlyphs::default();
            let expected = row(None, &case, (10.0, 80.0), color, bg);
            assert!(
                inked(&expected) > 100,
                "the fixture glyphs draw at {}",
                case.1
            );
            let first = row(Some(&mut cache), &case, (10.0, 80.0), color, bg);
            // Measured: COLR 3/255 on white and 2/255 on the dark background (layers composite
            // through an 8-bit tile, as they did through the atlas); CBDT is exact.
            let worst = max_difference(&expected, &first);
            assert!(
                worst <= 3,
                "size {}: a channel differs by {worst}/255",
                case.1
            );
            let rendered = cache.rendered;
            assert_eq!(rendered, case.2.len() as u64);
            cache.end_frame();
            assert_eq!(row(Some(&mut cache), &case, (10.0, 80.0), color, bg), first);
            assert_eq!(
                cache.rendered, rendered,
                "the second frame reuses every tile"
            );
        }
    }
}

#[test]
fn fractional_origins_snap_to_whole_pixels_without_new_tiles() {
    for case in cases() {
        let mut cache = ColorGlyphs::default();
        let white = BACKGROUNDS[0];
        let snapped = row(Some(&mut cache), &case, (10.0, 81.0), BLACK, white);
        let rendered = cache.rendered;
        // 10.3 rounds to 10 and 80.6 to 81: the same tiles, on the same pixels.
        assert_eq!(
            row(Some(&mut cache), &case, (10.3, 80.6), BLACK, white),
            snapped
        );
        assert_eq!(cache.rendered, rendered);
        let direct = row(None, &case, (10.0, 81.0), BLACK, white);
        assert!(max_difference(&direct, &snapped) <= 3);
    }
}

#[test]
fn scaled_text_draws_directly() {
    let case = &cases()[0];
    let mut cache = ColorGlyphs::default();
    let draw = |cache: Option<&mut ColorGlyphs>| {
        let (mut ctx, mut resources) = blank(BACKGROUNDS[0]);
        ctx.set_transform(Affine::translate((10.0, 60.0)) * Affine::scale(1.5));
        ctx.set_paint(BLACK);
        let glyphs = case
            .2
            .iter()
            .enumerate()
            .map(|(i, &id)| (id, i as f32 * 40.0, 0.0));
        match cache {
            Some(cache) => cache
                .draw(&mut ctx, &mut resources, &case.0, case.1, &[], glyphs)
                .unwrap(),
            None => direct(&mut ctx, &mut resources, &case.0, case.1, &[], glyphs).unwrap(),
        }
        finish(ctx, resources)
    };
    assert_eq!(draw(Some(&mut cache)), draw(None));
    assert_eq!(cache.rendered, 0);
}

#[test]
fn the_budget_keeps_what_the_frame_drew_and_drops_the_rest_oldest_first() {
    let font = font(COLR);
    let id = glyph(COLR, '😀');
    let mut cache = ColorGlyphs::default();
    let (mut ctx, mut resources) = blank(BACKGROUNDS[0]);
    ctx.set_paint(BLACK);
    // Fifty large sizes in one frame: well over budget, yet all of it is on screen.
    for size in (0..50).map(|i| 150.0 + i as f32) {
        cache
            .draw(
                &mut ctx,
                &mut resources,
                &font,
                size,
                &[],
                std::iter::once((id, 0.0, 300.0)),
            )
            .unwrap();
    }
    let drawn = cache.bytes();
    assert!(drawn > BUDGET_BYTES);
    cache.end_frame();
    assert_eq!(
        cache.bytes(),
        drawn,
        "nothing drawn in the frame just finished is dropped"
    );
    let small = |cache: &mut ColorGlyphs, ctx: &mut RenderContext, resources: &mut Resources| {
        cache
            .draw(
                ctx,
                resources,
                &font,
                40.0,
                &[],
                std::iter::once((id, 0.0, 60.0)),
            )
            .unwrap();
    };
    small(&mut cache, &mut ctx, &mut resources);
    cache.end_frame();
    assert!(cache.bytes() <= BUDGET_BYTES);
    let rendered = cache.rendered;
    small(&mut cache, &mut ctx, &mut resources);
    assert_eq!(
        cache.rendered, rendered,
        "the glyph still on screen kept its tile"
    );
}
