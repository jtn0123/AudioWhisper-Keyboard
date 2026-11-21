//! Object-reference/UTF-16 transport. Opaque tokens preserve arbitrary Compose values
//! (including non-sRGB colours and unspecified TextUnits) without serialization loss.
use crate::{
    jni::{self, illegal},
    snygg::{Packet, rule},
};
use snygg::{
    Text,
    editor::{self, RuleEdit},
    external::ExternalValue as ManagedValue,
    grammar::Rule,
    resolve::{self, CompiledTheme, Properties, PropertySets, Query, Stylesheet},
    text,
};

fn num(n: usize) -> Text {
    text(&n.to_string())
}
fn float(p: &mut Packet<'_>) -> jni::Result<f32> {
    let bits = chars::text::to_int_or_null(&p.next()?)
        .ok_or_else(|| illegal("invalid Snygg float bits"))?;
    Ok(f32::from_bits(bits as u32))
}
fn read_value(p: &mut Packet<'_>) -> jni::Result<ManagedValue> {
    Ok(match p.number()? {
        0 => ManagedValue::Opaque(p.next()?),
        1 => ManagedValue::Undefined,
        2 => ManagedValue::Inherit,
        3 => ManagedValue::Variable(p.next()?),
        4 => ManagedValue::Sp(float(p)?, false),
        5 => ManagedValue::Dynamic(false, p.next()?),
        6 => ManagedValue::Dynamic(true, p.next()?),
        7 => ManagedValue::Sp(float(p)?, true),
        _ => return Err(illegal("invalid Snygg managed value")),
    })
}
fn read_properties(p: &mut Packet<'_>) -> jni::Result<Properties<ManagedValue>> {
    (0..p.number()?)
        .map(|_| Ok((p.next()?, read_value(p)?)))
        .collect()
}
fn read_style(p: &mut Packet<'_>) -> jni::Result<Vec<(Rule, Properties<ManagedValue>)>> {
    (0..p.number()?)
        .map(|_| Ok((p.rule()?, read_properties(p)?)))
        .collect()
}
fn write_properties(out: &mut Vec<Text>, props: Properties<ManagedValue>) {
    out.push(num(props.len()));
    for (k, v) in props {
        out.push(k);
        match v {
            ManagedValue::Opaque(i) => out.extend([text("0"), i]),
            ManagedValue::Undefined => out.push(text("1")),
            ManagedValue::Inherit => out.push(text("2")),
            ManagedValue::Variable(k) => out.extend([text("3"), k]),
            ManagedValue::Sp(n, em) => out.extend([
                text(if em { "7" } else { "4" }),
                text(&(n.to_bits() as i32).to_string()),
            ]),
            ManagedValue::Dynamic(d, k) => out.extend([text(if d { "6" } else { "5" }), k]),
        }
    }
}
/// The answer to the operation `op` on `rows`, for the generated entry point.
pub fn dispatch(op: i32, rows: &[Text]) -> jni::Result<Vec<Text>> {
    let mut p = Packet { rows, at: 0 };
    let mut out = vec![];
    match op {
        0 => {
            let rules = (0..p.number()?)
                .map(|_| {
                    let rule = p.rule()?;
                    let sets = match p.number()? {
                        0 => PropertySets::Single(read_properties(&mut p)?),
                        1 => PropertySets::Multiple(
                            (0..p.number()?)
                                .map(|_| read_properties(&mut p))
                                .collect::<jni::Result<_>>()?,
                        ),
                        _ => return Err(illegal("invalid Snygg property sets")),
                    };
                    Ok((rule, sets))
                })
                .collect::<jni::Result<_>>()?;
            let compiled = match CompiledTheme::compile(&Stylesheet { rules }) {
                Ok(c) => c,
                Err(message) => return Ok(vec![text("!"), text(message)]),
            };
            out.push(num(compiled.style.len()));
            for (r, props) in compiled.style {
                out.extend(rule(r));
                write_properties(&mut out, props);
            }
            out.push(num(compiled.fonts.len()));
            for (name, sets) in compiled.fonts {
                out.extend([name, num(sets.len())]);
                for props in sets {
                    write_properties(&mut out, props);
                }
            }
        }
        1 => {
            let style = read_style(&mut p)?;
            let element = p.next()?;
            let selector = p.number()?;
            let attrs = (0..p.number()?)
                .map(|_| Ok((p.next()?, p.next()?)))
                .collect::<jni::Result<Vec<_>>>()?;
            let parent = read_properties(&mut p)?;
            let multiplier = float(&mut p)?;
            let light = read_properties(&mut p)?;
            let dark = read_properties(&mut p)?;
            let theme = CompiledTheme {
                style,
                fonts: vec![],
            };
            let props = theme.resolve_with_palette(
                &Query {
                    element: &element,
                    attributes: &attrs,
                    selector,
                },
                &parent,
                multiplier,
                &light,
                &dark,
            );
            write_properties(&mut out, props);
        }
        2 | 3 => {
            let mut current = read_properties(&mut p)?;
            if op == 2 {
                let style = read_properties(&mut p)?;
                let parent = read_properties(&mut p)?;
                resolve::apply(&mut current, &style, &parent, float(&mut p)?);
            } else {
                resolve::inherit(&mut current, &read_properties(&mut p)?);
            }
            write_properties(&mut out, current);
        }
        4 => {
            let properties = if p.number()? == 0 {
                snygg::spec::elements()
            } else {
                snygg::spec::fonts()
            };
            out.push(num(properties.len()));
            for property in properties {
                out.extend([
                    text(property.name),
                    num(usize::from(property.inherits)),
                    num(usize::from(property.required)),
                    num(property.encoders.len()),
                ]);
                out.extend(property.encoders.iter().map(|e| num(e.id())));
            }
        }
        5 => {
            let rules = (0..p.number()?)
                .map(|_| p.rule())
                .collect::<jni::Result<Vec<_>>>()?;
            let old = p.rule()?;
            let new = p.rule()?;
            let allow_add = p.number()? != 0;
            out = match editor::rule_edit(&rules, &old, &new, allow_add) {
                RuleEdit::Unchanged => vec![text("unchanged")],
                RuleEdit::Duplicate => vec![text("duplicate")],
                RuleEdit::Missing => vec![text("missing")],
                RuleEdit::Add => vec![text("add")],
                RuleEdit::Replace(i) => vec![text("replace"), num(i)],
            };
        }
        6 => {
            let keys = (0..p.number()?)
                .map(|_| p.next())
                .collect::<jni::Result<Vec<_>>>()?;
            let slot = editor::property_slot(&keys, &p.next()?);
            out.push(slot.map(num).unwrap_or_else(|| text("-1")));
        }
        7 => {
            let a = p.rule()?;
            let b = p.rule()?;
            let order = (0..p.number()?)
                .map(|_| p.next())
                .collect::<jni::Result<Vec<_>>>()?;
            out.push(text(&editor::compare_order(&a, &b, &order).to_string()));
        }
        _ => return Err(illegal("unknown Snygg compiler operation")),
    }
    Ok(out)
}
