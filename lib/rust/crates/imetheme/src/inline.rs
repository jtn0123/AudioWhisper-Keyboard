//! The inline autofill chips' colours: the `inline-autofill-chip` element's background and
//! foreground, queried as Compose queried it inside the keyboard's window, with the theme's
//! dynamic colours taken from the palette.

use snygg::resolve::{CompiledTheme, Properties, Query, get};
use snygg::value::{Unit, Value};
use snygg::{Text, text};

/// What a chip's background is when the theme gives no colour: white.
pub const BACKGROUND: u32 = 0xFFFF_FFFF;
/// What a chip's foreground is when the theme gives no colour: black.
pub const FOREGROUND: u32 = 0xFF00_0000;

/// The palette dynamic colours resolve in: each name's light and dark colour, as ARGB.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Palette {
    pub names: Vec<Text>,
    pub light: Vec<u32>,
    pub dark: Vec<u32>,
}

/// The chips' background and foreground in the stylesheet `json`, for the keyboard's `mode` and
/// `shift` state, as ARGB. A stylesheet that doesn't compile styles nothing.
pub fn chip(json: &str, mode: &[u16], shift: &[u16], palette: &Palette) -> (u32, u32) {
    let style = styled(
        json,
        mode,
        shift,
        palette,
        &["root", "window", "inline-autofill-chip"],
    );
    let argb = |name: &str, default| match get(&style, &text(name)) {
        Some(Value::Color(c)) => argb(*c),
        _ => default,
    };
    (
        argb("background", BACKGROUND),
        argb("foreground", FOREGROUND),
    )
}

/// The style of the last of `elements`, each queried inside the one before as Compose nested
/// them, in the stylesheet `json`: the root and window with the keyboard's `mode` and `shift`
/// state, any other element with no attributes.
pub fn styled(
    json: &str,
    mode: &[u16],
    shift: &[u16],
    palette: &Palette,
    elements: &[&str],
) -> Properties {
    let theme = CompiledTheme::from_json(json).unwrap_or_else(|_| CompiledTheme {
        style: vec![],
        fonts: vec![],
    });
    let attributes = [
        (text("mode"), mode.to_vec()),
        (text("shiftstate"), shift.to_vec()),
    ];
    // Compose passed only the colours the element's rules and its parent name, but only those
    // can be in the result, so the whole palette answers the same.
    let scheme = |colors: &[u32]| -> Properties {
        let named = palette.names.iter().zip(colors);
        named
            .map(|(name, &argb)| (name.clone(), color(argb)))
            .collect()
    };
    let (light, dark) = (scheme(&palette.light), scheme(&palette.dark));
    let mut style = vec![(text("font-size"), Value::Dimension(Unit::Sp, 14.0))];
    for element in elements {
        let keyed = matches!(*element, "root" | "window");
        let element = text(element);
        let query = Query {
            element: &element,
            attributes: if keyed { &attributes[..] } else { &[][..] },
            selector: 0,
        };
        style = theme.resolve_with_palette(&query, &style, 1.0, &light, &dark);
    }
    style
}

/// A colour's ARGB, from its quantized channel fractions.
pub fn argb(c: [f32; 4]) -> u32 {
    let b = c.map(|n| (n.clamp(0.0, 1.0) * 255.0).round() as u32);
    b[3] << 24 | b[0] << 16 | b[1] << 8 | b[2]
}

/// The colour an ARGB value is.
pub fn color(argb: u32) -> Value {
    Value::Color([
        (argb >> 16 & 255) as f32 / 255.0,
        (argb >> 8 & 255) as f32 / 255.0,
        (argb & 255) as f32 / 255.0,
        (argb >> 24) as f32 / 255.0,
    ])
}
