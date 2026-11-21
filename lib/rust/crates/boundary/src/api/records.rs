//! Typed results the bindings used to unpack by hand: a record read field by field from the row a
//! native method returns, and a value class whose properties read bits of one number.

use super::{Api, Type};
use crate::Return;
use crate::format::kdoc;
use crate::types::Element;

/// One property of a [`Type::Bits`] value class.
#[derive(Clone, Copy, Debug)]
pub struct Bit {
    pub name: &'static str,
    pub doc: &'static str,
    pub kind: BitKind,
    /// The lowest bit it is read from.
    pub shift: u32,
    /// Whether only its own module reads it.
    pub internal: bool,
}

/// A property's defaults, for `..BIT`.
pub const BIT: Bit = Bit {
    name: "",
    doc: "",
    kind: BitKind::Flag,
    shift: 0,
    internal: false,
};

/// How a [`Bit`] reads its bits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitKind {
    /// A `Boolean`: whether its bit is set.
    Flag,
    /// An unsigned `Int` this many bits wide.
    Int(u32),
    /// A signed `Int`: the 32 bits from its shift, a `Long`'s high half.
    Signed,
    /// A `Boolean?` in two bits: 1 is true, 2 false, and anything else null.
    Maybe,
}

/// A value class over an `Int`, or a `Long` if `long`, at `indent`.
pub fn bits_text(ty: &Type, indent: &str) -> String {
    let Type::Bits {
        name,
        doc,
        long,
        internal,
        fields,
    } = ty
    else {
        unreachable!("only bits are laid out as bits")
    };
    let number = if *long { "Long" } else { "Int" };
    let visibility = if *internal {
        " internal constructor"
    } else {
        ""
    };
    let mut text = kdoc(doc, indent);
    text.push_str(&format!("{indent}@JvmInline\n"));
    text.push_str(&format!(
        "{indent}value class {name}{visibility}(private val bits: {number}) {{\n"
    ));
    let inner = format!("{indent}    ");
    for bit in *fields {
        if !bit.doc.is_empty() {
            text.push_str(&kdoc(bit.doc, &inner));
        }
        let modifier = if bit.internal { "internal " } else { "" };
        text.push_str(&format!("{inner}{modifier}val {}", bit.name));
        text.push_str(&getter(bit, *long, &inner));
    }
    text.push_str(&format!("{indent}}}\n"));
    text
}

/// `: Type get() = …` for `bit`, ending its line or lines.
fn getter(bit: &Bit, long: bool, indent: &str) -> String {
    let (one, zero) = if long { ("1L", "0L") } else { ("1", "0") };
    let shifted = if bit.shift == 0 {
        "bits".to_owned()
    } else {
        format!("(bits ushr {})", bit.shift)
    };
    let int = |mask: u64| {
        if long {
            format!("({shifted} and 0x{mask:X}L).toInt()")
        } else {
            format!("{shifted} and 0x{mask:X}")
        }
    };
    match bit.kind {
        BitKind::Flag => format!(": Boolean get() = {shifted} and {one} != {zero}\n"),
        BitKind::Int(width) => format!(": Int get() = {}\n", int((1 << width) - 1)),
        BitKind::Signed => {
            assert!(long, "{}: only a Long has a high half", bit.name);
            format!(": Int get() = (bits shr {}).toInt()\n", bit.shift)
        }
        BitKind::Maybe => format!(
            ": Boolean?\n{indent}    get() = when ({}) {{\n{indent}        1 -> true\n\
             {indent}        2 -> false\n{indent}        else -> null\n{indent}    }}\n",
            int(3)
        ),
    }
}

/// The fields of the record `name`: one nested in `api`, else in another api, which a dotted
/// name such as `Suggester.Strip` names.
fn record_fields<'a>(name: &str, api: &'a Api, apis: &'a [Api]) -> &'a [(&'a str, &'a str)] {
    let (owner, record) = name.rsplit_once('.').unwrap_or(("", name));
    let owners = apis.iter().filter(|it| it.name == owner);
    let mut nested = std::iter::once(api).chain(owners).flat_map(|it| it.nested);
    let found = nested.find_map(|ty| match ty {
        Type::Record {
            name: n, fields, ..
        } if *n == record => Some(*fields),
        _ => None,
    });
    found.unwrap_or_else(|| panic!("{}: no record {name}", api.name))
}

/// Whether `ty` is an enum nested in some api.
fn is_enum(ty: &str, apis: &[Api]) -> bool {
    let name = ty.rsplit('.').next().unwrap_or(ty);
    apis.iter()
        .flat_map(|api| api.nested)
        .any(|nested| matches!(nested, Type::Enum { name: n, .. } if *n == name))
}

/// How a field of type `ty` reads position `at` of a row of `element`s, `None` for strings.
fn field(ty: &str, at: usize, element: Option<Element>, apis: &[Api]) -> String {
    let value = format!("it[{at}]");
    let int = match element {
        None => format!("{value}.toInt()"),
        Some(_) => value.clone(),
    };
    if is_enum(ty, apis) {
        return format!("{ty}.entries[{int}]");
    }
    match (element, ty) {
        (None, "String") => value,
        (None, "Boolean") => format!("WireFlag.decode({value})"),
        (None, "Char") => format!("{value}[0]"),
        (None, "Int") => int,
        (None, "Long" | "Float" | "Double") => format!("{value}.to{ty}()"),
        (Some(Element::Int), "Int") | (Some(Element::Long), "Long") => value,
        (Some(Element::Float), "Float") => value,
        (Some(Element::Int), "Boolean") => format!("{value} != 0"),
        (Some(Element::Long), "Boolean") => format!("{value} != 0L"),
        (Some(Element::Int), "Char") => format!("{value}.toChar()"),
        (Some(Element::Int), "Long") => format!("{value}.toLong()"),
        (Some(Element::Long), "Int") => format!("{value}.toInt()"),
        (element, ty) => panic!("no {ty} field in a row of {element:?}"),
    }
}

/// What follows a native call that returns `returns` to read it as the record `name`, or a list
/// of them if `many`, each in as many elements as it has fields. A record's last field may be a
/// `List<String>` of the rest of the row.
pub fn record(name: &str, many: bool, returns: Return, api: &Api, apis: &[Api]) -> String {
    let (element, nullable) = match returns {
        Return::Strings => (None, false),
        Return::Array(element) => (Some(element), false),
        Return::NullableArray(element) => (Some(element), true),
        other => panic!("{}: no record in {other:?}", api.name),
    };
    let fields = record_fields(name, api, apis);
    let mut values = Vec::with_capacity(fields.len());
    for (at, (field_name, ty)) in fields.iter().enumerate() {
        if *ty == "List<String>" {
            assert!(
                element.is_none() && !many,
                "{field_name}: the rest is a string row's"
            );
            assert_eq!(at + 1, fields.len(), "{field_name}: the rest comes last");
            values.push(format!("it.asList().subList({at}, it.size)"));
        } else {
            values.push(field(ty, at, element, apis));
        }
    }
    let made = format!("{name}({})", values.join(", "));
    if many {
        format!(".asList().chunked({}) {{ {made} }}", fields.len())
    } else if nullable {
        format!("?.let {{ {made} }}")
    } else {
        format!(".let {{ {made} }}")
    }
}

/// The type a member reading the record `name` returns.
pub fn record_type(name: &str, many: bool, returns: Return) -> String {
    if many {
        format!("List<{name}>")
    } else if matches!(returns, Return::NullableArray(_)) {
        format!("{name}?")
    } else {
        name.to_owned()
    }
}
