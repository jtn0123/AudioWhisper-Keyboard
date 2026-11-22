//! UTF-16 and f32-bit transport only; all value decisions live in the Java-free crate.
use crate::jni::{self, illegal};
use snygg::{
    Text,
    properties::{self, PropertyError, PropertyInput},
    text,
    value::{Corners, Encoder, Value},
    value_spec::{Failure, Mapping},
};
struct Packet<'a> {
    rows: &'a [Text],
    at: usize,
}
impl Packet<'_> {
    fn next(&mut self) -> jni::Result<Text> {
        let v = self
            .rows
            .get(self.at)
            .ok_or_else(|| illegal("short Snygg value packet"))?
            .clone();
        self.at += 1;
        Ok(v)
    }
    fn int(&mut self) -> jni::Result<i32> {
        chars::text::to_int_or_null(&self.next()?).ok_or_else(|| illegal("invalid Snygg number"))
    }
    fn count(&mut self) -> jni::Result<usize> {
        usize::try_from(self.int()?).map_err(|_| illegal("invalid Snygg count"))
    }
    fn encoder(&mut self) -> jni::Result<Encoder> {
        Encoder::from_id(self.count()?).ok_or_else(|| illegal("invalid Snygg encoder"))
    }
    fn float(&mut self) -> jni::Result<f32> {
        Ok(f32::from_bits(self.int()? as u32))
    }
    fn floats(&mut self) -> jni::Result<[f32; 4]> {
        Ok([self.float()?, self.float()?, self.float()?, self.float()?])
    }
    fn mapping(&mut self) -> jni::Result<Mapping> {
        (0..self.count()?)
            .map(|_| Ok((self.next()?, self.next()?)))
            .collect()
    }
    fn value(&mut self) -> jni::Result<Value> {
        Ok(match self.encoder()?.default_value() {
            Value::Dimension(u, _) => Value::Dimension(u, self.float()?),
            Value::Color(_) => Value::Color(self.floats()?),
            Value::DynamicColor { dark, .. } => Value::DynamicColor {
                dark,
                name: self.next()?,
            },
            Value::Enum(e, _) => Value::Enum(e, self.next()?),
            Value::FontWeight(_) => Encoder::FontWeight
                .deserialize(&self.next()?)
                .map_err(|_| illegal("invalid Snygg weight"))?,
            Value::Font(_) => Value::Font(self.next()?),
            Value::Padding(_) => Value::Padding(self.floats()?),
            Value::Shape(k, Corners::Dp(_)) => Value::Shape(k, Corners::Dp(self.floats()?)),
            Value::Shape(k, Corners::Percent(_)) => Value::Shape(
                k,
                Corners::Percent([self.int()?, self.int()?, self.int()?, self.int()?]),
            ),
            Value::MaxLines(_) => Value::MaxLines(self.int()?),
            Value::Uri(_) => Value::Uri(self.next()?),
            Value::Variable(_) => Value::Variable(self.next()?),
            v @ Value::Keyword(_) => v,
        })
    }
}
fn num(n: usize) -> Text {
    text(&n.to_string())
}
fn float(n: f32) -> Text {
    text(&(n.to_bits() as i32).to_string())
}
fn value(v: Value) -> Vec<Text> {
    let mut out = vec![num(v.encoder().id())];
    match v {
        Value::Keyword(_) => {}
        Value::Dimension(_, n) => out.push(float(n)),
        Value::Color(a) | Value::Padding(a) | Value::Shape(_, Corners::Dp(a)) => {
            out.extend(a.map(float))
        }
        Value::Shape(_, Corners::Percent(a)) => out.extend(a.map(|n| text(&n.to_string()))),
        Value::DynamicColor { name, .. }
        | Value::Enum(_, name)
        | Value::Font(name)
        | Value::Uri(name)
        | Value::Variable(name) => out.push(name),
        Value::FontWeight(n) | Value::MaxLines(n) => out.push(text(&n.to_string())),
    }
    out
}
fn failure(e: Failure) -> Vec<Text> {
    let (kind, message) = match e {
        Failure::State(m) => ("state", m),
        Failure::Argument(m) => ("argument", m),
        Failure::Number(m) => (
            "number",
            [text("For input string: \""), m, text("\"")].concat(),
        ),
    };
    vec![text("!"), text(kind), message]
}
/// The answer to the operation `op` on `rows`, for the generated entry point.
pub fn dispatch(op: i32, rows: &[Text]) -> jni::Result<Vec<Text>> {
    let mut p = Packet { rows, at: 0 };
    Ok(match op {
        0 => {
            let e = p.encoder()?;
            let specs = e.specs();
            let mut out = vec![num(specs.len())];
            for s in specs {
                out.extend([
                    text(s.id.as_deref().unwrap_or("")),
                    text(&s.pattern),
                    text(&s.template),
                ]);
            }
            out.push(num(e.keywords().len()));
            out.extend(e.keywords().iter().map(|s| text(s)));
            out.extend(value(e.default_value()));
            out
        }
        1 => {
            let e = p.encoder()?;
            match e.deserialize(&p.next()?) {
                Ok(v) => value(v),
                Err(e) => failure(e),
            }
        }
        2 => {
            let e = p.encoder()?;
            let v = p.value()?;
            match e.serialize(&v) {
                Ok(s) => vec![s],
                Err(e) => failure(e),
            }
        }
        3 => value(p.encoder()?.default_value()),
        4 | 5 => {
            let e = p.encoder()?;
            let index = p.count()?;
            let specs = e.specs();
            let spec = specs
                .get(index)
                .ok_or_else(|| illegal("invalid Snygg spec"))?;
            if op == 4 {
                match spec.parse(&p.next()?) {
                    Ok(map) => {
                        let mut out = vec![num(map.len())];
                        for (k, v) in map {
                            out.extend([k, v]);
                        }
                        out
                    }
                    Err(e) => failure(e),
                }
            } else {
                match spec.pack(&p.mapping()?) {
                    Ok(s) => vec![s],
                    Err(e) => failure(e),
                }
            }
        }
        6 => {
            let flags = p.count()?;
            let required = (0..p.count()?)
                .map(|_| p.next())
                .collect::<jni::Result<Vec<_>>>()?;
            let entries = (0..p.count()?)
                .map(|_| {
                    let name = p.next()?;
                    let known = p.count()?;
                    let encoders = (0..p.count()?)
                        .map(|_| p.encoder())
                        .collect::<jni::Result<Vec<_>>>()?;
                    let valid = p.count()?;
                    let input = p.next()?;
                    Ok(PropertyInput {
                        name,
                        encoders: (known != 0).then_some(encoders),
                        input: if valid != 0 { Ok(input) } else { Err(()) },
                    })
                })
                .collect::<jni::Result<Vec<_>>>()?;
            match properties::decode(
                &entries,
                &required,
                &properties::Configuration {
                    ignore_properties: flags & 1 != 0,
                    ignore_values: flags & 2 != 0,
                },
            ) {
                Ok(values) => {
                    let mut out = vec![num(values.len())];
                    for (i, v) in values {
                        let v = value(v);
                        out.extend([num(i), num(v.len())]);
                        out.extend(v);
                    }
                    out
                }
                Err(e) => {
                    let (kind, detail) = match e {
                        PropertyError::InvalidProperty(i) => ("property", num(i)),
                        PropertyError::InvalidValue(i) => ("value", num(i)),
                        PropertyError::Json(i) => ("json", num(i)),
                        PropertyError::Missing(k) => ("missing", k),
                    };
                    vec![text("!"), text(kind), detail]
                }
            }
        }
        7 => {
            let names = (0..p.count()?)
                .map(|_| p.next())
                .collect::<jni::Result<Vec<_>>>()?;
            let required = &rows[p.at..];
            properties::missing(&names, required)
                .cloned()
                .into_iter()
                .collect()
        }
        8 => {
            let es = (0..p.count()?)
                .map(|_| p.encoder())
                .collect::<jni::Result<Vec<_>>>()?;
            Encoder::select(&es, &p.next()?)
                .map(value)
                .unwrap_or_default()
        }
        9 => [
            snygg::value::HEX6_PATTERN,
            snygg::value::HEX8_PATTERN,
            "transparent",
            snygg::value::COLOR_RANGE,
            snygg::value::ALPHA_RANGE,
            snygg::value::FONT_NAME_PATTERN,
            snygg::value::VARIABLE_NAME_PATTERN,
        ]
        .map(text)
        .to_vec(),
        _ => return Err(illegal("unknown Snygg value operation")),
    })
}
