//! The colour maths behind the accent colour dialogs (Jetpref's ColorPickerPreference and
//! JetPrefColorPicker): Compose's `Color.hsv` and sRGB packing, Jetpref's `toHsv`, its text
//! formats and parser, AndroidX's `calculateLuminance`, and the preference's stored form.
//! Every step is f32 in Kotlin's order, so one input gives Compose's bytes, and the pictures the
//! dialog draws (the saturation/value plane, the hue bar, the checkered backing) as ARGB pixels.

/// FlorisBoard's `ColorMappings.colors`: the Presets grid, in order.
pub const PRESETS: [u32; 19] = [
    0xff4caf50, 0xfff44336, 0xffe91e63, 0xffff2c93, 0xff9c27b0, 0xff673ab7, 0xff3f51b5, 0xff2196f3,
    0xff03a9f4, 0xff00bcd4, 0xff009688, 0xff8bc34a, 0xffcddc39, 0xffffeb3b, 0xffffc107, 0xffff9800,
    0xff795548, 0xff607d8b, 0xff9e9e9e,
];
/// `Color.Unspecified` as the preference stores it: no colour of its own, the system accent.
pub const UNSPECIFIED: &str = "0000000000000010";

/// The text forms Jetpref's field shows and reads, in the order it tries them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Hex,
    Rgb,
    Hsv,
}

/// Compose's sRGB `Color(red, green, blue, alpha)`: each channel to 8 bits, half up.
pub fn pack(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let byte = |c: f32| ((c * 255.0 + 0.5) as u32).min(255);
    byte(a.clamp(0.0, 1.0)) << 24 | byte(r) << 16 | byte(g) << 8 | byte(b)
}

/// `Color.red`, `green`, `blue`, `alpha` of a packed sRGB colour.
pub fn channels(argb: u32) -> [f32; 4] {
    [16, 8, 0, 24].map(|shift| ((argb >> shift) & 0xff) as f32 / 255.0)
}

/// `Color.hsv(hue, saturation, value, alpha)`.
pub fn hsv(h: f32, s: f32, v: f32, a: f32) -> u32 {
    let c = |n: f32| {
        let k = (n + h / 60.0) % 6.0;
        v - v * s * 0f32.max(k.min((4.0 - k).min(1.0)))
    };
    pack(c(5.0), c(3.0), c(1.0), a)
}

/// Jetpref's `Color.toHsv()`: hue in degrees, saturation, value and alpha in 0..1.
pub fn to_hsv(argb: u32) -> [f32; 4] {
    let [r, g, b, a] = channels(argb);
    let max = r.max(g.max(b));
    let min = r.min(g.min(b));
    let diff = max - min;
    let h = if max == min {
        0.0
    } else if max == r {
        (60.0 * ((g - b) / diff) + 360.0) % 360.0
    } else if max == g {
        (60.0 * ((b - r) / diff) + 120.0) % 360.0
    } else {
        (60.0 * ((r - g) / diff) + 240.0) % 360.0
    };
    let s = if max == 0.0 { 0.0 } else { diff / max };
    [h, s, max, a]
}

/// Kotlin's `roundToInt` of a non-negative float.
fn round(x: f32) -> i32 {
    x.round() as i32
}

/// `ColorRepresentation.formatColor(color, withAlpha = false)`.
pub fn format(f: Format, argb: u32) -> String {
    let [r, g, b, _] = channels(argb).map(|c| round(c * 255.0));
    match f {
        Format::Hex => format!("#{r:02x}{g:02x}{b:02x}"),
        Format::Rgb => format!("rgb({r}, {g}, {b})"),
        Format::Hsv => {
            let [h, s, v, _] = to_hsv(argb);
            format!(
                "hsv({}, {}, {})",
                round(h),
                round(s * 100.0),
                round(v * 100.0)
            )
        }
    }
}

