//! Snygg transport only. A counted UTF-16 string packet preserves arbitrary backtick text
//! without JSON escaping, lossy UTF-8 conversion, Java callbacks or native object lifetimes.
use crate::jni::{self, illegal};
use jni_sys::{JNIEnv, jclass, jint, jobjectArray};
use snygg::{
    Error, Text,
    attributes::Attributes,
    document::{self, Configuration},
    grammar::{self, Rule},
    text,
};
use std::ptr;

pub(super) struct Packet<'a> {
    pub(super) rows: &'a [Text],
    pub(super) at: usize,
}
impl<'a> Packet<'a> {
    pub(super) fn next(&mut self) -> jni::Result<Text> {
        let value = self
            .rows
            .get(self.at)
            .ok_or_else(|| illegal("short Snygg packet"))?;
        self.at += 1;
        Ok(value.clone())
    }
    pub(super) fn number(&mut self) -> jni::Result<usize> {
        chars::text::to_int_or_null(&self.next()?)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| illegal("invalid Snygg packet count"))
    }
    pub(super) fn attributes(&mut self) -> jni::Result<Attributes> {
        let count = self.number()?;
        let mut attrs = Attributes::default();
        for _ in 0..count {
            let key = self.next()?;
            let count = self.number()?;
            let values = (0..count)
                .map(|_| self.next())
                .collect::<jni::Result<_>>()?;
            attrs.put(key, values);
        }
        Ok(attrs)
    }
    pub(super) fn rule(&mut self) -> jni::Result<Rule> {
        match self.number()? {
            0 => Ok(Rule::Defines),
            1 => Ok(Rule::Font(self.next()?)),
            2 => {
                let name = self.next()?;
                let selector = self.number()?;
                if selector >= grammar::SELECTORS.len() {
                    return Err(illegal("invalid Snygg selector"));
                }
                Ok(Rule::Element {
                    name,
                    selector,
                    attributes: self.attributes()?,
                })
            }
            _ => Err(illegal("invalid Snygg rule kind")),
        }
    }
}
fn count(n: usize) -> Text {
    text(&n.to_string())
}
fn attributes(a: Attributes) -> Vec<Text> {
    let mut out = vec![count(a.0.len())];
    for (key, values) in a.0 {
        out.push(key);
        out.push(count(values.len()));
        out.extend(values);
    }
    out
}
pub(super) fn rule(r: Rule) -> Vec<Text> {
    match r {
        Rule::Defines => vec![text("0")],
        Rule::Font(name) => vec![text("1"), name],
        Rule::Element {
            name,
            selector,
            attributes: a,
        } => {
            let mut out = vec![text("2"), name, count(selector)];
            out.extend(attributes(a));
            out
        }
    }
}
fn error(e: Error) -> Vec<Text> {
    let kind = match &e {
        Error::Number(_) => "number",
        Error::MissingSchema => "missing",
        Error::InvalidSchema(_) => "invalid",
        Error::UnsupportedSchema(_) => "unsupported",
        Error::InvalidRule(_) => "rule",
    };
    let value = match &e {
        Error::Number(v)
        | Error::InvalidSchema(v)
        | Error::UnsupportedSchema(v)
        | Error::InvalidRule(v) => v.clone(),
        Error::MissingSchema => Vec::new(),
    };
    vec![text("!"), text(kind), value, e.message()]
}
fn parsed(result: Result<Option<Rule>, Error>, filter: usize) -> Vec<Text> {
    match result {
        Ok(Some(r))
            if filter == 0
                || matches!(
                    (&r, filter),
                    (Rule::Defines, 1) | (Rule::Font(_), 2) | (Rule::Element { .. }, 3)
                ) =>
        {
            rule(r)
        }
        Err(e) => error(e),
        _ => vec![],
    }
}

