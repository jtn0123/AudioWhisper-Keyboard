//! SnyggValueIcon for the values that are not colours: the Material icon it drew for each kind,
//! by the `@icon:` names `SettingsIcons.kt` maps, and a shape's outline, which Rust draws:
//! `alwaysPercentShape` (a dp corner three times its whole dp as a percentage, at most 100)
//! bordered in onBackground, 22 dp with a 1 dp line, or 16 dp with a hairline in the small form,
//! centred where the icon goes.
use crate::panels::{Panel, rect};
use crate::settings_ui::{App, color};
use keys::Rect;
use snygg::value::{Corners, Encoder, ShapeKind, Unit, Value};

/// The Material icons SnyggValueIcon drew, by their `@icon:` names, in `icon_name`'s order,
/// then a variable's link.
pub const VALUE_ICONS: [&str; 18] = [
    "font_download",
    "format_italic",
    "format_bold",
    "padding",
    "straighten",
    "format_size",
    "format_align_left",
    "format_align_right",
    "format_align_justify",
    "format_align_center",
    "format_strikethrough",
    "format_underlined",
    "wrap_text",
    "attach_file",
    "open_in_full",
    "check_box",
    "check_box_outline_blank",
    "link",
];

/// The Material icon SnyggValueIcon drew for `v`, of [`VALUE_ICONS`]; None for a colour, a
/// shape, a variable, and what Compose drew nothing for (line counts, percentages, inherit and
/// undefined). Yes shows a ticked box beside No's empty one, where Compose drew FormatBold.
pub fn icon_name(v: &Value) -> Option<&'static str> {
    use Encoder::*;
    let word = |t: &[u16]| String::from_utf16_lossy(t);
    let k = match v {
        Value::Enum(GenericFont, _) | Value::Font(_) => 0,
        Value::Enum(FontStyle, _) => 1,
        Value::FontWeight(_) => 2,
        Value::Padding(_) => 3,
        Value::Dimension(Unit::Dp, _) => 4,
        Value::Dimension(Unit::Sp, _) => 5,
        Value::Enum(TextAlign, a) => match word(a).as_str() {
            "left" | "start" => 6,
            "right" | "end" => 7,
            "justify" => 8,
            _ => 9,
        },
        Value::Enum(TextDecoration, d) if word(d) == "line-through" => 10,
        Value::Enum(TextDecoration, _) => 11,
        Value::Enum(TextOverflow, _) => 12,
        Value::Uri(_) => 13,
        Value::Enum(ContentScale, _) => 14,
        Value::Keyword(Yes) => 15,
        Value::Keyword(No) => 16,
        _ => return None,
    };
    Some(VALUE_ICONS[k])
}

/// Shape `v`'s corners in a `size` px box as `alwaysPercentShape` makes them, in Compose's
/// order (top start, top end, bottom end, bottom start), and whether they are cut; None for any
/// other value. Corners too big for a side shrink together, as Skia fits a rounded rectangle.
pub fn shape_corners(v: &Value, size: f32) -> Option<([f32; 4], bool)> {
    let Value::Shape(kind, corners) = v else {
        return None;
    };
    let percent = match (kind, corners) {
        (ShapeKind::Rectangle, _) => [0.0; 4],
        (ShapeKind::Circle, _) => [50.0; 4],
        (_, Corners::Dp(dp)) => dp.map(|c| (c as i32).saturating_mul(3).min(100) as f32),
        (_, Corners::Percent(p)) => p.map(|c| c as f32),
    };
    let r = percent.map(|p| p.max(0.0) / 100.0 * size);
    let sides = [r[0] + r[1], r[1] + r[2], r[2] + r[3], r[3] + r[0]];
    let fit = sides
        .iter()
        .fold(1f32, |f, &s| if s > size { f.min(size / s) } else { f });
    Some((r.map(|c| c * fit), *kind == ShapeKind::Cut))
}

/// Whether (x, y) lies in the `w` x `h` box with corners `r` (round, or cut straight across).
fn inside(x: f32, y: f32, (w, h): (f32, f32), r: [f32; 4], cut: bool) -> bool {
    if x < 0.0 || y < 0.0 || x > w || y > h {
        return false;
    }
    // How far into each corner's square (x, y) lies, from the corner's centre.
    let reach = [
        (r[0] - x, r[0] - y, r[0]),
        (x - (w - r[1]), r[1] - y, r[1]),
        (x - (w - r[2]), y - (h - r[2]), r[2]),
        (r[3] - x, y - (h - r[3]), r[3]),
    ];
    reach.iter().all(|&(dx, dy, c)| {
        dx <= 0.0
            || dy <= 0.0
            || if cut {
                dx + dy <= c
            } else {
                dx * dx + dy * dy <= c * c
            }
    })
}

/// The outline of a `size` px square with `corners`, a `line` px stroke inside its edge as
/// `Modifier.border` draws one, in `ink`: 16 samples a pixel.
pub fn outline(size: usize, (r, cut): ([f32; 4], bool), line: f32, ink: u32) -> Vec<i32> {
    let n = size as f32;
    // The inner edge runs `line` in from the outer, round corners' radii and cut corners' cuts
    // shrinking with it.
    let shrink = if cut {
        line * (2.0 - std::f32::consts::SQRT_2)
    } else {
        line
    };
    let inner = r.map(|c| (c - shrink).max(0.0));
    let [red, green, blue, a] = color::channels(ink);
    let stroke = |x: f32, y: f32| {
        let inner_box = (n - 2.0 * line, n - 2.0 * line);
        inside(x, y, (n, n), r, cut) && !inside(x - line, y - line, inner_box, inner, cut)
    };
    let pixel = |i: usize| {
        let (x, y) = ((i % size) as f32, (i / size) as f32);
        let samples = (0..16).filter(|k| {
            let (sx, sy) = ((k % 4) as f32 / 4.0 + 0.125, (k / 4) as f32 / 4.0 + 0.125);
            stroke(x + sx, y + sy)
        });
        color::pack(red, green, blue, a * samples.count() as f32 / 16.0) as i32
    };
    (0..size * size).map(pixel).collect()
}

impl App {
    /// Shape `v`'s outline centred in `b`; nothing for any other value.
    pub(super) fn shape_icon(
        &mut self,
        p: &mut Panel,
        v: &Value,
        b: Rect,
        small: bool,
        clip: Rect,
    ) {
        let d = self.config[2];
        let (size, line) = if small {
            (16.0 * d, 1.0)
        } else {
            (22.0 * d, d)
        };
        let n = size.round() as usize;
        let Some(corners) = shape_corners(v, n as f32) else {
            return;
        };
        let ink = self.colors[15];
        let name = format!("@shape{n}:{line}:{corners:?}:{ink:08x}");
        self.cached(&name, n, n, &format!("@shape{n}:"), |w, _| {
            outline(w, corners, line, ink)
        });
        let inset = ((b.width() - n as f32) / 2.0).round();
        let at = rect(b.left + inset, b.top + inset, n as f32, n as f32);
        self.paint_image(p, &name, at, clip);
    }
}
