//! The property editor's values as the screen edits them: how each encoder is edited, the
//! encoders' names (Compose's `PropertyValueEncoderNameMap`), a value as the text its fields and
//! dropdowns show and back, the checks `ExtensionValidation` made of a variable's name and a
//! shape's corner, and a workspace file's URI as the file selector built it.
use snygg::{
    text,
    value::{Corners, Encoder, ShapeKind, Unit, Value},
    value_spec::number,
};

/// Why a variable's name is refused: ThemeComponentVariableName's messages.
pub const ENTER_NAME: &str = "ext__validation__enter_property";
pub const INVALID_NAME: &str = "ext__validation__error_property";
/// SnyggDpShapeValue's and SnyggPercentShapeValue's messages.
pub const ENTER_DP: &str = "ext__validation__enter_dp_size";
pub const NOT_A_NUMBER: &str = "ext__validation__enter_valid_number";
pub const NEGATIVE: &str = "ext__validation__enter_positive_number";
pub const ENTER_PERCENT: &str = "ext__validation__enter_percent_size";
pub const NOT_A_PERCENT: &str = "ext__validation__enter_number_between_0_100";
pub const ABOVE_HALF: &str = "ext__validation__hint_value_above_50_percent";
/// The messages the value fields show.
pub const MESSAGES: [&str; 10] = [
    ENTER_NAME,
    INVALID_NAME,
    ENTER_DP,
    NOT_A_NUMBER,
    NEGATIVE,
    ENTER_PERCENT,
    NOT_A_PERCENT,
    ABOVE_HALF,
    "settings__theme_editor__file_selector_no_files_text",
    "ext__editor__files__title",
];

/// How an encoder's value is edited.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Editing {
    /// A keyword or a shape without sizes: nothing to edit.
    Nothing,
    /// A dropdown: a variable, a colour name, a keyword, a font or a file.
    Choice,
    /// One field: a size or the line count.
    Line,
    /// A static colour: the colour picker (`value_color.rs`).
    Picker,
    /// Four fields: padding's sides or a shape's corners.
    Sides,
}

pub fn editing(e: Encoder) -> Editing {
    use Encoder::*;
    match e {
        Dp | Sp | Percentage | MaxLines => Editing::Line,
        Color => Editing::Picker,
        Padding | CutDp | RoundedDp | CutPercent | RoundedPercent => Editing::Sides,
        Inherit | Undefined | Yes | No | Rectangle | Circle => Editing::Nothing,
        _ => Editing::Choice,
    }
}

/// An encoder's name as the encoder dropdown shows it; Undefined is the select placeholder.
pub fn encoder_key(e: Encoder) -> &'static str {
    use Encoder::*;
    match e {
        Undefined => "general__select_dropdown_value_placeholder",
        Inherit => "snygg__property_value__explicit_inherit",
        Variable => "snygg__property_value__defined_var",
        Yes => "snygg__property_value__yes",
        No => "snygg__property_value__no",
        Color => "snygg__property_value__solid_color",
        DynamicLight => "snygg__property_value__material_you_light_color",
        DynamicDark => "snygg__property_value__material_you_dark_color",
        GenericFont => "snygg__property_value__font_family_generic",
        CustomFont => "snygg__property_value__font_family_custom",
        FontStyle => "snygg__property_value__font_style",
        FontWeight => "snygg__property_value__font_weight",
        Padding => "snygg__property_value__padding",
        Rectangle => "snygg__property_value__rectangle_shape",
        Circle => "snygg__property_value__circle_shape",
        CutDp => "snygg__property_value__cut_corner_shape_dp",
        CutPercent => "snygg__property_value__cut_corner_shape_percent",
        RoundedDp => "snygg__property_value__rounded_corner_shape_dp",
        RoundedPercent => "snygg__property_value__rounded_corner_shape_percent",
        Dp => "snygg__property_value__dp_size",
        Sp => "snygg__property_value__sp_size",
        Percentage => "snygg__property_value__percentage_size",
        ContentScale => "snygg__property_value__content_scale",
        TextAlign => "snygg__property_value__text_align",
        TextDecoration => "snygg__property_value__text_decoration_line",
        MaxLines => "snygg__property_value__text_max_lines",
        TextOverflow => "snygg__property_value__text_overflow",
        Uri => "snygg__property_value__uri",
    }
}