/// `ColorRepresentation.formatColor(color, withAlpha = true)`: `#rrggbbaa`, `rgba(r, g, b, a)` or
/// `hsva(h, s, v, a)`, the alpha as `"%.2f"` writes it in `Locale.ROOT` with its trailing zeros
/// and then its point trimmed (`1`, `0.5`, `0`).
pub fn format_alpha(f: Format, argb: u32) -> String {
    let k = argb >> 24;
    let plain = format(f, argb);
    if f == Format::Hex {
        return format!("{plain}{k:02x}");
    }
    // k / 255 to two places, half up: never a tie, as 200 k + 255 is odd and 510 is even.
    let hundredths = (200 * k + 255) / 510;
    let digits = format!("{}.{:02}", hundredths / 100, hundredths % 100);
    let alpha = digits.trim_end_matches('0').trim_end_matches('.');
    format!("{}a{}, {alpha})", &plain[..3], &plain[3..plain.len() - 1])
}

/// Compose's `compositeOver`: `fg` over `bg`, both straight sRGB.
pub fn over(fg: u32, bg: u32) -> u32 {
    let ([fr, fg_, fb, fa], [br, bg_, bb, ba]) = (channels(fg), channels(bg));
    let a = fa + ba * (1.0 - fa);
    let mix = |f: f32, b: f32| {
        if a == 0.0 {
            0.0
        } else {
            (f * fa + b * ba * (1.0 - fa)) / a
        }
    };
    pack(mix(fr, br), mix(fg_, bg_), mix(fb, bb), a)
}

/// Kotlin's `Char.isWhitespace`, which `String.trim()` strips: Java's whitespace and space
/// separators. Unicode's White_Space adds U+0085 and leaves out U+001C to U+001F.
fn kotlin_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || ('\u{1c}'..='\u{1f}').contains(&c)
}

/// Java regex `\s` without the Unicode flag.
fn regex_space(c: u8) -> bool {
    matches!(c, b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r')
}

/// A decimal with no leading zero (`0` itself excepted) no greater than `max`, as Jetpref's
/// `Int255`, `Int360` and `Int100` alternations accept.
fn number(digits: &[u8], max: u32) -> Option<u32> {
    let ok = !digits.is_empty()
        && digits.len() <= 3
        && digits.iter().all(u8::is_ascii_digit)
        && (digits.len() == 1 || digits[0] != b'0');
    let n = ok.then(|| digits.iter().fold(0, |n, d| n * 10 + u32::from(d - b'0')))?;
    (n <= max).then_some(n)
}

/// A call's three numbers' digits and its alpha's, if it has one.
type Parts<'a> = ([&'a [u8]; 3], Option<&'a [u8]>);

/// `name(a, b, c)` or, with an alpha, `name(a, b, c, alpha)` with Jetpref's separators: the
/// numbers' digits, or None.
fn call<'a>(text: &'a [u8], name: &[u8]) -> Option<Parts<'a>> {
    let n = name.len();
    if text.len() < n || !text[..n].eq_ignore_ascii_case(name) {
        return None;
    }
    let mut rest = &text[n..];
    if let [b'a' | b'A', tail @ ..] = rest {
        rest = tail;
    }
    let inner = rest.strip_prefix(b"(")?.strip_suffix(b")")?;
    let trim = |s: &'a [u8]| -> &'a [u8] {
        let start = s.iter().position(|&c| !regex_space(c)).unwrap_or(s.len());
        let end = s
            .iter()
            .rposition(|&c| !regex_space(c))
            .map_or(start, |e| e + 1);
        &s[start..end]
    };
    let parts: Vec<&[u8]> = inner.split(|&c| c == b',').map(trim).collect();
    match parts[..] {
        [a, b, c] => Some(([a, b, c], None)),
        [a, b, c, alpha] => Some(([a, b, c], Some(alpha))),
        _ => None,
    }
}

