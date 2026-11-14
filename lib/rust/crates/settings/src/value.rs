//! Ordered copied data. Strings remain UTF-16, including unpaired surrogates.
use crate::{Result, Text, text};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,
    String(Text),
    Number(i64),
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
}
impl Value {
    pub fn string(s: &str) -> Self {
        Self::String(text(s))
    }
    pub fn get(&self, name: &str) -> Result<&Self> {
        if let Self::Object(fields) = self {
            fields
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v)
                .ok_or("missing field")
        } else {
            Err("expected object")
        }
    }
    pub fn set(&mut self, name: &str, value: Self) -> Result<()> {
        if let Self::Object(fields) = self {
            if let Some((_, v)) = fields.iter_mut().find(|(k, _)| k == name) {
                *v = value;
            } else {
                fields.push((name.to_owned(), value));
            }
            Ok(())
        } else {
            Err("expected object")
        }
    }
    pub fn string_value(&self) -> Result<&[u16]> {
        if let Self::String(s) = self {
            Ok(s)
        } else {
            Err("expected string")
        }
    }
    pub fn number(&self) -> Result<i64> {
        if let Self::Number(n) = self {
            Ok(*n)
        } else {
            Err("expected integer")
        }
    }
    pub fn list(&self) -> Result<&[Self]> {
        if let Self::List(v) = self {
            Ok(v)
        } else {
            Err("expected list")
        }
    }
    pub fn pack(&self, output: &mut Vec<Text>) {
        match self {
            Self::Null => output.push(text("z")),
            Self::String(s) => {
                output.push(text("s"));
                output.push(s.clone());
            }
            Self::Number(n) => {
                output.push(text("n"));
                output.push(text(&n.to_string()));
            }
            Self::List(items) => {
                output.push(text("a"));
                output.push(text(&items.len().to_string()));
                for item in items {
                    item.pack(output);
                }
            }
            Self::Object(fields) => {
                output.push(text("o"));
                output.push(text(&fields.len().to_string()));
                for (key, value) in fields {
                    output.push(text(key));
                    value.pack(output);
                }
            }
        }
    }
    pub fn unpack(input: &mut std::slice::Iter<'_, Text>, depth: usize) -> Result<Self> {
        if depth > 256 {
            return Err("packet depth");
        }
        let tag = input.next().ok_or("missing tag")?;
        let mut next = || input.next().ok_or("missing value");
        Ok(match String::from_utf16_lossy(tag).as_str() {
            "z" => Self::Null,
            "s" => Self::String(next()?.clone()),
            "n" => Self::Number(crate::integer(next()?)?),
            "a" | "o" => {
                let count =
                    usize::try_from(crate::integer(next()?)?).map_err(|_| "invalid size")?;
                if count > input.len() {
                    return Err("invalid size");
                }
                if tag == &text("a") {
                    Self::List(
                        (0..count)
                            .map(|_| Self::unpack(input, depth + 1))
                            .collect::<Result<_>>()?,
                    )
                } else {
                    Self::Object(
                        (0..count)
                            .map(|_| {
                                let key = String::from_utf16(input.next().ok_or("missing key")?)
                                    .map_err(|_| "invalid key")?;
                                Ok((key, Self::unpack(input, depth + 1)?))
                            })
                            .collect::<Result<_>>()?,
                    )
                }
            }
            _ => return Err("invalid tag"),
        })
    }
    pub fn json(&self) -> Text {
        let mut out = Vec::new();
        self.write(&mut out);
        out
    }
    fn write(&self, out: &mut Text) {
        match self {
            Self::Null => out.extend(text("null")),
            Self::String(s) => quote(s, out),
            Self::Number(n) => out.extend(text(&n.to_string())),
            Self::List(items) => {
                out.push(91);
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push(44);
                    }
                    v.write(out);
                }
                out.push(93);
            }
            Self::Object(fields) => {
                out.push(123);
                for (i, (k, v)) in fields.iter().enumerate() {
                    if i > 0 {
                        out.push(44);
                    }
                    quote(&text(k), out);
                    out.push(58);
                    v.write(out);
                }
                out.push(125);
            }
        }
    }
}
pub fn object(fields: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Object(fields.into_iter().map(|(k, v)| (k.to_owned(), v)).collect())
}
fn quote(input: &[u16], out: &mut Text) {
    out.push(34);
    for &u in input {
        match u {
            8 => out.extend(text("\\b")),
            9 => out.extend(text("\\t")),
            10 => out.extend(text("\\n")),
            12 => out.extend(text("\\f")),
            13 => out.extend(text("\\r")),
            34 | 92 => {
                out.push(92);
                out.push(u);
            }
            0..=31 => out.extend(text(&format!("\\u{u:04x}"))),
            _ => out.push(u),
        }
    }
    out.push(34);
}
