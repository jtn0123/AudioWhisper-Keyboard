//! Where a run of glyphs can paint, so a frame skips the runs its damage cannot show. Every glyph
//! outline lies inside its font's bounds (`head`), placed where the glyph is. A variable font's
//! other instances and hinting move outlines a little past those bounds, which a quarter em and a
//! pixel cover. Colour glyphs, whose paint need not keep to the outline bounds, always draw.
use keys::Rect;
use parley::GlyphRun;
use read_fonts::{FontRef, TableProvider};

/// Whether `run`, drawn at `origin` and squeezed across by `squeeze`, can paint inside `damage`.
pub(crate) fn may_paint(
    run: &GlyphRun<'_, ()>,
    origin: (f64, f64),
    squeeze: f64,
    damage: Rect,
) -> bool {
    let data = run.run().font();
    let Ok(font) = FontRef::from_index(data.data.as_ref(), data.index) else {
        return true;
    };
    let Ok(head) = font.head() else {
        return true;
    };
    if font.colr().is_ok() || font.cbdt().is_ok() || font.sbix().is_ok() {
        return true;
    }
    let mut glyphs = run.positioned_glyphs();
    let Some(first) = glyphs.next() else {
        return false;
    };
    let [mut x0, mut y0] = [first.x, first.y];
    let [mut x1, mut y1] = [x0, y0];
    for glyph in glyphs {
        [x0, y0] = [x0.min(glyph.x), y0.min(glyph.y)];
        [x1, y1] = [x1.max(glyph.x), y1.max(glyph.y)];
    }
    let em = f64::from(run.run().font_size());
    let unit = em / f64::from(head.units_per_em().max(1));
    let pad = em / 4.0;
    // Font units point up; the screen's y points down.
    let left = squeeze * (f64::from(x0) + f64::from(head.x_min()) * unit - pad);
    let right = squeeze * (f64::from(x1) + f64::from(head.x_max()) * unit + pad);
    let top = f64::from(y0) - f64::from(head.y_max()) * unit - pad;
    let bottom = f64::from(y1) - f64::from(head.y_min()) * unit + pad;
    origin.0 + left - 1.0 < f64::from(damage.right)
        && origin.0 + right + 1.0 > f64::from(damage.left)
        && origin.1 + top - 1.0 < f64::from(damage.bottom)
        && origin.1 + bottom + 1.0 > f64::from(damage.top)
}
