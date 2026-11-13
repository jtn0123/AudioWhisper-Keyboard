//! Plain, Java-free values shared by the managed adapter and the future style resolver.
use crate::{
    Text, text,
    value_spec::{Failure, Spec},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Encoder {
    Inherit,
    Undefined,
    Yes,
    No,
    Dp,
    Sp,
    Percentage,
    Color,
    DynamicLight,
    DynamicDark,
    GenericFont,
    CustomFont,
    FontStyle,
    FontWeight,
    Padding,
    Rectangle,
    Circle,
    CutDp,
    CutPercent,
    RoundedDp,
    RoundedPercent,
    TextAlign,
    TextDecoration,
    MaxLines,
    TextOverflow,
    Uri,
    ContentScale,
    Variable,
}
pub const ENCODERS: [Encoder; 28] = [
    Encoder::Inherit,
    Encoder::Undefined,
    Encoder::Yes,
    Encoder::No,
    Encoder::Dp,
    Encoder::Sp,
    Encoder::Percentage,
    Encoder::Color,
    Encoder::DynamicLight,
    Encoder::DynamicDark,
    Encoder::GenericFont,
    Encoder::CustomFont,
    Encoder::FontStyle,
    Encoder::FontWeight,
    Encoder::Padding,
    Encoder::Rectangle,
    Encoder::Circle,
    Encoder::CutDp,
    Encoder::CutPercent,
    Encoder::RoundedDp,
    Encoder::RoundedPercent,
    Encoder::TextAlign,
    Encoder::TextDecoration,
    Encoder::MaxLines,
    Encoder::TextOverflow,
    Encoder::Uri,
    Encoder::ContentScale,
    Encoder::Variable,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    Dp,
    Sp,
    Percentage,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Circle,
    Cut,
    Rounded,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Corners {
    Dp([f32; 4]),
    Percent([i32; 4]),
}
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Keyword(Encoder),
    Dimension(Unit, f32),
    /// sRGB channel fractions, quantized to eight bits at parse time as Compose does.
    Color([f32; 4]),
    DynamicColor {
        dark: bool,
        name: Text,
    },
    /// Canonical enum spelling (font family/style, alignment, decoration, overflow, scale).
    Enum(Encoder, Text),
    FontWeight(i32),
    Font(Text),
    Padding([f32; 4]),
    Shape(ShapeKind, Corners),
    MaxLines(i32),
    Uri(Text),
    Variable(Text),
}
impl Value {
    pub fn encoder(&self) -> Encoder {
        match self {
            Self::Keyword(e) | Self::Enum(e, _) => *e,
            Self::Dimension(u, _) => match u {
                Unit::Dp => Encoder::Dp,
                Unit::Sp => Encoder::Sp,
                Unit::Percentage => Encoder::Percentage,
            },
            Self::Color(_) => Encoder::Color,
            Self::DynamicColor { dark, .. } => {
                if *dark {
                    Encoder::DynamicDark
                } else {
                    Encoder::DynamicLight
                }
            }
            Self::FontWeight(_) => Encoder::FontWeight,
            Self::Font(_) => Encoder::CustomFont,
            Self::Padding(_) => Encoder::Padding,
            Self::Shape(kind, corners) => match (kind, corners) {
                (ShapeKind::Rectangle, _) => Encoder::Rectangle,
                (ShapeKind::Circle, _) => Encoder::Circle,
                (ShapeKind::Cut, Corners::Dp(_)) => Encoder::CutDp,
                (ShapeKind::Cut, Corners::Percent(_)) => Encoder::CutPercent,
                (ShapeKind::Rounded, Corners::Dp(_)) => Encoder::RoundedDp,
                (ShapeKind::Rounded, Corners::Percent(_)) => Encoder::RoundedPercent,
            },
            Self::MaxLines(_) => Encoder::MaxLines,
            Self::Uri(_) => Encoder::Uri,
            Self::Variable(_) => Encoder::Variable,
        }
    }
}
pub const FONT_NAME_PATTERN: &str = r"`(?<fontName>[a-zA-Z0-9\s_-]+)`";
pub const VARIABLE_NAME_PATTERN: &str = "--[a-zA-Z0-9-]+";
pub const HEX6_PATTERN: &str = "#[a-fA-F0-9]{6}";
pub const HEX8_PATTERN: &str = "#[a-fA-F0-9]{8}";
pub const COLOR_NAMES: &[&str] = &[
    "primary",
    "onPrimary",
    "primaryContainer",
    "onPrimaryContainer",
    "inversePrimary",
    "secondary",
    "onSecondary",
    "secondaryContainer",
    "onSecondaryContainer",
    "tertiary",
    "onTertiary",
    "tertiaryContainer",
    "onTertiaryContainer",
    "background",
    "onBackground",
    "onSurface",
    "surfaceVariant",
    "onSurfaceVariant",
    "surfaceTint",
    "inverseSurface",
    "inverseOnSurface",
    "error",
    "onError",
    "errorContainer",
    "onErrorContainer",
    "outline",
    "outlineVariant",
    "scrim",
    "surfaceBright",
    "surfaceDim",
    "surfaceContainer",
    "surfaceContainerHigh",
    "surfaceContainerHighest",
    "surfaceContainerLow",
    "surfaceContainerLowest",
];
pub const CORNER_IDS: [&str; 4] = [
    "cornerSizeTopStart",
    "cornerSizeTopEnd",
    "cornerSizeBottomEnd",
    "cornerSizeBottomStart",
];
pub const PADDING_IDS: [&str; 4] = ["paddingStart", "paddingTop", "paddingEnd", "paddingBottom"];
pub const COLOR_RANGE: &str = "25[0-5]|2[0-4][0-9]|1[0-9][0-9]|[1-9]?[0-9]";
pub const ALPHA_RANGE: &str = r"1(?:[.]0)?|0(?:[.][0-9]*)?|[.][0-9]+";
impl Encoder {
    pub fn from_id(id: usize) -> Option<Self> {
        ENCODERS.get(id).copied()
    }
    pub fn id(self) -> usize {
        ENCODERS.iter().position(|&e| e == self).unwrap_or(0)
    }
    pub fn keywords(self) -> &'static [&'static str] {
        match self {
            Self::Inherit => &["inherit"],
            Self::Undefined => &["undefined"],
            Self::Yes => &["yes"],
            Self::No => &["no"],
            Self::GenericFont => &["system", "sans-serif", "serif", "monospace", "cursive"],
            Self::FontStyle => &["normal", "italic"],
            Self::FontWeight => &[
                "thin",
                "extra-light",
                "light",
                "normal",
                "medium",
                "semi-bold",
                "bold",
                "extra-bold",
                "black",
                "100",
                "200",
                "300",
                "400",
                "500",
                "600",
                "700",
                "800",
                "900",
            ],
            Self::TextAlign => &["left", "right", "center", "justify", "start", "end"],
            Self::TextDecoration => &["none", "underline", "line-through"],
            Self::TextOverflow => &["clip", "ellipsis", "visible"],
            Self::ContentScale => &[
                "crop",
                "fill-bounds",
                "fill-height",
                "fill-width",
                "fit",
                "inside",
                "none",
            ],
            _ => &[],
        }
    }
    pub fn specs(self) -> &'static [Spec] {
        static SPECS: std::sync::LazyLock<Vec<Vec<Spec>>> =
            std::sync::LazyLock::new(|| ENCODERS.iter().map(|e| e.make_specs()).collect());
        &SPECS[self.id()]
    }
    fn make_specs(self) -> Vec<Spec> {
        use Encoder::*;
        let atom = Spec::atom;
        let function = Spec::function;
        let list = Spec::list;
        let mut specs = vec![match self {
            Inherit | Undefined | Yes | No => atom(self.keywords()[0], self.keywords()[0], ""),
            Dp => atom("size", r"(?:0|[1-9][0-9]*)(?:[.][0-9]*)?|[.][0-9]+", "dp"),
            Sp => atom("size", r"[1-9][0-9]*(?:[.][0-9]*)?", "sp"),
            Percentage => Spec::percent("size", false),
            Color => color_spec(true),
            DynamicLight | DynamicDark => function(
                if self == DynamicLight {
                    "dynamic-light-color"
                } else {
                    "dynamic-dark-color"
                },
                list(",", &[atom("name", &COLOR_NAMES.join("|"), "")]),
            ),
            GenericFont => atom("genericFontFamily", &self.keywords().join("|"), ""),
            FontStyle | FontWeight | TextAlign | TextDecoration | TextOverflow | ContentScale => {
                atom("textAlign", &self.keywords().join("|"), "")
            }
            CustomFont => atom("enclosedFontName", FONT_NAME_PATTERN, ""),
            Padding => list(" ", &PADDING_IDS.map(|id| Spec::float(id, "dp"))),
            Rectangle | Circle => function(
                if self == Rectangle {
                    "rectangle"
                } else {
                    "circle"
                },
                Spec::nothing(),
            ),
            CutDp | CutPercent | RoundedDp | RoundedPercent => function(
                if matches!(self, CutDp | CutPercent) {
                    "cut-corner"
                } else {
                    "rounded-corner"
                },
                list(
                    ",",
                    &CORNER_IDS.map(|id| {
                        if matches!(self, CutPercent | RoundedPercent) {
                            Spec::percent(id, true)
                        } else {
                            Spec::float(id, "dp")
                        }
                    }),
                ),
            ),
            MaxLines => atom("textMaxLines", "none|[1-9][0-9]*", ""),
            Uri => function("uri", atom("enclosedUri", "`flex:/[^` ]+`", "")),
            Variable => function("var", atom("varKey", VARIABLE_NAME_PATTERN, "")),
        }];
        match self {
            Color => specs.extend([
                color_spec(false),
                atom("transparent", "transparent", ""),
                atom("hex", HEX6_PATTERN, ""),
                atom("hex", HEX8_PATTERN, ""),
            ]),
            Padding => specs.extend([
                list(
                    " ",
                    &[
                        Spec::float("paddingHorizontal", "dp"),
                        Spec::float("paddingVertical", "dp"),
                    ],
                ),
                list(" ", &[Spec::float("paddingAll", "dp")]),
            ]),
            _ => {}
        }
        specs
    }
    pub fn default_value(self) -> Value {
        use Encoder::*;
        match self {
            Inherit | Undefined | Yes | No => Value::Keyword(self),
            Dp => Value::Dimension(Unit::Dp, 0.0),
            Sp => Value::Dimension(Unit::Sp, 24.0),
            Percentage => Value::Dimension(Unit::Percentage, 0.0),
            Color => Value::Color([0.0, 0.0, 0.0, 1.0]),
            DynamicLight | DynamicDark => Value::DynamicColor {
                dark: self == DynamicDark,
                name: text("primary"),
            },
            GenericFont | FontStyle | TextAlign | TextDecoration | TextOverflow | ContentScale => {
                Value::Enum(
                    self,
                    text(self.keywords()[usize::from(self == ContentScale)]),
                )
            }
            FontWeight => Value::FontWeight(400),
            CustomFont => Value::Font(vec![]),
            Padding => Value::Padding([0.0; 4]),
            Rectangle | Circle | CutDp | RoundedDp | CutPercent | RoundedPercent => Value::Shape(
                match self {
                    Rectangle => ShapeKind::Rectangle,
                    Circle => ShapeKind::Circle,
                    CutDp | CutPercent => ShapeKind::Cut,
                    _ => ShapeKind::Rounded,
                },
                if matches!(self, CutPercent | RoundedPercent) {
                    Corners::Percent([0; 4])
                } else {
                    Corners::Dp([0.0; 4])
                },
            ),
            MaxLines => Value::MaxLines(i32::MAX),
            Uri => Value::Uri(vec![]),
            Variable => Value::Variable(vec![]),
        }
    }
    pub fn select(encoders: &[Self], input: &[u16]) -> Option<Value> {
        encoders.iter().find_map(|e| e.deserialize(input).ok())
    }
    pub fn validate_type(self, value: &Value) -> Result<(), Failure> {
        if self != value.encoder() {
            Err(Failure::argument("Failed requirement."))
        } else {
            Ok(())
        }
    }
}
fn color_spec(alpha: bool) -> Spec {
    let mut components = ["r", "g", "b"]
        .map(|id| Spec::atom(id, COLOR_RANGE, ""))
        .to_vec();
    if alpha {
        components.push(Spec::atom("a", ALPHA_RANGE, ""));
    }
    Spec::function(
        if alpha { "rgba" } else { "rgb" },
        Spec::list(",", &components),
    )
}