fn dispatch(op: jint, rows: &[Text]) -> jni::Result<Vec<Text>> {
    let mut p = Packet { rows, at: 0 };
    Ok(match op {
        0 => {
            let filter = p.number()?;
            let input = p.next()?;
            if (filter == 1 && input != text("@defines"))
                || (filter == 2 && !input.starts_with(&text("@font `")))
            {
                vec![]
            } else {
                parsed(grammar::parse(&input), filter)
            }
        }
        1 => vec![p.rule()?.format()],
        2 => {
            let a = p.rule()?;
            vec![text(&a.compare(&p.rule()?).to_string())]
        }
        3 => match grammar::parse_attributes(&p.next()?) {
            Ok(a) => attributes(a),
            Err(e) => error(e),
        },
        4 => vec![p.attributes()?.format()],
        5 => {
            let a = p.attributes()?;
            vec![text(&a.compare(&p.attributes()?).to_string())]
        }
        6 => {
            let mode = p.number()?;
            let mut a = p.attributes()?;
            let n = p.number()?;
            let pairs = (0..n)
                .map(|_| Ok((p.next()?, p.next()?)))
                .collect::<jni::Result<Vec<_>>>()?;
            a.edit(&pairs, mode as u8);
            attributes(a)
        }
        7 => match grammar::selector(&p.next()?) {
            Some(s) => vec![count(s)],
            None => vec![],
        },
        8 => {
            let s = p.number()?;
            vec![text(
                grammar::SELECTORS
                    .get(s)
                    .ok_or_else(|| illegal("invalid Snygg selector"))?,
            )]
        }
        9 => vec![text(if grammar::valid_name(&p.next()?) {
            "1"
        } else {
            "0"
        })],
        10 => vec![
            text(grammar::DEFINES_PATTERN),
            text(grammar::FONT_PATTERN),
            text(grammar::ATTRIBUTE_PATTERN),
            text(&grammar::element_pattern()),
            text(document::SCHEMA_V2),
        ],
        11 => {
            let flags = p.number()?;
            let present = p.number()?;
            let schema = p.next()?;
            let config = Configuration {
                ignore_missing_schema: flags & 1 != 0,
                ignore_invalid_schema: flags & 2 != 0,
                ignore_unsupported_schema: flags & 4 != 0,
                ignore_invalid_rules: flags & 8 != 0,
            };
            match document::schema((present != 0).then_some(schema.as_slice()), config) {
                Err(e) => error(e),
                Ok(schema) => {
                    let entries = document::plan(&rows[p.at..], config);
                    let mut out = vec![schema, count(entries.len())];
                    for entry in entries {
                        let r = match entry.rule {
                            Ok(r) => rule(r),
                            Err(e) => error(e),
                        };
                        out.extend([count(entry.source), count(entry.slot), count(r.len())]);
                        out.extend(r);
                    }
                    out
                }
            }
        }
        12 => {
            let n = p.number()?;
            let rules = (0..n).map(|_| p.rule()).collect::<jni::Result<Vec<_>>>()?;
            document::serialized_keys(&rules)
        }
        13 => {
            let mut a = p.attributes()?;
            for (_, values) in &mut a.0 {
                let mut seen = std::collections::HashSet::new();
                values.retain(|v| seen.insert(v.clone()));
            }
            attributes(a)
        }
        14 => {
            let kind = p.number()?;
            let value = p.next()?;
            let e = match kind {
                0 => Error::MissingSchema,
                1 => Error::InvalidSchema(value),
                2 => Error::UnsupportedSchema(value),
                3 => Error::InvalidRule(value),
                _ => return Err(illegal("unknown Snygg error kind")),
            };
            vec![e.message()]
        }
        _ => return Err(illegal("unknown Snygg operation")),
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_jtn0123_audiowhisper_rust_SnyggGrammar_call(
    env: *mut JNIEnv,
    _: jclass,
    op: jint,
    input: jobjectArray,
) -> jobjectArray {
    // SAFETY: the JVM supplies this thread's environment and live array reference. All
    // strings are copied before grammar code runs; no Java references escape this call.
    unsafe {
        jni::run(env, ptr::null_mut(), |env| {
            let rows = env
                .read_strings(input, "input")?
                .into_iter()
                .map(|r| r.ok_or(jni::Error::NullPointer("Snygg packet string")))
                .collect::<jni::Result<Vec<_>>>()?;
            env.make_strings(&dispatch(op, &rows)?)
        })
    }
}
