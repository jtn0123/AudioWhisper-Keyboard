//! Value codecs preserve spec order, f32 arithmetic and failure classes.
use crate::{
    Text, text,
    value::{CORNER_IDS, Corners, Encoder, PADDING_IDS, ShapeKind, Unit, Value},
    value_spec::{self, Failure, Mapping, get},
};
fn no_spec(input: &[u16]) -> Failure {
    Failure::State(
        [
            text("No matching spec found for \""),
            input.to_vec(),
            text("\""),
        ]
        .concat(),
    )
}
fn nums(map: &Mapping, ids: [&str; 4]) -> Result<[f32; 4], Failure> {
    let mut values = [0.0; 4];
    for (dst, id) in values.iter_mut().zip(ids) {
        *dst = value_spec::float(get(map, id))?;
    }
    Ok(values)
}
impl Encoder {
    pub fn deserialize(self, input: &[u16]) -> Result<Value, Failure> {
        use Encoder::*;
        if self == Undefined {
            return Err(Failure::state("Undefined is not meant to be deserialized"));
        }
        if matches!(self, Inherit | Yes | No) {
            return if chars::trim(input) == text(self.keywords()[0]) {
                Ok(Value::Keyword(self))
            } else {
                Err(Failure::state("Check failed."))
            };
        }
        let specs = self.specs();
        let (index, map) = if matches!(self, Color | Padding) {
            specs
                .iter()
                .enumerate()
                .find_map(|(i, s)| s.parse(input).ok().map(|m| (i, m)))
                .ok_or_else(|| no_spec(input))?
        } else {
            (0, specs[0].parse(input)?)
        };
        Ok(match self {
            Dp | Sp | Percentage => Value::Dimension(
                match self {
                    Dp => Unit::Dp,
                    Sp => Unit::Sp,
                    _ => Unit::Percentage,
                },
                value_spec::float(get(&map, "size"))?
                    / if self == Percentage { 100.0 } else { 1.0 },
            ),
            Color => {
                let mut rgba = match index {
                    0 | 1 => {
                        let mut a = [0.0, 0.0, 0.0, 1.0];
                        for (n, id) in ["r", "g", "b"].iter().enumerate() {
                            a[n] = value_spec::integer(get(&map, id))? as f32 / 255.0;
                        }
                        if index == 0 {
                            a[3] = value_spec::float(get(&map, "a"))?;
                        }
                        a
                    }
                    2 => [0.0; 4],
                    _ => {
                        let hex = get(&map, "hex");
                        let mut a = [0.0, 0.0, 0.0, 1.0];
                        for (i, c) in a
                            .iter_mut()
                            .enumerate()
                            .take(if index == 3 { 3 } else { 4 })
                        {
                            let s = String::from_utf16_lossy(&hex[1 + 2 * i..3 + 2 * i]);
                            *c = u8::from_str_radix(&s, 16).map_err(|_| no_spec(input))? as f32
                                / 255.0;
                        }
                        a
                    }
                };
                // Compose's sRGB constructor packs each channel as (x * 255 + 0.5).toInt().
                for c in &mut rgba {
                    *c = ((*c * 255.0 + 0.5) as i32) as f32 / 255.0;
                }
                Value::Color(rgba)
            }
            DynamicLight | DynamicDark => Value::DynamicColor {
                dark: self == DynamicDark,
                name: get(&map, "name").to_vec(),
            },
            GenericFont | FontStyle | TextAlign | TextDecoration | TextOverflow | ContentScale => {
                Value::Enum(
                    self,
                    get(
                        &map,
                        if self == GenericFont {
                            "genericFontFamily"
                        } else {
                            "textAlign"
                        },
                    )
                    .to_vec(),
                )
            }
            FontWeight => {
                let key = get(&map, "textAlign");
                let index = self
                    .keywords()
                    .iter()
                    .position(|s| text(s) == key)
                    .ok_or_else(|| no_spec(input))?;
                Value::FontWeight(((index % 9 + 1) * 100) as i32)
            }
            CustomFont => {
                let name = get(&map, "enclosedFontName");
                Value::Font(name[1..name.len() - 1].to_vec())
            }
            Padding => Value::Padding(match index {
                0 => nums(&map, PADDING_IDS)?,
                1 => {
                    let h = value_spec::float(get(&map, "paddingHorizontal"))?;
                    let v = value_spec::float(get(&map, "paddingVertical"))?;
                    [h, v, h, v]
                }
                _ => [value_spec::float(get(&map, "paddingAll"))?; 4],
            }),
            Rectangle | Circle => Value::Shape(
                if self == Rectangle {
                    ShapeKind::Rectangle
                } else {
                    ShapeKind::Circle
                },
                Corners::Dp([0.0; 4]),
            ),
            CutDp | CutPercent | RoundedDp | RoundedPercent => {
                let corners = if self == RoundedDp {
                    Corners::Dp(nums(&map, CORNER_IDS)?)
                } else {
                    let mut ints = [0; 4];
                    for (dst, id) in ints.iter_mut().zip(CORNER_IDS) {
                        *dst = value_spec::integer(get(&map, id))?;
                    }
                    if self == CutDp {
                        Corners::Dp(ints.map(|i| i as f32))
                    } else {
                        Corners::Percent(ints)
                    }
                };
                Value::Shape(
                    if matches!(self, CutDp | CutPercent) {
                        ShapeKind::Cut
                    } else {
                        ShapeKind::Rounded
                    },
                    corners,
                )
            }
            MaxLines => {
                let n = get(&map, "textMaxLines");
                Value::MaxLines(if n == text("none") {
                    i32::MAX
                } else {
                    value_spec::integer(n)?
                })
            }
            Uri => {
                let enclosed = get(&map, "enclosedUri");
                let uri = &enclosed[1..enclosed.len() - 1];
                crate::value_uri::validate(uri)?;
                Value::Uri(uri.to_vec())
            }
            Variable => Value::Variable(get(&map, "varKey").to_vec()),
            Inherit | Undefined | Yes | No => {
                return Err(Failure::state("invalid keyword encoder"));
            }
        })
    }
    pub fn serialize(self, value: &Value) -> Result<Text, Failure> {
        use Encoder::*;
        if self == Undefined {
            return Err(Failure::state("Undefined is not meant to be serialized"));
        }
        if matches!(self, Inherit | Yes | No) {
            return Ok(text(self.keywords()[0]));
        }
        self.validate_type(value)?;
        let mut map = vec![];
        let mut add = |key: &str, value: Text| map.push((text(key), value));
        match value {
            Value::Dimension(unit, n) => add(
                "size",
                value_spec::number(
                    *n * if *unit == Unit::Percentage {
                        100.0
                    } else {
                        1.0
                    },
                ),
            ),
            Value::Color(a) => {
                for (id, n) in ["r", "g", "b"].into_iter().zip(a) {
                    if n.is_nan() {
                        return Err(Failure::argument("Cannot round NaN value."));
                    }
                    add(id, text(&((n * 255.0 + 0.5).floor() as i32).to_string()));
                }
                add("a", value_spec::number(a[3]));
            }
            Value::DynamicColor { name, .. } => add("name", name.clone()),
            Value::Enum(_, key) => add(
                if self == GenericFont {
                    "genericFontFamily"
                } else {
                    "textAlign"
                },
                key.clone(),
            ),
            Value::FontWeight(n) => {
                let key = if *n >= 100 && *n <= 900 && n % 100 == 0 {
                    text(self.keywords()[(*n / 100 - 1) as usize])
                } else {
                    return Err(Failure::Argument(text("No key found for given value")));
                };
                add("textAlign", key);
            }
            Value::Font(name) => add(
                "enclosedFontName",
                [text("`"), name.clone(), text("`")].concat(),
            ),
            Value::Padding(a) => {
                for (id, n) in PADDING_IDS.into_iter().zip(a) {
                    add(id, value_spec::number(*n));
                }
            }
            Value::Shape(_, corners) if !matches!(self, Rectangle | Circle) => match corners {
                Corners::Dp(a) => {
                    for (id, n) in CORNER_IDS.into_iter().zip(a) {
                        add(id, value_spec::number(*n));
                    }
                }
                Corners::Percent(a) => {
                    for (id, n) in CORNER_IDS.into_iter().zip(a) {
                        add(id, text(&n.to_string()));
                    }
                }
            },
            Value::MaxLines(n) => {
                if *n < 1 {
                    return Err(Failure::argument("Failed requirement."));
                }
                add(
                    "textMaxLines",
                    text(&if *n == i32::MAX {
                        "none".into()
                    } else {
                        n.to_string()
                    }),
                );
            }
            Value::Uri(uri) => add("enclosedUri", [text("`"), uri.clone(), text("`")].concat()),
            Value::Variable(key) => add("varKey", key.clone()),
            _ => {}
        }
        self.specs()[0].pack(&map)
    }
}
