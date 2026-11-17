//! Key shadows. Vello works out a Gaussian for every pixel a blurred rectangle paints, and that
//! was nearly all of the CPU raster's time: about 10 ms of a full keyboard redraw, and most of a
//! key press. An opaque box drawn straight after its shadow hides most of it, so those pixels are
//! left out. Each pixel still painted gets the same lobes, worked out the same way, in the same
//! order; each one left out ends as the box's colour either way. The drawing is unchanged to the
//! bit.
use crate::{
    raster::{Raster, color},
    style::Style,
};
use keys::Rect;
use vello_cpu::kurbo::{self, BezPath};

/// Android elevation has a centred ambient lobe and an offset spot lobe: (colour, how far down
/// and how blurred, both in elevations). Values are physical pixels; damage reserves six
/// elevations around the box.
const LOBES: [(u32, f32, f32); 2] = [(0x10000000, 0.0, 0.30), (0x30000000, 1.20, 1.00)];

impl Raster {
    pub(crate) fn shadow(&mut self, r: Rect, style: &Style) {
        self.lobes(r, style, None);
    }

    /// `box_at` over its shadow, leaving out the shadow the box hides.
    pub(crate) fn shadowed_box(&mut self, r: Rect, style: &Style) {
        let hidden = (style.background >> 24 == 0xff).then(|| Cover::new(r, style));
        self.lobes(r, style, hidden);
        self.box_at(r, style);
    }

    fn lobes(&mut self, r: Rect, style: &Style, hidden: Option<Cover>) {
        if style.shadow <= 0.0 {
            return;
        }
        let radius = style.radii(r.width(), r.height())[0];
        for (argb, offset, blur) in LOBES {
            self.context.set_paint(color(argb));
            let y = style.shadow * offset;
            let rect = kurbo::Rect::new(
                r.left as f64,
                (r.top + y) as f64,
                r.right as f64,
                (r.bottom + y) as f64,
            );
            let sigma = style.shadow * blur;
            match hidden.and_then(|h| rim(rect, sigma, &h)) {
                Some(path) => self
                    .context
                    .fill_blurred_rounded_rect_in(&path, &rect, radius, sigma),
                None => self.context.fill_blurred_rounded_rect(&rect, radius, sigma),
            }
        }
    }
}

/// The box `box_at` draws: its rectangle and corners, the corners cut straight or rounded.
#[derive(Clone, Copy, Debug)]
struct Cover {
    r: Rect,
    /// Top-left, top-right, bottom-right, bottom-left, as `box_at` takes them.
    radii: [f32; 4],
    cut: bool,
}

/// How far inside a corner's outline a pixel must lie to count as painted solid. kurbo turns the
/// arc into curves to within 0.1 px and Vello flattens those to within 0.25 px, so the drawn
/// corner can sit up to 0.35 px inside the true one. Straight edges are drawn exactly.
const SLACK: f64 = 1.0;

impl Cover {
    fn new(r: Rect, style: &Style) -> Self {
        Self {
            r,
            radii: style.radii(r.width(), r.height()),
            cut: style.shape == snygg::value::ShapeKind::Cut,
        }
    }

    /// How far in from its left and right edges the box is solid on every row from `top` to
    /// `bottom`, a band inside it. Any corner shape stays inside its `radii` square.
    fn insets(&self, top: f64, bottom: f64) -> (f64, f64) {
        let [tl, tr, br, bl] = self.radii.map(f64::from);
        let (t, b) = (f64::from(self.r.top), f64::from(self.r.bottom));
        // How far a corner of size `c` reaches in on the band's row nearest its edge, `dy`
        // beyond the corner's centre: a cut is a 45° line, a rounded corner a quarter circle.
        let reach = |c: f64, dy: f64| match () {
            _ if dy <= 0.0 => 0.0,
            _ if self.cut => dy + SLACK,
            _ if dy >= c - SLACK => c,
            _ => c - ((c - SLACK).powi(2) - dy * dy).sqrt(),
        };
        (
            reach(tl, t + tl - top).max(reach(bl, bottom - (b - bl))),
            reach(tr, t + tr - top).max(reach(br, bottom - (b - br))),
        )
    }
}