/// Jetpref's alpha component, `1(\.0)?|0(\.[0-9]+)?|\.[0-9]+`, as `Float.parseFloat` reads it.
fn alpha(digits: &[u8]) -> Option<f32> {
    let ok = match digits {
        b"1" | b"1.0" | b"0" => true,
        [b'0', b'.', rest @ ..] | [b'.', rest @ ..] => {
            !rest.is_empty() && rest.iter().all(u8::is_ascii_digit)
        }
        _ => false,
    };
    std::str::from_utf8(digits)
        .ok()?
        .parse()
        .ok()
        .filter(|_| ok)
}

/// `Color.parseOrNull(text, withAlpha = false)`: the colour and the form it was written in.
pub fn parse(text: &str) -> Option<(u32, Format)> {
    parse_with(text, false)
}

/// `Color.parseOrNull(text, withAlpha)`: with `with_alpha`, `#rrggbbaa` and a fourth component
/// give the colour its alpha; without, they are no colour at all.
pub fn parse_with(text: &str, with_alpha: bool) -> Option<(u32, Format)> {
    let t = text.trim_matches(kotlin_space).as_bytes();
    if let Some(hex) = t.strip_prefix(b"#") {
        let long = with_alpha && hex.len() == 8;
        if hex.iter().all(u8::is_ascii_hexdigit) && (hex.len() == 6 || long) {
            let v = u32::from_str_radix(std::str::from_utf8(hex).ok()?, 16).ok()?;
            let argb = if long {
                v.rotate_right(8)
            } else {
                0xff00_0000 | v
            };
            return Some((argb, Format::Hex));
        }
        return None;
    }
    let opacity = |a: Option<&[u8]>| match a {
        None => Some(1.0),
        Some(a) if with_alpha => alpha(a),
        Some(_) => None,
    };
    if let Some(([r, g, b], a)) = call(t, b"rgb") {
        let [r, g, b] = [number(r, 255)?, number(g, 255)?, number(b, 255)?];
        let a = pack(0.0, 0.0, 0.0, opacity(a)?) & 0xff00_0000;
        return Some((a | r << 16 | g << 8 | b, Format::Rgb));
    }
    let ([h, s, v], a) = call(t, b"hsv")?;
    let [h, s, v] = [number(h, 360)?, number(s, 100)?, number(v, 100)?];
    let color = hsv(h as f32, s as f32 / 100.0, v as f32 / 100.0, opacity(a)?);
    Some((color, Format::Hsv))
}

/// AndroidX `ColorUtils.calculateLuminance`: relative luminance, 0 (black) to 1 (white).
pub fn luminance(argb: u32) -> f64 {
    let linear = |shift: u32| {
        let c = f64::from((argb >> shift) & 0xff) / 255.0;
        if c < 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    100.0 * (linear(16) * 0.2126 + linear(8) * 0.7152 + linear(0) * 0.0722) / 100.0
}

/// The stored preference as a packed sRGB colour; None for `Color.Unspecified` (the system
/// accent). A colour in another colour space, which nothing here writes, also reads as None.
pub fn decode(stored: &str) -> Option<u32> {
    let value = u64::from_str_radix(stored, 16).ok()?;
    (value & 0x3f == 0).then_some((value >> 32) as u32)
}

/// What the dialog's OK stores: Jetpref's `colorOverride` turns the system accent itself
/// (`isMaterialYou`) back into `Color.Unspecified`, so the row keeps following the wallpaper.
pub fn encode(argb: u32, system_accent: u32) -> String {
    if argb == system_accent {
        UNSPECIFIED.into()
    } else {
        format!("{:016x}", u64::from(argb) << 32)
    }
}

/// How much of the pixel centred at (`x`, `y`) a `w` x `h` rectangle with `r` corners covers.
fn coverage(x: f32, y: f32, w: f32, h: f32, r: f32) -> f32 {
    let dx = (r - x).max(x - (w - r)).max(0.0);
    let dy = (r - y).max(y - (h - r)).max(0.0);
    if dx == 0.0 || dy == 0.0 {
        let edge = x.min(w - x).min(y).min(h - y);
        return (edge + 0.5).clamp(0.0, 1.0);
    }
    (r - (dx * dx + dy * dy).sqrt() + 0.5).clamp(0.0, 1.0)
}

/// An ARGB pixel from straight channels in 0..1.
fn argb(a: f32, r: f32, g: f32, b: f32) -> i32 {
    pack(r, g, b, a) as i32
}

/// Fills `w` x `h` pixels from each centre's (x, y), with corners of radius `r` cut away.
fn picture(w: usize, h: usize, r: f32, pixel: impl Fn(f32, f32) -> [f32; 4]) -> Vec<i32> {
    let mut out = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let (cx, cy) = (x as f32 + 0.5, y as f32 + 0.5);
            let [a, red, green, blue] = pixel(cx, cy);
            out.push(argb(
                a * coverage(cx, cy, w as f32, h as f32, r),
                red,
                green,
                blue,
            ));
        }
    }
    out
}

