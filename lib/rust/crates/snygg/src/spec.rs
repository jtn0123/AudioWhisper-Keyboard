//! Built-in property schema, also consumed by the public Kotlin DSL adapter.
use crate::value::Encoder;
#[derive(Clone, Debug)]
pub struct Property {
    pub name: &'static str,
    pub encoders: Vec<Encoder>,
    pub inherits: bool,
    pub required: bool,
}
fn property(name: &'static str, encoders: &[Encoder], inherits: bool) -> Property {
    Property {
        name,
        encoders: encoders.to_vec(),
        inherits,
        required: false,
    }
}
pub fn elements() -> &'static [Property] {
    static PROPERTIES: std::sync::OnceLock<Vec<Property>> = std::sync::OnceLock::new();
    PROPERTIES.get_or_init(|| {
        use Encoder::*;
        vec![
            property("background", &[Color, DynamicLight, DynamicDark], false),
            property("foreground", &[Color, DynamicLight, DynamicDark], true),
            property("background-image", &[Uri], false),
            property("content-scale", &[ContentScale], false),
            property("border-color", &[Color, DynamicLight, DynamicDark], false),
            property("border-style", &[], false),
            property("border-width", &[Dp], false),
            property("font-family", &[GenericFont, CustomFont], true),
            property("font-size", &[Sp], true),
            property("font-style", &[FontStyle], true),
            property("font-weight", &[FontWeight], true),
            property("letter-spacing", &[Sp], true),
            property("line-height", &[Sp], true),
            property("margin", &[Padding], false),
            property("padding", &[Padding], false),
            property("shadow-color", &[Color, DynamicLight, DynamicDark], false),
            property("shadow-elevation", &[Dp], false),
            property(
                "shape",
                &[
                    Rectangle,
                    Circle,
                    RoundedDp,
                    RoundedPercent,
                    CutDp,
                    CutPercent,
                ],
                false,
            ),
            property("clip", &[Yes, No], false),
            property("text-align", &[TextAlign], true),
            property("text-decoration-line", &[TextDecoration], true),
            property("text-max-lines", &[MaxLines], true),
            property("text-overflow", &[TextOverflow], true),
        ]
    })
}
pub fn fonts() -> &'static [Property] {
    static PROPERTIES: std::sync::OnceLock<Vec<Property>> = std::sync::OnceLock::new();
    PROPERTIES.get_or_init(|| {
        use Encoder::*;
        let mut src = property("src", &[Uri], false);
        src.required = true;
        vec![
            src,
            property("font-style", &[FontStyle], false),
            property("font-weight", &[FontWeight], false),
        ]
    })
}
pub fn encoders(rule: &crate::grammar::Rule, name: &[u16]) -> Option<Vec<Encoder>> {
    use crate::{grammar::Rule, text};
    match rule {
        Rule::Defines => (name.starts_with(&text("--")) && crate::grammar::valid_name(&name[2..]))
            .then(all_encoders),
        Rule::Font(_) => fonts()
            .iter()
            .find(|p| text(p.name) == name)
            .map(|p| p.encoders.clone()),
        Rule::Element { .. } => elements().iter().find(|p| text(p.name) == name).map(|p| {
            [
                vec![Encoder::Inherit, Encoder::Variable],
                p.encoders.clone(),
            ]
            .concat()
        }),
    }
}

/// DSL declaration insertion order, including the implicit encoders added at build time.
pub fn all_encoders() -> Vec<Encoder> {
    let mut out = vec![];
    for p in fonts().iter().chain(elements()) {
        for &encoder in &p.encoders {
            if !out.contains(&encoder) {
                out.push(encoder);
            }
        }
    }
    out.extend([Encoder::Inherit, Encoder::Variable]);
    out
}

/// Cached keys avoid rebuilding specs or allocating absent inherited properties per query.
pub fn inherited() -> &'static [crate::Text] {
    static KEYS: std::sync::OnceLock<Vec<crate::Text>> = std::sync::OnceLock::new();
    KEYS.get_or_init(|| {
        elements()
            .iter()
            .filter(|p| p.inherits)
            .map(|p| crate::text(p.name))
            .collect()
    })
}
