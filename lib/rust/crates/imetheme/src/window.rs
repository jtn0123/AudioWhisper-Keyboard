//! The keyboard window's background, as Compose's `SnyggSurfaceView` drew it under the keys:
//! the `window` element's colour, black if it names none, and its image, scaled by its content
//! scale and centred; and the navigation bar beside it, as `SystemUiIme` set it.

use snygg::resolve::get;
use snygg::text;
use snygg::value::Value;

use crate::inline::{Palette, argb, styled};

/// What the background is drawn with when the theme names no colour: black.
pub const BACKGROUND: u32 = 0xFF00_0000;

/// The content scales a `content-scale` names, in snygg's order; crop when none is named.
const SCALES: [&str; 7] = [
    "crop",
    "fill-bounds",
    "fill-height",
    "fill-width",
    "fit",
    "inside",
    "none",
];

/// The window's background and the navigation bar that goes with it.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    /// The colour the surface is filled with, as ARGB.
    pub background: u32,
    /// The image's path inside the theme's directory, if the theme names one Android may load.
    pub image: Option<String>,
    /// The image's content scale, an index into `SCALES`.
    pub scale: usize,
    /// Whether the navigation bar's buttons are drawn dark, on a light background.
    pub light_bars: bool,
    /// Whether Android keeps the navigation bar readable over an image.
    pub contrast: bool,
}

/// The `window` element's background in the stylesheet `json`, for the keyboard's `mode` and
/// `shift` state.
pub fn window(json: &str, mode: &[u16], shift: &[u16], palette: &Palette) -> Window {
    let style = styled(json, mode, shift, palette, &["root", "window"]);
    let named = |name: &str| get(&style, &text(name));
    let color = match named("background") {
        Some(Value::Color(c)) => Some(*c),
        _ => None,
    };
    let uri = match named("background-image") {
        Some(Value::Uri(uri)) => Some(uri.clone()),
        _ => None,
    };
    let scale = match named("content-scale") {
        Some(Value::Enum(_, name)) => SCALES.iter().position(|s| text(s) == *name),
        _ => None,
    };
    Window {
        background: color.map(argb).unwrap_or(BACKGROUND),
        image: uri.as_deref().and_then(asset),
        scale: scale.unwrap_or(0),
        light_bars: uri.is_none() && color.is_some_and(|c| luminance(c) >= 0.5),
        contrast: uri.is_some(),
    }
}

/// The path inside the theme's directory a `flex:` URI names, as `FlorisAssetResolver` resolved
/// it through `java.net.URI`: no authority, the query and fragment dropped, percent escapes
/// decoded, `.` and `..` resolved, never above the directory. None for any other URI.
pub fn asset(uri: &[u16]) -> Option<String> {
    let uri = String::from_utf16(uri).ok()?;
    let mut path = uri.strip_prefix("flex:")?;
    path = &path[..path.find(['?', '#']).unwrap_or(path.len())];
    if let Some(rest) = path.strip_prefix("//") {
        path = rest;
    }
    let path = decode(path.strip_prefix('/')?)?;
    let mut parts: Vec<&str> = vec![];
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            part => parts.push(part),
        }
    }
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// `path` with its percent escapes decoded as UTF-8, or None if one is malformed.
fn decode(path: &str) -> Option<String> {
    let (mut bytes, mut rest) = (vec![], path.as_bytes());
    while let Some((&b, tail)) = rest.split_first() {
        if b == b'%' {
            let hex = std::str::from_utf8(tail.get(..2)?).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &tail[2..];
        } else {
            bytes.push(b);
            rest = tail;
        }
    }
    String::from_utf8(bytes).ok()
}

/// A colour's relative luminance, as Compose's `Color.luminance` gives it for sRGB.
pub fn luminance(c: [f32; 4]) -> f32 {
    let linear = |v: f32| {
        let v = f64::from(v);
        if v >= 0.04045 {
            ((v + 0.055) / 1.055).powf(2.4)
        } else {
            v / 12.92
        }
    };
    let y = 0.2126 * linear(c[0]) + 0.7152 * linear(c[1]) + 0.0722 * linear(c[2]);
    (y as f32).clamp(0.0, 1.0)
}

/// Where an `image` of its size is drawn on a `canvas` of its size, for `scale`, as
/// `SnyggSurfaceView` drew it: scaled, centred and rounded out to whole pixels (left, top,
/// right, bottom).
pub fn backdrop(scale: usize, image: [f32; 2], canvas: [f32; 2]) -> [i32; 4] {
    let (sx, sy) = (canvas[0] / image[0], canvas[1] / image[1]);
    let fit = sx.min(sy);
    let (x, y) = match SCALES.get(scale).copied().unwrap_or("crop") {
        "fill-bounds" => (sx, sy),
        "fill-height" => (sy, sy),
        "fill-width" => (sx, sx),
        "fit" => (fit, fit),
        "inside" if image[0] <= canvas[0] && image[1] <= canvas[1] => (1.0, 1.0),
        "inside" => (fit, fit),
        "none" => (1.0, 1.0),
        _ => (sx.max(sy), sx.max(sy)),
    };
    let size = [image[0] * x, image[1] * y];
    let left = canvas[0] / 2.0 - size[0] / 2.0;
    let top = canvas[1] / 2.0 - size[1] / 2.0;
    [
        left.floor() as i32,
        top.floor() as i32,
        (size[0] + left).ceil() as i32,
        (size[1] + top).ceil() as i32,
    ]
}

#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;