/// The rectangle Vello paints a blurred lobe into, traced exactly as Vello traces it, with the
/// pixels `cover` paints solid cut out. `None` when nothing is cut out.
///
/// Vello paints in 4×4 tiles, and composites a tile an edge crosses a little differently from a
/// solid run, so the hole keeps to whole tiles and every tile outside it renders as before: one
/// span of solid tiles per row of tiles, rows of equal spans merged. An edge on a tile boundary
/// is filed under the tile to its right, so the hole's right edges sit just inside it: the
/// hidden pixel they graze is painted over by the box.
fn rim(rect: kurbo::Rect, sigma: f32, cover: &Cover) -> Option<BezPath> {
    let up = |v: f64| (v / 4.0).ceil() * 4.0;
    let down = |v: f64| (v / 4.0).floor() * 4.0;
    let kernel = f64::from(2.5 * sigma);
    let outer = rect.abs().inflate(kernel, kernel);
    let r = cover.r;
    let bottom = down(f64::from(r.bottom).min(outer.y1));
    let mut path: Option<BezPath> = None;
    let mut hole = |[left, top, right, bottom]: [f64; 4]| {
        let path = path.get_or_insert_with(|| {
            let mut path = BezPath::new();
            path.move_to((outer.x0, outer.y0));
            path.line_to((outer.x1, outer.y0));
            path.line_to((outer.x1, outer.y1));
            path.line_to((outer.x0, outer.y1));
            path.close_path();
            path
        });
        // Wound against the outline, so the hole's winding is zero.
        let right = right - 1.0 / 16.0;
        path.move_to((left, top));
        path.line_to((left, bottom));
        path.line_to((right, bottom));
        path.line_to((right, top));
        path.close_path();
    };
    let mut span: Option<[f64; 4]> = None;
    let mut y = up(f64::from(r.top).max(outer.y0));
    while y + 4.0 <= bottom {
        let (left, right) = cover.insets(y, y + 4.0);
        let left = up((f64::from(r.left) + left).max(outer.x0));
        let right = down((f64::from(r.right) - right).min(outer.x1));
        match span {
            Some(ref mut s) if s[0] == left && s[2] == right => s[3] = y + 4.0,
            _ => {
                span.take().map(&mut hole);
                span = (left < right).then_some([left, y, right, y + 4.0]);
            }
        }
        y += 4.0;
    }
    span.map(&mut hole);
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use snygg::value::ShapeKind;

    const SIZE: (u16, u16) = (160, 184);

    fn paint(r: Rect, style: &Style, rim: bool) -> Vec<u8> {
        let mut raster = Raster::new(SIZE.0, SIZE.1, Vec::new());
        raster.context.reset();
        // A neighbour's box and shadow under this one, so the shadow blends over something.
        let neighbour = Rect {
            left: r.left - 30.0,
            top: r.top + 12.0,
            right: r.left + 6.0,
            bottom: r.bottom - 4.0,
        };
        raster.shadow(neighbour, style);
        let blue = Style {
            background: 0xff3366aa,
            ..style.clone()
        };
        raster.box_at(neighbour, &blue);
        if rim {
            raster.shadowed_box(r, style);
        } else {
            raster.shadow(r, style);
            raster.box_at(r, style);
        }
        let full = Rect {
            left: 0.0,
            top: 0.0,
            right: f32::from(SIZE.0),
            bottom: f32::from(SIZE.1),
        };
        raster.rasterize(full).expect("raster");
        raster.pixels().to_vec()
    }

    /// How many pixels differ, and the first difference: where, and the two values.
    fn differences(a: &[u8], b: &[u8]) -> (usize, Option<String>) {
        let (a, b) = (a.as_chunks::<4>().0, b.as_chunks::<4>().0);
        let mut found = (0..a.len()).filter(|&i| a[i] != b[i]);
        let width = usize::from(SIZE.0);
        let first = found
            .next()
            .map(|i| format!("({}, {}) {:?} {:?}", i % width, i / width, a[i], b[i]));
        (usize::from(first.is_some()) + found.count(), first)
    }

    #[test]
    fn leaving_out_the_hidden_shadow_changes_no_pixel() {
        let shapes = [
            (ShapeKind::Rectangle, [15.75; 4], false),
            (ShapeKind::Rectangle, [3.0, 9.0, 0.0, 22.5], false),
            (ShapeKind::Rectangle, [50.0; 4], true),
            (ShapeKind::Cut, [7.5; 4], false),
            (ShapeKind::Circle, [0.0; 4], false),
        ];
        let places = [(0.0, 0.0), (0.875, 0.375), (1.5, 2.5), (3.125, 3.75)];
        // A letter key at density 2.625 among them.
        let sizes = [(92.25, 134.75), (37.25, 44.5), (9.0, 61.0), (60.5, 20.25)];
        let mut cut = 0;
        for (shape, corners, percent) in shapes {
            // The default theme's 2dp at density 2.625 first.
            let elevations = [
                (5.25, 0xffffffff),
                (2.625, 0xffffffff),
                (6.0, 0xff202124),
                (1.0, 0xfffefefe),
            ];
            for (shadow, background) in elevations {
                for ((dx, dy), (w, h)) in places.into_iter().flat_map(|p| sizes.map(|s| (p, s))) {
                    let r = Rect {
                        left: 36.0 + dx,
                        top: 9.0 + dy,
                        right: 36.0 + dx + w,
                        bottom: 9.0 + dy + h,
                    };
                    let style = Style {
                        background,
                        shadow,
                        shape,
                        corners,
                        percent,
                        ..Style::default()
                    };
                    let outer = kurbo::Rect::new(0.0, 0.0, 200.0, 200.0);
                    cut += usize::from(rim(outer, 1.0, &Cover::new(r, &style)).is_some());
                    let (with_rim, whole) = (paint(r, &style, true), paint(r, &style, false));
                    let (count, first) = differences(&with_rim, &whole);
                    assert_eq!(count, 0, "{shape:?} {corners:?} {shadow} {r:?}: {first:?}");
                }
            }
        }
        assert!(cut > 200, "the cases must leave pixels out: {cut}");
    }

    #[test]
    fn a_round_button_hides_most_of_its_shadow() {
        // The smartbar's raised toggle: a 73.5 px disc at 2dp, density 2.625.
        let r = Rect {
            left: 15.75,
            top: 15.75,
            right: 89.25,
            bottom: 89.25,
        };
        let style = Style {
            background: 0xffffffff,
            shape: ShapeKind::Circle,
            shadow: 5.25,
            ..Style::default()
        };
        let cover = Cover::new(r, &style);
        let outer = kurbo::Rect::new(0.0, 0.0, 105.0, 105.0);
        let path = rim(outer, 0.0, &cover).expect("a hole");
        // The holes' area: the outline's minus what the path still paints.
        let hidden = 105.0 * 105.0 - kurbo::Shape::area(&path).abs();
        let disc = std::f64::consts::PI * 36.75 * 36.75;
        assert!(hidden > 0.6 * disc, "{hidden} of {disc}");
    }

    #[test]
    fn a_box_that_is_not_opaque_keeps_its_whole_shadow() {
        let r = Rect {
            left: 30.25,
            top: 10.5,
            right: 70.0,
            bottom: 60.0,
        };
        let style = Style {
            background: 0xccffffff,
            shadow: 2.625,
            corners: [6.0; 4],
            ..Style::default()
        };
        assert_eq!(paint(r, &style, true), paint(r, &style, false));
        let thin = Rect {
            left: 1.5,
            top: 0.0,
            right: 6.5,
            bottom: 9.0,
        };
        let cover = Cover::new(thin, &Style::default());
        assert!(rim(kurbo::Rect::new(0.0, 0.0, 10.0, 10.0), 1.0, &cover).is_none());
    }
}