/// The saturation/value plane for `hue` (`Color.hsv(hue, 1, 1)`): white to the hue across, then
/// black rising from transparent at the top to opaque at the bottom, corners rounded by `r`.
pub fn plane(w: usize, h: usize, hue: u32, r: f32) -> Vec<i32> {
    let [hr, hg, hb, _] = channels(hue);
    picture(w, h, r, |x, y| {
        let t = x / w as f32;
        let dark = y / h as f32;
        let mix = |c: f32| (1.0 + (c - 1.0) * t) * (1.0 - dark);
        [1.0, mix(hr), mix(hg), mix(hb)]
    })
}

/// The hue bar: red, yellow, green, cyan, blue, magenta and red again, evenly spaced, as a pill.
pub fn hue_bar(w: usize, h: usize) -> Vec<i32> {
    const STOPS: [u32; 7] = [
        0xffff0000, 0xffffff00, 0xff00ff00, 0xff00ffff, 0xff0000ff, 0xffff00ff, 0xffff0000,
    ];
    picture(w, h, h as f32 / 2.0, |x, _| {
        let t = (x / w as f32 * 6.0).clamp(0.0, 6.0);
        let i = (t as usize).min(5);
        let (from, to) = (channels(STOPS[i]), channels(STOPS[i + 1]));
        let f = t - i as f32;
        let mix = |k: usize| from[k] + (to[k] - from[k]) * f;
        [1.0, mix(0), mix(1), mix(2)]
    })
}

/// Jetpref's `checkeredBackground` in a circle: `cell`-pixel squares, `even` on the squares
/// whose column and row add to an even number, clear on the rest.
pub fn checkered(size: usize, cell: f32, even: u32) -> Vec<i32> {
    checkered_box(size, size, size as f32 / 2.0, cell, even)
}

/// [`checkered`] in a `w` x `h` box with corners of radius `r`: the alpha bar's backing and a
/// value icon's.
pub fn checkered_box(w: usize, h: usize, r: f32, cell: f32, even: u32) -> Vec<i32> {
    let [red, green, blue, a] = channels(even);
    picture(w, h, r, |x, y| {
        let odd = ((x / cell) as usize + (y / cell) as usize) % 2 == 1;
        [if odd { 0.0 } else { a }, red, green, blue]
    })
}

/// The alpha bar: Compose's linear gradient from `Color.Transparent` to `rgb` (interpolated
/// unpremultiplied, as Android's gradients are), as a pill.
pub fn alpha_bar(w: usize, h: usize, rgb: u32) -> Vec<i32> {
    let [r, g, b, _] = channels(rgb);
    picture(w, h, h as f32 / 2.0, |x, _| {
        let t = (x / w as f32).clamp(0.0, 1.0);
        [t, r * t, g * t, b * t]
    })
}