/// The unit a field shows after its text.
pub fn unit(e: Encoder) -> &'static str {
    use Encoder::*;
    match e {
        Dp | Padding | CutDp | RoundedDp => "dp",
        Sp => "sp",
        Percentage | CutPercent | RoundedPercent => "%",
        _ => "",
    }
}

/// The keywords a keyword encoder's dropdown lists. FontWeight lists its nine words once each,
/// as the stylesheet always writes the word; Compose listed all 18 keys, each weight twice.
pub fn keywords(e: Encoder) -> &'static [&'static str] {
    let all = e.keywords();
    if e == Encoder::FontWeight {
        &all[..9]
    } else {
        all
    }
}

fn lossy(t: &[u16]) -> String {
    String::from_utf16_lossy(t)
}
fn shown(n: f32) -> String {
    lossy(&number(n))
}

/// What the screen shows of `v`: the dropdown's choice, and the fields' texts. A percentage is
/// shown as a whole percent, where Compose showed the fraction.
pub fn texts_of(v: &Value) -> (String, [String; 4]) {
    let mut texts: [String; 4] = Default::default();
    let choice = match v {
        Value::Dimension(Unit::Percentage, f) => {
            texts[0] = shown(f * 100.0);
            String::new()
        }
        Value::Dimension(_, f) => {
            texts[0] = shown(*f);
            String::new()
        }
        Value::Color(_) | Value::MaxLines(_) => {
            let written = v.encoder().serialize(v);
            texts[0] = written.map_or_else(|_| String::new(), |t| lossy(&t));
            String::new()
        }
        Value::Padding(a) | Value::Shape(_, Corners::Dp(a)) => {
            texts = a.map(shown);
            String::new()
        }
        Value::Shape(_, Corners::Percent(a)) => {
            texts = a.map(|n| n.to_string());
            String::new()
        }
        Value::FontWeight(w) => {
            let at = usize::try_from(w / 100 - 1).ok().filter(|_| w % 100 == 0);
            at.and_then(|k| keywords(Encoder::FontWeight).get(k))
                .map_or_else(String::new, |k| (*k).to_owned())
        }
        Value::DynamicColor { name, .. }
        | Value::Enum(_, name)
        | Value::Font(name)
        | Value::Uri(name)
        | Value::Variable(name) => lossy(name),
        Value::Keyword(_) => String::new(),
    };
    (choice, texts)
}

/// A field's number, trimmed as Kotlin's `toFloatOrNull` takes it.
fn float(s: &str) -> Option<f32> {
    s.trim().parse().ok()
}
fn floats(texts: &[String; 4]) -> Option<[f32; 4]> {
    let all: Vec<f32> = texts.iter().map(|t| float(t)).collect::<Option<_>>()?;
    all.try_into().ok()
}
fn ints(texts: &[String; 4]) -> Option<[i32; 4]> {
    let all: Vec<i32> = texts
        .iter()
        .map(|t| t.trim().parse().ok())
        .collect::<Option<_>>()?;
    all.try_into().ok()
}

/// The value encoder `e` gives the screen's `choice` and field `texts`; None while a field does
/// not hold a number. Whether the stylesheet can hold it is [`reads_back`]'s question.
pub fn value_of(e: Encoder, choice: &str, texts: &[String; 4]) -> Option<Value> {
    use Encoder::*;
    let (t, kind) = (text(choice), shape_kind(e));
    Some(match e {
        Variable => Value::Variable(t),
        DynamicLight | DynamicDark => Value::DynamicColor {
            dark: e == DynamicDark,
            name: t,
        },
        GenericFont | FontStyle | TextAlign | TextDecoration | TextOverflow | ContentScale => {
            Value::Enum(e, t)
        }
        FontWeight => e.deserialize(&t).ok()?,
        MaxLines | Color => e.deserialize(&text(texts[0].trim())).ok()?,
        CustomFont => Value::Font(t),
        Uri => Value::Uri(t),
        Dp => Value::Dimension(Unit::Dp, float(&texts[0])?),
        Sp => Value::Dimension(Unit::Sp, float(&texts[0])?),
        Percentage => Value::Dimension(Unit::Percentage, float(&texts[0])? / 100.0),
        Padding => Value::Padding(floats(texts)?),
        CutDp | RoundedDp => Value::Shape(kind, Corners::Dp(floats(texts)?)),
        CutPercent | RoundedPercent => Value::Shape(kind, Corners::Percent(ints(texts)?)),
        Inherit | Undefined | Yes | No | Rectangle | Circle => e.default_value(),
    })
}
fn shape_kind(e: Encoder) -> ShapeKind {
    match e {
        Encoder::CutDp | Encoder::CutPercent => ShapeKind::Cut,
        Encoder::Circle => ShapeKind::Circle,
        Encoder::Rectangle => ShapeKind::Rectangle,
        _ => ShapeKind::Rounded,
    }
}

/// Whether the stylesheet can hold `v`: encoder `e` writes it and reads what it wrote. Compose
/// asked only that it be written, so a cut corner of 2.5 dp, which the stylesheet reads as a
/// whole number, was written and the theme then failed to load.
pub fn reads_back(e: Encoder, v: &Value) -> bool {
    let written = e.serialize(v);
    written.is_ok_and(|t| e.deserialize(&t).is_ok())
}

/// ThemeComponentVariableName: a name, then one matching `--[a-zA-Z0-9-]+`.
pub fn variable_check(name: &str) -> &'static str {
    let s = name.trim();
    let valid = s.strip_prefix("--").is_some_and(|rest| {
        !rest.is_empty() && rest.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    });
    match () {
        _ if s.is_empty() => ENTER_NAME,
        _ if !valid => INVALID_NAME,
        _ => "",
    }
}
/// SnyggDpShapeValue: a size, a number, not negative.
pub fn dp_check(s: &str) -> &'static str {
    match float(s) {
        _ if s.trim().is_empty() => ENTER_DP,
        None => NOT_A_NUMBER,
        Some(f) if f < 0.0 => NEGATIVE,
        Some(_) => "",
    }
}
/// SnyggPercentShapeValue: its message, or its hint above 50.
pub fn percent_check(s: &str) -> (&'static str, &'static str) {
    match s.trim().parse::<i32>().ok() {
        _ if s.trim().is_empty() => (ENTER_PERCENT, ""),
        None => (NOT_A_NUMBER, ""),
        Some(n) if !(0..=100).contains(&n) => (NOT_A_PERCENT, ""),
        Some(n) if n > 50 => ("", ABOVE_HALF),
        Some(_) => ("", ""),
    }
}

/// Field `k`'s message for encoder `e`, and its hint: the check Compose made of that kind of
/// field, then whether the stylesheet could hold the number at all.
pub fn field_check(e: Encoder, k: usize, texts: &[String; 4]) -> (&'static str, &'static str) {
    let s = texts[k].as_str();
    let (message, hint) = match e {
        Encoder::CutPercent | Encoder::RoundedPercent => percent_check(s),
        Encoder::Dp | Encoder::Padding | Encoder::CutDp | Encoder::RoundedDp => (dp_check(s), ""),
        _ => ("", ""),
    };
    if !message.is_empty() {
        return (message, hint);
    }
    // This field alone, the others at zero: whether the stylesheet holds its number.
    let mut alone: [String; 4] = Default::default();
    alone.iter_mut().for_each(|t| *t = "0".into());
    alone[if editing(e) == Editing::Sides { k } else { 0 }] = s.to_owned();
    let holds = value_of(e, "", &alone).is_some_and(|v| reads_back(e, &v));
    (if holds { "" } else { NOT_A_NUMBER }, hint)
}

/// Android's `Uri.encode(s, allow)`: letters, digits, `_-!.~'()*` and `allow` as they are,
/// every other character as its UTF-8 bytes in `%XX`.
pub fn uri_encode(s: &str, allow: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || "_-!.~'()*".contains(c) || allow.contains(c) {
            out.push(c);
        } else {
            let mut bytes = [0; 4];
            for b in c.encode_utf8(&mut bytes).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}
/// Workspace file `name` in folder `dir` (`fonts`, `images`) as the file selector wrote it:
/// `"flex:" + Uri.encode(relPath, "/")`.
pub fn flex_uri(dir: &str, name: &str) -> String {
    format!("flex:{}", uri_encode(&format!("/{dir}/{name}"), "/"))
}
